use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

use serde::Serialize;

pub const SENSOR_COUNT: usize = 12;
pub const HIDDEN_COUNT: usize = 24;
pub const ACTION_COUNT: usize = 4;
const FEATURE_COUNT: usize = HIDDEN_COUNT;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Heading {
    North,
    East,
    South,
    West,
}

impl Heading {
    fn left(self) -> Self {
        match self {
            Self::North => Self::West,
            Self::West => Self::South,
            Self::South => Self::East,
            Self::East => Self::North,
        }
    }

    fn right(self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }

    fn forward_delta(self) -> GridPosition {
        match self {
            Self::North => GridPosition { x: 0, y: -1 },
            Self::East => GridPosition { x: 1, y: 0 },
            Self::South => GridPosition { x: 0, y: 1 },
            Self::West => GridPosition { x: -1, y: 0 },
        }
    }

    fn relative_components(self, dx: i32, dy: i32) -> (f64, f64) {
        match self {
            Self::North => (-f64::from(dy), f64::from(dx)),
            Self::East => (f64::from(dx), f64::from(dy)),
            Self::South => (f64::from(dy), -f64::from(dx)),
            Self::West => (-f64::from(dx), -f64::from(dy)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentAction {
    Forward,
    TurnLeft,
    TurnRight,
    Eat,
}

impl AgentAction {
    pub const ALL: [Self; ACTION_COUNT] =
        [Self::Forward, Self::TurnLeft, Self::TurnRight, Self::Eat];
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArenaConfig {
    pub width: i32,
    pub height: i32,
    pub food_count: usize,
    pub hazard_count: usize,
    pub maximum_steps: usize,
    pub initial_energy: f64,
    pub maximum_energy: f64,
    pub passive_cost: f64,
    pub movement_cost: f64,
    pub collision_cost: f64,
    pub hazard_cost: f64,
    pub food_energy: f64,
}

impl Default for ArenaConfig {
    fn default() -> Self {
        Self {
            width: 15,
            height: 15,
            food_count: 7,
            hazard_count: 14,
            maximum_steps: 240,
            initial_energy: 28.0,
            maximum_energy: 40.0,
            passive_cost: 0.08,
            movement_cost: 0.04,
            collision_cost: 0.40,
            hazard_cost: 1.0,
            food_energy: 11.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ControllerConfig {
    pub hidden_leak: f64,
    pub adaptation_decay: f64,
    pub adaptation_strength: f64,
    pub softmax_temperature: f64,
    pub learning_rate: f64,
    pub eligibility_decay: f64,
    pub reward_baseline_decay: f64,
    pub weight_limit: f64,
}

impl Default for ControllerConfig {
    fn default() -> Self {
        Self {
            hidden_leak: 0.72,
            adaptation_decay: 0.94,
            adaptation_strength: 0.18,
            softmax_temperature: 0.75,
            learning_rate: 0.035,
            eligibility_decay: 0.88,
            reward_baseline_decay: 0.98,
            weight_limit: 3.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbodiedExperimentConfig {
    pub seed: u64,
    pub arena: ArenaConfig,
    pub controller: ControllerConfig,
    pub training_episodes: usize,
    pub evaluation_episodes: usize,
    pub curve_window: usize,
}

impl Default for EmbodiedExperimentConfig {
    fn default() -> Self {
        Self {
            seed: 0x4e45_5552_4f4c_4946,
            arena: ArenaConfig::default(),
            controller: ControllerConfig::default(),
            training_episodes: 1_200,
            evaluation_episodes: 80,
            curve_window: 40,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BehaviorFrame {
    pub step: usize,
    pub position: GridPosition,
    pub heading: Heading,
    pub energy: f64,
    pub foods_eaten: usize,
    pub action: AgentAction,
    pub reward: f64,
    pub action_probabilities: Vec<f64>,
    pub sensors: Vec<f64>,
    pub hidden_activity: Vec<f64>,
    pub remaining_food: Vec<GridPosition>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BehaviorTrace {
    pub label: String,
    pub map_seed: u64,
    pub hazards: Vec<GridPosition>,
    pub initial_food: Vec<GridPosition>,
    pub frames: Vec<BehaviorFrame>,
    pub summary: EpisodeSummary,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpisodeSummary {
    pub steps_survived: usize,
    pub foods_eaten: usize,
    pub final_energy: f64,
    pub collisions: usize,
    pub hazard_contacts: usize,
    pub completed: bool,
    pub total_reward: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluationReport {
    pub label: String,
    pub episode_count: usize,
    pub mean_steps_survived: f64,
    pub mean_foods_eaten: f64,
    pub mean_final_energy: f64,
    pub mean_collisions: f64,
    pub mean_hazard_contacts: f64,
    pub completion_fraction: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainingCurvePoint {
    pub episode: usize,
    pub mean_foods_eaten: f64,
    pub mean_steps_survived: f64,
    pub mean_final_energy: f64,
    pub mean_reward: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlasticityReport {
    pub changed_weight_count: usize,
    pub total_weight_count: usize,
    pub root_mean_square_change: f64,
    pub maximum_absolute_change: f64,
    pub lesioned_hidden_units: Vec<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbodiedAcceptanceReport {
    pub deterministic: bool,
    pub weights_changed: bool,
    pub learned_beats_learning_disabled: bool,
    pub shuffle_hurts_performance: bool,
    pub lesion_hurts_performance: bool,
    pub passed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbodiedExperimentResult {
    pub version: String,
    pub config: EmbodiedExperimentConfig,
    pub sensor_labels: Vec<String>,
    pub action_labels: Vec<String>,
    pub training_curve: Vec<TrainingCurvePoint>,
    pub evaluations: Vec<EvaluationReport>,
    pub plasticity: PlasticityReport,
    pub traces: Vec<BehaviorTrace>,
    pub acceptance: EmbodiedAcceptanceReport,
}

#[derive(Clone, Debug)]
struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
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

#[derive(Clone, Debug)]
struct Arena {
    config: ArenaConfig,
    position: GridPosition,
    heading: Heading,
    energy: f64,
    food: BTreeSet<GridPosition>,
    initial_food: Vec<GridPosition>,
    hazards: BTreeSet<GridPosition>,
    previous_reward: f64,
    step: usize,
    foods_eaten: usize,
    collisions: usize,
    hazard_contacts: usize,
    total_reward: f64,
}

impl Arena {
    fn new(config: ArenaConfig, seed: u64) -> Result<Self, EmbodiedError> {
        validate_arena(config)?;
        let mut rng = DeterministicRng::new(seed);
        let position = GridPosition {
            x: config.width / 2,
            y: config.height / 2,
        };
        let heading = Heading::North;
        let mut occupied = BTreeSet::from([position]);
        let mut hazards = BTreeSet::new();
        while hazards.len() < config.hazard_count {
            let candidate = GridPosition {
                x: rng.index(config.width as usize) as i32,
                y: rng.index(config.height as usize) as i32,
            };
            if occupied.insert(candidate) {
                hazards.insert(candidate);
            }
        }
        let mut food = BTreeSet::new();
        while food.len() < config.food_count {
            let candidate = GridPosition {
                x: rng.index(config.width as usize) as i32,
                y: rng.index(config.height as usize) as i32,
            };
            if occupied.insert(candidate) {
                food.insert(candidate);
            }
        }
        let initial_food = food.iter().copied().collect();
        Ok(Self {
            config,
            position,
            heading,
            energy: config.initial_energy,
            food,
            initial_food,
            hazards,
            previous_reward: 0.0,
            step: 0,
            foods_eaten: 0,
            collisions: 0,
            hazard_contacts: 0,
            total_reward: 0.0,
        })
    }

    fn in_bounds(&self, position: GridPosition) -> bool {
        position.x >= 0
            && position.y >= 0
            && position.x < self.config.width
            && position.y < self.config.height
    }

    fn forward_position(&self) -> GridPosition {
        let delta = self.heading.forward_delta();
        GridPosition {
            x: self.position.x + delta.x,
            y: self.position.y + delta.y,
        }
    }

    fn nearest<'a>(
        position: GridPosition,
        candidates: impl Iterator<Item = &'a GridPosition>,
    ) -> Option<(GridPosition, i32)> {
        candidates
            .map(|candidate| {
                let distance = (candidate.x - position.x).abs() + (candidate.y - position.y).abs();
                (*candidate, distance)
            })
            .min_by_key(|(candidate, distance)| (*distance, *candidate))
    }

    fn nearest_food_distance(&self) -> Option<i32> {
        Self::nearest(self.position, self.food.iter()).map(|(_, distance)| distance)
    }

    fn directional_signal(&self, target: GridPosition) -> [f64; 4] {
        let dx = target.x - self.position.x;
        let dy = target.y - self.position.y;
        let distance = (dx.abs() + dy.abs()).max(1) as f64;
        let intensity = 1.0 / (1.0 + distance * 0.18);
        let (forward, right) = self.heading.relative_components(dx, dy);
        [
            forward.max(0.0) / distance * intensity,
            (-right).max(0.0) / distance * intensity,
            right.max(0.0) / distance * intensity,
            (-forward).max(0.0) / distance * intensity,
        ]
    }

    fn sensors(&self) -> [f64; SENSOR_COUNT] {
        let food_direction = Self::nearest(self.position, self.food.iter())
            .map(|(target, _)| self.directional_signal(target))
            .unwrap_or([0.0; 4]);
        let hazard_direction = Self::nearest(self.position, self.hazards.iter())
            .map(|(target, _)| self.directional_signal(target))
            .unwrap_or([0.0; 4]);
        let forward = self.forward_position();
        [
            1.0,
            self.energy / self.config.maximum_energy,
            f64::from(self.food.contains(&self.position)),
            food_direction[0],
            food_direction[1],
            food_direction[2],
            food_direction[3],
            f64::from(!self.in_bounds(forward)),
            hazard_direction[0],
            hazard_direction[1],
            hazard_direction[2],
            self.previous_reward.tanh(),
        ]
    }

    fn step(&mut self, action: AgentAction) -> f64 {
        let before_energy = self.energy;
        let before_distance = self.nearest_food_distance();
        self.energy -= self.config.passive_cost;
        match action {
            AgentAction::Forward => {
                self.energy -= self.config.movement_cost;
                let candidate = self.forward_position();
                if self.in_bounds(candidate) {
                    self.position = candidate;
                } else {
                    self.energy -= self.config.collision_cost;
                    self.collisions += 1;
                }
            }
            AgentAction::TurnLeft => self.heading = self.heading.left(),
            AgentAction::TurnRight => self.heading = self.heading.right(),
            AgentAction::Eat => {
                if self.food.remove(&self.position) {
                    self.energy += self.config.food_energy;
                    self.foods_eaten += 1;
                }
            }
        }
        if self.hazards.contains(&self.position) {
            self.energy -= self.config.hazard_cost;
            self.hazard_contacts += 1;
        }
        self.energy = self.energy.clamp(0.0, self.config.maximum_energy);
        self.step += 1;
        let distance_progress = match (before_distance, self.nearest_food_distance()) {
            (Some(before), Some(after)) => f64::from(before - after),
            _ => 0.0,
        };
        let reward =
            (self.energy - before_energy) / self.config.food_energy + 0.035 * distance_progress;
        self.previous_reward = reward;
        self.total_reward += reward;
        reward
    }

    fn done(&self) -> bool {
        self.energy <= 0.0 || self.food.is_empty() || self.step >= self.config.maximum_steps
    }

    fn summary(&self) -> EpisodeSummary {
        EpisodeSummary {
            steps_survived: self.step,
            foods_eaten: self.foods_eaten,
            final_energy: self.energy,
            collisions: self.collisions,
            hazard_contacts: self.hazard_contacts,
            completed: self.food.is_empty(),
            total_reward: self.total_reward,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AdaptiveController {
    config: ControllerConfig,
    input_weights: [[f64; SENSOR_COUNT]; HIDDEN_COUNT],
    policy_weights: [[f64; FEATURE_COUNT]; ACTION_COUNT],
    initial_policy_weights: [[f64; FEATURE_COUNT]; ACTION_COUNT],
    eligibility: [[f64; FEATURE_COUNT]; ACTION_COUNT],
    hidden: [f64; HIDDEN_COUNT],
    adaptation: [f64; HIDDEN_COUNT],
    last_features: [f64; FEATURE_COUNT],
    last_probabilities: [f64; ACTION_COUNT],
    reward_baseline: f64,
}

impl AdaptiveController {
    pub fn new(config: ControllerConfig, seed: u64) -> Result<Self, EmbodiedError> {
        validate_controller(config)?;
        let mut rng = DeterministicRng::new(seed);
        let mut input_weights = [[0.0; SENSOR_COUNT]; HIDDEN_COUNT];
        for row in &mut input_weights {
            for weight in row {
                *weight = rng.signed(0.65);
            }
        }
        let mut policy_weights = [[0.0; FEATURE_COUNT]; ACTION_COUNT];
        for row in &mut policy_weights {
            for weight in row {
                *weight = rng.signed(0.08);
            }
        }
        Ok(Self {
            config,
            input_weights,
            policy_weights,
            initial_policy_weights: policy_weights,
            eligibility: [[0.0; FEATURE_COUNT]; ACTION_COUNT],
            hidden: [0.0; HIDDEN_COUNT],
            adaptation: [0.0; HIDDEN_COUNT],
            last_features: [0.0; FEATURE_COUNT],
            last_probabilities: [0.0; ACTION_COUNT],
            reward_baseline: 0.0,
        })
    }

    fn reset_episode_state(&mut self) {
        self.eligibility = [[0.0; FEATURE_COUNT]; ACTION_COUNT];
        self.hidden = [0.0; HIDDEN_COUNT];
        self.adaptation = [0.0; HIDDEN_COUNT];
        self.last_features = [0.0; FEATURE_COUNT];
        self.last_probabilities = [0.0; ACTION_COUNT];
    }

    fn choose_action(
        &mut self,
        sensors: [f64; SENSOR_COUNT],
        rng: &mut DeterministicRng,
    ) -> (AgentAction, [f64; ACTION_COUNT]) {
        for hidden in 0..HIDDEN_COUNT {
            let drive = self.input_weights[hidden]
                .iter()
                .zip(sensors)
                .map(|(weight, sensor)| weight * sensor)
                .sum::<f64>()
                - self.config.adaptation_strength * self.adaptation[hidden];
            let target = drive.tanh();
            self.hidden[hidden] = self.config.hidden_leak * self.hidden[hidden]
                + (1.0 - self.config.hidden_leak) * target;
            self.adaptation[hidden] = self.config.adaptation_decay * self.adaptation[hidden]
                + (1.0 - self.config.adaptation_decay) * self.hidden[hidden].abs();
        }
        self.last_features.copy_from_slice(&self.hidden);
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
        let mut chosen = ACTION_COUNT - 1;
        for (index, probability) in probabilities.iter().enumerate() {
            cumulative += probability;
            if draw <= cumulative {
                chosen = index;
                break;
            }
        }
        (AgentAction::ALL[chosen], probabilities)
    }

    fn apply_reward(&mut self, chosen: AgentAction, reward: f64, plastic: bool) {
        let chosen_index = AgentAction::ALL
            .iter()
            .position(|candidate| *candidate == chosen)
            .expect("action belongs to canonical list");
        for action in 0..ACTION_COUNT {
            let action_error = f64::from(action == chosen_index) - self.last_probabilities[action];
            for feature in 0..FEATURE_COUNT {
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
                for feature in 0..FEATURE_COUNT {
                    self.policy_weights[action][feature] = (self.policy_weights[action][feature]
                        + self.config.learning_rate
                            * advantage
                            * self.eligibility[action][feature])
                        .clamp(-self.config.weight_limit, self.config.weight_limit);
                }
            }
        }
    }

    fn shuffled(&self, seed: u64) -> Self {
        let mut result = self.clone();
        let mut flat = result
            .policy_weights
            .iter()
            .flatten()
            .copied()
            .collect::<Vec<_>>();
        let mut rng = DeterministicRng::new(seed);
        for index in (1..flat.len()).rev() {
            flat.swap(index, rng.index(index + 1));
        }
        for (destination, source) in result.policy_weights.iter_mut().flatten().zip(flat) {
            *destination = source;
        }
        result
    }

    fn lesioned(&self) -> (Self, Vec<usize>) {
        let mut importance = (0..HIDDEN_COUNT)
            .map(|hidden| {
                let score = (0..ACTION_COUNT)
                    .map(|action| self.policy_weights[action][hidden].abs())
                    .sum::<f64>();
                (score, hidden)
            })
            .collect::<Vec<_>>();
        importance.sort_by(|left, right| {
            right
                .0
                .total_cmp(&left.0)
                .then_with(|| left.1.cmp(&right.1))
        });
        let selected = importance
            .into_iter()
            .take(HIDDEN_COUNT / 4)
            .map(|(_, hidden)| hidden)
            .collect::<Vec<_>>();
        let mut result = self.clone();
        for &hidden in &selected {
            for action in 0..ACTION_COUNT {
                result.policy_weights[action][hidden] = 0.0;
            }
        }
        (result, selected)
    }

    fn plasticity_report(&self, lesioned_hidden_units: Vec<usize>) -> PlasticityReport {
        let changes = self
            .policy_weights
            .iter()
            .flatten()
            .zip(self.initial_policy_weights.iter().flatten())
            .map(|(current, initial)| current - initial)
            .collect::<Vec<_>>();
        PlasticityReport {
            changed_weight_count: changes.iter().filter(|change| change.abs() > 1e-12).count(),
            total_weight_count: changes.len(),
            root_mean_square_change: (changes.iter().map(|change| change * change).sum::<f64>()
                / changes.len() as f64)
                .sqrt(),
            maximum_absolute_change: changes
                .iter()
                .map(|change| change.abs())
                .fold(0.0, f64::max),
            lesioned_hidden_units,
        }
    }
}

fn run_episode(
    controller: &mut AdaptiveController,
    arena_config: ArenaConfig,
    map_seed: u64,
    action_seed: u64,
    plastic: bool,
    trace_label: Option<&str>,
) -> Result<(EpisodeSummary, Option<BehaviorTrace>), EmbodiedError> {
    let mut arena = Arena::new(arena_config, map_seed)?;
    let initial_food = arena.initial_food.clone();
    let hazards = arena.hazards.iter().copied().collect::<Vec<_>>();
    let mut action_rng = DeterministicRng::new(action_seed);
    controller.reset_episode_state();
    let mut frames = Vec::new();
    while !arena.done() {
        let sensors = arena.sensors();
        let (action, probabilities) = controller.choose_action(sensors, &mut action_rng);
        let hidden_activity = controller.hidden;
        let reward = arena.step(action);
        controller.apply_reward(action, reward, plastic);
        if trace_label.is_some() {
            frames.push(BehaviorFrame {
                step: arena.step,
                position: arena.position,
                heading: arena.heading,
                energy: arena.energy,
                foods_eaten: arena.foods_eaten,
                action,
                reward,
                action_probabilities: probabilities.to_vec(),
                sensors: sensors.to_vec(),
                hidden_activity: hidden_activity.to_vec(),
                remaining_food: arena.food.iter().copied().collect(),
            });
        }
    }
    let summary = arena.summary();
    let trace = trace_label.map(|label| BehaviorTrace {
        label: label.to_owned(),
        map_seed,
        hazards,
        initial_food,
        frames,
        summary,
    });
    Ok((summary, trace))
}

fn evaluate_controller(
    controller: &AdaptiveController,
    arena: ArenaConfig,
    episode_count: usize,
    seed: u64,
    label: &str,
) -> Result<EvaluationReport, EmbodiedError> {
    let mut summaries = Vec::with_capacity(episode_count);
    for episode in 0..episode_count {
        let mut evaluation_controller = controller.clone();
        let map_seed = seed.wrapping_add((episode as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15));
        let action_seed = map_seed ^ 0x4143_5449_4f4e_5301;
        summaries.push(
            run_episode(
                &mut evaluation_controller,
                arena,
                map_seed,
                action_seed,
                false,
                None,
            )?
            .0,
        );
    }
    let count = summaries.len() as f64;
    Ok(EvaluationReport {
        label: label.to_owned(),
        episode_count,
        mean_steps_survived: summaries
            .iter()
            .map(|summary| summary.steps_survived as f64)
            .sum::<f64>()
            / count,
        mean_foods_eaten: summaries
            .iter()
            .map(|summary| summary.foods_eaten as f64)
            .sum::<f64>()
            / count,
        mean_final_energy: summaries
            .iter()
            .map(|summary| summary.final_energy)
            .sum::<f64>()
            / count,
        mean_collisions: summaries
            .iter()
            .map(|summary| summary.collisions as f64)
            .sum::<f64>()
            / count,
        mean_hazard_contacts: summaries
            .iter()
            .map(|summary| summary.hazard_contacts as f64)
            .sum::<f64>()
            / count,
        completion_fraction: summaries.iter().filter(|summary| summary.completed).count() as f64
            / count,
    })
}

fn mean_episode(summaries: &[EpisodeSummary], episode: usize) -> TrainingCurvePoint {
    let count = summaries.len().max(1) as f64;
    TrainingCurvePoint {
        episode,
        mean_foods_eaten: summaries
            .iter()
            .map(|summary| summary.foods_eaten as f64)
            .sum::<f64>()
            / count,
        mean_steps_survived: summaries
            .iter()
            .map(|summary| summary.steps_survived as f64)
            .sum::<f64>()
            / count,
        mean_final_energy: summaries
            .iter()
            .map(|summary| summary.final_energy)
            .sum::<f64>()
            / count,
        mean_reward: summaries
            .iter()
            .map(|summary| summary.total_reward)
            .sum::<f64>()
            / count,
    }
}

pub fn run_embodied_experiment(
    config: EmbodiedExperimentConfig,
) -> Result<EmbodiedExperimentResult, EmbodiedError> {
    validate_experiment(config)?;
    let initial = AdaptiveController::new(config.controller, config.seed ^ 0x434f_4e54_524f_4c01)?;
    let mut learned = initial.clone();
    let mut learning_disabled = initial.clone();
    let mut recent = Vec::with_capacity(config.curve_window);
    let mut training_curve = Vec::new();
    for episode in 0..config.training_episodes {
        let map_seed = config
            .seed
            .wrapping_add((episode as u64).wrapping_mul(0xd1b5_4a32_d192_ed03));
        let action_seed = map_seed ^ 0x5452_4149_4e01;
        let (summary, _) = run_episode(
            &mut learned,
            config.arena,
            map_seed,
            action_seed,
            true,
            None,
        )?;
        let _ = run_episode(
            &mut learning_disabled,
            config.arena,
            map_seed,
            action_seed,
            false,
            None,
        )?;
        recent.push(summary);
        if recent.len() > config.curve_window {
            recent.remove(0);
        }
        if (episode + 1) % config.curve_window == 0 || episode + 1 == config.training_episodes {
            training_curve.push(mean_episode(&recent, episode + 1));
        }
    }

    let shuffled = learned.shuffled(config.seed ^ 0x5348_5546_464c_4501);
    let (lesioned, lesioned_hidden_units) = learned.lesioned();
    let evaluation_seed = config.seed ^ 0x4556_414c_5541_5445;
    let evaluations = vec![
        evaluate_controller(
            &initial,
            config.arena,
            config.evaluation_episodes,
            evaluation_seed,
            "untrained",
        )?,
        evaluate_controller(
            &learning_disabled,
            config.arena,
            config.evaluation_episodes,
            evaluation_seed,
            "learning-disabled",
        )?,
        evaluate_controller(
            &learned,
            config.arena,
            config.evaluation_episodes,
            evaluation_seed,
            "learned",
        )?,
        evaluate_controller(
            &shuffled,
            config.arena,
            config.evaluation_episodes,
            evaluation_seed,
            "shuffled",
        )?,
        evaluate_controller(
            &lesioned,
            config.arena,
            config.evaluation_episodes,
            evaluation_seed,
            "lesioned",
        )?,
    ];

    let trace_seed = evaluation_seed ^ 0x5452_4143_4501;
    let mut traces = Vec::new();
    for (label, source) in [
        ("untrained", &initial),
        ("learned", &learned),
        ("shuffled", &shuffled),
        ("lesioned", &lesioned),
    ] {
        let mut trace_controller = source.clone();
        let (_, trace) = run_episode(
            &mut trace_controller,
            config.arena,
            trace_seed,
            trace_seed ^ 0x4143_5449_4f4e,
            false,
            Some(label),
        )?;
        traces.push(trace.expect("trace label produces trace"));
    }

    let plasticity = learned.plasticity_report(lesioned_hidden_units);
    let report = |label: &str| {
        evaluations
            .iter()
            .find(|report| report.label == label)
            .expect("canonical evaluation exists")
    };
    let learned_report = report("learned");
    let disabled_report = report("learning-disabled");
    let shuffled_report = report("shuffled");
    let lesioned_report = report("lesioned");
    let weights_changed = plasticity.changed_weight_count > 0;
    let learned_beats_learning_disabled = learned_report.mean_foods_eaten
        >= disabled_report.mean_foods_eaten + 0.25
        && learned_report.mean_final_energy > disabled_report.mean_final_energy;
    let shuffle_hurts_performance =
        learned_report.mean_foods_eaten >= shuffled_report.mean_foods_eaten + 0.15;
    let lesion_hurts_performance =
        learned_report.mean_foods_eaten >= lesioned_report.mean_foods_eaten + 0.05;
    let acceptance = EmbodiedAcceptanceReport {
        deterministic: true,
        weights_changed,
        learned_beats_learning_disabled,
        shuffle_hurts_performance,
        lesion_hurts_performance,
        passed: weights_changed
            && learned_beats_learning_disabled
            && shuffle_hurts_performance
            && lesion_hurts_performance,
    };

    Ok(EmbodiedExperimentResult {
        version: "embodied-learning/v1".to_owned(),
        config,
        sensor_labels: [
            "bias",
            "energy",
            "food-here",
            "food-forward",
            "food-left",
            "food-right",
            "food-behind",
            "wall-forward",
            "hazard-forward",
            "hazard-left",
            "hazard-right",
            "previous-reward",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
        action_labels: ["forward", "turn-left", "turn-right", "eat"]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        training_curve,
        evaluations,
        plasticity,
        traces,
        acceptance,
    })
}

fn validate_arena(config: ArenaConfig) -> Result<(), EmbodiedError> {
    let cells = i64::from(config.width) * i64::from(config.height);
    let finite = [
        config.initial_energy,
        config.maximum_energy,
        config.passive_cost,
        config.movement_cost,
        config.collision_cost,
        config.hazard_cost,
        config.food_energy,
    ]
    .iter()
    .all(|value| value.is_finite());
    if config.width < 5
        || config.height < 5
        || config.food_count == 0
        || config.maximum_steps == 0
        || cells <= (config.food_count + config.hazard_count + 1) as i64
        || !finite
        || config.initial_energy <= 0.0
        || config.initial_energy > config.maximum_energy
        || config.passive_cost <= 0.0
        || config.movement_cost < 0.0
        || config.collision_cost < 0.0
        || config.hazard_cost < 0.0
        || config.food_energy <= 0.0
    {
        return Err(EmbodiedError::InvalidArena);
    }
    Ok(())
}

fn validate_controller(config: ControllerConfig) -> Result<(), EmbodiedError> {
    let finite = [
        config.hidden_leak,
        config.adaptation_decay,
        config.adaptation_strength,
        config.softmax_temperature,
        config.learning_rate,
        config.eligibility_decay,
        config.reward_baseline_decay,
        config.weight_limit,
    ]
    .iter()
    .all(|value| value.is_finite());
    if !finite
        || !(0.0..1.0).contains(&config.hidden_leak)
        || !(0.0..1.0).contains(&config.adaptation_decay)
        || config.adaptation_strength < 0.0
        || config.softmax_temperature <= 0.0
        || config.learning_rate <= 0.0
        || !(0.0..1.0).contains(&config.eligibility_decay)
        || !(0.0..1.0).contains(&config.reward_baseline_decay)
        || config.weight_limit <= 0.0
    {
        return Err(EmbodiedError::InvalidController);
    }
    Ok(())
}

fn validate_experiment(config: EmbodiedExperimentConfig) -> Result<(), EmbodiedError> {
    validate_arena(config.arena)?;
    validate_controller(config.controller)?;
    if config.training_episodes == 0 || config.evaluation_episodes == 0 || config.curve_window == 0
    {
        return Err(EmbodiedError::InvalidExperiment);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmbodiedError {
    InvalidArena,
    InvalidController,
    InvalidExperiment,
}

impl Display for EmbodiedError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArena => formatter.write_str("invalid embodied arena configuration"),
            Self::InvalidController => {
                formatter.write_str("invalid adaptive controller configuration")
            }
            Self::InvalidExperiment => {
                formatter.write_str("invalid embodied experiment configuration")
            }
        }
    }
}

impl Error for EmbodiedError {}
