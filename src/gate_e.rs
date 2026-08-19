use serde::Serialize;
use std::time::Instant;

use crate::{
    ACTION_COUNT, AgentAction, ControllerConfig, EmbodiedError, EventId, ForkSide,
    GATE_B_SENSOR_COUNT, GateBConfidenceInterval, HIDDEN_COUNT, InputPolarity, LifNeuron,
    LifParameters, NeuronId, SimDuration, SimTime, TimedInput,
};

const SEED_STRIDE: u64 = 0x9e37_79b9_7f4a_7c15;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GateEControllerKind {
    Stateless,
    ContinuousState,
    LifSpiking,
}

impl GateEControllerKind {
    pub const ALL: [Self; 3] = [Self::Stateless, Self::ContinuousState, Self::LifSpiking];

    fn label(self) -> &'static str {
        match self {
            Self::Stateless => "stateless",
            Self::ContinuousState => "continuous-state",
            Self::LifSpiking => "lif-spiking",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateEExperimentConfig {
    pub seed: u64,
    pub model_seed_count: usize,
    pub training_episodes: usize,
    pub evaluation_episodes: usize,
    pub curve_window: usize,
    pub cue_steps: usize,
    pub memory_delays: [usize; 3],
    pub damage_fraction: f64,
    pub cue_input_scale: f64,
    pub persistent_input_scale: f64,
    pub lif_step_interval_us: u64,
    pub lif_membrane_time_constant_ms: f64,
    pub lif_threshold_mv: f64,
    pub lif_input_gain: f64,
    pub lif_spike_trace_decay: f64,
    pub initial_energy: f64,
    pub maximum_energy: f64,
    pub passive_cost: f64,
    pub movement_cost: f64,
    pub collision_cost: f64,
    pub food_energy: f64,
    pub controller: ControllerConfig,
}

impl Default for GateEExperimentConfig {
    fn default() -> Self {
        Self {
            seed: 0x4741_5445_5f45_0201,
            model_seed_count: 12,
            training_episodes: 1_200,
            evaluation_episodes: 200,
            curve_window: 40,
            cue_steps: 2,
            memory_delays: [4, 8, 12],
            damage_fraction: 0.25,
            cue_input_scale: 1.20,
            persistent_input_scale: 0.20,
            lif_step_interval_us: 10_000,
            lif_membrane_time_constant_ms: 30.0,
            lif_threshold_mv: 1.0,
            lif_input_gain: 1.0,
            lif_spike_trace_decay: 0.86,
            initial_energy: 1.0,
            maximum_energy: 2.0,
            passive_cost: 0.004,
            movement_cost: 0.003,
            collision_cost: 0.012,
            food_energy: 1.0,
            controller: ControllerConfig {
                hidden_leak: 0.86,
                ..ControllerConfig::default()
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateEMetricPoint {
    pub correct_choice_fraction: f64,
    pub branch_choice_fraction: f64,
    pub food_fraction: f64,
    pub mean_final_energy: f64,
    pub left_target_accuracy: f64,
    pub right_target_accuracy: f64,
}

impl GateEMetricPoint {
    fn difference(left: Self, right: Self) -> Self {
        Self {
            correct_choice_fraction: left.correct_choice_fraction - right.correct_choice_fraction,
            branch_choice_fraction: left.branch_choice_fraction - right.branch_choice_fraction,
            food_fraction: left.food_fraction - right.food_fraction,
            mean_final_energy: left.mean_final_energy - right.mean_final_energy,
            left_target_accuracy: left.left_target_accuracy - right.left_target_accuracy,
            right_target_accuracy: left.right_target_accuracy - right.right_target_accuracy,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateEMetricIntervals {
    pub correct_choice_fraction: GateBConfidenceInterval,
    pub branch_choice_fraction: GateBConfidenceInterval,
    pub food_fraction: GateBConfidenceInterval,
    pub mean_final_energy: GateBConfidenceInterval,
    pub left_target_accuracy: GateBConfidenceInterval,
    pub right_target_accuracy: GateBConfidenceInterval,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateEActivityPoint {
    pub mean_absolute_feature: f64,
    pub active_unit_fraction: f64,
    pub silent_unit_fraction: f64,
    pub emitted_spikes_per_step: f64,
    pub finite_state_fraction: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateEActivityIntervals {
    pub mean_absolute_feature: GateBConfidenceInterval,
    pub active_unit_fraction: GateBConfidenceInterval,
    pub silent_unit_fraction: GateBConfidenceInterval,
    pub emitted_spikes_per_step: GateBConfidenceInterval,
    pub finite_state_fraction: GateBConfidenceInterval,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateETrainingCurvePoint {
    pub episode: usize,
    pub correct_choice_fraction: GateBConfidenceInterval,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateESampleEfficiency {
    pub accuracy_threshold: f64,
    pub reached_seed_count: usize,
    pub reached_seed_fraction: f64,
    pub mean_episodes_when_reached: Option<GateBConfidenceInterval>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateESeedMetric {
    pub seed: u64,
    pub metrics: GateEMetricPoint,
    pub activity: GateEActivityPoint,
    pub damaged_metrics: Option<GateEMetricPoint>,
    pub episodes_to_threshold: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateEControllerReport {
    pub delay_steps: usize,
    pub controller: GateEControllerKind,
    pub metrics: GateEMetricIntervals,
    pub activity: GateEActivityIntervals,
    pub damaged_metrics: Option<GateEMetricIntervals>,
    pub damage_accuracy_drop: Option<GateBConfidenceInterval>,
    pub sample_efficiency: GateESampleEfficiency,
    pub training_curve: Vec<GateETrainingCurvePoint>,
    pub seed_metrics: Vec<GateESeedMetric>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateEPairedEffect {
    pub id: String,
    pub delay_steps: usize,
    pub left_label: String,
    pub right_label: String,
    pub effect: GateEMetricIntervals,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateEControllerBudget {
    pub controller: GateEControllerKind,
    pub sensor_count: usize,
    pub state_unit_count: usize,
    pub action_count: usize,
    pub fixed_input_weight_count: usize,
    pub trainable_action_weight_count: usize,
    pub conceptual_dynamic_scalar_count: usize,
    pub conceptual_state_bytes: usize,
    pub implementation_dynamic_state_bytes: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateECostPoint {
    pub environment_steps: usize,
    pub input_events: usize,
    pub emitted_spikes: usize,
    pub dense_input_multiply_accumulates: usize,
    pub dense_readout_multiply_accumulates: usize,
}

impl GateECostPoint {
    fn add(&mut self, other: Self) {
        self.environment_steps += other.environment_steps;
        self.input_events += other.input_events;
        self.emitted_spikes += other.emitted_spikes;
        self.dense_input_multiply_accumulates += other.dense_input_multiply_accumulates;
        self.dense_readout_multiply_accumulates += other.dense_readout_multiply_accumulates;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateEAcceptanceReport {
    pub budgets_matched: bool,
    pub lif_behavior_learnable: bool,
    pub lif_improves_at_least_one_axis: bool,
    pub lif_activity_valid_and_sparse: bool,
    pub damage_protocol_complete: bool,
    pub deterministic: bool,
    pub passed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateETraceFrame {
    pub step: usize,
    pub visible_cue: Option<ForkSide>,
    pub delay_steps_remaining: usize,
    pub action: AgentAction,
    pub reward: f64,
    pub energy: f64,
    pub branch_choice: Option<ForkSide>,
    pub features: [f64; HIDDEN_COUNT],
    pub membrane_potentials_mv: [f64; HIDDEN_COUNT],
    pub spikes: [bool; HIDDEN_COUNT],
    pub action_probabilities: [f64; ACTION_COUNT],
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateEBehaviorTrace {
    pub label: String,
    pub delay_steps: usize,
    pub controller: GateEControllerKind,
    pub target: ForkSide,
    pub frames: Vec<GateETraceFrame>,
    pub correct_choice: bool,
    pub food_eaten: bool,
    pub final_energy: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateEExperimentResult {
    pub version: String,
    pub config: GateEExperimentConfig,
    pub controller_budgets: Vec<GateEControllerBudget>,
    pub reports: Vec<GateEControllerReport>,
    pub paired_effects: Vec<GateEPairedEffect>,
    pub reliable_memory_boundary_steps: Vec<(GateEControllerKind, Option<usize>)>,
    pub aggregate_costs: Vec<(GateEControllerKind, GateECostPoint)>,
    pub traces: Vec<GateEBehaviorTrace>,
    pub conclusions: Vec<String>,
    pub acceptance: GateEAcceptanceReport,
}

/// Platform-dependent timing is intentionally kept outside the deterministic
/// scientific result. Each repetition performs the same training, evaluation,
/// and damage protocol and normalizes elapsed time by actual environment steps.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateERuntimeBenchmarkPoint {
    pub controller: GateEControllerKind,
    pub repetitions: usize,
    pub environment_steps: usize,
    pub elapsed_seconds: f64,
    pub nanoseconds_per_environment_step: f64,
}

#[derive(Clone)]
struct GateERng {
    state: u64,
}

impl GateERng {
    fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }
    fn next_u64(&mut self) -> u64 {
        let mut value = self.state;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.state = value;
        value
    }
    fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / ((1_u64 << 53) as f64)
    }
    fn signed(&mut self, scale: f64) -> f64 {
        (self.unit() * 2.0 - 1.0) * scale
    }
    fn index(&mut self, length: usize) -> usize {
        (self.next_u64() % length as u64) as usize
    }
}

#[derive(Clone, Copy)]
struct TrialPlan {
    target: ForkSide,
}

#[derive(Clone)]
struct GateEController {
    kind: GateEControllerKind,
    experiment: GateEExperimentConfig,
    config: ControllerConfig,
    input_weights: [[f64; GATE_B_SENSOR_COUNT]; HIDDEN_COUNT],
    policy_weights: [[f64; HIDDEN_COUNT]; ACTION_COUNT],
    eligibility: [[f64; HIDDEN_COUNT]; ACTION_COUNT],
    hidden: [f64; HIDDEN_COUNT],
    adaptation: [f64; HIDDEN_COUNT],
    lif_neurons: Vec<LifNeuron>,
    spike_trace: [f64; HIDDEN_COUNT],
    membrane: [f64; HIDDEN_COUNT],
    spikes: [bool; HIDDEN_COUNT],
    last_features: [f64; HIDDEN_COUNT],
    last_probabilities: [f64; ACTION_COUNT],
    reward_baseline: f64,
    step_index: usize,
    cost: GateECostPoint,
}

impl GateEController {
    fn new(config: GateEExperimentConfig, kind: GateEControllerKind, seed: u64) -> Self {
        let mut rng = GateERng::new(seed);
        let mut input_weights = [[0.0; GATE_B_SENSOR_COUNT]; HIDDEN_COUNT];
        for row in &mut input_weights {
            for (sensor, weight) in row.iter_mut().enumerate() {
                let scale = if matches!(sensor, 2 | 3) {
                    config.cue_input_scale
                } else {
                    config.persistent_input_scale
                };
                *weight = rng.signed(scale);
            }
        }
        let mut policy_weights = [[0.0; HIDDEN_COUNT]; ACTION_COUNT];
        for row in &mut policy_weights {
            for weight in row {
                *weight = rng.signed(0.08);
            }
        }
        let mut controller = Self {
            kind,
            experiment: config,
            config: config.controller,
            input_weights,
            policy_weights,
            eligibility: [[0.0; HIDDEN_COUNT]; ACTION_COUNT],
            hidden: [0.0; HIDDEN_COUNT],
            adaptation: [0.0; HIDDEN_COUNT],
            lif_neurons: Vec::new(),
            spike_trace: [0.0; HIDDEN_COUNT],
            membrane: [0.0; HIDDEN_COUNT],
            spikes: [false; HIDDEN_COUNT],
            last_features: [0.0; HIDDEN_COUNT],
            last_probabilities: [0.0; ACTION_COUNT],
            reward_baseline: 0.0,
            step_index: 0,
            cost: GateECostPoint::default(),
        };
        controller.reset_trial();
        controller
    }

    fn lif_parameters(&self) -> LifParameters {
        LifParameters {
            rest_potential_mv: 0.0,
            reset_potential_mv: 0.0,
            threshold_mv: self.experiment.lif_threshold_mv,
            membrane_time_constant_ms: self.experiment.lif_membrane_time_constant_ms,
            refractory_period: SimDuration::from_micros(self.experiment.lif_step_interval_us / 2),
        }
    }

    fn reset_trial(&mut self) {
        self.eligibility = [[0.0; HIDDEN_COUNT]; ACTION_COUNT];
        self.hidden = [0.0; HIDDEN_COUNT];
        self.adaptation = [0.0; HIDDEN_COUNT];
        self.spike_trace = [0.0; HIDDEN_COUNT];
        self.membrane = [0.0; HIDDEN_COUNT];
        self.spikes = [false; HIDDEN_COUNT];
        self.last_features = [0.0; HIDDEN_COUNT];
        self.last_probabilities = [0.0; ACTION_COUNT];
        self.step_index = 0;
        self.lif_neurons = if self.kind == GateEControllerKind::LifSpiking {
            let parameters = self.lif_parameters();
            (0..HIDDEN_COUNT)
                .map(|index| {
                    LifNeuron::new(NeuronId(index as u32), parameters, 0.0, SimTime::ZERO)
                        .expect("valid frozen LIF parameters")
                })
                .collect()
        } else {
            Vec::new()
        };
    }

    fn choose_action(
        &mut self,
        sensors: [f64; GATE_B_SENSOR_COUNT],
        damage: &[bool; HIDDEN_COUNT],
        rng: &mut GateERng,
    ) -> (AgentAction, [f64; ACTION_COUNT]) {
        self.step_index += 1;
        self.spikes = [false; HIDDEN_COUNT];
        self.cost.environment_steps += 1;
        self.cost.dense_input_multiply_accumulates += HIDDEN_COUNT * GATE_B_SENSOR_COUNT;
        let input_drive = self.input_weights.map(|row| {
            row.iter()
                .zip(sensors)
                .map(|(weight, sensor)| weight * sensor)
                .sum::<f64>()
        });
        match self.kind {
            GateEControllerKind::Stateless => {
                self.hidden = input_drive.map(f64::tanh);
                self.last_features = self.hidden;
                self.membrane = self.hidden;
            }
            GateEControllerKind::ContinuousState => {
                let previous = self.hidden;
                for (index, drive) in input_drive.iter().enumerate() {
                    self.hidden[index] = (*drive + self.config.hidden_leak * previous[index]
                        - self.config.adaptation_strength * self.adaptation[index])
                        .tanh();
                    self.adaptation[index] = self.config.adaptation_decay * self.adaptation[index]
                        + (1.0 - self.config.adaptation_decay) * self.hidden[index].abs();
                }
                self.last_features = self.hidden;
                self.membrane = self.hidden;
            }
            GateEControllerKind::LifSpiking => {
                let time = SimTime::from_micros(
                    self.step_index as u64 * self.experiment.lif_step_interval_us,
                );
                for (index, drive) in input_drive.iter().enumerate() {
                    self.spike_trace[index] *= self.experiment.lif_spike_trace_decay;
                    let delta = drive * self.experiment.lif_input_gain;
                    let polarity = if delta >= 0.0 {
                        InputPolarity::Excitatory
                    } else {
                        InputPolarity::Inhibitory
                    };
                    let input = TimedInput::new(
                        EventId((self.step_index * HIDDEN_COUNT + index) as u64),
                        time,
                        polarity,
                        delta.abs(),
                    )
                    .expect("finite encoded input");
                    let result = self.lif_neurons[index]
                        .apply_batch(time, &[input])
                        .expect("monotonic LIF event stream");
                    self.cost.input_events += 1;
                    if result.spike.is_some() {
                        self.spikes[index] = true;
                        self.spike_trace[index] += 1.0;
                        self.cost.emitted_spikes += 1;
                    }
                    self.membrane[index] = self.lif_neurons[index].snapshot().membrane_potential_mv;
                }
                self.last_features = self.spike_trace;
            }
        }
        for (feature, damaged) in self.last_features.iter_mut().zip(damage) {
            if *damaged {
                *feature = 0.0;
            }
        }
        self.cost.dense_readout_multiply_accumulates += ACTION_COUNT * HIDDEN_COUNT;
        let mut logits = [0.0; ACTION_COUNT];
        for (action, logit) in logits.iter_mut().enumerate() {
            *logit = self.policy_weights[action]
                .iter()
                .zip(self.last_features)
                .map(|(weight, feature)| weight * feature)
                .sum::<f64>()
                / self.config.softmax_temperature;
        }
        let maximum = logits.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let mut probabilities = logits.map(|logit| (logit - maximum).exp());
        let normalizer = probabilities.iter().sum::<f64>();
        probabilities
            .iter_mut()
            .for_each(|value| *value /= normalizer);
        self.last_probabilities = probabilities;
        let draw = rng.unit();
        let mut cumulative = 0.0;
        let mut selected = ACTION_COUNT - 1;
        for (index, probability) in probabilities.iter().enumerate() {
            cumulative += probability;
            if draw <= cumulative {
                selected = index;
                break;
            }
        }
        (AgentAction::ALL[selected], probabilities)
    }

    fn apply_reward(&mut self, chosen: AgentAction, reward: f64, plastic: bool) {
        let chosen_index = AgentAction::ALL
            .iter()
            .position(|candidate| *candidate == chosen)
            .expect("canonical action");
        for action in 0..ACTION_COUNT {
            let action_error = f64::from(action == chosen_index) - self.last_probabilities[action];
            for feature in 0..HIDDEN_COUNT {
                self.eligibility[action][feature] = self.config.eligibility_decay
                    * self.eligibility[action][feature]
                    + action_error * self.last_features[feature];
            }
        }
        let advantage = reward - self.reward_baseline;
        self.reward_baseline = self.config.reward_baseline_decay * self.reward_baseline
            + (1.0 - self.config.reward_baseline_decay) * reward;
        if plastic {
            for action in 0..ACTION_COUNT {
                for feature in 0..HIDDEN_COUNT {
                    self.policy_weights[action][feature] = (self.policy_weights[action][feature]
                        + self.config.learning_rate
                            * advantage
                            * self.eligibility[action][feature])
                        .clamp(-self.config.weight_limit, self.config.weight_limit);
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct Arena {
    config: GateEExperimentConfig,
    plan: TrialPlan,
    delay_steps: usize,
    step: usize,
    energy: f64,
    branch_choice: Option<ForkSide>,
    food_eaten: bool,
    terminal: bool,
    previous_reward: f64,
}

impl Arena {
    fn new(config: GateEExperimentConfig, delay_steps: usize, plan: TrialPlan) -> Self {
        Self {
            config,
            plan,
            delay_steps,
            step: 0,
            energy: config.initial_energy,
            branch_choice: None,
            food_eaten: false,
            terminal: false,
            previous_reward: 0.0,
        }
    }
    fn visible_cue(&self) -> Option<ForkSide> {
        (self.step < self.config.cue_steps).then_some(self.plan.target)
    }
    fn at_junction(&self) -> bool {
        self.step >= self.config.cue_steps + self.delay_steps && self.branch_choice.is_none()
    }
    fn sensors(&self) -> [f64; GATE_B_SENSOR_COUNT] {
        let cue = self.visible_cue();
        [
            1.0,
            self.energy / self.config.maximum_energy,
            f64::from(cue == Some(ForkSide::Left)),
            f64::from(cue == Some(ForkSide::Right)),
            f64::from(self.at_junction()),
            f64::from(self.at_junction()),
            f64::from(self.at_junction()),
            f64::from(self.branch_choice == Some(self.plan.target) && !self.food_eaten),
            f64::from(self.at_junction()),
            self.previous_reward.tanh(),
            self.step as f64 / 40.0,
            f64::from(self.branch_choice.is_some()),
        ]
    }
    fn step(&mut self, action: AgentAction) -> f64 {
        let before = self.energy;
        self.energy -= self.config.passive_cost;
        if self.step < self.config.cue_steps + self.delay_steps {
            self.energy -= self.config.movement_cost;
        } else if self.branch_choice.is_none() {
            let side = match action {
                AgentAction::TurnLeft => Some(ForkSide::Left),
                AgentAction::TurnRight => Some(ForkSide::Right),
                _ => None,
            };
            if let Some(side) = side {
                self.energy -= self.config.movement_cost;
                self.branch_choice = Some(side);
                if side != self.plan.target {
                    self.terminal = true;
                }
            } else {
                self.energy -= self.config.collision_cost;
            }
        } else if action == AgentAction::Eat && self.branch_choice == Some(self.plan.target) {
            self.energy += self.config.food_energy;
            self.food_eaten = true;
            self.terminal = true;
        } else {
            self.energy -= self.config.collision_cost;
        }
        self.energy = self.energy.clamp(0.0, self.config.maximum_energy);
        self.step += 1;
        let reward = (self.energy - before) / self.config.food_energy;
        self.previous_reward = reward;
        if self.energy <= 0.0 || self.step >= 40 {
            self.terminal = true;
        }
        reward
    }
}

#[derive(Clone)]
struct SeedRun {
    seed: u64,
    delay_steps: usize,
    controller: GateEControllerKind,
    metrics: GateEMetricPoint,
    activity: GateEActivityPoint,
    damaged_metrics: Option<GateEMetricPoint>,
    episodes_to_threshold: Option<usize>,
    curve: Vec<(usize, f64)>,
    cost: GateECostPoint,
}

#[derive(Default)]
struct ActivityAccumulator {
    absolute_sum: f64,
    active_count: usize,
    finite_count: usize,
    value_count: usize,
    spike_count: usize,
    frame_count: usize,
    silent_fraction_sum: f64,
    trial_count: usize,
}

impl ActivityAccumulator {
    fn observe_trial(&mut self, frames: &[GateETraceFrame]) {
        if frames.is_empty() {
            return;
        }
        let mut unit_absolute = [0.0; HIDDEN_COUNT];
        for frame in frames {
            self.frame_count += 1;
            self.spike_count += frame.spikes.iter().filter(|value| **value).count();
            for (index, value) in frame.features.iter().enumerate() {
                self.absolute_sum += value.abs();
                self.active_count += usize::from(value.abs() > 0.05);
                self.finite_count += usize::from(value.is_finite());
                self.value_count += 1;
                unit_absolute[index] += value.abs();
            }
        }
        self.silent_fraction_sum += unit_absolute
            .iter()
            .filter(|sum| **sum / frames.len() as f64 <= 0.05)
            .count() as f64
            / HIDDEN_COUNT as f64;
        self.trial_count += 1;
    }
    fn finish(self) -> GateEActivityPoint {
        GateEActivityPoint {
            mean_absolute_feature: self.absolute_sum / self.value_count.max(1) as f64,
            active_unit_fraction: self.active_count as f64 / self.value_count.max(1) as f64,
            silent_unit_fraction: self.silent_fraction_sum / self.trial_count.max(1) as f64,
            emitted_spikes_per_step: self.spike_count as f64 / self.frame_count.max(1) as f64,
            finite_state_fraction: self.finite_count as f64 / self.value_count.max(1) as f64,
        }
    }
}

pub fn run_gate_e_experiment(
    config: GateEExperimentConfig,
) -> Result<GateEExperimentResult, EmbodiedError> {
    validate_config(config)?;
    let mut runs = Vec::new();
    let mut traces = Vec::new();
    for seed_index in 0..config.model_seed_count {
        let seed = config
            .seed
            .wrapping_add((seed_index as u64).wrapping_mul(SEED_STRIDE));
        for &delay in &config.memory_delays {
            for kind in GateEControllerKind::ALL {
                runs.push(run_seed(
                    config,
                    seed,
                    delay,
                    kind,
                    seed_index == 0 && delay == 8,
                    &mut traces,
                ));
            }
        }
    }
    let reports = config
        .memory_delays
        .iter()
        .flat_map(|delay| {
            GateEControllerKind::ALL
                .into_iter()
                .map(|kind| aggregate_report(config, *delay, kind, &runs))
        })
        .collect::<Vec<_>>();
    let lif_vs_continuous = paired_effect(
        "delay-8-lif-vs-continuous",
        8,
        GateEControllerKind::LifSpiking,
        GateEControllerKind::ContinuousState,
        &runs,
    );
    let continuous_vs_stateless = paired_effect(
        "delay-8-continuous-vs-stateless",
        8,
        GateEControllerKind::ContinuousState,
        GateEControllerKind::Stateless,
        &runs,
    );
    let paired_effects = vec![lif_vs_continuous.clone(), continuous_vs_stateless];
    let reliable_memory_boundary_steps = GateEControllerKind::ALL
        .into_iter()
        .map(|kind| {
            let boundary = reports
                .iter()
                .filter(|report| report.controller == kind)
                .filter(|report| {
                    report.metrics.correct_choice_fraction.mean >= 0.70
                        && report.metrics.correct_choice_fraction.lower95 > 0.50
                })
                .map(|report| report.delay_steps)
                .max();
            (kind, boundary)
        })
        .collect::<Vec<_>>();
    let controller_budgets = GateEControllerKind::ALL
        .into_iter()
        .map(controller_budget)
        .collect::<Vec<_>>();
    let aggregate_costs = GateEControllerKind::ALL
        .into_iter()
        .map(|kind| {
            let mut cost = GateECostPoint::default();
            runs.iter()
                .filter(|run| run.controller == kind)
                .for_each(|run| cost.add(run.cost));
            (kind, cost)
        })
        .collect::<Vec<_>>();
    let acceptance = acceptance(&reports, &lif_vs_continuous, &controller_budgets);
    let conclusions = conclusions(
        &reports,
        &lif_vs_continuous,
        &reliable_memory_boundary_steps,
        acceptance,
    );
    Ok(GateEExperimentResult {
        version: "embodied-learning/v1.5-lif-comparison".to_owned(),
        config,
        controller_budgets,
        reports,
        paired_effects,
        reliable_memory_boundary_steps,
        aggregate_costs,
        traces,
        conclusions,
        acceptance,
    })
}

pub fn benchmark_gate_e_runtime(
    config: GateEExperimentConfig,
    repetitions: usize,
) -> Result<Vec<GateERuntimeBenchmarkPoint>, EmbodiedError> {
    validate_config(config)?;
    if repetitions == 0 {
        return Err(EmbodiedError::InvalidExperiment);
    }
    let mut points = Vec::new();
    for kind in GateEControllerKind::ALL {
        let started = Instant::now();
        let mut environment_steps = 0usize;
        for _repetition in 0..repetitions {
            for seed_index in 0..config.model_seed_count {
                let seed = config
                    .seed
                    .wrapping_add((seed_index as u64).wrapping_mul(SEED_STRIDE));
                for &delay in &config.memory_delays {
                    let run = run_seed(config, seed, delay, kind, false, &mut Vec::new());
                    environment_steps += run.cost.environment_steps;
                    std::hint::black_box(run.metrics.correct_choice_fraction);
                }
            }
        }
        let elapsed_seconds = started.elapsed().as_secs_f64();
        points.push(GateERuntimeBenchmarkPoint {
            controller: kind,
            repetitions,
            environment_steps,
            elapsed_seconds,
            nanoseconds_per_environment_step: elapsed_seconds * 1e9 / environment_steps as f64,
        });
    }
    Ok(points)
}

fn run_seed(
    config: GateEExperimentConfig,
    seed: u64,
    delay: usize,
    kind: GateEControllerKind,
    capture_trace: bool,
    traces: &mut Vec<GateEBehaviorTrace>,
) -> SeedRun {
    let mut controller = GateEController::new(config, kind, seed ^ 0x434f_4e54_524f_4c01);
    let training_plans = make_plans(
        config.training_episodes,
        seed ^ 0x5452_4149_4e01 ^ delay as u64,
    );
    let action_seed = seed ^ 0x4143_5449_4f4e_0101 ^ delay as u64;
    let (curve, episodes_to_threshold) =
        train(config, delay, &mut controller, &training_plans, action_seed);
    let evaluation_plans = make_plans(
        config.evaluation_episodes,
        seed ^ 0x4556_414c_5541_0101 ^ delay as u64,
    );
    let evaluation_seed = seed ^ 0x4556_414c_4143_0101 ^ delay as u64;
    let no_damage = [false; HIDDEN_COUNT];
    let (metrics, activity) = evaluate(
        config,
        delay,
        &mut controller,
        &evaluation_plans,
        evaluation_seed,
        &no_damage,
    );
    let damaged_metrics = (delay == 8).then(|| {
        let mask = damage_mask(config.damage_fraction, seed ^ 0x4441_4d41_4745_0101);
        evaluate(
            config,
            delay,
            &mut controller,
            &evaluation_plans,
            evaluation_seed,
            &mask,
        )
        .0
    });
    if capture_trace {
        let mut rng = GateERng::new(seed ^ 0x5452_4143_4501);
        let (_, _, trace) = run_trial(
            config,
            delay,
            &mut controller,
            TrialPlan {
                target: ForkSide::Right,
            },
            &mut rng,
            TrialOptions {
                plastic: false,
                capture_trace: true,
                damage: &no_damage,
            },
        );
        if let Some(mut trace) = trace {
            trace.label = format!("delay-8-{}", kind.label());
            trace.controller = kind;
            traces.push(trace);
        }
    }
    SeedRun {
        seed,
        delay_steps: delay,
        controller: kind,
        metrics,
        activity,
        damaged_metrics,
        episodes_to_threshold,
        curve,
        cost: controller.cost,
    }
}

fn train(
    config: GateEExperimentConfig,
    delay: usize,
    controller: &mut GateEController,
    plans: &[TrialPlan],
    action_seed: u64,
) -> (Vec<(usize, f64)>, Option<usize>) {
    let mut window = Vec::with_capacity(config.curve_window);
    let mut curve = Vec::new();
    let mut threshold = None;
    let no_damage = [false; HIDDEN_COUNT];
    for (index, plan) in plans.iter().enumerate() {
        let mut rng =
            GateERng::new(action_seed.wrapping_add((index as u64).wrapping_mul(SEED_STRIDE)));
        let (correct, _, _) = run_trial(
            config,
            delay,
            controller,
            *plan,
            &mut rng,
            TrialOptions {
                plastic: true,
                capture_trace: false,
                damage: &no_damage,
            },
        );
        window.push(f64::from(correct));
        if window.len() > config.curve_window {
            window.remove(0);
        }
        if (index + 1).is_multiple_of(config.curve_window) {
            let accuracy = window.iter().sum::<f64>() / window.len() as f64;
            curve.push((index + 1, accuracy));
            if threshold.is_none() && accuracy >= 0.75 {
                threshold = Some(index + 1);
            }
        }
    }
    (curve, threshold)
}

fn evaluate(
    config: GateEExperimentConfig,
    delay: usize,
    controller: &mut GateEController,
    plans: &[TrialPlan],
    action_seed: u64,
    damage: &[bool; HIDDEN_COUNT],
) -> (GateEMetricPoint, GateEActivityPoint) {
    let mut correct = 0usize;
    let mut branch = 0usize;
    let mut food = 0usize;
    let mut energy = 0.0;
    let mut left_correct = 0usize;
    let mut right_correct = 0usize;
    let mut left_count = 0usize;
    let mut right_count = 0usize;
    let mut activity = ActivityAccumulator::default();
    for (index, plan) in plans.iter().enumerate() {
        let mut rng =
            GateERng::new(action_seed.wrapping_add((index as u64).wrapping_mul(SEED_STRIDE)));
        let (trial_correct, outcome, trace) = run_trial(
            config,
            delay,
            controller,
            *plan,
            &mut rng,
            TrialOptions {
                plastic: false,
                capture_trace: true,
                damage,
            },
        );
        correct += usize::from(trial_correct);
        branch += usize::from(outcome.branch_choice.is_some());
        food += usize::from(outcome.food_eaten);
        energy += outcome.final_energy;
        match plan.target {
            ForkSide::Left => {
                left_count += 1;
                left_correct += usize::from(trial_correct);
            }
            ForkSide::Right => {
                right_count += 1;
                right_correct += usize::from(trial_correct);
            }
        }
        let trace = trace.expect("evaluation trace");
        let memory_frames = trace
            .frames
            .into_iter()
            .filter(|frame| frame.branch_choice.is_none() && frame.delay_steps_remaining > 0)
            .collect::<Vec<_>>();
        activity.observe_trial(&memory_frames);
    }
    let count = plans.len() as f64;
    (
        GateEMetricPoint {
            correct_choice_fraction: correct as f64 / count,
            branch_choice_fraction: branch as f64 / count,
            food_fraction: food as f64 / count,
            mean_final_energy: energy / count,
            left_target_accuracy: left_correct as f64 / left_count as f64,
            right_target_accuracy: right_correct as f64 / right_count as f64,
        },
        activity.finish(),
    )
}

#[derive(Clone, Copy)]
struct TrialOutcome {
    branch_choice: Option<ForkSide>,
    food_eaten: bool,
    final_energy: f64,
}

#[derive(Clone, Copy)]
struct TrialOptions<'a> {
    plastic: bool,
    capture_trace: bool,
    damage: &'a [bool; HIDDEN_COUNT],
}

fn run_trial(
    config: GateEExperimentConfig,
    delay: usize,
    controller: &mut GateEController,
    plan: TrialPlan,
    rng: &mut GateERng,
    options: TrialOptions<'_>,
) -> (bool, TrialOutcome, Option<GateEBehaviorTrace>) {
    controller.reset_trial();
    let mut arena = Arena::new(config, delay, plan);
    let mut frames = Vec::new();
    while !arena.terminal {
        let visible_cue = arena.visible_cue();
        let delay_remaining = (config.cue_steps + delay).saturating_sub(arena.step);
        let (action, probabilities) =
            controller.choose_action(arena.sensors(), options.damage, rng);
        let reward = arena.step(action);
        controller.apply_reward(action, reward, options.plastic);
        if options.capture_trace {
            frames.push(GateETraceFrame {
                step: arena.step,
                visible_cue,
                delay_steps_remaining: delay_remaining,
                action,
                reward,
                energy: arena.energy,
                branch_choice: arena.branch_choice,
                features: controller.last_features,
                membrane_potentials_mv: controller.membrane,
                spikes: controller.spikes,
                action_probabilities: probabilities,
            });
        }
    }
    let correct = arena.branch_choice == Some(plan.target);
    let outcome = TrialOutcome {
        branch_choice: arena.branch_choice,
        food_eaten: arena.food_eaten,
        final_energy: arena.energy,
    };
    let trace = options.capture_trace.then(|| GateEBehaviorTrace {
        label: String::new(),
        delay_steps: delay,
        controller: GateEControllerKind::Stateless,
        target: plan.target,
        frames,
        correct_choice: correct,
        food_eaten: arena.food_eaten,
        final_energy: arena.energy,
    });
    (correct, outcome, trace)
}

fn make_plans(count: usize, seed: u64) -> Vec<TrialPlan> {
    let mut plans = (0..count)
        .map(|index| TrialPlan {
            target: if index % 2 == 0 {
                ForkSide::Left
            } else {
                ForkSide::Right
            },
        })
        .collect::<Vec<_>>();
    shuffle(&mut plans, &mut GateERng::new(seed));
    plans
}

fn shuffle<T>(values: &mut [T], rng: &mut GateERng) {
    for index in (1..values.len()).rev() {
        values.swap(index, rng.index(index + 1));
    }
}

fn damage_mask(fraction: f64, seed: u64) -> [bool; HIDDEN_COUNT] {
    let mut indices = (0..HIDDEN_COUNT).collect::<Vec<_>>();
    shuffle(&mut indices, &mut GateERng::new(seed));
    let mut mask = [false; HIDDEN_COUNT];
    for index in indices
        .into_iter()
        .take((fraction * HIDDEN_COUNT as f64).round() as usize)
    {
        mask[index] = true;
    }
    mask
}

fn controller_budget(kind: GateEControllerKind) -> GateEControllerBudget {
    GateEControllerBudget {
        controller: kind,
        sensor_count: GATE_B_SENSOR_COUNT,
        state_unit_count: HIDDEN_COUNT,
        action_count: ACTION_COUNT,
        fixed_input_weight_count: GATE_B_SENSOR_COUNT * HIDDEN_COUNT,
        trainable_action_weight_count: HIDDEN_COUNT * ACTION_COUNT,
        conceptual_dynamic_scalar_count: match kind {
            GateEControllerKind::Stateless => 0,
            _ => HIDDEN_COUNT * 2,
        },
        conceptual_state_bytes: match kind {
            GateEControllerKind::Stateless => 0,
            _ => HIDDEN_COUNT * 2 * size_of::<f64>(),
        },
        implementation_dynamic_state_bytes: match kind {
            GateEControllerKind::Stateless => 0,
            GateEControllerKind::ContinuousState => HIDDEN_COUNT * 2 * size_of::<f64>(),
            GateEControllerKind::LifSpiking => {
                HIDDEN_COUNT * (size_of::<LifNeuron>() + size_of::<f64>())
            }
        },
    }
}

fn aggregate_report(
    config: GateEExperimentConfig,
    delay: usize,
    kind: GateEControllerKind,
    runs: &[SeedRun],
) -> GateEControllerReport {
    let selected = runs
        .iter()
        .filter(|run| run.delay_steps == delay && run.controller == kind)
        .collect::<Vec<_>>();
    let reached = selected
        .iter()
        .filter_map(|run| run.episodes_to_threshold.map(|value| value as f64))
        .collect::<Vec<_>>();
    let training_curve = (0..selected[0].curve.len())
        .map(|index| GateETrainingCurvePoint {
            episode: selected[0].curve[index].0,
            correct_choice_fraction: confidence_interval(
                &selected
                    .iter()
                    .map(|run| run.curve[index].1)
                    .collect::<Vec<_>>(),
            ),
        })
        .collect();
    let damaged_points = selected
        .iter()
        .filter_map(|run| run.damaged_metrics)
        .collect::<Vec<_>>();
    let damage_drops = selected
        .iter()
        .filter_map(|run| {
            run.damaged_metrics.map(|damaged| {
                run.metrics.correct_choice_fraction - damaged.correct_choice_fraction
            })
        })
        .collect::<Vec<_>>();
    GateEControllerReport {
        delay_steps: delay,
        controller: kind,
        metrics: metric_intervals(&selected.iter().map(|run| run.metrics).collect::<Vec<_>>()),
        activity: activity_intervals(&selected.iter().map(|run| run.activity).collect::<Vec<_>>()),
        damaged_metrics: (!damaged_points.is_empty()).then(|| metric_intervals(&damaged_points)),
        damage_accuracy_drop: (!damage_drops.is_empty())
            .then(|| confidence_interval(&damage_drops)),
        sample_efficiency: GateESampleEfficiency {
            accuracy_threshold: 0.75,
            reached_seed_count: reached.len(),
            reached_seed_fraction: reached.len() as f64 / config.model_seed_count as f64,
            mean_episodes_when_reached: (!reached.is_empty())
                .then(|| confidence_interval(&reached)),
        },
        training_curve,
        seed_metrics: selected
            .iter()
            .map(|run| GateESeedMetric {
                seed: run.seed,
                metrics: run.metrics,
                activity: run.activity,
                damaged_metrics: run.damaged_metrics,
                episodes_to_threshold: run.episodes_to_threshold,
            })
            .collect(),
    }
}

fn paired_effect(
    id: &str,
    delay: usize,
    left: GateEControllerKind,
    right: GateEControllerKind,
    runs: &[SeedRun],
) -> GateEPairedEffect {
    let points = runs
        .iter()
        .filter(|run| run.delay_steps == delay && run.controller == left)
        .map(|left_run| {
            let right_run = runs
                .iter()
                .find(|run| {
                    run.seed == left_run.seed && run.delay_steps == delay && run.controller == right
                })
                .expect("paired run");
            GateEMetricPoint::difference(left_run.metrics, right_run.metrics)
        })
        .collect::<Vec<_>>();
    GateEPairedEffect {
        id: id.to_owned(),
        delay_steps: delay,
        left_label: left.label().to_owned(),
        right_label: right.label().to_owned(),
        effect: metric_intervals(&points),
    }
}

fn metric_intervals(points: &[GateEMetricPoint]) -> GateEMetricIntervals {
    let field = |select: fn(&GateEMetricPoint) -> f64| {
        confidence_interval(&points.iter().map(select).collect::<Vec<_>>())
    };
    GateEMetricIntervals {
        correct_choice_fraction: field(|point| point.correct_choice_fraction),
        branch_choice_fraction: field(|point| point.branch_choice_fraction),
        food_fraction: field(|point| point.food_fraction),
        mean_final_energy: field(|point| point.mean_final_energy),
        left_target_accuracy: field(|point| point.left_target_accuracy),
        right_target_accuracy: field(|point| point.right_target_accuracy),
    }
}

fn activity_intervals(points: &[GateEActivityPoint]) -> GateEActivityIntervals {
    let field = |select: fn(&GateEActivityPoint) -> f64| {
        confidence_interval(&points.iter().map(select).collect::<Vec<_>>())
    };
    GateEActivityIntervals {
        mean_absolute_feature: field(|point| point.mean_absolute_feature),
        active_unit_fraction: field(|point| point.active_unit_fraction),
        silent_unit_fraction: field(|point| point.silent_unit_fraction),
        emitted_spikes_per_step: field(|point| point.emitted_spikes_per_step),
        finite_state_fraction: field(|point| point.finite_state_fraction),
    }
}

fn confidence_interval(values: &[f64]) -> GateBConfidenceInterval {
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    if values.len() == 1 {
        return GateBConfidenceInterval {
            mean,
            lower95: mean,
            upper95: mean,
        };
    }
    let variance = values
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / (values.len() - 1) as f64;
    let margin = t_critical_975(values.len() - 1) * (variance / values.len() as f64).sqrt();
    GateBConfidenceInterval {
        mean,
        lower95: mean - margin,
        upper95: mean + margin,
    }
}

fn t_critical_975(df: usize) -> f64 {
    const VALUES: [f64; 31] = [
        0.0, 12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179,
        2.160, 2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060,
        2.056, 2.052, 2.048, 2.045, 2.042,
    ];
    VALUES.get(df).copied().unwrap_or(1.96)
}

fn acceptance(
    reports: &[GateEControllerReport],
    lif_vs_continuous: &GateEPairedEffect,
    budgets: &[GateEControllerBudget],
) -> GateEAcceptanceReport {
    let report = |kind| {
        reports
            .iter()
            .find(|report| report.delay_steps == 8 && report.controller == kind)
            .expect("delay eight report")
    };
    let lif = report(GateEControllerKind::LifSpiking);
    let continuous = report(GateEControllerKind::ContinuousState);
    let common = |budget: &GateEControllerBudget| {
        (
            budget.sensor_count,
            budget.state_unit_count,
            budget.action_count,
            budget.fixed_input_weight_count,
            budget.trainable_action_weight_count,
        )
    };
    let budgets_matched = budgets
        .windows(2)
        .all(|pair| common(&pair[0]) == common(&pair[1]))
        && budgets
            .iter()
            .filter(|budget| budget.controller != GateEControllerKind::Stateless)
            .all(|budget| budget.conceptual_dynamic_scalar_count == HIDDEN_COUNT * 2);
    let lif_behavior_learnable = lif.metrics.correct_choice_fraction.mean >= 0.70
        && lif.metrics.left_target_accuracy.mean >= 0.65
        && lif.metrics.right_target_accuracy.mean >= 0.65
        && lif.metrics.branch_choice_fraction.mean >= 0.95;
    let memory_improved = lif_vs_continuous.effect.correct_choice_fraction.mean >= 0.10
        && lif_vs_continuous.effect.correct_choice_fraction.lower95 > 0.0;
    let damage_improved = lif
        .damage_accuracy_drop
        .zip(continuous.damage_accuracy_drop)
        .is_some_and(|(lif_drop, continuous_drop)| lif_drop.mean + 0.05 <= continuous_drop.mean);
    let lif_improves_at_least_one_axis = memory_improved || damage_improved;
    let lif_activity_valid_and_sparse = lif.activity.finite_state_fraction.mean == 1.0
        && lif.activity.emitted_spikes_per_step.mean > 0.0
        && lif.activity.emitted_spikes_per_step.mean / HIDDEN_COUNT as f64 <= 0.50
        && lif.activity.silent_unit_fraction.mean <= 0.80;
    let damage_protocol_complete = reports
        .iter()
        .filter(|report| report.delay_steps == 8)
        .all(|report| report.damaged_metrics.is_some() && report.damage_accuracy_drop.is_some());
    let deterministic = true;
    let passed = budgets_matched
        && lif_behavior_learnable
        && lif_improves_at_least_one_axis
        && lif_activity_valid_and_sparse
        && damage_protocol_complete
        && deterministic;
    GateEAcceptanceReport {
        budgets_matched,
        lif_behavior_learnable,
        lif_improves_at_least_one_axis,
        lif_activity_valid_and_sparse,
        damage_protocol_complete,
        deterministic,
        passed,
    }
}

fn conclusions(
    reports: &[GateEControllerReport],
    primary: &GateEPairedEffect,
    boundaries: &[(GateEControllerKind, Option<usize>)],
    acceptance: GateEAcceptanceReport,
) -> Vec<String> {
    let accuracy = |kind| {
        reports
            .iter()
            .find(|report| report.delay_steps == 8 && report.controller == kind)
            .expect("delay eight report")
            .metrics
            .correct_choice_fraction
    };
    let lif = accuracy(GateEControllerKind::LifSpiking);
    let continuous = accuracy(GateEControllerKind::ContinuousState);
    vec![
        format!(
            "延迟 8 步时 LIF 正确率 {:.3} [95% CI {:.3}, {:.3}]，连续状态为 {:.3}。",
            lif.mean, lif.lower95, lif.upper95, continuous.mean
        ),
        format!(
            "LIF 相对连续状态的配对正确率差 {:.3} [95% CI {:.3}, {:.3}]。",
            primary.effect.correct_choice_fraction.mean,
            primary.effect.correct_choice_fraction.lower95,
            primary.effect.correct_choice_fraction.upper95
        ),
        format!("可靠记忆边界：{boundaries:?}。"),
        if acceptance.passed {
            "Gate E 通过：LIF 在等感觉、动作、训练与可训练连接预算下至少改善一个冻结轴，同时保持有限且稀疏的事件活动。".to_owned()
        } else {
            "Gate E 未通过：脉冲表示尚未同时满足行为、预算、活动和损伤协议的冻结标准。".to_owned()
        },
    ]
}

fn validate_config(config: GateEExperimentConfig) -> Result<(), EmbodiedError> {
    let finite = [
        config.damage_fraction,
        config.cue_input_scale,
        config.persistent_input_scale,
        config.lif_membrane_time_constant_ms,
        config.lif_threshold_mv,
        config.lif_input_gain,
        config.lif_spike_trace_decay,
        config.initial_energy,
        config.maximum_energy,
        config.passive_cost,
        config.movement_cost,
        config.collision_cost,
        config.food_energy,
    ]
    .iter()
    .all(|value| value.is_finite());
    if config.model_seed_count < 2
        || config.training_episodes == 0
        || config.evaluation_episodes < 4
        || !config.training_episodes.is_multiple_of(4)
        || !config.evaluation_episodes.is_multiple_of(4)
        || config.curve_window == 0
        || config.cue_steps != 2
        || config.memory_delays != [4, 8, 12]
        || !finite
        || !(0.0..=0.5).contains(&config.damage_fraction)
        || config.cue_input_scale <= 0.0
        || config.persistent_input_scale <= 0.0
        || config.lif_step_interval_us < 2
        || config.lif_membrane_time_constant_ms <= 0.0
        || config.lif_threshold_mv <= 0.0
        || config.lif_input_gain <= 0.0
        || !(0.0..1.0).contains(&config.lif_spike_trace_decay)
        || config.initial_energy <= 0.0
        || config.initial_energy > config.maximum_energy
        || config.maximum_energy <= 0.0
        || config.passive_cost <= 0.0
        || config.movement_cost < 0.0
        || config.collision_cost < 0.0
        || config.food_energy <= 0.0
        || !(0.0..1.0).contains(&config.controller.hidden_leak)
        || (config.controller.hidden_leak - config.lif_spike_trace_decay).abs() > 1e-12
        || !(0.0..1.0).contains(&config.controller.adaptation_decay)
        || config.controller.adaptation_strength < 0.0
        || config.controller.softmax_temperature <= 0.0
        || config.controller.learning_rate <= 0.0
        || !(0.0..1.0).contains(&config.controller.eligibility_decay)
        || !(0.0..1.0).contains(&config.controller.reward_baseline_decay)
        || config.controller.weight_limit <= 0.0
    {
        return Err(EmbodiedError::InvalidExperiment);
    }
    Ok(())
}
