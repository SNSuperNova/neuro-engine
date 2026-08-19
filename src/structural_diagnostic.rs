use serde::Serialize;

use crate::learnability_map::{
    ContinuousResource, DualTimescaleHomeostasis, HomeostasisMechanism, PlasticityMechanism,
    ResourceMechanism, SoftBoundedPlasticity, run_structural_diagnostic_seed, seed_partition,
};
use crate::map1::{parameter_points, protocol_config};
use crate::{
    EmbodiedError, M1Rule, M2CExperimentConfig, M2CMechanismParameter, Map0Interval,
    Map0ParameterPoint,
};

const DEVELOPMENT_SEED_LABEL: u64 = 0x4d32_4444_4556_0101;
const CONFIRMATION_SEED_LABEL: u64 = 0x4d32_4443_4f4e_4601;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum StructuralDiagnosticDecision {
    EvidenceInformative,
    EvidenceRankingWeak,
    EvidenceNotInformative,
    CounterfactualEffectsFlat,
    DiagnosticUnstable,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralDiagnosticConfig {
    pub m2c_protocol: M2CExperimentConfig,
    pub structural_parameters: M2CMechanismParameter,
    pub checkpoint_global_trials: [usize; 18],
    pub forward_training_trials: usize,
    pub evaluation_trial_count: usize,
    pub shuffled_ranking_count: usize,
    pub minimum_oracle_benefit: f64,
    pub minimum_spearman_advantage: f64,
    pub minimum_selected_benefit_advantage: f64,
}

impl Default for StructuralDiagnosticConfig {
    fn default() -> Self {
        Self {
            m2c_protocol: M2CExperimentConfig::default(),
            structural_parameters: M2CMechanismParameter {
                id: 3,
                rewiring_interval: 64,
                evidence_decay: 0.80,
            },
            checkpoint_global_trials: [
                64, 128, 192, 256, 320, 384, 448, 512, 576, 640, 704, 768, 832, 896, 960, 1024,
                1088, 1152,
            ],
            forward_training_trials: 32,
            evaluation_trial_count: 48,
            shuffled_ranking_count: 16,
            minimum_oracle_benefit: 0.02,
            minimum_spearman_advantage: 0.10,
            minimum_selected_benefit_advantage: 0.01,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct StructuralDiagnosticSeedProtocol {
    pub phase_trial_count: usize,
    pub exploration: f64,
    pub structural_parameters: M2CMechanismParameter,
    pub checkpoint_global_trials: [usize; 18],
    pub forward_training_trials: usize,
    pub evaluation_trial_count: usize,
    pub shuffled_ranking_count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralCandidateCounterfactual {
    pub source_unit: usize,
    pub evidence_score: f64,
    pub post_horizon_accuracy: f64,
    pub benefit_over_no_swap: f64,
    pub evidence_rank: f64,
    pub benefit_rank: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralCheckpointDiagnostic {
    pub global_trial: usize,
    pub phase_index: usize,
    pub rule: M1Rule,
    pub target_unit: usize,
    pub replaced_slot: usize,
    pub replaced_source_unit: usize,
    pub candidate_count: usize,
    pub no_swap_post_horizon_accuracy: f64,
    pub candidates: Vec<StructuralCandidateCounterfactual>,
    pub spearman_correlation: f64,
    pub mean_shuffled_spearman_correlation: f64,
    pub selected_source_unit: usize,
    pub selected_benefit: f64,
    pub random_mean_benefit: f64,
    pub oracle_source_unit: usize,
    pub oracle_benefit: f64,
    pub selected_regret: f64,
    pub selected_is_top_quartile: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralDiagnosticSeedResult {
    pub parameter_id: usize,
    pub seed: u64,
    pub checkpoints: Vec<StructuralCheckpointDiagnostic>,
    pub mean_spearman_correlation: f64,
    pub mean_shuffled_spearman_correlation: f64,
    pub mean_correlation_advantage: f64,
    pub mean_selected_benefit: f64,
    pub mean_random_benefit: f64,
    pub mean_selected_benefit_advantage: f64,
    pub mean_oracle_benefit: f64,
    pub mean_selected_regret: f64,
    pub top_quartile_hit_rate: f64,
    pub topology_digest_before: u64,
    pub topology_digest_after: u64,
    pub action_readout_digest_before: u64,
    pub action_readout_digest_after: u64,
    pub finite: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralDiagnosticSummary {
    pub seed_run_count: usize,
    pub checkpoint_count: usize,
    pub candidate_evaluation_count: usize,
    pub mean_spearman_correlation: Map0Interval,
    pub mean_shuffled_spearman_correlation: Map0Interval,
    pub mean_correlation_advantage: Map0Interval,
    pub mean_selected_benefit: Map0Interval,
    pub mean_random_benefit: Map0Interval,
    pub mean_selected_benefit_advantage: Map0Interval,
    pub mean_oracle_benefit: Map0Interval,
    pub mean_selected_regret: Map0Interval,
    pub mean_top_quartile_hit_rate: Map0Interval,
    pub finite_fraction: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralDiagnosticPhaseSummary {
    pub phase_index: usize,
    pub rule: M1Rule,
    pub checkpoint_count: usize,
    pub mean_spearman_correlation: Map0Interval,
    pub mean_correlation_advantage: Map0Interval,
    pub mean_selected_benefit: Map0Interval,
    pub mean_random_benefit: Map0Interval,
    pub mean_oracle_benefit: Map0Interval,
    pub mean_selected_regret: Map0Interval,
    pub top_quartile_hit_rate: Map0Interval,
    pub top_quartile_chance_level: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralDiagnosticAcceptanceReport {
    pub prior_m2c_candidate_frozen: bool,
    pub diagnostic_is_offline_only: bool,
    pub topology_unchanged_in_main_stream: bool,
    pub action_readout_remained_frozen: bool,
    pub development_and_confirmation_seeds_disjoint: bool,
    pub development_runs_complete: bool,
    pub confirmation_runs_complete: bool,
    pub all_checkpoints_complete: bool,
    pub all_legal_candidates_enumerated: bool,
    pub paired_horizons_and_randomness_frozen: bool,
    pub shuffled_ranking_controls_complete: bool,
    pub finite_outputs: bool,
    pub useful_counterfactual_swaps_exist: bool,
    pub evidence_ranking_beats_shuffle: bool,
    pub evidence_selection_beats_random: bool,
    pub evidence_informative: bool,
    pub stage_passed: bool,
    pub passed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralDiagnosticResult {
    pub version: String,
    pub config: StructuralDiagnosticConfig,
    pub evidence_contract: Vec<String>,
    pub counterfactual_contract: Vec<String>,
    pub carrier_parameters: Vec<Map0ParameterPoint>,
    pub development_seed_results: Vec<StructuralDiagnosticSeedResult>,
    pub development_summary: StructuralDiagnosticSummary,
    pub development_phase_summaries: Vec<StructuralDiagnosticPhaseSummary>,
    pub confirmation_seed_results: Vec<StructuralDiagnosticSeedResult>,
    pub confirmation_summary: StructuralDiagnosticSummary,
    pub confirmation_phase_summaries: Vec<StructuralDiagnosticPhaseSummary>,
    pub decision: StructuralDiagnosticDecision,
    pub acceptance: StructuralDiagnosticAcceptanceReport,
    pub conclusions: Vec<String>,
}

pub fn run_structural_diagnostic(
    config: StructuralDiagnosticConfig,
) -> Result<StructuralDiagnosticResult, EmbodiedError> {
    validate_config(config)?;
    let map1 = config
        .m2c_protocol
        .m1_protocol
        .map2_protocol
        .map2b_protocol
        .map2a_protocol
        .map1_protocol;
    let carrier = protocol_config(map1);
    let all_parameters = parameter_points(map1);
    let carrier_parameters = config
        .m2c_protocol
        .m1_protocol
        .reference_parameter_ids
        .iter()
        .map(|id| all_parameters[*id])
        .collect::<Vec<_>>();
    let development_seeds = seed_partition(
        map1.seed ^ DEVELOPMENT_SEED_LABEL,
        config.m2c_protocol.m1_protocol.development_seed_count,
    );
    let confirmation_seeds = seed_partition(
        map1.seed ^ CONFIRMATION_SEED_LABEL,
        config.m2c_protocol.m1_protocol.confirmation_seed_count,
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
            .m2c_protocol
            .m1_protocol
            .map2_protocol
            .map2b_protocol
            .map2a_protocol
            .soft_bound_scale,
    });
    let map2b = config.m2c_protocol.m1_protocol.map2_protocol.map2b_protocol;
    let resource = ResourceMechanism::Continuous(ContinuousResource {
        initial_level: map2b.initial_resource,
        supply_rate: map2b.supply_rate,
        maintenance_cost: map2b.maintenance_cost,
        activity_cost: map2b.activity_cost,
        plasticity_cost: map2b.plasticity_cost,
        minimum_modulation: map2b.minimum_modulation,
    });
    let protocol = StructuralDiagnosticSeedProtocol {
        phase_trial_count: config.m2c_protocol.m1_protocol.phase_trial_count,
        exploration: config.m2c_protocol.m1_protocol.map2_protocol.exploration,
        structural_parameters: config.structural_parameters,
        checkpoint_global_trials: config.checkpoint_global_trials,
        forward_training_trials: config.forward_training_trials,
        evaluation_trial_count: config.evaluation_trial_count,
        shuffled_ranking_count: config.shuffled_ranking_count,
    };
    let run = |seeds: &[u64]| {
        carrier_parameters
            .iter()
            .flat_map(|point| {
                seeds.iter().map(move |seed| {
                    run_structural_diagnostic_seed(
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
    let development_summary = summarize(&development_seed_results);
    let confirmation_summary = summarize(&confirmation_seed_results);
    let development_phase_summaries = summarize_phases(&development_seed_results);
    let confirmation_phase_summaries = summarize_phases(&confirmation_seed_results);
    let expected_development = carrier_parameters.len() * development_seeds.len();
    let expected_confirmation = carrier_parameters.len() * confirmation_seeds.len();
    let all_results = development_seed_results
        .iter()
        .chain(&confirmation_seed_results)
        .collect::<Vec<_>>();
    let useful_counterfactual_swaps_exist = confirmation_summary.mean_oracle_benefit.mean
        >= config.minimum_oracle_benefit
        && confirmation_summary.mean_oracle_benefit.lower95 > 0.0;
    let evidence_ranking_beats_shuffle = confirmation_summary.mean_correlation_advantage.mean
        >= config.minimum_spearman_advantage
        && confirmation_summary.mean_correlation_advantage.lower95 > 0.0;
    let evidence_selection_beats_random = confirmation_summary.mean_selected_benefit_advantage.mean
        >= config.minimum_selected_benefit_advantage
        && confirmation_summary.mean_selected_benefit_advantage.lower95 > 0.0;
    let finite_outputs = all_results.iter().all(|row| row.finite);
    let evidence_informative = useful_counterfactual_swaps_exist
        && evidence_ranking_beats_shuffle
        && evidence_selection_beats_random;
    let decision = if !finite_outputs {
        StructuralDiagnosticDecision::DiagnosticUnstable
    } else if !useful_counterfactual_swaps_exist {
        StructuralDiagnosticDecision::CounterfactualEffectsFlat
    } else if !evidence_ranking_beats_shuffle {
        StructuralDiagnosticDecision::EvidenceNotInformative
    } else if !evidence_selection_beats_random {
        StructuralDiagnosticDecision::EvidenceRankingWeak
    } else {
        StructuralDiagnosticDecision::EvidenceInformative
    };
    let acceptance = StructuralDiagnosticAcceptanceReport {
        prior_m2c_candidate_frozen: config.structural_parameters
            == M2CMechanismParameter {
                id: 3,
                rewiring_interval: 64,
                evidence_decay: 0.80,
            },
        diagnostic_is_offline_only: true,
        topology_unchanged_in_main_stream: all_results
            .iter()
            .all(|row| row.topology_digest_before == row.topology_digest_after),
        action_readout_remained_frozen: all_results
            .iter()
            .all(|row| row.action_readout_digest_before == row.action_readout_digest_after),
        development_and_confirmation_seeds_disjoint: development_seeds
            .iter()
            .all(|seed| !confirmation_seeds.contains(seed)),
        development_runs_complete: development_seed_results.len() == expected_development,
        confirmation_runs_complete: confirmation_seed_results.len() == expected_confirmation,
        all_checkpoints_complete: all_results
            .iter()
            .all(|row| row.checkpoints.len() == config.checkpoint_global_trials.len()),
        all_legal_candidates_enumerated: all_results.iter().all(|row| {
            row.checkpoints
                .iter()
                .all(|checkpoint| checkpoint.candidate_count == 17)
        }),
        paired_horizons_and_randomness_frozen: true,
        shuffled_ranking_controls_complete: all_results.iter().all(|row| {
            row.checkpoints
                .iter()
                .all(|checkpoint| checkpoint.mean_shuffled_spearman_correlation.is_finite())
        }),
        finite_outputs,
        useful_counterfactual_swaps_exist,
        evidence_ranking_beats_shuffle,
        evidence_selection_beats_random,
        evidence_informative,
        stage_passed: false,
        passed: false,
    };
    let protocol_complete = acceptance.prior_m2c_candidate_frozen
        && acceptance.diagnostic_is_offline_only
        && acceptance.topology_unchanged_in_main_stream
        && acceptance.action_readout_remained_frozen
        && acceptance.development_and_confirmation_seeds_disjoint
        && acceptance.development_runs_complete
        && acceptance.confirmation_runs_complete
        && acceptance.all_checkpoints_complete
        && acceptance.all_legal_candidates_enumerated
        && acceptance.paired_horizons_and_randomness_frozen
        && acceptance.shuffled_ranking_controls_complete
        && acceptance.finite_outputs;
    let acceptance = StructuralDiagnosticAcceptanceReport {
        stage_passed: protocol_complete,
        passed: protocol_complete,
        ..acceptance
    };
    let conclusions = conclusions(
        &confirmation_summary,
        &confirmation_phase_summaries,
        decision,
    );
    Ok(StructuralDiagnosticResult {
        version: "adaptive-mechanism/m2c-diagnostic-v0.4-offline-counterfactual".into(),
        config,
        evidence_contract: vec![
            "the frozen M2C reward-modulated local eligibility EMA is read without modification".into(),
            "rule identifiers, target labels, logits, and action readout weights are absent from evidence ranking".into(),
            "task accuracy is used only by the offline evaluator after every candidate has been chosen".into(),
        ],
        counterfactual_contract: vec![
            "the live stream never rewires; every swap occurs in an isolated controller clone".into(),
            "all 17 legal inactive sources replace the same weakest slot and inherit the same weight".into(),
            "no-swap and every candidate share forward horizon, trial symbols, exploration draws, and evaluation probes".into(),
            "statistics aggregate checkpoints within each carrier-parameter and seed run before confidence intervals".into(),
        ],
        carrier_parameters,
        development_seed_results,
        development_summary,
        development_phase_summaries,
        confirmation_seed_results,
        confirmation_summary,
        confirmation_phase_summaries,
        decision,
        acceptance,
        conclusions,
    })
}

fn summarize_phases(
    results: &[StructuralDiagnosticSeedResult],
) -> Vec<StructuralDiagnosticPhaseSummary> {
    M1Rule::SEQUENCE
        .into_iter()
        .enumerate()
        .map(|(phase_index, rule)| {
            let rows = results
                .iter()
                .flat_map(|result| {
                    result
                        .checkpoints
                        .iter()
                        .filter(|checkpoint| checkpoint.phase_index == phase_index)
                })
                .collect::<Vec<_>>();
            let values = |f: fn(&StructuralCheckpointDiagnostic) -> f64| {
                rows.iter().map(|row| f(row)).collect::<Vec<_>>()
            };
            StructuralDiagnosticPhaseSummary {
                phase_index,
                rule,
                checkpoint_count: rows.len(),
                mean_spearman_correlation: mean_interval(&values(|row| row.spearman_correlation)),
                mean_correlation_advantage: mean_interval(&values(|row| {
                    row.spearman_correlation - row.mean_shuffled_spearman_correlation
                })),
                mean_selected_benefit: mean_interval(&values(|row| row.selected_benefit)),
                mean_random_benefit: mean_interval(&values(|row| row.random_mean_benefit)),
                mean_oracle_benefit: mean_interval(&values(|row| row.oracle_benefit)),
                mean_selected_regret: mean_interval(&values(|row| row.selected_regret)),
                top_quartile_hit_rate: mean_interval(&values(|row| {
                    f64::from(row.selected_is_top_quartile)
                })),
                top_quartile_chance_level: 5.0 / 17.0,
            }
        })
        .collect()
}

fn summarize(results: &[StructuralDiagnosticSeedResult]) -> StructuralDiagnosticSummary {
    let values =
        |f: fn(&StructuralDiagnosticSeedResult) -> f64| results.iter().map(f).collect::<Vec<_>>();
    StructuralDiagnosticSummary {
        seed_run_count: results.len(),
        checkpoint_count: results.iter().map(|row| row.checkpoints.len()).sum(),
        candidate_evaluation_count: results
            .iter()
            .flat_map(|row| &row.checkpoints)
            .map(|checkpoint| checkpoint.candidate_count)
            .sum(),
        mean_spearman_correlation: mean_interval(&values(|row| row.mean_spearman_correlation)),
        mean_shuffled_spearman_correlation: mean_interval(&values(|row| {
            row.mean_shuffled_spearman_correlation
        })),
        mean_correlation_advantage: mean_interval(&values(|row| row.mean_correlation_advantage)),
        mean_selected_benefit: mean_interval(&values(|row| row.mean_selected_benefit)),
        mean_random_benefit: mean_interval(&values(|row| row.mean_random_benefit)),
        mean_selected_benefit_advantage: mean_interval(&values(|row| {
            row.mean_selected_benefit_advantage
        })),
        mean_oracle_benefit: mean_interval(&values(|row| row.mean_oracle_benefit)),
        mean_selected_regret: mean_interval(&values(|row| row.mean_selected_regret)),
        mean_top_quartile_hit_rate: mean_interval(&values(|row| row.top_quartile_hit_rate)),
        finite_fraction: results.iter().filter(|row| row.finite).count() as f64
            / results.len().max(1) as f64,
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
    summary: &StructuralDiagnosticSummary,
    phases: &[StructuralDiagnosticPhaseSummary],
    decision: StructuralDiagnosticDecision,
) -> Vec<String> {
    let strongest_phase = phases
        .iter()
        .max_by(|left, right| {
            left.mean_spearman_correlation
                .mean
                .total_cmp(&right.mean_spearman_correlation.mean)
        })
        .expect("five diagnostic phases");
    vec![
        format!(
            "独立确认包含 {} 个种子-载体运行、{} 个检查点和 {} 个候选换边反事实。",
            summary.seed_run_count, summary.checkpoint_count, summary.candidate_evaluation_count
        ),
        format!(
            "局部证据与反事实收益的平均 Spearman 相关为 {:.3} [{:.3}, {:.3}]；相对打乱排名的优势为 {:.3} [{:.3}, {:.3}]。",
            summary.mean_spearman_correlation.mean,
            summary.mean_spearman_correlation.lower95,
            summary.mean_spearman_correlation.upper95,
            summary.mean_correlation_advantage.mean,
            summary.mean_correlation_advantage.lower95,
            summary.mean_correlation_advantage.upper95,
        ),
        format!(
            "证据首选换边平均收益为 {:+.1} pp，随机候选期望为 {:+.1} pp，差为 {:+.1} pp [{:+.1}, {:+.1}]。",
            summary.mean_selected_benefit.mean * 100.0,
            summary.mean_random_benefit.mean * 100.0,
            summary.mean_selected_benefit_advantage.mean * 100.0,
            summary.mean_selected_benefit_advantage.lower95 * 100.0,
            summary.mean_selected_benefit_advantage.upper95 * 100.0,
        ),
        format!(
            "反事实 oracle 平均收益为 {:+.1} pp；证据首选的平均遗憾为 {:.1} pp，top-quartile 命中率为 {:.1}%。",
            summary.mean_oracle_benefit.mean * 100.0,
            summary.mean_selected_regret.mean * 100.0,
            summary.mean_top_quartile_hit_rate.mean * 100.0,
        ),
        format!(
            "相关最强的是阶段 {}（{:?}）：Spearman {:.3}，首选收益 {:+.1} pp，oracle 收益 {:+.1} pp；该阶段不能替代总体确认结论。",
            strongest_phase.phase_index + 1,
            strongest_phase.rule,
            strongest_phase.mean_spearman_correlation.mean,
            strongest_phase.mean_selected_benefit.mean * 100.0,
            strongest_phase.mean_oracle_benefit.mean * 100.0,
        ),
        format!("离线结构证据诊断决策为 {decision:?}。"),
        match decision {
            StructuralDiagnosticDecision::EvidenceInformative => "局部结构信用存在；M2C 在线失败更可能来自更新时机或结构动力学。".into(),
            StructuralDiagnosticDecision::EvidenceRankingWeak => "证据总体排序带有信息，但首选决策没有形成稳定收益；瓶颈在选择规则或时机。".into(),
            StructuralDiagnosticDecision::EvidenceNotInformative => "可用换边存在，但当前局部资格证据不能识别它们；瓶颈在结构信用信号。".into(),
            StructuralDiagnosticDecision::CounterfactualEffectsFlat => "在冻结的一边替换自由度和短期窗口内，连 oracle 换边也缺少收益；瓶颈在结构自由度或评价时间尺度。".into(),
            StructuralDiagnosticDecision::DiagnosticUnstable => "反事实诊断出现非有限输出，不能解释结构信用。".into(),
        },
    ]
}

fn validate_config(config: StructuralDiagnosticConfig) -> Result<(), EmbodiedError> {
    let total_trials = config.m2c_protocol.m1_protocol.phase_trial_count * M1Rule::SEQUENCE.len();
    let mut checkpoints = config.checkpoint_global_trials;
    checkpoints.sort_unstable();
    let valid_checkpoints = checkpoints == config.checkpoint_global_trials
        && checkpoints[0] > 0
        && checkpoints[17] <= total_trials
        && checkpoints
            .iter()
            .all(|trial| trial.is_multiple_of(config.structural_parameters.rewiring_interval));
    if config.m2c_protocol.m1_protocol.development_seed_count == 0
        || config.m2c_protocol.m1_protocol.confirmation_seed_count == 0
        || config.structural_parameters.rewiring_interval == 0
        || !config.structural_parameters.evidence_decay.is_finite()
        || !(0.0..1.0).contains(&config.structural_parameters.evidence_decay)
        || config.forward_training_trials == 0
        || config.evaluation_trial_count == 0
        || config.shuffled_ranking_count == 0
        || !valid_checkpoints
        || [
            config.minimum_oracle_benefit,
            config.minimum_spearman_advantage,
            config.minimum_selected_benefit_advantage,
        ]
        .iter()
        .any(|value| !value.is_finite() || *value < 0.0)
    {
        return Err(EmbodiedError::InvalidExperiment);
    }
    Ok(())
}
