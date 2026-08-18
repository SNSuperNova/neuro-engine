use serde::Serialize;

use crate::{
    ArenaConfig, ControllerConfig, EmbodiedError, EmbodiedExperimentConfig, EvaluationReport,
    RewardConfig, SensorConfig, TrainingCurvePoint, run_embodied_experiment,
};

const SEED_STRIDE: u64 = 0x9e37_79b9_7f4a_7c15;
const FOOD_THRESHOLD: f64 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateAExperimentConfig {
    pub seed: u64,
    pub model_seed_count: usize,
    pub training_episodes: usize,
    pub evaluation_episodes: usize,
    pub curve_window: usize,
    pub arena: ArenaConfig,
    pub controller: ControllerConfig,
}

impl Default for GateAExperimentConfig {
    fn default() -> Self {
        Self {
            seed: 0x4741_5445_5f41_0101,
            model_seed_count: 12,
            training_episodes: 1_200,
            evaluation_episodes: 80,
            curve_window: 40,
            arena: ArenaConfig::default(),
            controller: ControllerConfig::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfidenceInterval {
    pub mean: f64,
    pub lower95: f64,
    pub upper95: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateAMetricPoint {
    pub foods_eaten: f64,
    pub final_energy: f64,
    pub completion_fraction: f64,
    pub collisions: f64,
    pub hazard_contacts: f64,
    pub total_reward: f64,
    pub energy_reward: f64,
    pub distance_reward: f64,
}

impl GateAMetricPoint {
    fn from_report(report: &EvaluationReport) -> Self {
        Self {
            foods_eaten: report.mean_foods_eaten,
            final_energy: report.mean_final_energy,
            completion_fraction: report.completion_fraction,
            collisions: report.mean_collisions,
            hazard_contacts: report.mean_hazard_contacts,
            total_reward: report.mean_reward_breakdown.total,
            energy_reward: report.mean_reward_breakdown.energy_delta,
            distance_reward: report.mean_reward_breakdown.distance_progress,
        }
    }

    fn difference(left: Self, right: Self) -> Self {
        Self {
            foods_eaten: left.foods_eaten - right.foods_eaten,
            final_energy: left.final_energy - right.final_energy,
            completion_fraction: left.completion_fraction - right.completion_fraction,
            collisions: left.collisions - right.collisions,
            hazard_contacts: left.hazard_contacts - right.hazard_contacts,
            total_reward: left.total_reward - right.total_reward,
            energy_reward: left.energy_reward - right.energy_reward,
            distance_reward: left.distance_reward - right.distance_reward,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateAMetricIntervals {
    pub foods_eaten: ConfidenceInterval,
    pub final_energy: ConfidenceInterval,
    pub completion_fraction: ConfidenceInterval,
    pub collisions: ConfidenceInterval,
    pub hazard_contacts: ConfidenceInterval,
    pub total_reward: ConfidenceInterval,
    pub energy_reward: ConfidenceInterval,
    pub distance_reward: ConfidenceInterval,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateASeedRun {
    pub seed: u64,
    pub learned: GateAMetricPoint,
    pub learning_disabled: GateAMetricPoint,
    pub episodes_to_threshold: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateACurvePoint {
    pub episode: usize,
    pub foods_eaten: ConfidenceInterval,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateAVariantReport {
    pub id: String,
    pub label: String,
    pub reward: RewardConfig,
    pub sensors: SensorConfig,
    pub learned: GateAMetricIntervals,
    pub learning_disabled: GateAMetricIntervals,
    pub paired_effect: GateAMetricIntervals,
    pub sample_efficiency: GateASampleEfficiency,
    pub training_curve: Vec<GateACurvePoint>,
    pub seed_runs: Vec<GateASeedRun>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateASampleEfficiency {
    pub food_threshold: f64,
    pub reached_seed_count: usize,
    pub reached_seed_fraction: f64,
    pub mean_episodes_when_reached: Option<ConfidenceInterval>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateAAcceptanceReport {
    pub canonical_variants_complete: bool,
    pub multiple_model_seeds: bool,
    pub reward_components_recorded: bool,
    pub confidence_intervals_reported: bool,
    pub no_shaping_beats_learning_disabled: bool,
    pub passed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GateAExperimentResult {
    pub version: String,
    pub config: GateAExperimentConfig,
    pub variants: Vec<GateAVariantReport>,
    pub conclusions: Vec<String>,
    pub acceptance: GateAAcceptanceReport,
}

#[derive(Clone, Copy)]
struct VariantDefinition {
    id: &'static str,
    label: &'static str,
    reward: RewardConfig,
    sensors: SensorConfig,
}

pub fn run_gate_a_experiment(
    config: GateAExperimentConfig,
) -> Result<GateAExperimentResult, EmbodiedError> {
    validate_gate_a(config)?;
    let definitions = canonical_variants();
    let mut variants = Vec::with_capacity(definitions.len());
    for definition in definitions {
        variants.push(run_variant(config, definition)?);
    }

    let canonical_variants_complete = variants.len() == canonical_variants().len();
    let multiple_model_seeds = config.model_seed_count >= 8;
    let reward_components_recorded = variants.iter().all(|variant| {
        variant.seed_runs.iter().all(|run| {
            (run.learned.total_reward - run.learned.energy_reward - run.learned.distance_reward)
                .abs()
                < 1e-9
                && (run.learning_disabled.total_reward
                    - run.learning_disabled.energy_reward
                    - run.learning_disabled.distance_reward)
                    .abs()
                    < 1e-9
        })
    });
    let confidence_intervals_reported = variants.iter().all(|variant| {
        variant.learned.foods_eaten.lower95 <= variant.learned.foods_eaten.mean
            && variant.learned.foods_eaten.mean <= variant.learned.foods_eaten.upper95
            && variant.paired_effect.foods_eaten.lower95 <= variant.paired_effect.foods_eaten.mean
            && variant.paired_effect.foods_eaten.mean <= variant.paired_effect.foods_eaten.upper95
    });
    let no_shaping_beats_learning_disabled = variants
        .iter()
        .filter(|variant| variant.reward.distance_progress_weight == 0.0)
        .any(|variant| {
            variant.paired_effect.foods_eaten.lower95 > 0.0
                && variant.paired_effect.foods_eaten.mean >= 0.25
        });
    let passed = canonical_variants_complete
        && multiple_model_seeds
        && reward_components_recorded
        && confidence_intervals_reported
        && no_shaping_beats_learning_disabled;
    let acceptance = GateAAcceptanceReport {
        canonical_variants_complete,
        multiple_model_seeds,
        reward_components_recorded,
        confidence_intervals_reported,
        no_shaping_beats_learning_disabled,
        passed,
    };

    Ok(GateAExperimentResult {
        version: "embodied-learning/v1.1-reward-audit".to_owned(),
        config,
        conclusions: conclusions(&variants, acceptance),
        variants,
        acceptance,
    })
}

fn canonical_variants() -> Vec<VariantDefinition> {
    let reward = RewardConfig::default();
    let sensors = SensorConfig::default();
    vec![
        VariantDefinition {
            id: "baseline",
            label: "原始辅助",
            reward,
            sensors,
        },
        VariantDefinition {
            id: "shaping-0.020",
            label: "距离塑形 0.020",
            reward: RewardConfig {
                distance_progress_weight: 0.020,
                ..reward
            },
            sensors,
        },
        VariantDefinition {
            id: "shaping-0.010",
            label: "距离塑形 0.010",
            reward: RewardConfig {
                distance_progress_weight: 0.010,
                ..reward
            },
            sensors,
        },
        VariantDefinition {
            id: "no-distance-shaping",
            label: "无距离塑形",
            reward: RewardConfig {
                distance_progress_weight: 0.0,
                ..reward
            },
            sensors,
        },
        VariantDefinition {
            id: "direction-precision-0.50",
            label: "方向精度 50%",
            reward,
            sensors: SensorConfig {
                food_direction_precision: 0.50,
                ..sensors
            },
        },
        VariantDefinition {
            id: "direction-noise-0.15",
            label: "方向噪声 0.15",
            reward,
            sensors: SensorConfig {
                food_direction_noise: 0.15,
                ..sensors
            },
        },
        VariantDefinition {
            id: "direction-dropout-0.35",
            label: "方向遮挡 35%",
            reward,
            sensors: SensorConfig {
                food_direction_dropout: 0.35,
                ..sensors
            },
        },
        VariantDefinition {
            id: "no-food-direction",
            label: "无食物方向",
            reward,
            sensors: SensorConfig {
                food_direction_enabled: false,
                ..sensors
            },
        },
        VariantDefinition {
            id: "no-shaping-dropout-0.35",
            label: "无塑形 + 遮挡 35%",
            reward: RewardConfig {
                distance_progress_weight: 0.0,
                ..reward
            },
            sensors: SensorConfig {
                food_direction_dropout: 0.35,
                ..sensors
            },
        },
    ]
}

fn run_variant(
    config: GateAExperimentConfig,
    definition: VariantDefinition,
) -> Result<GateAVariantReport, EmbodiedError> {
    let mut seed_runs = Vec::with_capacity(config.model_seed_count);
    let mut curves: Vec<Vec<TrainingCurvePoint>> = Vec::with_capacity(config.model_seed_count);
    for index in 0..config.model_seed_count {
        let seed = config
            .seed
            .wrapping_add((index as u64).wrapping_mul(SEED_STRIDE));
        let result = run_embodied_experiment(EmbodiedExperimentConfig {
            seed,
            arena: config.arena,
            controller: config.controller,
            reward: definition.reward,
            sensors: definition.sensors,
            training_episodes: config.training_episodes,
            evaluation_episodes: config.evaluation_episodes,
            curve_window: config.curve_window,
        })?;
        let learned = result
            .evaluations
            .iter()
            .find(|report| report.label == "learned")
            .expect("embodied protocol contains learned evaluation");
        let disabled = result
            .evaluations
            .iter()
            .find(|report| report.label == "learning-disabled")
            .expect("embodied protocol contains disabled evaluation");
        seed_runs.push(GateASeedRun {
            seed,
            learned: GateAMetricPoint::from_report(learned),
            learning_disabled: GateAMetricPoint::from_report(disabled),
            episodes_to_threshold: result
                .training_curve
                .iter()
                .find(|point| point.mean_foods_eaten >= FOOD_THRESHOLD)
                .map(|point| point.episode),
        });
        curves.push(result.training_curve);
    }

    let learned_points = seed_runs.iter().map(|run| run.learned).collect::<Vec<_>>();
    let disabled_points = seed_runs
        .iter()
        .map(|run| run.learning_disabled)
        .collect::<Vec<_>>();
    let differences = seed_runs
        .iter()
        .map(|run| GateAMetricPoint::difference(run.learned, run.learning_disabled))
        .collect::<Vec<_>>();
    let reached = seed_runs
        .iter()
        .filter_map(|run| run.episodes_to_threshold.map(|episode| episode as f64))
        .collect::<Vec<_>>();
    let sample_efficiency = GateASampleEfficiency {
        food_threshold: FOOD_THRESHOLD,
        reached_seed_count: reached.len(),
        reached_seed_fraction: reached.len() as f64 / config.model_seed_count as f64,
        mean_episodes_when_reached: (!reached.is_empty()).then(|| confidence_interval(&reached)),
    };

    Ok(GateAVariantReport {
        id: definition.id.to_owned(),
        label: definition.label.to_owned(),
        reward: definition.reward,
        sensors: definition.sensors,
        learned: intervals(&learned_points),
        learning_disabled: intervals(&disabled_points),
        paired_effect: intervals(&differences),
        sample_efficiency,
        training_curve: aggregate_curves(&curves),
        seed_runs,
    })
}

fn intervals(points: &[GateAMetricPoint]) -> GateAMetricIntervals {
    let field = |select: fn(&GateAMetricPoint) -> f64| {
        confidence_interval(&points.iter().map(select).collect::<Vec<_>>())
    };
    GateAMetricIntervals {
        foods_eaten: field(|point| point.foods_eaten),
        final_energy: field(|point| point.final_energy),
        completion_fraction: field(|point| point.completion_fraction),
        collisions: field(|point| point.collisions),
        hazard_contacts: field(|point| point.hazard_contacts),
        total_reward: field(|point| point.total_reward),
        energy_reward: field(|point| point.energy_reward),
        distance_reward: field(|point| point.distance_reward),
    }
}

fn aggregate_curves(curves: &[Vec<TrainingCurvePoint>]) -> Vec<GateACurvePoint> {
    let point_count = curves.first().map_or(0, Vec::len);
    (0..point_count)
        .map(|index| GateACurvePoint {
            episode: curves[0][index].episode,
            foods_eaten: confidence_interval(
                &curves
                    .iter()
                    .map(|curve| curve[index].mean_foods_eaten)
                    .collect::<Vec<_>>(),
            ),
        })
        .collect()
}

fn confidence_interval(values: &[f64]) -> ConfidenceInterval {
    let count = values.len();
    let mean = values.iter().sum::<f64>() / count as f64;
    if count == 1 {
        return ConfidenceInterval {
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
    ConfidenceInterval {
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

fn conclusions(variants: &[GateAVariantReport], acceptance: GateAAcceptanceReport) -> Vec<String> {
    let find = |id: &str| {
        variants
            .iter()
            .find(|variant| variant.id == id)
            .expect("canonical Gate A variant exists")
    };
    let baseline = find("baseline");
    let no_shaping = find("no-distance-shaping");
    let no_direction = find("no-food-direction");
    let mut result = vec![
        format!(
            "原始辅助下学习相对关闭学习的食物增益为 {:.3}，95% CI [{:.3}, {:.3}]。",
            baseline.paired_effect.foods_eaten.mean,
            baseline.paired_effect.foods_eaten.lower95,
            baseline.paired_effect.foods_eaten.upper95,
        ),
        format!(
            "移除距离塑形后食物增益为 {:.3}，95% CI [{:.3}, {:.3}]。",
            no_shaping.paired_effect.foods_eaten.mean,
            no_shaping.paired_effect.foods_eaten.lower95,
            no_shaping.paired_effect.foods_eaten.upper95,
        ),
        format!(
            "完全移除食物方向后，学习后平均食物为 {:.3}，关闭学习为 {:.3}。",
            no_direction.learned.foods_eaten.mean, no_direction.learning_disabled.foods_eaten.mean,
        ),
    ];
    result.push(if acceptance.no_shaping_beats_learning_disabled {
        "稀疏能量后果足以在当前精确方向感觉下产生跨种子显著学习。".to_owned()
    } else {
        "当前训练预算下，移除距离塑形后没有得到跨种子显著学习；这是 Gate A 的能力边界。".to_owned()
    });
    result
}

fn validate_gate_a(config: GateAExperimentConfig) -> Result<(), EmbodiedError> {
    if config.model_seed_count < 2
        || config.training_episodes == 0
        || config.evaluation_episodes == 0
        || config.curve_window == 0
        || config.training_episodes < config.curve_window
    {
        return Err(EmbodiedError::InvalidExperiment);
    }
    Ok(())
}
