use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter};

use serde::Serialize;

use crate::experiment::{
    ExperimentError, Pattern3x3, Phase2ExperimentConfig, generate_network, pacemaker_inputs,
};
use crate::lif::{
    EventId, InputPolarity, LifNeuron, ModelError, NeuronId, SimDuration, SimTime, TimedInput,
};
use crate::metrics::{MetricsConfig, MetricsError, NetworkMetrics, compute_network_metrics};
use crate::network::{
    ActivityRegulatorConfig, ExternalInput, ExternalInputKind, InputOrigin, LogEvent,
    NetworkDefinition, NetworkError, NetworkRun, NeuronPolarity, simulate_network,
    simulate_network_with_regulator,
};

type SampledStateMatrices = (Vec<Vec<f32>>, Vec<Vec<bool>>);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Phase2PatternId {
    Blank,
    Single0,
    Single1,
    Single2,
    Single3,
    Single4,
    Single5,
    Single6,
    Single7,
    Single8,
    Horizontal,
    Vertical,
    MainDiagonal,
    AntiDiagonal,
}

impl Phase2PatternId {
    pub const PRIMARY: [Self; 4] = [
        Self::Horizontal,
        Self::Vertical,
        Self::MainDiagonal,
        Self::AntiDiagonal,
    ];

    pub const SINGLE_PIXELS: [Self; 9] = [
        Self::Single0,
        Self::Single1,
        Self::Single2,
        Self::Single3,
        Self::Single4,
        Self::Single5,
        Self::Single6,
        Self::Single7,
        Self::Single8,
    ];

    pub const fn pattern(self) -> Pattern3x3 {
        let mut cells = [false; 9];
        match self {
            Self::Blank => {}
            Self::Single0 => cells[0] = true,
            Self::Single1 => cells[1] = true,
            Self::Single2 => cells[2] = true,
            Self::Single3 => cells[3] = true,
            Self::Single4 => cells[4] = true,
            Self::Single5 => cells[5] = true,
            Self::Single6 => cells[6] = true,
            Self::Single7 => cells[7] = true,
            Self::Single8 => cells[8] = true,
            Self::Horizontal => {
                cells = [false, false, false, true, true, true, false, false, false]
            }
            Self::Vertical => cells = [false, true, false, false, true, false, false, true, false],
            Self::MainDiagonal => {
                cells = [true, false, false, false, true, false, false, false, true]
            }
            Self::AntiDiagonal => {
                cells = [false, false, true, false, true, false, true, false, false]
            }
        }
        Pattern3x3 { cells }
    }

