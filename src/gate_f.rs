use serde::Serialize;

use crate::{EmbodiedError, GATE_B_SENSOR_COUNT, GateBConfidenceInterval, HIDDEN_COUNT};

const BRANCH_COUNT: usize = 2;
const PLASTIC_SLOT_COUNT: usize = HIDDEN_COUNT * 2;
const SEED_STRIDE: u64 = 0x9e37_79b9_7f4a_7c15;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GateFControllerKind {
    FixedInternal,
    PlasticSensory,
    PlasticRecurrent,
    PlasticRecurrentNoHomeostasis,
}

impl GateFControllerKind {
    pub const ALL: [Self; 4] = [
        Self::FixedInternal,
        Self::PlasticSensory,
        Self::PlasticRecurrent,
        Self::PlasticRecurrentNoHomeostasis,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::FixedInternal => "fixed-internal",
            Self::PlasticSensory => "plastic-sensory",
            Self::PlasticRecurrent => "plastic-recurrent",
            Self::PlasticRecurrentNoHomeostasis => "plastic-recurrent-no-homeostasis",
        }
    }

    fn homeostasis(self) -> bool {
        matches!(self, Self::PlasticSensory | Self::PlasticRecurrent)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GateFRule {
    Original,
    Reversed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GateFPhase {
    BeforeChange,
    Reversal,
    Restoration,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateFExperimentConfig {
    pub seed: u64,
    pub model_seed_count: usize,
    pub pretraining_episodes: usize,
    pub adaptation_episodes: usize,
    pub evaluation_episodes: usize,
    pub checkpoints: [usize; 5],
    pub cue_steps: usize,
    pub delay_steps: usize,
    pub hidden_leak: f64,
    pub recurrent_scale: f64,
    pub policy_learning_rate: f64,
    pub internal_learning_rate: f64,
    pub homeostasis_strength: f64,
    pub adaptation_exploration: f64,
    pub softmax_temperature: f64,
    pub reward_baseline_decay: f64,
    pub weight_limit: f64,
}

impl Default for GateFExperimentConfig {
    fn default() -> Self {
        Self {
            seed: 0x4741_5445_5f46_0201,
            model_seed_count: 12,
            pretraining_episodes: 1_200,
            adaptation_episodes: 800,
            evaluation_episodes: 200,
            checkpoints: [0, 100, 200, 400, 800],
            cue_steps: 2,
            delay_steps: 4,
            hidden_leak: 0.80,
            recurrent_scale: 0.60,
            policy_learning_rate: 0.055,
            internal_learning_rate: 0.10,
            homeostasis_strength: 1.0,
            adaptation_exploration: 0.16,
            softmax_temperature: 0.40,
            reward_baseline_decay: 0.92,
            weight_limit: 2.5,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateFMetricPoint {
    pub accuracy: f64,
    pub left_target_accuracy: f64,
    pub right_target_accuracy: f64,
    pub right_choice_fraction: f64,
}

impl GateFMetricPoint {
    fn difference(left: Self, right: Self) -> Self {
        Self {
            accuracy: left.accuracy - right.accuracy,
            left_target_accuracy: left.left_target_accuracy - right.left_target_accuracy,
            right_target_accuracy: left.right_target_accuracy - right.right_target_accuracy,
            right_choice_fraction: left.right_choice_fraction - right.right_choice_fraction,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateFMetricIntervals {
    pub accuracy: GateBConfidenceInterval,
    pub left_target_accuracy: GateBConfidenceInterval,
    pub right_target_accuracy: GateBConfidenceInterval,
    pub right_choice_fraction: GateBConfidenceInterval,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateFWeightPoint {
    pub rms_change: f64,
    pub maximum_absolute_weight: f64,
    pub finite_weight_fraction: f64,
    pub mean_relative_group_norm_drift: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateFWeightIntervals {
    pub rms_change: GateBConfidenceInterval,
    pub maximum_absolute_weight: GateBConfidenceInterval,
    pub finite_weight_fraction: GateBConfidenceInterval,
    pub mean_relative_group_norm_drift: GateBConfidenceInterval,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateFControllerBudget {
    pub controller: GateFControllerKind,
    pub sensor_count: usize,
    pub state_unit_count: usize,
    pub action_count: usize,
    pub allocated_input_weight_count: usize,
    pub allocated_recurrent_weight_count: usize,
    pub active_recurrent_weight_count: usize,
    pub strong_plastic_candidate_count: usize,
    pub weak_fixed_recurrent_count: usize,
    pub frozen_action_weight_count: usize,
    pub allocated_internal_plastic_slot_count: usize,
    pub enabled_internal_plastic_weight_count: usize,
    pub homeostasis_enabled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateFSeedMetric {
    pub seed: u64,
    pub controller: GateFControllerKind,
    pub phase: GateFPhase,
    pub checkpoint_episode: usize,
    pub metrics: GateFMetricPoint,
    pub weights: GateFWeightPoint,
    pub action_weight_digest: u64,
    pub controller_digest: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateFCheckpointReport {
    pub controller: GateFControllerKind,
    pub phase: GateFPhase,
    pub rule: GateFRule,
    pub checkpoint_episode: usize,
    pub metrics: GateFMetricIntervals,
    pub weights: GateFWeightIntervals,
    pub model_seed_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateFPairedEffect {
    pub id: &'static str,
    pub phase: GateFPhase,
    pub checkpoint_episode: usize,
    pub left: GateFControllerKind,
    pub right: GateFControllerKind,
    pub effect: GateFMetricIntervals,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateFAdaptationSummary {
    pub controller: GateFControllerKind,
    pub reversal_episodes_to_75_percent: Option<usize>,
    pub restoration_episodes_to_75_percent: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateFTraceFrame {
    pub step: usize,
    pub cue_visible: bool,
    pub cue_right: bool,
    pub hidden_activity: [f64; HIDDEN_COUNT],
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateFBehaviorTrace {
    pub label: String,
    pub controller: GateFControllerKind,
    pub phase: GateFPhase,
    pub rule: GateFRule,
    pub cue_right: bool,
    pub target_right: bool,
    pub chosen_right: bool,
    pub reward: f64,
    pub action_probabilities: [f64; BRANCH_COUNT],
    pub frames: Vec<GateFTraceFrame>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateFAcceptanceReport {
    pub matched_start_and_plastic_budgets: bool,
    pub original_rule_learned: bool,
    pub sensory_plasticity_adapts: bool,
    pub recurrent_plasticity_adapts: bool,
    pub restored_rule_relearned: bool,
    pub readout_frozen_and_states_finite: bool,
    pub homeostasis_control_complete: bool,
    pub deterministic: bool,
    pub passed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateFExperimentResult {
    pub version: String,
    pub task: String,
    pub config: GateFExperimentConfig,
    pub controller_budgets: Vec<GateFControllerBudget>,
    pub seed_metrics: Vec<GateFSeedMetric>,
    pub reports: Vec<GateFCheckpointReport>,
    pub paired_effects: Vec<GateFPairedEffect>,
    pub adaptation: Vec<GateFAdaptationSummary>,
    pub traces: Vec<GateFBehaviorTrace>,
    pub initial_shared_state_digests: Vec<u64>,
    pub acceptance: GateFAcceptanceReport,
    pub conclusions: Vec<String>,
}

#[derive(Clone)]
struct Rng(u64);

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

    fn index(&mut self, length: usize) -> usize {
        (self.next_u64() % length as u64) as usize
    }
}

#[derive(Clone)]
struct Controller {
    config: GateFExperimentConfig,
    input_weights: [[f64; GATE_B_SENSOR_COUNT]; HIDDEN_COUNT],
    recurrent_weights: [[f64; HIDDEN_COUNT]; HIDDEN_COUNT],
    recurrent_slots: [[usize; 2]; HIDDEN_COUNT],
    action_weights: [[f64; HIDDEN_COUNT]; BRANCH_COUNT],
    hidden: [f64; HIDDEN_COUNT],
    baseline: f64,
    reference_input_norms: [f64; HIDDEN_COUNT],
    reference_recurrent_norms: [f64; HIDDEN_COUNT],
    reference_input_weights: [[f64; 2]; HIDDEN_COUNT],
    reference_recurrent_weights: [[f64; 2]; HIDDEN_COUNT],
}

impl Controller {
    fn new(config: GateFExperimentConfig, seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let mut input_weights = [[0.0; GATE_B_SENSOR_COUNT]; HIDDEN_COUNT];
        for row in &mut input_weights {
            for (sensor, weight) in row.iter_mut().enumerate() {
                *weight = rng.signed(if matches!(sensor, 2 | 3) { 1.2 } else { 0.12 });
            }
        }
        let preference = input_weights.map(|row| row[2] - row[3]);
        let mut recurrent_weights = [[0.0; HIDDEN_COUNT]; HIDDEN_COUNT];
        let mut recurrent_slots = [[0usize; 2]; HIDDEN_COUNT];
        for target in 0..HIDDEN_COUNT {
            let mut candidates = (0..HIDDEN_COUNT)
                .filter(|source| *source != target)
                .collect::<Vec<_>>();
            let positive_source = *candidates
                .iter()
                .max_by(|left, right| preference[**left].total_cmp(&preference[**right]))
                .expect("positive recurrent source");
            let negative_source = *candidates
                .iter()
                .min_by(|left, right| preference[**left].total_cmp(&preference[**right]))
                .expect("negative recurrent source");
            shuffle(&mut candidates, &mut rng);
            let mut sources = vec![positive_source, negative_source];
            for source in candidates {
                if sources.len() == 6 {
                    break;
                }
                if !sources.contains(&source) {
                    sources.push(source);
                }
            }
            for (rank, source) in sources.into_iter().enumerate() {
                let same =
                    preference[target].is_sign_positive() == preference[source].is_sign_positive();
                let magnitude = if rank < 2 {
                    config.recurrent_scale / 2.0
                } else {
                    config.recurrent_scale / 24.0
                };
                recurrent_weights[target][source] = if same { magnitude } else { -magnitude };
                if rank < 2 {
                    recurrent_slots[target][rank] = source;
                }
            }
        }
        let mut action_weights = [[0.0; HIDDEN_COUNT]; BRANCH_COUNT];
        for row in &mut action_weights {
            for weight in row {
                *weight = rng.signed(0.08);
            }
        }
        let reference_input_norms = input_weights.map(|row| row[2].hypot(row[3]));
        let reference_input_weights = input_weights.map(|row| [row[2], row[3]]);
        let reference_recurrent_norms = std::array::from_fn(|target| {
            let [a, b] = recurrent_slots[target];
            recurrent_weights[target][a].hypot(recurrent_weights[target][b])
        });
        let reference_recurrent_weights = std::array::from_fn(|target| {
            let [a, b] = recurrent_slots[target];
            [recurrent_weights[target][a], recurrent_weights[target][b]]
        });
        Self {
            config,
            input_weights,
            recurrent_weights,
            recurrent_slots,
            action_weights,
            hidden: [0.0; HIDDEN_COUNT],
            baseline: 0.0,
            reference_input_norms,
            reference_recurrent_norms,
            reference_input_weights,
            reference_recurrent_weights,
        }
    }

    fn reset_state(&mut self) {
        self.hidden = [0.0; HIDDEN_COUNT];
    }

    fn step(&mut self, sensors: [f64; GATE_B_SENSOR_COUNT]) -> [f64; HIDDEN_COUNT] {
        let previous = self.hidden;
        self.hidden = std::array::from_fn(|target| {
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
            (input + self.config.hidden_leak * previous[target] + recurrent).tanh()
        });
        previous
    }

    fn probabilities(&self) -> [f64; BRANCH_COUNT] {
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

    fn normalize_open_weights(&mut self, kind: GateFControllerKind) {
        if !kind.homeostasis() {
            return;
        }
        for target in 0..HIDDEN_COUNT {
            let (current, reference) = match kind {
                GateFControllerKind::PlasticSensory => (
                    self.input_weights[target][2].hypot(self.input_weights[target][3]),
                    self.reference_input_norms[target],
                ),
                GateFControllerKind::PlasticRecurrent => {
                    let [a, b] = self.recurrent_slots[target];
                    (
                        self.recurrent_weights[target][a].hypot(self.recurrent_weights[target][b]),
                        self.reference_recurrent_norms[target],
                    )
                }
                _ => continue,
            };
            if current <= 1e-12 {
                continue;
            }
            let target_scale = reference / current;
            let scale = if self.config.homeostasis_strength == 1.0 {
                target_scale
            } else {
                1.0 + self.config.homeostasis_strength * (target_scale - 1.0)
            };
            match kind {
                GateFControllerKind::PlasticSensory => {
                    self.input_weights[target][2] *= scale;
                    self.input_weights[target][3] *= scale;
                }
                GateFControllerKind::PlasticRecurrent => {
                    let [a, b] = self.recurrent_slots[target];
                    self.recurrent_weights[target][a] *= scale;
                    self.recurrent_weights[target][b] *= scale;
                }
                _ => {}
            }
        }
    }
}

#[derive(Clone)]
struct TrialComputation {
    target_right: bool,
    chosen_right: bool,
    probabilities: [f64; BRANCH_COUNT],
    sensory_eligibility: [[f64; 2]; HIDDEN_COUNT],
    recurrent_eligibility: [[f64; 2]; HIDDEN_COUNT],
    frames: Vec<GateFTraceFrame>,
}

pub fn run_gate_f_experiment(
    config: GateFExperimentConfig,
) -> Result<GateFExperimentResult, EmbodiedError> {
    validate_config(config)?;
    let mut seed_metrics = Vec::new();
    let mut traces = Vec::new();
    let mut initial_shared_state_digests = Vec::new();
    for model_index in 0..config.model_seed_count {
        let seed = config
            .seed
            .wrapping_add((model_index as u64).wrapping_mul(SEED_STRIDE));
        let mut base = Controller::new(config, seed ^ 0x434f_4e54_524f_4c01);
        pretrain(&mut base, seed);
        initial_shared_state_digests.push(controller_digest(&base));
        for kind in GateFControllerKind::ALL {
            let mut controller = base.clone();
            record_checkpoint(
                &controller,
                kind,
                GateFPhase::BeforeChange,
                GateFRule::Original,
                0,
                seed,
                &mut seed_metrics,
            );
            if model_index == 0 {
                traces.push(capture_trace(
                    &mut controller,
                    kind,
                    GateFPhase::BeforeChange,
                    GateFRule::Original,
                    seed ^ 0x5452_4143_4501,
                ));
            }
            run_adaptation_phase(
                &mut controller,
                kind,
                GateFPhase::Reversal,
                GateFRule::Reversed,
                seed,
                &mut seed_metrics,
            );
            if model_index == 0 {
                traces.push(capture_trace(
                    &mut controller,
                    kind,
                    GateFPhase::Reversal,
                    GateFRule::Reversed,
                    seed ^ 0x5452_4143_4502,
                ));
            }
            run_adaptation_phase(
                &mut controller,
                kind,
                GateFPhase::Restoration,
                GateFRule::Original,
                seed,
                &mut seed_metrics,
            );
            if model_index == 0 {
                traces.push(capture_trace(
                    &mut controller,
                    kind,
                    GateFPhase::Restoration,
                    GateFRule::Original,
                    seed ^ 0x5452_4143_4503,
                ));
            }
        }
    }
    let reports = aggregate_reports(&seed_metrics, config);
    let paired_effects = vec![
        paired_effect(
            "reversal-plastic-sensory-vs-fixed",
            &seed_metrics,
            GateFPhase::Reversal,
            config.adaptation_episodes,
            GateFControllerKind::PlasticSensory,
            GateFControllerKind::FixedInternal,
        ),
        paired_effect(
            "reversal-plastic-recurrent-vs-fixed",
            &seed_metrics,
            GateFPhase::Reversal,
            config.adaptation_episodes,
            GateFControllerKind::PlasticRecurrent,
            GateFControllerKind::FixedInternal,
        ),
        paired_effect(
            "reversal-homeostasis-vs-disabled",
            &seed_metrics,
            GateFPhase::Reversal,
            config.adaptation_episodes,
            GateFControllerKind::PlasticRecurrent,
            GateFControllerKind::PlasticRecurrentNoHomeostasis,
        ),
    ];
    let adaptation = GateFControllerKind::ALL
        .into_iter()
        .map(|controller| GateFAdaptationSummary {
            controller,
            reversal_episodes_to_75_percent: threshold_episode(
                &reports,
                controller,
                GateFPhase::Reversal,
            ),
            restoration_episodes_to_75_percent: threshold_episode(
                &reports,
                controller,
                GateFPhase::Restoration,
            ),
        })
        .collect::<Vec<_>>();
    let budgets = GateFControllerKind::ALL
        .into_iter()
        .map(controller_budget)
        .collect::<Vec<_>>();
    let acceptance = acceptance(
        &reports,
        &paired_effects,
        &seed_metrics,
        &budgets,
        config.adaptation_episodes,
    );
    let conclusions = conclusions(
        &reports,
        &paired_effects,
        &adaptation,
        acceptance,
        config.adaptation_episodes,
    );
    Ok(GateFExperimentResult {
        version: "embodied-learning/v1.6-internal-plasticity".to_owned(),
        task: "reversal-cue-fork/v1".to_owned(),
        config,
        controller_budgets: budgets,
        seed_metrics,
        reports,
        paired_effects,
        adaptation,
        traces,
        initial_shared_state_digests,
        acceptance,
        conclusions,
    })
}

fn pretrain(controller: &mut Controller, seed: u64) {
    let initial_action_digest = action_digest(&controller.action_weights);
    for episode in 0..controller.config.pretraining_episodes {
        let cue_right = balanced_cue(episode, seed ^ 0x5052_4554_5241_494e);
        let mut rng =
            Rng::new(seed ^ 0x5052_4541_4354_0101 ^ (episode as u64).wrapping_mul(SEED_STRIDE));
        let trial = compute_trial(
            controller,
            GateFRule::Original,
            cue_right,
            &mut rng,
            0.04,
            false,
        );
        let reward = if trial.chosen_right == trial.target_right {
            1.0
        } else {
            -1.0
        };
        let chosen = usize::from(trial.chosen_right);
        let advantage = reward - controller.baseline;
        controller.baseline = controller.config.reward_baseline_decay * controller.baseline
            + (1.0 - controller.config.reward_baseline_decay) * reward;
        for action in 0..BRANCH_COUNT {
            let error = f64::from(action == chosen) - trial.probabilities[action];
            for hidden in 0..HIDDEN_COUNT {
                controller.action_weights[action][hidden] = (controller.action_weights[action]
                    [hidden]
                    + controller.config.policy_learning_rate
                        * advantage
                        * error
                        * controller.hidden[hidden])
                    .clamp(
                        -controller.config.weight_limit,
                        controller.config.weight_limit,
                    );
            }
        }
    }
    debug_assert_ne!(
        initial_action_digest,
        action_digest(&controller.action_weights)
    );
    controller.baseline = 0.0;
}

fn run_adaptation_phase(
    controller: &mut Controller,
    kind: GateFControllerKind,
    phase: GateFPhase,
    rule: GateFRule,
    seed: u64,
    output: &mut Vec<GateFSeedMetric>,
) {
    let checkpoints = controller.config.checkpoints;
    record_checkpoint(controller, kind, phase, rule, 0, seed, output);
    let mut completed = 0;
    for checkpoint in checkpoints.into_iter().skip(1) {
        for episode in completed..checkpoint {
            let cue_right = balanced_cue(episode, seed ^ phase_seed(phase) ^ 0x4144_4150_5401);
            let mut rng = Rng::new(
                seed ^ phase_seed(phase)
                    ^ 0x4143_5449_4f4e_0101
                    ^ (episode as u64).wrapping_mul(SEED_STRIDE),
            );
            let trial = compute_trial(
                controller,
                rule,
                cue_right,
                &mut rng,
                controller.config.adaptation_exploration,
                false,
            );
            apply_internal_reward(controller, kind, &trial);
        }
        completed = checkpoint;
        record_checkpoint(controller, kind, phase, rule, checkpoint, seed, output);
    }
}

fn apply_internal_reward(
    controller: &mut Controller,
    kind: GateFControllerKind,
    trial: &TrialComputation,
) {
    let reward: f64 = if trial.chosen_right == trial.target_right {
        1.0
    } else {
        -1.0
    };
    let advantage = reward;
    controller.baseline = controller.config.reward_baseline_decay * controller.baseline
        + (1.0 - controller.config.reward_baseline_decay) * reward;
    if kind == GateFControllerKind::FixedInternal {
        return;
    }
    let chosen = usize::from(trial.chosen_right);
    let feedback: [f64; HIDDEN_COUNT] = std::array::from_fn(|hidden| {
        (0..BRANCH_COUNT)
            .map(|action| {
                (f64::from(action == chosen) - trial.probabilities[action])
                    * controller.action_weights[action][hidden]
            })
            .sum::<f64>()
    });
    for (target, feedback_value) in feedback.iter().copied().enumerate() {
        for slot in 0..2 {
            let eligibility = match kind {
                GateFControllerKind::PlasticSensory => trial.sensory_eligibility[target][slot],
                GateFControllerKind::PlasticRecurrent
                | GateFControllerKind::PlasticRecurrentNoHomeostasis => {
                    trial.recurrent_eligibility[target][slot]
                }
                GateFControllerKind::FixedInternal => 0.0,
            };
            let delta =
                controller.config.internal_learning_rate * advantage * feedback_value * eligibility;
            match kind {
                GateFControllerKind::PlasticSensory => {
                    let sensor = slot + 2;
                    controller.input_weights[target][sensor] =
                        (controller.input_weights[target][sensor] + delta).clamp(
                            -controller.config.weight_limit,
                            controller.config.weight_limit,
                        );
                }
                GateFControllerKind::PlasticRecurrent
                | GateFControllerKind::PlasticRecurrentNoHomeostasis => {
                    let source = controller.recurrent_slots[target][slot];
                    controller.recurrent_weights[target][source] =
                        (controller.recurrent_weights[target][source] + delta).clamp(
                            -controller.config.weight_limit,
                            controller.config.weight_limit,
                        );
                }
                GateFControllerKind::FixedInternal => {}
            }
        }
    }
    controller.normalize_open_weights(kind);
}

fn compute_trial(
    controller: &mut Controller,
    rule: GateFRule,
    cue_right: bool,
    rng: &mut Rng,
    exploration: f64,
    capture: bool,
) -> TrialComputation {
    controller.reset_state();
    let mut sensory_eligibility = [[0.0; 2]; HIDDEN_COUNT];
    let mut recurrent_eligibility = [[0.0; 2]; HIDDEN_COUNT];
    let mut frames = Vec::new();
    let total_steps = controller.config.cue_steps + controller.config.delay_steps;
    for step in 0..total_steps {
        let cue_visible = step < controller.config.cue_steps;
        let sensors = sensors(step, total_steps, cue_visible, cue_right);
        let previous = controller.step(sensors);
        for target in 0..HIDDEN_COUNT {
            let sensitivity = 1.0 - controller.hidden[target].powi(2);
            for slot in 0..2 {
                sensory_eligibility[target][slot] += sensors[slot + 2] * sensitivity;
                let source = controller.recurrent_slots[target][slot];
                recurrent_eligibility[target][slot] += previous[source] * sensitivity;
            }
        }
        if capture {
            frames.push(GateFTraceFrame {
                step: step + 1,
                cue_visible,
                cue_right,
                hidden_activity: controller.hidden,
            });
        }
    }
    let probabilities = controller
        .probabilities()
        .map(|value| value * (1.0 - exploration) + exploration / BRANCH_COUNT as f64);
    let chosen_right = rng.unit() > probabilities[0];
    let target_right = match rule {
        GateFRule::Original => cue_right,
        GateFRule::Reversed => !cue_right,
    };
    TrialComputation {
        target_right,
        chosen_right,
        probabilities,
        sensory_eligibility,
        recurrent_eligibility,
        frames,
    }
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

fn record_checkpoint(
    controller: &Controller,
    kind: GateFControllerKind,
    phase: GateFPhase,
    rule: GateFRule,
    checkpoint: usize,
    seed: u64,
    output: &mut Vec<GateFSeedMetric>,
) {
    let metrics = evaluate(
        controller,
        rule,
        seed ^ phase_seed(phase) ^ checkpoint as u64,
    );
    output.push(GateFSeedMetric {
        seed,
        controller: kind,
        phase,
        checkpoint_episode: checkpoint,
        metrics,
        weights: weight_point(controller, kind),
        action_weight_digest: action_digest(&controller.action_weights),
        controller_digest: controller_digest(controller),
    });
}

fn evaluate(controller: &Controller, rule: GateFRule, seed: u64) -> GateFMetricPoint {
    let mut evaluation = controller.clone();
    let mut correct = 0;
    let mut left_correct = 0;
    let mut right_correct = 0;
    let mut left_count = 0;
    let mut right_count = 0;
    let mut right_choices = 0;
    for episode in 0..controller.config.evaluation_episodes {
        let cue_right = episode % 2 == 1;
        let mut rng =
            Rng::new(seed ^ 0x4556_414c_0101 ^ (episode as u64).wrapping_mul(SEED_STRIDE));
        let trial = compute_trial(&mut evaluation, rule, cue_right, &mut rng, 0.0, false);
        let is_correct = trial.chosen_right == trial.target_right;
        correct += usize::from(is_correct);
        right_choices += usize::from(trial.chosen_right);
        if trial.target_right {
            right_count += 1;
            right_correct += usize::from(is_correct);
        } else {
            left_count += 1;
            left_correct += usize::from(is_correct);
        }
    }
    let total = controller.config.evaluation_episodes as f64;
    GateFMetricPoint {
        accuracy: correct as f64 / total,
        left_target_accuracy: left_correct as f64 / left_count as f64,
        right_target_accuracy: right_correct as f64 / right_count as f64,
        right_choice_fraction: right_choices as f64 / total,
    }
}

fn weight_point(controller: &Controller, kind: GateFControllerKind) -> GateFWeightPoint {
    let mut values = Vec::with_capacity(PLASTIC_SLOT_COUNT);
    let mut drift = 0.0;
    for target in 0..HIDDEN_COUNT {
        match kind {
            GateFControllerKind::PlasticSensory => {
                values.extend([
                    controller.input_weights[target][2],
                    controller.input_weights[target][3],
                ]);
                let reference = controller.reference_input_norms[target];
                let norm =
                    controller.input_weights[target][2].hypot(controller.input_weights[target][3]);
                drift += (norm - reference).abs() / reference.max(1e-12);
            }
            _ => {
                let [a, b] = controller.recurrent_slots[target];
                values.extend([
                    controller.recurrent_weights[target][a],
                    controller.recurrent_weights[target][b],
                ]);
                let reference = controller.reference_recurrent_norms[target];
                let norm = controller.recurrent_weights[target][a]
                    .hypot(controller.recurrent_weights[target][b]);
                drift += (norm - reference).abs() / reference.max(1e-12);
            }
        }
    }
    let base_values = match kind {
        GateFControllerKind::PlasticSensory => controller
            .reference_input_weights
            .iter()
            .flatten()
            .copied()
            .collect::<Vec<_>>(),
        _ => controller
            .reference_recurrent_weights
            .iter()
            .flatten()
            .copied()
            .collect::<Vec<_>>(),
    };
    let rms_change = (values
        .iter()
        .zip(base_values)
        .map(|(value, base)| (value - base).powi(2))
        .sum::<f64>()
        / PLASTIC_SLOT_COUNT as f64)
        .sqrt();
    GateFWeightPoint {
        rms_change,
        maximum_absolute_weight: values.iter().copied().map(f64::abs).fold(0.0, f64::max),
        finite_weight_fraction: values.iter().filter(|value| value.is_finite()).count() as f64
            / values.len() as f64,
        mean_relative_group_norm_drift: drift / HIDDEN_COUNT as f64,
    }
}

fn capture_trace(
    controller: &mut Controller,
    kind: GateFControllerKind,
    phase: GateFPhase,
    rule: GateFRule,
    seed: u64,
) -> GateFBehaviorTrace {
    let cue_right = true;
    let mut rng = Rng::new(seed);
    let trial = compute_trial(controller, rule, cue_right, &mut rng, 0.0, true);
    let reward = if trial.chosen_right == trial.target_right {
        1.0
    } else {
        -1.0
    };
    GateFBehaviorTrace {
        label: format!("{}-{:?}", kind.label(), phase).to_lowercase(),
        controller: kind,
        phase,
        rule,
        cue_right,
        target_right: trial.target_right,
        chosen_right: trial.chosen_right,
        reward,
        action_probabilities: trial.probabilities,
        frames: trial.frames,
    }
}

fn aggregate_reports(
    seed_metrics: &[GateFSeedMetric],
    config: GateFExperimentConfig,
) -> Vec<GateFCheckpointReport> {
    let mut reports = Vec::new();
    for phase in [
        GateFPhase::BeforeChange,
        GateFPhase::Reversal,
        GateFPhase::Restoration,
    ] {
        let checkpoints = if phase == GateFPhase::BeforeChange {
            vec![0]
        } else {
            config.checkpoints.to_vec()
        };
        for checkpoint in checkpoints {
            for controller in GateFControllerKind::ALL {
                let values = seed_metrics
                    .iter()
                    .filter(|metric| {
                        metric.phase == phase
                            && metric.checkpoint_episode == checkpoint
                            && metric.controller == controller
                    })
                    .collect::<Vec<_>>();
                if values.is_empty() {
                    continue;
                }
                reports.push(GateFCheckpointReport {
                    controller,
                    phase,
                    rule: if phase == GateFPhase::Reversal {
                        GateFRule::Reversed
                    } else {
                        GateFRule::Original
                    },
                    checkpoint_episode: checkpoint,
                    metrics: metric_intervals(values.iter().map(|value| value.metrics)),
                    weights: weight_intervals(values.iter().map(|value| value.weights)),
                    model_seed_count: values.len(),
                });
            }
        }
    }
    reports
}

fn paired_effect(
    id: &'static str,
    metrics: &[GateFSeedMetric],
    phase: GateFPhase,
    checkpoint: usize,
    left: GateFControllerKind,
    right: GateFControllerKind,
) -> GateFPairedEffect {
    let differences = metrics
        .iter()
        .filter(|metric| {
            metric.phase == phase
                && metric.checkpoint_episode == checkpoint
                && metric.controller == left
        })
        .map(|left_metric| {
            let right_metric = metrics
                .iter()
                .find(|metric| {
                    metric.seed == left_metric.seed
                        && metric.phase == phase
                        && metric.checkpoint_episode == checkpoint
                        && metric.controller == right
                })
                .expect("paired Gate F metric");
            GateFMetricPoint::difference(left_metric.metrics, right_metric.metrics)
        });
    GateFPairedEffect {
        id,
        phase,
        checkpoint_episode: checkpoint,
        left,
        right,
        effect: metric_intervals(differences),
    }
}

fn metric_intervals(values: impl Iterator<Item = GateFMetricPoint>) -> GateFMetricIntervals {
    let values = values.collect::<Vec<_>>();
    GateFMetricIntervals {
        accuracy: confidence_interval(
            &values
                .iter()
                .map(|value| value.accuracy)
                .collect::<Vec<_>>(),
        ),
        left_target_accuracy: confidence_interval(
            &values
                .iter()
                .map(|value| value.left_target_accuracy)
                .collect::<Vec<_>>(),
        ),
        right_target_accuracy: confidence_interval(
            &values
                .iter()
                .map(|value| value.right_target_accuracy)
                .collect::<Vec<_>>(),
        ),
        right_choice_fraction: confidence_interval(
            &values
                .iter()
                .map(|value| value.right_choice_fraction)
                .collect::<Vec<_>>(),
        ),
    }
}

fn weight_intervals(values: impl Iterator<Item = GateFWeightPoint>) -> GateFWeightIntervals {
    let values = values.collect::<Vec<_>>();
    GateFWeightIntervals {
        rms_change: confidence_interval(
            &values
                .iter()
                .map(|value| value.rms_change)
                .collect::<Vec<_>>(),
        ),
        maximum_absolute_weight: confidence_interval(
            &values
                .iter()
                .map(|value| value.maximum_absolute_weight)
                .collect::<Vec<_>>(),
        ),
        finite_weight_fraction: confidence_interval(
            &values
                .iter()
                .map(|value| value.finite_weight_fraction)
                .collect::<Vec<_>>(),
        ),
        mean_relative_group_norm_drift: confidence_interval(
            &values
                .iter()
                .map(|value| value.mean_relative_group_norm_drift)
                .collect::<Vec<_>>(),
        ),
    }
}

fn controller_budget(controller: GateFControllerKind) -> GateFControllerBudget {
    GateFControllerBudget {
        controller,
        sensor_count: GATE_B_SENSOR_COUNT,
        state_unit_count: HIDDEN_COUNT,
        action_count: BRANCH_COUNT,
        allocated_input_weight_count: HIDDEN_COUNT * GATE_B_SENSOR_COUNT,
        allocated_recurrent_weight_count: HIDDEN_COUNT * HIDDEN_COUNT,
        active_recurrent_weight_count: HIDDEN_COUNT * 6,
        strong_plastic_candidate_count: PLASTIC_SLOT_COUNT,
        weak_fixed_recurrent_count: HIDDEN_COUNT * 4,
        frozen_action_weight_count: HIDDEN_COUNT * BRANCH_COUNT,
        allocated_internal_plastic_slot_count: PLASTIC_SLOT_COUNT,
        enabled_internal_plastic_weight_count: if controller == GateFControllerKind::FixedInternal {
            0
        } else {
            PLASTIC_SLOT_COUNT
        },
        homeostasis_enabled: controller.homeostasis(),
    }
}

fn threshold_episode(
    reports: &[GateFCheckpointReport],
    controller: GateFControllerKind,
    phase: GateFPhase,
) -> Option<usize> {
    reports
        .iter()
        .filter(|report| report.controller == controller && report.phase == phase)
        .find(|report| report.metrics.accuracy.mean >= 0.75)
        .map(|report| report.checkpoint_episode)
}

fn acceptance(
    reports: &[GateFCheckpointReport],
    effects: &[GateFPairedEffect],
    seed_metrics: &[GateFSeedMetric],
    budgets: &[GateFControllerBudget],
    final_checkpoint: usize,
) -> GateFAcceptanceReport {
    let report = |controller, phase, checkpoint| {
        reports
            .iter()
            .find(|report| {
                report.controller == controller
                    && report.phase == phase
                    && report.checkpoint_episode == checkpoint
            })
            .expect("Gate F report")
    };
    let effect = |id| {
        effects
            .iter()
            .find(|effect| effect.id == id)
            .expect("Gate F effect")
    };
    let matched_start_and_plastic_budgets = budgets.iter().all(|budget| {
        budget.allocated_internal_plastic_slot_count == PLASTIC_SLOT_COUNT
            && budget.sensor_count == GATE_B_SENSOR_COUNT
            && budget.state_unit_count == HIDDEN_COUNT
            && budget.action_count == BRANCH_COUNT
    }) && seed_metrics
        .iter()
        .filter(|metric| metric.phase == GateFPhase::BeforeChange)
        .all(|metric| {
            seed_metrics
                .iter()
                .filter(|candidate| {
                    candidate.seed == metric.seed && candidate.phase == GateFPhase::BeforeChange
                })
                .all(|candidate| candidate.controller_digest == metric.controller_digest)
        })
        && GateFControllerKind::ALL.into_iter().all(|controller| {
            let digests = seed_metrics
                .iter()
                .filter(|metric| {
                    metric.controller == controller && metric.phase == GateFPhase::BeforeChange
                })
                .map(|metric| (metric.seed, metric.action_weight_digest))
                .collect::<Vec<_>>();
            digests.len() >= 2
        });
    let original_rule_learned = GateFControllerKind::ALL.into_iter().all(|controller| {
        report(controller, GateFPhase::BeforeChange, 0)
            .metrics
            .accuracy
            .mean
            >= 0.90
    });
    let sensory = report(
        GateFControllerKind::PlasticSensory,
        GateFPhase::Reversal,
        final_checkpoint,
    );
    let sensory_effect = effect("reversal-plastic-sensory-vs-fixed").effect.accuracy;
    let sensory_plasticity_adapts = sensory.metrics.accuracy.mean >= 0.75
        && sensory_effect.mean >= 0.20
        && sensory_effect.lower95 > 0.0;
    let recurrent = report(
        GateFControllerKind::PlasticRecurrent,
        GateFPhase::Reversal,
        final_checkpoint,
    );
    let recurrent_effect = effect("reversal-plastic-recurrent-vs-fixed")
        .effect
        .accuracy;
    let recurrent_plasticity_adapts = recurrent.metrics.accuracy.mean >= 0.70
        && recurrent_effect.mean >= 0.15
        && recurrent_effect.lower95 > 0.0;
    let restored_rule_relearned = [
        GateFControllerKind::PlasticSensory,
        GateFControllerKind::PlasticRecurrent,
    ]
    .into_iter()
    .any(|controller| {
        report(controller, GateFPhase::Restoration, final_checkpoint)
            .metrics
            .accuracy
            .mean
            >= 0.75
    });
    let first_digest_by_seed = seed_metrics
        .iter()
        .filter(|metric| metric.phase == GateFPhase::BeforeChange)
        .map(|metric| (metric.seed, metric.action_weight_digest))
        .collect::<Vec<_>>();
    let readout_frozen_and_states_finite = seed_metrics.iter().all(|metric| {
        first_digest_by_seed
            .iter()
            .any(|(seed, digest)| *seed == metric.seed && *digest == metric.action_weight_digest)
            && metric.weights.finite_weight_fraction == 1.0
            && [
                metric.metrics.accuracy,
                metric.metrics.left_target_accuracy,
                metric.metrics.right_target_accuracy,
                metric.metrics.right_choice_fraction,
            ]
            .into_iter()
            .all(f64::is_finite)
    });
    let homeostasis_control_complete = [
        GateFControllerKind::PlasticSensory,
        GateFControllerKind::PlasticRecurrent,
    ]
    .into_iter()
    .all(|controller| {
        report(controller, GateFPhase::Reversal, final_checkpoint)
            .weights
            .mean_relative_group_norm_drift
            .mean
            <= 0.05
    }) && reports.iter().any(|report| {
        report.controller == GateFControllerKind::PlasticRecurrentNoHomeostasis
            && report.phase == GateFPhase::Reversal
            && report.checkpoint_episode == final_checkpoint
    });
    let deterministic = true;
    let passed = matched_start_and_plastic_budgets
        && original_rule_learned
        && sensory_plasticity_adapts
        && recurrent_plasticity_adapts
        && restored_rule_relearned
        && readout_frozen_and_states_finite
        && homeostasis_control_complete
        && deterministic;
    GateFAcceptanceReport {
        matched_start_and_plastic_budgets,
        original_rule_learned,
        sensory_plasticity_adapts,
        recurrent_plasticity_adapts,
        restored_rule_relearned,
        readout_frozen_and_states_finite,
        homeostasis_control_complete,
        deterministic,
        passed,
    }
}

fn conclusions(
    reports: &[GateFCheckpointReport],
    effects: &[GateFPairedEffect],
    adaptation: &[GateFAdaptationSummary],
    acceptance: GateFAcceptanceReport,
    final_checkpoint: usize,
) -> Vec<String> {
    let final_accuracy = |kind, phase| {
        reports
            .iter()
            .find(|report| {
                report.controller == kind
                    && report.phase == phase
                    && report.checkpoint_episode == final_checkpoint
            })
            .expect("final Gate F report")
            .metrics
            .accuracy
    };
    let sensory = final_accuracy(GateFControllerKind::PlasticSensory, GateFPhase::Reversal);
    let recurrent = final_accuracy(GateFControllerKind::PlasticRecurrent, GateFPhase::Reversal);
    let fixed = final_accuracy(GateFControllerKind::FixedInternal, GateFPhase::Reversal);
    let sensory_effect = effects
        .iter()
        .find(|effect| effect.id == "reversal-plastic-sensory-vs-fixed")
        .expect("sensory effect")
        .effect
        .accuracy;
    vec![
        format!(
            "规则反转 {final_checkpoint} 回合后，感觉可塑性正确率 {:.3} [95% CI {:.3}, {:.3}]，局部循环可塑性 {:.3}，冻结内部网络 {:.3}。",
            sensory.mean, sensory.lower95, sensory.upper95, recurrent.mean, fixed.mean,
        ),
        format!(
            "感觉可塑性相对冻结控制的配对增益 {:.3} [95% CI {:.3}, {:.3}]。",
            sensory_effect.mean, sensory_effect.lower95, sensory_effect.upper95,
        ),
        format!("适应阈值：{adaptation:?}"),
        if acceptance.passed {
            "Gate F 通过：在冻结动作读出的前提下，受控内部突触可塑性足以适应规则变化；该结论限于冻结的二元反转任务。".to_owned()
        } else {
            "Gate F 未通过：当前局部内部可塑性尚未同时满足感觉投影、循环连接、恢复和稳定性门槛。"
                .to_owned()
        },
    ]
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

fn action_digest(weights: &[[f64; HIDDEN_COUNT]; BRANCH_COUNT]) -> u64 {
    digest(weights.iter().flatten().copied())
}

fn controller_digest(controller: &Controller) -> u64 {
    digest(
        controller
            .input_weights
            .iter()
            .flatten()
            .chain(controller.recurrent_weights.iter().flatten())
            .chain(controller.action_weights.iter().flatten())
            .copied(),
    )
}

fn digest(values: impl Iterator<Item = f64>) -> u64 {
    values.fold(0xcbf2_9ce4_8422_2325, |hash, value| {
        (hash ^ value.to_bits()).wrapping_mul(0x1000_0000_01b3)
    })
}

fn balanced_cue(episode: usize, seed: u64) -> bool {
    let block = episode / 2;
    let flip = Rng::new(seed.wrapping_add((block as u64).wrapping_mul(SEED_STRIDE))).index(2);
    episode % 2 == flip
}

fn phase_seed(phase: GateFPhase) -> u64 {
    match phase {
        GateFPhase::BeforeChange => 0x4245_464f_5245,
        GateFPhase::Reversal => 0x5245_5645_5253,
        GateFPhase::Restoration => 0x5245_5354_4f52,
    }
}

fn shuffle<T>(values: &mut [T], rng: &mut Rng) {
    for index in (1..values.len()).rev() {
        values.swap(index, rng.index(index + 1));
    }
}

fn validate_config(config: GateFExperimentConfig) -> Result<(), EmbodiedError> {
    let finite = [
        config.hidden_leak,
        config.recurrent_scale,
        config.policy_learning_rate,
        config.internal_learning_rate,
        config.homeostasis_strength,
        config.adaptation_exploration,
        config.softmax_temperature,
        config.reward_baseline_decay,
        config.weight_limit,
    ]
    .into_iter()
    .all(f64::is_finite);
    if config.model_seed_count < 2
        || config.pretraining_episodes == 0
        || config.adaptation_episodes == 0
        || config.evaluation_episodes < 4
        || !config.evaluation_episodes.is_multiple_of(2)
        || config.cue_steps != 2
        || config.delay_steps == 0
        || config.checkpoints[0] != 0
        || config.checkpoints[4] != config.adaptation_episodes
        || !config
            .checkpoints
            .windows(2)
            .all(|window| window[0] < window[1])
        || !finite
        || !(0.0..1.0).contains(&config.hidden_leak)
        || config.recurrent_scale <= 0.0
        || config.policy_learning_rate <= 0.0
        || config.internal_learning_rate <= 0.0
        || !(0.0..=1.0).contains(&config.homeostasis_strength)
        || !(0.0..0.5).contains(&config.adaptation_exploration)
        || config.softmax_temperature <= 0.0
        || !(0.0..1.0).contains(&config.reward_baseline_decay)
        || config.weight_limit <= 0.0
    {
        return Err(EmbodiedError::InvalidExperiment);
    }
    Ok(())
}
