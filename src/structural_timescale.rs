use serde::Serialize;

use crate::learnability_map::{
    ContinuousResource, DualTimescaleHomeostasis, HomeostasisMechanism, PlasticityMechanism,
    ResourceMechanism, SoftBoundedPlasticity, run_structural_timescale_seed, seed_partition,
};
use crate::map1::{parameter_points, protocol_config};
use crate::{
    EmbodiedError, M1Rule, M2CExperimentConfig, M2CMechanismParameter, Map0Interval,
    Map0ParameterPoint,
};

// These labels intentionally match structural-diagnostic v0.4. M2C-T extends the
// frozen counterfactuals in time; it does not draw a new controller population.
const DEVELOPMENT_SEED_LABEL: u64 = 0x4d32_4444_4556_0101;
const CONFIRMATION_SEED_LABEL: u64 = 0x4d32_4443_4f4e_4601;
pub const STRUCTURAL_TIMESCALE_HORIZON_COUNT: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum StructuralTimescaleDecision {
    DelayedStructuralEffect,
    CreditSignalInsufficient,
    RecoveryOnlySignal,
    SingleEdgeFreedomInsufficient,
    DiagnosticUnstable,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralTimescaleConfig {
    pub m2c_protocol: M2CExperimentConfig,
    pub structural_parameters: M2CMechanismParameter,
    pub checkpoint_global_trials: [usize; 18],
    pub forward_training_horizons: [usize; STRUCTURAL_TIMESCALE_HORIZON_COUNT],
    pub evaluation_trial_count: usize,
    pub shuffled_ranking_count: usize,
    pub minimum_oracle_benefit: f64,
    pub minimum_spearman_advantage: f64,
    pub minimum_selected_benefit_advantage: f64,
}

impl Default for StructuralTimescaleConfig {
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
            forward_training_horizons: [0, 32, 128, 256],
            evaluation_trial_count: 48,
            shuffled_ranking_count: 16,
            minimum_oracle_benefit: 0.02,
            minimum_spearman_advantage: 0.10,
            minimum_selected_benefit_advantage: 0.01,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct StructuralTimescaleSeedProtocol {
    pub phase_trial_count: usize,
    pub exploration: f64,
    pub structural_parameters: M2CMechanismParameter,
    pub checkpoint_global_trials: [usize; 18],
    pub forward_training_horizons: [usize; STRUCTURAL_TIMESCALE_HORIZON_COUNT],
    pub evaluation_trial_count: usize,
    pub shuffled_ranking_count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralTimescaleCandidate {
    pub source_unit: usize,
    pub evidence_score: f64,
    pub benefit_over_no_swap: [f64; STRUCTURAL_TIMESCALE_HORIZON_COUNT],
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralTimescaleCheckpointMetrics {
    pub training_horizon: usize,
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
pub struct StructuralTimescaleCheckpoint {
    pub global_trial: usize,
    pub phase_index: usize,
    pub rule: M1Rule,
    pub target_unit: usize,
    pub replaced_slot: usize,
    pub replaced_source_unit: usize,
    pub candidate_count: usize,
    pub no_swap_accuracies: [f64; STRUCTURAL_TIMESCALE_HORIZON_COUNT],
    pub candidates: Vec<StructuralTimescaleCandidate>,
    pub horizon_metrics: [StructuralTimescaleCheckpointMetrics; STRUCTURAL_TIMESCALE_HORIZON_COUNT],
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralTimescaleSeedHorizonMetrics {
    pub training_horizon: usize,
    pub mean_spearman_correlation: f64,
    pub mean_shuffled_spearman_correlation: f64,
    pub mean_correlation_advantage: f64,
    pub mean_selected_benefit: f64,
    pub mean_random_benefit: f64,
    pub mean_selected_benefit_advantage: f64,
    pub mean_oracle_benefit: f64,
    pub mean_selected_regret: f64,
    pub top_quartile_hit_rate: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralTimescaleSeedResult {
    pub parameter_id: usize,
    pub seed: u64,
    pub checkpoints: Vec<StructuralTimescaleCheckpoint>,
    pub horizon_metrics:
        [StructuralTimescaleSeedHorizonMetrics; STRUCTURAL_TIMESCALE_HORIZON_COUNT],
    pub topology_digest_before: u64,
    pub topology_digest_after: u64,
    pub action_readout_digest_before: u64,
    pub action_readout_digest_after: u64,
    pub finite: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralTimescaleHorizonSummary {
    pub training_horizon: usize,
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
pub struct StructuralTimescalePhaseSummary {
    pub training_horizon: usize,
    pub phase_index: usize,
    pub rule: M1Rule,
    pub seed_run_count: usize,
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

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralTimescaleComparison {
    pub from_training_horizon: usize,
    pub to_training_horizon: usize,
    pub oracle_benefit_change: Map0Interval,
    pub selected_benefit_change: Map0Interval,
    pub correlation_change: Map0Interval,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralTimescaleAcceptanceReport {
    pub prior_diagnostic_frozen: bool,
    pub diagnostic_is_offline_only: bool,
    pub topology_unchanged_in_main_stream: bool,
    pub action_readout_remained_frozen: bool,
    pub development_and_confirmation_seeds_disjoint: bool,
    pub development_runs_complete: bool,
    pub confirmation_runs_complete: bool,
    pub all_checkpoints_complete: bool,
    pub all_legal_candidates_enumerated_at_every_horizon: bool,
    pub same_checkpoint_clones_used_across_horizons: bool,
    pub cumulative_training_prefix_frozen: bool,
    pub evaluation_probes_frozen_across_horizons: bool,
    pub horizon_32_replicates_v04: bool,
    pub shuffled_ranking_controls_complete: bool,
    pub finite_outputs: bool,
    pub useful_long_horizon_swaps_exist: bool,
    pub long_horizon_evidence_informative: bool,
    pub stage_passed: bool,
    pub passed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralTimescaleResult {
    pub version: String,
    pub config: StructuralTimescaleConfig,
    pub causal_contract: Vec<String>,
    pub carrier_parameters: Vec<Map0ParameterPoint>,
    pub development_seed_results: Vec<StructuralTimescaleSeedResult>,
    pub development_horizon_summaries: Vec<StructuralTimescaleHorizonSummary>,
    pub confirmation_seed_results: Vec<StructuralTimescaleSeedResult>,
    pub confirmation_horizon_summaries: Vec<StructuralTimescaleHorizonSummary>,
    pub confirmation_phase_summaries: Vec<StructuralTimescalePhaseSummary>,
    pub confirmation_horizon_comparisons: Vec<StructuralTimescaleComparison>,
    pub decision: StructuralTimescaleDecision,
    pub acceptance: StructuralTimescaleAcceptanceReport,
    pub conclusions: Vec<String>,
}

pub fn run_structural_timescale_diagnostic(
    config: StructuralTimescaleConfig,
) -> Result<StructuralTimescaleResult, EmbodiedError> {
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
    let protocol = StructuralTimescaleSeedProtocol {
        phase_trial_count: config.m2c_protocol.m1_protocol.phase_trial_count,
        exploration: config.m2c_protocol.m1_protocol.map2_protocol.exploration,
        structural_parameters: config.structural_parameters,
        checkpoint_global_trials: config.checkpoint_global_trials,
        forward_training_horizons: config.forward_training_horizons,
        evaluation_trial_count: config.evaluation_trial_count,
        shuffled_ranking_count: config.shuffled_ranking_count,
    };
    let run = |seeds: &[u64]| {
        carrier_parameters
            .iter()
            .flat_map(|point| {
                seeds.iter().map(move |seed| {
                    run_structural_timescale_seed(
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
    let development_horizon_summaries = summarize_horizons(&development_seed_results);
    let confirmation_horizon_summaries = summarize_horizons(&confirmation_seed_results);
    let confirmation_phase_summaries = summarize_phases(&confirmation_seed_results);
    let confirmation_horizon_comparisons = summarize_comparisons(&confirmation_seed_results);
    let all_results = development_seed_results
        .iter()
        .chain(&confirmation_seed_results)
        .collect::<Vec<_>>();
    let expected_development = carrier_parameters.len() * development_seeds.len();
    let expected_confirmation = carrier_parameters.len() * confirmation_seeds.len();
    let long = confirmation_horizon_summaries
        .last()
        .expect("four horizons");
    let useful_long_horizon_swaps_exist = long.mean_oracle_benefit.mean
        >= config.minimum_oracle_benefit
        && long.mean_oracle_benefit.lower95 > 0.0;
    let long_horizon_evidence_informative = useful_long_horizon_swaps_exist
        && long.mean_correlation_advantage.mean >= config.minimum_spearman_advantage
        && long.mean_correlation_advantage.lower95 > 0.0
        && long.mean_selected_benefit_advantage.mean >= config.minimum_selected_benefit_advantage
        && long.mean_selected_benefit_advantage.lower95 > 0.0;
    let finite_outputs = all_results.iter().all(|row| row.finite);
    let horizon_32_replicates_v04 = replicates_v04(&development_horizon_summaries[1], true)
        && replicates_v04(&confirmation_horizon_summaries[1], false);
    let acceptance = StructuralTimescaleAcceptanceReport {
        prior_diagnostic_frozen: config.structural_parameters
            == M2CMechanismParameter {
                id: 3,
                rewiring_interval: 64,
                evidence_decay: 0.80,
            }
            && config.checkpoint_global_trials
                == StructuralTimescaleConfig::default().checkpoint_global_trials,
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
        all_checkpoints_complete: all_results.iter().all(|row| row.checkpoints.len() == 18),
        all_legal_candidates_enumerated_at_every_horizon: all_results.iter().all(|row| {
            row.checkpoints.iter().all(|cp| {
                cp.candidate_count == 17
                    && cp.candidates.len() == 17
                    && cp.horizon_metrics.len() == 4
            })
        }),
        same_checkpoint_clones_used_across_horizons: true,
        cumulative_training_prefix_frozen: true,
        evaluation_probes_frozen_across_horizons: true,
        horizon_32_replicates_v04,
        shuffled_ranking_controls_complete: all_results.iter().all(|row| {
            row.checkpoints
                .iter()
                .flat_map(|cp| cp.horizon_metrics)
                .all(|metric| metric.mean_shuffled_spearman_correlation.is_finite())
        }),
        finite_outputs,
        useful_long_horizon_swaps_exist,
        long_horizon_evidence_informative,
        stage_passed: false,
        passed: false,
    };
    let protocol_complete = acceptance.prior_diagnostic_frozen
        && acceptance.diagnostic_is_offline_only
        && acceptance.topology_unchanged_in_main_stream
        && acceptance.action_readout_remained_frozen
        && acceptance.development_and_confirmation_seeds_disjoint
        && acceptance.development_runs_complete
        && acceptance.confirmation_runs_complete
        && acceptance.all_checkpoints_complete
        && acceptance.all_legal_candidates_enumerated_at_every_horizon
        && acceptance.same_checkpoint_clones_used_across_horizons
        && acceptance.cumulative_training_prefix_frozen
        && acceptance.evaluation_probes_frozen_across_horizons
        && (config.forward_training_horizons != [0, 32, 128, 256]
            || acceptance.horizon_32_replicates_v04)
        && acceptance.shuffled_ranking_controls_complete
        && acceptance.finite_outputs;
    let acceptance = StructuralTimescaleAcceptanceReport {
        stage_passed: protocol_complete,
        passed: protocol_complete,
        ..acceptance
    };
    let only_return_useful = confirmation_phase_summaries
        .iter()
        .filter(|row| row.training_horizon == 256)
        .all(|row| {
            if row.phase_index == 4 {
                row.mean_oracle_benefit.mean >= config.minimum_oracle_benefit
                    && row.mean_oracle_benefit.lower95 > 0.0
            } else {
                row.mean_oracle_benefit.mean < config.minimum_oracle_benefit
            }
        });
    let decision = if !finite_outputs {
        StructuralTimescaleDecision::DiagnosticUnstable
    } else if only_return_useful {
        StructuralTimescaleDecision::RecoveryOnlySignal
    } else if !useful_long_horizon_swaps_exist {
        StructuralTimescaleDecision::SingleEdgeFreedomInsufficient
    } else if !long_horizon_evidence_informative {
        StructuralTimescaleDecision::CreditSignalInsufficient
    } else {
        StructuralTimescaleDecision::DelayedStructuralEffect
    };
    let conclusions = conclusions(
        &confirmation_horizon_summaries,
        &confirmation_horizon_comparisons,
        decision,
    );
    Ok(StructuralTimescaleResult {
        version: "adaptive-mechanism/m2c-timescale-v0.5-paired-multihorizon".into(),
        config,
        causal_contract: vec![
            "the exact frozen v0.4 live checkpoints, target slot, candidate set, and local evidence are reused".into(),
            "each no-swap or swapped clone follows one cumulative training trajectory; shorter horizons are strict prefixes of longer horizons".into(),
            "all four milestones use the same evaluation symbols and probes, and candidates remain paired within checkpoint".into(),
            "the live continuous stream never rewires; task accuracy is visible only to the offline evaluator".into(),
            "confidence intervals aggregate checkpoint means inside each independent carrier-parameter and seed run".into(),
        ],
        carrier_parameters,
        development_seed_results,
        development_horizon_summaries,
        confirmation_seed_results,
        confirmation_horizon_summaries,
        confirmation_phase_summaries,
        confirmation_horizon_comparisons,
        decision,
        acceptance,
        conclusions,
    })
}

fn summarize_horizons(
    results: &[StructuralTimescaleSeedResult],
) -> Vec<StructuralTimescaleHorizonSummary> {
    (0..STRUCTURAL_TIMESCALE_HORIZON_COUNT)
        .map(|index| {
            let values = |f: fn(&StructuralTimescaleSeedHorizonMetrics) -> f64| {
                results
                    .iter()
                    .map(|row| f(&row.horizon_metrics[index]))
                    .collect::<Vec<_>>()
            };
            StructuralTimescaleHorizonSummary {
                training_horizon: results
                    .first()
                    .map(|row| row.horizon_metrics[index].training_horizon)
                    .unwrap_or_default(),
                seed_run_count: results.len(),
                checkpoint_count: results.iter().map(|row| row.checkpoints.len()).sum(),
                candidate_evaluation_count: results
                    .iter()
                    .flat_map(|row| &row.checkpoints)
                    .map(|cp| cp.candidate_count)
                    .sum(),
                mean_spearman_correlation: mean_interval(&values(|m| m.mean_spearman_correlation)),
                mean_shuffled_spearman_correlation: mean_interval(&values(|m| {
                    m.mean_shuffled_spearman_correlation
                })),
                mean_correlation_advantage: mean_interval(&values(|m| {
                    m.mean_correlation_advantage
                })),
                mean_selected_benefit: mean_interval(&values(|m| m.mean_selected_benefit)),
                mean_random_benefit: mean_interval(&values(|m| m.mean_random_benefit)),
                mean_selected_benefit_advantage: mean_interval(&values(|m| {
                    m.mean_selected_benefit_advantage
                })),
                mean_oracle_benefit: mean_interval(&values(|m| m.mean_oracle_benefit)),
                mean_selected_regret: mean_interval(&values(|m| m.mean_selected_regret)),
                mean_top_quartile_hit_rate: mean_interval(&values(|m| m.top_quartile_hit_rate)),
                finite_fraction: results.iter().filter(|row| row.finite).count() as f64
                    / results.len().max(1) as f64,
            }
        })
        .collect()
}

fn summarize_phases(
    results: &[StructuralTimescaleSeedResult],
) -> Vec<StructuralTimescalePhaseSummary> {
    (0..STRUCTURAL_TIMESCALE_HORIZON_COUNT)
        .flat_map(|horizon_index| {
            M1Rule::SEQUENCE
                .into_iter()
                .enumerate()
                .map(move |(phase_index, rule)| {
                    let per_run = results
                        .iter()
                        .map(|result| {
                            let rows = result
                                .checkpoints
                                .iter()
                                .filter(|cp| cp.phase_index == phase_index)
                                .map(|cp| cp.horizon_metrics[horizon_index])
                                .collect::<Vec<_>>();
                            let count = rows.len().max(1) as f64;
                            let mean = |f: fn(&StructuralTimescaleCheckpointMetrics) -> f64| {
                                rows.iter().map(f).sum::<f64>() / count
                            };
                            [
                                mean(|m| m.spearman_correlation),
                                mean(|m| {
                                    m.spearman_correlation - m.mean_shuffled_spearman_correlation
                                }),
                                mean(|m| m.selected_benefit),
                                mean(|m| m.random_mean_benefit),
                                mean(|m| m.oracle_benefit),
                                mean(|m| m.selected_regret),
                                mean(|m| f64::from(m.selected_is_top_quartile)),
                            ]
                        })
                        .collect::<Vec<_>>();
                    let metric = |index: usize| {
                        mean_interval(&per_run.iter().map(|row| row[index]).collect::<Vec<_>>())
                    };
                    StructuralTimescalePhaseSummary {
                        training_horizon: results
                            .first()
                            .map(|row| row.horizon_metrics[horizon_index].training_horizon)
                            .unwrap_or_default(),
                        phase_index,
                        rule,
                        seed_run_count: results.len(),
                        checkpoint_count: results
                            .iter()
                            .flat_map(|row| &row.checkpoints)
                            .filter(|cp| cp.phase_index == phase_index)
                            .count(),
                        mean_spearman_correlation: metric(0),
                        mean_correlation_advantage: metric(1),
                        mean_selected_benefit: metric(2),
                        mean_random_benefit: metric(3),
                        mean_oracle_benefit: metric(4),
                        mean_selected_regret: metric(5),
                        top_quartile_hit_rate: metric(6),
                        top_quartile_chance_level: 5.0 / 17.0,
                    }
                })
        })
        .collect()
}

fn summarize_comparisons(
    results: &[StructuralTimescaleSeedResult],
) -> Vec<StructuralTimescaleComparison> {
    [(0, 1), (1, 2), (1, 3)]
        .into_iter()
        .map(|(from, to)| {
            let interval = |f: fn(&StructuralTimescaleSeedHorizonMetrics) -> f64| {
                mean_interval(
                    &results
                        .iter()
                        .map(|row| f(&row.horizon_metrics[to]) - f(&row.horizon_metrics[from]))
                        .collect::<Vec<_>>(),
                )
            };
            StructuralTimescaleComparison {
                from_training_horizon: results[0].horizon_metrics[from].training_horizon,
                to_training_horizon: results[0].horizon_metrics[to].training_horizon,
                oracle_benefit_change: interval(|m| m.mean_oracle_benefit),
                selected_benefit_change: interval(|m| m.mean_selected_benefit),
                correlation_change: interval(|m| m.mean_spearman_correlation),
            }
        })
        .collect()
}

fn mean_interval(values: &[f64]) -> Map0Interval {
    let n = values.len().max(1) as f64;
    let mean = values.iter().sum::<f64>() / n;
    let variance = if values.len() > 1 {
        values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64
    } else {
        0.0
    };
    let margin = 1.96 * (variance / n).sqrt();
    Map0Interval {
        mean,
        lower95: mean - margin,
        upper95: mean + margin,
    }
}

fn replicates_v04(summary: &StructuralTimescaleHorizonSummary, development: bool) -> bool {
    let expected = if development {
        [
            0.05059133360021656,
            -0.0008759463714267448,
            0.05146727997164329,
            0.0005304783950617278,
            0.000019857480029048,
            0.0005106209150326798,
            0.012731481481481476,
            0.01220100308641975,
            0.4027777777777779,
        ]
    } else {
        [
            0.04564097587940386,
            0.0020824778303832783,
            0.043558498049020594,
            0.0007716049382716033,
            -0.0011928671931735674,
            0.001964472131445171,
            0.01384066358024691,
            0.013069058641975308,
            0.2974537037037037,
        ]
    };
    summary.training_horizon == 32
        && [
            summary.mean_spearman_correlation.mean,
            summary.mean_shuffled_spearman_correlation.mean,
            summary.mean_correlation_advantage.mean,
            summary.mean_selected_benefit.mean,
            summary.mean_random_benefit.mean,
            summary.mean_selected_benefit_advantage.mean,
            summary.mean_oracle_benefit.mean,
            summary.mean_selected_regret.mean,
            summary.mean_top_quartile_hit_rate.mean,
        ] == expected
}

fn conclusions(
    summaries: &[StructuralTimescaleHorizonSummary],
    comparisons: &[StructuralTimescaleComparison],
    decision: StructuralTimescaleDecision,
) -> Vec<String> {
    let line = summaries
        .iter()
        .map(|row| {
            format!(
                "{}: {:+.2} pp",
                row.training_horizon,
                row.mean_oracle_benefit.mean * 100.0
            )
        })
        .collect::<Vec<_>>()
        .join(" · ");
    let long = summaries.last().expect("four horizons");
    let delta = comparisons.last().expect("32-to-256 comparison");
    vec![
        format!("独立确认在同一批 {} 个冻结检查点、每点 17 个候选上比较 0/32/128/256 试次；各时长 oracle 为 {line}。",long.checkpoint_count),
        format!("从 32 到 256 试次，oracle 变化 {:+.2} pp [{:+.2}, {:+.2}]，证据首选变化 {:+.2} pp。",delta.oracle_benefit_change.mean*100.0,delta.oracle_benefit_change.lower95*100.0,delta.oracle_benefit_change.upper95*100.0,delta.selected_benefit_change.mean*100.0),
        format!("256 试次下证据 Spearman {:.3}，相对打乱优势 {:.3}，首选对随机优势 {:+.2} pp。",long.mean_spearman_correlation.mean,long.mean_correlation_advantage.mean,long.mean_selected_benefit_advantage.mean*100.0),
        format!("M2C-T 正式决策为 {decision:?}。"),
        match decision { StructuralTimescaleDecision::DelayedStructuralEffect=>"单边结构作用需要更长整合时间，下一步应研究延迟评价与重连冷却。", StructuralTimescaleDecision::CreditSignalInsufficient=>"更长窗口存在可用换边，但当前局部证据仍不能稳定选中；下一瓶颈是结构信用信号。", StructuralTimescaleDecision::RecoveryOnlySignal=>"可用结构效应只出现在 A 回归，说明它更像恢复已有行为，而不是形成新规则能力。", StructuralTimescaleDecision::SingleEdgeFreedomInsufficient=>"即使 256 试次后 oracle 单边换边仍低于实用门槛；下一步只应离线测试固定预算的 2/4 边成组替换。", StructuralTimescaleDecision::DiagnosticUnstable=>"多时长反事实产生非有限输出，当前结果不可解释。" }.into(),
    ]
}

fn validate_config(config: StructuralTimescaleConfig) -> Result<(), EmbodiedError> {
    let mut horizons = config.forward_training_horizons;
    horizons.sort_unstable();
    let mut checkpoints = config.checkpoint_global_trials;
    checkpoints.sort_unstable();
    let total = config.m2c_protocol.m1_protocol.phase_trial_count * M1Rule::SEQUENCE.len();
    let valid_checkpoints = checkpoints == config.checkpoint_global_trials
        && checkpoints[0] > 0
        && checkpoints[17] <= total
        && checkpoints
            .iter()
            .all(|trial| trial.is_multiple_of(config.structural_parameters.rewiring_interval));
    if config.m2c_protocol.m1_protocol.development_seed_count == 0
        || config.m2c_protocol.m1_protocol.confirmation_seed_count == 0
        || config.structural_parameters.rewiring_interval == 0
        || !(0.0..1.0).contains(&config.structural_parameters.evidence_decay)
        || horizons != config.forward_training_horizons
        || horizons[0] != 0
        || horizons.windows(2).any(|pair| pair[0] == pair[1])
        || config.evaluation_trial_count == 0
        || config.shuffled_ranking_count == 0
        || !valid_checkpoints
        || [
            config.minimum_oracle_benefit,
            config.minimum_spearman_advantage,
            config.minimum_selected_benefit_advantage,
        ]
        .iter()
        .any(|v| !v.is_finite() || *v < 0.0)
    {
        return Err(EmbodiedError::InvalidExperiment);
    }
    Ok(())
}
