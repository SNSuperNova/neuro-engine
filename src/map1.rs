use serde::Serialize;

use crate::learnability_map::{
    DualTimescaleHomeostasis, HomeostasisMechanism, Map0Control, Map0ControlSummary, Map0Interval,
    Map0SeedResult, halton, interpolate, normalized_distance, result_is_finite, run_map_seed,
    seed_partition, summarize,
};
use crate::{EmbodiedError, Map0ExperimentConfig, Map0ParameterPoint, Map0ParameterSummary};

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Map1ExperimentConfig {
    pub seed: u64,
    pub development_config_count: usize,
    pub development_seed_count: usize,
    pub confirmation_seed_count: usize,
    pub confirmation_candidate_count: usize,
    pub recurrent_gain_range: [f64; 2],
    pub internal_learning_rate_range: [f64; 2],
    pub homeostasis_strength_range: [f64; 2],
    pub exploration_rate_range: [f64; 2],
    pub activity_target: f64,
    pub activity_ema_rate: f64,
    pub excitability_adjustment_rate: f64,
    pub weight_norm_relaxation_rate: f64,
    pub minimum_excitability_gain: f64,
    pub maximum_excitability_gain: f64,
    pub probe_protocol: Map0ExperimentConfig,
}

impl Default for Map1ExperimentConfig {
    fn default() -> Self {
        let mut probe_protocol = Map0ExperimentConfig::default();
        probe_protocol.recurrent_gain_range = [0.10, 0.45];
        probe_protocol.internal_learning_rate_range = [0.06, 0.16];
        probe_protocol.homeostasis_strength_range = [0.25, 1.0];
        probe_protocol.exploration_rate_range = [0.04, 0.18];
        Self {
            // Reuse Map 0's base seed and partition labels so a mechanism
            // comparison never changes the sampled random worlds.
            seed: 0x4d41_5030_5f30_0101,
            development_config_count: 48,
            development_seed_count: 8,
            confirmation_seed_count: 12,
            confirmation_candidate_count: 6,
            recurrent_gain_range: probe_protocol.recurrent_gain_range,
            internal_learning_rate_range: probe_protocol.internal_learning_rate_range,
            homeostasis_strength_range: probe_protocol.homeostasis_strength_range,
            exploration_rate_range: probe_protocol.exploration_rate_range,
            activity_target: 0.35,
            activity_ema_rate: 0.05,
            excitability_adjustment_rate: 0.02,
            weight_norm_relaxation_rate: 0.03,
            minimum_excitability_gain: 0.65,
            maximum_excitability_gain: 1.35,
            probe_protocol,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Map1MechanismComparison {
    pub parameter_id: usize,
    pub dual_timescale_formation_probability: Map0Interval,
    pub reference_norm_formation_probability: Map0Interval,
    pub formation_probability_change: f64,
    pub mean_probe_score_change: f64,
    pub repeated_reversal_accuracy_change: f64,
    pub perturbation_recovery_gain_change: f64,
    pub mean_weight_drift_change: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Map1AcceptanceReport {
    pub only_homeostasis_mechanism_changed: bool,
    pub development_and_confirmation_seeds_disjoint: bool,
    pub all_development_runs_complete: bool,
    pub confirmation_runs_complete: bool,
    pub reference_mechanism_controls_complete: bool,
    pub causal_controls_complete: bool,
    pub causal_intervention_detected: bool,
    pub finite_outputs: bool,
    pub stable_region_found: bool,
    pub mechanism_improves_tradeoff: bool,
    pub passed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Map1ExperimentResult {
    pub version: String,
    pub config: Map1ExperimentConfig,
    pub development_parameters: Vec<Map0ParameterPoint>,
    pub development_seed_results: Vec<Map0SeedResult>,
    pub development_summaries: Vec<Map0ParameterSummary>,
    pub confirmation_parameter_ids: Vec<usize>,
    pub confirmation_seed_results: Vec<Map0SeedResult>,
    pub confirmation_summaries: Vec<Map0ParameterSummary>,
    pub reference_norm_seed_results: Vec<Map0SeedResult>,
    pub reference_norm_summaries: Vec<Map0ParameterSummary>,
    pub mechanism_comparisons: Vec<Map1MechanismComparison>,
    pub control_seed_results: Vec<Map0SeedResult>,
    pub control_summaries: Vec<Map0ControlSummary>,
    pub stable_region_parameter_ids: Vec<usize>,
    pub acceptance: Map1AcceptanceReport,
    pub conclusions: Vec<String>,
}

pub fn run_map1_experiment(
    config: Map1ExperimentConfig,
) -> Result<Map1ExperimentResult, EmbodiedError> {
    validate_config(config)?;
    let protocol = protocol_config(config);
    let dual = HomeostasisMechanism::DualTimescale(DualTimescaleHomeostasis {
        activity_target: config.activity_target,
        activity_ema_rate: config.activity_ema_rate,
        excitability_adjustment_rate: config.excitability_adjustment_rate,
        weight_norm_relaxation_rate: config.weight_norm_relaxation_rate,
        minimum_excitability_gain: config.minimum_excitability_gain,
        maximum_excitability_gain: config.maximum_excitability_gain,
    });
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
                protocol,
                *point,
                *seed,
                Map0Control::Baseline,
                dual,
            ));
        }
    }
    let development_summaries = summarize(
        &parameters,
        &development_seed_results,
        Map0Control::Baseline,
    );
    let confirmation_parameter_ids = select_candidates(config, &development_summaries);
    let confirmation_parameters = confirmation_parameter_ids
        .iter()
        .map(|id| parameters[*id])
        .collect::<Vec<_>>();

    let mut confirmation_seed_results = Vec::new();
    let mut reference_norm_seed_results = Vec::new();
    let mut control_seed_results = Vec::new();
    for point in &confirmation_parameters {
        for seed in &confirmation_seeds {
            confirmation_seed_results.push(run_map_seed(
                protocol,
                *point,
                *seed,
                Map0Control::Baseline,
                dual,
            ));
            reference_norm_seed_results.push(run_map_seed(
                protocol,
                *point,
                *seed,
                Map0Control::ReferenceNormHomeostasis,
                HomeostasisMechanism::ReferenceNorm,
            ));
        }
        for control in Map0Control::CAUSAL_CONTROLS {
            for seed in &confirmation_seeds {
                control_seed_results.push(run_map_seed(protocol, *point, *seed, control, dual));
            }
        }
    }
    let confirmation_summaries = summarize(
        &confirmation_parameters,
        &confirmation_seed_results,
        Map0Control::Baseline,
    );
    let reference_norm_summaries = summarize(
        &confirmation_parameters,
        &reference_norm_seed_results,
        Map0Control::ReferenceNormHomeostasis,
    );
    let mechanism_comparisons =
        mechanism_comparisons(&confirmation_summaries, &reference_norm_summaries);
    let mut control_summaries = Vec::new();
    for point in &confirmation_parameters {
        let baseline = confirmation_summaries
            .iter()
            .find(|summary| summary.parameters.id == point.id)
            .expect("Map 1 confirmation baseline");
        for control in Map0Control::CAUSAL_CONTROLS {
            let summary = summarize(&[*point], &control_seed_results, control)
                .into_iter()
                .next()
                .expect("Map 1 control summary");
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
            <= -protocol.thresholds.minimum_causal_probability_drop
            || summary.mean_probe_score_change_from_baseline
                <= -protocol.thresholds.minimum_causal_probe_score_drop
    });
    let mechanism_improves_tradeoff = mechanism_comparisons.iter().any(|comparison| {
        (comparison.formation_probability_change > 0.0
            || comparison.mean_probe_score_change >= 0.05)
            && comparison.mean_weight_drift_change <= 0.0
    });
    let all_results = development_seed_results
        .iter()
        .chain(&confirmation_seed_results)
        .chain(&reference_norm_seed_results)
        .chain(&control_seed_results)
        .collect::<Vec<_>>();
    let finite_outputs = all_results.iter().all(|result| result_is_finite(result));
    let seeds_disjoint = development_seeds
        .iter()
        .all(|seed| !confirmation_seeds.contains(seed));
    let stable_region_found = stable_region_parameter_ids.len()
        >= protocol.thresholds.minimum_region_size
        && causal_intervention_detected;
    let mut acceptance = Map1AcceptanceReport {
        only_homeostasis_mechanism_changed: same_probe_protocol(config.probe_protocol, protocol),
        development_and_confirmation_seeds_disjoint: seeds_disjoint,
        all_development_runs_complete: development_seed_results.len()
            == config.development_config_count * config.development_seed_count,
        confirmation_runs_complete: confirmation_seed_results.len()
            == confirmation_parameter_ids.len() * config.confirmation_seed_count,
        reference_mechanism_controls_complete: reference_norm_seed_results.len()
            == confirmation_parameter_ids.len() * config.confirmation_seed_count,
        causal_controls_complete: control_seed_results.len()
            == confirmation_parameter_ids.len()
                * Map0Control::CAUSAL_CONTROLS.len()
                * config.confirmation_seed_count,
        causal_intervention_detected,
        finite_outputs,
        stable_region_found,
        mechanism_improves_tradeoff,
        passed: false,
    };
    acceptance.passed = acceptance.only_homeostasis_mechanism_changed
        && acceptance.development_and_confirmation_seeds_disjoint
        && acceptance.all_development_runs_complete
        && acceptance.confirmation_runs_complete
        && acceptance.reference_mechanism_controls_complete
        && acceptance.causal_controls_complete
        && acceptance.finite_outputs;
    let conclusions = conclusions(
        &development_summaries,
        &confirmation_summaries,
        &mechanism_comparisons,
        &control_summaries,
        &stable_region_parameter_ids,
    );
    Ok(Map1ExperimentResult {
        version: "learnability-map/v0.2-dual-timescale-homeostasis".to_owned(),
        config,
        development_parameters: parameters,
        development_seed_results,
        development_summaries,
        confirmation_parameter_ids,
        confirmation_seed_results,
        confirmation_summaries,
        reference_norm_seed_results,
        reference_norm_summaries,
        mechanism_comparisons,
        control_seed_results,
        control_summaries,
        stable_region_parameter_ids,
        acceptance,
        conclusions,
    })
}

pub(crate) fn protocol_config(config: Map1ExperimentConfig) -> Map0ExperimentConfig {
    Map0ExperimentConfig {
        seed: config.seed,
        development_config_count: config.development_config_count,
        development_seed_count: config.development_seed_count,
        confirmation_seed_count: config.confirmation_seed_count,
        confirmation_candidate_count: config.confirmation_candidate_count,
        recurrent_gain_range: config.recurrent_gain_range,
        internal_learning_rate_range: config.internal_learning_rate_range,
        homeostasis_strength_range: config.homeostasis_strength_range,
        exploration_rate_range: config.exploration_rate_range,
        ..config.probe_protocol
    }
}

pub(crate) fn same_probe_protocol(
    original: Map0ExperimentConfig,
    actual: Map0ExperimentConfig,
) -> bool {
    original.pretraining_episodes == actual.pretraining_episodes
        && original.adaptation_episodes == actual.adaptation_episodes
        && original.evaluation_episodes == actual.evaluation_episodes
        && original.threshold_check_interval == actual.threshold_check_interval
        && original.cue_steps == actual.cue_steps
        && original.memory_delay_steps == actual.memory_delay_steps
        && original.reward_delay_steps == actual.reward_delay_steps
        && original.eligibility_decay == actual.eligibility_decay
        && original.hidden_leak == actual.hidden_leak
        && original.policy_learning_rate == actual.policy_learning_rate
        && original.softmax_temperature == actual.softmax_temperature
        && original.reward_baseline_decay == actual.reward_baseline_decay
        && original.weight_limit == actual.weight_limit
        && original.thresholds == actual.thresholds
}

pub(crate) fn parameter_points(config: Map1ExperimentConfig) -> Vec<Map0ParameterPoint> {
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

pub(crate) fn select_candidates(
    config: Map1ExperimentConfig,
    summaries: &[Map0ParameterSummary],
) -> Vec<usize> {
    let mut ranked = summaries.iter().collect::<Vec<_>>();
    ranked.sort_by(|left, right| rank(right).total_cmp(&rank(left)));
    let Some(best) = ranked.first().copied() else {
        return Vec::new();
    };
    let protocol = protocol_config(config);
    let mut selected = vec![best.parameters.id];
    let mut neighbors = summaries.iter().collect::<Vec<_>>();
    neighbors.sort_by(|left, right| {
        normalized_distance(protocol, best.parameters, left.parameters).total_cmp(
            &normalized_distance(protocol, best.parameters, right.parameters),
        )
    });
    for neighbor in neighbors
        .into_iter()
        .filter(|summary| summary.parameters.id != best.parameters.id)
        .take(protocol.thresholds.minimum_region_size.saturating_sub(1))
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

pub(crate) fn rank(summary: &Map0ParameterSummary) -> f64 {
    summary.formation_probability.mean * 2.0 + summary.mean_probe_score
}

pub(crate) fn stable_region(
    config: Map1ExperimentConfig,
    summaries: &[Map0ParameterSummary],
) -> Vec<usize> {
    let protocol = protocol_config(config);
    let eligible = summaries
        .iter()
        .filter(|summary| {
            summary.formation_probability.mean
                >= protocol.thresholds.minimum_region_formation_probability
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
                .expect("Map 1 component summary");
            for candidate in &eligible {
                if !component.contains(&candidate.parameters.id)
                    && normalized_distance(protocol, current.parameters, candidate.parameters)
                        <= 0.55
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

fn mechanism_comparisons(
    dual: &[Map0ParameterSummary],
    reference: &[Map0ParameterSummary],
) -> Vec<Map1MechanismComparison> {
    dual.iter()
        .map(|current| {
            let old = reference
                .iter()
                .find(|summary| summary.parameters.id == current.parameters.id)
                .expect("reference norm summary");
            Map1MechanismComparison {
                parameter_id: current.parameters.id,
                dual_timescale_formation_probability: current.formation_probability,
                reference_norm_formation_probability: old.formation_probability,
                formation_probability_change: current.formation_probability.mean
                    - old.formation_probability.mean,
                mean_probe_score_change: current.mean_probe_score - old.mean_probe_score,
                repeated_reversal_accuracy_change: current.mean_repeated_reversal_accuracy
                    - old.mean_repeated_reversal_accuracy,
                perturbation_recovery_gain_change: current.mean_perturbation_recovery_gain
                    - old.mean_perturbation_recovery_gain,
                mean_weight_drift_change: current.mean_relative_weight_drift
                    - old.mean_relative_weight_drift,
            }
        })
        .collect()
}

fn conclusions(
    development: &[Map0ParameterSummary],
    confirmation: &[Map0ParameterSummary],
    comparisons: &[Map1MechanismComparison],
    controls: &[Map0ControlSummary],
    stable_region: &[usize],
) -> Vec<String> {
    let mut output = Vec::new();
    if let Some(best) = development
        .iter()
        .max_by(|left, right| rank(left).total_cmp(&rank(right)))
    {
        output.push(format!(
            "Map 1 开发扫描最佳配置 {} 的形成概率为 {:.1}%，平均探针得分为 {:.1}%。",
            best.parameters.id,
            best.formation_probability.mean * 100.0,
            best.mean_probe_score * 100.0,
        ));
    }
    if let Some(best) = confirmation
        .iter()
        .max_by(|left, right| rank(left).total_cmp(&rank(right)))
    {
        output.push(format!(
            "独立确认最佳配置 {} 的形成概率为 {:.1}%（Wilson 95% 区间 {:.1}%–{:.1}%）。",
            best.parameters.id,
            best.formation_probability.mean * 100.0,
            best.formation_probability.lower95 * 100.0,
            best.formation_probability.upper95 * 100.0,
        ));
    }
    let mean_change = |f: fn(&Map1MechanismComparison) -> f64| {
        comparisons.iter().map(f).sum::<f64>() / comparisons.len().max(1) as f64
    };
    output.push(format!(
        "相同配置和确认种子下，双时间尺度机制相对旧参考范数的平均探针得分变化为 {:+.1} 个百分点，权重漂移变化为 {:+.3}。",
        mean_change(|item| item.mean_probe_score_change) * 100.0,
        mean_change(|item| item.mean_weight_drift_change),
    ));
    if stable_region.is_empty() {
        output.push("Map 1 没有找到满足预注册条件的连续稳定区域，不能进入 Gate G。".to_owned());
    } else {
        output.push(format!(
            "Map 1 找到由 {} 个确认配置组成的候选稳定区域，可以进入 Gate G 冻结讨论。",
            stable_region.len(),
        ));
    }
    if let Some(strongest) = controls.iter().min_by(|left, right| {
        left.mean_probe_score_change_from_baseline
            .total_cmp(&right.mean_probe_score_change_from_baseline)
    }) {
        output.push(format!(
            "因果对照中 {:?} 造成最大的平均探针得分变化：{:+.1} 个百分点。",
            strongest.control,
            strongest.mean_probe_score_change_from_baseline * 100.0,
        ));
    }
    output
}

pub(crate) fn validate_config(config: Map1ExperimentConfig) -> Result<(), EmbodiedError> {
    let finite = [
        config.recurrent_gain_range[0],
        config.recurrent_gain_range[1],
        config.internal_learning_rate_range[0],
        config.internal_learning_rate_range[1],
        config.homeostasis_strength_range[0],
        config.homeostasis_strength_range[1],
        config.exploration_rate_range[0],
        config.exploration_rate_range[1],
        config.activity_target,
        config.activity_ema_rate,
        config.excitability_adjustment_rate,
        config.weight_norm_relaxation_rate,
        config.minimum_excitability_gain,
        config.maximum_excitability_gain,
    ]
    .into_iter()
    .all(f64::is_finite);
    let ordered = |range: [f64; 2]| range[0] < range[1];
    if !finite
        || config.development_config_count < 4
        || config.development_seed_count < 2
        || config.confirmation_seed_count < 2
        || config.confirmation_candidate_count
            < config.probe_protocol.thresholds.minimum_region_size
        || config.confirmation_candidate_count > config.development_config_count
        || !ordered(config.recurrent_gain_range)
        || config.recurrent_gain_range[0] <= 0.0
        || !ordered(config.internal_learning_rate_range)
        || config.internal_learning_rate_range[0] <= 0.0
        || !ordered(config.homeostasis_strength_range)
        || config.homeostasis_strength_range[0] < 0.0
        || config.homeostasis_strength_range[1] > 1.0
        || !ordered(config.exploration_rate_range)
        || config.exploration_rate_range[0] < 0.0
        || config.exploration_rate_range[1] >= 0.5
        || !(0.0..1.0).contains(&config.activity_target)
        || !(0.0..1.0).contains(&config.activity_ema_rate)
        || config.excitability_adjustment_rate <= 0.0
        || !(0.0..=1.0).contains(&config.weight_norm_relaxation_rate)
        || config.minimum_excitability_gain <= 0.0
        || config.minimum_excitability_gain >= 1.0
        || config.maximum_excitability_gain <= 1.0
        || config.minimum_excitability_gain >= config.maximum_excitability_gain
    {
        return Err(EmbodiedError::InvalidExperiment);
    }
    Ok(())
}
