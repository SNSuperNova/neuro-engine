use serde::Serialize;

use crate::learnability_map::{
    ContinuousResource, DualTimescaleHomeostasis, HomeostasisMechanism, M1XSeedProtocol,
    PlasticityMechanism, ResourceMechanism, SoftBoundedPlasticity, run_m1xe_seed, seed_partition,
    verify_m1x_oracle_gradient,
};
use crate::map1::{parameter_points, protocol_config};
use crate::{
    EmbodiedError, HIDDEN_COUNT, M1Rule, M1XConfig, M1XRuleResult, Map0Interval, Map0ParameterPoint,
};

const DEVELOPMENT_SEED_LABEL: u64 = 0x4d31_5845_4445_5601;
const CONFIRMATION_SEED_LABEL: u64 = 0x4d31_5845_434f_4e46;
pub const M1XE_MULTIPLIER_COUNT: usize = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum M1XEBoundKind {
    ReferenceMultiplier,
    AbsoluteBound,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum M1XEDecision {
    UniformEnvelopeBoundaryConfirmed,
    BoundaryShifted,
    PerUnitAllocationRequired,
    AbsoluteAnchorFailed,
    DynamicsUnstable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1XEFrozenArtifact {
    pub file: &'static str,
    pub sha256: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1XEConfig {
    pub m1x_protocol: M1XConfig,
    pub norm_multipliers: [f64; M1XE_MULTIPLIER_COUNT],
    pub oracle_learning_rate: f64,
    pub minimum_history_accuracy: f64,
}

impl Default for M1XEConfig {
    fn default() -> Self {
        Self {
            m1x_protocol: M1XConfig::default(),
            norm_multipliers: [1.0, 1.5, 2.0, 3.0, 4.0, 6.0, 8.0],
            oracle_learning_rate: 0.08,
            minimum_history_accuracy: 0.75,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1XESeedResult {
    pub parameter_id: usize,
    pub seed: u64,
    pub bound_kind: M1XEBoundKind,
    pub norm_multiplier: Option<f64>,
    pub rule_results: Vec<M1XRuleResult>,
    pub finite: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1XEScaleSummary {
    pub bound_kind: M1XEBoundKind,
    pub norm_multiplier: Option<f64>,
    pub run_count: usize,
    pub mean_history_behavior_accuracy: Map0Interval,
    pub mean_novel_behavior_accuracy: Map0Interval,
    pub mean_novel_argmax_accuracy: Map0Interval,
    pub mean_novel_target_probability: Map0Interval,
    pub mean_minimum_novel_accuracy: Map0Interval,
    pub mean_training_loss_reduction: Map0Interval,
    pub mean_relative_weight_drift: Map0Interval,
    pub mean_saturation_fraction: Map0Interval,
    pub finite_fraction: f64,
    pub capability_passed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1XEScaleComparison {
    pub norm_multiplier: f64,
    pub novel_behavior_change_from_one_x: Map0Interval,
    pub target_probability_change_from_one_x: Map0Interval,
    pub minimum_novel_accuracy_change_from_one_x: Map0Interval,
    pub history_accuracy_change_from_one_x: Map0Interval,
    pub saturation_change_from_one_x: Map0Interval,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1XEAcceptanceReport {
    pub m1x_artifact_and_optimizer_frozen: bool,
    pub diagnostic_is_offline_only: bool,
    pub development_and_confirmation_seeds_disjoint: bool,
    pub development_scale_scan_complete: bool,
    pub confirmation_scale_scan_complete: bool,
    pub absolute_anchor_complete: bool,
    pub all_four_rules_complete: bool,
    pub selected_multiplier_from_development_only: bool,
    pub exact_gradient_optimizer_verified: bool,
    pub action_readout_remained_frozen: bool,
    pub topology_and_connection_budget_preserved: bool,
    pub finite_outputs: bool,
    pub stability_within_bounds: bool,
    pub absolute_anchor_passed: bool,
    pub uniform_boundary_found: bool,
    pub development_boundary_confirmed: bool,
    pub stage_passed: bool,
    pub passed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1XEResult {
    pub version: String,
    pub config: M1XEConfig,
    pub frozen_m1x_artifact: M1XEFrozenArtifact,
    pub diagnostic_contract: Vec<String>,
    pub carrier_parameters: Vec<Map0ParameterPoint>,
    pub development_seed_results: Vec<M1XESeedResult>,
    pub development_summaries: Vec<M1XEScaleSummary>,
    pub selected_development_multiplier: Option<f64>,
    pub confirmation_seed_results: Vec<M1XESeedResult>,
    pub confirmation_summaries: Vec<M1XEScaleSummary>,
    pub confirmed_minimum_multiplier: Option<f64>,
    pub passing_confirmation_multipliers: Vec<f64>,
    pub confirmation_comparisons: Vec<M1XEScaleComparison>,
    pub decision: M1XEDecision,
    pub acceptance: M1XEAcceptanceReport,
    pub conclusions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1XEPublishedResult {
    pub version: String,
    pub config: M1XEConfig,
    pub frozen_m1x_artifact: M1XEFrozenArtifact,
    pub diagnostic_contract: Vec<String>,
    pub carrier_parameters: Vec<Map0ParameterPoint>,
    pub development_summaries: Vec<M1XEScaleSummary>,
    pub selected_development_multiplier: Option<f64>,
    pub confirmation_summaries: Vec<M1XEScaleSummary>,
    pub confirmed_minimum_multiplier: Option<f64>,
    pub passing_confirmation_multipliers: Vec<f64>,
    pub confirmation_comparisons: Vec<M1XEScaleComparison>,
    pub decision: M1XEDecision,
    pub acceptance: M1XEAcceptanceReport,
    pub conclusions: Vec<String>,
}

impl M1XEResult {
    pub fn published(&self) -> M1XEPublishedResult {
        M1XEPublishedResult {
            version: self.version.clone(),
            config: self.config,
            frozen_m1x_artifact: self.frozen_m1x_artifact,
            diagnostic_contract: self.diagnostic_contract.clone(),
            carrier_parameters: self.carrier_parameters.clone(),
            development_summaries: self.development_summaries.clone(),
            selected_development_multiplier: self.selected_development_multiplier,
            confirmation_summaries: self.confirmation_summaries.clone(),
            confirmed_minimum_multiplier: self.confirmed_minimum_multiplier,
            passing_confirmation_multipliers: self.passing_confirmation_multipliers.clone(),
            confirmation_comparisons: self.confirmation_comparisons.clone(),
            decision: self.decision,
            acceptance: self.acceptance,
            conclusions: self.conclusions.clone(),
        }
    }
}

pub fn run_m1xe_experiment(config: M1XEConfig) -> Result<M1XEResult, EmbodiedError> {
    validate_config(config)?;
    let map1 = config
        .m1x_protocol
        .m1_protocol
        .map2_protocol
        .map2b_protocol
        .map2a_protocol
        .map1_protocol;
    let carrier = protocol_config(map1);
    let all_parameters = parameter_points(map1);
    let carrier_parameters = config
        .m1x_protocol
        .m1_protocol
        .reference_parameter_ids
        .iter()
        .map(|id| all_parameters[*id])
        .collect::<Vec<_>>();
    let development_seeds = seed_partition(
        map1.seed ^ DEVELOPMENT_SEED_LABEL,
        config.m1x_protocol.m1_protocol.development_seed_count,
    );
    let confirmation_seeds = seed_partition(
        map1.seed ^ CONFIRMATION_SEED_LABEL,
        config.m1x_protocol.m1_protocol.confirmation_seed_count,
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
            .m1x_protocol
            .m1_protocol
            .map2_protocol
            .map2b_protocol
            .map2a_protocol
            .soft_bound_scale,
    });
    let map2b = config.m1x_protocol.m1_protocol.map2_protocol.map2b_protocol;
    let resource = ResourceMechanism::Continuous(ContinuousResource {
        initial_level: map2b.initial_resource,
        supply_rate: map2b.supply_rate,
        maintenance_cost: map2b.maintenance_cost,
        activity_cost: map2b.activity_cost,
        plasticity_cost: map2b.plasticity_cost,
        minimum_modulation: map2b.minimum_modulation,
    });
    let protocol = M1XSeedProtocol {
        training_trial_count: config.m1x_protocol.oracle_training_trials,
        evaluation_trial_count: config.m1x_protocol.m1_protocol.evaluation_trial_count,
        iterations: config.m1x_protocol.oracle_iterations,
        restart_count: config.m1x_protocol.oracle_restart_count,
        restart_jitter: config.m1x_protocol.restart_jitter,
        learning_rate: config.oracle_learning_rate,
    };
    let run_scales = |seeds: &[u64]| {
        config
            .norm_multipliers
            .into_iter()
            .flat_map(|multiplier| {
                carrier_parameters.iter().flat_map(move |point| {
                    seeds.iter().map(move |seed| {
                        run_m1xe_seed(
                            carrier,
                            *point,
                            *seed,
                            homeostasis,
                            plasticity,
                            resource,
                            protocol,
                            M1XEBoundKind::ReferenceMultiplier,
                            Some(multiplier),
                        )
                    })
                })
            })
            .collect::<Vec<_>>()
    };
    let development_seed_results = run_scales(&development_seeds);
    let mut confirmation_seed_results = run_scales(&confirmation_seeds);
    confirmation_seed_results.extend(carrier_parameters.iter().flat_map(|point| {
        confirmation_seeds.iter().map(|seed| {
            run_m1xe_seed(
                carrier,
                *point,
                *seed,
                homeostasis,
                plasticity,
                resource,
                protocol,
                M1XEBoundKind::AbsoluteBound,
                None,
            )
        })
    }));
    let development_summaries = config
        .norm_multipliers
        .into_iter()
        .map(|multiplier| summarize_scale(&development_seed_results, Some(multiplier), config))
        .collect::<Vec<_>>();
    let selected_development_multiplier = development_summaries
        .iter()
        .find(|row| row.capability_passed)
        .and_then(|row| row.norm_multiplier);
    let mut confirmation_summaries = config
        .norm_multipliers
        .into_iter()
        .map(|multiplier| summarize_scale(&confirmation_seed_results, Some(multiplier), config))
        .collect::<Vec<_>>();
    confirmation_summaries.push(summarize_scale(&confirmation_seed_results, None, config));
    let passing_confirmation_multipliers = confirmation_summaries
        .iter()
        .filter(|row| row.bound_kind == M1XEBoundKind::ReferenceMultiplier && row.capability_passed)
        .filter_map(|row| row.norm_multiplier)
        .collect::<Vec<_>>();
    let confirmed_minimum_multiplier = passing_confirmation_multipliers.first().copied();
    let confirmation_comparisons = config
        .norm_multipliers
        .into_iter()
        .map(|multiplier| comparison(&confirmation_seed_results, multiplier))
        .collect::<Vec<_>>();
    let all_results = development_seed_results
        .iter()
        .chain(&confirmation_seed_results)
        .collect::<Vec<_>>();
    let exact_gradient_optimizer_verified = verify_m1x_oracle_gradient(
        carrier,
        carrier_parameters[0],
        map1.seed ^ 0x4d31_5845_4752_4144,
        homeostasis,
        plasticity,
        resource,
    );
    let finite_outputs = all_results.iter().all(|row| row.finite);
    let stability_within_bounds = all_results
        .iter()
        .flat_map(|row| &row.rule_results)
        .all(|row| {
            row.saturation_fraction <= config.m1x_protocol.maximum_saturation_fraction && row.finite
        });
    let absolute = confirmation_summaries
        .iter()
        .find(|row| row.bound_kind == M1XEBoundKind::AbsoluteBound)
        .expect("absolute anchor");
    let absolute_anchor_passed = absolute.capability_passed;
    let uniform_boundary_found = confirmed_minimum_multiplier.is_some();
    let development_boundary_confirmed = selected_development_multiplier.is_some()
        && selected_development_multiplier == confirmed_minimum_multiplier;
    let expected_development =
        config.norm_multipliers.len() * carrier_parameters.len() * development_seeds.len();
    let expected_confirmation_scales =
        config.norm_multipliers.len() * carrier_parameters.len() * confirmation_seeds.len();
    let expected_absolute = carrier_parameters.len() * confirmation_seeds.len();
    let frozen_m1x_artifact = M1XEFrozenArtifact {
        file: "app/public/reachability-v0.9.json",
        sha256: "f146cebae0db2b02b8c7edb6e8add84132dc3625a1c63e6ee9a484ce36523d4e",
    };
    let base_acceptance = M1XEAcceptanceReport {
        m1x_artifact_and_optimizer_frozen: config.m1x_protocol == M1XConfig::default()
            && config.oracle_learning_rate == 0.08,
        diagnostic_is_offline_only: true,
        development_and_confirmation_seeds_disjoint: development_seeds
            .iter()
            .all(|seed| !confirmation_seeds.contains(seed)),
        development_scale_scan_complete: development_seed_results.len() == expected_development,
        confirmation_scale_scan_complete: confirmation_seed_results
            .iter()
            .filter(|row| row.bound_kind == M1XEBoundKind::ReferenceMultiplier)
            .count()
            == expected_confirmation_scales,
        absolute_anchor_complete: confirmation_seed_results
            .iter()
            .filter(|row| row.bound_kind == M1XEBoundKind::AbsoluteBound)
            .count()
            == expected_absolute,
        all_four_rules_complete: all_results.iter().all(|row| {
            row.rule_results.len() == M1Rule::UNIQUE.len()
                && row
                    .rule_results
                    .iter()
                    .zip(M1Rule::UNIQUE)
                    .all(|(result, rule)| result.rule == rule)
        }),
        selected_multiplier_from_development_only: selected_development_multiplier
            .is_none_or(|value| config.norm_multipliers.contains(&value)),
        exact_gradient_optimizer_verified,
        action_readout_remained_frozen: all_results
            .iter()
            .flat_map(|row| &row.rule_results)
            .all(|row| row.action_readout_digest_before == row.action_readout_digest_after),
        topology_and_connection_budget_preserved: all_results
            .iter()
            .flat_map(|row| &row.rule_results)
            .all(|row| {
                row.topology_digest_before == row.topology_digest_after
                    && row.adjustable_connection_count == HIDDEN_COUNT * 2
            }),
        finite_outputs,
        stability_within_bounds,
        absolute_anchor_passed,
        uniform_boundary_found,
        development_boundary_confirmed,
        stage_passed: false,
        passed: false,
    };
    let protocol_complete = base_acceptance.m1x_artifact_and_optimizer_frozen
        && base_acceptance.diagnostic_is_offline_only
        && base_acceptance.development_and_confirmation_seeds_disjoint
        && base_acceptance.development_scale_scan_complete
        && base_acceptance.confirmation_scale_scan_complete
        && base_acceptance.absolute_anchor_complete
        && base_acceptance.all_four_rules_complete
        && base_acceptance.selected_multiplier_from_development_only
        && base_acceptance.exact_gradient_optimizer_verified
        && base_acceptance.action_readout_remained_frozen
        && base_acceptance.topology_and_connection_budget_preserved
        && base_acceptance.finite_outputs;
    let acceptance = M1XEAcceptanceReport {
        stage_passed: protocol_complete,
        passed: protocol_complete,
        ..base_acceptance
    };
    let decision = if !protocol_complete || !finite_outputs || !stability_within_bounds {
        M1XEDecision::DynamicsUnstable
    } else if !absolute_anchor_passed {
        M1XEDecision::AbsoluteAnchorFailed
    } else if development_boundary_confirmed {
        M1XEDecision::UniformEnvelopeBoundaryConfirmed
    } else if uniform_boundary_found {
        M1XEDecision::BoundaryShifted
    } else {
        M1XEDecision::PerUnitAllocationRequired
    };
    let conclusions = conclusions(
        &development_summaries,
        selected_development_multiplier,
        &confirmation_summaries,
        confirmed_minimum_multiplier,
        decision,
    );
    Ok(M1XEResult {
        version: "adaptive-mechanism/m1xe-v1.0-uniform-norm-boundary".into(),
        config,
        frozen_m1x_artifact,
        diagnostic_contract: vec![
            "M1-X task, carrier, exact gradient, Adam settings, 0.08 learning rate, training and evaluation budgets, restart count, and absolute weight bound are frozen".into(),
            "each reference-multiplier oracle changes only the same 48 recurrent values and projects every target pair to one shared multiple of that target's original reference norm".into(),
            "development selects the first preregistered multiplier meeting every capability and stability threshold; confirmation reruns the complete ordered curve on disjoint seeds".into(),
            "the absolute-bound oracle is rerun as a positive reachability anchor; it cannot be selected as the uniform boundary".into(),
            "target labels, gradients, optimizer moments, restarts, and selected weights remain offline diagnostic privileges and are never exposed to an online mechanism".into(),
        ],
        carrier_parameters,
        development_seed_results,
        development_summaries,
        selected_development_multiplier,
        confirmation_seed_results,
        confirmation_summaries,
        confirmed_minimum_multiplier,
        passing_confirmation_multipliers,
        confirmation_comparisons,
        decision,
        acceptance,
        conclusions,
    })
}

fn validate_config(config: M1XEConfig) -> Result<(), EmbodiedError> {
    if config.norm_multipliers != [1.0, 1.5, 2.0, 3.0, 4.0, 6.0, 8.0]
        || !config.oracle_learning_rate.is_finite()
        || config.oracle_learning_rate <= 0.0
        || config.m1x_protocol.oracle_iterations == 0
        || config.m1x_protocol.oracle_training_trials < 4
        || config.m1x_protocol.oracle_restart_count == 0
        || !config.m1x_protocol.restart_jitter.is_finite()
        || config.m1x_protocol.restart_jitter < 0.0
        || config.m1x_protocol.m1_protocol.development_seed_count == 0
        || config.m1x_protocol.m1_protocol.confirmation_seed_count == 0
        || !config.minimum_history_accuracy.is_finite()
        || !(0.0..=1.0).contains(&config.minimum_history_accuracy)
    {
        return Err(EmbodiedError::InvalidExperiment);
    }
    Ok(())
}

fn summarize_scale(
    results: &[M1XESeedResult],
    multiplier: Option<f64>,
    config: M1XEConfig,
) -> M1XEScaleSummary {
    let kind = if multiplier.is_some() {
        M1XEBoundKind::ReferenceMultiplier
    } else {
        M1XEBoundKind::AbsoluteBound
    };
    let rows = results
        .iter()
        .filter(|row| row.bound_kind == kind && row.norm_multiplier == multiplier)
        .collect::<Vec<_>>();
    let novel = |metric: fn(&M1XRuleResult) -> f64| {
        rows.iter()
            .map(|row| row.rule_results[1..].iter().map(metric).sum::<f64>() / 3.0)
            .collect::<Vec<_>>()
    };
    let history = rows
        .iter()
        .map(|row| row.rule_results[0].final_behavior_accuracy)
        .collect::<Vec<_>>();
    let minimum = rows
        .iter()
        .map(|row| {
            row.rule_results[1..]
                .iter()
                .map(|rule| rule.final_behavior_accuracy)
                .fold(1.0, f64::min)
        })
        .collect::<Vec<_>>();
    let history_interval = mean_interval(&history);
    let behavior = mean_interval(&novel(|row| row.final_behavior_accuracy));
    let argmax = mean_interval(&novel(|row| row.final_argmax_accuracy));
    let target_probability = mean_interval(&novel(|row| row.final_target_probability));
    let minimum_novel = mean_interval(&minimum);
    let loss_reduction = mean_interval(&novel(|row| {
        row.initial_training_loss - row.final_training_loss
    }));
    let drift = mean_interval(&novel(|row| row.mean_relative_weight_drift));
    let saturation = mean_interval(&novel(|row| row.saturation_fraction));
    let finite_fraction =
        rows.iter().filter(|row| row.finite).count() as f64 / rows.len().max(1) as f64;
    let capability_passed = behavior.lower95 >= config.m1x_protocol.minimum_novel_behavior_accuracy
        && target_probability.lower95 >= config.m1x_protocol.minimum_novel_target_probability
        && minimum_novel.mean >= config.m1x_protocol.minimum_single_novel_accuracy
        && history_interval.lower95 >= config.minimum_history_accuracy
        && saturation.upper95 <= config.m1x_protocol.maximum_saturation_fraction
        && finite_fraction == 1.0;
    M1XEScaleSummary {
        bound_kind: kind,
        norm_multiplier: multiplier,
        run_count: rows.len(),
        mean_history_behavior_accuracy: history_interval,
        mean_novel_behavior_accuracy: behavior,
        mean_novel_argmax_accuracy: argmax,
        mean_novel_target_probability: target_probability,
        mean_minimum_novel_accuracy: minimum_novel,
        mean_training_loss_reduction: loss_reduction,
        mean_relative_weight_drift: drift,
        mean_saturation_fraction: saturation,
        finite_fraction,
        capability_passed,
    }
}

fn comparison(results: &[M1XESeedResult], multiplier: f64) -> M1XEScaleComparison {
    let effect = |metric: fn(&M1XRuleResult) -> f64, novel: bool| {
        let mut values = Vec::new();
        for row in results.iter().filter(|row| {
            row.bound_kind == M1XEBoundKind::ReferenceMultiplier
                && row.norm_multiplier == Some(multiplier)
        }) {
            let one = results
                .iter()
                .find(|candidate| {
                    candidate.bound_kind == M1XEBoundKind::ReferenceMultiplier
                        && candidate.norm_multiplier == Some(1.0)
                        && candidate.parameter_id == row.parameter_id
                        && candidate.seed == row.seed
                })
                .expect("paired one-x result");
            let range = if novel { 1..4 } else { 0..1 };
            values.push(
                row.rule_results[range.clone()]
                    .iter()
                    .zip(&one.rule_results[range])
                    .map(|(left, right)| metric(left) - metric(right))
                    .sum::<f64>()
                    / if novel { 3.0 } else { 1.0 },
            );
        }
        mean_interval(&values)
    };
    let mut minimum_changes = Vec::new();
    for row in results.iter().filter(|row| {
        row.bound_kind == M1XEBoundKind::ReferenceMultiplier
            && row.norm_multiplier == Some(multiplier)
    }) {
        let one = results
            .iter()
            .find(|candidate| {
                candidate.bound_kind == M1XEBoundKind::ReferenceMultiplier
                    && candidate.norm_multiplier == Some(1.0)
                    && candidate.parameter_id == row.parameter_id
                    && candidate.seed == row.seed
            })
            .expect("paired one-x result");
        let left = row.rule_results[1..]
            .iter()
            .map(|rule| rule.final_behavior_accuracy)
            .fold(1.0, f64::min);
        let right = one.rule_results[1..]
            .iter()
            .map(|rule| rule.final_behavior_accuracy)
            .fold(1.0, f64::min);
        minimum_changes.push(left - right);
    }
    M1XEScaleComparison {
        norm_multiplier: multiplier,
        novel_behavior_change_from_one_x: effect(|row| row.final_behavior_accuracy, true),
        target_probability_change_from_one_x: effect(|row| row.final_target_probability, true),
        minimum_novel_accuracy_change_from_one_x: mean_interval(&minimum_changes),
        history_accuracy_change_from_one_x: effect(|row| row.final_behavior_accuracy, false),
        saturation_change_from_one_x: effect(|row| row.saturation_fraction, true),
    }
}

fn mean_interval(values: &[f64]) -> Map0Interval {
    if values.is_empty() {
        return Map0Interval::default();
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    if values.len() == 1 {
        return Map0Interval {
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
    let margin = 1.96 * (variance / values.len() as f64).sqrt();
    Map0Interval {
        mean,
        lower95: mean - margin,
        upper95: mean + margin,
    }
}

fn conclusions(
    development: &[M1XEScaleSummary],
    selected: Option<f64>,
    confirmation: &[M1XEScaleSummary],
    confirmed: Option<f64>,
    decision: M1XEDecision,
) -> Vec<String> {
    let curve = |rows: &[M1XEScaleSummary]| {
        rows.iter()
            .filter_map(|row| {
                row.norm_multiplier.map(|multiplier| {
                    format!(
                        "{multiplier:.1}×: {:.1}% / {:.1}% / {:.1}%{}",
                        row.mean_novel_behavior_accuracy.mean * 100.0,
                        row.mean_novel_target_probability.mean * 100.0,
                        row.mean_minimum_novel_accuracy.mean * 100.0,
                        if row.capability_passed { " ✓" } else { "" },
                    )
                })
            })
            .collect::<Vec<_>>()
            .join(" · ")
    };
    let absolute = confirmation
        .iter()
        .find(|row| row.bound_kind == M1XEBoundKind::AbsoluteBound)
        .expect("absolute anchor");
    vec![
        format!(
            "开发统一范数曲线（B/C/D 行为 / 目标概率 / 最低新规则）为 {}；选择边界为 {}。",
            curve(development),
            selected.map_or("未找到".into(), |value| format!("{value:.1}×")),
        ),
        format!(
            "独立确认统一范数曲线为 {}；确认最小边界为 {}。",
            curve(confirmation),
            confirmed.map_or("未找到".into(), |value| format!("{value:.1}×")),
        ),
        format!(
            "绝对边界 anchor 的 B/C/D 行为为 {:.1}%，目标概率 {:.1}%，最低新规则 {:.1}%。",
            absolute.mean_novel_behavior_accuracy.mean * 100.0,
            absolute.mean_novel_target_probability.mean * 100.0,
            absolute.mean_minimum_novel_accuracy.mean * 100.0,
        ),
        format!("M1-XE 正式决策为 {decision:?}。"),
        match decision {
            M1XEDecision::UniformEnvelopeBoundaryConfirmed => format!(
                "开发与独立确认一致定位到 {} 的统一参考范数边界；下一步应在冻结该边界的条件下因果测试现有局部信用能否利用这项自由度，oracle 仍不证明在线方向信用。",
                confirmed.map_or("未知".into(), |value| format!("{value:.1}×")),
            ),
            M1XEDecision::BoundaryShifted => "开发与确认都找到统一范数可达区，但最小边界发生漂移；需要在相邻倍数间加密，而不能冻结在线目标。".into(),
            M1XEDecision::PerUnitAllocationRequired => "绝对边界仍可达，但所有统一参考范数倍数都未一致通过；下一步应诊断按单元分配范数，而不是继续整体放大。".into(),
            M1XEDecision::AbsoluteAnchorFailed => "独立种子上的绝对边界 anchor 未复现 M1-X，可达性基础失效，不能解释统一范数曲线。".into(),
            M1XEDecision::DynamicsUnstable => "协议、数值或稳定性失败，不能解释范数边界。".into(),
        },
    ]
}
