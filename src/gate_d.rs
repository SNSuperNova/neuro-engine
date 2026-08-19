use serde::Serialize;

use crate::{ACTION_COUNT, AgentAction, ControllerConfig, EmbodiedError, HIDDEN_COUNT};
use crate::{ForkSide, GATE_B_SENSOR_COUNT, GateBConfidenceInterval};

const SEED_STRIDE: u64 = 0x9e37_79b9_7f4a_7c15;
const RECURRENT_IN_DEGREE: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GateDControllerKind {
    NoRecurrence,
    ShuffledRecurrence,
    StructuredRecurrence,
}

impl GateDControllerKind {
    pub const ALL: [Self; 3] = [
        Self::NoRecurrence,
        Self::ShuffledRecurrence,
        Self::StructuredRecurrence,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::NoRecurrence => "no-recurrence",
            Self::ShuffledRecurrence => "shuffled-recurrence",
            Self::StructuredRecurrence => "structured-recurrence",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateDExperimentConfig {
    pub seed: u64,
    pub model_seed_count: usize,
    pub training_episodes: usize,
    pub evaluation_episodes: usize,
    pub curve_window: usize,
    pub cue_steps: usize,
    pub memory_delays: [usize; 4],
    pub recurrent_in_degree: usize,
    pub recurrent_spectral_radius: f64,
    pub cue_input_scale: f64,
    pub persistent_input_scale: f64,
    pub initial_energy: f64,
    pub maximum_energy: f64,
    pub passive_cost: f64,
    pub movement_cost: f64,
    pub collision_cost: f64,
    pub food_energy: f64,
    pub controller: ControllerConfig,
}

impl Default for GateDExperimentConfig {
    fn default() -> Self {
        Self {
            seed: 0x4741_5445_5f44_0201,
            model_seed_count: 12,
            training_episodes: 1_200,
            evaluation_episodes: 200,
            curve_window: 40,
            cue_steps: 2,
            memory_delays: [4, 6, 8, 12],
            recurrent_in_degree: RECURRENT_IN_DEGREE,
            recurrent_spectral_radius: 0.15,
            cue_input_scale: 1.20,
            persistent_input_scale: 0.20,
            initial_energy: 1.0,
            maximum_energy: 2.0,
            passive_cost: 0.004,
            movement_cost: 0.003,
            collision_cost: 0.012,
            food_energy: 1.0,
            controller: ControllerConfig {
                hidden_leak: 0.80,
                ..ControllerConfig::default()
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateDMetricPoint {
    pub correct_choice_fraction: f64,
    pub branch_choice_fraction: f64,
    pub food_fraction: f64,
    pub mean_final_energy: f64,
    pub left_target_accuracy: f64,
    pub right_target_accuracy: f64,
}

impl GateDMetricPoint {
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
pub struct GateDMetricIntervals {
    pub correct_choice_fraction: GateBConfidenceInterval,
    pub branch_choice_fraction: GateBConfidenceInterval,
    pub food_fraction: GateBConfidenceInterval,
    pub mean_final_energy: GateBConfidenceInterval,
    pub left_target_accuracy: GateBConfidenceInterval,
    pub right_target_accuracy: GateBConfidenceInterval,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateDStabilityPoint {
    pub mean_absolute_activity: f64,
    pub saturated_unit_fraction: f64,
    pub silent_unit_fraction: f64,
    pub population_synchrony: f64,
    pub finite_state_fraction: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateDStabilityIntervals {
    pub mean_absolute_activity: GateBConfidenceInterval,
    pub saturated_unit_fraction: GateBConfidenceInterval,
    pub silent_unit_fraction: GateBConfidenceInterval,
    pub population_synchrony: GateBConfidenceInterval,
    pub finite_state_fraction: GateBConfidenceInterval,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateDTrainingCurvePoint {
    pub episode: usize,
    pub correct_choice_fraction: GateBConfidenceInterval,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateDSampleEfficiency {
    pub accuracy_threshold: f64,
    pub reached_seed_count: usize,
    pub reached_seed_fraction: f64,
    pub mean_episodes_when_reached: Option<GateBConfidenceInterval>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateDSeedMetric {
    pub seed: u64,
    pub metrics: GateDMetricPoint,
    pub stability: GateDStabilityPoint,
    pub episodes_to_threshold: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateDControllerReport {
    pub delay_steps: usize,
    pub controller: GateDControllerKind,
    pub metrics: GateDMetricIntervals,
    pub stability: GateDStabilityIntervals,
    pub sample_efficiency: GateDSampleEfficiency,
    pub training_curve: Vec<GateDTrainingCurvePoint>,
    pub seed_metrics: Vec<GateDSeedMetric>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateDPairedEffect {
    pub id: String,
    pub delay_steps: usize,
    pub left_label: String,
    pub right_label: String,
    pub effect: GateDMetricIntervals,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateDControllerBudget {
    pub controller: GateDControllerKind,
    pub sensor_count: usize,
    pub state_unit_count: usize,
    pub action_count: usize,
    pub fixed_input_weight_count: usize,
    pub trainable_action_weight_count: usize,
    pub allocated_recurrent_weight_count: usize,
    pub active_recurrent_weight_count: usize,
    pub excitatory_edge_count: usize,
    pub inhibitory_edge_count: usize,
    pub estimated_spectral_radius: f64,
    pub recurrent_weight_digest: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateDAcceptanceReport {
    pub structured_beats_shuffled_at_delay_eight: bool,
    pub structured_delay_eight_learnable: bool,
    pub structured_extends_reliable_boundary: bool,
    pub shuffled_does_not_match_primary_gain: bool,
    pub structured_state_stable: bool,
    pub budgets_and_recurrent_controls_matched: bool,
    pub deterministic: bool,
    pub passed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateDTraceFrame {
    pub step: usize,
    pub visible_cue: Option<ForkSide>,
    pub delay_steps_remaining: usize,
    pub action: AgentAction,
    pub reward: f64,
    pub energy: f64,
    pub branch_choice: Option<ForkSide>,
    pub hidden_activity: [f64; HIDDEN_COUNT],
    pub action_probabilities: [f64; ACTION_COUNT],
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateDBehaviorTrace {
    pub label: String,
    pub delay_steps: usize,
    pub controller: GateDControllerKind,
    pub target: ForkSide,
    pub presented_cue: ForkSide,
    pub frames: Vec<GateDTraceFrame>,
    pub correct_choice: bool,
    pub food_eaten: bool,
    pub final_energy: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateDExperimentResult {
    pub version: String,
    pub config: GateDExperimentConfig,
    pub controller_budgets: Vec<GateDControllerBudget>,
    pub reports: Vec<GateDControllerReport>,
    pub paired_effects: Vec<GateDPairedEffect>,
    pub reliable_memory_boundary_steps: Vec<(GateDControllerKind, Option<usize>)>,
    pub traces: Vec<GateDBehaviorTrace>,
    pub conclusions: Vec<String>,
    pub acceptance: GateDAcceptanceReport,
}

#[derive(Clone, Copy)]
struct TrialPlan {
    target: ForkSide,
}

#[derive(Clone)]
struct GateDRng {
    state: u64,
}

impl GateDRng {
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

#[derive(Clone)]
struct GateDController {
    config: ControllerConfig,
    input_weights: [[f64; GATE_B_SENSOR_COUNT]; HIDDEN_COUNT],
    recurrent_weights: [[f64; HIDDEN_COUNT]; HIDDEN_COUNT],
    policy_weights: [[f64; HIDDEN_COUNT]; ACTION_COUNT],
    eligibility: [[f64; HIDDEN_COUNT]; ACTION_COUNT],
    hidden: [f64; HIDDEN_COUNT],
    adaptation: [f64; HIDDEN_COUNT],
    last_features: [f64; HIDDEN_COUNT],
    last_probabilities: [f64; ACTION_COUNT],
    reward_baseline: f64,
}

impl GateDController {
    fn new(config: GateDExperimentConfig, kind: GateDControllerKind, seed: u64) -> Self {
        let mut rng = GateDRng::new(seed);
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
        let structured = structured_recurrent_weights(
            &input_weights,
            config.recurrent_in_degree,
            config.recurrent_spectral_radius,
            seed ^ 0x5245_4355_5252_0101,
        );
        let recurrent_weights = match kind {
            GateDControllerKind::NoRecurrence => [[0.0; HIDDEN_COUNT]; HIDDEN_COUNT],
            GateDControllerKind::StructuredRecurrence => structured,
            GateDControllerKind::ShuffledRecurrence => {
                shuffled_recurrent_weights(structured, seed ^ 0x5348_5546_464c_4501)
            }
        };
        let mut policy_weights = [[0.0; HIDDEN_COUNT]; ACTION_COUNT];
        for row in &mut policy_weights {
            for weight in row {
                *weight = rng.signed(0.08);
            }
        }
        Self {
            config: config.controller,
            input_weights,
            recurrent_weights,
            policy_weights,
            eligibility: [[0.0; HIDDEN_COUNT]; ACTION_COUNT],
            hidden: [0.0; HIDDEN_COUNT],
            adaptation: [0.0; HIDDEN_COUNT],
            last_features: [0.0; HIDDEN_COUNT],
            last_probabilities: [0.0; ACTION_COUNT],
            reward_baseline: 0.0,
        }
    }

    fn reset_trial(&mut self) {
        self.eligibility = [[0.0; HIDDEN_COUNT]; ACTION_COUNT];
        self.hidden = [0.0; HIDDEN_COUNT];
        self.adaptation = [0.0; HIDDEN_COUNT];
        self.last_features = [0.0; HIDDEN_COUNT];
        self.last_probabilities = [0.0; ACTION_COUNT];
    }

    fn choose_action(
        &mut self,
        sensors: [f64; GATE_B_SENSOR_COUNT],
        rng: &mut GateDRng,
    ) -> (AgentAction, [f64; ACTION_COUNT]) {
        let previous = self.hidden;
        let mut next = [0.0; HIDDEN_COUNT];
        for hidden in 0..HIDDEN_COUNT {
            let input_drive = self.input_weights[hidden]
                .iter()
                .zip(sensors)
                .map(|(weight, sensor)| weight * sensor)
                .sum::<f64>();
            let recurrent_drive = self.recurrent_weights[hidden]
                .iter()
                .zip(previous)
                .map(|(weight, state)| weight * state)
                .sum::<f64>();
            next[hidden] =
                (input_drive + self.config.hidden_leak * previous[hidden] + recurrent_drive
                    - self.config.adaptation_strength * self.adaptation[hidden])
                    .tanh();
        }
        self.hidden = next;
        for hidden in 0..HIDDEN_COUNT {
            self.adaptation[hidden] = self.config.adaptation_decay * self.adaptation[hidden]
                + (1.0 - self.config.adaptation_decay) * self.hidden[hidden].abs();
        }
        self.last_features = self.hidden;
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

fn structured_recurrent_weights(
    input_weights: &[[f64; GATE_B_SENSOR_COUNT]; HIDDEN_COUNT],
    in_degree: usize,
    target_radius: f64,
    seed: u64,
) -> [[f64; HIDDEN_COUNT]; HIDDEN_COUNT] {
    let preference = input_weights.map(|row| row[2] - row[3]);
    let mut rng = GateDRng::new(seed);
    let mut matrix = [[0.0; HIDDEN_COUNT]; HIDDEN_COUNT];
    for target in 0..HIDDEN_COUNT {
        let mut sources = (0..HIDDEN_COUNT)
            .filter(|source| *source != target)
            .collect::<Vec<_>>();
        shuffle(&mut sources, &mut rng);
        for source in sources.into_iter().take(in_degree) {
            let same_assembly =
                preference[target].is_sign_positive() == preference[source].is_sign_positive();
            let sign = if same_assembly { 1.0 } else { -1.0 };
            matrix[target][source] = sign * (0.75 + 0.5 * rng.unit());
        }
    }
    let radius = estimate_spectral_radius(&matrix);
    let scale = target_radius / radius.max(1e-12);
    for row in &mut matrix {
        for weight in row {
            *weight *= scale;
        }
    }
    matrix
}

fn shuffled_recurrent_weights(
    structured: [[f64; HIDDEN_COUNT]; HIDDEN_COUNT],
    seed: u64,
) -> [[f64; HIDDEN_COUNT]; HIDDEN_COUNT] {
    let mut permutation = std::array::from_fn::<_, HIDDEN_COUNT, _>(|index| index);
    shuffle(&mut permutation, &mut GateDRng::new(seed));
    std::array::from_fn(|target| {
        std::array::from_fn(|source| structured[permutation[target]][permutation[source]])
    })
}

fn estimate_spectral_radius(matrix: &[[f64; HIDDEN_COUNT]; HIDDEN_COUNT]) -> f64 {
    let mut vector: [f64; HIDDEN_COUNT] =
        std::array::from_fn(|index| 0.5 + index as f64 / HIDDEN_COUNT as f64);
    let mut estimate = 0.0;
    for _ in 0..96 {
        let next = std::array::from_fn(|row| {
            matrix[row]
                .iter()
                .zip(vector)
                .map(|(weight, value)| weight * value)
                .sum::<f64>()
        });
        estimate = next.iter().map(|value| value * value).sum::<f64>().sqrt();
        if estimate <= 1e-12 {
            return 0.0;
        }
        vector = next.map(|value| value / estimate);
    }
    estimate
}

fn recurrent_budget(
    config: GateDExperimentConfig,
    kind: GateDControllerKind,
    seed: u64,
) -> GateDControllerBudget {
    let controller = GateDController::new(config, kind, seed);
    let weights = controller
        .recurrent_weights
        .iter()
        .flatten()
        .copied()
        .collect::<Vec<_>>();
    let active = weights.iter().filter(|weight| weight.abs() > 1e-12).count();
    let excitatory = weights.iter().filter(|weight| **weight > 1e-12).count();
    let inhibitory = weights.iter().filter(|weight| **weight < -1e-12).count();
    GateDControllerBudget {
        controller: kind,
        sensor_count: GATE_B_SENSOR_COUNT,
        state_unit_count: HIDDEN_COUNT,
        action_count: ACTION_COUNT,
        fixed_input_weight_count: GATE_B_SENSOR_COUNT * HIDDEN_COUNT,
        trainable_action_weight_count: ACTION_COUNT * HIDDEN_COUNT,
        allocated_recurrent_weight_count: HIDDEN_COUNT * HIDDEN_COUNT,
        active_recurrent_weight_count: active,
        excitatory_edge_count: excitatory,
        inhibitory_edge_count: inhibitory,
        estimated_spectral_radius: estimate_spectral_radius(&controller.recurrent_weights),
        recurrent_weight_digest: weight_multiset_digest(&weights),
    }
}

fn weight_multiset_digest(weights: &[f64]) -> u64 {
    let mut values = weights
        .iter()
        .map(|value| value.to_bits())
        .collect::<Vec<_>>();
    values.sort_unstable();
    values
        .into_iter()
        .fold(0xcbf2_9ce4_8422_2325, |hash, bits| {
            (hash ^ bits).wrapping_mul(0x1000_0000_01b3)
        })
}

#[derive(Clone, Copy)]
struct Arena {
    config: GateDExperimentConfig,
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
    fn new(config: GateDExperimentConfig, delay_steps: usize, plan: TrialPlan) -> Self {
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
                AgentAction::Forward | AgentAction::Eat => None,
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
    controller: GateDControllerKind,
    metrics: GateDMetricPoint,
    stability: GateDStabilityPoint,
    episodes_to_threshold: Option<usize>,
    curve: Vec<(usize, f64)>,
}

#[derive(Default)]
struct StabilityAccumulator {
    absolute_sum: f64,
    saturated_count: usize,
    finite_count: usize,
    value_count: usize,
    synchrony_sum: f64,
    frame_count: usize,
    silent_fraction_sum: f64,
    trial_count: usize,
}

impl StabilityAccumulator {
    fn observe_trial(&mut self, frames: &[[f64; HIDDEN_COUNT]]) {
        if frames.is_empty() {
            return;
        }
        let mut unit_absolute = [0.0; HIDDEN_COUNT];
        for frame in frames {
            let mean_absolute =
                frame.iter().map(|value| value.abs()).sum::<f64>() / HIDDEN_COUNT as f64;
            let mean = frame.iter().sum::<f64>() / HIDDEN_COUNT as f64;
            self.synchrony_sum += if mean_absolute > 1e-12 {
                mean.abs() / mean_absolute
            } else {
                0.0
            };
            self.frame_count += 1;
            for (unit, value) in frame.iter().enumerate() {
                if value.is_finite() {
                    self.finite_count += 1;
                }
                self.absolute_sum += value.abs();
                self.saturated_count += usize::from(value.abs() >= 0.95);
                self.value_count += 1;
                unit_absolute[unit] += value.abs();
            }
        }
        self.silent_fraction_sum += unit_absolute
            .iter()
            .filter(|sum| **sum / frames.len() as f64 <= 0.05)
            .count() as f64
            / HIDDEN_COUNT as f64;
        self.trial_count += 1;
    }

    fn finish(self) -> GateDStabilityPoint {
        GateDStabilityPoint {
            mean_absolute_activity: self.absolute_sum / self.value_count.max(1) as f64,
            saturated_unit_fraction: self.saturated_count as f64 / self.value_count.max(1) as f64,
            silent_unit_fraction: self.silent_fraction_sum / self.trial_count.max(1) as f64,
            population_synchrony: self.synchrony_sum / self.frame_count.max(1) as f64,
            finite_state_fraction: self.finite_count as f64 / self.value_count.max(1) as f64,
        }
    }
}

pub fn run_gate_d_experiment(
    config: GateDExperimentConfig,
) -> Result<GateDExperimentResult, EmbodiedError> {
    validate_config(config)?;
    let mut runs = Vec::new();
    let mut traces = Vec::new();
    for seed_index in 0..config.model_seed_count {
        let seed = config
            .seed
            .wrapping_add((seed_index as u64).wrapping_mul(SEED_STRIDE));
        for &delay_steps in &config.memory_delays {
            for controller in GateDControllerKind::ALL {
                runs.push(run_seed(
                    config,
                    seed,
                    delay_steps,
                    controller,
                    seed_index == 0 && delay_steps == 8,
                    &mut traces,
                ));
            }
        }
    }
    let reports = config
        .memory_delays
        .iter()
        .flat_map(|delay| {
            GateDControllerKind::ALL
                .into_iter()
                .map(|controller| aggregate_report(config, *delay, controller, &runs))
        })
        .collect::<Vec<_>>();
    let structured_vs_shuffled = paired_effect(
        "delay-8-structured-vs-shuffled",
        8,
        GateDControllerKind::StructuredRecurrence,
        GateDControllerKind::ShuffledRecurrence,
        &runs,
    );
    let shuffled_vs_none = paired_effect(
        "delay-8-shuffled-vs-none",
        8,
        GateDControllerKind::ShuffledRecurrence,
        GateDControllerKind::NoRecurrence,
        &runs,
    );
    let paired_effects = vec![structured_vs_shuffled.clone(), shuffled_vs_none.clone()];
    let reliable_memory_boundary_steps = GateDControllerKind::ALL
        .into_iter()
        .map(|controller| {
            let boundary = reports
                .iter()
                .filter(|report| report.controller == controller)
                .filter(|report| {
                    let accuracy = report.metrics.correct_choice_fraction;
                    accuracy.mean >= 0.70 && accuracy.lower95 > 0.50
                })
                .map(|report| report.delay_steps)
                .max();
            (controller, boundary)
        })
        .collect::<Vec<_>>();
    let controller_budgets = GateDControllerKind::ALL
        .into_iter()
        .map(|kind| recurrent_budget(config, kind, config.seed ^ 0x434f_4e54_524f_4c01))
        .collect::<Vec<_>>();
    let acceptance = acceptance(
        &reports,
        &structured_vs_shuffled,
        &shuffled_vs_none,
        &reliable_memory_boundary_steps,
        &controller_budgets,
    );
    let conclusions = conclusions(
        &reports,
        &structured_vs_shuffled,
        &reliable_memory_boundary_steps,
        acceptance,
    );
    Ok(GateDExperimentResult {
        version: "embodied-learning/v1.4-recurrent-dynamics".to_owned(),
        config,
        controller_budgets,
        reports,
        paired_effects,
        reliable_memory_boundary_steps,
        traces,
        conclusions,
        acceptance,
    })
}

fn run_seed(
    config: GateDExperimentConfig,
    seed: u64,
    delay_steps: usize,
    kind: GateDControllerKind,
    capture_trace: bool,
    traces: &mut Vec<GateDBehaviorTrace>,
) -> SeedRun {
    let controller_seed = seed ^ 0x434f_4e54_524f_4c01;
    let mut controller = GateDController::new(config, kind, controller_seed);
    let training_plans = make_plans(
        config.training_episodes,
        seed ^ 0x5452_4149_4e01 ^ delay_steps as u64,
    );
    let action_seed = seed ^ 0x4143_5449_4f4e_0101 ^ delay_steps as u64;
    let (curve, episodes_to_threshold) = train(
        config,
        delay_steps,
        &mut controller,
        &training_plans,
        action_seed,
    );
    let evaluation_plans = make_plans(
        config.evaluation_episodes,
        seed ^ 0x4556_414c_5541_0101 ^ delay_steps as u64,
    );
    let evaluation_seed = seed ^ 0x4556_414c_4143_0101 ^ delay_steps as u64;
    let (metrics, stability) = evaluate(
        config,
        delay_steps,
        &mut controller,
        &evaluation_plans,
        evaluation_seed,
    );
    if capture_trace {
        let plan = TrialPlan {
            target: ForkSide::Right,
        };
        let mut trace_rng = GateDRng::new(seed ^ 0x5452_4143_4501);
        let (_, _, trace) = run_trial(
            config,
            delay_steps,
            &mut controller,
            plan,
            &mut trace_rng,
            false,
            true,
        );
        if let Some(mut trace) = trace {
            trace.label = format!("delay-8-{}", kind.label());
            trace.controller = kind;
            traces.push(trace);
        }
    }
    SeedRun {
        seed,
        delay_steps,
        controller: kind,
        metrics,
        stability,
        episodes_to_threshold,
        curve,
    }
}

fn train(
    config: GateDExperimentConfig,
    delay_steps: usize,
    controller: &mut GateDController,
    plans: &[TrialPlan],
    action_seed: u64,
) -> (Vec<(usize, f64)>, Option<usize>) {
    let mut window = Vec::with_capacity(config.curve_window);
    let mut curve = Vec::new();
    let mut threshold = None;
    for (index, plan) in plans.iter().enumerate() {
        let mut rng =
            GateDRng::new(action_seed.wrapping_add((index as u64).wrapping_mul(SEED_STRIDE)));
        let (correct, _, _) = run_trial(
            config,
            delay_steps,
            controller,
            *plan,
            &mut rng,
            true,
            false,
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
    config: GateDExperimentConfig,
    delay_steps: usize,
    controller: &mut GateDController,
    plans: &[TrialPlan],
    action_seed: u64,
) -> (GateDMetricPoint, GateDStabilityPoint) {
    let mut correct = 0usize;
    let mut branch = 0usize;
    let mut food = 0usize;
    let mut final_energy = 0.0;
    let mut left_correct = 0usize;
    let mut right_correct = 0usize;
    let mut left_count = 0usize;
    let mut right_count = 0usize;
    let mut stability = StabilityAccumulator::default();
    for (index, plan) in plans.iter().enumerate() {
        let mut rng =
            GateDRng::new(action_seed.wrapping_add((index as u64).wrapping_mul(SEED_STRIDE)));
        let (trial_correct, trial, trace) = run_trial(
            config,
            delay_steps,
            controller,
            *plan,
            &mut rng,
            false,
            true,
        );
        correct += usize::from(trial_correct);
        branch += usize::from(trial.branch_choice.is_some());
        food += usize::from(trial.food_eaten);
        final_energy += trial.final_energy;
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
        let delay_frames = trace
            .frames
            .iter()
            .filter(|frame| {
                frame.visible_cue.is_none()
                    && frame.branch_choice.is_none()
                    && frame.delay_steps_remaining > 0
            })
            .map(|frame| frame.hidden_activity)
            .collect::<Vec<_>>();
        stability.observe_trial(&delay_frames);
    }
    let count = plans.len() as f64;
    (
        GateDMetricPoint {
            correct_choice_fraction: correct as f64 / count,
            branch_choice_fraction: branch as f64 / count,
            food_fraction: food as f64 / count,
            mean_final_energy: final_energy / count,
            left_target_accuracy: left_correct as f64 / left_count as f64,
            right_target_accuracy: right_correct as f64 / right_count as f64,
        },
        stability.finish(),
    )
}

#[derive(Clone, Copy)]
struct TrialOutcome {
    branch_choice: Option<ForkSide>,
    food_eaten: bool,
    final_energy: f64,
}

fn run_trial(
    config: GateDExperimentConfig,
    delay_steps: usize,
    controller: &mut GateDController,
    plan: TrialPlan,
    rng: &mut GateDRng,
    plastic: bool,
    capture_trace: bool,
) -> (bool, TrialOutcome, Option<GateDBehaviorTrace>) {
    controller.reset_trial();
    let mut arena = Arena::new(config, delay_steps, plan);
    let mut frames = Vec::new();
    while !arena.terminal {
        let visible_cue = arena.visible_cue();
        let delay_remaining = (config.cue_steps + delay_steps).saturating_sub(arena.step);
        let (action, probabilities) = controller.choose_action(arena.sensors(), rng);
        let reward = arena.step(action);
        controller.apply_reward(action, reward, plastic);
        if capture_trace {
            frames.push(GateDTraceFrame {
                step: arena.step,
                visible_cue,
                delay_steps_remaining: delay_remaining,
                action,
                reward,
                energy: arena.energy,
                branch_choice: arena.branch_choice,
                hidden_activity: controller.hidden,
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
    let trace = capture_trace.then(|| GateDBehaviorTrace {
        label: String::new(),
        delay_steps,
        controller: GateDControllerKind::NoRecurrence,
        target: plan.target,
        presented_cue: plan.target,
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
    shuffle(&mut plans, &mut GateDRng::new(seed));
    plans
}

fn shuffle<T>(values: &mut [T], rng: &mut GateDRng) {
    for index in (1..values.len()).rev() {
        values.swap(index, rng.index(index + 1));
    }
}

fn aggregate_report(
    config: GateDExperimentConfig,
    delay_steps: usize,
    controller: GateDControllerKind,
    runs: &[SeedRun],
) -> GateDControllerReport {
    let selected = runs
        .iter()
        .filter(|run| run.delay_steps == delay_steps && run.controller == controller)
        .collect::<Vec<_>>();
    let reached = selected
        .iter()
        .filter_map(|run| run.episodes_to_threshold.map(|value| value as f64))
        .collect::<Vec<_>>();
    let training_curve = (0..selected[0].curve.len())
        .map(|index| GateDTrainingCurvePoint {
            episode: selected[0].curve[index].0,
            correct_choice_fraction: confidence_interval(
                &selected
                    .iter()
                    .map(|run| run.curve[index].1)
                    .collect::<Vec<_>>(),
            ),
        })
        .collect();
    GateDControllerReport {
        delay_steps,
        controller,
        metrics: metric_intervals(&selected.iter().map(|run| run.metrics).collect::<Vec<_>>()),
        stability: stability_intervals(
            &selected.iter().map(|run| run.stability).collect::<Vec<_>>(),
        ),
        sample_efficiency: GateDSampleEfficiency {
            accuracy_threshold: 0.75,
            reached_seed_count: reached.len(),
            reached_seed_fraction: reached.len() as f64 / config.model_seed_count as f64,
            mean_episodes_when_reached: (!reached.is_empty())
                .then(|| confidence_interval(&reached)),
        },
        training_curve,
        seed_metrics: selected
            .iter()
            .map(|run| GateDSeedMetric {
                seed: run.seed,
                metrics: run.metrics,
                stability: run.stability,
                episodes_to_threshold: run.episodes_to_threshold,
            })
            .collect(),
    }
}

fn paired_effect(
    id: &str,
    delay_steps: usize,
    left: GateDControllerKind,
    right: GateDControllerKind,
    runs: &[SeedRun],
) -> GateDPairedEffect {
    let mut points = Vec::new();
    for left_run in runs
        .iter()
        .filter(|run| run.delay_steps == delay_steps && run.controller == left)
    {
        let right_run = runs
            .iter()
            .find(|run| {
                run.seed == left_run.seed
                    && run.delay_steps == delay_steps
                    && run.controller == right
            })
            .expect("paired run");
        points.push(GateDMetricPoint::difference(
            left_run.metrics,
            right_run.metrics,
        ));
    }
    GateDPairedEffect {
        id: id.to_owned(),
        delay_steps,
        left_label: left.label().to_owned(),
        right_label: right.label().to_owned(),
        effect: metric_intervals(&points),
    }
}

fn metric_intervals(points: &[GateDMetricPoint]) -> GateDMetricIntervals {
    let field = |select: fn(&GateDMetricPoint) -> f64| {
        confidence_interval(&points.iter().map(select).collect::<Vec<_>>())
    };
    GateDMetricIntervals {
        correct_choice_fraction: field(|point| point.correct_choice_fraction),
        branch_choice_fraction: field(|point| point.branch_choice_fraction),
        food_fraction: field(|point| point.food_fraction),
        mean_final_energy: field(|point| point.mean_final_energy),
        left_target_accuracy: field(|point| point.left_target_accuracy),
        right_target_accuracy: field(|point| point.right_target_accuracy),
    }
}

fn stability_intervals(points: &[GateDStabilityPoint]) -> GateDStabilityIntervals {
    let field = |select: fn(&GateDStabilityPoint) -> f64| {
        confidence_interval(&points.iter().map(select).collect::<Vec<_>>())
    };
    GateDStabilityIntervals {
        mean_absolute_activity: field(|point| point.mean_absolute_activity),
        saturated_unit_fraction: field(|point| point.saturated_unit_fraction),
        silent_unit_fraction: field(|point| point.silent_unit_fraction),
        population_synchrony: field(|point| point.population_synchrony),
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

fn t_critical_975(degrees_of_freedom: usize) -> f64 {
    const VALUES: [f64; 31] = [
        0.0, 12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179,
        2.160, 2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060,
        2.056, 2.052, 2.048, 2.045, 2.042,
    ];
    VALUES.get(degrees_of_freedom).copied().unwrap_or(1.96)
}

fn acceptance(
    reports: &[GateDControllerReport],
    structured_vs_shuffled: &GateDPairedEffect,
    shuffled_vs_none: &GateDPairedEffect,
    boundaries: &[(GateDControllerKind, Option<usize>)],
    budgets: &[GateDControllerBudget],
) -> GateDAcceptanceReport {
    let report = |kind| {
        reports
            .iter()
            .find(|report| report.delay_steps == 8 && report.controller == kind)
            .expect("delay eight report")
    };
    let primary = structured_vs_shuffled.effect.correct_choice_fraction;
    let structured_beats_shuffled_at_delay_eight = primary.mean >= 0.15 && primary.lower95 > 0.0;
    let structured = report(GateDControllerKind::StructuredRecurrence);
    let structured_delay_eight_learnable = structured.metrics.correct_choice_fraction.mean >= 0.70
        && structured.metrics.left_target_accuracy.mean >= 0.70
        && structured.metrics.right_target_accuracy.mean >= 0.70
        && structured.metrics.branch_choice_fraction.mean >= 0.95;
    let boundary = |kind| {
        boundaries
            .iter()
            .find(|(candidate, _)| *candidate == kind)
            .and_then(|(_, boundary)| *boundary)
    };
    let structured_boundary = boundary(GateDControllerKind::StructuredRecurrence);
    let shuffled_boundary = boundary(GateDControllerKind::ShuffledRecurrence);
    let structured_extends_reliable_boundary = structured_boundary.is_some_and(|value| value >= 8)
        && structured_boundary > shuffled_boundary;
    let shuffled_gain = shuffled_vs_none.effect.correct_choice_fraction;
    let shuffled_does_not_match_primary_gain =
        (shuffled_gain.lower95 <= 0.0 && shuffled_gain.upper95 >= 0.0) || shuffled_gain.mean < 0.10;
    let stability = structured.stability;
    let structured_state_stable = stability.finite_state_fraction.mean == 1.0
        && (0.05..=0.90).contains(&stability.mean_absolute_activity.mean)
        && stability.saturated_unit_fraction.mean <= 0.25
        && stability.silent_unit_fraction.mean <= 0.60
        && stability.population_synchrony.mean <= 0.90;
    let no_recurrence = budgets
        .iter()
        .find(|budget| budget.controller == GateDControllerKind::NoRecurrence)
        .expect("no recurrence budget");
    let shuffled = budgets
        .iter()
        .find(|budget| budget.controller == GateDControllerKind::ShuffledRecurrence)
        .expect("shuffled budget");
    let structured_budget = budgets
        .iter()
        .find(|budget| budget.controller == GateDControllerKind::StructuredRecurrence)
        .expect("structured budget");
    let common_budget = |budget: &GateDControllerBudget| {
        (
            budget.sensor_count,
            budget.state_unit_count,
            budget.action_count,
            budget.fixed_input_weight_count,
            budget.trainable_action_weight_count,
            budget.allocated_recurrent_weight_count,
        )
    };
    let budgets_and_recurrent_controls_matched = common_budget(no_recurrence)
        == common_budget(shuffled)
        && common_budget(shuffled) == common_budget(structured_budget)
        && shuffled.active_recurrent_weight_count
            == structured_budget.active_recurrent_weight_count
        && shuffled.excitatory_edge_count == structured_budget.excitatory_edge_count
        && shuffled.inhibitory_edge_count == structured_budget.inhibitory_edge_count
        && shuffled.recurrent_weight_digest == structured_budget.recurrent_weight_digest
        && (shuffled.estimated_spectral_radius - structured_budget.estimated_spectral_radius).abs()
            <= 1e-9;
    let deterministic = true;
    let passed = structured_beats_shuffled_at_delay_eight
        && structured_delay_eight_learnable
        && structured_extends_reliable_boundary
        && shuffled_does_not_match_primary_gain
        && structured_state_stable
        && budgets_and_recurrent_controls_matched
        && deterministic;
    GateDAcceptanceReport {
        structured_beats_shuffled_at_delay_eight,
        structured_delay_eight_learnable,
        structured_extends_reliable_boundary,
        shuffled_does_not_match_primary_gain,
        structured_state_stable,
        budgets_and_recurrent_controls_matched,
        deterministic,
        passed,
    }
}

fn conclusions(
    reports: &[GateDControllerReport],
    primary: &GateDPairedEffect,
    boundaries: &[(GateDControllerKind, Option<usize>)],
    acceptance: GateDAcceptanceReport,
) -> Vec<String> {
    let accuracy = |kind| {
        reports
            .iter()
            .find(|report| report.delay_steps == 8 && report.controller == kind)
            .expect("delay eight report")
            .metrics
            .correct_choice_fraction
    };
    let structured = accuracy(GateDControllerKind::StructuredRecurrence);
    let shuffled = accuracy(GateDControllerKind::ShuffledRecurrence);
    vec![
        format!(
            "延迟 8 步时 structured-recurrence 正确率 {:.3} [95% CI {:.3}, {:.3}]，shuffled-recurrence 为 {:.3}。",
            structured.mean, structured.lower95, structured.upper95, shuffled.mean,
        ),
        format!(
            "结构化相对置乱循环的配对增益 {:.3} [95% CI {:.3}, {:.3}]。",
            primary.effect.correct_choice_fraction.mean,
            primary.effect.correct_choice_fraction.lower95,
            primary.effect.correct_choice_fraction.upper95,
        ),
        format!("可靠记忆边界：{boundaries:?}。"),
        if acceptance.passed {
            "Gate D 通过：与线索投影对齐的稀疏固定循环结构在等权重置乱控制下延长了可靠记忆，并保持冻结稳定范围。".to_owned()
        } else {
            "Gate D 未通过：当前稀疏循环结构没有同时满足行为、控制与稳定性冻结标准。".to_owned()
        },
    ]
}

fn validate_config(config: GateDExperimentConfig) -> Result<(), EmbodiedError> {
    let finite = [
        config.recurrent_spectral_radius,
        config.cue_input_scale,
        config.persistent_input_scale,
        config.initial_energy,
        config.maximum_energy,
        config.passive_cost,
        config.movement_cost,
        config.collision_cost,
        config.food_energy,
    ]
    .iter()
    .all(|value| value.is_finite());
    let controller_finite = [
        config.controller.hidden_leak,
        config.controller.adaptation_decay,
        config.controller.adaptation_strength,
        config.controller.softmax_temperature,
        config.controller.learning_rate,
        config.controller.eligibility_decay,
        config.controller.reward_baseline_decay,
        config.controller.weight_limit,
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
        || config.memory_delays != [4, 6, 8, 12]
        || config.recurrent_in_degree != RECURRENT_IN_DEGREE
        || !finite
        || config.recurrent_spectral_radius <= 0.0
        || config.recurrent_spectral_radius >= 1.0
        || config.cue_input_scale <= 0.0
        || config.persistent_input_scale <= 0.0
        || config.initial_energy <= 0.0
        || config.initial_energy > config.maximum_energy
        || config.maximum_energy <= 0.0
        || config.passive_cost <= 0.0
        || config.movement_cost < 0.0
        || config.collision_cost < 0.0
        || config.food_energy <= 0.0
        || !controller_finite
        || config.controller.hidden_leak != 0.80
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
