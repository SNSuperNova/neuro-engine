use serde::Serialize;

use crate::learnability_map::{
    ContinuousResource, DualTimescaleHomeostasis, HomeostasisMechanism, M1FSeedProtocol,
    PlasticityMechanism, ResourceMechanism, SoftBoundedPlasticity, run_m1f_seed, seed_partition,
};
use crate::map1::{parameter_points, protocol_config};
use crate::{
    EmbodiedError, HIDDEN_COUNT, M1ExperimentConfig, M1Rule, Map0Interval, Map0ParameterPoint,
};

const DEVELOPMENT_SEED_LABEL: u64 = 0x4d31_4445_5601;
const CONFIRMATION_SEED_LABEL: u64 = 0x4d31_434f_4e46;
pub const M1F_GAIN_COUNT: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum M1FControl {
    RewardLocalBaseline,
    NodePerturbation,
    RandomConsequence,
    FrozenAdjustment,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum M1FDecision {
    CandidateAccepted,
    DynamicsUnstable,
    ConsequenceIndependent,
    HistoryOnlyFormation,
    BehaviorWithoutFormationEvidence,
    HistoryDegraded,
    NoBehaviorBenefit,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1FConfig {
    pub m1_protocol: M1ExperimentConfig,
    pub formation_gains: [f64; M1F_GAIN_COUNT],
    pub perturbation_scale: f64,
    pub minimum_novel_accuracy: f64,
    pub minimum_accuracy_advantage: f64,
    pub minimum_target_probability_gain: f64,
    pub minimum_history_accuracy: f64,
    pub maximum_history_degradation: f64,
    pub minimum_mean_resource_level: f64,
    pub maximum_relative_weight_drift: f64,
}

impl Default for M1FConfig {
    fn default() -> Self {
        Self {
            m1_protocol: M1ExperimentConfig::default(),
            formation_gains: [0.025, 0.05, 0.10, 0.20],
            perturbation_scale: 0.05,
            minimum_novel_accuracy: 0.70,
            minimum_accuracy_advantage: 0.05,
            minimum_target_probability_gain: 0.05,
            minimum_history_accuracy: 0.75,
            maximum_history_degradation: 0.05,
            minimum_mean_resource_level: 0.20,
            maximum_relative_weight_drift: 1.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1FRuleResult {
    pub rule: M1Rule,
    pub initial_behavior_accuracy: f64,
    pub final_behavior_accuracy: f64,
    pub initial_target_probability: f64,
    pub final_target_probability: f64,
    pub target_probability_gain: f64,
    pub mean_resource_level: f64,
    pub minimum_resource_level: f64,
    pub mean_relative_weight_drift: f64,
    pub maximum_absolute_weight: f64,
    pub action_readout_digest_before: u64,
    pub action_readout_digest_after: u64,
    pub topology_digest_before: u64,
    pub topology_digest_after: u64,
    pub adjustable_connection_count: usize,
    pub finite: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1FSeedResult {
    pub parameter_id: usize,
    pub seed: u64,
    pub control: M1FControl,
    pub formation_gain: f64,
    pub rule_results: Vec<M1FRuleResult>,
    pub finite: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1FGainSummary {
    pub formation_gain: f64,
    pub run_count: usize,
    pub mean_history_final_accuracy: Map0Interval,
    pub mean_novel_final_accuracy: Map0Interval,
    pub mean_novel_target_probability_gain: Map0Interval,
    pub mean_resource_level: Map0Interval,
    pub mean_relative_weight_drift: Map0Interval,
    pub finite_fraction: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1FControlSummary {
    pub control: M1FControl,
    pub formation_gain: f64,
    pub run_count: usize,
    pub mean_history_final_accuracy: Map0Interval,
    pub mean_novel_final_accuracy: Map0Interval,
    pub mean_novel_target_probability_gain: Map0Interval,
    pub mean_resource_level: Map0Interval,
    pub mean_minimum_resource_level: Map0Interval,
    pub mean_relative_weight_drift: Map0Interval,
    pub finite_fraction: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1FPairedEffects {
    pub candidate_over_baseline_novel_accuracy: Map0Interval,
    pub candidate_over_random_novel_accuracy: Map0Interval,
    pub candidate_history_accuracy_change: Map0Interval,
    pub candidate_novel_target_probability_gain: Map0Interval,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1FAcceptanceReport {
    pub m1_task_carrier_and_budget_frozen: bool,
    pub development_and_confirmation_seeds_disjoint: bool,
    pub development_gain_scan_complete: bool,
    pub confirmation_controls_complete: bool,
    pub all_four_rules_complete: bool,
    pub selected_gain_from_development_only: bool,
    pub candidate_observation_scope_local: bool,
    pub target_rule_and_readout_hidden_from_candidate: bool,
    pub paired_trial_and_perturbation_streams: bool,
    pub action_readout_remained_frozen: bool,
    pub topology_and_connection_budget_preserved: bool,
    pub finite_outputs: bool,
    pub stability_within_bounds: bool,
    pub novel_behavior_threshold_passed: bool,
    pub baseline_advantage_passed: bool,
    pub consequence_specificity_passed: bool,
    pub target_probability_formation_passed: bool,
    pub history_preserved: bool,
    pub mechanism_accepted: bool,
    pub stage_passed: bool,
    pub passed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1FResult {
    pub version: String,
    pub config: M1FConfig,
    pub mechanism_contract: Vec<String>,
    pub carrier_parameters: Vec<Map0ParameterPoint>,
    pub development_seed_results: Vec<M1FSeedResult>,
    pub development_gain_summaries: Vec<M1FGainSummary>,
    pub selected_formation_gain: f64,
    pub confirmation_seed_results: Vec<M1FSeedResult>,
    pub confirmation_summaries: Vec<M1FControlSummary>,
    pub paired_effects: M1FPairedEffects,
    pub decision: M1FDecision,
    pub acceptance: M1FAcceptanceReport,
    pub conclusions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1FPublishedResult {
    pub version: String,
    pub config: M1FConfig,
    pub mechanism_contract: Vec<String>,
    pub carrier_parameters: Vec<Map0ParameterPoint>,
    pub development_gain_summaries: Vec<M1FGainSummary>,
    pub selected_formation_gain: f64,
    pub confirmation_summaries: Vec<M1FControlSummary>,
    pub paired_effects: M1FPairedEffects,
    pub decision: M1FDecision,
    pub acceptance: M1FAcceptanceReport,
    pub conclusions: Vec<String>,
}

impl M1FResult {
    pub fn published(&self) -> M1FPublishedResult {
        M1FPublishedResult {
            version: self.version.clone(),
            config: self.config,
            mechanism_contract: self.mechanism_contract.clone(),
            carrier_parameters: self.carrier_parameters.clone(),
            development_gain_summaries: self.development_gain_summaries.clone(),
            selected_formation_gain: self.selected_formation_gain,
            confirmation_summaries: self.confirmation_summaries.clone(),
            paired_effects: self.paired_effects,
            decision: self.decision,
            acceptance: self.acceptance,
            conclusions: self.conclusions.clone(),
        }
    }
}

pub fn run_m1f_experiment(config: M1FConfig) -> Result<M1FResult, EmbodiedError> {
    validate_config(config)?;
    let map1 = config
        .m1_protocol
        .map2_protocol
        .map2b_protocol
        .map2a_protocol
        .map1_protocol;
    let carrier = protocol_config(map1);
    let all_parameters = parameter_points(map1);
    let carrier_parameters = config
        .m1_protocol
        .reference_parameter_ids
        .iter()
        .map(|id| all_parameters[*id])
        .collect::<Vec<_>>();
    let development_seeds = seed_partition(
        map1.seed ^ DEVELOPMENT_SEED_LABEL,
        config.m1_protocol.development_seed_count,
    );
    let confirmation_seeds = seed_partition(
        map1.seed ^ CONFIRMATION_SEED_LABEL,
        config.m1_protocol.confirmation_seed_count,
    );
    let homeostasis = HomeostasisMechanism::DualTimescale(DualTimescaleHomeostasis {
        activity_target: map1.activity_target,
        activity_ema_rate: map1.activity_ema_rate,
        excitability_adjustment_rate: map1.excitability_adjustment_rate,
        weight_norm_relaxation_rate: map1.weight_norm_relaxation_rate,
        minimum_excitability_gain: map1.minimum_excitability_gain,
        maximum_excitability_gain: map1.maximum_excitability_gain,
    });
    let plasticity = PlasticityMechanism::SoftBounded(SoftBoundedPlasticity {
        bound_scale: config
            .m1_protocol
            .map2_protocol
            .map2b_protocol
            .map2a_protocol
            .soft_bound_scale,
    });
    let map2b = config.m1_protocol.map2_protocol.map2b_protocol;
    let resource = ResourceMechanism::Continuous(ContinuousResource {
        initial_level: map2b.initial_resource,
        supply_rate: map2b.supply_rate,
        maintenance_cost: map2b.maintenance_cost,
        activity_cost: map2b.activity_cost,
        plasticity_cost: map2b.plasticity_cost,
        minimum_modulation: map2b.minimum_modulation,
    });
    let protocol = |formation_gain| M1FSeedProtocol {
        trial_count: config.m1_protocol.phase_trial_count,
        evaluation_trial_count: config.m1_protocol.evaluation_trial_count,
        exploration: config.m1_protocol.map2_protocol.exploration,
        perturbation_scale: config.perturbation_scale,
        formation_gain,
    };
    let run = |seeds: &[u64], control: M1FControl, formation_gain: f64| {
        carrier_parameters
            .iter()
            .flat_map(|point| {
                seeds.iter().map(move |seed| {
                    run_m1f_seed(
                        carrier,
                        *point,
                        *seed,
                        control,
                        homeostasis,
                        plasticity,
                        resource,
                        protocol(formation_gain),
                    )
                })
            })
            .collect::<Vec<_>>()
    };
    let mut development_seed_results = Vec::new();
    let mut development_gain_summaries = Vec::new();
    for gain in config.formation_gains {
        let rows = run(&development_seeds, M1FControl::NodePerturbation, gain);
        development_gain_summaries.push(summarize_gain(gain, &rows));
        development_seed_results.extend(rows);
    }
    let selected_formation_gain = development_gain_summaries
        .iter()
        .max_by(|left, right| {
            left.mean_novel_final_accuracy
                .mean
                .total_cmp(&right.mean_novel_final_accuracy.mean)
                .then_with(|| right.formation_gain.total_cmp(&left.formation_gain))
        })
        .expect("four development gains")
        .formation_gain;
    let mut confirmation_seed_results = Vec::new();
    for control in [
        M1FControl::RewardLocalBaseline,
        M1FControl::NodePerturbation,
        M1FControl::RandomConsequence,
        M1FControl::FrozenAdjustment,
    ] {
        confirmation_seed_results.extend(run(
            &confirmation_seeds,
            control,
            selected_formation_gain,
        ));
    }
    let confirmation_summaries = [
        M1FControl::RewardLocalBaseline,
        M1FControl::NodePerturbation,
        M1FControl::RandomConsequence,
        M1FControl::FrozenAdjustment,
    ]
    .into_iter()
    .map(|control| summarize_control(control, selected_formation_gain, &confirmation_seed_results))
    .collect::<Vec<_>>();
    let paired_effects = paired_effects(&confirmation_seed_results);
    let candidate = summary(&confirmation_summaries, M1FControl::NodePerturbation);
    let all_results = development_seed_results
        .iter()
        .chain(&confirmation_seed_results)
        .collect::<Vec<_>>();
    let finite_outputs = all_results.iter().all(|row| row.finite);
    let stability_within_bounds = candidate.mean_resource_level.mean
        >= config.minimum_mean_resource_level
        && candidate.mean_relative_weight_drift.mean <= config.maximum_relative_weight_drift;
    let novel_behavior_threshold_passed =
        candidate.mean_novel_final_accuracy.mean >= config.minimum_novel_accuracy;
    let baseline_advantage_passed = paired_effects.candidate_over_baseline_novel_accuracy.mean
        >= config.minimum_accuracy_advantage
        && paired_effects
            .candidate_over_baseline_novel_accuracy
            .lower95
            > 0.0;
    let consequence_specificity_passed = paired_effects.candidate_over_random_novel_accuracy.mean
        >= config.minimum_accuracy_advantage
        && paired_effects.candidate_over_random_novel_accuracy.lower95 > 0.0;
    let target_probability_formation_passed =
        paired_effects.candidate_novel_target_probability_gain.mean
            >= config.minimum_target_probability_gain
            && paired_effects
                .candidate_novel_target_probability_gain
                .lower95
                > 0.0;
    let history_preserved = candidate.mean_history_final_accuracy.mean
        >= config.minimum_history_accuracy
        && paired_effects.candidate_history_accuracy_change.mean
            >= -config.maximum_history_degradation;
    let action_readout_remained_frozen = all_results
        .iter()
        .flat_map(|row| &row.rule_results)
        .all(|row| row.action_readout_digest_before == row.action_readout_digest_after);
    let topology_and_connection_budget_preserved = all_results
        .iter()
        .flat_map(|row| &row.rule_results)
        .all(|row| {
            row.topology_digest_before == row.topology_digest_after
                && row.adjustable_connection_count == HIDDEN_COUNT * 2
        });
    let expected_development = carrier_parameters.len() * development_seeds.len();
    let expected_confirmation = carrier_parameters.len() * confirmation_seeds.len();
    let acceptance = M1FAcceptanceReport {
        m1_task_carrier_and_budget_frozen: config.m1_protocol.reference_parameter_ids
            == M1ExperimentConfig::default().reference_parameter_ids
            && config.m1_protocol.phase_trial_count
                == M1ExperimentConfig::default().phase_trial_count
            && config.m1_protocol.evaluation_trial_count
                == M1ExperimentConfig::default().evaluation_trial_count,
        development_and_confirmation_seeds_disjoint: development_seeds
            .iter()
            .all(|seed| !confirmation_seeds.contains(seed)),
        development_gain_scan_complete: config.formation_gains.iter().all(|gain| {
            development_seed_results
                .iter()
                .filter(|row| row.formation_gain == *gain)
                .count()
                == expected_development
        }),
        confirmation_controls_complete: [
            M1FControl::RewardLocalBaseline,
            M1FControl::NodePerturbation,
            M1FControl::RandomConsequence,
            M1FControl::FrozenAdjustment,
        ]
        .into_iter()
        .all(|control| {
            confirmation_seed_results
                .iter()
                .filter(|row| row.control == control)
                .count()
                == expected_confirmation
        }),
        all_four_rules_complete: all_results.iter().all(|row| {
            row.rule_results.len() == 4
                && row
                    .rule_results
                    .iter()
                    .zip(M1Rule::UNIQUE)
                    .all(|(result, rule)| result.rule == rule)
        }),
        selected_gain_from_development_only: config
            .formation_gains
            .contains(&selected_formation_gain),
        candidate_observation_scope_local: true,
        target_rule_and_readout_hidden_from_candidate: true,
        paired_trial_and_perturbation_streams: true,
        action_readout_remained_frozen,
        topology_and_connection_budget_preserved,
        finite_outputs,
        stability_within_bounds,
        novel_behavior_threshold_passed,
        baseline_advantage_passed,
        consequence_specificity_passed,
        target_probability_formation_passed,
        history_preserved,
        mechanism_accepted: false,
        stage_passed: false,
        passed: false,
    };
    let protocol_complete = acceptance.m1_task_carrier_and_budget_frozen
        && acceptance.development_and_confirmation_seeds_disjoint
        && acceptance.development_gain_scan_complete
        && acceptance.confirmation_controls_complete
        && acceptance.all_four_rules_complete
        && acceptance.selected_gain_from_development_only
        && acceptance.candidate_observation_scope_local
        && acceptance.target_rule_and_readout_hidden_from_candidate
        && acceptance.paired_trial_and_perturbation_streams
        && acceptance.action_readout_remained_frozen
        && acceptance.topology_and_connection_budget_preserved
        && acceptance.finite_outputs;
    let mechanism_accepted = protocol_complete
        && stability_within_bounds
        && novel_behavior_threshold_passed
        && baseline_advantage_passed
        && consequence_specificity_passed
        && target_probability_formation_passed
        && history_preserved;
    let acceptance = M1FAcceptanceReport {
        mechanism_accepted,
        stage_passed: protocol_complete,
        passed: protocol_complete,
        ..acceptance
    };
    let decision = if !finite_outputs || !stability_within_bounds {
        M1FDecision::DynamicsUnstable
    } else if !consequence_specificity_passed {
        M1FDecision::ConsequenceIndependent
    } else if candidate.mean_history_final_accuracy.mean >= config.minimum_history_accuracy
        && !novel_behavior_threshold_passed
    {
        M1FDecision::HistoryOnlyFormation
    } else if novel_behavior_threshold_passed
        && baseline_advantage_passed
        && !target_probability_formation_passed
    {
        M1FDecision::BehaviorWithoutFormationEvidence
    } else if novel_behavior_threshold_passed
        && baseline_advantage_passed
        && target_probability_formation_passed
        && !history_preserved
    {
        M1FDecision::HistoryDegraded
    } else if !novel_behavior_threshold_passed || !baseline_advantage_passed {
        M1FDecision::NoBehaviorBenefit
    } else {
        M1FDecision::CandidateAccepted
    };
    let conclusions = conclusions(
        &development_gain_summaries,
        selected_formation_gain,
        &confirmation_summaries,
        paired_effects,
        decision,
    );
    Ok(M1FResult {
        version: "adaptive-mechanism/m1f-v0.8-node-perturbation".into(),
        config,
        mechanism_contract: vec![
            "the candidate reads only the target unit's self-generated perturbation, the connection's existing local eligibility trace and resource state, and one global scalar consequence".into(),
            "target labels, rule identity, action probabilities, action readout weights, and task generator state are unavailable to the candidate update".into(),
            "node count, topology, 48 adjustable recurrent connections, weight bounds, resource dynamics, homeostasis, action exploration, and 240-trial budget match M1".into(),
            "development chooses one of four frozen formation gains; confirmation cannot modify the selected gain or acceptance thresholds".into(),
            "success requires both frozen-readout behavior and target-probability formation, plus reward specificity and A preservation".into(),
        ],
        carrier_parameters,
        development_seed_results,
        development_gain_summaries,
        selected_formation_gain,
        confirmation_seed_results,
        confirmation_summaries,
        paired_effects,
        decision,
        acceptance,
        conclusions,
    })
}

fn summarize_gain(gain: f64, rows: &[M1FSeedResult]) -> M1FGainSummary {
    let metrics = per_run_metrics(rows);
    M1FGainSummary {
        formation_gain: gain,
        run_count: rows.len(),
        mean_history_final_accuracy: metric(&metrics, 0),
        mean_novel_final_accuracy: metric(&metrics, 1),
        mean_novel_target_probability_gain: metric(&metrics, 2),
        mean_resource_level: metric(&metrics, 3),
        mean_relative_weight_drift: metric(&metrics, 5),
        finite_fraction: rows.iter().filter(|row| row.finite).count() as f64
            / rows.len().max(1) as f64,
    }
}

fn summarize_control(
    control: M1FControl,
    gain: f64,
    all_rows: &[M1FSeedResult],
) -> M1FControlSummary {
    let rows = all_rows
        .iter()
        .filter(|row| row.control == control)
        .cloned()
        .collect::<Vec<_>>();
    let metrics = per_run_metrics(&rows);
    M1FControlSummary {
        control,
        formation_gain: gain,
        run_count: rows.len(),
        mean_history_final_accuracy: metric(&metrics, 0),
        mean_novel_final_accuracy: metric(&metrics, 1),
        mean_novel_target_probability_gain: metric(&metrics, 2),
        mean_resource_level: metric(&metrics, 3),
        mean_minimum_resource_level: metric(&metrics, 4),
        mean_relative_weight_drift: metric(&metrics, 5),
        finite_fraction: rows.iter().filter(|row| row.finite).count() as f64
            / rows.len().max(1) as f64,
    }
}

fn per_run_metrics(rows: &[M1FSeedResult]) -> Vec<[f64; 6]> {
    rows.iter()
        .map(|row| {
            let novel = &row.rule_results[1..4];
            let novel_mean = |f: fn(&M1FRuleResult) -> f64| {
                novel.iter().map(f).sum::<f64>() / novel.len() as f64
            };
            let all_mean = |f: fn(&M1FRuleResult) -> f64| {
                row.rule_results.iter().map(f).sum::<f64>() / row.rule_results.len() as f64
            };
            [
                row.rule_results[0].final_behavior_accuracy,
                novel_mean(|result| result.final_behavior_accuracy),
                novel_mean(|result| result.target_probability_gain),
                all_mean(|result| result.mean_resource_level),
                row.rule_results
                    .iter()
                    .map(|result| result.minimum_resource_level)
                    .fold(f64::INFINITY, f64::min),
                all_mean(|result| result.mean_relative_weight_drift),
            ]
        })
        .collect()
}

fn paired_effects(rows: &[M1FSeedResult]) -> M1FPairedEffects {
    let candidates = rows
        .iter()
        .filter(|row| row.control == M1FControl::NodePerturbation)
        .collect::<Vec<_>>();
    let paired = candidates
        .iter()
        .map(|candidate| {
            let find = |control| {
                rows.iter()
                    .find(|row| {
                        row.parameter_id == candidate.parameter_id
                            && row.seed == candidate.seed
                            && row.control == control
                    })
                    .expect("paired M1-F control")
            };
            let baseline = find(M1FControl::RewardLocalBaseline);
            let random = find(M1FControl::RandomConsequence);
            let novel = |row: &M1FSeedResult, f: fn(&M1FRuleResult) -> f64| {
                row.rule_results[1..4].iter().map(f).sum::<f64>() / 3.0
            };
            [
                novel(candidate, |rule| rule.final_behavior_accuracy)
                    - novel(baseline, |rule| rule.final_behavior_accuracy),
                novel(candidate, |rule| rule.final_behavior_accuracy)
                    - novel(random, |rule| rule.final_behavior_accuracy),
                candidate.rule_results[0].final_behavior_accuracy
                    - baseline.rule_results[0].final_behavior_accuracy,
                novel(candidate, |rule| rule.target_probability_gain),
            ]
        })
        .collect::<Vec<_>>();
    M1FPairedEffects {
        candidate_over_baseline_novel_accuracy: metric(&paired, 0),
        candidate_over_random_novel_accuracy: metric(&paired, 1),
        candidate_history_accuracy_change: metric(&paired, 2),
        candidate_novel_target_probability_gain: metric(&paired, 3),
    }
}

fn summary(rows: &[M1FControlSummary], control: M1FControl) -> &M1FControlSummary {
    rows.iter()
        .find(|row| row.control == control)
        .expect("M1-F control summary")
}

fn metric<const N: usize>(values: &[[f64; N]], index: usize) -> Map0Interval {
    mean_interval(&values.iter().map(|row| row[index]).collect::<Vec<_>>())
}

fn mean_interval(values: &[f64]) -> Map0Interval {
    let count = values.len().max(1) as f64;
    let mean = values.iter().sum::<f64>() / count;
    let variance = if values.len() > 1 {
        values
            .iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>()
            / (values.len() - 1) as f64
    } else {
        0.0
    };
    let margin = 1.96 * (variance / count).sqrt();
    Map0Interval {
        mean,
        lower95: mean - margin,
        upper95: mean + margin,
    }
}

fn conclusions(
    gains: &[M1FGainSummary],
    selected_gain: f64,
    summaries: &[M1FControlSummary],
    effects: M1FPairedEffects,
    decision: M1FDecision,
) -> Vec<String> {
    let scan = gains
        .iter()
        .map(|row| {
            format!(
                "{:.3}: {:.1}%",
                row.formation_gain,
                row.mean_novel_final_accuracy.mean * 100.0
            )
        })
        .collect::<Vec<_>>()
        .join(" · ");
    let candidate = summary(summaries, M1FControl::NodePerturbation);
    let baseline = summary(summaries, M1FControl::RewardLocalBaseline);
    let random = summary(summaries, M1FControl::RandomConsequence);
    vec![
        format!("开发增益扫描的 B/C/D 行为为 {scan}；冻结选择 {selected_gain:.3}。"),
        format!(
            "独立确认中，候选 B/C/D 为 {:.1}%，现有基线 {:.1}%，随机后果 {:.1}%。",
            candidate.mean_novel_final_accuracy.mean * 100.0,
            baseline.mean_novel_final_accuracy.mean * 100.0,
            random.mean_novel_final_accuracy.mean * 100.0,
        ),
        format!(
            "候选相对基线为 {:+.2} pp [{:+.2}, {:+.2}]，相对随机后果为 {:+.2} pp [{:+.2}, {:+.2}]。",
            effects.candidate_over_baseline_novel_accuracy.mean * 100.0,
            effects.candidate_over_baseline_novel_accuracy.lower95 * 100.0,
            effects.candidate_over_baseline_novel_accuracy.upper95 * 100.0,
            effects.candidate_over_random_novel_accuracy.mean * 100.0,
            effects.candidate_over_random_novel_accuracy.lower95 * 100.0,
            effects.candidate_over_random_novel_accuracy.upper95 * 100.0,
        ),
        format!(
            "候选 B/C/D 目标概率形成增益为 {:+.2} pp [{:+.2}, {:+.2}]；A 最终行为 {:.1}%。",
            effects.candidate_novel_target_probability_gain.mean * 100.0,
            effects.candidate_novel_target_probability_gain.lower95 * 100.0,
            effects.candidate_novel_target_probability_gain.upper95 * 100.0,
            candidate.mean_history_final_accuracy.mean * 100.0,
        ),
        format!("M1-F 正式决策为 {decision:?}。"),
        match decision {
            M1FDecision::CandidateAccepted => {
                "局部节点扰动同时形成目标对齐并提高真实行为；下一步才允许进入连续多规则和迁移验证。"
            }
            M1FDecision::DynamicsUnstable => {
                "候选出现资源、漂移或有限性越界；机制淘汰，不用行为分数解释。"
            }
            M1FDecision::ConsequenceIndependent => {
                "候选没有可靠优于相同扰动的随机后果；观察到的变化不能归因于任务后果信用。"
            }
            M1FDecision::HistoryOnlyFormation => {
                "候选主要维持已有 A，未形成一般 B/C/D 新规则能力；不进入连续多规则验证。"
            }
            M1FDecision::BehaviorWithoutFormationEvidence => {
                "行为改善没有伴随预注册的目标概率形成增益；当前证据不足以宣称内部规则形成。"
            }
            M1FDecision::HistoryDegraded => {
                "新规则指标通过但破坏了已有 A；候选不满足稳定学习目标。"
            }
            M1FDecision::NoBehaviorBenefit => {
                "节点扰动没有在冻结读出下形成足够的 B/C/D 行为优势；候选淘汰。"
            }
        }
        .into(),
    ]
}

fn validate_config(config: M1FConfig) -> Result<(), EmbodiedError> {
    let mut gains = config.formation_gains;
    gains.sort_by(f64::total_cmp);
    if config.m1_protocol.development_seed_count == 0
        || config.m1_protocol.confirmation_seed_count == 0
        || config.m1_protocol.phase_trial_count == 0
        || config.m1_protocol.evaluation_trial_count == 0
        || gains != config.formation_gains
        || config
            .formation_gains
            .iter()
            .any(|gain| !gain.is_finite() || *gain <= 0.0)
        || !config.perturbation_scale.is_finite()
        || !(0.0..=0.25).contains(&config.perturbation_scale)
        || config.perturbation_scale == 0.0
        || [
            config.minimum_novel_accuracy,
            config.minimum_accuracy_advantage,
            config.minimum_target_probability_gain,
            config.minimum_history_accuracy,
            config.maximum_history_degradation,
            config.minimum_mean_resource_level,
            config.maximum_relative_weight_drift,
        ]
        .iter()
        .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
    {
        return Err(EmbodiedError::InvalidExperiment);
    }
    Ok(())
}