    pub const fn class_index(self) -> Option<usize> {
        match self {
            Self::Horizontal => Some(0),
            Self::Vertical => Some(1),
            Self::MainDiagonal => Some(2),
            Self::AntiDiagonal => Some(3),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrialSplit {
    Train,
    Test,
    Control,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Phase2ProtocolConfig {
    pub base: Phase2ExperimentConfig,
    pub structure_seed_count: usize,
    pub structure_seed_scan_limit: usize,
    pub structure_seed_screen_trials: usize,
    pub trials_per_pattern: usize,
    pub training_trials_per_pattern: usize,
    pub single_pixel_trials: usize,
    pub analysis_bin_width: SimDuration,
    pub input_jitter: SimDuration,
    pub functional_group_size: usize,
    pub label_permutation_count: usize,
    pub connection_swap_multiplier: usize,
    pub activity_regulator: Option<ActivityRegulatorConfig>,
}

impl Default for Phase2ProtocolConfig {
    fn default() -> Self {
        Self {
            base: Phase2ExperimentConfig {
                pattern_interval: SimDuration::from_micros(50_000),
                pattern_magnitude_mv: 6.0,
                ..Phase2ExperimentConfig::default()
            },
            structure_seed_count: 3,
            structure_seed_scan_limit: 512,
            structure_seed_screen_trials: 10,
            trials_per_pattern: 30,
            training_trials_per_pattern: 20,
            single_pixel_trials: 3,
            analysis_bin_width: SimDuration::from_micros(10_000),
            input_jitter: SimDuration::from_micros(20_000),
            functional_group_size: 10,
            label_permutation_count: 100,
            connection_swap_multiplier: 10,
            activity_regulator: Some(ActivityRegulatorConfig {
                unique_spike_threshold: 21,
                cooldown: SimDuration::from_micros(3_000),
                ..ActivityRegulatorConfig::default()
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Phase2StabilityCriteria {
    pub minimum_firing_rate_hz: f64,
    pub maximum_firing_rate_hz: f64,
    pub minimum_active_fraction: f64,
    pub maximum_active_fraction: f64,
    pub maximum_synchronous_fraction: f64,
    pub maximum_silence_ms: f64,
    pub maximum_peak_autocorrelation: f64,
}

impl Default for Phase2StabilityCriteria {
    fn default() -> Self {
        Self {
            minimum_firing_rate_hz: 4.0,
            maximum_firing_rate_hz: 10.0,
            minimum_active_fraction: 0.45,
            maximum_active_fraction: 0.70,
            maximum_synchronous_fraction: 0.10,
            maximum_silence_ms: 30.0,
            maximum_peak_autocorrelation: 0.75,
        }
    }
}

impl Phase2StabilityCriteria {
    pub fn input_driven() -> Self {
        Self {
            maximum_peak_autocorrelation: 1.0,
            ..Self::default()
        }
    }

    pub fn accepts(self, metrics: &NetworkMetrics) -> bool {
        (self.minimum_firing_rate_hz..=self.maximum_firing_rate_hz)
            .contains(&metrics.mean_firing_rate_hz)
            && (self.minimum_active_fraction..=self.maximum_active_fraction)
                .contains(&metrics.active_neuron_fraction)
            && metrics.synchronous_burst_bin_fraction <= self.maximum_synchronous_fraction
            && metrics.maximum_silence_ms <= self.maximum_silence_ms
            && metrics.peak_autocorrelation <= self.maximum_peak_autocorrelation
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrialMetadata {
    pub structure_seed: u64,
    pub trial_index: usize,
    pub trial_seed: u64,
    pub pacemaker_phase_ms: f64,
    pub pattern: Phase2PatternId,
    pub split: TrialSplit,
    pub event_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrialResponse {
    pub metadata: TrialMetadata,
    pub neuron_ids: Vec<u32>,
    pub bin_width_ms: f64,
    pub binned_spike_counts: Vec<Vec<u16>>,
    pub sampled_membrane_potential_mv: Vec<Vec<f32>>,
    pub refractory: Vec<Vec<bool>>,
    pub excitatory_arrival_counts: Vec<Vec<u16>>,
    pub inhibitory_arrival_counts: Vec<Vec<u16>>,
    pub first_spike_latency_ms: Vec<Option<f64>>,
    pub regulator_episode_count: usize,
    pub suppressed_propagation_count: usize,
    pub metrics: NetworkMetrics,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Phase2RawEvent {
    Input {
        sequence: u64,
        time_us: u64,
        target: u32,
        polarity: String,
        magnitude_mv: f64,
        origin: String,
        origin_event_id: u64,
        source: Option<u32>,
        synapse_id: Option<u32>,
        ignored_during_refractory: bool,
    },
    Spike {
        spike_id: u64,
        time_us: u64,
        neuron_id: u32,
    },
    Regulation {
        episode_id: u64,
        time_us: u64,
        trigger_neuron_id: u32,
        unique_spike_count: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Phase2TrialArtifact {
    pub response: TrialResponse,
    pub raw_events: Vec<Phase2RawEvent>,
}

impl TrialResponse {
    pub fn feature_vector(&self, stimulus_bin_count: usize) -> Vec<f64> {
        let neuron_count = self.neuron_ids.len();
        let bin_count = self.binned_spike_counts.len();
        let boundaries = [
            0,
            stimulus_bin_count / 4,
            stimulus_bin_count / 2,
            stimulus_bin_count,
            bin_count,
        ];
        let mut features = Vec::with_capacity(neuron_count * 11);
        for neuron in 0..neuron_count {
            for range in boundaries.windows(2) {
                features.push(
                    self.binned_spike_counts[range[0]..range[1]]
                        .iter()
                        .map(|bin| f64::from(bin[neuron]))
                        .sum(),
                );
            }
            for range in boundaries.windows(2) {
                let samples = &self.sampled_membrane_potential_mv[range[0]..range[1]];
                let mean = if samples.is_empty() {
                    0.0
                } else {
                    samples
                        .iter()
                        .map(|bin| f64::from(bin[neuron]))
                        .sum::<f64>()
                        / samples.len() as f64
                };
                features.push(mean);
            }
            let duration_ms = bin_count as f64 * self.bin_width_ms;
            features.push(
                self.first_spike_latency_ms[neuron].unwrap_or(duration_ms)
                    / duration_ms.max(f64::EPSILON),
            );
            features.push(
                self.excitatory_arrival_counts
                    .iter()
                    .map(|bin| f64::from(bin[neuron]))
                    .sum(),
            );
            features.push(
                self.inhibitory_arrival_counts
                    .iter()
                    .map(|bin| f64::from(bin[neuron]))
                    .sum(),
            );
        }
        features
    }

    fn total_spikes_by_neuron(&self) -> Vec<f64> {
        (0..self.neuron_ids.len())
            .map(|neuron| {
                self.binned_spike_counts
                    .iter()
                    .map(|bin| f64::from(bin[neuron]))
                    .sum()
            })
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccuracyReport {
    pub accuracy: f64,
    pub correct: usize,
    pub total: usize,
    pub chance_accuracy: f64,
    pub confidence_low: f64,
    pub confidence_high: f64,
    pub per_class_accuracy: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadoutReport {
    pub prototype: AccuracyReport,
    pub linear: AccuracyReport,
    pub label_permutation_mean_accuracy: f64,
    pub label_permutation_max_accuracy: f64,
    pub permutation_p_value: f64,
    pub within_class_distance: f64,
    pub between_class_distance: f64,
    pub separation_ratio: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionalNeuronScore {
    pub neuron_id: u32,
    pub polarity: String,
    pub response_delta: f64,
    pub reliability: f64,
    pub selectivity: f64,
    pub latency_std_ms: f64,
    pub coactivity_correlation: f64,
    pub readout_importance: f64,
    pub score: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionalGroupReport {
    pub id: String,
    pub pattern: Phase2PatternId,
    pub members: Vec<FunctionalNeuronScore>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AblationComparison {
    pub pattern: Phase2PatternId,
    pub candidate_group_ids: Vec<u32>,
    pub matched_group_ids: Vec<u32>,
    pub intact: AccuracyReport,
    pub candidate_ablation: AccuracyReport,
    pub matched_ablation: AccuracyReport,
    pub candidate_stability_pass_fraction: f64,
    pub matched_stability_pass_fraction: f64,
    pub target_accuracy_drop: f64,
    pub matched_target_accuracy_drop: f64,
    pub non_target_accuracy_drop: f64,
    pub selective_effect: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeedExperimentReport {
    pub structure_seed: u64,
    pub stability_pass_fraction: f64,
    pub severe_instability_count: usize,
    pub mean_regulator_episode_count: f64,
    pub mean_suppressed_propagation_count: f64,
    pub readout: ReadoutReport,
    pub shuffled_connectivity_readout: ReadoutReport,
    pub functional_groups: Vec<FunctionalGroupReport>,
    pub ablations: Vec<AblationComparison>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Phase2HypothesisReport {
    pub h1_reproducible_response: bool,
    pub h2_decodable_information: bool,
    pub h3_causal_functional_groups: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Phase2ProtocolReport {
    pub version: String,
    pub selected_structure_seeds: Vec<u64>,
    pub rejected_structure_seed_count: usize,
    pub trials_per_pattern_per_seed: usize,
    pub training_trials_per_pattern_per_seed: usize,
    pub input_neuron_ids: Vec<u32>,
    pub pattern_magnitude_mv: f64,
    pub pattern_interval_ms: f64,
    pub input_jitter_ms: f64,
    pub activity_regulator: Option<ActivityRegulatorReport>,
    pub patterns: Vec<Phase2PatternId>,
    pub baseline_stability_criteria: Phase2StabilityCriteria,
    pub input_stability_criteria: Phase2StabilityCriteria,
    pub regulated_input_stability: InputStabilityEvaluation,
    pub unregulated_input_stability: InputStabilityEvaluation,
    pub seeds: Vec<SeedExperimentReport>,
    pub hypotheses: Phase2HypothesisReport,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityRegulatorReport {
    pub window_ms: f64,
    pub unique_spike_threshold: usize,
    pub cooldown_ms: f64,
}

impl From<ActivityRegulatorConfig> for ActivityRegulatorReport {
    fn from(value: ActivityRegulatorConfig) -> Self {
        Self {
            window_ms: value.window.as_micros() as f64 / 1_000.0,
            unique_spike_threshold: value.unique_spike_threshold,
            cooldown_ms: value.cooldown.as_micros() as f64 / 1_000.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Phase2ProtocolResult {
    pub report: Phase2ProtocolReport,
    pub trials: Vec<TrialResponse>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructureSeedScreen {
    pub seed: u64,
    pub accepted: bool,
    pub pass_fraction: f64,
    pub metrics: NetworkMetrics,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputStabilityEvaluation {
    pub pattern_magnitude_mv: f64,
    pub pattern_interval_ms: f64,
    pub input_jitter_ms: f64,
    pub trial_count: usize,
    pub pass_fraction: f64,
    pub severe_instability_count: usize,
    pub mean_firing_rate_hz: f64,
    pub mean_active_fraction: f64,
    pub mean_synchronous_fraction: f64,
}

fn response_candidate_ids(config: Phase2ExperimentConfig) -> Vec<NeuronId> {
    let excluded = config
        .input_neuron_ids
        .into_iter()
        .chain((0..config.pacemaker_neuron_count).map(NeuronId))
        .collect::<BTreeSet<_>>();
    (0..config.network.neuron_count)
        .map(NeuronId)
        .filter(|id| !excluded.contains(id))
        .collect()
}

fn metric_config(config: Phase2ExperimentConfig) -> MetricsConfig {
    MetricsConfig {
        window_start: config.pattern_start,
        window_end: config.pattern_end,
        bin_width: config.metric_bin_width,
        synchronous_fraction_threshold: 0.20,
        maximum_period_lag: SimDuration::from_micros(200_000),
    }
}

fn trial_split(config: Phase2ProtocolConfig, trial_index: usize) -> TrialSplit {
    if trial_index < config.training_trials_per_pattern {
        TrialSplit::Train
    } else {
        TrialSplit::Test
    }
}

fn trial_seed(structure_seed: u64, trial_index: usize) -> u64 {
    structure_seed
        ^ (trial_index as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ 0x5032_5452_4941_4c01
}

fn trial_inputs(
    definition: &NetworkDefinition,
    config: Phase2ProtocolConfig,
    pattern: Phase2PatternId,
    trial_index: usize,
) -> Result<(Vec<ExternalInput>, f64), Phase2AnalysisError> {
    let phase_range = config.base.pacemaker_interval.as_micros();
    let phase = (trial_index as u64 * 2_000) % phase_range;
    let pacemaker_start = config
        .base
        .pacemaker_start
        .checked_add(SimDuration::from_micros(phase))
        .map_err(Phase2AnalysisError::Model)?;
    let pacemaker_targets = (0..config.base.pacemaker_neuron_count)
        .map(NeuronId)
        .collect::<Vec<_>>();
    let mut inputs = pacemaker_inputs(
        &pacemaker_targets,
        pacemaker_start,
        config.base.run_end,
        config.base.pacemaker_interval,
        config.base.pacemaker_magnitude_mv,
        100_000,
    )?;
    inputs.extend(jittered_pattern_inputs(
        definition,
        config,
        pattern.pattern(),
        trial_seed(config.base.network.seed, trial_index),
    )?);
    Ok((inputs, phase as f64 / 1_000.0))
}

fn jittered_pattern_inputs(
    definition: &NetworkDefinition,
    config: Phase2ProtocolConfig,
    pattern: Pattern3x3,
    seed: u64,
) -> Result<Vec<ExternalInput>, Phase2AnalysisError> {
    if pattern == Pattern3x3::BLANK {
        return Ok(Vec::new());
    }
    let known = definition
        .neurons()
        .iter()
        .map(|neuron| neuron.id)
        .collect::<BTreeSet<_>>();
    if config
        .base
        .input_neuron_ids
        .iter()
        .any(|id| !known.contains(id))
    {
        return Err(Phase2AnalysisError::InvalidConfig(
            "input neuron is absent from generated network",
        ));
    }
    let mut next_id = 2_000_000_u64;
    let jitter = config.input_jitter.as_micros() as i64;
    let mut result = Vec::new();
    for (cell, (active, target)) in pattern
        .cells
        .iter()
        .zip(config.base.input_neuron_ids)
        .enumerate()
    {
        if !active {
            continue;
        }
        let mut random =
            DeterministicRng::new(seed ^ (cell as u64 + 1).wrapping_mul(0xd1b5_4a32_d192_ed03));
        let mut pulse_time = config.base.pattern_start.as_micros();
        while pulse_time < config.base.pattern_end.as_micros() {
            let offset = random.symmetric_i64(jitter);
            let time = (pulse_time as i128 + i128::from(offset)).clamp(
                config.base.pattern_start.as_micros() as i128,
                config.base.pattern_end.as_micros() as i128 - 1,
            ) as u64;
            result.push(ExternalInput::new(
                EventId(next_id),
                target,
                SimTime::from_micros(time),
                InputPolarity::Excitatory,
                config.base.pattern_magnitude_mv,
                ExternalInputKind::Stimulus,
            ));
            next_id = next_id
                .checked_add(1)
                .ok_or(Phase2AnalysisError::InvalidConfig(
                    "pattern event id overflow",
                ))?;
            pulse_time = pulse_time
                .checked_add(config.base.pattern_interval.as_micros())
                .ok_or(Phase2AnalysisError::InvalidConfig(
                    "pattern schedule overflow",
                ))?;
        }
    }
    Ok(result)
}

fn run_trial(
    definition: &NetworkDefinition,
    config: Phase2ProtocolConfig,
    structure_seed: u64,
    pattern: Phase2PatternId,
    trial_index: usize,
    split: TrialSplit,
) -> Result<TrialResponse, Phase2AnalysisError> {
    let (inputs, pacemaker_phase_ms) = trial_inputs(definition, config, pattern, trial_index)?;
    let run = simulate_protocol_network(definition, &inputs, config)?;
    extract_trial_response(
        definition,
        &run,
        config,
        TrialMetadata {
            structure_seed,
            trial_index,
            trial_seed: trial_seed(structure_seed, trial_index),
            pacemaker_phase_ms,
            pattern,
            split,
            event_digest: format!("{:016x}", run.event_log.stable_digest()),
        },
    )
}

pub fn run_phase2_trial_artifact(
    mut config: Phase2ProtocolConfig,
    structure_seed: u64,
    pattern: Phase2PatternId,
    trial_index: usize,
    split: TrialSplit,
) -> Result<Phase2TrialArtifact, Phase2AnalysisError> {
    validate_protocol_config(config)?;
    config.base.network.seed = structure_seed;
    let definition = generate_network(config.base.network)?;
    let (inputs, pacemaker_phase_ms) = trial_inputs(&definition, config, pattern, trial_index)?;
    let run = simulate_protocol_network(&definition, &inputs, config)?;
    let metadata = TrialMetadata {
        structure_seed,
        trial_index,
        trial_seed: trial_seed(structure_seed, trial_index),
        pacemaker_phase_ms,
        pattern,
        split,
        event_digest: format!("{:016x}", run.event_log.stable_digest()),
    };
    let response = extract_trial_response(&definition, &run, config, metadata)?;
    let raw_events = run
        .event_log
        .events
        .iter()
        .map(|event| match event {
            LogEvent::Input(input) => {
                let (origin, origin_event_id, source, synapse_id) = match input.origin {
                    InputOrigin::External {
                        external_event_id,
                        kind,
                    } => (
                        match kind {
                            ExternalInputKind::Initialization => "initialization",
                            ExternalInputKind::Pacemaker => "pacemaker",
                            ExternalInputKind::Stimulus => "stimulus",
                        },
                        external_event_id.0,
                        None,
                        None,
                    ),
                    InputOrigin::Synaptic {
                        spike_id,
                        synapse_id,
                        source,
                    } => ("synaptic", spike_id.0, Some(source.0), Some(synapse_id.0)),
                };
                Phase2RawEvent::Input {
                    sequence: input.sequence.0,
                    time_us: input.time.as_micros(),
                    target: input.target.0,
                    polarity: match input.polarity {
                        InputPolarity::Excitatory => "excitatory".to_owned(),
                        InputPolarity::Inhibitory => "inhibitory".to_owned(),
                    },
                    magnitude_mv: input.magnitude_mv,
                    origin: origin.to_owned(),
                    origin_event_id,
                    source,
                    synapse_id,
                    ignored_during_refractory: input.ignored_during_refractory,
                }
            }
            LogEvent::Spike(spike) => Phase2RawEvent::Spike {
                spike_id: spike.id.0,
                time_us: spike.time.as_micros(),
                neuron_id: spike.neuron_id.0,
            },
            LogEvent::Regulation(record) => Phase2RawEvent::Regulation {
                episode_id: record.episode_id,
                time_us: record.time.as_micros(),
                trigger_neuron_id: record.trigger_neuron_id.0,
                unique_spike_count: record.unique_spike_count,
            },
        })
        .collect();
    Ok(Phase2TrialArtifact {
        response,
        raw_events,
    })
}

fn simulate_protocol_network(
    definition: &NetworkDefinition,
    inputs: &[ExternalInput],
    config: Phase2ProtocolConfig,
) -> Result<NetworkRun, Phase2AnalysisError> {
    Ok(match config.activity_regulator {
        Some(regulator) => {
            simulate_network_with_regulator(definition, inputs, config.base.run_end, regulator)?
        }
        None => simulate_network(definition, inputs, config.base.run_end)?,
    })
}

fn run_trial_metrics(
    definition: &NetworkDefinition,
    config: Phase2ProtocolConfig,
    pattern: Phase2PatternId,
    trial_index: usize,
) -> Result<NetworkMetrics, Phase2AnalysisError> {
    let (inputs, _) = trial_inputs(definition, config, pattern, trial_index)?;
    let run = simulate_protocol_network(definition, &inputs, config)?;
    Ok(compute_network_metrics(
        definition,
        &run,
        metric_config(config.base),
    )?)
}

fn extract_trial_response(
    definition: &NetworkDefinition,
    run: &NetworkRun,
    config: Phase2ProtocolConfig,
    metadata: TrialMetadata,
) -> Result<TrialResponse, Phase2AnalysisError> {
    let candidate_ids = response_candidate_ids(config.base);
    let index_by_id = candidate_ids
        .iter()
        .enumerate()
        .map(|(index, id)| (*id, index))
        .collect::<BTreeMap<_, _>>();
    let duration = config.base.run_end.as_micros() - config.base.pattern_start.as_micros();
    let bin_count = duration.div_ceil(config.analysis_bin_width.as_micros()) as usize;
    let neuron_count = candidate_ids.len();
    let mut spike_counts = vec![vec![0_u16; neuron_count]; bin_count];
    let mut excitatory = vec![vec![0_u16; neuron_count]; bin_count];
    let mut inhibitory = vec![vec![0_u16; neuron_count]; bin_count];
    let mut first_latency = vec![None; neuron_count];

    for spike in run
        .event_log
        .spikes()
        .filter(|spike| spike.time >= config.base.pattern_start && spike.time < config.base.run_end)
    {
        let Some(&neuron) = index_by_id.get(&spike.neuron_id) else {
            continue;
        };
        let bin = ((spike.time.as_micros() - config.base.pattern_start.as_micros())
            / config.analysis_bin_width.as_micros()) as usize;
        spike_counts[bin][neuron] = spike_counts[bin][neuron].saturating_add(1);
        first_latency[neuron].get_or_insert(
            (spike.time.as_micros() - config.base.pattern_start.as_micros()) as f64 / 1_000.0,
        );
    }
    for input in run
        .event_log
        .inputs()
        .filter(|input| input.time >= config.base.pattern_start && input.time < config.base.run_end)
    {
        let Some(&neuron) = index_by_id.get(&input.target) else {
            continue;
        };
        let bin = ((input.time.as_micros() - config.base.pattern_start.as_micros())
            / config.analysis_bin_width.as_micros()) as usize;
        let target = match input.polarity {
            InputPolarity::Excitatory => &mut excitatory[bin][neuron],
            InputPolarity::Inhibitory => &mut inhibitory[bin][neuron],
        };
        *target = target.saturating_add(1);
    }

    let (potentials, refractory) = reconstruct_sampled_states(
        definition,
        run,
        &candidate_ids,
        config.base.pattern_start,
        config.base.run_end,
        config.analysis_bin_width,
    )?;
    let metrics = compute_network_metrics(definition, run, metric_config(config.base))?;
    Ok(TrialResponse {
        metadata,
        neuron_ids: candidate_ids.iter().map(|id| id.0).collect(),
        bin_width_ms: config.analysis_bin_width.as_micros() as f64 / 1_000.0,
        binned_spike_counts: spike_counts,
        sampled_membrane_potential_mv: potentials,
        refractory,
        excitatory_arrival_counts: excitatory,
        inhibitory_arrival_counts: inhibitory,
        first_spike_latency_ms: first_latency,
        regulator_episode_count: run.regulation_episode_count,
        suppressed_propagation_count: run.suppressed_propagation_count,
        metrics,
    })
}

fn reconstruct_sampled_states(
    definition: &NetworkDefinition,
    run: &NetworkRun,
    candidate_ids: &[NeuronId],
    start: SimTime,
    end: SimTime,
    bin_width: SimDuration,
) -> Result<SampledStateMatrices, Phase2AnalysisError> {
    let bin_count = (end.as_micros() - start.as_micros()).div_ceil(bin_width.as_micros()) as usize;
    let mut potentials = vec![vec![0.0_f32; candidate_ids.len()]; bin_count];
    let mut refractory = vec![vec![false; candidate_ids.len()]; bin_count];
    let inputs_by_neuron = run.event_log.inputs().fold(
        BTreeMap::<NeuronId, BTreeMap<SimTime, Vec<_>>>::new(),
        |mut grouped, input| {
            grouped
                .entry(input.target)
                .or_default()
                .entry(input.time)
                .or_default()
                .push(input);
            grouped
        },
    );
    let spec_by_id = definition
        .neurons()
        .iter()
        .map(|neuron| (neuron.id, neuron))
        .collect::<BTreeMap<_, _>>();

    for (column, id) in candidate_ids.iter().enumerate() {
        let spec = spec_by_id[id];
        let mut neuron = LifNeuron::new(
            *id,
            spec.parameters,
            spec.initial_potential_mv,
            definition.start_time(),
        )?;
        let groups = inputs_by_neuron.get(id);
        let mut input_iter = groups
            .into_iter()
            .flat_map(|groups| groups.iter())
            .peekable();
        for bin in 0..bin_count {
            let sample_micros = (start.as_micros()
                + (bin as u64 + 1).saturating_mul(bin_width.as_micros()))
            .min(end.as_micros());
            let sample_time = SimTime::from_micros(sample_micros);
            while input_iter
                .peek()
                .is_some_and(|(time, _)| **time <= sample_time)
            {
                let (time, records) = input_iter.next().expect("peeked input group must exist");
                let inputs = records
                    .iter()
                    .map(|record| {
                        TimedInput::new(
                            record.sequence,
                            record.time,
                            record.polarity,
                            record.magnitude_mv,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                neuron.apply_batch(*time, &inputs)?;
            }
            let snapshot = neuron.advance_to(sample_time)?;
            potentials[bin][column] = snapshot.membrane_potential_mv as f32;
            refractory[bin][column] = snapshot.is_refractory();
        }
    }
    Ok((potentials, refractory))
}

#[derive(Clone, Copy, Debug)]
struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
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

    fn index(&mut self, upper: usize) -> usize {
        (self.next_u64() % upper as u64) as usize
    }

    fn symmetric_i64(&mut self, magnitude: i64) -> i64 {
        if magnitude == 0 {
            return 0;
        }
        let width = magnitude.saturating_mul(2).saturating_add(1) as u64;
        (self.next_u64() % width) as i64 - magnitude
    }
}

#[derive(Clone, Debug)]
struct LabeledSample {
    features: Vec<f64>,
    label: usize,
}

#[derive(Clone, Debug)]
struct Standardizer {
    mean: Vec<f64>,
    scale: Vec<f64>,
}

impl Standardizer {
    fn fit(samples: &[LabeledSample]) -> Result<Self, Phase2AnalysisError> {
        let first = samples.first().ok_or(Phase2AnalysisError::InvalidConfig(
            "cannot fit readout without training samples",
        ))?;
        let dimension = first.features.len();
        let mut mean = vec![0.0; dimension];
        for sample in samples {
            if sample.features.len() != dimension {
                return Err(Phase2AnalysisError::InvalidConfig(
                    "inconsistent response feature dimension",
                ));
            }
            for (target, value) in mean.iter_mut().zip(&sample.features) {
                *target += value;
            }
        }
        for value in &mut mean {
            *value /= samples.len() as f64;
        }
        let mut scale = vec![0.0; dimension];
        for sample in samples {
            for ((target, value), center) in scale.iter_mut().zip(&sample.features).zip(&mean) {
                *target += (value - center).powi(2);
            }
        }
        for (index, value) in scale.iter_mut().enumerate() {
            let minimum_scale = match index % 11 {
                0..=3 => 0.5,
                4..=7 => 0.25,
                8 => 0.05,
                _ => 1.0,
            };
            *value = (*value / samples.len() as f64).sqrt().max(minimum_scale);
        }
        Ok(Self { mean, scale })
    }

    fn transform(&self, features: &[f64]) -> Vec<f64> {
        features
            .iter()
            .zip(&self.mean)
            .zip(&self.scale)
            .map(|((value, mean), scale)| (value - mean) / scale)
            .collect()
    }

    fn transform_samples(&self, samples: &[LabeledSample]) -> Vec<LabeledSample> {
        samples
            .iter()
            .map(|sample| LabeledSample {
                features: self.transform(&sample.features),
                label: sample.label,
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
struct PrototypeReadout {
    centroids: Vec<Vec<f64>>,
}

impl PrototypeReadout {
    fn train(samples: &[LabeledSample], labels: &[usize], class_count: usize) -> Self {
        let dimension = samples[0].features.len();
        let mut centroids = vec![vec![0.0; dimension]; class_count];
        let mut counts = vec![0_usize; class_count];
        for (sample, &label) in samples.iter().zip(labels) {
            counts[label] += 1;
            for (target, value) in centroids[label].iter_mut().zip(&sample.features) {
                *target += value;
            }
        }
        for (centroid, count) in centroids.iter_mut().zip(counts) {
            if count > 0 {
                for value in centroid {
                    *value /= count as f64;
                }
            }
        }
        Self { centroids }
    }

    fn predict(&self, features: &[f64]) -> usize {
        self.centroids
            .iter()
            .enumerate()
            .min_by(|(_, left), (_, right)| {
                squared_distance(features, left).total_cmp(&squared_distance(features, right))
            })
            .map(|(label, _)| label)
            .unwrap_or(0)
    }
}

#[derive(Clone, Debug)]
struct LinearReadout {
    weights: Vec<Vec<f64>>,
    bias: Vec<f64>,
}

impl LinearReadout {
    fn train(samples: &[LabeledSample], class_count: usize) -> Self {
        let dimension = samples[0].features.len();
        let mut weights = vec![vec![0.0; dimension]; class_count];
        let mut bias = vec![0.0; class_count];
        let epochs = 180;
        let regularization = 0.001;
        for epoch in 0..epochs {
            let learning_rate = 0.18 / (1.0 + epoch as f64 / 45.0);
            let mut weight_gradient = vec![vec![0.0; dimension]; class_count];
            let mut bias_gradient = vec![0.0; class_count];
            for sample in samples {
                let probabilities = softmax_scores(&weights, &bias, &sample.features);
                for class in 0..class_count {
                    let error = probabilities[class] - f64::from(class == sample.label);
                    bias_gradient[class] += error;
                    for (gradient, feature) in
                        weight_gradient[class].iter_mut().zip(&sample.features)
                    {
                        *gradient += error * feature;
                    }
                }
            }
            let inverse_count = 1.0 / samples.len() as f64;
            for class in 0..class_count {
                bias[class] -= learning_rate * bias_gradient[class] * inverse_count;
                for dimension_index in 0..dimension {
                    let gradient = weight_gradient[class][dimension_index] * inverse_count
                        + regularization * weights[class][dimension_index];
                    weights[class][dimension_index] -= learning_rate * gradient;
                }
            }
        }
        Self { weights, bias }
    }

    fn predict(&self, features: &[f64]) -> usize {
        self.weights
            .iter()
            .zip(&self.bias)
            .enumerate()
            .map(|(class, (weights, bias))| {
                let score = weights
                    .iter()
                    .zip(features)
                    .map(|(weight, feature)| weight * feature)
                    .sum::<f64>()
                    + bias;
                (class, score)
            })
            .max_by(|left, right| left.1.total_cmp(&right.1))
            .map(|(class, _)| class)
            .unwrap_or(0)
    }

    fn neuron_importance(&self, class: usize, neuron_index: usize) -> f64 {
        let start = neuron_index * 11;
        self.weights[class][start..start + 11]
            .iter()
            .map(|weight| weight.abs())
            .sum()
    }
}

#[derive(Clone, Debug)]
struct TrainedReadouts {
    standardizer: Standardizer,
    linear: LinearReadout,
    report: ReadoutReport,
}

fn softmax_scores(weights: &[Vec<f64>], bias: &[f64], features: &[f64]) -> Vec<f64> {
    let mut scores = weights
        .iter()
        .zip(bias)
        .map(|(weights, bias)| {
            weights
                .iter()
                .zip(features)
                .map(|(weight, feature)| weight * feature)
                .sum::<f64>()
                + bias
        })
        .collect::<Vec<_>>();
    let maximum = scores.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    for score in &mut scores {
        *score = (*score - maximum).exp();
    }
    let total = scores.iter().sum::<f64>().max(f64::MIN_POSITIVE);
    for score in &mut scores {
        *score /= total;
    }
    scores
}

fn squared_distance(left: &[f64], right: &[f64]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(left, right)| (left - right).powi(2))
        .sum()
}

fn accuracy_report(
    predictions: &[usize],
    samples: &[LabeledSample],
    class_count: usize,
) -> AccuracyReport {
    let mut correct_by_class = vec![0_usize; class_count];
    let mut total_by_class = vec![0_usize; class_count];
    let correct = predictions
        .iter()
        .zip(samples)
        .filter(|(prediction, sample)| {
            total_by_class[sample.label] += 1;
            if **prediction == sample.label {
                correct_by_class[sample.label] += 1;
                true
            } else {
                false
            }
        })
        .count();
    let total = samples.len();
    let accuracy = correct as f64 / total.max(1) as f64;
    let (confidence_low, confidence_high) = wilson_interval(correct, total);
    AccuracyReport {
        accuracy,
        correct,
        total,
        chance_accuracy: 1.0 / class_count as f64,
        confidence_low,
        confidence_high,
        per_class_accuracy: correct_by_class
            .into_iter()
            .zip(total_by_class)
            .map(|(correct, total)| correct as f64 / total.max(1) as f64)
            .collect(),
    }
}

fn wilson_interval(correct: usize, total: usize) -> (f64, f64) {
    if total == 0 {
        return (0.0, 0.0);
    }
    let z = 1.959_963_984_540_054;
    let n = total as f64;
    let p = correct as f64 / n;
    let denominator = 1.0 + z * z / n;
    let center = (p + z * z / (2.0 * n)) / denominator;
    let margin = z * ((p * (1.0 - p) + z * z / (4.0 * n)) / n).sqrt() / denominator;
    ((center - margin).max(0.0), (center + margin).min(1.0))
}

fn labeled_samples(
    trials: &[TrialResponse],
    split: TrialSplit,
    stimulus_bin_count: usize,
) -> Vec<LabeledSample> {
    trials
        .iter()
        .filter(|trial| trial.metadata.split == split)
        .filter_map(|trial| {
            trial
                .metadata
                .pattern
                .class_index()
                .map(|label| LabeledSample {
                    features: trial.feature_vector(stimulus_bin_count),
                    label,
                })
        })
        .collect()
}

fn train_and_evaluate_readouts(
    trials: &[TrialResponse],
    config: Phase2ProtocolConfig,
    permutation_seed: u64,
) -> Result<TrainedReadouts, Phase2AnalysisError> {
    let stimulus_bin_count = (config.base.pattern_end.as_micros()
        - config.base.pattern_start.as_micros())
    .div_ceil(config.analysis_bin_width.as_micros()) as usize;
    let train_raw = labeled_samples(trials, TrialSplit::Train, stimulus_bin_count);
    let test_raw = labeled_samples(trials, TrialSplit::Test, stimulus_bin_count);
    if train_raw.is_empty() || test_raw.is_empty() {
        return Err(Phase2AnalysisError::InvalidConfig(
            "readout requires non-empty train and test splits",
        ));
    }
    let standardizer = Standardizer::fit(&train_raw)?;
    let train = standardizer.transform_samples(&train_raw);
    let test = standardizer.transform_samples(&test_raw);
    let labels = train.iter().map(|sample| sample.label).collect::<Vec<_>>();
    let prototype = PrototypeReadout::train(&train, &labels, Phase2PatternId::PRIMARY.len());
    let linear = LinearReadout::train(&train, Phase2PatternId::PRIMARY.len());
    let prototype_predictions = test
        .iter()
        .map(|sample| prototype.predict(&sample.features))
        .collect::<Vec<_>>();
    let linear_predictions = test
        .iter()
        .map(|sample| linear.predict(&sample.features))
        .collect::<Vec<_>>();
    let prototype_report = accuracy_report(
        &prototype_predictions,
        &test,
        Phase2PatternId::PRIMARY.len(),
    );
    let linear_report = accuracy_report(&linear_predictions, &test, Phase2PatternId::PRIMARY.len());
    let (within_class_distance, between_class_distance) = class_distances(&test);

    let mut random = DeterministicRng::new(permutation_seed);
    let mut permutation_accuracies = Vec::with_capacity(config.label_permutation_count);
    for _ in 0..config.label_permutation_count {
        let mut permuted_labels = labels.clone();
        fisher_yates(&mut permuted_labels, &mut random);
        let permuted =
            PrototypeReadout::train(&train, &permuted_labels, Phase2PatternId::PRIMARY.len());
        let predictions = test
            .iter()
            .map(|sample| permuted.predict(&sample.features))
            .collect::<Vec<_>>();
        permutation_accuracies
            .push(accuracy_report(&predictions, &test, Phase2PatternId::PRIMARY.len()).accuracy);
    }
    let observed = prototype_report.accuracy.max(linear_report.accuracy);
    let exceeded = permutation_accuracies
        .iter()
        .filter(|accuracy| **accuracy >= observed)
        .count();
    let permutation_p_value = (exceeded + 1) as f64 / (config.label_permutation_count + 1) as f64;
    let report = ReadoutReport {
        prototype: prototype_report,
        linear: linear_report,
        label_permutation_mean_accuracy: permutation_accuracies.iter().sum::<f64>()
            / permutation_accuracies.len().max(1) as f64,
        label_permutation_max_accuracy: permutation_accuracies.iter().copied().fold(0.0, f64::max),
        permutation_p_value,
        within_class_distance,
        between_class_distance,
        separation_ratio: between_class_distance / within_class_distance.max(f64::EPSILON),
    };
    Ok(TrainedReadouts {
        standardizer,
        linear,
        report,
    })
}

fn class_distances(samples: &[LabeledSample]) -> (f64, f64) {
    let mut within_total = 0.0;
    let mut within_count = 0_usize;
    let mut between_total = 0.0;
    let mut between_count = 0_usize;
    for left in 0..samples.len() {
        for right in left + 1..samples.len() {
            let distance =
                squared_distance(&samples[left].features, &samples[right].features).sqrt();
            if samples[left].label == samples[right].label {
                within_total += distance;
                within_count += 1;
            } else {
                between_total += distance;
                between_count += 1;
            }
        }
    }
    (
        within_total / within_count.max(1) as f64,
        between_total / between_count.max(1) as f64,
    )
}

fn fisher_yates(values: &mut [usize], random: &mut DeterministicRng) {
    for index in (1..values.len()).rev() {
        values.swap(index, random.index(index + 1));
    }
}

pub fn silence_neurons(
    definition: &NetworkDefinition,
    silenced: &BTreeSet<NeuronId>,
) -> Result<NetworkDefinition, Phase2AnalysisError> {
    let neurons = definition.neurons().to_vec();
    let synapses = definition
        .synapses()
        .iter()
        .filter(|synapse| {
            !silenced.contains(&synapse.source) && !silenced.contains(&synapse.target)
        })
        .cloned()
        .collect();
    Ok(NetworkDefinition::new(
        definition.start_time(),
        neurons,
        synapses,
    )?)
}

pub fn degree_preserving_connection_shuffle(
    definition: &NetworkDefinition,
    seed: u64,
    swap_attempts: usize,
) -> Result<NetworkDefinition, Phase2AnalysisError> {
    let mut synapses = definition.synapses().to_vec();
    if synapses.len() < 2 {
        return Ok(definition.clone());
    }
    let mut edges = synapses
        .iter()
        .map(|synapse| (synapse.source, synapse.target))
        .collect::<BTreeSet<_>>();
    let mut random = DeterministicRng::new(seed);
    for _ in 0..swap_attempts {
        let left = random.index(synapses.len());
        let mut right = random.index(synapses.len());
        if left == right {
            right = (right + 1) % synapses.len();
        }
        let left_edge = (synapses[left].source, synapses[left].target);
        let right_edge = (synapses[right].source, synapses[right].target);
        let proposed_left = (left_edge.0, right_edge.1);
        let proposed_right = (right_edge.0, left_edge.1);
        if proposed_left.0 == proposed_left.1
            || proposed_right.0 == proposed_right.1
            || proposed_left == proposed_right
        {
            continue;
        }
        edges.remove(&left_edge);
        edges.remove(&right_edge);
        if edges.contains(&proposed_left) || edges.contains(&proposed_right) {
            edges.insert(left_edge);
            edges.insert(right_edge);
            continue;
        }
        synapses[left].target = proposed_left.1;
        synapses[right].target = proposed_right.1;
        edges.insert(proposed_left);
        edges.insert(proposed_right);
    }
    Ok(NetworkDefinition::new(
        definition.start_time(),
        definition.neurons().to_vec(),
        synapses,
    )?)
}

fn collect_seed_trials(
    definition: &NetworkDefinition,
    config: Phase2ProtocolConfig,
    structure_seed: u64,
) -> Result<Vec<TrialResponse>, Phase2AnalysisError> {
    let capacity = config.trials_per_pattern * (Phase2PatternId::PRIMARY.len() + 1)
        + config.single_pixel_trials * Phase2PatternId::SINGLE_PIXELS.len();
    let mut trials = Vec::with_capacity(capacity);
    for trial_index in 0..config.trials_per_pattern {
        let split = trial_split(config, trial_index);
        trials.push(run_trial(
            definition,
            config,
            structure_seed,
            Phase2PatternId::Blank,
            trial_index,
            split,
        )?);
        for pattern in Phase2PatternId::PRIMARY {
            trials.push(run_trial(
                definition,
                config,
                structure_seed,
                pattern,
                trial_index,
                split,
            )?);
        }
    }
    for single_trial in 0..config.single_pixel_trials {
        let trial_index = single_trial * config.trials_per_pattern / config.single_pixel_trials;
        for pattern in Phase2PatternId::SINGLE_PIXELS {
            trials.push(run_trial(
                definition,
                config,
                structure_seed,
                pattern,
                trial_index,
                TrialSplit::Control,
            )?);
        }
    }
    Ok(trials)
}

fn collect_primary_trials(
    definition: &NetworkDefinition,
    config: Phase2ProtocolConfig,
    structure_seed: u64,
) -> Result<Vec<TrialResponse>, Phase2AnalysisError> {
    let mut trials = Vec::with_capacity(config.trials_per_pattern * Phase2PatternId::PRIMARY.len());
    for trial_index in 0..config.trials_per_pattern {
        for pattern in Phase2PatternId::PRIMARY {
            trials.push(run_trial(
                definition,
                config,
                structure_seed,
                pattern,
                trial_index,
                trial_split(config, trial_index),
            )?);
        }
    }
    Ok(trials)
}

fn select_stable_structure_seeds(
    config: Phase2ProtocolConfig,
    criteria: Phase2StabilityCriteria,
) -> Result<(Vec<u64>, usize), Phase2AnalysisError> {
    let mut selected = Vec::with_capacity(config.structure_seed_count);
    let mut rejected = 0_usize;
    for candidate in 0..config.structure_seed_scan_limit {
        let seed = config
            .base
            .network
            .seed
            .wrapping_add((candidate as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15));
        let mut candidate_config = config;
        candidate_config.base.network.seed = seed;
        let definition = generate_network(candidate_config.base.network)?;
        let mut metrics = Vec::with_capacity(config.structure_seed_screen_trials);
        for trial_index in 0..config.structure_seed_screen_trials {
            metrics.push(run_trial_metrics(
                &definition,
                candidate_config,
                Phase2PatternId::Blank,
                trial_index,
            )?);
        }
        let passed = metrics
            .iter()
            .filter(|metrics| criteria.accepts(metrics))
            .count();
        let screen = StructureSeedScreen {
            seed,
            accepted: passed == config.structure_seed_screen_trials,
            pass_fraction: passed as f64 / config.structure_seed_screen_trials as f64,
            metrics: metrics.remove(0),
        };
        if screen.accepted {
            selected.push(screen.seed);
            if selected.len() == config.structure_seed_count {
                return Ok((selected, rejected));
            }
        } else {
            rejected += 1;
        }
    }
    Err(Phase2AnalysisError::InvalidConfig(
        "not enough stable structure seeds were found within the scan limit",
    ))
}

pub fn screen_phase2_structure_seeds(
    config: Phase2ProtocolConfig,
    criteria: Phase2StabilityCriteria,
) -> Result<Vec<StructureSeedScreen>, Phase2AnalysisError> {
    let mut screens = Vec::with_capacity(config.structure_seed_scan_limit);
    for candidate in 0..config.structure_seed_scan_limit {
        let seed = config
            .base
            .network
            .seed
            .wrapping_add((candidate as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15));
        let mut candidate_config = config;
        candidate_config.base.network.seed = seed;
        let definition = generate_network(candidate_config.base.network)?;
        let mut metrics = Vec::with_capacity(config.structure_seed_screen_trials);
        for trial_index in 0..config.structure_seed_screen_trials {
            metrics.push(run_trial_metrics(
                &definition,
                candidate_config,
                Phase2PatternId::Blank,
                trial_index,
            )?);
        }
        let passed = metrics
            .iter()
            .filter(|metrics| criteria.accepts(metrics))
            .count();
        screens.push(StructureSeedScreen {
            seed,
            accepted: passed == config.structure_seed_screen_trials,
            pass_fraction: passed as f64 / config.structure_seed_screen_trials as f64,
            metrics: metrics.remove(0),
        });
    }
    Ok(screens)
}

pub fn evaluate_phase2_input_stability(
    config: Phase2ProtocolConfig,
    structure_seeds: &[u64],
    trials_per_pattern: usize,
) -> Result<InputStabilityEvaluation, Phase2AnalysisError> {
    if structure_seeds.is_empty() || trials_per_pattern == 0 {
        return Err(Phase2AnalysisError::InvalidConfig(
            "input stability scan requires seeds and trials",
        ));
    }
    let criteria = Phase2StabilityCriteria::input_driven();
    let mut metrics = Vec::new();
    for &seed in structure_seeds {
        let mut seed_config = config;
        seed_config.base.network.seed = seed;
        let definition = generate_network(seed_config.base.network)?;
        for trial_index in 0..trials_per_pattern {
            for pattern in Phase2PatternId::PRIMARY {
                metrics.push(run_trial_metrics(
                    &definition,
                    seed_config,
                    pattern,
                    trial_index,
                )?);
            }
        }
    }
    let passed = metrics
        .iter()
        .filter(|metrics| criteria.accepts(metrics))
        .count();
    let severe_instability_count = metrics
        .iter()
        .filter(|metrics| {
            metrics.mean_firing_rate_hz > 20.0
                || metrics.active_neuron_fraction > 0.85
                || metrics.synchronous_burst_bin_fraction > 0.30
        })
        .count();
    Ok(InputStabilityEvaluation {
        pattern_magnitude_mv: config.base.pattern_magnitude_mv,
        pattern_interval_ms: config.base.pattern_interval.as_micros() as f64 / 1_000.0,
        input_jitter_ms: config.input_jitter.as_micros() as f64 / 1_000.0,
        trial_count: metrics.len(),
        pass_fraction: passed as f64 / metrics.len() as f64,
        severe_instability_count,
        mean_firing_rate_hz: metrics
            .iter()
            .map(|metrics| metrics.mean_firing_rate_hz)
            .sum::<f64>()
            / metrics.len() as f64,
        mean_active_fraction: metrics
            .iter()
            .map(|metrics| metrics.active_neuron_fraction)
            .sum::<f64>()
            / metrics.len() as f64,
        mean_synchronous_fraction: metrics
            .iter()
            .map(|metrics| metrics.synchronous_burst_bin_fraction)
            .sum::<f64>()
            / metrics.len() as f64,
    })
}

fn stability_summary(trials: &[TrialResponse], criteria: Phase2StabilityCriteria) -> (f64, usize) {
    let passed = trials
        .iter()
        .filter(|trial| criteria.accepts(&trial.metrics))
        .count();
    let severe = trials
        .iter()
        .filter(|trial| {
            trial.metrics.mean_firing_rate_hz > 20.0
                || trial.metrics.active_neuron_fraction > 0.85
                || trial.metrics.synchronous_burst_bin_fraction > 0.30
        })
        .count();
    (passed as f64 / trials.len().max(1) as f64, severe)
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len().max(1) as f64
}

fn standard_deviation(values: &[f64]) -> f64 {
    let center = mean(values);
    (values
        .iter()
        .map(|value| (value - center).powi(2))
        .sum::<f64>()
        / values.len().max(1) as f64)
        .sqrt()
}

fn pearson_correlation(left: &[f64], right: &[f64]) -> f64 {
    if left.len() != right.len() || left.is_empty() {
        return 0.0;
    }
    let left_mean = mean(left);
    let right_mean = mean(right);
    let covariance = left
        .iter()
        .zip(right)
        .map(|(left, right)| (left - left_mean) * (right - right_mean))
        .sum::<f64>();
    let left_scale = left
        .iter()
        .map(|value| (value - left_mean).powi(2))
        .sum::<f64>()
        .sqrt();
    let right_scale = right
        .iter()
        .map(|value| (value - right_mean).powi(2))
        .sum::<f64>()
        .sqrt();
    covariance / (left_scale * right_scale).max(f64::EPSILON)
}

fn discover_functional_groups(
    definition: &NetworkDefinition,
    trials: &[TrialResponse],
    readouts: &TrainedReadouts,
    config: Phase2ProtocolConfig,
) -> Result<Vec<FunctionalGroupReport>, Phase2AnalysisError> {
    let first = trials.first().ok_or(Phase2AnalysisError::InvalidConfig(
        "functional group discovery requires trial responses",
    ))?;
    let neuron_ids = first.neuron_ids.clone();
    let polarity_by_id = definition
        .neurons()
        .iter()
        .map(|neuron| (neuron.id.0, neuron.polarity))
        .collect::<BTreeMap<_, _>>();
    let blank_by_trial = trials
        .iter()
        .filter(|trial| {
            trial.metadata.split == TrialSplit::Train
                && trial.metadata.pattern == Phase2PatternId::Blank
        })
        .map(|trial| (trial.metadata.trial_index, trial.total_spikes_by_neuron()))
        .collect::<BTreeMap<_, _>>();
    let mut groups = Vec::new();

    for pattern in Phase2PatternId::PRIMARY {
        let class = pattern.class_index().expect("primary pattern has a class");
        let class_trials = trials
            .iter()
            .filter(|trial| {
                trial.metadata.split == TrialSplit::Train && trial.metadata.pattern == pattern
            })
            .collect::<Vec<_>>();
        let other_trials = trials
            .iter()
            .filter(|trial| {
                trial.metadata.split == TrialSplit::Train
                    && trial.metadata.pattern.class_index().is_some()
                    && trial.metadata.pattern != pattern
            })
            .collect::<Vec<_>>();
        let class_counts = class_trials
            .iter()
            .map(|trial| trial.total_spikes_by_neuron())
            .collect::<Vec<_>>();
        let other_counts = other_trials
            .iter()
            .map(|trial| trial.total_spikes_by_neuron())
            .collect::<Vec<_>>();
        let population_delta = class_trials
            .iter()
            .zip(&class_counts)
            .map(|(trial, counts)| {
                let blank = &blank_by_trial[&trial.metadata.trial_index];
                counts.iter().sum::<f64>() - blank.iter().sum::<f64>()
            })
            .collect::<Vec<_>>();
        let mut scores = Vec::with_capacity(neuron_ids.len());
        for (neuron, &neuron_id) in neuron_ids.iter().enumerate() {
            let deltas = class_trials
                .iter()
                .zip(&class_counts)
                .map(|(trial, counts)| {
                    counts[neuron] - blank_by_trial[&trial.metadata.trial_index][neuron]
                })
                .collect::<Vec<_>>();
            let response_delta = mean(&deltas);
            let response_std = standard_deviation(&deltas);
            let direction = response_delta.signum();
            let reliability = deltas
                .iter()
                .filter(|delta| delta.signum() == direction || delta.abs() < f64::EPSILON)
                .count() as f64
                / deltas.len().max(1) as f64;
            let class_mean = mean(
                &class_counts
                    .iter()
                    .map(|counts| counts[neuron])
                    .collect::<Vec<_>>(),
            );
            let other_values = other_counts
                .iter()
                .map(|counts| counts[neuron])
                .collect::<Vec<_>>();
            let selectivity = (class_mean - mean(&other_values)).abs()
                / (standard_deviation(&other_values) + 1.0);
            let latencies = class_trials
                .iter()
                .filter_map(|trial| trial.first_spike_latency_ms[neuron])
                .collect::<Vec<_>>();
            let latency_std_ms = standard_deviation(&latencies);
            let coactivity_correlation = pearson_correlation(&deltas, &population_delta);
            let readout_importance = readouts.linear.neuron_importance(class, neuron);
            let raw_score = response_delta.abs() / (response_std + 1.0)
                * reliability
                * (1.0 + selectivity)
                * (1.0 + coactivity_correlation.max(0.0))
                * (1.0 + readout_importance);
            scores.push(FunctionalNeuronScore {
                neuron_id,
                polarity: match polarity_by_id[&neuron_id] {
                    NeuronPolarity::Excitatory => "excitatory".to_owned(),
                    NeuronPolarity::Inhibitory => "inhibitory".to_owned(),
                },
                response_delta,
                reliability,
                selectivity,
                latency_std_ms,
                coactivity_correlation,
                readout_importance,
                score: raw_score,
            });
        }
        scores.sort_by(|left, right| {
            right
                .score
                .total_cmp(&left.score)
                .then_with(|| left.neuron_id.cmp(&right.neuron_id))
        });
        scores.truncate(config.functional_group_size.min(scores.len()));
        groups.push(FunctionalGroupReport {
            id: format!("F{class}"),
            pattern,
            members: scores,
        });
    }
    Ok(groups)
}

fn matched_control_group(
    definition: &NetworkDefinition,
    trials: &[TrialResponse],
    group: &FunctionalGroupReport,
    seed: u64,
) -> Vec<NeuronId> {
    let group_ids = group
        .members
        .iter()
        .map(|member| NeuronId(member.neuron_id))
        .collect::<BTreeSet<_>>();
    let blank_trials = trials
        .iter()
        .filter(|trial| {
            trial.metadata.split == TrialSplit::Train
                && trial.metadata.pattern == Phase2PatternId::Blank
        })
        .collect::<Vec<_>>();
    let neuron_ids = blank_trials[0]
        .neuron_ids
        .iter()
        .copied()
        .map(NeuronId)
        .collect::<Vec<_>>();
    let rates = (0..neuron_ids.len())
        .map(|neuron| {
            mean(
                &blank_trials
                    .iter()
                    .map(|trial| trial.total_spikes_by_neuron()[neuron])
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    let polarity = definition
        .neurons()
        .iter()
        .map(|neuron| (neuron.id, neuron.polarity))
        .collect::<BTreeMap<_, _>>();
    let index_by_id = neuron_ids
        .iter()
        .enumerate()
        .map(|(index, id)| (*id, index))
        .collect::<BTreeMap<_, _>>();
    let mut random = DeterministicRng::new(seed);
    let mut selected = BTreeSet::new();
    for member in &group.members {
        let member_id = NeuronId(member.neuron_id);
        let member_rate = rates[index_by_id[&member_id]];
        let mut candidates = neuron_ids
            .iter()
            .copied()
            .filter(|candidate| !group_ids.contains(candidate) && !selected.contains(candidate))
            .filter(|candidate| polarity[candidate] == polarity[&member_id])
            .map(|candidate| {
                let difference = (rates[index_by_id[&candidate]] - member_rate).abs();
                (difference, candidate)
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| {
            left.0
                .total_cmp(&right.0)
                .then_with(|| left.1.cmp(&right.1))
        });
        let pool = candidates.len().min(5);
        if pool > 0 {
            selected.insert(candidates[random.index(pool)].1);
        }
    }
    selected.into_iter().collect()
}

fn evaluate_linear_on_trials(
    readouts: &TrainedReadouts,
    trials: &[TrialResponse],
    config: Phase2ProtocolConfig,
) -> AccuracyReport {
    let stimulus_bin_count = (config.base.pattern_end.as_micros()
        - config.base.pattern_start.as_micros())
    .div_ceil(config.analysis_bin_width.as_micros()) as usize;
    let raw = labeled_samples(trials, TrialSplit::Test, stimulus_bin_count);
    let samples = readouts.standardizer.transform_samples(&raw);
    let predictions = samples
        .iter()
        .map(|sample| readouts.linear.predict(&sample.features))
        .collect::<Vec<_>>();
    accuracy_report(&predictions, &samples, Phase2PatternId::PRIMARY.len())
}

fn run_test_trials(
    definition: &NetworkDefinition,
    config: Phase2ProtocolConfig,
    structure_seed: u64,
) -> Result<Vec<TrialResponse>, Phase2AnalysisError> {
    let mut trials = Vec::new();
    for trial_index in config.training_trials_per_pattern..config.trials_per_pattern {
        for pattern in Phase2PatternId::PRIMARY {
            trials.push(run_trial(
                definition,
                config,
                structure_seed,
                pattern,
                trial_index,
                TrialSplit::Test,
            )?);
        }
    }
    Ok(trials)
}

fn validate_ablation_groups(
    definition: &NetworkDefinition,
    trials: &[TrialResponse],
    groups: &[FunctionalGroupReport],
    readouts: &TrainedReadouts,
    config: Phase2ProtocolConfig,
    criteria: Phase2StabilityCriteria,
    structure_seed: u64,
) -> Result<Vec<AblationComparison>, Phase2AnalysisError> {
    let mut reports = Vec::new();
    for (group_index, group) in groups.iter().enumerate() {
        let candidate_ids = group
            .members
            .iter()
            .map(|member| NeuronId(member.neuron_id))
            .collect::<BTreeSet<_>>();
        let matched_ids = matched_control_group(
            definition,
            trials,
            group,
            structure_seed ^ group_index as u64 ^ 0x4142_4c41_5445,
        );
        let matched_set = matched_ids.iter().copied().collect::<BTreeSet<_>>();
        let candidate_definition = silence_neurons(definition, &candidate_ids)?;
        let matched_definition = silence_neurons(definition, &matched_set)?;
        let candidate_trials = run_test_trials(&candidate_definition, config, structure_seed)?;
        let matched_trials = run_test_trials(&matched_definition, config, structure_seed)?;
        let candidate_report = evaluate_linear_on_trials(readouts, &candidate_trials, config);
        let matched_report = evaluate_linear_on_trials(readouts, &matched_trials, config);
        let intact = readouts.report.linear.clone();
        let class = group
            .pattern
            .class_index()
            .expect("functional group pattern has class");
        let target_accuracy_drop =
            intact.per_class_accuracy[class] - candidate_report.per_class_accuracy[class];
        let matched_target_accuracy_drop =
            intact.per_class_accuracy[class] - matched_report.per_class_accuracy[class];
        let non_target_accuracy_drop = (0..Phase2PatternId::PRIMARY.len())
            .filter(|candidate_class| *candidate_class != class)
            .map(|candidate_class| {
                intact.per_class_accuracy[candidate_class]
                    - candidate_report.per_class_accuracy[candidate_class]
            })
            .sum::<f64>()
            / (Phase2PatternId::PRIMARY.len() - 1) as f64;
        let (candidate_stability_pass_fraction, _) = stability_summary(&candidate_trials, criteria);
        let (matched_stability_pass_fraction, _) = stability_summary(&matched_trials, criteria);
        reports.push(AblationComparison {
            pattern: group.pattern,
            candidate_group_ids: candidate_ids.iter().map(|id| id.0).collect(),
            matched_group_ids: matched_ids.iter().map(|id| id.0).collect(),
            intact,
            candidate_ablation: candidate_report,
            matched_ablation: matched_report,
            candidate_stability_pass_fraction,
            matched_stability_pass_fraction,
            target_accuracy_drop,
            matched_target_accuracy_drop,
            non_target_accuracy_drop,
            selective_effect: target_accuracy_drop >= matched_target_accuracy_drop + 0.05
                && target_accuracy_drop > non_target_accuracy_drop,
        });
    }
    Ok(reports)
}

fn validate_protocol_config(config: Phase2ProtocolConfig) -> Result<(), Phase2AnalysisError> {
    if config.structure_seed_count == 0
        || config.structure_seed_scan_limit < config.structure_seed_count
        || config.structure_seed_screen_trials == 0
        || config.trials_per_pattern < 4
        || config.training_trials_per_pattern == 0
        || config.training_trials_per_pattern >= config.trials_per_pattern
        || config.single_pixel_trials == 0
        || config.analysis_bin_width.as_micros() == 0
        || config.functional_group_size == 0
        || config.label_permutation_count == 0
    {
        return Err(Phase2AnalysisError::InvalidConfig(
            "invalid phase 2 protocol dimensions",
        ));
    }
    if config.base.pattern_end >= config.base.run_end
        || config.base.pattern_start >= config.base.pattern_end
        || config.input_jitter.as_micros() >= config.base.pattern_interval.as_micros()
    {
        return Err(Phase2AnalysisError::InvalidConfig(
            "invalid phase 2 protocol timing",
        ));
    }
    Ok(())
}

pub fn run_phase2_protocol(
    config: Phase2ProtocolConfig,
) -> Result<Phase2ProtocolResult, Phase2AnalysisError> {
    validate_protocol_config(config)?;
    let baseline_criteria = Phase2StabilityCriteria::default();
    let input_criteria = Phase2StabilityCriteria::input_driven();
    let (selected_structure_seeds, rejected_structure_seed_count) =
        select_stable_structure_seeds(config, baseline_criteria)?;
    let stability_trials = config.structure_seed_screen_trials;
    let regulated_input_stability =
        evaluate_phase2_input_stability(config, &selected_structure_seeds, stability_trials)?;
    let unregulated_input_stability = evaluate_phase2_input_stability(
        Phase2ProtocolConfig {
            activity_regulator: None,
            ..config
        },
        &selected_structure_seeds,
        stability_trials,
    )?;
    let mut all_trials = Vec::new();
    let mut seed_reports = Vec::new();

    for &structure_seed in &selected_structure_seeds {
        let mut seed_config = config;
        seed_config.base.network.seed = structure_seed;
        let definition = generate_network(seed_config.base.network)?;
        let trials = collect_seed_trials(&definition, seed_config, structure_seed)?;
        let (stability_pass_fraction, severe_instability_count) =
            stability_summary(&trials, input_criteria);
        let readouts = train_and_evaluate_readouts(
            &trials,
            seed_config,
            structure_seed ^ 0x5045_524d_5554_4501,
        )?;
        let groups = discover_functional_groups(&definition, &trials, &readouts, seed_config)?;

        let shuffled_definition = degree_preserving_connection_shuffle(
            &definition,
            structure_seed ^ 0x5348_5546_464c_4501,
            definition
                .synapses()
                .len()
                .saturating_mul(config.connection_swap_multiplier),
        )?;
        let shuffled_trials =
            collect_primary_trials(&shuffled_definition, seed_config, structure_seed)?;
        let shuffled_readout = train_and_evaluate_readouts(
            &shuffled_trials,
            seed_config,
            structure_seed ^ 0x5348_5546_5045_524d,
        )?;
        let ablations = validate_ablation_groups(
            &definition,
            &trials,
            &groups,
            &readouts,
            seed_config,
            input_criteria,
            structure_seed,
        )?;
        seed_reports.push(SeedExperimentReport {
            structure_seed,
            stability_pass_fraction,
            severe_instability_count,
            mean_regulator_episode_count: trials
                .iter()
                .map(|trial| trial.regulator_episode_count as f64)
                .sum::<f64>()
                / trials.len().max(1) as f64,
            mean_suppressed_propagation_count: trials
                .iter()
                .map(|trial| trial.suppressed_propagation_count as f64)
                .sum::<f64>()
                / trials.len().max(1) as f64,
            readout: readouts.report,
            shuffled_connectivity_readout: shuffled_readout.report,
            functional_groups: groups,
            ablations,
        });
        all_trials.extend(trials);
    }

    let h1_reproducible_response = seed_reports
        .iter()
        .all(|seed| seed.readout.separation_ratio > 1.0);
    let h2_decodable_information = seed_reports.iter().all(|seed| {
        let best = seed
            .readout
            .prototype
            .accuracy
            .max(seed.readout.linear.accuracy);
        best > seed.readout.prototype.chance_accuracy && seed.readout.permutation_p_value <= 0.05
    });
    let h3_causal_functional_groups = seed_reports.iter().all(|seed| {
        seed.ablations
            .iter()
            .filter(|ablation| ablation.selective_effect)
            .count()
            >= Phase2PatternId::PRIMARY.len() / 2
    });
    let report = Phase2ProtocolReport {
        version: "phase2/v1".to_owned(),
        selected_structure_seeds: selected_structure_seeds.clone(),
        rejected_structure_seed_count,
        trials_per_pattern_per_seed: config.trials_per_pattern,
        training_trials_per_pattern_per_seed: config.training_trials_per_pattern,
        input_neuron_ids: config.base.input_neuron_ids.iter().map(|id| id.0).collect(),
        pattern_magnitude_mv: config.base.pattern_magnitude_mv,
        pattern_interval_ms: config.base.pattern_interval.as_micros() as f64 / 1_000.0,
        input_jitter_ms: config.input_jitter.as_micros() as f64 / 1_000.0,
        activity_regulator: config.activity_regulator.map(ActivityRegulatorReport::from),
        patterns: Phase2PatternId::PRIMARY.to_vec(),
        baseline_stability_criteria: baseline_criteria,
        input_stability_criteria: input_criteria,
        regulated_input_stability,
        unregulated_input_stability,
        seeds: seed_reports,
        hypotheses: Phase2HypothesisReport {
            h1_reproducible_response,
            h2_decodable_information,
            h3_causal_functional_groups,
        },
    };
    Ok(Phase2ProtocolResult {
        report,
        trials: all_trials,
    })
}

#[derive(Debug)]
pub enum Phase2AnalysisError {
    InvalidConfig(&'static str),
    Experiment(ExperimentError),
    Network(NetworkError),
    Metrics(MetricsError),
    Model(ModelError),
}

impl Display for Phase2AnalysisError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(message) => formatter.write_str(message),
            Self::Experiment(error) => Display::fmt(error, formatter),
            Self::Network(error) => Display::fmt(error, formatter),
            Self::Metrics(error) => Display::fmt(error, formatter),
            Self::Model(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for Phase2AnalysisError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Experiment(error) => Some(error),
            Self::Network(error) => Some(error),
            Self::Metrics(error) => Some(error),
            Self::Model(error) => Some(error),
            Self::InvalidConfig(_) => None,
        }
    }
}

impl From<ExperimentError> for Phase2AnalysisError {
    fn from(value: ExperimentError) -> Self {
        Self::Experiment(value)
    }
}

impl From<NetworkError> for Phase2AnalysisError {
    fn from(value: NetworkError) -> Self {
        Self::Network(value)
    }
}

impl From<MetricsError> for Phase2AnalysisError {
    fn from(value: MetricsError) -> Self {
        Self::Metrics(value)
    }
}

impl From<ModelError> for Phase2AnalysisError {
    fn from(value: ModelError) -> Self {
        Self::Model(value)
    }
}
