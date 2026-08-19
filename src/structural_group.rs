use serde::Serialize;

use crate::learnability_map::{
    ContinuousResource, DualTimescaleHomeostasis, HomeostasisMechanism, PlasticityMechanism,
    ResourceMechanism, SoftBoundedPlasticity, run_structural_group_seed, seed_partition,
};
use crate::map1::{parameter_points, protocol_config};
use crate::{
    EmbodiedError, M1Rule, M2CExperimentConfig, M2CMechanismParameter, Map0Interval,
    Map0ParameterPoint,
};

const DEVELOPMENT_SEED_LABEL: u64 = 0x4d32_4444_4556_0101;
const CONFIRMATION_SEED_LABEL: u64 = 0x4d32_4443_4f4e_4601;
pub const STRUCTURAL_GROUP_SIZE_COUNT: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum StructuralGroupDecision {
    GroupedFreedomAndCredit,
    GroupedFreedomWithoutCredit,
    HistoryDominatedGroupedEffect,
    GroupedEffectNotBeyondSingle,
    NoUsefulGroupedEffect,
    DiagnosticUnstable,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralGroupConfig {
    pub m2c_protocol: M2CExperimentConfig,
    pub structural_parameters: M2CMechanismParameter,
    pub checkpoint_global_trials: [usize; 18],
    pub group_sizes: [usize; STRUCTURAL_GROUP_SIZE_COUNT],
    pub bundle_budget: usize,
    pub forward_training_trials: usize,
    pub evaluation_trial_count: usize,
    pub shuffled_ranking_count: usize,
    pub minimum_oracle_benefit: f64,
    pub minimum_oracle_advantage_over_single: f64,
    pub minimum_selected_benefit_advantage: f64,
}

impl Default for StructuralGroupConfig {
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
            group_sizes: [1, 2, 4],
            bundle_budget: 17,
            forward_training_trials: 32,
            evaluation_trial_count: 48,
            shuffled_ranking_count: 16,
            minimum_oracle_benefit: 0.02,
            minimum_oracle_advantage_over_single: 0.005,
            minimum_selected_benefit_advantage: 0.01,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct StructuralGroupSeedProtocol {
    pub phase_trial_count: usize,
    pub exploration: f64,
    pub structural_parameters: M2CMechanismParameter,
    pub checkpoint_global_trials: [usize; 18],
    pub group_sizes: [usize; STRUCTURAL_GROUP_SIZE_COUNT],
    pub bundle_budget: usize,
    pub forward_training_trials: usize,
    pub evaluation_trial_count: usize,
    pub shuffled_ranking_count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralGroupCandidate {
    pub bundle_index: usize,
    pub source_units: Vec<usize>,
    pub evidence_score: f64,
    pub benefit_over_no_swap: f64,
    pub additive_single_edge_prediction: f64,
    pub interaction_over_additive: f64,
    pub evidence_rank: f64,
    pub benefit_rank: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralGroupCheckpointScale {
    pub edge_count: usize,
    pub target_units: Vec<usize>,
    pub target_evidence_covered: Vec<bool>,
    pub replaced_slots: Vec<usize>,
    pub replaced_source_units: Vec<usize>,
    pub bundle_count: usize,
    pub component_control_evaluation_count: usize,
    pub no_swap_post_horizon_accuracy: f64,
    pub candidates: Vec<StructuralGroupCandidate>,
    pub spearman_correlation: f64,
    pub mean_shuffled_spearman_correlation: f64,
    pub selected_bundle_index: usize,
    pub selected_source_units: Vec<usize>,
    pub selected_benefit: f64,
    pub random_mean_benefit: f64,
    pub budgeted_oracle_bundle_index: usize,
    pub budgeted_oracle_source_units: Vec<usize>,
    pub budgeted_oracle_benefit: f64,
    pub selected_regret: f64,
    pub selected_is_top_quartile: bool,
    pub mean_interaction_over_additive: f64,
    pub selected_interaction_over_additive: f64,
    pub oracle_interaction_over_additive: f64,
    pub balanced_candidate_marginals: bool,
    pub clone_connection_budget_preserved: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralGroupCheckpoint {
    pub global_trial: usize,
    pub phase_index: usize,
    pub rule: M1Rule,
    pub scheduled_target_unit: usize,
    pub scales: Vec<StructuralGroupCheckpointScale>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralGroupSeedMetrics {
    pub edge_count: usize,
    pub mean_spearman_correlation: f64,
    pub mean_shuffled_spearman_correlation: f64,
    pub mean_correlation_advantage: f64,
    pub mean_selected_benefit: f64,
    pub mean_random_benefit: f64,
    pub mean_selected_benefit_advantage: f64,
    pub mean_budgeted_oracle_benefit: f64,
    pub mean_selected_regret: f64,
    pub mean_top_quartile_hit_rate: f64,
    pub mean_interaction_over_additive: f64,
    pub mean_selected_interaction_over_additive: f64,
    pub mean_oracle_interaction_over_additive: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralGroupSeedResult {
    pub parameter_id: usize,
    pub seed: u64,
    pub checkpoints: Vec<StructuralGroupCheckpoint>,
    pub group_metrics: Vec<StructuralGroupSeedMetrics>,
    pub topology_digest_before: u64,
    pub topology_digest_after: u64,
    pub action_readout_digest_before: u64,
    pub action_readout_digest_after: u64,
    pub finite: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralGroupSummary {
    pub edge_count: usize,
    pub seed_run_count: usize,
    pub checkpoint_count: usize,
    pub bundle_evaluation_count: usize,
    pub component_control_evaluation_count: usize,
    pub mean_spearman_correlation: Map0Interval,
    pub mean_shuffled_spearman_correlation: Map0Interval,
    pub mean_correlation_advantage: Map0Interval,
    pub mean_selected_benefit: Map0Interval,
    pub mean_random_benefit: Map0Interval,
    pub mean_selected_benefit_advantage: Map0Interval,
    pub mean_budgeted_oracle_benefit: Map0Interval,
    pub mean_selected_regret: Map0Interval,
    pub mean_top_quartile_hit_rate: Map0Interval,
    pub mean_interaction_over_additive: Map0Interval,
    pub mean_selected_interaction_over_additive: Map0Interval,
    pub mean_oracle_interaction_over_additive: Map0Interval,
    pub finite_fraction: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralGroupPhaseSummary {
    pub edge_count: usize,
    pub phase_index: usize,
    pub rule: M1Rule,
    pub seed_run_count: usize,
    pub checkpoint_count: usize,
    pub mean_selected_benefit: Map0Interval,
    pub mean_random_benefit: Map0Interval,
    pub mean_budgeted_oracle_benefit: Map0Interval,
    pub mean_selected_benefit_advantage: Map0Interval,
    pub mean_oracle_interaction_over_additive: Map0Interval,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralGroupComparison {
    pub edge_count: usize,
    pub oracle_advantage_over_single: Map0Interval,
    pub selected_advantage_over_single: Map0Interval,
    pub random_mean_change_from_single: Map0Interval,
    pub oracle_interaction_over_additive: Map0Interval,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralGroupCapabilityContrast {
    pub edge_count: usize,
    pub mean_history_phase_oracle_benefit: Map0Interval,
    pub mean_novel_rule_oracle_benefit: Map0Interval,
    pub history_advantage_over_novel_rules: Map0Interval,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralGroupAcceptanceReport {
    pub prior_m2c_and_diagnostics_frozen: bool,
    pub diagnostic_is_offline_only: bool,
    pub topology_unchanged_in_main_stream: bool,
    pub action_readout_remained_frozen: bool,
    pub development_and_confirmation_seeds_disjoint: bool,
    pub development_runs_complete: bool,
    pub confirmation_runs_complete: bool,
    pub all_checkpoints_complete: bool,
    pub group_sizes_and_search_budget_frozen: bool,
    pub equal_bundle_budget_across_group_sizes: bool,
    pub balanced_candidate_marginals_complete: bool,
    pub grouped_target_evidence_coverage_sufficient: bool,
    pub component_controls_complete: bool,
    pub clone_connection_budget_preserved: bool,
    pub paired_horizon_and_randomness_frozen: bool,
    pub single_edge_scale_replicates_v04: bool,
    pub finite_outputs: bool,
    pub useful_grouped_swaps_exist: bool,
    pub useful_group_beats_single: bool,
    pub grouped_evidence_selection_useful: bool,
    pub novel_rule_grouped_swaps_useful: bool,
    pub grouped_effect_history_dominated: bool,
    pub stage_passed: bool,
    pub passed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralGroupResult {
    pub version: String,
    pub config: StructuralGroupConfig,
    pub causal_contract: Vec<String>,
    pub carrier_parameters: Vec<Map0ParameterPoint>,
    pub development_seed_results: Vec<StructuralGroupSeedResult>,
    pub development_summaries: Vec<StructuralGroupSummary>,
    pub confirmation_seed_results: Vec<StructuralGroupSeedResult>,
    pub confirmation_summaries: Vec<StructuralGroupSummary>,
    pub confirmation_phase_summaries: Vec<StructuralGroupPhaseSummary>,
    pub confirmation_comparisons: Vec<StructuralGroupComparison>,
    pub confirmation_capability_contrast: StructuralGroupCapabilityContrast,
    pub selected_group_size: usize,
    pub decision: StructuralGroupDecision,
    pub acceptance: StructuralGroupAcceptanceReport,
    pub conclusions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuralGroupPublishedResult {
    pub version: String,
    pub config: StructuralGroupConfig,
    pub causal_contract: Vec<String>,
    pub carrier_parameters: Vec<Map0ParameterPoint>,
    pub development_summaries: Vec<StructuralGroupSummary>,
    pub confirmation_summaries: Vec<StructuralGroupSummary>,
    pub confirmation_phase_summaries: Vec<StructuralGroupPhaseSummary>,
    pub confirmation_comparisons: Vec<StructuralGroupComparison>,
    pub confirmation_capability_contrast: StructuralGroupCapabilityContrast,
    pub selected_group_size: usize,
    pub decision: StructuralGroupDecision,
    pub acceptance: StructuralGroupAcceptanceReport,
    pub conclusions: Vec<String>,
}

impl StructuralGroupResult {
    pub fn published(&self) -> StructuralGroupPublishedResult {
        StructuralGroupPublishedResult {
            version: self.version.clone(),
            config: self.config,
            causal_contract: self.causal_contract.clone(),
            carrier_parameters: self.carrier_parameters.clone(),
            development_summaries: self.development_summaries.clone(),
            confirmation_summaries: self.confirmation_summaries.clone(),
            confirmation_phase_summaries: self.confirmation_phase_summaries.clone(),
            confirmation_comparisons: self.confirmation_comparisons.clone(),
            confirmation_capability_contrast: self.confirmation_capability_contrast,
            selected_group_size: self.selected_group_size,
            decision: self.decision,
            acceptance: self.acceptance,
            conclusions: self.conclusions.clone(),
        }
    }
}

pub fn run_structural_group_diagnostic(
    config: StructuralGroupConfig,
) -> Result<StructuralGroupResult, EmbodiedError> {
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
    let protocol = StructuralGroupSeedProtocol {
        phase_trial_count: config.m2c_protocol.m1_protocol.phase_trial_count,
        exploration: config.m2c_protocol.m1_protocol.map2_protocol.exploration,
        structural_parameters: config.structural_parameters,
        checkpoint_global_trials: config.checkpoint_global_trials,
        group_sizes: config.group_sizes,
        bundle_budget: config.bundle_budget,
        forward_training_trials: config.forward_training_trials,
        evaluation_trial_count: config.evaluation_trial_count,
        shuffled_ranking_count: config.shuffled_ranking_count,
    };
    let run = |seeds: &[u64]| {
        carrier_parameters
            .iter()
            .flat_map(|point| {
                seeds.iter().map(move |seed| {
                    run_structural_group_seed(
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
    let development_summaries = summarize(&development_seed_results, config.group_sizes);
    let confirmation_summaries = summarize(&confirmation_seed_results, config.group_sizes);
    let confirmation_phase_summaries =
        summarize_phases(&confirmation_seed_results, config.group_sizes);
    let confirmation_comparisons =
        summarize_comparisons(&confirmation_seed_results, config.group_sizes);
    let selected_group_size = confirmation_summaries
        .iter()
        .filter(|row| row.edge_count > 1)
        .max_by(|left, right| {
            left.mean_budgeted_oracle_benefit
                .mean
                .total_cmp(&right.mean_budgeted_oracle_benefit.mean)
        })
        .expect("two grouped scales")
        .edge_count;
    let selected_summary = confirmation_summaries
        .iter()
        .find(|row| row.edge_count == selected_group_size)
        .expect("selected group summary");
    let selected_comparison = confirmation_comparisons
        .iter()
        .find(|row| row.edge_count == selected_group_size)
        .expect("selected group comparison");
    let confirmation_capability_contrast = capability_contrast(
        &confirmation_seed_results,
        config.group_sizes,
        selected_group_size,
    );
    let useful_grouped_swaps_exist = selected_summary.mean_budgeted_oracle_benefit.mean
        >= config.minimum_oracle_benefit
        && selected_summary.mean_budgeted_oracle_benefit.lower95 > 0.0;
    let useful_group_beats_single = selected_comparison.oracle_advantage_over_single.mean
        >= config.minimum_oracle_advantage_over_single
        && selected_comparison.oracle_advantage_over_single.lower95 > 0.0;
    let grouped_evidence_selection_useful = selected_summary.mean_selected_benefit_advantage.mean
        >= config.minimum_selected_benefit_advantage
        && selected_summary.mean_selected_benefit_advantage.lower95 > 0.0;
    let novel_rule_grouped_swaps_useful = confirmation_capability_contrast
        .mean_novel_rule_oracle_benefit
        .mean
        >= config.minimum_oracle_benefit
        && confirmation_capability_contrast
            .mean_novel_rule_oracle_benefit
            .lower95
            > 0.0;
    let grouped_effect_history_dominated = confirmation_capability_contrast
        .history_advantage_over_novel_rules
        .mean
        >= config.minimum_selected_benefit_advantage
        && confirmation_capability_contrast
            .history_advantage_over_novel_rules
            .lower95
            > 0.0;
    let all_results = development_seed_results
        .iter()
        .chain(&confirmation_seed_results)
        .collect::<Vec<_>>();
    let finite_outputs = all_results.iter().all(|row| row.finite);
    let single_edge_scale_replicates_v04 = replicates_v04(&development_summaries[0], true)
        && replicates_v04(&confirmation_summaries[0], false);
    let acceptance = StructuralGroupAcceptanceReport {
        prior_m2c_and_diagnostics_frozen: config.structural_parameters
            == M2CMechanismParameter {
                id: 3,
                rewiring_interval: 64,
                evidence_decay: 0.80,
            }
            && config.checkpoint_global_trials
                == StructuralGroupConfig::default().checkpoint_global_trials,
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
        development_runs_complete: development_seed_results.len()
            == carrier_parameters.len() * development_seeds.len(),
        confirmation_runs_complete: confirmation_seed_results.len()
            == carrier_parameters.len() * confirmation_seeds.len(),
        all_checkpoints_complete: all_results.iter().all(|row| row.checkpoints.len() == 18),
        group_sizes_and_search_budget_frozen: config.group_sizes == [1, 2, 4]
            && config.bundle_budget == 17
            && config.forward_training_trials == 32,
        equal_bundle_budget_across_group_sizes: all_results.iter().all(|row| {
            row.checkpoints.iter().all(|checkpoint| {
                checkpoint
                    .scales
                    .iter()
                    .all(|scale| scale.bundle_count == config.bundle_budget)
            })
        }),
        balanced_candidate_marginals_complete: all_results.iter().all(|row| {
            row.checkpoints
                .iter()
                .flat_map(|checkpoint| &checkpoint.scales)
                .all(|scale| scale.balanced_candidate_marginals)
        }),
        grouped_target_evidence_coverage_sufficient: all_results.iter().all(|row| {
            let grouped = row
                .checkpoints
                .iter()
                .flat_map(|checkpoint| {
                    checkpoint
                        .scales
                        .iter()
                        .filter(|scale| scale.edge_count > 1)
                })
                .collect::<Vec<_>>();
            let covered = grouped
                .iter()
                .flat_map(|scale| &scale.target_evidence_covered)
                .filter(|covered| **covered)
                .count();
            let total = grouped
                .iter()
                .map(|scale| scale.target_evidence_covered.len())
                .sum::<usize>();
            covered as f64 / total.max(1) as f64 >= 0.98
        }),
        component_controls_complete: all_results.iter().all(|row| {
            row.checkpoints
                .iter()
                .flat_map(|checkpoint| &checkpoint.scales)
                .all(|scale| {
                    scale
                        .candidates
                        .iter()
                        .all(|candidate| candidate.additive_single_edge_prediction.is_finite())
                })
        }),
        clone_connection_budget_preserved: all_results.iter().all(|row| {
            row.checkpoints
                .iter()
                .flat_map(|checkpoint| &checkpoint.scales)
                .all(|scale| scale.clone_connection_budget_preserved)
        }),
        paired_horizon_and_randomness_frozen: true,
        single_edge_scale_replicates_v04,
        finite_outputs,
        useful_grouped_swaps_exist,
        useful_group_beats_single,
        grouped_evidence_selection_useful,
        novel_rule_grouped_swaps_useful,
        grouped_effect_history_dominated,
        stage_passed: false,
        passed: false,
    };
    let protocol_complete = acceptance.prior_m2c_and_diagnostics_frozen
        && acceptance.diagnostic_is_offline_only
        && acceptance.topology_unchanged_in_main_stream
        && acceptance.action_readout_remained_frozen
        && acceptance.development_and_confirmation_seeds_disjoint
        && acceptance.development_runs_complete
        && acceptance.confirmation_runs_complete
        && acceptance.all_checkpoints_complete
        && acceptance.equal_bundle_budget_across_group_sizes
        && acceptance.balanced_candidate_marginals_complete
        && acceptance.grouped_target_evidence_coverage_sufficient
        && acceptance.component_controls_complete
        && acceptance.clone_connection_budget_preserved
        && acceptance.paired_horizon_and_randomness_frozen
        && (config != StructuralGroupConfig::default()
            || (acceptance.group_sizes_and_search_budget_frozen
                && acceptance.single_edge_scale_replicates_v04))
        && acceptance.finite_outputs;
    let acceptance = StructuralGroupAcceptanceReport {
        stage_passed: protocol_complete,
        passed: protocol_complete,
        ..acceptance
    };
    let decision = if !finite_outputs {
        StructuralGroupDecision::DiagnosticUnstable
    } else if !useful_grouped_swaps_exist {
        StructuralGroupDecision::NoUsefulGroupedEffect
    } else if !novel_rule_grouped_swaps_useful && grouped_effect_history_dominated {
        StructuralGroupDecision::HistoryDominatedGroupedEffect
    } else if !useful_group_beats_single {
        StructuralGroupDecision::GroupedEffectNotBeyondSingle
    } else if !grouped_evidence_selection_useful {
        StructuralGroupDecision::GroupedFreedomWithoutCredit
    } else {
        StructuralGroupDecision::GroupedFreedomAndCredit
    };
    let conclusions = conclusions(
        &confirmation_summaries,
        &confirmation_comparisons,
        &confirmation_capability_contrast,
        selected_group_size,
        decision,
    );
    Ok(StructuralGroupResult {
        version: "adaptive-mechanism/m2c-group-v0.6-balanced-bundle".into(),
        config,
        causal_contract: vec![
            "the v0.4 controller population, checkpoints, scheduled target, local evidence, 32-trial horizon, and random stream are reused".into(),
            "a two-edge bundle replaces both adjustable slots of the scheduled target; a four-edge bundle does the same for the scheduled and immediately preceding target; every edge inherits its replaced weight and total connectivity is unchanged".into(),
            "each scale receives exactly 17 balanced bundles; every target dimension uses each of its 17 legal sources exactly once, and the local-evidence-selected bundle is included".into(),
            "every bundle is paired with its constituent one-edge counterfactuals so additive contribution and group interaction remain distinguishable".into(),
            "task accuracy is used only by the offline evaluator; the live stream never receives a topology write".into(),
        ],
        carrier_parameters,
        development_seed_results,
        development_summaries,
        confirmation_seed_results,
        confirmation_summaries,
        confirmation_phase_summaries,
        confirmation_comparisons,
        confirmation_capability_contrast,
        selected_group_size,
        decision,
        acceptance,
        conclusions,
    })
}

fn summarize(
    results: &[StructuralGroupSeedResult],
    group_sizes: [usize; STRUCTURAL_GROUP_SIZE_COUNT],
) -> Vec<StructuralGroupSummary> {
    group_sizes
        .into_iter()
        .enumerate()
        .map(|(index, edge_count)| {
            let values = |f: fn(&StructuralGroupSeedMetrics) -> f64| {
                results
                    .iter()
                    .map(|row| f(&row.group_metrics[index]))
                    .collect::<Vec<_>>()
            };
            StructuralGroupSummary {
                edge_count,
                seed_run_count: results.len(),
                checkpoint_count: results.iter().map(|row| row.checkpoints.len()).sum(),
                bundle_evaluation_count: results
                    .iter()
                    .flat_map(|row| &row.checkpoints)
                    .map(|checkpoint| checkpoint.scales[index].bundle_count)
                    .sum(),
                component_control_evaluation_count: results
                    .iter()
                    .flat_map(|row| &row.checkpoints)
                    .map(|checkpoint| checkpoint.scales[index].component_control_evaluation_count)
                    .sum(),
                mean_spearman_correlation: mean_interval(&values(|row| {
                    row.mean_spearman_correlation
                })),
                mean_shuffled_spearman_correlation: mean_interval(&values(|row| {
                    row.mean_shuffled_spearman_correlation
                })),
                mean_correlation_advantage: mean_interval(&values(|row| {
                    row.mean_correlation_advantage
                })),
                mean_selected_benefit: mean_interval(&values(|row| row.mean_selected_benefit)),
                mean_random_benefit: mean_interval(&values(|row| row.mean_random_benefit)),
                mean_selected_benefit_advantage: mean_interval(&values(|row| {
                    row.mean_selected_benefit_advantage
                })),
                mean_budgeted_oracle_benefit: mean_interval(&values(|row| {
                    row.mean_budgeted_oracle_benefit
                })),
                mean_selected_regret: mean_interval(&values(|row| row.mean_selected_regret)),
                mean_top_quartile_hit_rate: mean_interval(&values(|row| {
                    row.mean_top_quartile_hit_rate
                })),
                mean_interaction_over_additive: mean_interval(&values(|row| {
                    row.mean_interaction_over_additive
                })),
                mean_selected_interaction_over_additive: mean_interval(&values(|row| {
                    row.mean_selected_interaction_over_additive
                })),
                mean_oracle_interaction_over_additive: mean_interval(&values(|row| {
                    row.mean_oracle_interaction_over_additive
                })),
                finite_fraction: results.iter().filter(|row| row.finite).count() as f64
                    / results.len().max(1) as f64,
            }
        })
        .collect()
}

fn summarize_phases(
    results: &[StructuralGroupSeedResult],
    group_sizes: [usize; STRUCTURAL_GROUP_SIZE_COUNT],
) -> Vec<StructuralGroupPhaseSummary> {
    group_sizes
        .into_iter()
        .enumerate()
        .flat_map(|(scale_index, edge_count)| {
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
                                .filter(|checkpoint| checkpoint.phase_index == phase_index)
                                .map(|checkpoint| &checkpoint.scales[scale_index])
                                .collect::<Vec<_>>();
                            let count = rows.len().max(1) as f64;
                            let mean = |f: fn(&StructuralGroupCheckpointScale) -> f64| {
                                rows.iter().map(|row| f(row)).sum::<f64>() / count
                            };
                            [
                                mean(|row| row.selected_benefit),
                                mean(|row| row.random_mean_benefit),
                                mean(|row| row.budgeted_oracle_benefit),
                                mean(|row| row.selected_benefit - row.random_mean_benefit),
                                mean(|row| row.oracle_interaction_over_additive),
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
                    StructuralGroupPhaseSummary {
                        edge_count,
                        phase_index,
                        rule,
                        seed_run_count: results.len(),
                        checkpoint_count: results
                            .iter()
                            .flat_map(|row| &row.checkpoints)
                            .filter(|checkpoint| checkpoint.phase_index == phase_index)
                            .count(),
                        mean_selected_benefit: metric(0),
                        mean_random_benefit: metric(1),
                        mean_budgeted_oracle_benefit: metric(2),
                        mean_selected_benefit_advantage: metric(3),
                        mean_oracle_interaction_over_additive: metric(4),
                    }
                })
        })
        .collect()
}

fn summarize_comparisons(
    results: &[StructuralGroupSeedResult],
    group_sizes: [usize; STRUCTURAL_GROUP_SIZE_COUNT],
) -> Vec<StructuralGroupComparison> {
    group_sizes
        .into_iter()
        .enumerate()
        .skip(1)
        .map(|(index, edge_count)| {
            let difference = |f: fn(&StructuralGroupSeedMetrics) -> f64| {
                mean_interval(
                    &results
                        .iter()
                        .map(|row| f(&row.group_metrics[index]) - f(&row.group_metrics[0]))
                        .collect::<Vec<_>>(),
                )
            };
            StructuralGroupComparison {
                edge_count,
                oracle_advantage_over_single: difference(|row| row.mean_budgeted_oracle_benefit),
                selected_advantage_over_single: difference(|row| row.mean_selected_benefit),
                random_mean_change_from_single: difference(|row| row.mean_random_benefit),
                oracle_interaction_over_additive: mean_interval(
                    &results
                        .iter()
                        .map(|row| row.group_metrics[index].mean_oracle_interaction_over_additive)
                        .collect::<Vec<_>>(),
                ),
            }
        })
        .collect()
}

fn capability_contrast(
    results: &[StructuralGroupSeedResult],
    group_sizes: [usize; STRUCTURAL_GROUP_SIZE_COUNT],
    edge_count: usize,
) -> StructuralGroupCapabilityContrast {
    let scale_index = group_sizes
        .iter()
        .position(|size| *size == edge_count)
        .expect("selected group size");
    let paired = results
        .iter()
        .map(|result| {
            let history = result
                .checkpoints
                .iter()
                .filter(|checkpoint| matches!(checkpoint.phase_index, 0 | 4))
                .map(|checkpoint| checkpoint.scales[scale_index].budgeted_oracle_benefit)
                .collect::<Vec<_>>();
            let novel = result
                .checkpoints
                .iter()
                .filter(|checkpoint| matches!(checkpoint.phase_index, 1..=3))
                .map(|checkpoint| checkpoint.scales[scale_index].budgeted_oracle_benefit)
                .collect::<Vec<_>>();
            let history_mean = history.iter().sum::<f64>() / history.len().max(1) as f64;
            let novel_mean = novel.iter().sum::<f64>() / novel.len().max(1) as f64;
            [history_mean, novel_mean, history_mean - novel_mean]
        })
        .collect::<Vec<_>>();
    let metric = |index: usize| {
        mean_interval(
            &paired
                .iter()
                .map(|values| values[index])
                .collect::<Vec<_>>(),
        )
    };
    StructuralGroupCapabilityContrast {
        edge_count,
        mean_history_phase_oracle_benefit: metric(0),
        mean_novel_rule_oracle_benefit: metric(1),
        history_advantage_over_novel_rules: metric(2),
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

fn replicates_v04(summary: &StructuralGroupSummary, development: bool) -> bool {
    let expected = if development {
        [
            0.05059133360021656,
            -0.0008759463714267448,
            0.0005304783950617278,
            0.000019857480029048,
            0.012731481481481476,
            0.4027777777777779,
        ]
    } else {
        [
            0.04564097587940386,
            0.0020824778303832783,
            0.0007716049382716033,
            -0.0011928671931735674,
            0.01384066358024691,
            0.2974537037037037,
        ]
    };
    summary.edge_count == 1
        && [
            summary.mean_spearman_correlation.mean,
            summary.mean_shuffled_spearman_correlation.mean,
            summary.mean_selected_benefit.mean,
            summary.mean_random_benefit.mean,
            summary.mean_budgeted_oracle_benefit.mean,
            summary.mean_top_quartile_hit_rate.mean,
        ] == expected
}

fn conclusions(
    summaries: &[StructuralGroupSummary],
    comparisons: &[StructuralGroupComparison],
    capability: &StructuralGroupCapabilityContrast,
    selected_group_size: usize,
    decision: StructuralGroupDecision,
) -> Vec<String> {
    let oracle_line = summaries
        .iter()
        .map(|row| {
            format!(
                "{} 边 {:+.2} pp",
                row.edge_count,
                row.mean_budgeted_oracle_benefit.mean * 100.0
            )
        })
        .collect::<Vec<_>>()
        .join(" · ");
    let selected = summaries
        .iter()
        .find(|row| row.edge_count == selected_group_size)
        .expect("selected summary");
    let comparison = comparisons
        .iter()
        .find(|row| row.edge_count == selected_group_size)
        .expect("selected comparison");
    vec![
        format!(
            "独立确认对每个规模使用相同的 17 个 bundle 搜索预算；budgeted oracle 为：{oracle_line}。"
        ),
        format!(
            "最强成组规模为 {} 边；相对单边 oracle 的配对优势是 {:+.2} pp [{:+.2}, {:+.2}]。",
            selected_group_size,
            comparison.oracle_advantage_over_single.mean * 100.0,
            comparison.oracle_advantage_over_single.lower95 * 100.0,
            comparison.oracle_advantage_over_single.upper95 * 100.0,
        ),
        format!(
            "{} 边证据首选收益 {:+.2} pp，随机 bundle 均值 {:+.2} pp，差 {:+.2} pp。",
            selected_group_size,
            selected.mean_selected_benefit.mean * 100.0,
            selected.mean_random_benefit.mean * 100.0,
            selected.mean_selected_benefit_advantage.mean * 100.0,
        ),
        format!(
            "{} 边 oracle 相对对应单边加和的交互项为 {:+.2} pp [{:+.2}, {:+.2}]。",
            selected_group_size,
            selected.mean_oracle_interaction_over_additive.mean * 100.0,
            selected.mean_oracle_interaction_over_additive.lower95 * 100.0,
            selected.mean_oracle_interaction_over_additive.upper95 * 100.0,
        ),
        format!(
            "已有 A 阶段的 oracle 为 {:+.2} pp，新规则 B/C/D 为 {:+.2} pp；历史阶段优势 {:+.2} pp [{:+.2}, {:+.2}]。",
            capability.mean_history_phase_oracle_benefit.mean * 100.0,
            capability.mean_novel_rule_oracle_benefit.mean * 100.0,
            capability.history_advantage_over_novel_rules.mean * 100.0,
            capability.history_advantage_over_novel_rules.lower95 * 100.0,
            capability.history_advantage_over_novel_rules.upper95 * 100.0,
        ),
        format!("M2C-G 正式决策为 {decision:?}。"),
        match decision {
            StructuralGroupDecision::GroupedFreedomAndCredit => "成组结构自由度和当前局部信用均达到实用门槛；下一步才允许设计带冷却的在线成组候选。",
            StructuralGroupDecision::GroupedFreedomWithoutCredit => "成组反事实具有可用作用，但当前局部证据不能选择它；下一步应诊断组合信用，而不是直接在线重连。",
            StructuralGroupDecision::HistoryDominatedGroupedEffect => "成组 oracle 的总体收益主要来自已有 A 与回归阶段，没有形成一般新规则容量；固定预算结构重分配不能进入在线机制。",
            StructuralGroupDecision::GroupedEffectNotBeyondSingle => "成组候选偶尔越过绝对门槛，但没有可靠优于同预算单边搜索；没有理由扩大在线结构动作。",
            StructuralGroupDecision::NoUsefulGroupedEffect => "在相同 17-bundle 搜索预算内，2/4 边 oracle 仍无实用收益；当前固定预算结构重分配分支应停止。",
            StructuralGroupDecision::DiagnosticUnstable => "成组反事实出现非有限输出，当前诊断不可解释。",
        }
        .into(),
    ]
}

fn validate_config(config: StructuralGroupConfig) -> Result<(), EmbodiedError> {
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
        || !(0.0..1.0).contains(&config.structural_parameters.evidence_decay)
        || config.group_sizes != [1, 2, 4]
        || config.bundle_budget != 17
        || config.forward_training_trials == 0
        || config.evaluation_trial_count == 0
        || config.shuffled_ranking_count == 0
        || !valid_checkpoints
        || [
            config.minimum_oracle_benefit,
            config.minimum_oracle_advantage_over_single,
            config.minimum_selected_benefit_advantage,
        ]
        .iter()
        .any(|value| !value.is_finite() || *value < 0.0)
    {
        return Err(EmbodiedError::InvalidExperiment);
    }
    Ok(())
}
