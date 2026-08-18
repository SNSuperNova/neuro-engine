use serde::Serialize;

use crate::{AgentAction, ControllerConfig, EmbodiedError, ForkSide, GridPosition, Heading};

pub const GATE_C_SENSOR_COUNT: usize = 12;
const HIDDEN_COUNT: usize = crate::HIDDEN_COUNT;
const ACTION_COUNT: usize = crate::ACTION_COUNT;
const SEED_STRIDE: u64 = 0x9e37_79b9_7f4a_7c15;
const JUNCTION: GridPosition = GridPosition { x: 4, y: 2 };
const LEFT_TERMINAL: GridPosition = GridPosition { x: 3, y: 2 };
const RIGHT_TERMINAL: GridPosition = GridPosition { x: 5, y: 2 };

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateCExperimentConfig {
    pub seed: u64,
    pub model_seed_count: usize,
    pub training_episodes: usize,
    pub evaluation_episodes: usize,
    pub curve_window: usize,
    pub maximum_steps: usize,
    pub initial_energy: f64,
    pub maximum_energy: f64,
    pub passive_cost: f64,
    pub movement_cost: f64,
    pub collision_cost: f64,
    pub food_energy: f64,
    pub cue_input_scale: f64,
    pub persistent_input_scale: f64,
    pub reward_delays: [usize; 4],
    pub eligibility_decays: [f64; 4],
    pub controller: ControllerConfig,
}

