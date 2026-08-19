use serde::Serialize;

use crate::adaptive_mechanism::{
    ActivityAdjustment, ActivityObservation, AdjustmentMechanism, HomeostasisAdjustment,
    HomeostasisObservation, MechanismStateContract, PlasticityAdjustment, PlasticityObservation,
    REFERENCE_MECHANISM_ID,
};
use crate::mechanism_m1::{
    M1Control, M1PhaseResult, M1Rule, M1SeedProtocol, M1SeedResult, M1SingleRuleResult,
};

use crate::{EmbodiedError, GATE_B_SENSOR_COUNT, HIDDEN_COUNT};

const ACTION_COUNT: usize = 2;
const ACTIVE_RECURRENT_PER_UNIT: usize = 6;
const PLASTIC_RECURRENT_PER_UNIT: usize = 2;
const SEED_STRIDE: u64 = 0x9e37_79b9_7f4a_7c15;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Map0ParameterPoint {
    pub id: usize,
    pub recurrent_gain: f64,
    pub internal_learning_rate: f64,
    pub homeostasis_strength: f64,
    pub exploration_rate: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Map0Thresholds {
    pub memory_accuracy: f64,
    pub delayed_credit_accuracy: f64,
    pub reversal_accuracy: f64,
    pub restoration_accuracy: f64,
    pub repeated_reversal_accuracy: f64,
    pub perturbation_recovery_accuracy: f64,
    pub minimum_perturbation_damage_drop: f64,
    pub minimum_perturbation_recovery_gain: f64,
    pub maximum_saturation_fraction: f64,
    pub maximum_synchrony_fraction: f64,
    pub maximum_relative_weight_drift: f64,
    pub minimum_region_formation_probability: f64,
    pub minimum_region_size: usize,
    pub minimum_causal_probability_drop: f64,
    pub minimum_causal_probe_score_drop: f64,
}

impl Default for Map0Thresholds {
    fn default() -> Self {
        Self {
            memory_accuracy: 0.65,
            delayed_credit_accuracy: 0.65,
            reversal_accuracy: 0.70,
            restoration_accuracy: 0.70,
            repeated_reversal_accuracy: 0.70,
            perturbation_recovery_accuracy: 0.65,
            minimum_perturbation_damage_drop: 0.15,
            minimum_perturbation_recovery_gain: 0.10,
            maximum_saturation_fraction: 0.25,
            maximum_synchrony_fraction: 0.50,
            maximum_relative_weight_drift: 1.0,
            minimum_region_formation_probability: 0.50,
            minimum_region_size: 3,
            minimum_causal_probability_drop: 0.15,
            minimum_causal_probe_score_drop: 0.10,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Map0ExperimentConfig {
    pub seed: u64,
    pub development_config_count: usize,
    pub development_seed_count: usize,
    pub confirmation_seed_count: usize,
    pub confirmation_candidate_count: usize,
    pub pretraining_episodes: usize,
    pub adaptation_episodes: usize,
    pub evaluation_episodes: usize,
    pub threshold_check_interval: usize,
    pub cue_steps: usize,
    pub memory_delay_steps: usize,
    pub reward_delay_steps: usize,
    pub eligibility_decay: f64,
    pub hidden_leak: f64,
    pub policy_learning_rate: f64,
    pub softmax_temperature: f64,
    pub reward_baseline_decay: f64,
    pub weight_limit: f64,
    pub recurrent_gain_range: [f64; 2],
    pub internal_learning_rate_range: [f64; 2],
    pub homeostasis_strength_range: [f64; 2],
    pub exploration_rate_range: [f64; 2],
    pub thresholds: Map0Thresholds,
}

impl Default for Map0ExperimentConfig {
    fn default() -> Self {
        Self {
            seed: 0x4d41_5030_5f30_0101,
            development_config_count: 64,
            development_seed_count: 8,
            confirmation_seed_count: 12,
            confirmation_candidate_count: 6,
            pretraining_episodes: 320,
            adaptation_episodes: 240,
            evaluation_episodes: 80,
            threshold_check_interval: 20,
            cue_steps: 2,
            memory_delay_steps: 8,
            reward_delay_steps: 6,
            eligibility_decay: 0.88,
            hidden_leak: 0.80,
            policy_learning_rate: 0.055,
            softmax_temperature: 0.40,
            reward_baseline_decay: 0.92,
            weight_limit: 2.5,
            recurrent_gain_range: [0.08, 0.92],
            internal_learning_rate_range: [0.02, 0.18],
            homeostasis_strength_range: [0.0, 1.0],
            exploration_rate_range: [0.04, 0.24],
            thresholds: Map0Thresholds::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Map0Control {
    Baseline,
    ReferenceNormHomeostasis,
    AdditivePlasticity,
    NoResourceAccounting,
    NoResourceSupply,
    ResetBetweenTrials,
    FrozenPlasticity,
    ShuffledStructure,
    NoHomeostasis,
    NoExploration,
    RandomReward,
}

impl Map0Control {
    pub const CAUSAL_CONTROLS: [Self; 5] = [
        Self::FrozenPlasticity,
        Self::ShuffledStructure,
        Self::NoHomeostasis,
        Self::NoExploration,
        Self::RandomReward,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Map0RegionClass {
    Rigid,
    LearnableStable,
    TaskSpecialized,
    Unstable,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Map0ProbeMetrics {
    pub memory_accuracy: f64,
    pub delayed_credit_accuracy: f64,
    pub reversal_accuracy: f64,
    pub restoration_accuracy: f64,
    pub repeated_reversal_accuracy: f64,
    pub perturbation_recovery_accuracy: f64,
    pub perturbation_initial_accuracy: f64,
    pub perturbation_damage_drop: f64,
    pub perturbation_recovery_gain: f64,
    pub first_reversal_episodes_to_threshold: Option<usize>,
    pub restoration_episodes_to_threshold: Option<usize>,
    pub repeated_reversal_episodes_to_threshold: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Map0DynamicsMetrics {
    pub finite_activity_fraction: f64,
    pub saturation_fraction: f64,
    pub silent_fraction: f64,
    pub synchrony_fraction: f64,
    pub marginal_effective_dimension: f64,
    pub finite_horizon_perturbation_gain: f64,
    pub maximum_absolute_weight: f64,
    pub mean_relative_weight_drift: f64,
    pub mean_excitability_gain: f64,
    pub minimum_excitability_gain: f64,
    pub maximum_excitability_gain: f64,
    pub mean_resource_level: f64,
    pub minimum_resource_level: f64,
    pub maximum_resource_level: f64,
    pub resource_constrained_fraction: f64,
    pub approximate_state_updates: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Map0SeedResult {
    pub parameter_id: usize,
    pub seed: u64,
    pub control: Map0Control,
    pub probes: Map0ProbeMetrics,
    pub dynamics: Map0DynamicsMetrics,
    pub passed_probe_count: usize,
    pub class: Map0RegionClass,
    pub formed: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Map0Interval {
    pub mean: f64,
    pub lower95: f64,
    pub upper95: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Map0ParameterSummary {
    pub parameters: Map0ParameterPoint,
    pub control: Map0Control,
    pub seed_count: usize,
    pub formation_probability: Map0Interval,
    pub mean_probe_score: f64,
    pub mean_memory_accuracy: f64,
    pub mean_delayed_credit_accuracy: f64,
    pub mean_reversal_accuracy: f64,
    pub mean_restoration_accuracy: f64,
    pub mean_repeated_reversal_accuracy: f64,
    pub mean_perturbation_recovery_accuracy: f64,
    pub mean_perturbation_initial_accuracy: f64,
    pub mean_perturbation_damage_drop: f64,
    pub mean_perturbation_recovery_gain: f64,
    pub mean_saturation_fraction: f64,
    pub mean_synchrony_fraction: f64,
    pub mean_relative_weight_drift: f64,
    pub mean_resource_level: f64,
    pub mean_resource_constrained_fraction: f64,
    pub dominant_class: Map0RegionClass,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Map0ControlSummary {
    pub control: Map0Control,
    pub parameter_id: usize,
    pub formation_probability: Map0Interval,
    pub mean_probe_score: f64,
    pub formation_probability_change_from_baseline: f64,
    pub mean_probe_score_change_from_baseline: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Map0AcceptanceReport {
    pub deterministic_sampling: bool,
    pub development_and_confirmation_seeds_disjoint: bool,
    pub all_development_runs_complete: bool,
    pub confirmation_runs_complete: bool,
    pub causal_controls_complete: bool,
    pub causal_intervention_detected: bool,
    pub finite_outputs: bool,
    pub stable_region_found: bool,
    pub passed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Map0ExperimentResult {
    pub version: String,
    pub config: Map0ExperimentConfig,
    pub development_parameters: Vec<Map0ParameterPoint>,
    pub development_seed_results: Vec<Map0SeedResult>,
    pub development_summaries: Vec<Map0ParameterSummary>,
    pub confirmation_parameter_ids: Vec<usize>,
    pub confirmation_seed_results: Vec<Map0SeedResult>,
    pub confirmation_summaries: Vec<Map0ParameterSummary>,
    pub control_seed_results: Vec<Map0SeedResult>,
    pub control_summaries: Vec<Map0ControlSummary>,
    pub stable_region_parameter_ids: Vec<usize>,
    pub acceptance: Map0AcceptanceReport,
    pub conclusions: Vec<String>,
}

#[derive(Clone)]
struct Rng(u64);

#[derive(Clone, Copy)]
pub(crate) struct DualTimescaleHomeostasis {
    pub activity_target: f64,
    pub activity_ema_rate: f64,
    pub excitability_adjustment_rate: f64,
    pub weight_norm_relaxation_rate: f64,
    pub minimum_excitability_gain: f64,
    pub maximum_excitability_gain: f64,
}

#[derive(Clone, Copy)]
pub(crate) enum HomeostasisMechanism {
    ReferenceNorm,
    DualTimescale(DualTimescaleHomeostasis),
}

#[derive(Clone, Copy)]
pub(crate) struct SoftBoundedPlasticity {
    pub bound_scale: f64,
}

#[derive(Clone, Copy)]
pub(crate) enum PlasticityMechanism {
    Additive,
    SoftBounded(SoftBoundedPlasticity),
}

#[derive(Clone, Copy)]
pub(crate) struct ContinuousResource {
    pub initial_level: f64,
    pub supply_rate: f64,
    pub maintenance_cost: f64,
    pub activity_cost: f64,
    pub plasticity_cost: f64,
    pub minimum_modulation: f64,
}

#[derive(Clone, Copy)]
pub(crate) enum ResourceMechanism {
    Disabled,
    Continuous(ContinuousResource),
}

#[derive(Clone, Copy)]
pub(crate) struct ReferenceAdjustmentMechanism {
    homeostasis: HomeostasisMechanism,
    plasticity: PlasticityMechanism,
    resource: ResourceMechanism,
}

impl ReferenceAdjustmentMechanism {
    pub(crate) fn new(
        homeostasis: HomeostasisMechanism,
        plasticity: PlasticityMechanism,
        resource: ResourceMechanism,
    ) -> Self {
        Self {
            homeostasis,
            plasticity,
            resource,
        }
    }
}

impl AdjustmentMechanism for ReferenceAdjustmentMechanism {
    fn mechanism_id(&self) -> &'static str {
        REFERENCE_MECHANISM_ID
    }

    fn mechanism_state_contract(&self) -> MechanismStateContract {
        MechanismStateContract {
            scalar_state_per_unit: 0,
            value_range: "none".into(),
            update_budget: "zero".into(),
            freeze_mode: "not applicable".into(),
        }
    }

    fn initial_resource(&self) -> f64 {
        match self.resource {
            ResourceMechanism::Disabled => 1.0,
            ResourceMechanism::Continuous(config) => config.initial_level,
        }
    }

    fn excitability_bounds(&self) -> [f64; 2] {
        match self.homeostasis {
            HomeostasisMechanism::ReferenceNorm => [1.0, 1.0],
            HomeostasisMechanism::DualTimescale(config) => [
                config.minimum_excitability_gain,
                config.maximum_excitability_gain,
            ],
        }
    }

    fn adjust_activity(&mut self, observation: ActivityObservation) -> ActivityAdjustment {
        match self.resource {
            ResourceMechanism::Disabled => ActivityAdjustment {
                activity: observation.raw_activity,
                resource_level: 1.0,
            },
            ResourceMechanism::Continuous(config) => {
                let modulation = config.minimum_modulation
                    + (1.0 - config.minimum_modulation) * observation.resource_level;
                let activity = observation.raw_activity * modulation;
                ActivityAdjustment {
                    activity,
                    resource_level: (observation.resource_level + config.supply_rate
                        - config.maintenance_cost
                        - config.activity_cost * activity.abs())
                    .clamp(0.0, 1.0),
                }
            }
        }
    }

    fn adjust_plasticity(&mut self, observation: PlasticityObservation) -> PlasticityAdjustment {
        let recurrent_weight = match self.plasticity {
            PlasticityMechanism::Additive => {
                (observation.current_weight + observation.proposed_delta).clamp(
                    -observation.absolute_weight_limit,
                    observation.absolute_weight_limit,
                )
            }
            PlasticityMechanism::SoftBounded(config) => {
                let bound = observation.reference_norm / 2.0_f64.sqrt() * config.bound_scale;
                let headroom = if observation.proposed_delta >= 0.0 {
                    (bound - observation.current_weight) / (2.0 * bound)
                } else {
                    (bound + observation.current_weight) / (2.0 * bound)
                }
                .clamp(0.0, 1.0);
                (observation.current_weight + observation.proposed_delta * headroom)
                    .clamp(-bound, bound)
            }
        };
        let resource_level = match self.resource {
            ResourceMechanism::Disabled => 1.0,
            ResourceMechanism::Continuous(config) => (observation.resource_level
                - config.plasticity_cost * (recurrent_weight - observation.current_weight).abs())
            .max(0.0),
        };
        PlasticityAdjustment {
            recurrent_weight,
            resource_level,
        }
    }

    fn adjust_homeostasis(&mut self, observation: HomeostasisObservation) -> HomeostasisAdjustment {
        let (activity_ema, excitability_gain, weight_rate) = match self.homeostasis {
            HomeostasisMechanism::ReferenceNorm => {
                (observation.activity_ema, observation.excitability_gain, 1.0)
            }
            HomeostasisMechanism::DualTimescale(config) => {
                let activity_ema = (1.0 - config.activity_ema_rate) * observation.activity_ema
                    + config.activity_ema_rate * observation.mean_absolute_activity;
                let adjustment = (config.excitability_adjustment_rate
                    * observation.strength
                    * (config.activity_target - activity_ema))
                    .exp();
                let excitability_gain = (observation.excitability_gain * adjustment).clamp(
                    config.minimum_excitability_gain,
                    config.maximum_excitability_gain,
                );
                (
                    activity_ema,
                    excitability_gain,
                    config.weight_norm_relaxation_rate,
                )
            }
        };
        let mut recurrent_weights = observation.recurrent_weights;
        let current = recurrent_weights[0].hypot(recurrent_weights[1]);
        if current > 1e-12 {
            let target_scale = observation.reference_recurrent_norm / current;
            let correction = (observation.strength * weight_rate).min(1.0);
            let scale = if correction == 1.0 {
                target_scale
            } else {
                target_scale.powf(correction)
            };
            recurrent_weights[0] *= scale;
            recurrent_weights[1] *= scale;
        }
        HomeostasisAdjustment {
            activity_ema,
            excitability_gain,
            recurrent_weights,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuousStreamConfig {
    pub trial_count: usize,
    pub minimum_change_gap: usize,
    pub change_probability: f64,
    pub settling_window: usize,
    pub exploration: f64,
    pub reset_between_trials: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuousSeedResult {
    pub parameter_id: usize,
    pub seed: u64,
    pub control: Map0Control,
    pub trial_count: usize,
    pub rule_change_count: usize,
    pub state_reset_count: usize,
    pub overall_accuracy: f64,
    pub post_change_accuracy: f64,
    pub settled_accuracy: f64,
    pub recovery_gain: f64,
    pub mean_resource_level: f64,
    pub minimum_resource_level: f64,
    pub mean_relative_weight_drift: f64,
    pub maximum_absolute_weight: f64,
    pub finite: bool,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }

    fn next_u64(&mut self) -> u64 {
        let mut value = self.0;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.0 = value;
        value
    }

    fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / ((1_u64 << 53) as f64)
    }

    fn signed(&mut self, scale: f64) -> f64 {
        (self.unit() * 2.0 - 1.0) * scale
    }
}

#[derive(Clone)]
struct MapController<M: AdjustmentMechanism> {
    config: Map0ExperimentConfig,
    parameters: Map0ParameterPoint,
    input_weights: [[f64; GATE_B_SENSOR_COUNT]; HIDDEN_COUNT],
    recurrent_weights: [[f64; HIDDEN_COUNT]; HIDDEN_COUNT],
    recurrent_slots: [[usize; PLASTIC_RECURRENT_PER_UNIT]; HIDDEN_COUNT],
    action_weights: [[f64; HIDDEN_COUNT]; ACTION_COUNT],
    hidden: [f64; HIDDEN_COUNT],
    lesioned: [bool; HIDDEN_COUNT],
    baseline: f64,
    reference_recurrent_norms: [f64; HIDDEN_COUNT],
    adjustment_mechanism: M,
    resource: [f64; HIDDEN_COUNT],
    activity_ema: [f64; HIDDEN_COUNT],
    excitability_gain: [f64; HIDDEN_COUNT],
}

#[derive(Clone)]
struct Trial {
    target_right: bool,
    chosen_right: bool,
    probabilities: [f64; ACTION_COUNT],
    recurrent_eligibility: [[f64; PLASTIC_RECURRENT_PER_UNIT]; HIDDEN_COUNT],
    activity: Vec<[f64; HIDDEN_COUNT]>,
    resource: Vec<[f64; HIDDEN_COUNT]>,
    mean_absolute_activity: [f64; HIDDEN_COUNT],
}

#[derive(Default)]
struct ActivityAccumulator {
    samples: usize,
    finite: usize,
    saturated: usize,
    silent: usize,
    synchronized: usize,
    sums: [f64; HIDDEN_COUNT],
    sums_squared: [f64; HIDDEN_COUNT],
    resource_samples: usize,
    resource_sum: f64,
    resource_minimum: f64,
    resource_maximum: f64,
    resource_constrained: usize,
}

impl<M: AdjustmentMechanism> MapController<M> {
    fn new(
        config: Map0ExperimentConfig,
        parameters: Map0ParameterPoint,
        seed: u64,
        adjustment_mechanism: M,
    ) -> Self {
        let mut rng = Rng::new(seed);
        let mut input_weights = [[0.0; GATE_B_SENSOR_COUNT]; HIDDEN_COUNT];
        for row in &mut input_weights {
            for (sensor, weight) in row.iter_mut().enumerate() {
                *weight = rng.signed(if matches!(sensor, 2 | 3) { 1.2 } else { 0.12 });
            }
        }
        let preference = input_weights.map(|row| row[2] - row[3]);
        let mut recurrent_weights = [[0.0; HIDDEN_COUNT]; HIDDEN_COUNT];
        let mut recurrent_slots = [[0; PLASTIC_RECURRENT_PER_UNIT]; HIDDEN_COUNT];
        for target in 0..HIDDEN_COUNT {
            let mut candidates = (0..HIDDEN_COUNT)
                .filter(|source| *source != target)
                .collect::<Vec<_>>();
            candidates.sort_by(|left, right| preference[*right].total_cmp(&preference[*left]));
            let positive = candidates[0];
            let negative = *candidates.last().expect("negative source");
            recurrent_slots[target] = [positive, negative];
            let mut sources = vec![positive, negative];
            let offset = (rng.next_u64() as usize) % candidates.len();
            for index in 0..candidates.len() {
                let source = candidates[(index + offset) % candidates.len()];
                if !sources.contains(&source) {
                    sources.push(source);
                }
                if sources.len() == ACTIVE_RECURRENT_PER_UNIT {
                    break;
                }
            }
            for (rank, source) in sources.into_iter().enumerate() {
                let same =
                    preference[target].is_sign_positive() == preference[source].is_sign_positive();
                let magnitude = if rank < PLASTIC_RECURRENT_PER_UNIT {
                    parameters.recurrent_gain / 2.0
                } else {
                    parameters.recurrent_gain / 24.0
                };
                recurrent_weights[target][source] = if same { magnitude } else { -magnitude };
            }
        }
        let mut action_weights = [[0.0; HIDDEN_COUNT]; ACTION_COUNT];
        for row in &mut action_weights {
            for weight in row {
                *weight = rng.signed(0.08);
            }
        }
        let reference_recurrent_norms = std::array::from_fn(|target| {
            let [a, b] = recurrent_slots[target];
            recurrent_weights[target][a].hypot(recurrent_weights[target][b])
        });
        let proposed_resource = adjustment_mechanism.initial_resource();
        let initial_resource = if proposed_resource.is_finite() {
            proposed_resource.clamp(0.0, 1.0)
        } else {
            1.0
        };
        Self {
            config,
            parameters,
            input_weights,
            recurrent_weights,
            recurrent_slots,
            action_weights,
            hidden: [0.0; HIDDEN_COUNT],
            lesioned: [false; HIDDEN_COUNT],
            baseline: 0.0,
            reference_recurrent_norms,
            adjustment_mechanism,
            resource: [initial_resource; HIDDEN_COUNT],
            activity_ema: [0.0; HIDDEN_COUNT],
            excitability_gain: [1.0; HIDDEN_COUNT],
        }
    }

    fn reset_state(&mut self) {
        self.hidden = [0.0; HIDDEN_COUNT];
    }

    fn step(&mut self, sensors: [f64; GATE_B_SENSOR_COUNT]) -> [f64; HIDDEN_COUNT] {
        let previous = self.hidden;
        let raw_next: [f64; HIDDEN_COUNT] = std::array::from_fn(|target| {
            let input = self.input_weights[target]
                .iter()
                .zip(sensors)
                .map(|(weight, value)| weight * value)
                .sum::<f64>();
            let recurrent = self.recurrent_weights[target]
                .iter()
                .zip(previous)
                .map(|(weight, value)| weight * value)
                .sum::<f64>();
            if self.lesioned[target] {
                0.0
            } else {
                (self.excitability_gain[target]
                    * (input + self.config.hidden_leak * previous[target] + recurrent))
                    .tanh()
            }
        });
        let mut next = [0.0; HIDDEN_COUNT];
        for target in 0..HIDDEN_COUNT {
            let action = self
                .adjustment_mechanism
                .adjust_activity(ActivityObservation {
                    unit: target,
                    raw_activity: raw_next[target],
                    resource_level: self.resource[target],
                });
            next[target] = if action.activity.is_finite() {
                action.activity.clamp(-1.0, 1.0)
            } else {
                raw_next[target]
            };
            self.resource[target] = if action.resource_level.is_finite() {
                action.resource_level.clamp(0.0, 1.0)
            } else {
                self.resource[target]
            };
        }
        self.hidden = next;
        previous
    }

    fn probabilities(&self) -> [f64; ACTION_COUNT] {
        let logits = self.action_weights.map(|row| {
            row.iter()
                .zip(self.hidden)
                .map(|(weight, value)| weight * value)
                .sum::<f64>()
                / self.config.softmax_temperature
        });
        let maximum = logits.into_iter().fold(f64::NEG_INFINITY, f64::max);
        let mut values = logits.map(|value| (value - maximum).exp());
        let total = values.iter().sum::<f64>();
        values.iter_mut().for_each(|value| *value /= total);
        values
    }

    fn train_readout(&mut self, seed: u64) {
        for episode in 0..self.config.pretraining_episodes {
            let cue_right = balanced_cue(episode, seed ^ 0x5052_4554_5241_494e);
            let mut rng =
                Rng::new(seed ^ 0x5052_4541_4354_0101 ^ (episode as u64).wrapping_mul(SEED_STRIDE));
            let trial = self.trial(
                true,
                cue_right,
                self.config.memory_delay_steps,
                &mut rng,
                0.04,
                true,
                false,
            );
            let reward = if trial.chosen_right == trial.target_right {
                1.0
            } else {
                -1.0
            };
            let advantage = reward - self.baseline;
            self.baseline = self.config.reward_baseline_decay * self.baseline
                + (1.0 - self.config.reward_baseline_decay) * reward;
            let chosen = usize::from(trial.chosen_right);
            for action in 0..ACTION_COUNT {
                let error = f64::from(action == chosen) - trial.probabilities[action];
                for hidden in 0..HIDDEN_COUNT {
                    self.action_weights[action][hidden] = (self.action_weights[action][hidden]
                        + self.config.policy_learning_rate
                            * advantage
                            * error
                            * self.hidden[hidden])
                        .clamp(-self.config.weight_limit, self.config.weight_limit);
                }
            }
        }
        self.baseline = 0.0;
    }

    fn train_multisymbol_readout(&mut self, seed: u64) {
        for episode in 0..self.config.pretraining_episodes {
            let symbol = (episode + seed.count_ones() as usize) % 4;
            let mut rng =
                Rng::new(seed ^ 0x4d31_5052_4541_4354 ^ (episode as u64).wrapping_mul(SEED_STRIDE));
            let trial = self.symbol_trial(M1Rule::A, symbol, &mut rng, 0.04, true, false);
            let reward = if trial.chosen_right == trial.target_right {
                1.0
            } else {
                -1.0
            };
            let advantage = reward - self.baseline;
            self.baseline = self.config.reward_baseline_decay * self.baseline
                + (1.0 - self.config.reward_baseline_decay) * reward;
            let chosen = usize::from(trial.chosen_right);
            for action in 0..ACTION_COUNT {
                let error = f64::from(action == chosen) - trial.probabilities[action];
                for hidden in 0..HIDDEN_COUNT {
                    self.action_weights[action][hidden] = (self.action_weights[action][hidden]
                        + self.config.policy_learning_rate
                            * advantage
                            * error
                            * self.hidden[hidden])
                        .clamp(-self.config.weight_limit, self.config.weight_limit);
                }
            }
        }
        self.baseline = 0.0;
    }

    fn trial(
        &mut self,
        original_rule: bool,
        cue_right: bool,
        delay_steps: usize,
        rng: &mut Rng,
        exploration: f64,
        reset_before: bool,
        capture_activity: bool,
    ) -> Trial {
        if reset_before {
            self.reset_state();
        }
        let mut eligibility = [[0.0; PLASTIC_RECURRENT_PER_UNIT]; HIDDEN_COUNT];
        let mut activity = Vec::new();
        let mut resource = Vec::new();
        let mut activity_sum = [0.0; HIDDEN_COUNT];
        let total_steps = self.config.cue_steps + delay_steps;
        for step in 0..total_steps {
            let cue_visible = step < self.config.cue_steps;
            let sensors = sensors(step, total_steps, cue_visible, cue_right);
            let previous = self.step(sensors);
            for target in 0..HIDDEN_COUNT {
                activity_sum[target] += self.hidden[target].abs();
                let sensitivity = 1.0 - self.hidden[target].powi(2);
                for slot in 0..PLASTIC_RECURRENT_PER_UNIT {
                    let source = self.recurrent_slots[target][slot];
                    eligibility[target][slot] = self.config.eligibility_decay
                        * eligibility[target][slot]
                        + previous[source] * sensitivity;
                }
            }
            if capture_activity {
                activity.push(self.hidden);
                resource.push(self.resource);
            }
        }
        let probabilities = self
            .probabilities()
            .map(|value| value * (1.0 - exploration) + exploration / ACTION_COUNT as f64);
        let chosen_right = rng.unit() > probabilities[0];
        Trial {
            target_right: if original_rule { cue_right } else { !cue_right },
            chosen_right,
            probabilities,
            recurrent_eligibility: eligibility,
            activity,
            resource,
            mean_absolute_activity: activity_sum.map(|sum| sum / total_steps as f64),
        }
    }

    fn symbol_trial(
        &mut self,
        rule: M1Rule,
        symbol: usize,
        rng: &mut Rng,
        exploration: f64,
        reset_before: bool,
        capture_activity: bool,
    ) -> Trial {
        if reset_before {
            self.reset_state();
        }
        let mut eligibility = [[0.0; PLASTIC_RECURRENT_PER_UNIT]; HIDDEN_COUNT];
        let mut activity = Vec::new();
        let mut resource = Vec::new();
        let mut activity_sum = [0.0; HIDDEN_COUNT];
        let total_steps = self.config.cue_steps + self.config.memory_delay_steps;
        for step in 0..total_steps {
            let cue_visible = step < self.config.cue_steps;
            let sensors = symbol_sensors(step, total_steps, cue_visible, symbol);
            let previous = self.step(sensors);
            for target in 0..HIDDEN_COUNT {
                activity_sum[target] += self.hidden[target].abs();
                let sensitivity = 1.0 - self.hidden[target].powi(2);
                for slot in 0..PLASTIC_RECURRENT_PER_UNIT {
                    let source = self.recurrent_slots[target][slot];
                    eligibility[target][slot] = self.config.eligibility_decay
                        * eligibility[target][slot]
                        + previous[source] * sensitivity;
                }
            }
            if capture_activity {
                activity.push(self.hidden);
                resource.push(self.resource);
            }
        }
        let probabilities = self
            .probabilities()
            .map(|value| value * (1.0 - exploration) + exploration / ACTION_COUNT as f64);
        let chosen_right = rng.unit() > probabilities[0];
        Trial {
            target_right: rule.target_right(symbol),
            chosen_right,
            probabilities,
            recurrent_eligibility: eligibility,
            activity,
            resource,
            mean_absolute_activity: activity_sum.map(|sum| sum / total_steps as f64),
        }
    }

    fn adapt_trial(
        &mut self,
        trial: &Trial,
        control: Map0Control,
        random_reward: bool,
        reward_seed: u64,
        reward_delay_steps: usize,
    ) {
        let rewarded_choice = if random_reward {
            Rng::new(reward_seed).unit() >= 0.5
        } else {
            trial.target_right
        };
        let reward = if trial.chosen_right == rewarded_choice {
            1.0
        } else {
            -1.0
        };
        self.baseline = self.config.reward_baseline_decay * self.baseline
            + (1.0 - self.config.reward_baseline_decay) * reward;
        if control == Map0Control::FrozenPlasticity {
            return;
        }
        let chosen = usize::from(trial.chosen_right);
        let feedback: [f64; HIDDEN_COUNT] = std::array::from_fn(|hidden| {
            (0..ACTION_COUNT)
                .map(|action| {
                    (f64::from(action == chosen) - trial.probabilities[action])
                        * self.action_weights[action][hidden]
                })
                .sum::<f64>()
        });
        let delayed_scale = self
            .config
            .eligibility_decay
            .powi(reward_delay_steps as i32);
        for (target, feedback_value) in feedback.into_iter().enumerate() {
            for slot in 0..PLASTIC_RECURRENT_PER_UNIT {
                let source = self.recurrent_slots[target][slot];
                let delta = self.parameters.internal_learning_rate
                    * reward
                    * feedback_value
                    * trial.recurrent_eligibility[target][slot]
                    * delayed_scale;
                let action = self
                    .adjustment_mechanism
                    .adjust_plasticity(PlasticityObservation {
                        target_unit: target,
                        source_unit: source,
                        current_weight: self.recurrent_weights[target][source],
                        proposed_delta: delta,
                        reference_norm: self.reference_recurrent_norms[target],
                        absolute_weight_limit: self.config.weight_limit,
                        resource_level: self.resource[target],
                    });
                self.recurrent_weights[target][source] = if action.recurrent_weight.is_finite() {
                    action
                        .recurrent_weight
                        .clamp(-self.config.weight_limit, self.config.weight_limit)
                } else {
                    self.recurrent_weights[target][source]
                };
                self.resource[target] = if action.resource_level.is_finite() {
                    action.resource_level.clamp(0.0, 1.0)
                } else {
                    self.resource[target]
                };
            }
        }
        let homeostasis = if control == Map0Control::NoHomeostasis {
            0.0
        } else {
            self.parameters.homeostasis_strength
        };
        if homeostasis > 0.0 {
            self.apply_homeostasis(trial, homeostasis);
        }
    }

    fn apply_homeostasis(&mut self, trial: &Trial, strength: f64) {
        for target in 0..HIDDEN_COUNT {
            let [a, b] = self.recurrent_slots[target];
            let action = self
                .adjustment_mechanism
                .adjust_homeostasis(HomeostasisObservation {
                    unit: target,
                    strength,
                    mean_absolute_activity: trial.mean_absolute_activity[target],
                    activity_ema: self.activity_ema[target],
                    excitability_gain: self.excitability_gain[target],
                    recurrent_weights: [
                        self.recurrent_weights[target][a],
                        self.recurrent_weights[target][b],
                    ],
                    reference_recurrent_norm: self.reference_recurrent_norms[target],
                });
            self.activity_ema[target] = if action.activity_ema.is_finite() {
                action.activity_ema.clamp(0.0, 1.0)
            } else {
                self.activity_ema[target]
            };
            let [minimum_gain, maximum_gain] = self.adjustment_mechanism.excitability_bounds();
            self.excitability_gain[target] = if action.excitability_gain.is_finite()
                && minimum_gain.is_finite()
                && maximum_gain.is_finite()
                && minimum_gain > 0.0
                && minimum_gain <= maximum_gain
            {
                action.excitability_gain.clamp(minimum_gain, maximum_gain)
            } else {
                self.excitability_gain[target]
            };
            for (source, proposed) in [a, b].into_iter().zip(action.recurrent_weights) {
                if proposed.is_finite() {
                    self.recurrent_weights[target][source] =
                        proposed.clamp(-self.config.weight_limit, self.config.weight_limit);
                }
            }
        }
    }

    fn shuffle_structure(&mut self, seed: u64) {
        let mut rng = Rng::new(seed);
        for target in 0..HIDDEN_COUNT {
            for index in (1..HIDDEN_COUNT).rev() {
                let swap = (rng.next_u64() % (index as u64 + 1)) as usize;
                self.recurrent_weights[target].swap(index, swap);
            }
        }
    }

    fn lesion_quarter(&mut self) {
        let mut importance = (0..HIDDEN_COUNT)
            .map(|hidden| {
                (
                    hidden,
                    (self.action_weights[1][hidden] - self.action_weights[0][hidden]).abs(),
                )
            })
            .collect::<Vec<_>>();
        importance.sort_by(|left, right| right.1.total_cmp(&left.1));
        for (hidden, _) in importance.into_iter().take(HIDDEN_COUNT / 4) {
            self.lesioned[hidden] = true;
        }
    }

    fn weight_dynamics(&self) -> (f64, f64) {
        let mut maximum = 0.0_f64;
        let mut drift = 0.0;
        for target in 0..HIDDEN_COUNT {
            let [a, b] = self.recurrent_slots[target];
            maximum = maximum
                .max(self.recurrent_weights[target][a].abs())
                .max(self.recurrent_weights[target][b].abs());
            let reference = self.reference_recurrent_norms[target];
            let current =
                self.recurrent_weights[target][a].hypot(self.recurrent_weights[target][b]);
            drift += (current - reference).abs() / reference.max(1e-12);
        }
        (maximum, drift / HIDDEN_COUNT as f64)
    }

    fn action_readout_digest(&self) -> u64 {
        self.action_weights
            .iter()
            .flatten()
            .fold(0xcbf2_9ce4_8422_2325, |hash, value| {
                (hash ^ value.to_bits()).wrapping_mul(0x1000_0000_01b3)
            })
    }

    fn perturbation_gain(&self) -> f64 {
        let mut left = self.clone();
        let mut right = self.clone();
        left.reset_state();
        right.reset_state();
        right.hidden[0] = 1e-6;
        let initial = 1e-6;
        let input = sensors(2, 12, false, false);
        for _ in 0..12 {
            left.step(input);
            right.step(input);
        }
        let distance = left
            .hidden
            .iter()
            .zip(right.hidden)
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt();
        distance / initial
    }
}

impl ActivityAccumulator {
    fn push(&mut self, values: [f64; HIDDEN_COUNT], resource: [f64; HIDDEN_COUNT]) {
        self.samples += 1;
        let mean = values.iter().sum::<f64>() / HIDDEN_COUNT as f64;
        let variance = values
            .iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>()
            / HIDDEN_COUNT as f64;
        let mean_abs = values.iter().map(|value| value.abs()).sum::<f64>() / HIDDEN_COUNT as f64;
        self.synchronized += usize::from(variance.sqrt() < 0.05 && mean_abs > 0.20);
        for (index, value) in values.into_iter().enumerate() {
            self.finite += usize::from(value.is_finite());
            self.saturated += usize::from(value.abs() > 0.98);
            self.silent += usize::from(value.abs() < 0.02);
            if value.is_finite() {
                self.sums[index] += value;
                self.sums_squared[index] += value * value;
            }
        }
        for level in resource {
            self.resource_samples += 1;
            self.resource_sum += level;
            self.resource_minimum = if self.resource_samples == 1 {
                level
            } else {
                self.resource_minimum.min(level)
            };
            self.resource_maximum = self.resource_maximum.max(level);
            self.resource_constrained += usize::from(level < 0.5);
        }
    }

    fn finish<M: AdjustmentMechanism>(
        &self,
        controller: &MapController<M>,
        approximate_state_updates: u64,
    ) -> Map0DynamicsMetrics {
        let scalar_samples = (self.samples * HIDDEN_COUNT).max(1) as f64;
        let samples = self.samples.max(1) as f64;
        let variances = std::array::from_fn::<_, HIDDEN_COUNT, _>(|index| {
            (self.sums_squared[index] / samples - (self.sums[index] / samples).powi(2)).max(0.0)
        });
        let variance_sum = variances.iter().sum::<f64>();
        let effective_dimension = if variance_sum <= 1e-12 {
            0.0
        } else {
            variance_sum.powi(2)
                / variances
                    .iter()
                    .map(|value| value * value)
                    .sum::<f64>()
                    .max(1e-12)
        };
        let (maximum_absolute_weight, mean_relative_weight_drift) = controller.weight_dynamics();
        let mean_excitability_gain =
            controller.excitability_gain.iter().sum::<f64>() / HIDDEN_COUNT as f64;
        let minimum_excitability_gain = controller
            .excitability_gain
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min);
        let maximum_excitability_gain = controller
            .excitability_gain
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let resource_samples = self.resource_samples.max(1) as f64;
        Map0DynamicsMetrics {
            finite_activity_fraction: self.finite as f64 / scalar_samples,
            saturation_fraction: self.saturated as f64 / scalar_samples,
            silent_fraction: self.silent as f64 / scalar_samples,
            synchrony_fraction: self.synchronized as f64 / samples,
            marginal_effective_dimension: effective_dimension,
            finite_horizon_perturbation_gain: controller.perturbation_gain(),
            maximum_absolute_weight,
            mean_relative_weight_drift,
            mean_excitability_gain,
            minimum_excitability_gain,
            maximum_excitability_gain,
            mean_resource_level: self.resource_sum / resource_samples,
            minimum_resource_level: self.resource_minimum,
            maximum_resource_level: self.resource_maximum,
            resource_constrained_fraction: self.resource_constrained as f64 / resource_samples,
            approximate_state_updates,
        }
    }
}

pub fn run_map0_experiment(
    config: Map0ExperimentConfig,
) -> Result<Map0ExperimentResult, EmbodiedError> {
    validate_config(config)?;
    let parameters = parameter_points(config);
    let development_seeds =
        seed_partition(config.seed ^ 0x4445_5601, config.development_seed_count);
    let confirmation_seeds = seed_partition(
        config.seed ^ 0x434f_4e46_0101,
        config.confirmation_seed_count,
    );
    let mut development_seed_results = Vec::new();
    for point in &parameters {
        for seed in &development_seeds {
            development_seed_results.push(run_map_seed(
                config,
                *point,
                *seed,
                Map0Control::Baseline,
                HomeostasisMechanism::ReferenceNorm,
            ));
        }
    }
    let development_summaries = summarize(
        &parameters,
        &development_seed_results,
        Map0Control::Baseline,
    );
    let confirmation_parameter_ids = select_confirmation_candidates(config, &development_summaries);
    let mut confirmation_seed_results = Vec::new();
    for id in &confirmation_parameter_ids {
        let point = parameters[*id];
        for seed in &confirmation_seeds {
            confirmation_seed_results.push(run_map_seed(
                config,
                point,
                *seed,
                Map0Control::Baseline,
                HomeostasisMechanism::ReferenceNorm,
            ));
        }
    }
    let confirmation_parameters = confirmation_parameter_ids
        .iter()
        .map(|id| parameters[*id])
        .collect::<Vec<_>>();
    let confirmation_summaries = summarize(
        &confirmation_parameters,
        &confirmation_seed_results,
        Map0Control::Baseline,
    );
    let mut control_seed_results = Vec::new();
    for point in &confirmation_parameters {
        for control in Map0Control::CAUSAL_CONTROLS {
            for seed in &confirmation_seeds {
                control_seed_results.push(run_map_seed(
                    config,
                    *point,
                    *seed,
                    control,
                    HomeostasisMechanism::ReferenceNorm,
                ));
            }
        }
    }
    let mut control_summaries = Vec::new();
    for point in &confirmation_parameters {
        let baseline = confirmation_summaries
            .iter()
            .find(|summary| summary.parameters.id == point.id)
            .expect("confirmation baseline");
        for control in Map0Control::CAUSAL_CONTROLS {
            let summary = summarize(&[*point], &control_seed_results, control)
                .into_iter()
                .next()
                .expect("control summary");
            control_summaries.push(Map0ControlSummary {
                control,
                parameter_id: point.id,
                formation_probability: summary.formation_probability,
                mean_probe_score: summary.mean_probe_score,
                formation_probability_change_from_baseline: summary.formation_probability.mean
                    - baseline.formation_probability.mean,
                mean_probe_score_change_from_baseline: summary.mean_probe_score
                    - baseline.mean_probe_score,
            });
        }
    }
    let stable_region_parameter_ids = stable_region(config, &confirmation_summaries);
    let causal_intervention_detected = control_summaries.iter().any(|summary| {
        summary.formation_probability_change_from_baseline
            <= -config.thresholds.minimum_causal_probability_drop
            || summary.mean_probe_score_change_from_baseline
                <= -config.thresholds.minimum_causal_probe_score_drop
    });
    let all_results = development_seed_results
        .iter()
        .chain(&confirmation_seed_results)
        .chain(&control_seed_results)
        .collect::<Vec<_>>();
    let finite_outputs = all_results.iter().all(|result| result_is_finite(result));
    let seeds_disjoint = development_seeds
        .iter()
        .all(|seed| !confirmation_seeds.contains(seed));
    let acceptance = Map0AcceptanceReport {
        deterministic_sampling: parameters == parameter_points(config),
        development_and_confirmation_seeds_disjoint: seeds_disjoint,
        all_development_runs_complete: development_seed_results.len()
            == config.development_config_count * config.development_seed_count,
        confirmation_runs_complete: confirmation_seed_results.len()
            == confirmation_parameter_ids.len() * config.confirmation_seed_count,
        causal_controls_complete: control_seed_results.len()
            == confirmation_parameter_ids.len()
                * Map0Control::CAUSAL_CONTROLS.len()
                * config.confirmation_seed_count,
        causal_intervention_detected,
        finite_outputs,
        stable_region_found: stable_region_parameter_ids.len()
            >= config.thresholds.minimum_region_size
            && causal_intervention_detected,
        passed: false,
    };
    let acceptance = Map0AcceptanceReport {
        passed: acceptance.deterministic_sampling
            && acceptance.development_and_confirmation_seeds_disjoint
            && acceptance.all_development_runs_complete
            && acceptance.confirmation_runs_complete
            && acceptance.causal_controls_complete
            && acceptance.finite_outputs,
        ..acceptance
    };
    let conclusions = conclusions(
        &development_summaries,
        &confirmation_summaries,
        &control_summaries,
        &stable_region_parameter_ids,
    );
    Ok(Map0ExperimentResult {
        version: "learnability-map/v0.1".to_owned(),
        config,
        development_parameters: parameters,
        development_seed_results,
        development_summaries,
        confirmation_parameter_ids,
        confirmation_seed_results,
        confirmation_summaries,
        control_seed_results,
        control_summaries,
        stable_region_parameter_ids,
        acceptance,
        conclusions,
    })
}

pub(crate) fn run_map_seed(
    config: Map0ExperimentConfig,
    parameters: Map0ParameterPoint,
    seed: u64,
    control: Map0Control,
    homeostasis_mechanism: HomeostasisMechanism,
) -> Map0SeedResult {
    run_map_seed_with_plasticity(
        config,
        parameters,
        seed,
        control,
        homeostasis_mechanism,
        PlasticityMechanism::Additive,
    )
}

pub(crate) fn run_map_seed_with_plasticity(
    config: Map0ExperimentConfig,
    parameters: Map0ParameterPoint,
    seed: u64,
    control: Map0Control,
    homeostasis_mechanism: HomeostasisMechanism,
    plasticity_mechanism: PlasticityMechanism,
) -> Map0SeedResult {
    run_map_seed_with_mechanisms(
        config,
        parameters,
        seed,
        control,
        homeostasis_mechanism,
        plasticity_mechanism,
        ResourceMechanism::Disabled,
    )
}

pub(crate) fn run_map_seed_with_mechanisms(
    config: Map0ExperimentConfig,
    parameters: Map0ParameterPoint,
    seed: u64,
    control: Map0Control,
    homeostasis_mechanism: HomeostasisMechanism,
    plasticity_mechanism: PlasticityMechanism,
    resource_mechanism: ResourceMechanism,
) -> Map0SeedResult {
    run_map_seed_with_adjustment_mechanism(
        config,
        parameters,
        seed,
        control,
        ReferenceAdjustmentMechanism::new(
            homeostasis_mechanism,
            plasticity_mechanism,
            resource_mechanism,
        ),
    )
}

/// Runs the frozen closed-loop Map probe with a candidate mechanism.
///
/// The mechanism can affect the carrier only through [`AdjustmentMechanism`]
/// actions; the task generator, labels and action readout remain private.
pub fn run_map_seed_with_adjustment_mechanism<M: AdjustmentMechanism>(
    config: Map0ExperimentConfig,
    parameters: Map0ParameterPoint,
    seed: u64,
    control: Map0Control,
    mechanism: M,
) -> Map0SeedResult {
    let mut controller =
        MapController::new(config, parameters, seed ^ 0x434f_4e54_524f_4c01, mechanism);
    controller.train_readout(seed);
    if control == Map0Control::ShuffledStructure {
        controller.shuffle_structure(seed ^ 0x5348_5546_464c_4501);
    }
    let mut activity = ActivityAccumulator::default();
    let memory_accuracy = evaluate(
        &controller,
        true,
        config.memory_delay_steps,
        seed ^ 0x4d45_4d01,
        Some(&mut activity),
    );
    let mut delayed = controller.clone();
    let (_, delayed_credit_accuracy) = adapt_phase(
        &mut delayed,
        false,
        config.reward_delay_steps,
        control,
        seed ^ 0x4445_4c41_5901,
    );
    let mut switching = controller.clone();
    let (first_reversal_episodes, reversal_accuracy) =
        adapt_phase(&mut switching, false, 0, control, seed ^ 0x5245_5601);
    let (restoration_episodes, restoration_accuracy) =
        adapt_phase(&mut switching, true, 0, control, seed ^ 0x5245_5354_0101);
    let (repeated_reversal_episodes, repeated_reversal_accuracy) =
        adapt_phase(&mut switching, false, 0, control, seed ^ 0x5245_5632_0101);
    let mut perturbed = controller.clone();
    perturbed.lesion_quarter();
    let perturbation_initial_accuracy = evaluate(
        &perturbed,
        true,
        config.memory_delay_steps,
        seed ^ 0x5045_5254_4245_464f,
        None,
    );
    let (_, perturbation_recovery_accuracy) =
        adapt_phase(&mut perturbed, true, 0, control, seed ^ 0x5045_5254_0101);
    let _ = evaluate(
        &switching,
        false,
        config.memory_delay_steps,
        seed ^ 0x4143_5401,
        Some(&mut activity),
    );
    let approximate_trials = config.pretraining_episodes
        + config.adaptation_episodes * 5
        + config.evaluation_episodes * 7;
    let dynamics = activity.finish(
        &switching,
        (approximate_trials * (config.cue_steps + config.memory_delay_steps) * HIDDEN_COUNT) as u64,
    );
    let probes = Map0ProbeMetrics {
        memory_accuracy,
        delayed_credit_accuracy,
        reversal_accuracy,
        restoration_accuracy,
        repeated_reversal_accuracy,
        perturbation_recovery_accuracy,
        perturbation_initial_accuracy,
        perturbation_damage_drop: memory_accuracy - perturbation_initial_accuracy,
        perturbation_recovery_gain: perturbation_recovery_accuracy - perturbation_initial_accuracy,
        first_reversal_episodes_to_threshold: first_reversal_episodes,
        restoration_episodes_to_threshold: restoration_episodes,
        repeated_reversal_episodes_to_threshold: repeated_reversal_episodes,
    };
    classify_seed(
        config.thresholds,
        parameters.id,
        seed,
        control,
        probes,
        dynamics,
    )
}

pub(crate) fn run_continuous_seed_with_mechanisms(
    config: Map0ExperimentConfig,
    parameters: Map0ParameterPoint,
    seed: u64,
    control: Map0Control,
    homeostasis_mechanism: HomeostasisMechanism,
    plasticity_mechanism: PlasticityMechanism,
    resource_mechanism: ResourceMechanism,
    stream: ContinuousStreamConfig,
) -> ContinuousSeedResult {
    run_continuous_seed_with_adjustment_mechanism(
        config,
        parameters,
        seed,
        control,
        ReferenceAdjustmentMechanism::new(
            homeostasis_mechanism,
            plasticity_mechanism,
            resource_mechanism,
        ),
        stream,
    )
}

/// Runs the frozen continuous closed-loop stream with a candidate mechanism.
pub fn run_continuous_seed_with_adjustment_mechanism<M: AdjustmentMechanism>(
    config: Map0ExperimentConfig,
    parameters: Map0ParameterPoint,
    seed: u64,
    control: Map0Control,
    mechanism: M,
    stream: ContinuousStreamConfig,
) -> ContinuousSeedResult {
    let mut controller =
        MapController::new(config, parameters, seed ^ 0x434f_4e54_524f_4c01, mechanism);
    controller.train_readout(seed);
    controller.reset_state();
    let mut original_rule = true;
    let mut since_change = 0usize;
    let mut has_changed = false;
    let mut rule_change_count = 0usize;
    let mut overall_correct = 0usize;
    let mut post_change_correct = 0usize;
    let mut post_change_trials = 0usize;
    let mut settled_correct = 0usize;
    let mut settled_trials = 0usize;
    let mut resource_sum = 0.0;
    let mut resource_samples = 0usize;
    let mut resource_minimum = f64::INFINITY;
    let mut change_rng = Rng::new(seed ^ 0x434f_4e54_5354_524d);
    for episode in 0..stream.trial_count {
        if since_change >= stream.minimum_change_gap
            && change_rng.unit() < stream.change_probability
        {
            original_rule = !original_rule;
            since_change = 0;
            has_changed = true;
            rule_change_count += 1;
        }
        let cue_right = balanced_cue(episode, seed ^ 0x4355_455f_5354_524d);
        let mut rng =
            Rng::new(seed ^ 0x4143_545f_5354_524d ^ (episode as u64).wrapping_mul(SEED_STRIDE));
        let trial = controller.trial(
            original_rule,
            cue_right,
            config.memory_delay_steps,
            &mut rng,
            stream.exploration,
            stream.reset_between_trials,
            false,
        );
        let correct = trial.chosen_right == trial.target_right;
        overall_correct += usize::from(correct);
        if has_changed && since_change < stream.settling_window {
            post_change_correct += usize::from(correct);
            post_change_trials += 1;
        } else if has_changed {
            settled_correct += usize::from(correct);
            settled_trials += 1;
        }
        controller.adapt_trial(
            &trial,
            control,
            control == Map0Control::RandomReward,
            seed ^ 0x5245_575f_5354_524d ^ episode as u64,
            0,
        );
        for level in controller.resource {
            resource_sum += level;
            resource_samples += 1;
            resource_minimum = resource_minimum.min(level);
        }
        since_change += 1;
    }
    let ratio = |numerator: usize, denominator: usize| numerator as f64 / denominator.max(1) as f64;
    let post_change_accuracy = ratio(post_change_correct, post_change_trials);
    let settled_accuracy = ratio(settled_correct, settled_trials);
    let (maximum_absolute_weight, mean_relative_weight_drift) = controller.weight_dynamics();
    let mean_resource_level = resource_sum / resource_samples.max(1) as f64;
    let finite = controller.hidden.iter().all(|value| value.is_finite())
        && controller.resource.iter().all(|value| value.is_finite())
        && [
            post_change_accuracy,
            settled_accuracy,
            maximum_absolute_weight,
            mean_relative_weight_drift,
            mean_resource_level,
            resource_minimum,
        ]
        .into_iter()
        .all(f64::is_finite);
    ContinuousSeedResult {
        parameter_id: parameters.id,
        seed,
        control,
        trial_count: stream.trial_count,
        rule_change_count,
        state_reset_count: usize::from(stream.reset_between_trials) * stream.trial_count,
        overall_accuracy: ratio(overall_correct, stream.trial_count),
        post_change_accuracy,
        settled_accuracy,
        recovery_gain: settled_accuracy - post_change_accuracy,
        mean_resource_level,
        minimum_resource_level: resource_minimum,
        mean_relative_weight_drift,
        maximum_absolute_weight,
        finite,
    }
}

pub(crate) fn run_multirule_seed_with_mechanisms(
    config: Map0ExperimentConfig,
    parameters: Map0ParameterPoint,
    seed: u64,
    control: M1Control,
    homeostasis_mechanism: HomeostasisMechanism,
    plasticity_mechanism: PlasticityMechanism,
    resource_mechanism: ResourceMechanism,
    protocol: M1SeedProtocol,
) -> M1SeedResult {
    let mut controller = MapController::new(
        config,
        parameters,
        seed ^ 0x4d31_434f_4e54_524c,
        ReferenceAdjustmentMechanism::new(
            homeostasis_mechanism,
            plasticity_mechanism,
            resource_mechanism,
        ),
    );
    controller.train_multisymbol_readout(seed);
    controller.reset_state();
    let readout_before = controller.action_readout_digest();
    if control == M1Control::SingleRuleCapacity {
        return run_single_rule_capacity(controller, parameters, seed, protocol, readout_before);
    }
    let map_control = match control {
        M1Control::FrozenPlasticity => Map0Control::FrozenPlasticity,
        M1Control::RandomConsequence => Map0Control::RandomReward,
        _ => Map0Control::Baseline,
    };
    let reset_between_trials = control == M1Control::ResetBetweenTrials;
    let mut phase_results = Vec::new();
    let mut resource_sum = 0.0;
    let mut resource_samples = 0usize;
    let mut resource_minimum = f64::INFINITY;
    let mut state_reset_count = 0usize;
    for (phase_index, rule) in M1Rule::SEQUENCE.into_iter().enumerate() {
        let phase_seed =
            seed ^ 0x4d31_5048_4153_4501 ^ (phase_index as u64).wrapping_mul(SEED_STRIDE);
        let initial_accuracy = evaluate_multisymbol_rule(
            &controller,
            rule,
            protocol.evaluation_trial_count,
            phase_seed ^ 0x494e_4954,
        );
        let mut trials_to_threshold =
            (initial_accuracy >= protocol.accuracy_threshold).then_some(0);
        let mut correct = 0usize;
        for episode in 0..protocol.phase_trial_count {
            let symbol = (episode + phase_seed.count_ones() as usize) % 4;
            let mut rng = Rng::new(
                phase_seed ^ 0x4d31_4143_5449_4f4e ^ (episode as u64).wrapping_mul(SEED_STRIDE),
            );
            let trial = controller.symbol_trial(
                rule,
                symbol,
                &mut rng,
                protocol.exploration,
                reset_between_trials,
                false,
            );
            state_reset_count += usize::from(reset_between_trials);
            correct += usize::from(trial.chosen_right == trial.target_right);
            controller.adapt_trial(
                &trial,
                map_control,
                control == M1Control::RandomConsequence,
                phase_seed ^ 0x5245_5741_5244 ^ episode as u64,
                0,
            );
            for level in controller.resource {
                resource_sum += level;
                resource_samples += 1;
                resource_minimum = resource_minimum.min(level);
            }
            let completed = episode + 1;
            if trials_to_threshold.is_none()
                && completed.is_multiple_of(protocol.threshold_check_interval)
            {
                let accuracy = evaluate_multisymbol_rule(
                    &controller,
                    rule,
                    protocol.evaluation_trial_count,
                    phase_seed ^ 0x4348_4543_4b01 ^ completed as u64,
                );
                if accuracy >= protocol.accuracy_threshold {
                    trials_to_threshold = Some(completed);
                }
            }
        }
        let departure_accuracy = evaluate_multisymbol_rule(
            &controller,
            rule,
            protocol.evaluation_trial_count,
            phase_seed ^ 0x4649_4e41_4c01,
        );
        phase_results.push(M1PhaseResult {
            phase_index,
            rule,
            initial_accuracy,
            online_accuracy: correct as f64 / protocol.phase_trial_count as f64,
            departure_accuracy,
            trials_to_threshold,
        });
    }
    let initial_a_accuracy = phase_results[0].initial_accuracy;
    let departure_a_accuracy = phase_results[0].departure_accuracy;
    let return_a_initial_accuracy = phase_results[4].initial_accuracy;
    let return_a_final_accuracy = phase_results[4].departure_accuracy;
    let mean_novel_rule_final_accuracy = phase_results[1..4]
        .iter()
        .map(|phase| phase.departure_accuracy)
        .sum::<f64>()
        / 3.0;
    let acquisition_trials = phase_results[1..4]
        .iter()
        .filter_map(|phase| phase.trials_to_threshold)
        .collect::<Vec<_>>();
    let mean_first_acquisition_trials = (acquisition_trials.len() == 3)
        .then(|| acquisition_trials.iter().sum::<usize>() as f64 / acquisition_trials.len() as f64);
    let return_reacquisition_trials = phase_results[4].trials_to_threshold;
    let reacquisition_speedup = mean_first_acquisition_trials
        .zip(return_reacquisition_trials)
        .map(|(first, reacquisition)| first - reacquisition as f64);
    let (maximum_absolute_weight, mean_relative_weight_drift) = controller.weight_dynamics();
    let mean_resource_level = resource_sum / resource_samples.max(1) as f64;
    let readout_after = controller.action_readout_digest();
    let finite = controller.hidden.iter().all(|value| value.is_finite())
        && controller.resource.iter().all(|value| value.is_finite())
        && phase_results.iter().all(|phase| {
            [
                phase.initial_accuracy,
                phase.online_accuracy,
                phase.departure_accuracy,
            ]
            .into_iter()
            .all(f64::is_finite)
        })
        && [
            mean_novel_rule_final_accuracy,
            mean_resource_level,
            resource_minimum,
            maximum_absolute_weight,
            mean_relative_weight_drift,
        ]
        .into_iter()
        .all(f64::is_finite);
    M1SeedResult {
        parameter_id: parameters.id,
        seed,
        control,
        phase_results,
        single_rule_results: Vec::new(),
        initial_a_accuracy,
        departure_a_accuracy,
        return_a_initial_accuracy,
        return_a_final_accuracy,
        retention_drop: departure_a_accuracy - return_a_initial_accuracy,
        mean_novel_rule_final_accuracy,
        mean_first_acquisition_trials,
        return_reacquisition_trials,
        reacquisition_speedup,
        minimum_single_rule_final_accuracy: 0.0,
        mean_resource_level,
        minimum_resource_level: resource_minimum,
        mean_relative_weight_drift,
        maximum_absolute_weight,
        state_reset_count,
        action_readout_digest_before: readout_before,
        action_readout_digest_after: readout_after,
        finite,
    }
}

fn run_single_rule_capacity<M: AdjustmentMechanism>(
    controller: MapController<M>,
    parameters: Map0ParameterPoint,
    seed: u64,
    protocol: M1SeedProtocol,
    readout_before: u64,
) -> M1SeedResult {
    let mut single_rule_results = Vec::new();
    let mut resource_sum = 0.0;
    let mut resource_samples = 0usize;
    let mut resource_minimum = f64::INFINITY;
    let mut drift_sum = 0.0;
    let mut maximum_absolute_weight = 0.0_f64;
    let mut all_finite = true;
    let mut readout_after = readout_before;
    for (rule_index, rule) in M1Rule::UNIQUE.into_iter().enumerate() {
        let mut candidate = controller.clone();
        candidate.reset_state();
        let rule_seed =
            seed ^ 0x4d31_5349_4e47_4c45 ^ (rule_index as u64).wrapping_mul(SEED_STRIDE);
        let initial_accuracy = evaluate_multisymbol_rule(
            &candidate,
            rule,
            protocol.evaluation_trial_count,
            rule_seed ^ 0x494e_4954,
        );
        let mut trials_to_threshold =
            (initial_accuracy >= protocol.accuracy_threshold).then_some(0);
        for episode in 0..protocol.phase_trial_count {
            let symbol = (episode + rule_seed.count_ones() as usize) % 4;
            let mut rng = Rng::new(
                rule_seed ^ 0x4d31_4143_5449_4f4e ^ (episode as u64).wrapping_mul(SEED_STRIDE),
            );
            let trial =
                candidate.symbol_trial(rule, symbol, &mut rng, protocol.exploration, false, false);
            candidate.adapt_trial(
                &trial,
                Map0Control::Baseline,
                false,
                rule_seed ^ 0x5245_5741_5244 ^ episode as u64,
                0,
            );
            for level in candidate.resource {
                resource_sum += level;
                resource_samples += 1;
                resource_minimum = resource_minimum.min(level);
            }
            let completed = episode + 1;
            if trials_to_threshold.is_none()
                && completed.is_multiple_of(protocol.threshold_check_interval)
                && evaluate_multisymbol_rule(
                    &candidate,
                    rule,
                    protocol.evaluation_trial_count,
                    rule_seed ^ 0x4348_4543_4b01 ^ completed as u64,
                ) >= protocol.accuracy_threshold
            {
                trials_to_threshold = Some(completed);
            }
        }
        let final_accuracy = evaluate_multisymbol_rule(
            &candidate,
            rule,
            protocol.evaluation_trial_count,
            rule_seed ^ 0x4649_4e41_4c01,
        );
        let (maximum, drift) = candidate.weight_dynamics();
        maximum_absolute_weight = maximum_absolute_weight.max(maximum);
        drift_sum += drift;
        readout_after = candidate.action_readout_digest();
        all_finite &= candidate.hidden.iter().all(|value| value.is_finite())
            && candidate.resource.iter().all(|value| value.is_finite())
            && final_accuracy.is_finite()
            && drift.is_finite();
        single_rule_results.push(M1SingleRuleResult {
            rule,
            initial_accuracy,
            final_accuracy,
            trials_to_threshold,
        });
    }
    let minimum_single_rule_final_accuracy = single_rule_results
        .iter()
        .map(|result| result.final_accuracy)
        .fold(f64::INFINITY, f64::min);
    let initial_a_accuracy = single_rule_results[0].initial_accuracy;
    let final_a_accuracy = single_rule_results[0].final_accuracy;
    M1SeedResult {
        parameter_id: parameters.id,
        seed,
        control: M1Control::SingleRuleCapacity,
        phase_results: Vec::new(),
        single_rule_results,
        initial_a_accuracy,
        departure_a_accuracy: final_a_accuracy,
        return_a_initial_accuracy: final_a_accuracy,
        return_a_final_accuracy: final_a_accuracy,
        retention_drop: 0.0,
        mean_novel_rule_final_accuracy: 0.0,
        mean_first_acquisition_trials: None,
        return_reacquisition_trials: None,
        reacquisition_speedup: None,
        minimum_single_rule_final_accuracy,
        mean_resource_level: resource_sum / resource_samples.max(1) as f64,
        minimum_resource_level: resource_minimum,
        mean_relative_weight_drift: drift_sum / M1Rule::UNIQUE.len() as f64,
        maximum_absolute_weight,
        state_reset_count: 0,
        action_readout_digest_before: readout_before,
        action_readout_digest_after: readout_after,
        finite: all_finite,
    }
}

fn evaluate_multisymbol_rule<M: AdjustmentMechanism>(
    controller: &MapController<M>,
    rule: M1Rule,
    trial_count: usize,
    seed: u64,
) -> f64 {
    let mut evaluation = controller.clone();
    let mut correct = 0usize;
    for episode in 0..trial_count {
        let symbol = (episode + seed.count_ones() as usize) % 4;
        let mut rng =
            Rng::new(seed ^ 0x4d31_4556_414c_0101 ^ (episode as u64).wrapping_mul(SEED_STRIDE));
        let trial = evaluation.symbol_trial(rule, symbol, &mut rng, 0.0, false, false);
        correct += usize::from(trial.chosen_right == trial.target_right);
    }
    correct as f64 / trial_count.max(1) as f64
}

fn adapt_phase<M: AdjustmentMechanism>(
    controller: &mut MapController<M>,
    original_rule: bool,
    reward_delay_steps: usize,
    control: Map0Control,
    seed: u64,
) -> (Option<usize>, f64) {
    let exploration = if control == Map0Control::NoExploration {
        0.0
    } else {
        controller.parameters.exploration_rate
    };
    let mut threshold_episode = None;
    for episode in 0..controller.config.adaptation_episodes {
        let cue_right = balanced_cue(episode, seed ^ 0x4355_4501);
        let mut rng = Rng::new(seed ^ (episode as u64).wrapping_mul(SEED_STRIDE));
        let trial = controller.trial(
            original_rule,
            cue_right,
            controller.config.memory_delay_steps,
            &mut rng,
            exploration,
            true,
            false,
        );
        controller.adapt_trial(
            &trial,
            control,
            control == Map0Control::RandomReward,
            seed ^ 0x5245_5741_5244 ^ episode as u64,
            reward_delay_steps,
        );
        let completed = episode + 1;
        if threshold_episode.is_none()
            && completed.is_multiple_of(controller.config.threshold_check_interval)
        {
            let accuracy = evaluate(
                controller,
                original_rule,
                controller.config.memory_delay_steps,
                seed ^ 0x4348_4543_4b01 ^ completed as u64,
                None,
            );
            if accuracy >= controller.config.thresholds.reversal_accuracy {
                threshold_episode = Some(completed);
            }
        }
    }
    let accuracy = evaluate(
        controller,
        original_rule,
        controller.config.memory_delay_steps,
        seed ^ 0x4649_4e41_4c01,
        None,
    );
    (threshold_episode, accuracy)
}

fn evaluate<M: AdjustmentMechanism>(
    controller: &MapController<M>,
    original_rule: bool,
    delay_steps: usize,
    seed: u64,
    mut activity: Option<&mut ActivityAccumulator>,
) -> f64 {
    let mut evaluation = controller.clone();
    let mut correct = 0;
    for episode in 0..controller.config.evaluation_episodes {
        let cue_right = episode % 2 == 1;
        let mut rng = Rng::new(seed ^ (episode as u64).wrapping_mul(SEED_STRIDE));
        let trial = evaluation.trial(
            original_rule,
            cue_right,
            delay_steps,
            &mut rng,
            0.0,
            true,
            activity.is_some(),
        );
        correct += usize::from(trial.chosen_right == trial.target_right);
        if let Some(accumulator) = activity.as_deref_mut() {
            for (values, resource) in trial.activity.into_iter().zip(trial.resource) {
                accumulator.push(values, resource);
            }
        }
    }
    correct as f64 / controller.config.evaluation_episodes as f64
}

fn classify_seed(
    thresholds: Map0Thresholds,
    parameter_id: usize,
    seed: u64,
    control: Map0Control,
    probes: Map0ProbeMetrics,
    dynamics: Map0DynamicsMetrics,
) -> Map0SeedResult {
    let retention_speed = match (
        probes.first_reversal_episodes_to_threshold,
        probes.repeated_reversal_episodes_to_threshold,
    ) {
        (Some(first), Some(repeated)) => repeated <= first,
        _ => false,
    };
    let probe_passes = [
        probes.memory_accuracy >= thresholds.memory_accuracy,
        probes.delayed_credit_accuracy >= thresholds.delayed_credit_accuracy,
        probes.reversal_accuracy >= thresholds.reversal_accuracy,
        probes.restoration_accuracy >= thresholds.restoration_accuracy,
        probes.repeated_reversal_accuracy >= thresholds.repeated_reversal_accuracy
            && retention_speed,
        probes.perturbation_recovery_accuracy >= thresholds.perturbation_recovery_accuracy
            && probes.perturbation_damage_drop >= thresholds.minimum_perturbation_damage_drop
            && probes.perturbation_recovery_gain >= thresholds.minimum_perturbation_recovery_gain,
    ];
    let passed_probe_count = probe_passes.into_iter().filter(|passed| *passed).count();
    let stable = dynamics.finite_activity_fraction == 1.0
        && dynamics.saturation_fraction <= thresholds.maximum_saturation_fraction
        && dynamics.synchrony_fraction <= thresholds.maximum_synchrony_fraction
        && dynamics.mean_relative_weight_drift <= thresholds.maximum_relative_weight_drift;
    let class = if !stable {
        Map0RegionClass::Unstable
    } else if passed_probe_count == probe_passes.len() {
        Map0RegionClass::LearnableStable
    } else if passed_probe_count >= 2 {
        Map0RegionClass::TaskSpecialized
    } else {
        Map0RegionClass::Rigid
    };
    Map0SeedResult {
        parameter_id,
        seed,
        control,
        probes,
        dynamics,
        passed_probe_count,
        class,
        formed: class == Map0RegionClass::LearnableStable,
    }
}

pub(crate) fn summarize(
    parameters: &[Map0ParameterPoint],
    results: &[Map0SeedResult],
    control: Map0Control,
) -> Vec<Map0ParameterSummary> {
    parameters
        .iter()
        .map(|parameters| {
            let rows = results
                .iter()
                .filter(|row| row.parameter_id == parameters.id && row.control == control)
                .collect::<Vec<_>>();
            let count = rows.len().max(1) as f64;
            let formed = rows.iter().filter(|row| row.formed).count();
            let mean = |value: fn(&Map0SeedResult) -> f64| {
                rows.iter().map(|row| value(row)).sum::<f64>() / count
            };
            let mut class_counts = [0usize; 4];
            for row in &rows {
                class_counts[class_index(row.class)] += 1;
            }
            let dominant_class = class_from_index(
                class_counts
                    .iter()
                    .enumerate()
                    .max_by_key(|(_, count)| *count)
                    .map(|(index, _)| index)
                    .unwrap_or(0),
            );
            Map0ParameterSummary {
                parameters: *parameters,
                control,
                seed_count: rows.len(),
                formation_probability: wilson_interval(formed, rows.len()),
                mean_probe_score: mean(|row| row.passed_probe_count as f64 / 6.0),
                mean_memory_accuracy: mean(|row| row.probes.memory_accuracy),
                mean_delayed_credit_accuracy: mean(|row| row.probes.delayed_credit_accuracy),
                mean_reversal_accuracy: mean(|row| row.probes.reversal_accuracy),
                mean_restoration_accuracy: mean(|row| row.probes.restoration_accuracy),
                mean_repeated_reversal_accuracy: mean(|row| row.probes.repeated_reversal_accuracy),
                mean_perturbation_recovery_accuracy: mean(|row| {
                    row.probes.perturbation_recovery_accuracy
                }),
                mean_perturbation_initial_accuracy: mean(|row| {
                    row.probes.perturbation_initial_accuracy
                }),
                mean_perturbation_damage_drop: mean(|row| row.probes.perturbation_damage_drop),
                mean_perturbation_recovery_gain: mean(|row| row.probes.perturbation_recovery_gain),
                mean_saturation_fraction: mean(|row| row.dynamics.saturation_fraction),
                mean_synchrony_fraction: mean(|row| row.dynamics.synchrony_fraction),
                mean_relative_weight_drift: mean(|row| row.dynamics.mean_relative_weight_drift),
                mean_resource_level: mean(|row| row.dynamics.mean_resource_level),
                mean_resource_constrained_fraction: mean(|row| {
                    row.dynamics.resource_constrained_fraction
                }),
                dominant_class,
            }
        })
        .collect()
}

fn parameter_points(config: Map0ExperimentConfig) -> Vec<Map0ParameterPoint> {
    (1..=config.development_config_count)
        .map(|index| Map0ParameterPoint {
            id: index - 1,
            recurrent_gain: interpolate(config.recurrent_gain_range, halton(index, 2)),
            internal_learning_rate: interpolate(
                config.internal_learning_rate_range,
                halton(index, 3),
            ),
            homeostasis_strength: interpolate(config.homeostasis_strength_range, halton(index, 5)),
            exploration_rate: interpolate(config.exploration_rate_range, halton(index, 7)),
        })
        .collect()
}

pub(crate) fn halton(mut index: usize, base: usize) -> f64 {
    let mut result = 0.0;
    let mut fraction = 1.0;
    while index > 0 {
        fraction /= base as f64;
        result += fraction * (index % base) as f64;
        index /= base;
    }
    result
}

pub(crate) fn interpolate(range: [f64; 2], unit: f64) -> f64 {
    range[0] + (range[1] - range[0]) * unit
}

pub(crate) fn seed_partition(seed: u64, count: usize) -> Vec<u64> {
    (0..count)
        .map(|index| seed.wrapping_add((index as u64).wrapping_mul(SEED_STRIDE)))
        .collect()
}

fn select_confirmation_candidates(
    config: Map0ExperimentConfig,
    summaries: &[Map0ParameterSummary],
) -> Vec<usize> {
    let mut ranked = summaries.iter().collect::<Vec<_>>();
    ranked.sort_by(|left, right| summary_rank(right).total_cmp(&summary_rank(left)));
    let Some(best) = ranked.first().copied() else {
        return Vec::new();
    };
    let mut selected = vec![best.parameters.id];
    let mut neighbors = summaries.iter().collect::<Vec<_>>();
    neighbors.sort_by(|left, right| {
        normalized_distance(config, best.parameters, left.parameters).total_cmp(
            &normalized_distance(config, best.parameters, right.parameters),
        )
    });
    for neighbor in neighbors
        .into_iter()
        .filter(|summary| summary.parameters.id != best.parameters.id)
        .take(config.thresholds.minimum_region_size.saturating_sub(1))
    {
        selected.push(neighbor.parameters.id);
    }
    for summary in ranked {
        if selected.len() == config.confirmation_candidate_count {
            break;
        }
        if !selected.contains(&summary.parameters.id) {
            selected.push(summary.parameters.id);
        }
    }
    selected
}

pub(crate) fn summary_rank(summary: &Map0ParameterSummary) -> f64 {
    summary.formation_probability.mean * 2.0 + summary.mean_probe_score
}

fn stable_region(config: Map0ExperimentConfig, summaries: &[Map0ParameterSummary]) -> Vec<usize> {
    let eligible = summaries
        .iter()
        .filter(|summary| {
            summary.formation_probability.mean
                >= config.thresholds.minimum_region_formation_probability
        })
        .collect::<Vec<_>>();
    let mut best = Vec::new();
    for start in &eligible {
        let mut component = vec![start.parameters.id];
        let mut cursor = 0;
        while cursor < component.len() {
            let current = summaries
                .iter()
                .find(|summary| summary.parameters.id == component[cursor])
                .expect("component summary");
            for candidate in &eligible {
                if !component.contains(&candidate.parameters.id)
                    && normalized_distance(config, current.parameters, candidate.parameters) <= 0.55
                {
                    component.push(candidate.parameters.id);
                }
            }
            cursor += 1;
        }
        if component.len() > best.len() {
            best = component;
        }
    }
    best.sort_unstable();
    best
}

pub(crate) fn normalized_distance(
    config: Map0ExperimentConfig,
    left: Map0ParameterPoint,
    right: Map0ParameterPoint,
) -> f64 {
    let normalized = |a: f64, b: f64, range: [f64; 2]| (a - b) / (range[1] - range[0]).max(1e-12);
    (normalized(
        left.recurrent_gain,
        right.recurrent_gain,
        config.recurrent_gain_range,
    )
    .powi(2)
        + normalized(
            left.internal_learning_rate,
            right.internal_learning_rate,
            config.internal_learning_rate_range,
        )
        .powi(2)
        + normalized(
            left.homeostasis_strength,
            right.homeostasis_strength,
            config.homeostasis_strength_range,
        )
        .powi(2)
        + normalized(
            left.exploration_rate,
            right.exploration_rate,
            config.exploration_rate_range,
        )
        .powi(2))
    .sqrt()
}

pub(crate) fn wilson_interval(successes: usize, total: usize) -> Map0Interval {
    if total == 0 {
        return Map0Interval::default();
    }
    let n = total as f64;
    let probability = successes as f64 / n;
    let z = 1.959_963_984_540_054;
    let denominator = 1.0 + z * z / n;
    let center = (probability + z * z / (2.0 * n)) / denominator;
    let margin =
        z * ((probability * (1.0 - probability) / n + z * z / (4.0 * n * n)).sqrt()) / denominator;
    Map0Interval {
        mean: probability,
        lower95: (center - margin).max(0.0),
        upper95: (center + margin).min(1.0),
    }
}

pub(crate) fn result_is_finite(result: &Map0SeedResult) -> bool {
    [
        result.probes.memory_accuracy,
        result.probes.delayed_credit_accuracy,
        result.probes.reversal_accuracy,
        result.probes.restoration_accuracy,
        result.probes.repeated_reversal_accuracy,
        result.probes.perturbation_recovery_accuracy,
        result.probes.perturbation_initial_accuracy,
        result.probes.perturbation_damage_drop,
        result.probes.perturbation_recovery_gain,
        result.dynamics.finite_activity_fraction,
        result.dynamics.saturation_fraction,
        result.dynamics.silent_fraction,
        result.dynamics.synchrony_fraction,
        result.dynamics.marginal_effective_dimension,
        result.dynamics.finite_horizon_perturbation_gain,
        result.dynamics.maximum_absolute_weight,
        result.dynamics.mean_relative_weight_drift,
        result.dynamics.mean_excitability_gain,
        result.dynamics.minimum_excitability_gain,
        result.dynamics.maximum_excitability_gain,
        result.dynamics.mean_resource_level,
        result.dynamics.minimum_resource_level,
        result.dynamics.maximum_resource_level,
        result.dynamics.resource_constrained_fraction,
    ]
    .into_iter()
    .all(f64::is_finite)
}

fn class_index(class: Map0RegionClass) -> usize {
    match class {
        Map0RegionClass::Rigid => 0,
        Map0RegionClass::LearnableStable => 1,
        Map0RegionClass::TaskSpecialized => 2,
        Map0RegionClass::Unstable => 3,
    }
}

fn class_from_index(index: usize) -> Map0RegionClass {
    match index {
        1 => Map0RegionClass::LearnableStable,
        2 => Map0RegionClass::TaskSpecialized,
        3 => Map0RegionClass::Unstable,
        _ => Map0RegionClass::Rigid,
    }
}

fn balanced_cue(episode: usize, seed: u64) -> bool {
    ((episode as u64).wrapping_add(seed.count_ones() as u64) & 1) == 1
}

fn sensors(
    step: usize,
    total_steps: usize,
    cue_visible: bool,
    cue_right: bool,
) -> [f64; GATE_B_SENSOR_COUNT] {
    let mut values = [0.0; GATE_B_SENSOR_COUNT];
    values[0] = 1.0;
    values[1] = step as f64 / total_steps.max(1) as f64;
    if cue_visible {
        values[if cue_right { 3 } else { 2 }] = 1.0;
    }
    values[4] = f64::from(!cue_visible);
    values[5] = f64::from(step + 1 == total_steps);
    values
}

fn symbol_sensors(
    step: usize,
    total_steps: usize,
    cue_visible: bool,
    symbol: usize,
) -> [f64; GATE_B_SENSOR_COUNT] {
    let mut values = [0.0; GATE_B_SENSOR_COUNT];
    values[0] = 1.0;
    values[1] = step as f64 / total_steps.max(1) as f64;
    if cue_visible {
        values[2] = if symbol & 1 == 0 { -1.0 } else { 1.0 };
        values[3] = if symbol & 2 == 0 { -1.0 } else { 1.0 };
    }
    values[4] = f64::from(!cue_visible);
    values[5] = f64::from(step + 1 == total_steps);
    values
}

fn conclusions(
    development: &[Map0ParameterSummary],
    confirmation: &[Map0ParameterSummary],
    controls: &[Map0ControlSummary],
    stable_region: &[usize],
) -> Vec<String> {
    let best_development = development
        .iter()
        .max_by(|left, right| summary_rank(left).total_cmp(&summary_rank(right)));
    let best_confirmation = confirmation
        .iter()
        .max_by(|left, right| summary_rank(left).total_cmp(&summary_rank(right)));
    let mut output = Vec::new();
    if let Some(best) = best_development {
        output.push(format!(
            "开发扫描最佳配置 {} 的形成概率为 {:.1}%，平均探针得分为 {:.1}%。",
            best.parameters.id,
            best.formation_probability.mean * 100.0,
            best.mean_probe_score * 100.0,
        ));
    }
    if let Some(best) = best_confirmation {
        output.push(format!(
            "独立确认最佳配置 {} 的形成概率为 {:.1}%（Wilson 95% 区间 {:.1}%–{:.1}%）。",
            best.parameters.id,
            best.formation_probability.mean * 100.0,
            best.formation_probability.lower95 * 100.0,
            best.formation_probability.upper95 * 100.0,
        ));
    }
    if stable_region.is_empty() {
        output.push("未找到满足预注册标准的连续稳定可学习区域；该结果是机制修改的边界证据，不支持直接进入 Gate G。".to_owned());
    } else {
        output.push(format!(
            "找到由 {} 个确认配置组成的候选稳定可学习区域，可以进入 Gate G 冻结讨论。",
            stable_region.len(),
        ));
    }
    let strongest_probability = controls.iter().min_by(|left, right| {
        left.formation_probability_change_from_baseline
            .total_cmp(&right.formation_probability_change_from_baseline)
    });
    if let Some(strongest) = strongest_probability
        && strongest.formation_probability_change_from_baseline < 0.0
    {
        output.push(format!(
            "因果对照中 {:?} 造成最大的形成概率变化：{:+.1} 个百分点。",
            strongest.control,
            strongest.formation_probability_change_from_baseline * 100.0,
        ));
    } else if let Some(strongest) = controls.iter().min_by(|left, right| {
        left.mean_probe_score_change_from_baseline
            .total_cmp(&right.mean_probe_score_change_from_baseline)
    }) {
        output.push(format!(
            "确认配置的形成概率均为零；因果对照中 {:?} 造成最大的平均探针得分变化：{:+.1} 个百分点。",
            strongest.control,
            strongest.mean_probe_score_change_from_baseline * 100.0,
        ));
    }
    output
}

fn validate_config(config: Map0ExperimentConfig) -> Result<(), EmbodiedError> {
    let finite = [
        config.eligibility_decay,
        config.hidden_leak,
        config.policy_learning_rate,
        config.softmax_temperature,
        config.reward_baseline_decay,
        config.weight_limit,
        config.recurrent_gain_range[0],
        config.recurrent_gain_range[1],
        config.internal_learning_rate_range[0],
        config.internal_learning_rate_range[1],
        config.homeostasis_strength_range[0],
        config.homeostasis_strength_range[1],
        config.exploration_rate_range[0],
        config.exploration_rate_range[1],
        config.thresholds.minimum_causal_probability_drop,
        config.thresholds.minimum_causal_probe_score_drop,
        config.thresholds.minimum_perturbation_damage_drop,
        config.thresholds.minimum_perturbation_recovery_gain,
    ]
    .into_iter()
    .all(f64::is_finite);
    let ordered_positive = |range: [f64; 2]| range[0] > 0.0 && range[0] < range[1];
    if !finite
        || config.development_config_count < 4
        || config.development_seed_count < 2
        || config.confirmation_seed_count < 2
        || config.confirmation_candidate_count < config.thresholds.minimum_region_size
        || config.confirmation_candidate_count > config.development_config_count
        || config.pretraining_episodes == 0
        || config.adaptation_episodes == 0
        || config.evaluation_episodes < 4
        || !config.evaluation_episodes.is_multiple_of(2)
        || config.threshold_check_interval == 0
        || !config
            .adaptation_episodes
            .is_multiple_of(config.threshold_check_interval)
        || config.cue_steps == 0
        || config.memory_delay_steps == 0
        || !(0.0..1.0).contains(&config.eligibility_decay)
        || !(0.0..1.0).contains(&config.hidden_leak)
        || config.policy_learning_rate <= 0.0
        || config.softmax_temperature <= 0.0
        || !(0.0..1.0).contains(&config.reward_baseline_decay)
        || config.weight_limit <= 0.0
        || !ordered_positive(config.recurrent_gain_range)
        || !ordered_positive(config.internal_learning_rate_range)
        || config.homeostasis_strength_range[0] < 0.0
        || config.homeostasis_strength_range[0] >= config.homeostasis_strength_range[1]
        || config.homeostasis_strength_range[1] > 1.0
        || config.exploration_rate_range[0] < 0.0
        || config.exploration_rate_range[0] >= config.exploration_rate_range[1]
        || config.exploration_rate_range[1] >= 0.5
        || !(0.0..=1.0).contains(&config.thresholds.minimum_causal_probability_drop)
        || !(0.0..=1.0).contains(&config.thresholds.minimum_causal_probe_score_drop)
        || !(0.0..=1.0).contains(&config.thresholds.minimum_perturbation_damage_drop)
        || !(0.0..=1.0).contains(&config.thresholds.minimum_perturbation_recovery_gain)
    {
        return Err(EmbodiedError::InvalidExperiment);
    }
    Ok(())
}
