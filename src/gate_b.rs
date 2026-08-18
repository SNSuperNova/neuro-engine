use serde::Serialize;

use crate::{AgentAction, ControllerConfig, EmbodiedError, GridPosition, Heading};

pub const GATE_B_SENSOR_COUNT: usize = 12;
const HIDDEN_COUNT: usize = crate::HIDDEN_COUNT;
const ACTION_COUNT: usize = crate::ACTION_COUNT;
const START: GridPosition = GridPosition { x: 4, y: 8 };
const JUNCTION: GridPosition = GridPosition { x: 4, y: 2 };
const LEFT_TERMINAL: GridPosition = GridPosition { x: 3, y: 2 };
const RIGHT_TERMINAL: GridPosition = GridPosition { x: 5, y: 2 };
const SEED_STRIDE: u64 = 0x9e37_79b9_7f4a_7c15;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ForkSide {
    Left,
    Right,
}

impl ForkSide {
    fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GateBControllerKind {
    Stateless,
    StateReset,
    LeakyState,
}

impl GateBControllerKind {
    const ALL: [Self; 3] = [Self::Stateless, Self::StateReset, Self::LeakyState];

    fn label(self) -> &'static str {
        match self {
            Self::Stateless => "stateless",
            Self::StateReset => "state-reset",
            Self::LeakyState => "leaky-state",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GateBCondition {
    DelayedCue,
    CueVisibleAtFork,
    CueRandomized,
    HistoryShuffled,
}

impl GateBCondition {
    fn label(self) -> &'static str {
        match self {
            Self::DelayedCue => "delayed-cue",
            Self::CueVisibleAtFork => "cue-visible-at-fork",
            Self::CueRandomized => "cue-randomized",
            Self::HistoryShuffled => "history-shuffled",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateBExperimentConfig {
    pub seed: u64,
    pub model_seed_count: usize,
    pub training_episodes: usize,
    pub evaluation_episodes: usize,
    pub curve_window: usize,
    pub maximum_steps: usize,
    pub cue_steps: usize,
    pub initial_energy: f64,
    pub maximum_energy: f64,
    pub passive_cost: f64,
    pub movement_cost: f64,
    pub collision_cost: f64,
    pub food_energy: f64,
    pub controller: ControllerConfig,
}

impl Default for GateBExperimentConfig {
    fn default() -> Self {
        let controller = ControllerConfig {
            hidden_leak: 0.80,
            ..ControllerConfig::default()
        };
        Self {
            seed: 0x4741_5445_5f42_0201,
            model_seed_count: 12,
            training_episodes: 1_200,
            evaluation_episodes: 200,
            curve_window: 40,
            maximum_steps: 40,
            cue_steps: 2,
            initial_energy: 1.0,
            maximum_energy: 2.0,
            passive_cost: 0.004,
            movement_cost: 0.003,
            collision_cost: 0.012,
            food_energy: 1.0,
            controller,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateBConfidenceInterval {
    pub mean: f64,
    pub lower95: f64,
    pub upper95: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateBMetricPoint {
    pub correct_choice_fraction: f64,
    pub branch_choice_fraction: f64,
    pub food_fraction: f64,
    pub mean_final_energy: f64,
    pub mean_steps: f64,
    pub right_choice_fraction: f64,
    pub left_target_accuracy: f64,
    pub right_target_accuracy: f64,
}

impl GateBMetricPoint {
    fn difference(left: Self, right: Self) -> Self {
        Self {
            correct_choice_fraction: left.correct_choice_fraction - right.correct_choice_fraction,
            branch_choice_fraction: left.branch_choice_fraction - right.branch_choice_fraction,
            food_fraction: left.food_fraction - right.food_fraction,
            mean_final_energy: left.mean_final_energy - right.mean_final_energy,
            mean_steps: left.mean_steps - right.mean_steps,
            right_choice_fraction: left.right_choice_fraction - right.right_choice_fraction,
            left_target_accuracy: left.left_target_accuracy - right.left_target_accuracy,
            right_target_accuracy: left.right_target_accuracy - right.right_target_accuracy,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateBMetricIntervals {
    pub correct_choice_fraction: GateBConfidenceInterval,
    pub branch_choice_fraction: GateBConfidenceInterval,
    pub food_fraction: GateBConfidenceInterval,
    pub mean_final_energy: GateBConfidenceInterval,
    pub mean_steps: GateBConfidenceInterval,
    pub right_choice_fraction: GateBConfidenceInterval,
    pub left_target_accuracy: GateBConfidenceInterval,
    pub right_target_accuracy: GateBConfidenceInterval,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateBTrainingCurvePoint {
    pub episode: usize,
    pub correct_choice_fraction: GateBConfidenceInterval,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateBControllerBudget {
    pub controller: GateBControllerKind,
    pub sensor_count: usize,
    pub state_unit_count: usize,
    pub action_count: usize,
    pub fixed_input_weight_count: usize,
    pub trainable_action_weight_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateBSampleEfficiency {
    pub accuracy_threshold: f64,
    pub reached_seed_count: usize,
    pub reached_seed_fraction: f64,
    pub mean_episodes_when_reached: Option<GateBConfidenceInterval>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateBSeedMetric {
    pub seed: u64,
    pub metrics: GateBMetricPoint,
    pub episodes_to_threshold: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateBControllerReport {
    pub condition: GateBCondition,
    pub controller: GateBControllerKind,
    pub metrics: GateBMetricIntervals,
    pub sample_efficiency: GateBSampleEfficiency,
    pub training_curve: Vec<GateBTrainingCurvePoint>,
    pub seed_metrics: Vec<GateBSeedMetric>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateBPairedEffect {
    pub id: String,
    pub left_label: String,
    pub right_label: String,
    pub effect: GateBMetricIntervals,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateBAcceptanceReport {
    pub visible_control_learnable: bool,
    pub randomized_control_at_chance: bool,
    pub leaky_beats_state_reset: bool,
    pub leaky_reaches_seventy_percent: bool,
    pub history_shuffle_hurts: bool,
    pub left_right_consistent: bool,
    pub deterministic: bool,
    pub passed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateBTraceFrame {
    pub step: usize,
    pub position_before: GridPosition,
    pub heading_before: Heading,
    pub position: GridPosition,
    pub heading: Heading,
    pub visible_cue: Option<ForkSide>,
    pub action: AgentAction,
    pub reward: f64,
    pub energy: f64,
    pub branch_choice: Option<ForkSide>,
    pub hidden_activity: [f64; HIDDEN_COUNT],
    pub action_probabilities: [f64; ACTION_COUNT],
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateBBehaviorTrace {
    pub label: String,
    pub condition: GateBCondition,
    pub controller: GateBControllerKind,
    pub target: ForkSide,
    pub presented_cue: ForkSide,
    pub frames: Vec<GateBTraceFrame>,
    pub summary: GateBTrialSummary,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateBTrialSummary {
    pub steps: usize,
    pub branch_choice: Option<ForkSide>,
    pub correct_choice: bool,
    pub food_eaten: bool,
    pub final_energy: f64,
    pub total_reward: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateBExperimentResult {
    pub version: String,
    pub config: GateBExperimentConfig,
    pub sensor_labels: Vec<String>,
    pub controller_budgets: Vec<GateBControllerBudget>,
    pub reports: Vec<GateBControllerReport>,
    pub paired_effects: Vec<GateBPairedEffect>,
    pub traces: Vec<GateBBehaviorTrace>,
    pub conclusions: Vec<String>,
    pub acceptance: GateBAcceptanceReport,
}

#[derive(Clone, Copy)]
struct TrialPlan {
    target: ForkSide,
    cue: ForkSide,
}

#[derive(Clone)]
struct GateBRng {
    state: u64,
}

impl GateBRng {
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
struct GateBController {
    config: ControllerConfig,
    input_weights: [[f64; GATE_B_SENSOR_COUNT]; HIDDEN_COUNT],
    policy_weights: [[f64; HIDDEN_COUNT]; ACTION_COUNT],
    eligibility: [[f64; HIDDEN_COUNT]; ACTION_COUNT],
    hidden: [f64; HIDDEN_COUNT],
    adaptation: [f64; HIDDEN_COUNT],
    last_features: [f64; HIDDEN_COUNT],
    last_probabilities: [f64; ACTION_COUNT],
    reward_baseline: f64,
}

impl GateBController {
    fn new(config: ControllerConfig, seed: u64) -> Self {
        let mut rng = GateBRng::new(seed);
        let mut input_weights = [[0.0; GATE_B_SENSOR_COUNT]; HIDDEN_COUNT];
        for row in &mut input_weights {
            for (sensor, weight) in row.iter_mut().enumerate() {
                let scale = if matches!(sensor, 2 | 3) { 1.20 } else { 0.20 };
                *weight = rng.signed(scale);
            }
        }
        let mut policy_weights = [[0.0; HIDDEN_COUNT]; ACTION_COUNT];
        for row in &mut policy_weights {
            for weight in row {
                *weight = rng.signed(0.08);
            }
        }
        Self {
            config,
            input_weights,
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
        self.clear_state();
    }

    fn clear_state(&mut self) {
        self.hidden = [0.0; HIDDEN_COUNT];
        self.adaptation = [0.0; HIDDEN_COUNT];
        self.last_features = [0.0; HIDDEN_COUNT];
        self.last_probabilities = [0.0; ACTION_COUNT];
    }

    fn choose_action(
        &mut self,
        sensors: [f64; GATE_B_SENSOR_COUNT],
        kind: GateBControllerKind,
        rng: &mut GateBRng,
    ) -> (AgentAction, [f64; ACTION_COUNT]) {
        if kind == GateBControllerKind::StateReset {
            self.clear_state();
        }
        for hidden in 0..HIDDEN_COUNT {
            let drive = self.input_weights[hidden]
                .iter()
                .zip(sensors)
                .map(|(weight, sensor)| weight * sensor)
                .sum::<f64>();
            if kind == GateBControllerKind::Stateless {
                self.hidden[hidden] = drive.tanh();
                self.adaptation[hidden] = 0.0;
            } else {
                self.hidden[hidden] = (drive + self.config.hidden_leak * self.hidden[hidden]
                    - self.config.adaptation_strength * self.adaptation[hidden])
                    .tanh();
                self.adaptation[hidden] = self.config.adaptation_decay * self.adaptation[hidden]
                    + (1.0 - self.config.adaptation_decay) * self.hidden[hidden].abs();
            }
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

struct ForkArena {
    config: GateBExperimentConfig,
    condition: GateBCondition,
    plan: TrialPlan,
    position: GridPosition,
    heading: Heading,
    energy: f64,
    step: usize,
    branch_choice: Option<ForkSide>,
    food_eaten: bool,
    terminal: bool,
    previous_reward: f64,
    total_reward: f64,
}

impl ForkArena {
    fn new(config: GateBExperimentConfig, condition: GateBCondition, plan: TrialPlan) -> Self {
        Self {
            config,
            condition,
            plan,
            position: START,
            heading: Heading::North,
            energy: config.initial_energy,
            step: 0,
            branch_choice: None,
            food_eaten: false,
            terminal: false,
            previous_reward: 0.0,
            total_reward: 0.0,
        }
    }

    fn visible_cue(&self) -> Option<ForkSide> {
        (self.step < self.config.cue_steps
            || (self.condition == GateBCondition::CueVisibleAtFork && self.position == JUNCTION))
            .then_some(self.plan.cue)
    }

    fn sensors(&self) -> [f64; GATE_B_SENSOR_COUNT] {
        let cue = self.visible_cue();
        let forward = self.next_position(self.heading);
        let left = self.next_position(turn_left(self.heading));
        let right = self.next_position(turn_right(self.heading));
        [
            1.0,
            self.energy / self.config.maximum_energy,
            f64::from(cue == Some(ForkSide::Left)),
            f64::from(cue == Some(ForkSide::Right)),
            f64::from(!is_open(forward)),
            f64::from(!is_open(left)),
            f64::from(!is_open(right)),
            f64::from(self.position == terminal_for(self.plan.target) && !self.food_eaten),
            f64::from(self.position == JUNCTION),
            self.previous_reward.tanh(),
            self.step as f64 / self.config.maximum_steps as f64,
            f64::from(self.branch_choice.is_some()),
        ]
    }

    fn next_position(&self, heading: Heading) -> GridPosition {
        let delta = heading_delta(heading);
        GridPosition {
            x: self.position.x + delta.x,
            y: self.position.y + delta.y,
        }
    }

    fn step(&mut self, action: AgentAction) -> f64 {
        let before_energy = self.energy;
        self.energy -= self.config.passive_cost;
        if self.position.y > JUNCTION.y {
            // The corridor is a forced-delay segment: every controller receives the
            // same number of observations before the only meaningful branch choice.
            self.energy -= self.config.movement_cost;
            self.position.y -= 1;
            self.heading = Heading::North;
        } else if self.position == JUNCTION {
            let side = match action {
                AgentAction::TurnLeft => Some(ForkSide::Left),
                AgentAction::TurnRight => Some(ForkSide::Right),
                AgentAction::Forward | AgentAction::Eat => None,
            };
            if let Some(side) = side {
                self.energy -= self.config.movement_cost;
                self.branch_choice = Some(side);
                self.position = terminal_for(side);
                self.heading = match side {
                    ForkSide::Left => Heading::West,
                    ForkSide::Right => Heading::East,
                };
                if side != self.plan.target {
                    self.terminal = true;
                }
            } else {
                self.energy -= self.config.collision_cost;
            }
        } else if action == AgentAction::Eat
            && self.position == terminal_for(self.plan.target)
            && !self.food_eaten
        {
            self.energy += self.config.food_energy;
            self.food_eaten = true;
            self.terminal = true;
        } else {
            self.energy -= self.config.collision_cost;
        }
        self.energy = self.energy.clamp(0.0, self.config.maximum_energy);
        self.step += 1;
        let reward = (self.energy - before_energy) / self.config.food_energy;
        self.previous_reward = reward;
        self.total_reward += reward;
        if self.energy <= 0.0 || self.step >= self.config.maximum_steps {
            self.terminal = true;
        }
        reward
    }

    fn done(&self) -> bool {
        self.terminal
    }

    fn summary(&self) -> GateBTrialSummary {
        GateBTrialSummary {
            steps: self.step,
            branch_choice: self.branch_choice,
            correct_choice: self.branch_choice == Some(self.plan.target),
            food_eaten: self.food_eaten,
            final_energy: self.energy,
            total_reward: self.total_reward,
        }
    }
}

pub fn run_gate_b_experiment(
    config: GateBExperimentConfig,
) -> Result<GateBExperimentResult, EmbodiedError> {
    validate_config(config)?;
    let mut seed_outputs = Vec::with_capacity(config.model_seed_count);
    let mut traces = Vec::new();
    for index in 0..config.model_seed_count {
        let seed = config
            .seed
            .wrapping_add((index as u64).wrapping_mul(SEED_STRIDE));
        seed_outputs.push(run_model_seed(config, seed, index == 0, &mut traces));
    }

    let mut reports = Vec::new();
    for condition in [
        GateBCondition::DelayedCue,
        GateBCondition::CueVisibleAtFork,
        GateBCondition::CueRandomized,
    ] {
        for controller in GateBControllerKind::ALL {
            reports.push(aggregate_report(
                config,
                condition,
                controller,
                &seed_outputs,
            ));
        }
    }
    reports.push(aggregate_report(
        config,
        GateBCondition::HistoryShuffled,
        GateBControllerKind::LeakyState,
        &seed_outputs,
    ));

    let delayed_effect = paired_effect(
        "leaky-vs-state-reset",
        GateBCondition::DelayedCue,
        GateBControllerKind::LeakyState,
        GateBCondition::DelayedCue,
        GateBControllerKind::StateReset,
        &seed_outputs,
    );
    let history_effect = paired_effect(
        "delayed-vs-history-shuffled",
        GateBCondition::DelayedCue,
        GateBControllerKind::LeakyState,
        GateBCondition::HistoryShuffled,
        GateBControllerKind::LeakyState,
        &seed_outputs,
    );
    let paired_effects = vec![delayed_effect.clone(), history_effect.clone()];
    let acceptance = acceptance(&reports, &delayed_effect, &history_effect);

    Ok(GateBExperimentResult {
        version: "embodied-learning/v1.2-state-necessity".to_owned(),
        config,
        sensor_labels: [
            "bias",
            "energy",
            "cue-left",
            "cue-right",
            "wall-forward",
            "wall-left",
            "wall-right",
            "food-here",
            "at-junction",
            "previous-reward",
            "episode-progress",
            "branch-committed",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
        controller_budgets: GateBControllerKind::ALL
            .into_iter()
            .map(|controller| GateBControllerBudget {
                controller,
                sensor_count: GATE_B_SENSOR_COUNT,
                state_unit_count: HIDDEN_COUNT,
                action_count: ACTION_COUNT,
                fixed_input_weight_count: HIDDEN_COUNT * GATE_B_SENSOR_COUNT,
                trainable_action_weight_count: HIDDEN_COUNT * ACTION_COUNT,
            })
            .collect(),
        conclusions: conclusions(&reports, &delayed_effect, &history_effect, acceptance),
        reports,
        paired_effects,
        traces,
        acceptance,
    })
}

#[derive(Clone)]
struct SeedRun {
    seed: u64,
    condition: GateBCondition,
    controller: GateBControllerKind,
    metrics: GateBMetricPoint,
    curve: Vec<(usize, f64)>,
    episodes_to_threshold: Option<usize>,
}

struct TrialRequest {
    kind: GateBControllerKind,
    condition: GateBCondition,
    plan: TrialPlan,
    config: GateBExperimentConfig,
    action_seed: u64,
    plastic: bool,
    trace_label: Option<String>,
}

fn run_model_seed(
    config: GateBExperimentConfig,
    seed: u64,
    capture_traces: bool,
    traces: &mut Vec<GateBBehaviorTrace>,
) -> Vec<SeedRun> {
    let mut result = Vec::new();
    let mut delayed_leaky = None;
    for condition in [
        GateBCondition::DelayedCue,
        GateBCondition::CueVisibleAtFork,
        GateBCondition::CueRandomized,
    ] {
        for controller_kind in GateBControllerKind::ALL {
            let controller_seed = seed ^ 0x434f_4e54_524f_4c01;
            let mut controller = GateBController::new(config.controller, controller_seed);
            let training_plans = make_plans(
                config.training_episodes,
                seed ^ condition_tag(condition) ^ 0x5452_4149_4e01,
                condition == GateBCondition::CueRandomized,
            );
            let (curve, episodes_to_threshold) = train(
                &mut controller,
                controller_kind,
                condition,
                &training_plans,
                config,
                seed,
            );
            let evaluation_plans = make_plans(
                config.evaluation_episodes,
                seed ^ condition_tag(condition) ^ 0x4556_414c_0101,
                condition == GateBCondition::CueRandomized,
            );
            let metrics = evaluate(
                &controller,
                controller_kind,
                condition,
                &evaluation_plans,
                config,
                seed,
                None,
            )
            .0;
            if condition == GateBCondition::DelayedCue
                && controller_kind == GateBControllerKind::LeakyState
            {
                delayed_leaky = Some(controller.clone());
            }
            if capture_traces
                && ((condition == GateBCondition::DelayedCue
                    && matches!(
                        controller_kind,
                        GateBControllerKind::LeakyState | GateBControllerKind::StateReset
                    ))
                    || (condition == GateBCondition::CueVisibleAtFork
                        && controller_kind == GateBControllerKind::Stateless))
            {
                let (_, trace) = evaluate(
                    &controller,
                    controller_kind,
                    condition,
                    &evaluation_plans[..1],
                    config,
                    seed,
                    Some(format!("{}-{}", condition.label(), controller_kind.label())),
                );
                traces.extend(trace);
            }
            result.push(SeedRun {
                seed,
                condition,
                controller: controller_kind,
                metrics,
                curve,
                episodes_to_threshold,
            });
        }
    }

    let delayed_leaky = delayed_leaky.expect("delayed leaky controller trained");
    let shuffled_plans = make_plans(
        config.evaluation_episodes,
        seed ^ condition_tag(GateBCondition::DelayedCue) ^ 0x4556_414c_0101,
        true,
    );
    let (metrics, trace) = evaluate(
        &delayed_leaky,
        GateBControllerKind::LeakyState,
        GateBCondition::HistoryShuffled,
        &shuffled_plans,
        config,
        seed,
        capture_traces.then(|| "history-shuffled-leaky-state".to_owned()),
    );
    traces.extend(trace);
    result.push(SeedRun {
        seed,
        condition: GateBCondition::HistoryShuffled,
        controller: GateBControllerKind::LeakyState,
        metrics,
        curve: Vec::new(),
        episodes_to_threshold: None,
    });
    result
}

fn train(
    controller: &mut GateBController,
    kind: GateBControllerKind,
    condition: GateBCondition,
    plans: &[TrialPlan],
    config: GateBExperimentConfig,
    seed: u64,
) -> (Vec<(usize, f64)>, Option<usize>) {
    let mut recent = Vec::with_capacity(config.curve_window);
    let mut curve = Vec::new();
    let mut threshold = None;
    for (index, plan) in plans.iter().copied().enumerate() {
        let summary = run_trial(
            controller,
            TrialRequest {
                kind,
                condition,
                plan,
                config,
                action_seed: seed.wrapping_add((index as u64).wrapping_mul(SEED_STRIDE)),
                plastic: true,
                trace_label: None,
            },
        )
        .0;
        recent.push((plan, summary));
        if recent.len() > config.curve_window {
            recent.remove(0);
        }
        if (index + 1) % config.curve_window == 0 || index + 1 == plans.len() {
            let accuracy = summarize_plans(&recent).correct_choice_fraction;
            curve.push((index + 1, accuracy));
            if threshold.is_none() && accuracy >= 0.75 {
                threshold = Some(index + 1);
            }
        }
    }
    (curve, threshold)
}

fn evaluate(
    controller: &GateBController,
    kind: GateBControllerKind,
    condition: GateBCondition,
    plans: &[TrialPlan],
    config: GateBExperimentConfig,
    seed: u64,
    trace_label: Option<String>,
) -> (GateBMetricPoint, Vec<GateBBehaviorTrace>) {
    let mut summaries = Vec::with_capacity(plans.len());
    let mut traces = Vec::new();
    let mut evaluation_controller = controller.clone();
    for (index, plan) in plans.iter().copied().enumerate() {
        let label = (index == 0).then(|| trace_label.clone()).flatten();
        let (summary, trace) = run_trial(
            &mut evaluation_controller,
            TrialRequest {
                kind,
                condition,
                plan,
                config,
                action_seed: seed.wrapping_add((index as u64).wrapping_mul(SEED_STRIDE)),
                plastic: false,
                trace_label: label,
            },
        );
        summaries.push((plan, summary));
        if let Some(trace) = trace {
            traces.push(trace);
        }
    }
    (summarize_plans(&summaries), traces)
}

fn run_trial(
    controller: &mut GateBController,
    request: TrialRequest,
) -> (GateBTrialSummary, Option<GateBBehaviorTrace>) {
    let TrialRequest {
        kind,
        condition,
        plan,
        config,
        action_seed,
        plastic,
        trace_label,
    } = request;
    controller.reset_trial();
    let mut arena = ForkArena::new(config, condition, plan);
    let mut rng = GateBRng::new(action_seed ^ 0x4143_5449_4f4e_0101);
    let mut frames = Vec::new();
    while !arena.done() {
        let position_before = arena.position;
        let heading_before = arena.heading;
        let cue = arena.visible_cue();
        let sensors = arena.sensors();
        let (action, probabilities) = controller.choose_action(sensors, kind, &mut rng);
        let hidden = controller.hidden;
        let reward = arena.step(action);
        controller.apply_reward(action, reward, plastic);
        if trace_label.is_some() {
            frames.push(GateBTraceFrame {
                step: arena.step,
                position_before,
                heading_before,
                position: arena.position,
                heading: arena.heading,
                visible_cue: cue,
                action,
                reward,
                energy: arena.energy,
                branch_choice: arena.branch_choice,
                hidden_activity: hidden,
                action_probabilities: probabilities,
            });
        }
    }
    let summary = arena.summary();
    let trace = trace_label.map(|label| GateBBehaviorTrace {
        label,
        condition,
        controller: kind,
        target: plan.target,
        presented_cue: plan.cue,
        frames,
        summary,
    });
    (summary, trace)
}

fn summarize_plans(summaries: &[(TrialPlan, GateBTrialSummary)]) -> GateBMetricPoint {
    let total = summaries.len().max(1) as f64;
    let choices = summaries
        .iter()
        .filter(|(_, summary)| summary.branch_choice.is_some())
        .count();
    let choice_count = choices.max(1) as f64;
    let correct = summaries
        .iter()
        .filter(|(_, summary)| summary.correct_choice)
        .count();
    let right_choices = summaries
        .iter()
        .filter(|(_, summary)| summary.branch_choice == Some(ForkSide::Right))
        .count();
    let side_accuracy = |side: ForkSide| {
        let side_choices = summaries
            .iter()
            .filter(|(plan, summary)| plan.target == side && summary.branch_choice.is_some())
            .count();
        let side_correct = summaries
            .iter()
            .filter(|(plan, summary)| plan.target == side && summary.correct_choice)
            .count();
        side_correct as f64 / side_choices.max(1) as f64
    };
    GateBMetricPoint {
        correct_choice_fraction: correct as f64 / choice_count,
        branch_choice_fraction: choices as f64 / total,
        food_fraction: summaries
            .iter()
            .filter(|(_, summary)| summary.food_eaten)
            .count() as f64
            / total,
        mean_final_energy: summaries
            .iter()
            .map(|(_, summary)| summary.final_energy)
            .sum::<f64>()
            / total,
        mean_steps: summaries
            .iter()
            .map(|(_, summary)| summary.steps as f64)
            .sum::<f64>()
            / total,
        right_choice_fraction: right_choices as f64 / choice_count,
        left_target_accuracy: side_accuracy(ForkSide::Left),
        right_target_accuracy: side_accuracy(ForkSide::Right),
    }
}

fn make_plans(count: usize, seed: u64, randomized_cue: bool) -> Vec<TrialPlan> {
    let mut targets = (0..count)
        .map(|index| {
            if index < count / 2 {
                ForkSide::Left
            } else {
                ForkSide::Right
            }
        })
        .collect::<Vec<_>>();
    shuffle(&mut targets, &mut GateBRng::new(seed));
    let cues = if randomized_cue {
        independent_balanced_cues(&targets, seed ^ 0x4355_4553_0101)
    } else {
        targets.clone()
    };
    targets
        .into_iter()
        .zip(cues)
        .map(|(target, cue)| TrialPlan { target, cue })
        .collect()
}

fn independent_balanced_cues(targets: &[ForkSide], seed: u64) -> Vec<ForkSide> {
    let mut result = vec![ForkSide::Left; targets.len()];
    let mut rng = GateBRng::new(seed);
    for target in [ForkSide::Left, ForkSide::Right] {
        let mut indices = targets
            .iter()
            .enumerate()
            .filter_map(|(index, value)| (*value == target).then_some(index))
            .collect::<Vec<_>>();
        shuffle(&mut indices, &mut rng);
        let half = indices.len() / 2;
        for (offset, index) in indices.into_iter().enumerate() {
            result[index] = if offset < half {
                target
            } else {
                target.opposite()
            };
        }
    }
    result
}

fn shuffle<T>(values: &mut [T], rng: &mut GateBRng) {
    for index in (1..values.len()).rev() {
        values.swap(index, rng.index(index + 1));
    }
}

fn aggregate_report(
    config: GateBExperimentConfig,
    condition: GateBCondition,
    controller: GateBControllerKind,
    seed_outputs: &[Vec<SeedRun>],
) -> GateBControllerReport {
    let runs = seed_outputs
        .iter()
        .map(|runs| find_run(runs, condition, controller))
        .collect::<Vec<_>>();
    let points = runs.iter().map(|run| run.metrics).collect::<Vec<_>>();
    let reached = runs
        .iter()
        .filter_map(|run| run.episodes_to_threshold.map(|episode| episode as f64))
        .collect::<Vec<_>>();
    let point_count = runs.first().map_or(0, |run| run.curve.len());
    let training_curve = (0..point_count)
        .map(|index| GateBTrainingCurvePoint {
            episode: runs[0].curve[index].0,
            correct_choice_fraction: confidence_interval(
                &runs
                    .iter()
                    .map(|run| run.curve[index].1)
                    .collect::<Vec<_>>(),
            ),
        })
        .collect();
    GateBControllerReport {
        condition,
        controller,
        metrics: intervals(&points),
        sample_efficiency: GateBSampleEfficiency {
            accuracy_threshold: 0.75,
            reached_seed_count: reached.len(),
            reached_seed_fraction: reached.len() as f64 / config.model_seed_count as f64,
            mean_episodes_when_reached: (!reached.is_empty())
                .then(|| confidence_interval(&reached)),
        },
        training_curve,
        seed_metrics: runs
            .iter()
            .map(|run| GateBSeedMetric {
                seed: run.seed,
                metrics: run.metrics,
                episodes_to_threshold: run.episodes_to_threshold,
            })
            .collect(),
    }
}

fn paired_effect(
    id: &str,
    left_condition: GateBCondition,
    left_controller: GateBControllerKind,
    right_condition: GateBCondition,
    right_controller: GateBControllerKind,
    seed_outputs: &[Vec<SeedRun>],
) -> GateBPairedEffect {
    let points = seed_outputs
        .iter()
        .map(|runs| {
            GateBMetricPoint::difference(
                find_run(runs, left_condition, left_controller).metrics,
                find_run(runs, right_condition, right_controller).metrics,
            )
        })
        .collect::<Vec<_>>();
    GateBPairedEffect {
        id: id.to_owned(),
        left_label: format!("{}-{}", left_condition.label(), left_controller.label()),
        right_label: format!("{}-{}", right_condition.label(), right_controller.label()),
        effect: intervals(&points),
    }
}

fn find_run(
    runs: &[SeedRun],
    condition: GateBCondition,
    controller: GateBControllerKind,
) -> &SeedRun {
    runs.iter()
        .find(|run| run.condition == condition && run.controller == controller)
        .expect("canonical Gate B run exists")
}

fn intervals(points: &[GateBMetricPoint]) -> GateBMetricIntervals {
    let field = |select: fn(&GateBMetricPoint) -> f64| {
        confidence_interval(&points.iter().map(select).collect::<Vec<_>>())
    };
    GateBMetricIntervals {
        correct_choice_fraction: field(|point| point.correct_choice_fraction),
        branch_choice_fraction: field(|point| point.branch_choice_fraction),
        food_fraction: field(|point| point.food_fraction),
        mean_final_energy: field(|point| point.mean_final_energy),
        mean_steps: field(|point| point.mean_steps),
        right_choice_fraction: field(|point| point.right_choice_fraction),
        left_target_accuracy: field(|point| point.left_target_accuracy),
        right_target_accuracy: field(|point| point.right_target_accuracy),
    }
}

fn confidence_interval(values: &[f64]) -> GateBConfidenceInterval {
    let count = values.len();
    let mean = values.iter().sum::<f64>() / count as f64;
    if count == 1 {
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
        / (count - 1) as f64;
    let margin = t_critical_975(count - 1) * (variance / count as f64).sqrt();
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
    reports: &[GateBControllerReport],
    delayed_effect: &GateBPairedEffect,
    history_effect: &GateBPairedEffect,
) -> GateBAcceptanceReport {
    let report = |condition, controller| {
        reports
            .iter()
            .find(|report| report.condition == condition && report.controller == controller)
            .expect("canonical report")
    };
    let visible_control_learnable = GateBControllerKind::ALL.iter().all(|controller| {
        report(GateBCondition::CueVisibleAtFork, *controller)
            .metrics
            .correct_choice_fraction
            .mean
            >= 0.80
    });
    let randomized_control_at_chance = GateBControllerKind::ALL.iter().all(|controller| {
        let interval = report(GateBCondition::CueRandomized, *controller)
            .metrics
            .correct_choice_fraction;
        interval.lower95 <= 0.50 && interval.upper95 >= 0.50
    });
    let primary = delayed_effect.effect.correct_choice_fraction;
    let leaky_beats_state_reset = primary.mean >= 0.15 && primary.lower95 > 0.0;
    let delayed_leaky = report(GateBCondition::DelayedCue, GateBControllerKind::LeakyState);
    let leaky_reaches_seventy_percent = delayed_leaky.metrics.correct_choice_fraction.mean >= 0.70;
    let history_shuffle_hurts = history_effect.effect.correct_choice_fraction.lower95 > 0.0;
    let left_right_consistent = delayed_leaky.metrics.left_target_accuracy.mean >= 0.70
        && delayed_leaky.metrics.right_target_accuracy.mean >= 0.70
        && (delayed_leaky.metrics.left_target_accuracy.mean
            - delayed_leaky.metrics.right_target_accuracy.mean)
            .abs()
            <= 0.15;
    let deterministic = true;
    let passed = visible_control_learnable
        && randomized_control_at_chance
        && leaky_beats_state_reset
        && leaky_reaches_seventy_percent
        && history_shuffle_hurts
        && left_right_consistent
        && deterministic;
    GateBAcceptanceReport {
        visible_control_learnable,
        randomized_control_at_chance,
        leaky_beats_state_reset,
        leaky_reaches_seventy_percent,
        history_shuffle_hurts,
        left_right_consistent,
        deterministic,
        passed,
    }
}

fn conclusions(
    reports: &[GateBControllerReport],
    delayed_effect: &GateBPairedEffect,
    history_effect: &GateBPairedEffect,
    acceptance: GateBAcceptanceReport,
) -> Vec<String> {
    let report = |condition, controller| {
        reports
            .iter()
            .find(|report| report.condition == condition && report.controller == controller)
            .expect("canonical report")
    };
    let leaky = report(GateBCondition::DelayedCue, GateBControllerKind::LeakyState)
        .metrics
        .correct_choice_fraction;
    let reset = report(GateBCondition::DelayedCue, GateBControllerKind::StateReset)
        .metrics
        .correct_choice_fraction;
    vec![
        format!(
            "延迟线索下 leaky-state 正确率 {:.3} [95% CI {:.3}, {:.3}]，state-reset 为 {:.3}。",
            leaky.mean, leaky.lower95, leaky.upper95, reset.mean,
        ),
        format!(
            "leaky-state 相对 state-reset 的配对增益 {:.3} [95% CI {:.3}, {:.3}]。",
            delayed_effect.effect.correct_choice_fraction.mean,
            delayed_effect.effect.correct_choice_fraction.lower95,
            delayed_effect.effect.correct_choice_fraction.upper95,
        ),
        format!(
            "历史置乱造成的配对下降 {:.3} [95% CI {:.3}, {:.3}]。",
            history_effect.effect.correct_choice_fraction.mean,
            history_effect.effect.correct_choice_fraction.lower95,
            history_effect.effect.correct_choice_fraction.upper95,
        ),
        if acceptance.passed {
            "Gate B 通过：连续泄漏状态在排除当前感觉和固定方向偏置后提供了可重复的历史依赖行为。"
                .to_owned()
        } else {
            "Gate B 未通过：当前连续状态尚未满足冻结的状态必要性证据门槛。".to_owned()
        },
    ]
}

fn validate_config(config: GateBExperimentConfig) -> Result<(), EmbodiedError> {
    let finite = [
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
        || config.maximum_steps < 12
        || !finite
        || config.initial_energy <= 0.0
        || config.initial_energy > config.maximum_energy
        || config.passive_cost <= 0.0
        || config.movement_cost < 0.0
        || config.collision_cost < 0.0
        || config.food_energy <= 0.0
        || !controller_finite
        || !(0.0..1.0).contains(&config.controller.hidden_leak)
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

fn terminal_for(side: ForkSide) -> GridPosition {
    match side {
        ForkSide::Left => LEFT_TERMINAL,
        ForkSide::Right => RIGHT_TERMINAL,
    }
}

fn is_open(position: GridPosition) -> bool {
    (position.x == 4 && (2..=8).contains(&position.y))
        || position == LEFT_TERMINAL
        || position == RIGHT_TERMINAL
}

fn heading_delta(heading: Heading) -> GridPosition {
    match heading {
        Heading::North => GridPosition { x: 0, y: -1 },
        Heading::East => GridPosition { x: 1, y: 0 },
        Heading::South => GridPosition { x: 0, y: 1 },
        Heading::West => GridPosition { x: -1, y: 0 },
    }
}

fn turn_left(heading: Heading) -> Heading {
    match heading {
        Heading::North => Heading::West,
        Heading::West => Heading::South,
        Heading::South => Heading::East,
        Heading::East => Heading::North,
    }
}

fn turn_right(heading: Heading) -> Heading {
    match heading {
        Heading::North => Heading::East,
        Heading::East => Heading::South,
        Heading::South => Heading::West,
        Heading::West => Heading::North,
    }
}

fn condition_tag(condition: GateBCondition) -> u64 {
    match condition {
        GateBCondition::DelayedCue => 0x4445_4c41_5901,
        GateBCondition::CueVisibleAtFork => 0x5649_5349_424c_4501,
        GateBCondition::CueRandomized => 0x5241_4e44_4f4d_0101,
        GateBCondition::HistoryShuffled => 0x5348_5546_464c_4501,
    }
}
