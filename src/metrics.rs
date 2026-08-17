use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::lif::{InputPolarity, NeuronId, SimDuration, SimTime};
use crate::network::{NetworkDefinition, NetworkRun};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MetricsConfig {
    pub window_start: SimTime,
    pub window_end: SimTime,
    pub bin_width: SimDuration,
    pub synchronous_fraction_threshold: f64,
    pub maximum_period_lag: SimDuration,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NetworkMetrics {
    pub duration_ms: f64,
    pub total_spikes: usize,
    pub mean_firing_rate_hz: f64,
    pub active_neuron_fraction: f64,
    pub excitatory_arrival_count: usize,
    pub inhibitory_arrival_count: usize,
    pub ignored_arrival_count: usize,
    pub excitatory_arrival_magnitude_mv: f64,
    pub inhibitory_arrival_magnitude_mv: f64,
    pub synchronous_burst_count: usize,
    pub synchronous_burst_bin_fraction: f64,
    pub maximum_silence_ms: f64,
    pub mean_silence_ms: f64,
    pub peak_autocorrelation: f64,
    pub peak_autocorrelation_lag_ms: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrajectoryDifference {
    pub bin_count: usize,
    pub mean_rms_spike_difference: f64,
    pub maximum_rms_spike_difference: f64,
    pub changed_bin_fraction: f64,
    pub last_changed_time_ms: Option<f64>,
}

pub fn compute_network_metrics(
    definition: &NetworkDefinition,
    run: &NetworkRun,
    config: MetricsConfig,
) -> Result<NetworkMetrics, MetricsError> {
    validate_window(
        run,
        config.window_start,
        config.window_end,
        config.bin_width,
    )?;
    if !config.synchronous_fraction_threshold.is_finite()
        || !(0.0..=1.0).contains(&config.synchronous_fraction_threshold)
    {
        return Err(MetricsError::InvalidSynchronousFraction(
            config.synchronous_fraction_threshold,
        ));
    }

    let neuron_count = definition.neurons().len();
    let duration_micros = config.window_end.as_micros() - config.window_start.as_micros();
    let duration_seconds = duration_micros as f64 / 1_000_000.0;
    let duration_ms = duration_micros as f64 / 1_000.0;
    let bin_count = duration_micros.div_ceil(config.bin_width.as_micros()) as usize;
    let mut spike_counts = vec![0_u64; bin_count];
    let mut active_by_bin = vec![BTreeSet::<NeuronId>::new(); bin_count];
    let mut active_neurons = BTreeSet::new();
    let mut spike_times = Vec::new();

    for spike in run
        .event_log
        .spikes()
        .filter(|spike| spike.time >= config.window_start && spike.time < config.window_end)
    {
        let bin = ((spike.time.as_micros() - config.window_start.as_micros())
            / config.bin_width.as_micros()) as usize;
        spike_counts[bin] += 1;
        active_by_bin[bin].insert(spike.neuron_id);
        active_neurons.insert(spike.neuron_id);
        spike_times.push(spike.time.as_micros());
    }

    let mut excitatory_arrival_count = 0;
    let mut inhibitory_arrival_count = 0;
    let mut ignored_arrival_count = 0;
    let mut excitatory_arrival_magnitude_mv = 0.0;
    let mut inhibitory_arrival_magnitude_mv = 0.0;
    for input in run
        .event_log
        .inputs()
        .filter(|input| input.time >= config.window_start && input.time < config.window_end)
    {
        if input.ignored_during_refractory {
            ignored_arrival_count += 1;
        }
        match input.polarity {
            InputPolarity::Excitatory => {
                excitatory_arrival_count += 1;
                excitatory_arrival_magnitude_mv += input.magnitude_mv;
            }
            InputPolarity::Inhibitory => {
                inhibitory_arrival_count += 1;
                inhibitory_arrival_magnitude_mv += input.magnitude_mv;
            }
        }
    }

    let synchronous = active_by_bin
        .iter()
        .map(|neurons| neurons.len() as f64 / neuron_count as f64)
        .map(|fraction| fraction >= config.synchronous_fraction_threshold)
        .collect::<Vec<_>>();
    let synchronous_bin_count = synchronous.iter().filter(|is_sync| **is_sync).count();
    let synchronous_burst_count = synchronous
        .iter()
        .enumerate()
        .filter(|(index, is_sync)| **is_sync && (*index == 0 || !synchronous[*index - 1]))
        .count();

    let (maximum_silence_ms, mean_silence_ms) = silence_metrics(
        &spike_times,
        config.window_start.as_micros(),
        config.window_end.as_micros(),
    );
    let maximum_lag_bins =
        (config.maximum_period_lag.as_micros() / config.bin_width.as_micros()) as usize;
    let (peak_autocorrelation, peak_lag_bins) =
        peak_autocorrelation(&spike_counts, maximum_lag_bins);

    Ok(NetworkMetrics {
        duration_ms,
        total_spikes: spike_times.len(),
        mean_firing_rate_hz: spike_times.len() as f64 / neuron_count as f64 / duration_seconds,
        active_neuron_fraction: active_neurons.len() as f64 / neuron_count as f64,
        excitatory_arrival_count,
        inhibitory_arrival_count,
        ignored_arrival_count,
        excitatory_arrival_magnitude_mv,
        inhibitory_arrival_magnitude_mv,
        synchronous_burst_count,
        synchronous_burst_bin_fraction: synchronous_bin_count as f64 / bin_count as f64,
        maximum_silence_ms,
        mean_silence_ms,
        peak_autocorrelation,
        peak_autocorrelation_lag_ms: peak_lag_bins
            .map(|lag| lag as f64 * config.bin_width.as_micros() as f64 / 1_000.0),
    })
}

pub fn compare_spike_trajectories(
    definition: &NetworkDefinition,
    control: &NetworkRun,
    variant: &NetworkRun,
    window_start: SimTime,
    window_end: SimTime,
    bin_width: SimDuration,
) -> Result<TrajectoryDifference, MetricsError> {
    validate_window(control, window_start, window_end, bin_width)?;
    validate_window(variant, window_start, window_end, bin_width)?;

    let duration_micros = window_end.as_micros() - window_start.as_micros();
    let bin_count = duration_micros.div_ceil(bin_width.as_micros()) as usize;
    let neuron_count = definition.neurons().len();
    let index_by_id = definition
        .neurons()
        .iter()
        .enumerate()
        .map(|(index, neuron)| (neuron.id, index))
        .collect::<BTreeMap<_, _>>();
    let control_bins = spike_state_bins(
        control,
        &index_by_id,
        window_start,
        window_end,
        bin_width,
        bin_count,
        neuron_count,
    )?;
    let variant_bins = spike_state_bins(
        variant,
        &index_by_id,
        window_start,
        window_end,
        bin_width,
        bin_count,
        neuron_count,
    )?;

    let mut distances = Vec::with_capacity(bin_count);
    for (control_bin, variant_bin) in control_bins.iter().zip(&variant_bins) {
        let squared = control_bin
            .iter()
            .zip(variant_bin)
            .map(|(left, right)| {
                let difference = f64::from(*left) - f64::from(*right);
                difference * difference
            })
            .sum::<f64>();
        distances.push((squared / neuron_count as f64).sqrt());
    }

    let changed = distances.iter().filter(|distance| **distance > 0.0).count();
    let last_changed = distances.iter().rposition(|distance| *distance > 0.0);
    Ok(TrajectoryDifference {
        bin_count,
        mean_rms_spike_difference: distances.iter().sum::<f64>() / bin_count as f64,
        maximum_rms_spike_difference: distances.iter().copied().fold(0.0, f64::max),
        changed_bin_fraction: changed as f64 / bin_count as f64,
        last_changed_time_ms: last_changed
            .map(|bin| (bin + 1) as f64 * bin_width.as_micros() as f64 / 1_000.0),
    })
}

fn validate_window(
    run: &NetworkRun,
    start: SimTime,
    end: SimTime,
    bin_width: SimDuration,
) -> Result<(), MetricsError> {
    if start < run.start_time || end > run.end_time || end <= start {
        return Err(MetricsError::WindowOutsideRun {
            run_start: run.start_time,
            run_end: run.end_time,
            requested_start: start,
            requested_end: end,
        });
    }
    if bin_width.as_micros() == 0 {
        return Err(MetricsError::ZeroBinWidth);
    }
    Ok(())
}

fn spike_state_bins(
    run: &NetworkRun,
    index_by_id: &BTreeMap<NeuronId, usize>,
    start: SimTime,
    end: SimTime,
    bin_width: SimDuration,
    bin_count: usize,
    neuron_count: usize,
) -> Result<Vec<Vec<u32>>, MetricsError> {
    let mut bins = vec![vec![0_u32; neuron_count]; bin_count];
    for spike in run
        .event_log
        .spikes()
        .filter(|spike| spike.time >= start && spike.time < end)
    {
        let neuron = *index_by_id
            .get(&spike.neuron_id)
            .ok_or(MetricsError::UnknownNeuronInLog(spike.neuron_id))?;
        let bin = ((spike.time.as_micros() - start.as_micros()) / bin_width.as_micros()) as usize;
        bins[bin][neuron] += 1;
    }
    Ok(bins)
}

fn silence_metrics(spike_times: &[u64], start: u64, end: u64) -> (f64, f64) {
    let mut previous = start;
    let mut gaps = Vec::with_capacity(spike_times.len() + 1);
    for &time in spike_times {
        gaps.push(time - previous);
        previous = time;
    }
    gaps.push(end - previous);

    let maximum = gaps.iter().copied().max().unwrap_or(end - start);
    let mean = gaps.iter().sum::<u64>() as f64 / gaps.len() as f64;
    (maximum as f64 / 1_000.0, mean / 1_000.0)
}

fn peak_autocorrelation(values: &[u64], maximum_lag: usize) -> (f64, Option<usize>) {
    if values.len() < 3 {
        return (0.0, None);
    }
    let mut best = 0.0;
    let mut best_lag = None;
    for lag in 1..=maximum_lag.min(values.len() - 2) {
        let left = &values[..values.len() - lag];
        let right = &values[lag..];
        let left_mean = left.iter().map(|value| *value as f64).sum::<f64>() / left.len() as f64;
        let right_mean = right.iter().map(|value| *value as f64).sum::<f64>() / right.len() as f64;
        let mut covariance = 0.0;
        let mut left_variance = 0.0;
        let mut right_variance = 0.0;
        for (&left_value, &right_value) in left.iter().zip(right) {
            let left_delta = left_value as f64 - left_mean;
            let right_delta = right_value as f64 - right_mean;
            covariance += left_delta * right_delta;
            left_variance += left_delta * left_delta;
            right_variance += right_delta * right_delta;
        }
        let denominator = (left_variance * right_variance).sqrt();
        if denominator > 0.0 {
            let correlation = covariance / denominator;
            if correlation > best {
                best = correlation;
                best_lag = Some(lag);
            }
        }
    }
    (best, best_lag)
}

#[derive(Clone, Debug, PartialEq)]
pub enum MetricsError {
    WindowOutsideRun {
        run_start: SimTime,
        run_end: SimTime,
        requested_start: SimTime,
        requested_end: SimTime,
    },
    ZeroBinWidth,
    InvalidSynchronousFraction(f64),
    UnknownNeuronInLog(NeuronId),
}

impl Display for MetricsError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::WindowOutsideRun {
                run_start,
                run_end,
                requested_start,
                requested_end,
            } => write!(
                formatter,
                "metrics window {}..{} us is outside run {}..{} us",
                requested_start.as_micros(),
                requested_end.as_micros(),
                run_start.as_micros(),
                run_end.as_micros()
            ),
            Self::ZeroBinWidth => formatter.write_str("metrics bin width must be positive"),
            Self::InvalidSynchronousFraction(value) => write!(
                formatter,
                "synchronous fraction threshold must be in 0..=1, got {value}"
            ),
            Self::UnknownNeuronInLog(id) => {
                write!(formatter, "event log references unknown neuron {}", id.0)
            }
        }
    }
}

impl Error for MetricsError {}