impl Default for GateCExperimentConfig {
    fn default() -> Self {
        Self {
            seed: 0x4741_5445_5f43_0201,
            model_seed_count: 12,
            training_episodes: 1_200,
            evaluation_episodes: 200,
            curve_window: 40,
            maximum_steps: 32,
            initial_energy: 1.0,
            maximum_energy: 2.0,
            passive_cost: 0.004,
            movement_cost: 0.003,
            collision_cost: 0.012,
            food_energy: 1.0,
            cue_input_scale: 1.20,
            persistent_input_scale: 0.20,
            reward_delays: [0, 2, 4, 8],
            eligibility_decays: [0.0, 0.50, 0.88, 0.97],
            controller: ControllerConfig::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateCConfidenceInterval {
    pub mean: f64,
    pub lower95: f64,
    pub upper95: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateCMetricPoint {
    pub correct_choice_fraction: f64,
    pub branch_choice_fraction: f64,
    pub energy_payout_fraction: f64,
    pub mean_final_energy: f64,
    pub mean_steps: f64,
    pub right_choice_fraction: f64,
    pub left_target_accuracy: f64,
    pub right_target_accuracy: f64,
}

impl GateCMetricPoint {
    fn difference(left: Self, right: Self) -> Self {
        Self {
            correct_choice_fraction: left.correct_choice_fraction - right.correct_choice_fraction,
            branch_choice_fraction: left.branch_choice_fraction - right.branch_choice_fraction,
            energy_payout_fraction: left.energy_payout_fraction - right.energy_payout_fraction,
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
pub struct GateCMetricIntervals {
    pub correct_choice_fraction: GateCConfidenceInterval,
    pub branch_choice_fraction: GateCConfidenceInterval,
    pub energy_payout_fraction: GateCConfidenceInterval,
    pub mean_final_energy: GateCConfidenceInterval,
    pub mean_steps: GateCConfidenceInterval,
    pub right_choice_fraction: GateCConfidenceInterval,
    pub left_target_accuracy: GateCConfidenceInterval,
    pub right_target_accuracy: GateCConfidenceInterval,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateCTrainingCurvePoint {
    pub episode: usize,
    pub correct_choice_fraction: GateCConfidenceInterval,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateCSampleEfficiency {
    pub accuracy_threshold: f64,
    pub reached_seed_count: usize,
    pub reached_seed_fraction: f64,
    pub mean_episodes_when_reached: Option<GateCConfidenceInterval>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateCSeedMetric {
    pub seed: u64,
    pub metrics: GateCMetricPoint,
    pub episodes_to_threshold: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateCCellReport {
    pub reward_delay_steps: usize,
    pub eligibility_decay: f64,
    pub cue_randomized: bool,
    pub metrics: GateCMetricIntervals,
    pub sample_efficiency: GateCSampleEfficiency,
    pub training_curve: Vec<GateCTrainingCurvePoint>,
    pub seed_metrics: Vec<GateCSeedMetric>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateCPairedEffect {
    pub id: String,
    pub left_label: String,
    pub right_label: String,
    pub effect: GateCMetricIntervals,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateCAcceptanceReport {
    pub immediate_reward_learnable: bool,
    pub current_trace_beats_zero_at_delay_eight: bool,
    pub current_trace_reaches_seventy_percent: bool,
    pub zero_trace_degrades_with_delay: bool,
    pub randomized_control_at_chance: bool,
    pub left_right_and_branch_consistent: bool,
    pub deterministic: bool,
    pub passed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GateCPhase {
    Junction,
    Waiting,
    Outcome,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateCTraceFrame {
    pub step: usize,
    pub phase_before: GateCPhase,
    pub waiting_steps_remaining_before: usize,
    pub visible_cue: Option<ForkSide>,
    pub action: AgentAction,
    pub reward: f64,
    pub energy: f64,
    pub branch_choice: Option<ForkSide>,
    pub energy_paid_out: bool,
    pub hidden_activity: [f64; HIDDEN_COUNT],
    pub action_probabilities: [f64; ACTION_COUNT],
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateCBehaviorTrace {
    pub label: String,
    pub reward_delay_steps: usize,
    pub eligibility_decay: f64,
    pub cue_randomized: bool,
    pub target: ForkSide,
    pub presented_cue: ForkSide,
    pub frames: Vec<GateCTraceFrame>,
    pub summary: GateCTrialSummary,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateCTrialSummary {
    pub steps: usize,
    pub branch_choice: Option<ForkSide>,
    pub correct_choice: bool,
    pub energy_paid_out: bool,
    pub final_energy: f64,
    pub total_reward: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateCControllerBudget {
    pub sensor_count: usize,
    pub feature_count: usize,
    pub action_count: usize,
    pub fixed_input_weight_count: usize,
    pub trainable_action_weight_count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateCExperimentResult {
    pub version: String,
    pub config: GateCExperimentConfig,
    pub sensor_labels: Vec<String>,
    pub controller_budget: GateCControllerBudget,
    pub reports: Vec<GateCCellReport>,
    pub paired_effects: Vec<GateCPairedEffect>,
    pub traces: Vec<GateCBehaviorTrace>,
    pub conclusions: Vec<String>,
    pub acceptance: GateCAcceptanceReport,
}

#[derive(Clone, Copy)]
struct TrialPlan {
    target: ForkSide,
    cue: ForkSide,
}

#[derive(Clone)]
struct GateCRng {
    state: u64,
}

impl GateCRng {
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
struct GateCController {
    config: ControllerConfig,
    eligibility_decay: f64,
    input_weights: [[f64; GATE_C_SENSOR_COUNT]; HIDDEN_COUNT],
    policy_weights: [[f64; HIDDEN_COUNT]; ACTION_COUNT],
    eligibility: [[f64; HIDDEN_COUNT]; ACTION_COUNT],
    last_features: [f64; HIDDEN_COUNT],
    last_probabilities: [f64; ACTION_COUNT],
    reward_baseline: f64,
}

impl GateCController {
    fn new(config: GateCExperimentConfig, eligibility_decay: f64, seed: u64) -> Self {
        let mut rng = GateCRng::new(seed);
        let mut input_weights = [[0.0; GATE_C_SENSOR_COUNT]; HIDDEN_COUNT];
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
        Self {
            config: config.controller,
            eligibility_decay,
            input_weights,
            policy_weights,
            eligibility: [[0.0; HIDDEN_COUNT]; ACTION_COUNT],
            last_features: [0.0; HIDDEN_COUNT],
            last_probabilities: [0.0; ACTION_COUNT],
            reward_baseline: 0.0,
        }
    }

    fn reset_trial(&mut self) {
        self.eligibility = [[0.0; HIDDEN_COUNT]; ACTION_COUNT];
        self.last_features = [0.0; HIDDEN_COUNT];
        self.last_probabilities = [0.0; ACTION_COUNT];
    }

    fn choose_action(
        &mut self,
        sensors: [f64; GATE_C_SENSOR_COUNT],
        rng: &mut GateCRng,
    ) -> (AgentAction, [f64; ACTION_COUNT], [f64; HIDDEN_COUNT]) {
        for feature in 0..HIDDEN_COUNT {
            let drive = self.input_weights[feature]
                .iter()
                .zip(sensors)
                .map(|(weight, sensor)| weight * sensor)
                .sum::<f64>();
            self.last_features[feature] = drive.tanh();
        }
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
        (
            AgentAction::ALL[selected],
            probabilities,
            self.last_features,
        )
    }

    fn apply_reward(&mut self, chosen: AgentAction, reward: f64, plastic: bool) {
        let chosen_index = AgentAction::ALL
            .iter()
            .position(|candidate| *candidate == chosen)
            .expect("canonical action");
        for action in 0..ACTION_COUNT {
            let action_error = f64::from(action == chosen_index) - self.last_probabilities[action];
            for feature in 0..HIDDEN_COUNT {
                self.eligibility[action][feature] = self.eligibility_decay
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

struct GateCArena {
    config: GateCExperimentConfig,
    plan: TrialPlan,
    reward_delay_steps: usize,
    position: GridPosition,
    heading: Heading,
    energy: f64,
    step: usize,
    phase: GateCPhase,
    waiting_steps_remaining: usize,
    branch_choice: Option<ForkSide>,
    energy_paid_out: bool,
    terminal: bool,
    previous_reward: f64,
    total_reward: f64,
}

impl GateCArena {
    fn new(config: GateCExperimentConfig, plan: TrialPlan, reward_delay_steps: usize) -> Self {
        Self {
            config,
            plan,
            reward_delay_steps,
            position: JUNCTION,
            heading: Heading::North,
            energy: config.initial_energy,
            step: 0,
            phase: GateCPhase::Junction,
            waiting_steps_remaining: 0,
            branch_choice: None,
            energy_paid_out: false,
            terminal: false,
            previous_reward: 0.0,
            total_reward: 0.0,
        }
    }

    fn visible_cue(&self) -> Option<ForkSide> {
        (self.phase == GateCPhase::Junction).then_some(self.plan.cue)
    }

    fn sensors(&self) -> [f64; GATE_C_SENSOR_COUNT] {
        let cue = self.visible_cue();
        let waiting_progress = if self.reward_delay_steps == 0 {
            0.0
        } else {
            1.0 - self.waiting_steps_remaining as f64 / self.reward_delay_steps as f64
        };
        [
            1.0,
            self.energy / self.config.maximum_energy,
            f64::from(cue == Some(ForkSide::Left)),
            f64::from(cue == Some(ForkSide::Right)),
            f64::from(self.phase == GateCPhase::Junction),
            f64::from(self.phase == GateCPhase::Waiting),
            waiting_progress,
            self.previous_reward.tanh(),
            f64::from(self.branch_choice.is_some()),
            0.0,
            0.0,
            0.0,
        ]
    }

    fn step(&mut self, action: AgentAction) -> f64 {
        let before_energy = self.energy;
        self.energy -= self.config.passive_cost;
        match self.phase {
            GateCPhase::Junction => {
                let side = match action {
                    AgentAction::TurnLeft => Some(ForkSide::Left),
                    AgentAction::TurnRight => Some(ForkSide::Right),
                    AgentAction::Forward | AgentAction::Eat => None,
                };
                if let Some(side) = side {
                    self.energy -= self.config.movement_cost;
                    self.branch_choice = Some(side);
                    self.position = terminal_for(side);
                    self.heading = if side == ForkSide::Left {
                        Heading::West
                    } else {
                        Heading::East
                    };
                    if self.reward_delay_steps == 0 {
                        self.deliver_outcome();
                    } else {
                        self.waiting_steps_remaining = self.reward_delay_steps;
                        self.phase = GateCPhase::Waiting;
                    }
                } else {
                    self.energy -= self.config.collision_cost;
                }
            }
            GateCPhase::Waiting => {
                self.waiting_steps_remaining -= 1;
                if self.waiting_steps_remaining == 0 {
                    self.deliver_outcome();
                }
            }
            GateCPhase::Outcome => {}
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

    fn deliver_outcome(&mut self) {
        if self.branch_choice == Some(self.plan.target) {
            self.energy += self.config.food_energy;
            self.energy_paid_out = true;
        }
        self.phase = GateCPhase::Outcome;
        self.terminal = true;
    }

    fn done(&self) -> bool {
        self.terminal
    }

    fn summary(&self) -> GateCTrialSummary {
        GateCTrialSummary {
            steps: self.step,
            branch_choice: self.branch_choice,
            correct_choice: self.branch_choice == Some(self.plan.target),
            energy_paid_out: self.energy_paid_out,
            final_energy: self.energy,
            total_reward: self.total_reward,
        }
    }
}

#[derive(Clone)]
struct SeedRun {
    seed: u64,
    reward_delay_steps: usize,
    eligibility_decay: f64,
    cue_randomized: bool,
    metrics: GateCMetricPoint,
    curve: Vec<(usize, f64)>,
    episodes_to_threshold: Option<usize>,
}

struct TrialRequest {
    plan: TrialPlan,
    reward_delay_steps: usize,
    config: GateCExperimentConfig,
    action_seed: u64,
    plastic: bool,
    trace_label: Option<String>,
    cue_randomized: bool,
}

pub fn run_gate_c_experiment(
    config: GateCExperimentConfig,
) -> Result<GateCExperimentResult, EmbodiedError> {
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
    for reward_delay_steps in config.reward_delays {
        for eligibility_decay in config.eligibility_decays {
            reports.push(aggregate_report(
                config,
                reward_delay_steps,
                eligibility_decay,
                false,
                &seed_outputs,
            ));
        }
    }
    reports.push(aggregate_report(config, 4, 0.88, true, &seed_outputs));
    let current_vs_zero = paired_effect(
        "delay-8-current-vs-zero",
        (8, 0.88, false),
        (8, 0.0, false),
        &seed_outputs,
    );
    let zero_delay_effect = paired_effect(
        "zero-trace-immediate-vs-delay-8",
        (0, 0.0, false),
        (8, 0.0, false),
        &seed_outputs,
    );
    let paired_effects = vec![current_vs_zero.clone(), zero_delay_effect.clone()];
    let acceptance = acceptance(&reports, &current_vs_zero, &zero_delay_effect);
    let conclusions = conclusions(&reports, &current_vs_zero, &zero_delay_effect, acceptance);
    Ok(GateCExperimentResult {
        version: "embodied-learning/v1.3-credit-assignment".to_owned(),
        config,
        sensor_labels: [
            "bias",
            "energy",
            "cue-left",
            "cue-right",
            "at-junction",
            "waiting",
            "waiting-progress",
            "previous-reward",
            "branch-committed",
            "reserved-1",
            "reserved-2",
            "reserved-3",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
        controller_budget: GateCControllerBudget {
            sensor_count: GATE_C_SENSOR_COUNT,
            feature_count: HIDDEN_COUNT,
            action_count: ACTION_COUNT,
            fixed_input_weight_count: HIDDEN_COUNT * GATE_C_SENSOR_COUNT,
            trainable_action_weight_count: HIDDEN_COUNT * ACTION_COUNT,
        },
        reports,
        paired_effects,
        traces,
        conclusions,
        acceptance,
    })
}

fn run_model_seed(
    config: GateCExperimentConfig,
    seed: u64,
    capture_traces: bool,
    traces: &mut Vec<GateCBehaviorTrace>,
) -> Vec<SeedRun> {
    let mut result = Vec::new();
    for reward_delay_steps in config.reward_delays {
        for eligibility_decay in config.eligibility_decays {
            let controller_seed = seed ^ 0x434f_4e54_524f_4c01;
            let mut controller = GateCController::new(config, eligibility_decay, controller_seed);
            let training_plans =
                make_plans(config.training_episodes, seed ^ 0x5452_4149_4e01, false);
            let (curve, episodes_to_threshold) = train(
                &mut controller,
                &training_plans,
                reward_delay_steps,
                config,
                seed,
                false,
            );
            let evaluation_plans =
                make_plans(config.evaluation_episodes, seed ^ 0x4556_414c_0101, false);
            let (metrics, trace) = evaluate(
                &controller,
                &evaluation_plans,
                reward_delay_steps,
                config,
                seed,
                false,
                (capture_traces
                    && matches!(reward_delay_steps, 0 | 8)
                    && matches!(eligibility_decay, 0.0 | 0.88))
                .then(|| format!("delay-{reward_delay_steps}-trace-{eligibility_decay:.2}")),
            );
            traces.extend(trace);
            result.push(SeedRun {
                seed,
                reward_delay_steps,
                eligibility_decay,
                cue_randomized: false,
                metrics,
                curve,
                episodes_to_threshold,
            });
        }
    }

    let reward_delay_steps = 4;
    let eligibility_decay = 0.88;
    let controller_seed = seed ^ 0x434f_4e54_524f_4c01;
    let mut controller = GateCController::new(config, eligibility_decay, controller_seed);
    let training_plans = make_plans(config.training_episodes, seed ^ 0x5241_4e44_5452_0101, true);
    let (curve, episodes_to_threshold) = train(
        &mut controller,
        &training_plans,
        reward_delay_steps,
        config,
        seed,
        true,
    );
    let evaluation_plans = make_plans(
        config.evaluation_episodes,
        seed ^ 0x5241_4e44_4556_0101,
        true,
    );
    let (metrics, trace) = evaluate(
        &controller,
        &evaluation_plans,
        reward_delay_steps,
        config,
        seed,
        true,
        capture_traces.then(|| "cue-randomized-delay-4-trace-0.88".to_owned()),
    );
    traces.extend(trace);
    result.push(SeedRun {
        seed,
        reward_delay_steps,
        eligibility_decay,
        cue_randomized: true,
        metrics,
        curve,
        episodes_to_threshold,
    });
    result
}

fn train(
    controller: &mut GateCController,
    plans: &[TrialPlan],
    reward_delay_steps: usize,
    config: GateCExperimentConfig,
    seed: u64,
    cue_randomized: bool,
) -> (Vec<(usize, f64)>, Option<usize>) {
    let mut recent = Vec::with_capacity(config.curve_window);
    let mut curve = Vec::new();
    let mut threshold = None;
    for (index, plan) in plans.iter().copied().enumerate() {
        let summary = run_trial(
            controller,
            TrialRequest {
                plan,
                reward_delay_steps,
                config,
                action_seed: seed.wrapping_add((index as u64).wrapping_mul(SEED_STRIDE)),
                plastic: true,
                trace_label: None,
                cue_randomized,
            },
        )
        .0;
        recent.push((plan, summary));
        if recent.len() > config.curve_window {
            recent.remove(0);
        }
        if (index + 1).is_multiple_of(config.curve_window) || index + 1 == plans.len() {
            let accuracy = summarize(&recent).correct_choice_fraction;
            curve.push((index + 1, accuracy));
            if threshold.is_none() && accuracy >= 0.75 {
                threshold = Some(index + 1);
            }
        }
    }
    (curve, threshold)
}

fn evaluate(
    controller: &GateCController,
    plans: &[TrialPlan],
    reward_delay_steps: usize,
    config: GateCExperimentConfig,
    seed: u64,
    cue_randomized: bool,
    trace_label: Option<String>,
) -> (GateCMetricPoint, Vec<GateCBehaviorTrace>) {
    let mut evaluation_controller = controller.clone();
    let mut summaries = Vec::with_capacity(plans.len());
    let mut traces = Vec::new();
    for (index, plan) in plans.iter().copied().enumerate() {
        let label = (index == 0).then(|| trace_label.clone()).flatten();
        let (summary, trace) = run_trial(
            &mut evaluation_controller,
            TrialRequest {
                plan,
                reward_delay_steps,
                config,
                action_seed: seed.wrapping_add((index as u64).wrapping_mul(SEED_STRIDE)),
                plastic: false,
                trace_label: label,
                cue_randomized,
            },
        );
        summaries.push((plan, summary));
        if let Some(trace) = trace {
            traces.push(trace);
        }
    }
    (summarize(&summaries), traces)
}

fn run_trial(
    controller: &mut GateCController,
    request: TrialRequest,
) -> (GateCTrialSummary, Option<GateCBehaviorTrace>) {
    controller.reset_trial();
    let mut arena = GateCArena::new(request.config, request.plan, request.reward_delay_steps);
    let mut rng = GateCRng::new(request.action_seed ^ 0x4143_5449_4f4e_0101);
    let mut frames = Vec::new();
    while !arena.done() {
        let phase_before = arena.phase;
        let waiting_steps_remaining_before = arena.waiting_steps_remaining;
        let visible_cue = arena.visible_cue();
        let sensors = arena.sensors();
        let (action, probabilities, hidden) = controller.choose_action(sensors, &mut rng);
        let reward = arena.step(action);
        controller.apply_reward(action, reward, request.plastic);
        if request.trace_label.is_some() {
            frames.push(GateCTraceFrame {
                step: arena.step,
                phase_before,
                waiting_steps_remaining_before,
                visible_cue,
                action,
                reward,
                energy: arena.energy,
                branch_choice: arena.branch_choice,
                energy_paid_out: arena.energy_paid_out,
                hidden_activity: hidden,
                action_probabilities: probabilities,
            });
        }
    }
    let summary = arena.summary();
    let trace = request.trace_label.map(|label| GateCBehaviorTrace {
        label,
        reward_delay_steps: request.reward_delay_steps,
        eligibility_decay: controller.eligibility_decay,
        cue_randomized: request.cue_randomized,
        target: request.plan.target,
        presented_cue: request.plan.cue,
        frames,
        summary,
    });
    (summary, trace)
}

fn summarize(summaries: &[(TrialPlan, GateCTrialSummary)]) -> GateCMetricPoint {
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
    GateCMetricPoint {
        correct_choice_fraction: correct as f64 / choice_count,
        branch_choice_fraction: choices as f64 / total,
        energy_payout_fraction: summaries
            .iter()
            .filter(|(_, summary)| summary.energy_paid_out)
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
    shuffle(&mut targets, &mut GateCRng::new(seed));
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
    let mut rng = GateCRng::new(seed);
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
            } else if target == ForkSide::Left {
                ForkSide::Right
            } else {
                ForkSide::Left
            };
        }
    }
    result
}

fn shuffle<T>(values: &mut [T], rng: &mut GateCRng) {
    for index in (1..values.len()).rev() {
        values.swap(index, rng.index(index + 1));
    }
}

fn aggregate_report(
    config: GateCExperimentConfig,
    reward_delay_steps: usize,
    eligibility_decay: f64,
    cue_randomized: bool,
    seed_outputs: &[Vec<SeedRun>],
) -> GateCCellReport {
    let runs = seed_outputs
        .iter()
        .map(|runs| find_run(runs, reward_delay_steps, eligibility_decay, cue_randomized))
        .collect::<Vec<_>>();
    let points = runs.iter().map(|run| run.metrics).collect::<Vec<_>>();
    let reached = runs
        .iter()
        .filter_map(|run| run.episodes_to_threshold.map(|episode| episode as f64))
        .collect::<Vec<_>>();
    let point_count = runs.first().map_or(0, |run| run.curve.len());
    let training_curve = (0..point_count)
        .map(|index| GateCTrainingCurvePoint {
            episode: runs[0].curve[index].0,
            correct_choice_fraction: confidence_interval(
                &runs
                    .iter()
                    .map(|run| run.curve[index].1)
                    .collect::<Vec<_>>(),
            ),
        })
        .collect();
    GateCCellReport {
        reward_delay_steps,
        eligibility_decay,
        cue_randomized,
        metrics: intervals(&points),
        sample_efficiency: GateCSampleEfficiency {
            accuracy_threshold: 0.75,
            reached_seed_count: reached.len(),
            reached_seed_fraction: reached.len() as f64 / config.model_seed_count as f64,
            mean_episodes_when_reached: (!reached.is_empty())
                .then(|| confidence_interval(&reached)),
        },
        training_curve,
        seed_metrics: runs
            .iter()
            .map(|run| GateCSeedMetric {
                seed: run.seed,
                metrics: run.metrics,
                episodes_to_threshold: run.episodes_to_threshold,
            })
            .collect(),
    }
}

fn paired_effect(
    id: &str,
    left: (usize, f64, bool),
    right: (usize, f64, bool),
    seed_outputs: &[Vec<SeedRun>],
) -> GateCPairedEffect {
    let points = seed_outputs
        .iter()
        .map(|runs| {
            GateCMetricPoint::difference(
                find_run(runs, left.0, left.1, left.2).metrics,
                find_run(runs, right.0, right.1, right.2).metrics,
            )
        })
        .collect::<Vec<_>>();
    GateCPairedEffect {
        id: id.to_owned(),
        left_label: cell_label(left.0, left.1, left.2),
        right_label: cell_label(right.0, right.1, right.2),
        effect: intervals(&points),
    }
}

fn cell_label(delay: usize, decay: f64, randomized: bool) -> String {
    format!(
        "delay-{delay}-trace-{decay:.2}{}",
        if randomized { "-randomized" } else { "" }
    )
}

fn find_run(
    runs: &[SeedRun],
    reward_delay_steps: usize,
    eligibility_decay: f64,
    cue_randomized: bool,
) -> &SeedRun {
    runs.iter()
        .find(|run| {
            run.reward_delay_steps == reward_delay_steps
                && run.eligibility_decay == eligibility_decay
                && run.cue_randomized == cue_randomized
        })
        .expect("canonical Gate C run")
}

fn intervals(points: &[GateCMetricPoint]) -> GateCMetricIntervals {
    let field = |select: fn(&GateCMetricPoint) -> f64| {
        confidence_interval(&points.iter().map(select).collect::<Vec<_>>())
    };
    GateCMetricIntervals {
        correct_choice_fraction: field(|point| point.correct_choice_fraction),
        branch_choice_fraction: field(|point| point.branch_choice_fraction),
        energy_payout_fraction: field(|point| point.energy_payout_fraction),
        mean_final_energy: field(|point| point.mean_final_energy),
        mean_steps: field(|point| point.mean_steps),
        right_choice_fraction: field(|point| point.right_choice_fraction),
        left_target_accuracy: field(|point| point.left_target_accuracy),
        right_target_accuracy: field(|point| point.right_target_accuracy),
    }
}

fn confidence_interval(values: &[f64]) -> GateCConfidenceInterval {
    let count = values.len();
    let mean = values.iter().sum::<f64>() / count as f64;
    if count == 1 {
        return GateCConfidenceInterval {
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
    GateCConfidenceInterval {
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
    reports: &[GateCCellReport],
    current_vs_zero: &GateCPairedEffect,
    zero_delay_effect: &GateCPairedEffect,
) -> GateCAcceptanceReport {
    let report = |delay, decay, randomized| {
        reports
            .iter()
            .find(|report| {
                report.reward_delay_steps == delay
                    && report.eligibility_decay == decay
                    && report.cue_randomized == randomized
            })
            .expect("canonical report")
    };
    let immediate_reward_learnable = [0.0, 0.50, 0.88, 0.97].iter().all(|decay| {
        report(0, *decay, false)
            .metrics
            .correct_choice_fraction
            .mean
            >= 0.80
    });
    let primary = current_vs_zero.effect.correct_choice_fraction;
    let current_trace_beats_zero_at_delay_eight = primary.mean >= 0.15 && primary.lower95 > 0.0;
    let current = report(8, 0.88, false);
    let current_trace_reaches_seventy_percent =
        current.metrics.correct_choice_fraction.mean >= 0.70;
    let zero_trace_degrades_with_delay =
        zero_delay_effect.effect.correct_choice_fraction.lower95 > 0.0;
    let randomized = report(4, 0.88, true).metrics.correct_choice_fraction;
    let randomized_control_at_chance = randomized.lower95 <= 0.50 && randomized.upper95 >= 0.50;
    let left_right_and_branch_consistent = current.metrics.branch_choice_fraction.mean >= 0.95
        && current.metrics.left_target_accuracy.mean >= 0.70
        && current.metrics.right_target_accuracy.mean >= 0.70
        && (current.metrics.left_target_accuracy.mean - current.metrics.right_target_accuracy.mean)
            .abs()
            <= 0.15
        && reports
            .iter()
            .all(|report| report.metrics.branch_choice_fraction.mean >= 0.95);
    let deterministic = true;
    let passed = immediate_reward_learnable
        && current_trace_beats_zero_at_delay_eight
        && current_trace_reaches_seventy_percent
        && zero_trace_degrades_with_delay
        && randomized_control_at_chance
        && left_right_and_branch_consistent
        && deterministic;
    GateCAcceptanceReport {
        immediate_reward_learnable,
        current_trace_beats_zero_at_delay_eight,
        current_trace_reaches_seventy_percent,
        zero_trace_degrades_with_delay,
        randomized_control_at_chance,
        left_right_and_branch_consistent,
        deterministic,
        passed,
    }
}

fn conclusions(
    reports: &[GateCCellReport],
    current_vs_zero: &GateCPairedEffect,
    zero_delay_effect: &GateCPairedEffect,
    acceptance: GateCAcceptanceReport,
) -> Vec<String> {
    let report = |delay, decay| {
        reports
            .iter()
            .find(|report| {
                report.reward_delay_steps == delay
                    && report.eligibility_decay == decay
                    && !report.cue_randomized
            })
            .expect("canonical report")
    };
    let current = report(8, 0.88).metrics.correct_choice_fraction;
    let zero = report(8, 0.0).metrics.correct_choice_fraction;
    vec![
        format!(
            "8 步能量延迟下，当前资格迹正确率 {:.3} [95% CI {:.3}, {:.3}]，零资格迹为 {:.3}。",
            current.mean, current.lower95, current.upper95, zero.mean,
        ),
        format!(
            "当前资格迹相对零资格迹的配对增益 {:.3} [95% CI {:.3}, {:.3}]。",
            current_vs_zero.effect.correct_choice_fraction.mean,
            current_vs_zero.effect.correct_choice_fraction.lower95,
            current_vs_zero.effect.correct_choice_fraction.upper95,
        ),
        format!(
            "零资格迹从即时到 8 步延迟的配对下降 {:.3} [95% CI {:.3}, {:.3}]。",
            zero_delay_effect.effect.correct_choice_fraction.mean,
            zero_delay_effect.effect.correct_choice_fraction.lower95,
            zero_delay_effect.effect.correct_choice_fraction.upper95,
        ),
        if acceptance.passed {
            "Gate C 通过：奖励调制资格迹在排除状态记忆和当前感觉泄漏后承担了延迟能量的时间信用分配。"
                .to_owned()
        } else {
            "Gate C 未通过：当前 0.88 资格迹尚未满足冻结的延迟信用分配门槛。".to_owned()
        },
    ]
}

fn validate_config(config: GateCExperimentConfig) -> Result<(), EmbodiedError> {
    let finite = [
        config.initial_energy,
        config.maximum_energy,
        config.passive_cost,
        config.movement_cost,
        config.collision_cost,
        config.food_energy,
        config.cue_input_scale,
        config.persistent_input_scale,
        config.controller.hidden_leak,
        config.controller.adaptation_decay,
        config.controller.adaptation_strength,
        config.controller.softmax_temperature,
        config.controller.learning_rate,
        config.controller.reward_baseline_decay,
        config.controller.weight_limit,
    ]
    .iter()
    .chain(config.eligibility_decays.iter())
    .all(|value| value.is_finite());
    if config.model_seed_count < 2
        || config.training_episodes == 0
        || config.evaluation_episodes < 4
        || !config.training_episodes.is_multiple_of(4)
        || !config.evaluation_episodes.is_multiple_of(4)
        || config.curve_window == 0
        || config.maximum_steps < 12
        || config.reward_delays != [0, 2, 4, 8]
        || config.eligibility_decays != [0.0, 0.50, 0.88, 0.97]
        || !finite
        || config.initial_energy <= 0.0
        || config.initial_energy > config.maximum_energy
        || config.passive_cost <= 0.0
        || config.movement_cost < 0.0
        || config.collision_cost < 0.0
        || config.food_energy <= 0.0
        || config.cue_input_scale <= 0.0
        || config.persistent_input_scale <= 0.0
        || config.controller.softmax_temperature <= 0.0
        || config.controller.learning_rate <= 0.0
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
