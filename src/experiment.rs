use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::lif::{EventId, InputPolarity, LifParameters, NeuronId, SimDuration, SimTime};
use crate::metrics::{
    MetricsConfig, MetricsError, NetworkMetrics, TrajectoryDifference, compare_spike_trajectories,
    compute_network_metrics,
};
use crate::network::{
    ExternalInput, ExternalInputKind, NetworkDefinition, NetworkError, NetworkRun, NeuronPolarity,
    NeuronSpec, Position3, PropagationSpec, SynapseId, SynapseSpec, simulate_network,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeneratedNetworkConfig {
    pub neuron_count: u32,
    pub inhibitory_neuron_count: u32,
    pub out_degree: u32,
    pub inhibitory_out_degree: u32,
    pub seed: u64,
    pub excitatory_weight_mv: f64,
    pub inhibitory_weight_mv: f64,
    pub conduction_velocity_units_per_ms: f64,
    pub minimum_synaptic_delay: SimDuration,
    pub synaptic_delay_jitter: SimDuration,
    pub threshold_jitter_mv: f64,
    pub time_constant_jitter_ms: f64,
}

impl Default for GeneratedNetworkConfig {
    fn default() -> Self {
        Self {
            neuron_count: 100,
            inhibitory_neuron_count: 20,
            out_degree: 9,
            inhibitory_out_degree: 32,
            seed: 0x4e45_5552_4f32_0001,
            excitatory_weight_mv: 8.0,
            inhibitory_weight_mv: 13.2,
            conduction_velocity_units_per_ms: 0.25,
            minimum_synaptic_delay: SimDuration::from_micros(500),
            synaptic_delay_jitter: SimDuration::from_micros(1_000),
            threshold_jitter_mv: 1.0,
            time_constant_jitter_ms: 2.0,
        }
    }
}

pub fn generate_network(
    config: GeneratedNetworkConfig,
) -> Result<NetworkDefinition, ExperimentError> {
    validate_generated_config(config)?;
    let mut random = SplitMix64::new(config.seed);
    let excitatory_count = config.neuron_count - config.inhibitory_neuron_count;
    let mut neurons = Vec::with_capacity(config.neuron_count as usize);
    for id in 0..config.neuron_count {
        let threshold_jitter = random.symmetric(config.threshold_jitter_mv);
        let time_constant_jitter = random.symmetric(config.time_constant_jitter_ms);
        neurons.push(NeuronSpec {
            id: NeuronId(id),
            polarity: if id < excitatory_count {
                NeuronPolarity::Excitatory
            } else {
                NeuronPolarity::Inhibitory
            },
            position: Position3::new(random.unit(), random.unit(), random.unit()),
            parameters: LifParameters {
                rest_potential_mv: -70.0,
                reset_potential_mv: -68.0,
                threshold_mv: -55.0 + threshold_jitter,
                membrane_time_constant_ms: 12.0 + time_constant_jitter,
                refractory_period: SimDuration::from_micros(2_000),
            },
            initial_potential_mv: -70.0,
        });
    }

    let excitatory_capacity = excitatory_count as usize * config.out_degree as usize;
    let inhibitory_capacity =
        config.inhibitory_neuron_count as usize * config.inhibitory_out_degree as usize;
    let mut synapses = Vec::with_capacity(excitatory_capacity + inhibitory_capacity);
    let mut next_synapse_id = 0_u32;
    for source in 0..config.neuron_count {
        let source_out_degree = if source < excitatory_count {
            config.out_degree
        } else {
            config.inhibitory_out_degree
        };
        let mut targets = std::collections::BTreeSet::new();
        targets.insert((source + 1) % config.neuron_count);
        while targets.len() < source_out_degree as usize {
            let target = random.index(config.neuron_count);
            if target != source {
                targets.insert(target);
            }
        }

        for target in targets {
            let source_position = neurons[source as usize].position;
            let target_position = neurons[target as usize].position;
            let path_length = euclidean_distance(source_position, target_position) * 1.15 + 0.01;
            let delay_jitter = if config.synaptic_delay_jitter.as_micros() == 0 {
                0
            } else {
                random.next_u64() % (config.synaptic_delay_jitter.as_micros() + 1)
            };
            let magnitude_mv = if source < excitatory_count {
                config.excitatory_weight_mv
            } else {
                config.inhibitory_weight_mv
            };
            synapses.push(SynapseSpec {
                id: SynapseId(next_synapse_id),
                source: NeuronId(source),
                target: NeuronId(target),
                magnitude_mv,
                propagation: PropagationSpec {
                    path_length,
                    conduction_velocity_units_per_ms: config.conduction_velocity_units_per_ms,
                    synaptic_delay: SimDuration::from_micros(
                        config.minimum_synaptic_delay.as_micros() + delay_jitter,
                    ),
                },
            });
            next_synapse_id = next_synapse_id
                .checked_add(1)
                .ok_or(ExperimentError::SynapseIdOverflow)?;
        }
    }

    NetworkDefinition::new(SimTime::ZERO, neurons, synapses).map_err(ExperimentError::Network)
}

pub fn pacemaker_inputs(
    targets: &[NeuronId],
    start: SimTime,
    end: SimTime,
    interval: SimDuration,
    magnitude_mv: f64,
    first_event_id: u64,
) -> Result<Vec<ExternalInput>, ExperimentError> {
    if targets.is_empty() {
        return Err(ExperimentError::NoPacemakerTargets);
    }
    if interval.as_micros() == 0 {
        return Err(ExperimentError::ZeroPacemakerInterval);
    }
    if end <= start {
        return Err(ExperimentError::InvalidScheduleWindow { start, end });
    }

    let spacing = (interval.as_micros() / targets.len() as u64).max(1);
    let mut inputs = Vec::new();
    let mut pulse_start = start.as_micros();
    let mut next_id = first_event_id;
    while pulse_start < end.as_micros() {
        for (offset, target) in targets.iter().enumerate() {
            let time = pulse_start.saturating_add(spacing.saturating_mul(offset as u64));
            if time >= end.as_micros() {
                break;
            }
            inputs.push(ExternalInput::new(
                EventId(next_id),
                *target,
                SimTime::from_micros(time),
                InputPolarity::Excitatory,
                magnitude_mv,
                ExternalInputKind::Pacemaker,
            ));
            next_id = next_id
                .checked_add(1)
                .ok_or(ExperimentError::ExternalEventIdOverflow)?;
        }
        pulse_start = pulse_start
            .checked_add(interval.as_micros())
            .ok_or(ExperimentError::ScheduleTimeOverflow)?;
    }
    Ok(inputs)
}

pub fn local_stimulus_inputs(
    definition: &NetworkDefinition,
    center: Position3,
    target_count: usize,
    time: SimTime,
    magnitude_mv: f64,
    first_event_id: u64,
) -> Result<Vec<ExternalInput>, ExperimentError> {
    if target_count == 0 || target_count > definition.neurons().len() {
        return Err(ExperimentError::InvalidStimulusTargetCount(target_count));
    }
    let mut by_distance = definition
        .neurons()
        .iter()
        .map(|neuron| (squared_distance(neuron.position, center), neuron.id))
        .collect::<Vec<_>>();
    by_distance.sort_by(|left, right| {
        left.0
            .total_cmp(&right.0)
            .then_with(|| left.1.cmp(&right.1))
    });

    by_distance
        .into_iter()
        .take(target_count)
        .enumerate()
        .map(|(offset, (_, target))| {
            let id = first_event_id
                .checked_add(offset as u64)
                .ok_or(ExperimentError::ExternalEventIdOverflow)?;
            Ok(ExternalInput::new(
                EventId(id),
                target,
                time,
                InputPolarity::Excitatory,
                magnitude_mv,
                ExternalInputKind::Stimulus,
            ))
        })
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gate2ExperimentConfig {
    pub network: GeneratedNetworkConfig,
    pub run_end: SimTime,
    pub initialization_time: SimTime,
    pub pacemaker_start: SimTime,
    pub pacemaker_stop: SimTime,
    pub pacemaker_interval: SimDuration,
    pub pacemaker_magnitude_mv: f64,
    pub pacemaker_neuron_count: u32,
    pub stimulus_time: SimTime,
    pub stimulus_neuron_count: usize,
    pub stimulus_magnitude_mv: f64,
    pub metric_bin_width: SimDuration,
}

impl Default for Gate2ExperimentConfig {
    fn default() -> Self {
        Self {
            network: GeneratedNetworkConfig::default(),
            run_end: SimTime::from_micros(2_500_000),
            initialization_time: SimTime::from_micros(100_000),
            pacemaker_start: SimTime::from_micros(100_000),
            pacemaker_stop: SimTime::from_micros(1_500_000),
            pacemaker_interval: SimDuration::from_micros(20_000),
            pacemaker_magnitude_mv: 16.0,
            pacemaker_neuron_count: 4,
            stimulus_time: SimTime::from_micros(1_200_000),
            stimulus_neuron_count: 8,
            stimulus_magnitude_mv: 16.0,
            metric_bin_width: SimDuration::from_micros(10_000),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Gate2Summary {
    pub no_drive: NetworkMetrics,
    pub driven: NetworkMetrics,
    pub withdrawal: NetworkMetrics,
    pub stimulus_trajectory: TrajectoryDifference,
    pub no_drive_last_spike_ms: Option<f64>,
    pub withdrawal_last_spike_after_stop_ms: Option<f64>,
    pub no_drive_digest: u64,
    pub driven_digest: u64,
    pub stimulus_control_digest: u64,
    pub stimulus_variant_digest: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Gate2ExperimentResult {
    pub definition: NetworkDefinition,
    pub no_drive_run: NetworkRun,
    pub driven_withdrawal_run: NetworkRun,
    pub stimulus_control_run: NetworkRun,
    pub stimulus_variant_run: NetworkRun,
    pub summary: Gate2Summary,
}

/// Frozen engineering acceptance bands for `experiment-001/v1`.
/// These bounds classify the chosen pacemaker-supported regime; they are not
/// claims about biological firing rates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gate2AcceptanceCriteria {
    pub maximum_no_drive_rate_hz: f64,
    pub maximum_no_drive_tail_ms: f64,
    pub minimum_driven_rate_hz: f64,
    pub maximum_driven_rate_hz: f64,
    pub minimum_active_fraction: f64,
    pub maximum_active_fraction: f64,
    pub maximum_synchronous_bin_fraction: f64,
    pub maximum_silence_ms: f64,
    pub maximum_peak_autocorrelation: f64,
    pub minimum_inhibitory_to_excitatory_arrival_ratio: f64,
    pub maximum_withdrawal_rate_hz: f64,
    pub minimum_trajectory_difference: f64,
    pub minimum_changed_bin_fraction: f64,
}

impl Gate2AcceptanceCriteria {
    pub const fn experiment_001_v1() -> Self {
        Self {
            maximum_no_drive_rate_hz: 0.5,
            maximum_no_drive_tail_ms: 50.0,
            minimum_driven_rate_hz: 4.0,
            maximum_driven_rate_hz: 10.0,
            minimum_active_fraction: 0.45,
            maximum_active_fraction: 0.70,
            maximum_synchronous_bin_fraction: 0.10,
            maximum_silence_ms: 30.0,
            maximum_peak_autocorrelation: 0.75,
            minimum_inhibitory_to_excitatory_arrival_ratio: 0.40,
            maximum_withdrawal_rate_hz: 0.5,
            minimum_trajectory_difference: 0.25,
            minimum_changed_bin_fraction: 0.50,
        }
    }

    pub fn evaluate(self, summary: &Gate2Summary) -> Gate2AcceptanceReport {
        let mut violations = Vec::new();
        check_max(
            &mut violations,
            "no_drive.mean_firing_rate_hz",
            summary.no_drive.mean_firing_rate_hz,
            self.maximum_no_drive_rate_hz,
        );
        check_max(
            &mut violations,
            "no_drive.last_spike_after_initialization_ms",
            summary.no_drive_last_spike_ms.unwrap_or(0.0),
            self.maximum_no_drive_tail_ms,
        );
        check_range(
            &mut violations,
            "driven.mean_firing_rate_hz",
            summary.driven.mean_firing_rate_hz,
            self.minimum_driven_rate_hz,
            self.maximum_driven_rate_hz,
        );
        check_range(
            &mut violations,
            "driven.active_neuron_fraction",
            summary.driven.active_neuron_fraction,
            self.minimum_active_fraction,
            self.maximum_active_fraction,
        );
        check_max(
            &mut violations,
            "driven.synchronous_burst_bin_fraction",
            summary.driven.synchronous_burst_bin_fraction,
            self.maximum_synchronous_bin_fraction,
        );
        check_max(
            &mut violations,
            "driven.maximum_silence_ms",
            summary.driven.maximum_silence_ms,
            self.maximum_silence_ms,
        );
        check_max(
            &mut violations,
            "driven.peak_autocorrelation",
            summary.driven.peak_autocorrelation,
            self.maximum_peak_autocorrelation,
        );
        let inhibition_ratio = summary.driven.inhibitory_arrival_count as f64
            / summary.driven.excitatory_arrival_count.max(1) as f64;
        check_min(
            &mut violations,
            "driven.inhibitory_to_excitatory_arrival_ratio",
            inhibition_ratio,
            self.minimum_inhibitory_to_excitatory_arrival_ratio,
        );
        check_max(
            &mut violations,
            "withdrawal.mean_firing_rate_hz",
            summary.withdrawal.mean_firing_rate_hz,
            self.maximum_withdrawal_rate_hz,
        );
        check_min(
            &mut violations,
            "stimulus.mean_rms_spike_difference",
            summary.stimulus_trajectory.mean_rms_spike_difference,
            self.minimum_trajectory_difference,
        );
        check_min(
            &mut violations,
            "stimulus.changed_bin_fraction",
            summary.stimulus_trajectory.changed_bin_fraction,
            self.minimum_changed_bin_fraction,
        );

        Gate2AcceptanceReport {
            passed: violations.is_empty(),
            violations,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Gate2AcceptanceReport {
    pub passed: bool,
    pub violations: Vec<String>,
}

fn check_min(violations: &mut Vec<String>, name: &str, actual: f64, minimum: f64) {
    if actual < minimum {
        violations.push(format!("{name}={actual:.6} is below minimum {minimum:.6}"));
    }
}

fn check_max(violations: &mut Vec<String>, name: &str, actual: f64, maximum: f64) {
    if actual > maximum {
        violations.push(format!("{name}={actual:.6} exceeds maximum {maximum:.6}"));
    }
}

fn check_range(violations: &mut Vec<String>, name: &str, actual: f64, minimum: f64, maximum: f64) {
    check_min(violations, name, actual, minimum);
    check_max(violations, name, actual, maximum);
}

pub fn run_gate2_experiment(
    config: Gate2ExperimentConfig,
) -> Result<Gate2ExperimentResult, ExperimentError> {
    validate_gate2_config(config)?;
    let definition = generate_network(config.network)?;
    let pacemaker_targets = (0..config.pacemaker_neuron_count)
        .map(NeuronId)
        .collect::<Vec<_>>();

    let initialization = pacemaker_targets
        .iter()
        .enumerate()
        .map(|(offset, target)| {
            ExternalInput::new(
                EventId(offset as u64),
                *target,
                config.initialization_time,
                InputPolarity::Excitatory,
                config.pacemaker_magnitude_mv,
                ExternalInputKind::Initialization,
            )
        })
        .collect::<Vec<_>>();
    let no_drive_run = simulate_network(&definition, &initialization, config.run_end)
        .map_err(ExperimentError::Network)?;

    let driven_inputs = pacemaker_inputs(
        &pacemaker_targets,
        config.pacemaker_start,
        config.pacemaker_stop,
        config.pacemaker_interval,
        config.pacemaker_magnitude_mv,
        10_000,
    )?;
    let driven_withdrawal_run = simulate_network(&definition, &driven_inputs, config.run_end)
        .map_err(ExperimentError::Network)?;

    let continuous_pacemaker = pacemaker_inputs(
        &pacemaker_targets,
        config.pacemaker_start,
        config.run_end,
        config.pacemaker_interval,
        config.pacemaker_magnitude_mv,
        100_000,
    )?;
    let stimulus_control_run = simulate_network(&definition, &continuous_pacemaker, config.run_end)
        .map_err(ExperimentError::Network)?;
    let mut stimulus_inputs = continuous_pacemaker;
    stimulus_inputs.extend(local_stimulus_inputs(
        &definition,
        Position3::new(0.5, 0.5, 0.5),
        config.stimulus_neuron_count,
        config.stimulus_time,
        config.stimulus_magnitude_mv,
        1_000_000,
    )?);
    let stimulus_variant_run = simulate_network(&definition, &stimulus_inputs, config.run_end)
        .map_err(ExperimentError::Network)?;

    let metric_config = |window_start, window_end| MetricsConfig {
        window_start,
        window_end,
        bin_width: config.metric_bin_width,
        synchronous_fraction_threshold: 0.20,
        maximum_period_lag: SimDuration::from_micros(200_000),
    };
    let no_drive = compute_network_metrics(
        &definition,
        &no_drive_run,
        metric_config(config.initialization_time, config.pacemaker_stop),
    )
    .map_err(ExperimentError::Metrics)?;
    let driven = compute_network_metrics(
        &definition,
        &driven_withdrawal_run,
        metric_config(SimTime::from_micros(500_000), config.pacemaker_stop),
    )
    .map_err(ExperimentError::Metrics)?;
    let withdrawal = compute_network_metrics(
        &definition,
        &driven_withdrawal_run,
        metric_config(config.pacemaker_stop, config.run_end),
    )
    .map_err(ExperimentError::Metrics)?;
    let stimulus_trajectory = compare_spike_trajectories(
        &definition,
        &stimulus_control_run,
        &stimulus_variant_run,
        config.stimulus_time,
        config.run_end,
        config.metric_bin_width,
    )
    .map_err(ExperimentError::Metrics)?;

    let no_drive_last_spike_ms = last_spike_at_or_after(&no_drive_run, config.initialization_time)
        .map(|time| (time.as_micros() - config.initialization_time.as_micros()) as f64 / 1_000.0);
    let withdrawal_last_spike_after_stop_ms =
        last_spike_at_or_after(&driven_withdrawal_run, config.pacemaker_stop)
            .map(|time| (time.as_micros() - config.pacemaker_stop.as_micros()) as f64 / 1_000.0);

    let summary = Gate2Summary {
        no_drive,
        driven,
        withdrawal,
        stimulus_trajectory,
        no_drive_last_spike_ms,
        withdrawal_last_spike_after_stop_ms,
        no_drive_digest: no_drive_run.event_log.stable_digest(),
        driven_digest: driven_withdrawal_run.event_log.stable_digest(),
        stimulus_control_digest: stimulus_control_run.event_log.stable_digest(),
        stimulus_variant_digest: stimulus_variant_run.event_log.stable_digest(),
    };

    Ok(Gate2ExperimentResult {
        definition,
        no_drive_run,
        driven_withdrawal_run,
        stimulus_control_run,
        stimulus_variant_run,
        summary,
    })
}

fn last_spike_at_or_after(run: &NetworkRun, start: SimTime) -> Option<SimTime> {
    run.event_log
        .spikes()
        .filter(|spike| spike.time >= start)
        .map(|spike| spike.time)
        .max()
}

fn validate_generated_config(config: GeneratedNetworkConfig) -> Result<(), ExperimentError> {
    if !(10..=300).contains(&config.neuron_count) {
        return Err(ExperimentError::InvalidNeuronCount(config.neuron_count));
    }
    if config.inhibitory_neuron_count == 0 || config.inhibitory_neuron_count >= config.neuron_count
    {
        return Err(ExperimentError::InvalidInhibitoryCount(
            config.inhibitory_neuron_count,
        ));
    }
    if config.out_degree == 0 || config.out_degree >= config.neuron_count {
        return Err(ExperimentError::InvalidOutDegree(config.out_degree));
    }
    if config.inhibitory_out_degree == 0 || config.inhibitory_out_degree >= config.neuron_count {
        return Err(ExperimentError::InvalidInhibitoryOutDegree(
            config.inhibitory_out_degree,
        ));
    }
    for (name, value) in [
        ("excitatory_weight_mv", config.excitatory_weight_mv),
        ("inhibitory_weight_mv", config.inhibitory_weight_mv),
        (
            "conduction_velocity_units_per_ms",
            config.conduction_velocity_units_per_ms,
        ),
    ] {
        if !value.is_finite() || value <= 0.0 {
            return Err(ExperimentError::InvalidPositiveParameter { name, value });
        }
    }
    for (name, value) in [
        ("threshold_jitter_mv", config.threshold_jitter_mv),
        ("time_constant_jitter_ms", config.time_constant_jitter_ms),
    ] {
        if !value.is_finite() || value < 0.0 {
            return Err(ExperimentError::InvalidNonNegativeParameter { name, value });
        }
    }
    Ok(())
}

fn validate_gate2_config(config: Gate2ExperimentConfig) -> Result<(), ExperimentError> {
    if config.pacemaker_neuron_count == 0
        || config.pacemaker_neuron_count
            > config.network.neuron_count - config.network.inhibitory_neuron_count
    {
        return Err(ExperimentError::InvalidPacemakerNeuronCount(
            config.pacemaker_neuron_count,
        ));
    }
    if !(config.initialization_time < config.pacemaker_stop
        && config.pacemaker_start < config.pacemaker_stop
        && config.pacemaker_stop < config.run_end
        && config.stimulus_time > config.pacemaker_start
        && config.stimulus_time < config.run_end)
    {
        return Err(ExperimentError::InvalidExperimentTimeline);
    }
    if config.metric_bin_width.as_micros() == 0 {
        return Err(ExperimentError::ZeroMetricBinWidth);
    }
    Ok(())
}

fn euclidean_distance(left: Position3, right: Position3) -> f64 {
    squared_distance(left, right).sqrt()
}

fn squared_distance(left: Position3, right: Position3) -> f64 {
    (left.x - right.x).powi(2) + (left.y - right.y).powi(2) + (left.z - right.z).powi(2)
}

#[derive(Clone, Copy, Debug)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / ((1_u64 << 53) as f64))
    }

    fn symmetric(&mut self, magnitude: f64) -> f64 {
        (self.unit() * 2.0 - 1.0) * magnitude
    }

    fn index(&mut self, upper_exclusive: u32) -> u32 {
        (self.next_u64() % u64::from(upper_exclusive)) as u32
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExperimentError {
    InvalidNeuronCount(u32),
    InvalidInhibitoryCount(u32),
    InvalidOutDegree(u32),
    InvalidInhibitoryOutDegree(u32),
    InvalidPositiveParameter { name: &'static str, value: f64 },
    InvalidNonNegativeParameter { name: &'static str, value: f64 },
    SynapseIdOverflow,
    ExternalEventIdOverflow,
    ScheduleTimeOverflow,
    NoPacemakerTargets,
    ZeroPacemakerInterval,
    InvalidScheduleWindow { start: SimTime, end: SimTime },
    InvalidStimulusTargetCount(usize),
    InvalidPacemakerNeuronCount(u32),
    InvalidExperimentTimeline,
    ZeroMetricBinWidth,
    Network(NetworkError),
    Metrics(MetricsError),
}

impl Display for ExperimentError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNeuronCount(count) => {
                write!(formatter, "neuron count must be in 10..=300, got {count}")
            }
            Self::InvalidInhibitoryCount(count) => {
                write!(formatter, "invalid inhibitory neuron count {count}")
            }
            Self::InvalidOutDegree(degree) => write!(formatter, "invalid out-degree {degree}"),
            Self::InvalidInhibitoryOutDegree(degree) => {
                write!(formatter, "invalid inhibitory out-degree {degree}")
            }
            Self::InvalidPositiveParameter { name, value } => {
                write!(formatter, "{name} must be finite and positive, got {value}")
            }
            Self::InvalidNonNegativeParameter { name, value } => write!(
                formatter,
                "{name} must be finite and non-negative, got {value}"
            ),
            Self::SynapseIdOverflow => formatter.write_str("synapse id overflow"),
            Self::ExternalEventIdOverflow => formatter.write_str("external event id overflow"),
            Self::ScheduleTimeOverflow => formatter.write_str("schedule time overflow"),
            Self::NoPacemakerTargets => formatter.write_str("pacemaker target list is empty"),
            Self::ZeroPacemakerInterval => {
                formatter.write_str("pacemaker interval must be positive")
            }
            Self::InvalidScheduleWindow { start, end } => write!(
                formatter,
                "invalid schedule window {}..{} us",
                start.as_micros(),
                end.as_micros()
            ),
            Self::InvalidStimulusTargetCount(count) => {
                write!(formatter, "invalid local stimulus target count {count}")
            }
            Self::InvalidPacemakerNeuronCount(count) => {
                write!(formatter, "invalid pacemaker neuron count {count}")
            }
            Self::InvalidExperimentTimeline => formatter.write_str("invalid experiment timeline"),
            Self::ZeroMetricBinWidth => formatter.write_str("metric bin width must be positive"),
            Self::Network(error) => Display::fmt(error, formatter),
            Self::Metrics(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for ExperimentError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Network(error) => Some(error),
            Self::Metrics(error) => Some(error),
            _ => None,
        }
    }
}
