use serde::Serialize;

use crate::learnability_map::{
    ContinuousResource, DualTimescaleHomeostasis, HomeostasisMechanism, PlasticityMechanism,
    RepresentationCapacitySeedProtocol, ResourceMechanism, SoftBoundedPlasticity,
    run_representation_capacity_seed, seed_partition,
};
use crate::map1::{parameter_points, protocol_config};
use crate::{
    EmbodiedError, HIDDEN_COUNT, M1ExperimentConfig, M1Rule, Map0Interval, Map0ParameterPoint,
};

const DEVELOPMENT_SEED_LABEL: u64 = 0x4d31_4445_5601;
const CONFIRMATION_SEED_LABEL: u64 = 0x4d31_434f_4e46;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RepresentationCapacityDecision {
    BaselineCapabilityPresent,
    CreditRoutingBottleneck,
    LearnedRepresentationReadoutBottleneck,
    LatentSymbolCodeWithoutRuleFormation,
    RepresentationInsufficient,
    InconclusiveBoundary,
    DiagnosticUnstable,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepresentationCapacityConfig {
    pub m1_protocol: M1ExperimentConfig,
    pub probe_training_trials: usize,
    pub probe_evaluation_trials: usize,
    pub probe_ridge: f64,
    pub shuffled_label_repeats: usize,
    pub behavior_accuracy_threshold: f64,
    pub hidden_probe_accuracy_threshold: f64,
    pub minimum_representation_gain: f64,
    pub minimum_target_credit_gain: f64,
    pub minimum_readout_rescue_gap: f64,
    pub maximum_shuffled_probe_accuracy: f64,
}

impl Default for RepresentationCapacityConfig {
    fn default() -> Self {
        Self {
            m1_protocol: M1ExperimentConfig::default(),
            probe_training_trials: 128,
            probe_evaluation_trials: 128,
            probe_ridge: 0.01,
            shuffled_label_repeats: 16,
            behavior_accuracy_threshold: 0.70,
            hidden_probe_accuracy_threshold: 0.75,
            minimum_representation_gain: 0.05,
            minimum_target_credit_gain: 0.05,
            minimum_readout_rescue_gap: 0.10,
            maximum_shuffled_probe_accuracy: 0.60,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepresentationRuleResult {
    pub rule: M1Rule,
    pub raw_sensor_probe_accuracy: f64,
    pub pre_behavior_accuracy: f64,
    pub pre_hidden_probe_accuracy: f64,
    pub pre_shuffled_probe_accuracy: f64,
    pub reward_local_behavior_accuracy: f64,
    pub reward_local_hidden_probe_accuracy: f64,
    pub reward_local_shuffled_probe_accuracy: f64,
    pub reward_local_probe_gain_over_pre: f64,
    pub reward_local_readout_rescue_gap: f64,
    pub target_directed_behavior_accuracy: f64,
    pub target_directed_hidden_probe_accuracy: f64,
    pub target_directed_shuffled_probe_accuracy: f64,
    pub target_directed_behavior_gain: f64,
    pub target_directed_probe_gain_over_pre: f64,
    pub reward_local_relative_weight_drift: f64,
    pub target_directed_relative_weight_drift: f64,
    pub action_readout_digest_before: u64,
    pub reward_local_action_readout_digest_after: u64,
    pub target_directed_action_readout_digest_after: u64,
    pub topology_digest_before: u64,
    pub reward_local_topology_digest_after: u64,
    pub target_directed_topology_digest_after: u64,
    pub adjustable_connection_count: usize,
    pub finite: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepresentationCapacitySeedResult {
    pub parameter_id: usize,
    pub seed: u64,
    pub rule_results: Vec<RepresentationRuleResult>,
    pub main_stream_topology_digest_before: u64,
    pub main_stream_topology_digest_after: u64,
    pub main_stream_action_readout_digest_before: u64,
    pub main_stream_action_readout_digest_after: u64,
    pub finite: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepresentationRuleSummary {
    pub rule: M1Rule,
    pub seed_run_count: usize,
    pub raw_sensor_probe_accuracy: Map0Interval,
    pub pre_behavior_accuracy: Map0Interval,
    pub pre_hidden_probe_accuracy: Map0Interval,
    pub pre_shuffled_probe_accuracy: Map0Interval,
    pub reward_local_behavior_accuracy: Map0Interval,
    pub reward_local_hidden_probe_accuracy: Map0Interval,
    pub reward_local_shuffled_probe_accuracy: Map0Interval,
    pub reward_local_probe_gain_over_pre: Map0Interval,
    pub reward_local_readout_rescue_gap: Map0Interval,
    pub target_directed_behavior_accuracy: Map0Interval,
    pub target_directed_hidden_probe_accuracy: Map0Interval,
    pub target_directed_shuffled_probe_accuracy: Map0Interval,
    pub target_directed_behavior_gain: Map0Interval,
    pub target_directed_probe_gain_over_pre: Map0Interval,
    pub reward_local_relative_weight_drift: Map0Interval,
    pub target_directed_relative_weight_drift: Map0Interval,
    pub finite_fraction: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepresentationNovelSummary {
    pub seed_run_count: usize,
    pub raw_sensor_probe_accuracy: Map0Interval,
    pub pre_behavior_accuracy: Map0Interval,
    pub pre_hidden_probe_accuracy: Map0Interval,
    pub pre_shuffled_probe_accuracy: Map0Interval,
    pub reward_local_behavior_accuracy: Map0Interval,
    pub reward_local_hidden_probe_accuracy: Map0Interval,
    pub reward_local_shuffled_probe_accuracy: Map0Interval,
    pub reward_local_probe_gain_over_pre: Map0Interval,
    pub reward_local_readout_rescue_gap: Map0Interval,
    pub target_directed_behavior_accuracy: Map0Interval,
    pub target_directed_hidden_probe_accuracy: Map0Interval,
    pub target_directed_shuffled_probe_accuracy: Map0Interval,
    pub target_directed_behavior_gain: Map0Interval,
    pub target_directed_probe_gain_over_pre: Map0Interval,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepresentationCapacityAcceptanceReport {
    pub m1_task_and_carrier_frozen: bool,
    pub diagnostic_is_offline_only: bool,
    pub development_and_confirmation_seeds_disjoint: bool,
    pub development_runs_complete: bool,
    pub confirmation_runs_complete: bool,
    pub all_four_rules_complete: bool,
    pub probe_train_and_evaluation_streams_disjoint: bool,
    pub raw_sensor_linearity_control_valid: bool,
    pub shuffled_label_controls_valid: bool,
    pub main_stream_unchanged: bool,
    pub action_readout_remained_frozen: bool,
    pub topology_and_connection_budget_preserved: bool,
    pub target_credit_uses_matched_trials_and_variables: bool,
    pub finite_outputs: bool,
    pub latent_novel_code_accessible: bool,
    pub reward_local_representation_formed: bool,
    pub alternative_readout_rescues_novel_rules: bool,
    pub target_directed_credit_rescues_novel_rules: bool,
    pub stage_passed: bool,
    pub passed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepresentationCapacityResult {
    pub version: String,
    pub config: RepresentationCapacityConfig,
    pub causal_contract: Vec<String>,
    pub carrier_parameters: Vec<Map0ParameterPoint>,
    pub development_seed_results: Vec<RepresentationCapacitySeedResult>,
    pub development_summaries: Vec<RepresentationRuleSummary>,
    pub development_novel_summary: RepresentationNovelSummary,
    pub confirmation_seed_results: Vec<RepresentationCapacitySeedResult>,
    pub confirmation_summaries: Vec<RepresentationRuleSummary>,
    pub confirmation_novel_summary: RepresentationNovelSummary,
    pub decision: RepresentationCapacityDecision,
    pub acceptance: RepresentationCapacityAcceptanceReport,
    pub conclusions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepresentationCapacityPublishedResult {
    pub version: String,
    pub config: RepresentationCapacityConfig,
    pub causal_contract: Vec<String>,
    pub carrier_parameters: Vec<Map0ParameterPoint>,
    pub development_summaries: Vec<RepresentationRuleSummary>,
    pub development_novel_summary: RepresentationNovelSummary,
    pub confirmation_summaries: Vec<RepresentationRuleSummary>,
    pub confirmation_novel_summary: RepresentationNovelSummary,
    pub decision: RepresentationCapacityDecision,
    pub acceptance: RepresentationCapacityAcceptanceReport,
    pub conclusions: Vec<String>,
}

impl RepresentationCapacityResult {
    pub fn published(&self) -> RepresentationCapacityPublishedResult {
        RepresentationCapacityPublishedResult {
            version: self.version.clone(),
            config: self.config,
            causal_contract: self.causal_contract.clone(),
            carrier_parameters: self.carrier_parameters.clone(),
            development_summaries: self.development_summaries.clone(),
            development_novel_summary: self.development_novel_summary,
            confirmation_summaries: self.confirmation_summaries.clone(),
            confirmation_novel_summary: self.confirmation_novel_summary,
            decision: self.decision,
            acceptance: self.acceptance,
            conclusions: self.conclusions.clone(),
        }
    }
}

pub fn run_representation_capacity_diagnostic(
    config: RepresentationCapacityConfig,
) -> Result<RepresentationCapacityResult, EmbodiedError> {
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
    let protocol = RepresentationCapacitySeedProtocol {
        adaptation_trial_count: config.m1_protocol.phase_trial_count,
        behavior_evaluation_trial_count: config.m1_protocol.evaluation_trial_count,
        exploration: config.m1_protocol.map2_protocol.exploration,
        probe_training_trials: config.probe_training_trials,
        probe_evaluation_trials: config.probe_evaluation_trials,
        probe_ridge: config.probe_ridge,
        shuffled_label_repeats: config.shuffled_label_repeats,
    };
    let run = |seeds: &[u64]| {
        carrier_parameters
            .iter()
            .flat_map(|point| {
                seeds.iter().map(move |seed| {
                    run_representation_capacity_seed(
                        carrier,
                        *point,
                        *seed,
                        homeostasis,
                        plasticity,
                        resource,
                        protocol,
                    )
                })
            })
            .collect::<Vec<_>>()
    };
    let development_seed_results = run(&development_seeds);
    let confirmation_seed_results = run(&confirmation_seeds);
    let development_summaries = summarize_rules(&development_seed_results);
    let confirmation_summaries = summarize_rules(&confirmation_seed_results);
    let development_novel_summary = summarize_novel(&development_seed_results);
    let confirmation_novel_summary = summarize_novel(&confirmation_seed_results);
    let all_results = development_seed_results
        .iter()
        .chain(&confirmation_seed_results)
        .collect::<Vec<_>>();
    let finite_outputs = all_results.iter().all(|row| row.finite);
    let raw_sensor_linearity_control_valid =
        confirmation_summaries[0].raw_sensor_probe_accuracy.mean >= 0.95
            && confirmation_summaries[1].raw_sensor_probe_accuracy.mean >= 0.95
            && confirmation_summaries[2].raw_sensor_probe_accuracy.mean
                <= config.maximum_shuffled_probe_accuracy
            && confirmation_summaries[3].raw_sensor_probe_accuracy.mean
                <= config.maximum_shuffled_probe_accuracy;
    let shuffled_label_controls_valid = confirmation_summaries.iter().all(|row| {
        row.pre_shuffled_probe_accuracy.mean <= config.maximum_shuffled_probe_accuracy
            && row.reward_local_shuffled_probe_accuracy.mean
                <= config.maximum_shuffled_probe_accuracy
            && row.target_directed_shuffled_probe_accuracy.mean
                <= config.maximum_shuffled_probe_accuracy
    });
    let latent_novel_code_accessible = confirmation_novel_summary.pre_hidden_probe_accuracy.mean
        >= config.hidden_probe_accuracy_threshold;
    let reward_local_representation_formed = confirmation_novel_summary
        .reward_local_hidden_probe_accuracy
        .mean
        >= config.hidden_probe_accuracy_threshold
        && confirmation_novel_summary
            .reward_local_probe_gain_over_pre
            .mean
            >= config.minimum_representation_gain;
    let alternative_readout_rescues_novel_rules = confirmation_novel_summary
        .reward_local_hidden_probe_accuracy
        .mean
        >= config.hidden_probe_accuracy_threshold
        && confirmation_novel_summary
            .reward_local_readout_rescue_gap
            .mean
            >= config.minimum_readout_rescue_gap;
    let target_directed_credit_rescues_novel_rules = confirmation_novel_summary
        .target_directed_behavior_accuracy
        .mean
        >= config.behavior_accuracy_threshold
        && confirmation_novel_summary
            .target_directed_behavior_gain
            .mean
            >= config.minimum_target_credit_gain;
    let acceptance = RepresentationCapacityAcceptanceReport {
        m1_task_and_carrier_frozen: config.m1_protocol.reference_parameter_ids
            == M1ExperimentConfig::default().reference_parameter_ids
            && config.m1_protocol.phase_trial_count
                == M1ExperimentConfig::default().phase_trial_count
            && config.m1_protocol.evaluation_trial_count
                == M1ExperimentConfig::default().evaluation_trial_count,
        diagnostic_is_offline_only: true,
        development_and_confirmation_seeds_disjoint: development_seeds
            .iter()
            .all(|seed| !confirmation_seeds.contains(seed)),
        development_runs_complete: development_seed_results.len()
            == carrier_parameters.len() * development_seeds.len(),
        confirmation_runs_complete: confirmation_seed_results.len()
            == carrier_parameters.len() * confirmation_seeds.len(),
        all_four_rules_complete: all_results.iter().all(|row| {
            row.rule_results.len() == M1Rule::UNIQUE.len()
                && row
                    .rule_results
                    .iter()
                    .zip(M1Rule::UNIQUE)
                    .all(|(result, rule)| result.rule == rule)
        }),
        probe_train_and_evaluation_streams_disjoint: true,
        raw_sensor_linearity_control_valid,
        shuffled_label_controls_valid,
        main_stream_unchanged: all_results.iter().all(|row| {
            row.main_stream_topology_digest_before == row.main_stream_topology_digest_after
                && row.main_stream_action_readout_digest_before
                    == row.main_stream_action_readout_digest_after
        }),
        action_readout_remained_frozen: all_results.iter().flat_map(|row| &row.rule_results).all(
            |result| {
                result.action_readout_digest_before
                    == result.reward_local_action_readout_digest_after
                    && result.action_readout_digest_before
                        == result.target_directed_action_readout_digest_after
            },
        ),
        topology_and_connection_budget_preserved: all_results
            .iter()
            .flat_map(|row| &row.rule_results)
            .all(|result| {
                result.topology_digest_before == result.reward_local_topology_digest_after
                    && result.topology_digest_before == result.target_directed_topology_digest_after
                    && result.adjustable_connection_count == HIDDEN_COUNT * 2
            }),
        target_credit_uses_matched_trials_and_variables: true,
        finite_outputs,
        latent_novel_code_accessible,
        reward_local_representation_formed,
        alternative_readout_rescues_novel_rules,
        target_directed_credit_rescues_novel_rules,
        stage_passed: false,
        passed: false,
    };
    let protocol_complete = acceptance.m1_task_and_carrier_frozen
        && acceptance.diagnostic_is_offline_only
        && acceptance.development_and_confirmation_seeds_disjoint
        && acceptance.development_runs_complete
        && acceptance.confirmation_runs_complete
        && acceptance.all_four_rules_complete
        && acceptance.probe_train_and_evaluation_streams_disjoint
        && acceptance.raw_sensor_linearity_control_valid
        && acceptance.shuffled_label_controls_valid
        && acceptance.main_stream_unchanged
        && acceptance.action_readout_remained_frozen
        && acceptance.topology_and_connection_budget_preserved
        && acceptance.target_credit_uses_matched_trials_and_variables
        && acceptance.finite_outputs;
    let acceptance = RepresentationCapacityAcceptanceReport {
        stage_passed: protocol_complete,
        passed: protocol_complete,
        ..acceptance
    };
    let baseline_capability = confirmation_novel_summary
        .reward_local_behavior_accuracy
        .mean
        >= config.behavior_accuracy_threshold;
    let decision = if !protocol_complete {
        RepresentationCapacityDecision::DiagnosticUnstable
    } else if baseline_capability {
        RepresentationCapacityDecision::BaselineCapabilityPresent
    } else if target_directed_credit_rescues_novel_rules {
        RepresentationCapacityDecision::CreditRoutingBottleneck
    } else if reward_local_representation_formed && alternative_readout_rescues_novel_rules {
        RepresentationCapacityDecision::LearnedRepresentationReadoutBottleneck
    } else if latent_novel_code_accessible
        && confirmation_novel_summary
            .reward_local_probe_gain_over_pre
            .mean
            < config.minimum_representation_gain
        && alternative_readout_rescues_novel_rules
    {
        RepresentationCapacityDecision::LatentSymbolCodeWithoutRuleFormation
    } else if confirmation_novel_summary
        .reward_local_hidden_probe_accuracy
        .mean
        < config.hidden_probe_accuracy_threshold
    {
        RepresentationCapacityDecision::RepresentationInsufficient
    } else {
        RepresentationCapacityDecision::InconclusiveBoundary
    };
    let conclusions = conclusions(&confirmation_novel_summary, decision);
    Ok(RepresentationCapacityResult {
        version: "adaptive-mechanism/m1-representation-v0.7".into(),
        config,
        causal_contract: vec![
            "M1 rules, carrier parameters, A-trained frozen action readout, 240-trial per-rule budget, and seed partitions are reused exactly".into(),
            "probe training and evaluation use disjoint deterministic streams and never write state, weights, topology, resources, or readout back to the live controller".into(),
            "the target-directed clone changes only the credit signal; it uses the same symbols, trials, adjustable recurrent slots, weight bounds, resource mechanism, and homeostasis".into(),
            "raw-sensor and shuffled-label probes are paired controls; all formal claims are aggregated over parameter-by-seed runs rather than probe trials".into(),
            "a successful external decoder establishes accessible information only; it is not counted as system-level rule learning".into(),
        ],
        carrier_parameters,
        development_seed_results,
        development_summaries,
        development_novel_summary,
        confirmation_seed_results,
        confirmation_summaries,
        confirmation_novel_summary,
        decision,
        acceptance,
        conclusions,
    })
}

fn summarize_rules(results: &[RepresentationCapacitySeedResult]) -> Vec<RepresentationRuleSummary> {
    M1Rule::UNIQUE
        .into_iter()
        .enumerate()
        .map(|(index, rule)| {
            let values = |f: fn(&RepresentationRuleResult) -> f64| {
                results
                    .iter()
                    .map(|row| f(&row.rule_results[index]))
                    .collect::<Vec<_>>()
            };
            RepresentationRuleSummary {
                rule,
                seed_run_count: results.len(),
                raw_sensor_probe_accuracy: mean_interval(&values(|row| {
                    row.raw_sensor_probe_accuracy
                })),
                pre_behavior_accuracy: mean_interval(&values(|row| row.pre_behavior_accuracy)),
                pre_hidden_probe_accuracy: mean_interval(&values(|row| {
                    row.pre_hidden_probe_accuracy
                })),
                pre_shuffled_probe_accuracy: mean_interval(&values(|row| {
                    row.pre_shuffled_probe_accuracy
                })),
                reward_local_behavior_accuracy: mean_interval(&values(|row| {
                    row.reward_local_behavior_accuracy
                })),
                reward_local_hidden_probe_accuracy: mean_interval(&values(|row| {
                    row.reward_local_hidden_probe_accuracy
                })),
                reward_local_shuffled_probe_accuracy: mean_interval(&values(|row| {
                    row.reward_local_shuffled_probe_accuracy
                })),
                reward_local_probe_gain_over_pre: mean_interval(&values(|row| {
                    row.reward_local_probe_gain_over_pre
                })),
                reward_local_readout_rescue_gap: mean_interval(&values(|row| {
                    row.reward_local_readout_rescue_gap
                })),
                target_directed_behavior_accuracy: mean_interval(&values(|row| {
                    row.target_directed_behavior_accuracy
                })),
                target_directed_hidden_probe_accuracy: mean_interval(&values(|row| {
                    row.target_directed_hidden_probe_accuracy
                })),
                target_directed_shuffled_probe_accuracy: mean_interval(&values(|row| {
                    row.target_directed_shuffled_probe_accuracy
                })),
                target_directed_behavior_gain: mean_interval(&values(|row| {
                    row.target_directed_behavior_gain
                })),
                target_directed_probe_gain_over_pre: mean_interval(&values(|row| {
                    row.target_directed_probe_gain_over_pre
                })),
                reward_local_relative_weight_drift: mean_interval(&values(|row| {
                    row.reward_local_relative_weight_drift
                })),
                target_directed_relative_weight_drift: mean_interval(&values(|row| {
                    row.target_directed_relative_weight_drift
                })),
                finite_fraction: results
                    .iter()
                    .filter(|row| row.rule_results[index].finite)
                    .count() as f64
                    / results.len().max(1) as f64,
            }
        })
        .collect()
}

fn summarize_novel(results: &[RepresentationCapacitySeedResult]) -> RepresentationNovelSummary {
    let per_run = results
        .iter()
        .map(|row| {
            let novel = &row.rule_results[1..4];
            let mean = |f: fn(&RepresentationRuleResult) -> f64| {
                novel.iter().map(|result| f(result)).sum::<f64>() / novel.len() as f64
            };
            [
                mean(|row| row.raw_sensor_probe_accuracy),
                mean(|row| row.pre_behavior_accuracy),
                mean(|row| row.pre_hidden_probe_accuracy),
                mean(|row| row.pre_shuffled_probe_accuracy),
                mean(|row| row.reward_local_behavior_accuracy),
                mean(|row| row.reward_local_hidden_probe_accuracy),
                mean(|row| row.reward_local_shuffled_probe_accuracy),
                mean(|row| row.reward_local_probe_gain_over_pre),
                mean(|row| row.reward_local_readout_rescue_gap),
                mean(|row| row.target_directed_behavior_accuracy),
                mean(|row| row.target_directed_hidden_probe_accuracy),
                mean(|row| row.target_directed_shuffled_probe_accuracy),
                mean(|row| row.target_directed_behavior_gain),
                mean(|row| row.target_directed_probe_gain_over_pre),
            ]
        })
        .collect::<Vec<_>>();
    let metric = |index: usize| {
        mean_interval(
            &per_run
                .iter()
                .map(|values| values[index])
                .collect::<Vec<_>>(),
        )
    };
    RepresentationNovelSummary {
        seed_run_count: results.len(),
        raw_sensor_probe_accuracy: metric(0),
        pre_behavior_accuracy: metric(1),
        pre_hidden_probe_accuracy: metric(2),
        pre_shuffled_probe_accuracy: metric(3),
        reward_local_behavior_accuracy: metric(4),
        reward_local_hidden_probe_accuracy: metric(5),
        reward_local_shuffled_probe_accuracy: metric(6),
        reward_local_probe_gain_over_pre: metric(7),
        reward_local_readout_rescue_gap: metric(8),
        target_directed_behavior_accuracy: metric(9),
        target_directed_hidden_probe_accuracy: metric(10),
        target_directed_shuffled_probe_accuracy: metric(11),
        target_directed_behavior_gain: metric(12),
        target_directed_probe_gain_over_pre: metric(13),
    }
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
    novel: &RepresentationNovelSummary,
    decision: RepresentationCapacityDecision,
) -> Vec<String> {
    vec![
        format!(
            "B/C/D 奖励局部调整的正式行为为 {:.1}%，训练前隐藏探针为 {:.1}%，训练后隐藏探针为 {:.1}%。",
            novel.reward_local_behavior_accuracy.mean * 100.0,
            novel.pre_hidden_probe_accuracy.mean * 100.0,
            novel.reward_local_hidden_probe_accuracy.mean * 100.0,
        ),
        format!(
            "奖励局部调整带来的隐藏探针增益为 {:+.2} pp [{:+.2}, {:+.2}]，外部读出相对固定读出的救援差为 {:+.2} pp。",
            novel.reward_local_probe_gain_over_pre.mean * 100.0,
            novel.reward_local_probe_gain_over_pre.lower95 * 100.0,
            novel.reward_local_probe_gain_over_pre.upper95 * 100.0,
            novel.reward_local_readout_rescue_gap.mean * 100.0,
        ),
        format!(
            "目标定向信用的正式行为为 {:.1}%，相对奖励局部调整的配对增益为 {:+.2} pp [{:+.2}, {:+.2}]。",
            novel.target_directed_behavior_accuracy.mean * 100.0,
            novel.target_directed_behavior_gain.mean * 100.0,
            novel.target_directed_behavior_gain.lower95 * 100.0,
            novel.target_directed_behavior_gain.upper95 * 100.0,
        ),
        format!(
            "原始两位输入探针为 {:.1}%，随机标签隐藏探针为 {:.1}%；M1-R 正式决策为 {decision:?}。",
            novel.raw_sensor_probe_accuracy.mean * 100.0,
            novel.reward_local_shuffled_probe_accuracy.mean * 100.0,
        ),
        match decision {
            RepresentationCapacityDecision::BaselineCapabilityPresent => "M1 的新规则失败没有在本次配对复现；先审计协议差异，不能进入新机制。",
            RepresentationCapacityDecision::CreditRoutingBottleneck => "相同可塑变量在目标定向信用下达到能力门槛；下一阶段应比较局部信用估计，而不是扩大结构或规模。",
            RepresentationCapacityDecision::LearnedRepresentationReadoutBottleneck => "奖励调整形成了额外可读表征，但冻结读出无法表达；下一阶段应诊断表征—读出接口。",
            RepresentationCapacityDecision::LatentSymbolCodeWithoutRuleFormation => "外部探针利用的是训练前已存在的潜在符号码，奖励调整没有形成足够的规则相关几何变化；不能把探针救援算作学习成功。",
            RepresentationCapacityDecision::RepresentationInsufficient => "训练后状态仍不支持稳定线性解码；下一阶段才允许单因素测试可塑子空间或节点容量。",
            RepresentationCapacityDecision::InconclusiveBoundary => "结果落在预注册门槛之间；保持机制冻结并先扩大独立确认，不按方向性结果选机制。",
            RepresentationCapacityDecision::DiagnosticUnstable => "协议或负对照未通过，当前结果不可作机制路由。",
        }
        .into(),
    ]
}

fn validate_config(config: RepresentationCapacityConfig) -> Result<(), EmbodiedError> {
    if config.m1_protocol.development_seed_count == 0
        || config.m1_protocol.confirmation_seed_count == 0
        || config.m1_protocol.phase_trial_count == 0
        || config.m1_protocol.evaluation_trial_count == 0
        || config.probe_training_trials < HIDDEN_COUNT + 1
        || config.probe_evaluation_trials == 0
        || !config.probe_training_trials.is_multiple_of(4)
        || !config.probe_evaluation_trials.is_multiple_of(4)
        || !config.probe_ridge.is_finite()
        || config.probe_ridge <= 0.0
        || config.shuffled_label_repeats == 0
        || [
            config.behavior_accuracy_threshold,
            config.hidden_probe_accuracy_threshold,
            config.minimum_representation_gain,
            config.minimum_target_credit_gain,
            config.minimum_readout_rescue_gap,
            config.maximum_shuffled_probe_accuracy,
        ]
        .iter()
        .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
    {
        return Err(EmbodiedError::InvalidExperiment);
    }
    Ok(())
}
