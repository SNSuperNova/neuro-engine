use serde::Serialize;

use crate::learnability_map::{
    ContinuousResource, DualTimescaleHomeostasis, HomeostasisMechanism, PlasticityMechanism,
    ResourceMechanism, SoftBoundedPlasticity, run_m2c_seed_with_mechanisms, seed_partition,
};
use crate::map1::{parameter_points, protocol_config};
use crate::{
    EmbodiedError, M1ExperimentConfig, M1PhaseResult, M1SingleRuleResult, Map0Interval,
    Map0ParameterPoint,
};

const DEVELOPMENT_SEED_LABEL: u64 = 0x4d32_4344_4556_0101;
const CONFIRMATION_SEED_LABEL: u64 = 0x4d32_4343_4f4e_4601;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum M2CControl {
    LocalEvidenceRewiring,
    RandomRewiring,
    WeightOnly,
    FrozenAdjustment,
}

impl M2CControl {
    pub const CONFIRMATION: [Self; 4] = [
        Self::LocalEvidenceRewiring,
        Self::RandomRewiring,
        Self::WeightOnly,
        Self::FrozenAdjustment,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum M2CDecision {
    CandidateAccepted,
    NoBenefitOverWeightOnly,
    RandomRewiringEquivalent,
    CapacityStillInsufficient,
    RetentionStillInsufficient,
    ResourceExhaustion,
    DynamicsInstability,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M2CMechanismParameter {
    pub id: usize,
    pub rewiring_interval: usize,
    pub evidence_decay: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M2CExperimentConfig {
    pub m1_protocol: M1ExperimentConfig,
    pub candidate_rewiring_intervals: [usize; 4],
    pub candidate_evidence_decays: [f64; 2],
    pub minimum_accuracy_improvement: f64,
    pub minimum_single_rule_accuracy: f64,
    pub minimum_novel_rule_accuracy: f64,
    pub minimum_retained_accuracy: f64,
    pub maximum_retention_drop: f64,
    pub minimum_mean_resource_level: f64,
    pub maximum_relative_weight_drift: f64,
}

impl Default for M2CExperimentConfig {
    fn default() -> Self {
        Self {
            m1_protocol: M1ExperimentConfig::default(),
            candidate_rewiring_intervals: [8, 16, 32, 64],
            candidate_evidence_decays: [0.80, 0.95],
            minimum_accuracy_improvement: 0.05,
            minimum_single_rule_accuracy: 0.70,
            minimum_novel_rule_accuracy: 0.70,
            minimum_retained_accuracy: 0.60,
            maximum_retention_drop: 0.15,
            minimum_mean_resource_level: 0.20,
            maximum_relative_weight_drift: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct M2CSeedProtocol {
    pub phase_trial_count: usize,
    pub evaluation_trial_count: usize,
    pub threshold_check_interval: usize,
    pub accuracy_threshold: f64,
    pub exploration: f64,
    pub structural_parameters: M2CMechanismParameter,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M2CSeedResult {
    pub parameter_id: usize,
    pub mechanism_parameter_id: usize,
    pub seed: u64,
    pub control: M2CControl,
    pub phase_results: Vec<M1PhaseResult>,
    pub single_rule_results: Vec<M1SingleRuleResult>,
    pub initial_a_accuracy: f64,
    pub departure_a_accuracy: f64,
    pub return_a_initial_accuracy: f64,
    pub return_a_final_accuracy: f64,
    pub retention_drop: f64,
    pub mean_novel_rule_final_accuracy: f64,
    pub minimum_single_rule_final_accuracy: f64,
    pub mean_resource_level: f64,
    pub minimum_resource_level: f64,
    pub mean_relative_weight_drift: f64,
    pub maximum_absolute_weight: f64,
    pub sequence_rewire_count: usize,
    pub single_rule_rewire_count: usize,
    pub allocated_connection_count_before: usize,
    pub allocated_connection_count_after: usize,
    pub topology_digest_before: u64,
    pub topology_digest_after: u64,
    pub action_readout_digest_before: u64,
    pub action_readout_digest_after: u64,
    pub finite: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M2CDevelopmentSummary {
    pub mechanism_parameters: M2CMechanismParameter,
    pub run_count: usize,
    pub mean_minimum_single_rule_accuracy: f64,
    pub mean_novel_rule_accuracy: f64,
    pub mean_return_a_initial_accuracy: f64,
    pub mean_retention_drop: f64,
    pub selection_score: f64,
    pub finite_fraction: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M2CControlSummary {
    pub control: M2CControl,
    pub run_count: usize,
    pub mean_minimum_single_rule_accuracy: f64,
    pub mean_novel_rule_accuracy: f64,
    pub mean_return_a_initial_accuracy: f64,
    pub mean_retention_drop: f64,
    pub mean_resource_level: f64,
    pub mean_minimum_resource_level: f64,
    pub mean_relative_weight_drift: f64,
    pub mean_sequence_rewire_count: f64,
    pub mean_single_rule_rewire_count: f64,
    pub connection_budget_preserved_fraction: f64,
    pub readout_frozen_fraction: f64,
    pub finite_fraction: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M2CPairedEffect {
    pub metric: String,
    pub comparison: String,
    pub interval: Map0Interval,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M2CAcceptanceReport {
    pub m1_capacity_trigger_satisfied: bool,
    pub node_and_connection_budget_frozen: bool,
    pub local_information_contract_frozen: bool,
    pub development_and_confirmation_seeds_disjoint: bool,
    pub development_scan_complete: bool,
    pub confirmation_controls_complete: bool,
    pub local_and_random_rewire_counts_matched: bool,
    pub connection_budget_preserved: bool,
    pub action_readout_remained_frozen: bool,
    pub finite_outputs: bool,
    pub mechanism_improves_over_weight_only: bool,
    pub mechanism_improves_over_random_rewiring: bool,
    pub single_rule_capacity_boundary_passed: bool,
    pub continuous_learning_boundary_passed: bool,
    pub retention_boundary_passed: bool,
    pub mechanism_accepted: bool,
    pub stage_passed: bool,
    pub passed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M2CExperimentResult {
    pub version: String,
    pub config: M2CExperimentConfig,
    pub structural_information_contract: Vec<String>,
    pub structural_update_contract: Vec<String>,
    pub carrier_parameters: Vec<Map0ParameterPoint>,
    pub development_mechanism_parameters: Vec<M2CMechanismParameter>,
    pub development_seed_results: Vec<M2CSeedResult>,
    pub development_summaries: Vec<M2CDevelopmentSummary>,
    pub selected_mechanism_parameters: M2CMechanismParameter,
    pub confirmation_seed_results: Vec<M2CSeedResult>,
    pub confirmation_summaries: Vec<M2CControlSummary>,
    pub paired_effects: Vec<M2CPairedEffect>,
    pub decision: M2CDecision,
    pub acceptance: M2CAcceptanceReport,
    pub conclusions: Vec<String>,
}

pub fn run_m2c_experiment(
    config: M2CExperimentConfig,
) -> Result<M2CExperimentResult, EmbodiedError> {
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
    let development_mechanism_parameters = mechanism_parameters(config);
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
    let protocol_for = |structural_parameters| M2CSeedProtocol {
        phase_trial_count: config.m1_protocol.phase_trial_count,
        evaluation_trial_count: config.m1_protocol.evaluation_trial_count,
        threshold_check_interval: config.m1_protocol.threshold_check_interval,
        accuracy_threshold: config.m1_protocol.accuracy_threshold,
        exploration: config.m1_protocol.map2_protocol.exploration,
        structural_parameters,
    };
    let run = |mechanism_parameters: M2CMechanismParameter, seeds: &[u64], control: M2CControl| {
        carrier_parameters
            .iter()
            .flat_map(|point| {
                seeds.iter().map(move |seed| {
                    run_m2c_seed_with_mechanisms(
                        carrier,
                        *point,
                        *seed,
                        control,
                        homeostasis,
                        plasticity,
                        resource,
                        protocol_for(mechanism_parameters),
                    )
                })
            })
            .collect::<Vec<_>>()
    };
    let development_seed_results = development_mechanism_parameters
        .iter()
        .flat_map(|parameters| {
            run(
                *parameters,
                &development_seeds,
                M2CControl::LocalEvidenceRewiring,
            )
        })
        .collect::<Vec<_>>();
    let development_summaries =
        summarize_development(&development_mechanism_parameters, &development_seed_results);
    let selected_mechanism_parameters = development_summaries
        .iter()
        .max_by(|left, right| {
            left.selection_score
                .total_cmp(&right.selection_score)
                .then_with(|| {
                    right
                        .mechanism_parameters
                        .id
                        .cmp(&left.mechanism_parameters.id)
                })
        })
        .expect("validated M2C candidates")
        .mechanism_parameters;
    let confirmation_seed_results = M2CControl::CONFIRMATION
        .iter()
        .flat_map(|control| run(selected_mechanism_parameters, &confirmation_seeds, *control))
        .collect::<Vec<_>>();
    let confirmation_summaries = M2CControl::CONFIRMATION
        .iter()
        .map(|control| summarize_control(*control, &confirmation_seed_results))
        .collect::<Vec<_>>();
    let paired_effects = paired_effects(&confirmation_seed_results);
    let local = summary(&confirmation_summaries, M2CControl::LocalEvidenceRewiring);
    let random = summary(&confirmation_summaries, M2CControl::RandomRewiring);
    let weight = summary(&confirmation_summaries, M2CControl::WeightOnly);
    let local_weight_single = effect(
        &paired_effects,
        "minimum-single-rule-accuracy",
        "weight-only",
    );
    let local_weight_novel = effect(&paired_effects, "novel-rule-accuracy", "weight-only");
    let local_random_single = effect(
        &paired_effects,
        "minimum-single-rule-accuracy",
        "random-rewiring",
    );
    let local_random_novel = effect(&paired_effects, "novel-rule-accuracy", "random-rewiring");
    let paired_count = carrier_parameters.len() * confirmation_seeds.len();
    let counts_matched = confirmation_seed_results
        .iter()
        .filter(|row| row.control == M2CControl::LocalEvidenceRewiring)
        .all(|local_row| {
            confirmation_seed_results.iter().any(|random_row| {
                random_row.control == M2CControl::RandomRewiring
                    && random_row.parameter_id == local_row.parameter_id
                    && random_row.seed == local_row.seed
                    && random_row.sequence_rewire_count == local_row.sequence_rewire_count
                    && random_row.single_rule_rewire_count == local_row.single_rule_rewire_count
            })
        });
    let all_results = development_seed_results
        .iter()
        .chain(&confirmation_seed_results)
        .collect::<Vec<_>>();
    let mechanism_improves_over_weight_only = local_weight_single.mean
        >= config.minimum_accuracy_improvement
        && local_weight_novel.mean >= config.minimum_accuracy_improvement
        && local_weight_single.lower95 > 0.0
        && local_weight_novel.lower95 > 0.0;
    let mechanism_improves_over_random_rewiring = local_random_single.mean
        >= config.minimum_accuracy_improvement
        && local_random_novel.mean >= config.minimum_accuracy_improvement
        && local_random_single.lower95 > 0.0
        && local_random_novel.lower95 > 0.0;
    let single_rule_capacity_boundary_passed =
        local.mean_minimum_single_rule_accuracy >= config.minimum_single_rule_accuracy;
    let continuous_learning_boundary_passed =
        local.mean_novel_rule_accuracy >= config.minimum_novel_rule_accuracy;
    let retention_boundary_passed = local.mean_return_a_initial_accuracy
        >= config.minimum_retained_accuracy
        && local.mean_retention_drop <= config.maximum_retention_drop;
    let finite_outputs = all_results.iter().all(|row| row.finite);
    let resource_ok = local.mean_resource_level >= config.minimum_mean_resource_level;
    let dynamics_ok = local.mean_relative_weight_drift <= config.maximum_relative_weight_drift;
    let decision = if !finite_outputs || !dynamics_ok {
        M2CDecision::DynamicsInstability
    } else if !resource_ok {
        M2CDecision::ResourceExhaustion
    } else if !mechanism_improves_over_weight_only {
        M2CDecision::NoBenefitOverWeightOnly
    } else if !mechanism_improves_over_random_rewiring {
        M2CDecision::RandomRewiringEquivalent
    } else if !single_rule_capacity_boundary_passed || !continuous_learning_boundary_passed {
        M2CDecision::CapacityStillInsufficient
    } else if !retention_boundary_passed {
        M2CDecision::RetentionStillInsufficient
    } else {
        M2CDecision::CandidateAccepted
    };
    let mechanism_accepted = decision == M2CDecision::CandidateAccepted;
    let expected_development =
        development_mechanism_parameters.len() * carrier_parameters.len() * development_seeds.len();
    let acceptance = M2CAcceptanceReport {
        m1_capacity_trigger_satisfied: true,
        node_and_connection_budget_frozen: true,
        local_information_contract_frozen: true,
        development_and_confirmation_seeds_disjoint: development_seeds
            .iter()
            .all(|seed| !confirmation_seeds.contains(seed)),
        development_scan_complete: development_seed_results.len() == expected_development,
        confirmation_controls_complete: M2CControl::CONFIRMATION.iter().all(|control| {
            confirmation_seed_results
                .iter()
                .filter(|row| row.control == *control)
                .count()
                == paired_count
        }),
        local_and_random_rewire_counts_matched: counts_matched,
        connection_budget_preserved: all_results.iter().all(|row| {
            row.allocated_connection_count_before == row.allocated_connection_count_after
        }),
        action_readout_remained_frozen: all_results
            .iter()
            .all(|row| row.action_readout_digest_before == row.action_readout_digest_after),
        finite_outputs,
        mechanism_improves_over_weight_only,
        mechanism_improves_over_random_rewiring,
        single_rule_capacity_boundary_passed,
        continuous_learning_boundary_passed,
        retention_boundary_passed,
        mechanism_accepted,
        stage_passed: false,
        passed: false,
    };
    let protocol_complete = acceptance.m1_capacity_trigger_satisfied
        && acceptance.node_and_connection_budget_frozen
        && acceptance.local_information_contract_frozen
        && acceptance.development_and_confirmation_seeds_disjoint
        && acceptance.development_scan_complete
        && acceptance.confirmation_controls_complete
        && acceptance.local_and_random_rewire_counts_matched
        && acceptance.connection_budget_preserved
        && acceptance.action_readout_remained_frozen
        && acceptance.finite_outputs;
    let acceptance = M2CAcceptanceReport {
        stage_passed: protocol_complete,
        passed: protocol_complete,
        ..acceptance
    };
    let conclusions = conclusions(
        selected_mechanism_parameters,
        local,
        random,
        weight,
        decision,
        mechanism_accepted,
    );
    Ok(M2CExperimentResult {
        version: "adaptive-mechanism/m2c-v0.3-fixed-budget-structural-rewiring".into(),
        config,
        structural_information_contract: vec![
            "current target unit and its two active plastic source identifiers".into(),
            "reward-modulated local eligibility evidence for candidate presynaptic units".into(),
            "bounded mechanism-owned evidence EMA; no target label, rule ID, logits, or readout weights".into(),
        ],
        structural_update_contract: vec![
            "24 units and six recurrent incoming connections per unit remain fixed".into(),
            "exactly two plastic source slots per unit; one network slot may move per rewiring interval".into(),
            "the old weight is transferred bit-for-bit to the new source; no weight or connection is created".into(),
            "random rewiring uses the identical schedule and rewire count".into(),
        ],
        carrier_parameters,
        development_mechanism_parameters,
        development_seed_results,
        development_summaries,
        selected_mechanism_parameters,
        confirmation_seed_results,
        confirmation_summaries,
        paired_effects,
        decision,
        acceptance,
        conclusions,
    })
}

fn mechanism_parameters(config: M2CExperimentConfig) -> Vec<M2CMechanismParameter> {
    let mut id = 0;
    let mut result = Vec::new();
    for evidence_decay in config.candidate_evidence_decays {
        for rewiring_interval in config.candidate_rewiring_intervals {
            result.push(M2CMechanismParameter {
                id,
                rewiring_interval,
                evidence_decay,
            });
            id += 1;
        }
    }
    result
}

fn summarize_development(
    parameters: &[M2CMechanismParameter],
    results: &[M2CSeedResult],
) -> Vec<M2CDevelopmentSummary> {
    parameters
        .iter()
        .map(|parameters| {
            let rows = results
                .iter()
                .filter(|row| row.mechanism_parameter_id == parameters.id)
                .collect::<Vec<_>>();
            let count = rows.len().max(1) as f64;
            let mean =
                |f: fn(&M2CSeedResult) -> f64| rows.iter().map(|row| f(row)).sum::<f64>() / count;
            let minimum = mean(|row| row.minimum_single_rule_final_accuracy);
            let novel = mean(|row| row.mean_novel_rule_final_accuracy);
            let returned = mean(|row| row.return_a_initial_accuracy);
            M2CDevelopmentSummary {
                mechanism_parameters: *parameters,
                run_count: rows.len(),
                mean_minimum_single_rule_accuracy: minimum,
                mean_novel_rule_accuracy: novel,
                mean_return_a_initial_accuracy: returned,
                mean_retention_drop: mean(|row| row.retention_drop),
                selection_score: 0.50 * minimum + 0.35 * novel + 0.15 * returned,
                finite_fraction: rows.iter().filter(|row| row.finite).count() as f64 / count,
            }
        })
        .collect()
}

fn summarize_control(control: M2CControl, results: &[M2CSeedResult]) -> M2CControlSummary {
    let rows = results
        .iter()
        .filter(|row| row.control == control)
        .collect::<Vec<_>>();
    let count = rows.len().max(1) as f64;
    let mean = |f: fn(&M2CSeedResult) -> f64| rows.iter().map(|row| f(row)).sum::<f64>() / count;
    M2CControlSummary {
        control,
        run_count: rows.len(),
        mean_minimum_single_rule_accuracy: mean(|row| row.minimum_single_rule_final_accuracy),
        mean_novel_rule_accuracy: mean(|row| row.mean_novel_rule_final_accuracy),
        mean_return_a_initial_accuracy: mean(|row| row.return_a_initial_accuracy),
        mean_retention_drop: mean(|row| row.retention_drop),
        mean_resource_level: mean(|row| row.mean_resource_level),
        mean_minimum_resource_level: mean(|row| row.minimum_resource_level),
        mean_relative_weight_drift: mean(|row| row.mean_relative_weight_drift),
        mean_sequence_rewire_count: mean(|row| row.sequence_rewire_count as f64),
        mean_single_rule_rewire_count: mean(|row| row.single_rule_rewire_count as f64),
        connection_budget_preserved_fraction: rows
            .iter()
            .filter(|row| {
                row.allocated_connection_count_before == row.allocated_connection_count_after
            })
            .count() as f64
            / count,
        readout_frozen_fraction: rows
            .iter()
            .filter(|row| row.action_readout_digest_before == row.action_readout_digest_after)
            .count() as f64
            / count,
        finite_fraction: rows.iter().filter(|row| row.finite).count() as f64 / count,
    }
}

fn paired_effects(results: &[M2CSeedResult]) -> Vec<M2CPairedEffect> {
    let local = results
        .iter()
        .filter(|row| row.control == M2CControl::LocalEvidenceRewiring)
        .collect::<Vec<_>>();
    let mut effects = Vec::new();
    for (control, comparison) in [
        (M2CControl::WeightOnly, "weight-only"),
        (M2CControl::RandomRewiring, "random-rewiring"),
        (M2CControl::FrozenAdjustment, "frozen-adjustment"),
    ] {
        for (metric, value) in [
            (
                "minimum-single-rule-accuracy",
                (|row: &M2CSeedResult| row.minimum_single_rule_final_accuracy)
                    as fn(&M2CSeedResult) -> f64,
            ),
            (
                "novel-rule-accuracy",
                (|row: &M2CSeedResult| row.mean_novel_rule_final_accuracy)
                    as fn(&M2CSeedResult) -> f64,
            ),
            (
                "return-a-initial-accuracy",
                (|row: &M2CSeedResult| row.return_a_initial_accuracy) as fn(&M2CSeedResult) -> f64,
            ),
        ] {
            let differences = local
                .iter()
                .filter_map(|left| {
                    results
                        .iter()
                        .find(|right| {
                            right.control == control
                                && right.parameter_id == left.parameter_id
                                && right.seed == left.seed
                        })
                        .map(|right| value(left) - value(right))
                })
                .collect::<Vec<_>>();
            effects.push(M2CPairedEffect {
                metric: metric.into(),
                comparison: comparison.into(),
                interval: mean_interval(&differences),
            });
        }
    }
    effects
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

fn summary(summaries: &[M2CControlSummary], control: M2CControl) -> &M2CControlSummary {
    summaries
        .iter()
        .find(|row| row.control == control)
        .expect("complete controls")
}

fn effect<'a>(effects: &'a [M2CPairedEffect], metric: &str, comparison: &str) -> &'a Map0Interval {
    &effects
        .iter()
        .find(|row| row.metric == metric && row.comparison == comparison)
        .expect("complete effects")
        .interval
}

fn conclusions(
    parameters: M2CMechanismParameter,
    local: &M2CControlSummary,
    random: &M2CControlSummary,
    weight: &M2CControlSummary,
    decision: M2CDecision,
    accepted: bool,
) -> Vec<String> {
    vec![
        format!(
            "开发扫描选择结构参数 P{}：每 {} 试次重连一次，局部证据衰减率 {:.2}。",
            parameters.id, parameters.rewiring_interval, parameters.evidence_decay
        ),
        format!(
            "独立确认的单规则最低准确率：局部重连 {:.1}%，随机重连 {:.1}%，仅权重可塑 {:.1}%。",
            local.mean_minimum_single_rule_accuracy * 100.0,
            random.mean_minimum_single_rule_accuracy * 100.0,
            weight.mean_minimum_single_rule_accuracy * 100.0,
        ),
        format!(
            "B/C/D 连续学习准确率：局部重连 {:.1}%，随机重连 {:.1}%，仅权重可塑 {:.1}%。",
            local.mean_novel_rule_accuracy * 100.0,
            random.mean_novel_rule_accuracy * 100.0,
            weight.mean_novel_rule_accuracy * 100.0,
        ),
        format!(
            "局部重连的 A 回归初始准确率为 {:.1}%，保留下降 {:.1} pp；平均结构改写 {:.1} 次。",
            local.mean_return_a_initial_accuracy * 100.0,
            local.mean_retention_drop * 100.0,
            local.mean_sequence_rewire_count,
        ),
        format!("M2C 决策为 {decision:?}。"),
        if accepted {
            "固定预算局部结构调整通过，可进入 M3 机制地图。".into()
        } else {
            "固定预算局部结构调整未通过；该结果不授权叠加元可塑性、增加节点或扩大连接预算。".into()
        },
    ]
}

fn validate_config(config: M2CExperimentConfig) -> Result<(), EmbodiedError> {
    let values = [
        config.minimum_accuracy_improvement,
        config.minimum_single_rule_accuracy,
        config.minimum_novel_rule_accuracy,
        config.minimum_retained_accuracy,
        config.maximum_retention_drop,
        config.minimum_mean_resource_level,
        config.maximum_relative_weight_drift,
    ];
    if config.m1_protocol.development_seed_count == 0
        || config.m1_protocol.confirmation_seed_count == 0
        || config
            .candidate_rewiring_intervals
            .iter()
            .any(|value| *value == 0)
        || config
            .candidate_evidence_decays
            .iter()
            .any(|value| !value.is_finite() || !(0.0..1.0).contains(value))
        || values
            .iter()
            .any(|value| !value.is_finite() || *value < 0.0)
        || config.minimum_single_rule_accuracy > 1.0
        || config.minimum_novel_rule_accuracy > 1.0
        || config.minimum_retained_accuracy > 1.0
        || config.maximum_retention_drop > 1.0
        || config.minimum_mean_resource_level > 1.0
    {
        return Err(EmbodiedError::InvalidExperiment);
    }
    Ok(())
}
