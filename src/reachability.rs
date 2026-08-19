use serde::Serialize;

use crate::learnability_map::{
    ContinuousResource, DualTimescaleHomeostasis, HomeostasisMechanism, M1XSeedProtocol,
    PlasticityMechanism, ResourceMechanism, SoftBoundedPlasticity, run_m1x_seed, seed_partition,
    verify_m1x_oracle_gradient,
};
use crate::map1::{parameter_points, protocol_config};
use crate::{
    EmbodiedError, HIDDEN_COUNT, M1ExperimentConfig, M1Rule, Map0Interval, Map0ParameterPoint,
};

const DEVELOPMENT_SEED_LABEL: u64 = 0x4d31_585f_4445_5601;
const CONFIRMATION_SEED_LABEL: u64 = 0x4d31_585f_434f_4e46;
pub const M1X_LEARNING_RATE_COUNT: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum M1XControl {
    FrozenWeights,
    NormEnvelopeOracle,
    AbsoluteBoundOracle,
    ShuffledTargetOracle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum M1XDecision {
    ReachableWithinCarrierEnvelope,
    ReachableOnlyAtAbsoluteBounds,
    FixedSubspaceUnreached,
    OptimizationInconclusive,
    DynamicsUnstable,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1XConfig {
    pub m1_protocol: M1ExperimentConfig,
    pub learning_rates: [f64; M1X_LEARNING_RATE_COUNT],
    pub oracle_iterations: usize,
    pub oracle_training_trials: usize,
    pub oracle_restart_count: usize,
    pub restart_jitter: f64,
    pub minimum_novel_behavior_accuracy: f64,
    pub minimum_novel_target_probability: f64,
    pub minimum_single_novel_accuracy: f64,
    pub minimum_oracle_advantage: f64,
    pub minimum_loss_reduction: f64,
    pub maximum_saturation_fraction: f64,
}

impl Default for M1XConfig {
    fn default() -> Self {
        Self {
            m1_protocol: M1ExperimentConfig::default(),
            learning_rates: [0.01, 0.03, 0.08],
            oracle_iterations: 120,
            oracle_training_trials: 32,
            oracle_restart_count: 3,
            restart_jitter: 0.08,
            minimum_novel_behavior_accuracy: 0.70,
            minimum_novel_target_probability: 0.70,
            minimum_single_novel_accuracy: 0.65,
            minimum_oracle_advantage: 0.10,
            minimum_loss_reduction: 0.10,
            maximum_saturation_fraction: 0.25,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1XRuleResult {
    pub rule: M1Rule,
    pub initial_behavior_accuracy: f64,
    pub initial_target_probability: f64,
    pub final_behavior_accuracy: f64,
    pub final_argmax_accuracy: f64,
    pub final_target_probability: f64,
    pub initial_training_loss: f64,
    pub final_training_loss: f64,
    pub selected_restart: usize,
    pub mean_relative_weight_drift: f64,
    pub maximum_absolute_weight: f64,
    pub saturation_fraction: f64,
    pub action_readout_digest_before: u64,
    pub action_readout_digest_after: u64,
    pub topology_digest_before: u64,
    pub topology_digest_after: u64,
    pub adjustable_connection_count: usize,
    pub finite: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1XSeedResult {
    pub parameter_id: usize,
    pub seed: u64,
    pub control: M1XControl,
    pub learning_rate: f64,
    pub rule_results: Vec<M1XRuleResult>,
    pub finite: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1XLearningRateSummary {
    pub learning_rate: f64,
    pub run_count: usize,
    pub mean_novel_behavior_accuracy: Map0Interval,
    pub mean_novel_argmax_accuracy: Map0Interval,
    pub mean_novel_target_probability: Map0Interval,
    pub mean_minimum_novel_accuracy: Map0Interval,
    pub mean_training_loss_reduction: Map0Interval,
    pub mean_saturation_fraction: Map0Interval,
    pub finite_fraction: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1XControlSummary {
    pub control: M1XControl,
    pub learning_rate: f64,
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
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1XPairedEffects {
    pub absolute_over_frozen_behavior: Map0Interval,
    pub absolute_over_shuffled_behavior: Map0Interval,
    pub absolute_over_frozen_target_probability: Map0Interval,
    pub absolute_over_shuffled_target_probability: Map0Interval,
    pub absolute_over_envelope_behavior: Map0Interval,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1XAcceptanceReport {
    pub m1_task_carrier_and_readout_frozen: bool,
    pub diagnostic_is_offline_only: bool,
    pub development_and_confirmation_seeds_disjoint: bool,
    pub development_learning_rate_scan_complete: bool,
    pub confirmation_controls_complete: bool,
    pub all_four_rules_complete: bool,
    pub selected_learning_rate_from_development_only: bool,
    pub exact_gradient_optimizer_verified: bool,
    pub matched_search_budgets: bool,
    pub action_readout_remained_frozen: bool,
    pub topology_and_connection_budget_preserved: bool,
    pub finite_outputs: bool,
    pub stability_within_bounds: bool,
    pub optimizer_effective: bool,
    pub task_specificity_passed: bool,
    pub envelope_reachability_passed: bool,
    pub absolute_reachability_passed: bool,
    pub stage_passed: bool,
    pub passed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1XResult {
    pub version: String,
    pub config: M1XConfig,
    pub diagnostic_contract: Vec<String>,
    pub carrier_parameters: Vec<Map0ParameterPoint>,
    pub development_seed_results: Vec<M1XSeedResult>,
    pub development_learning_rate_summaries: Vec<M1XLearningRateSummary>,
    pub selected_learning_rate: f64,
    pub confirmation_seed_results: Vec<M1XSeedResult>,
    pub confirmation_summaries: Vec<M1XControlSummary>,
    pub paired_effects: M1XPairedEffects,
    pub decision: M1XDecision,
    pub acceptance: M1XAcceptanceReport,
    pub conclusions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1XPublishedResult {
    pub version: String,
    pub config: M1XConfig,
    pub diagnostic_contract: Vec<String>,
    pub carrier_parameters: Vec<Map0ParameterPoint>,
    pub development_learning_rate_summaries: Vec<M1XLearningRateSummary>,
    pub selected_learning_rate: f64,
    pub confirmation_summaries: Vec<M1XControlSummary>,
    pub paired_effects: M1XPairedEffects,
    pub decision: M1XDecision,
    pub acceptance: M1XAcceptanceReport,
    pub conclusions: Vec<String>,
}

impl M1XResult {
    pub fn published(&self) -> M1XPublishedResult {
        M1XPublishedResult {
            version: self.version.clone(),
            config: self.config,
            diagnostic_contract: self.diagnostic_contract.clone(),
            carrier_parameters: self.carrier_parameters.clone(),
            development_learning_rate_summaries: self.development_learning_rate_summaries.clone(),
            selected_learning_rate: self.selected_learning_rate,
            confirmation_summaries: self.confirmation_summaries.clone(),
            paired_effects: self.paired_effects,
            decision: self.decision,
            acceptance: self.acceptance,
            conclusions: self.conclusions.clone(),
        }
    }
}

pub fn run_m1x_experiment(config: M1XConfig) -> Result<M1XResult, EmbodiedError> {
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
    let protocol = |learning_rate| M1XSeedProtocol {
        training_trial_count: config.oracle_training_trials,
        evaluation_trial_count: config.m1_protocol.evaluation_trial_count,
        iterations: config.oracle_iterations,
        restart_count: config.oracle_restart_count,
        restart_jitter: config.restart_jitter,
        learning_rate,
    };
    let development_seed_results = config
        .learning_rates
        .into_iter()
        .flat_map(|learning_rate| {
            carrier_parameters.iter().flat_map({
                let development_seeds = &development_seeds;
                move |point| {
                    development_seeds.iter().map(move |seed| {
                        run_m1x_seed(
                            carrier,
                            *point,
                            *seed,
                            homeostasis,
                            plasticity,
                            resource,
                            protocol(learning_rate),
                            M1XControl::AbsoluteBoundOracle,
                        )
                    })
                }
            })
        })
        .collect::<Vec<_>>();
    let development_learning_rate_summaries = config
        .learning_rates
        .into_iter()
        .map(|learning_rate| summarize_learning_rate(&development_seed_results, learning_rate))
        .collect::<Vec<_>>();
    let selected_learning_rate = development_learning_rate_summaries
        .iter()
        .max_by(|left, right| {
            learning_rate_rank(left)
                .total_cmp(&learning_rate_rank(right))
                .then_with(|| right.learning_rate.total_cmp(&left.learning_rate))
        })
        .expect("three learning rates")
        .learning_rate;
    let confirmation_seed_results = M1XControl::all()
        .into_iter()
        .flat_map(|control| {
            carrier_parameters.iter().flat_map({
                let confirmation_seeds = &confirmation_seeds;
                move |point| {
                    confirmation_seeds.iter().map(move |seed| {
                        run_m1x_seed(
                            carrier,
                            *point,
                            *seed,
                            homeostasis,
                            plasticity,
                            resource,
                            protocol(selected_learning_rate),
                            control,
                        )
                    })
                }
            })
        })
        .collect::<Vec<_>>();
    let confirmation_summaries = M1XControl::all()
        .into_iter()
        .map(|control| summarize_control(&confirmation_seed_results, control))
        .collect::<Vec<_>>();
    let paired_effects = paired_effects(&confirmation_seed_results);
    let all_results = development_seed_results
        .iter()
        .chain(&confirmation_seed_results)
        .collect::<Vec<_>>();
    let summary = |control| {
        confirmation_summaries
            .iter()
            .find(|row| row.control == control)
            .expect("all controls summarized")
    };
    let absolute = summary(M1XControl::AbsoluteBoundOracle);
    let envelope = summary(M1XControl::NormEnvelopeOracle);
    let finite_outputs = all_results.iter().all(|row| row.finite);
    let exact_gradient_optimizer_verified = verify_m1x_oracle_gradient(
        carrier,
        carrier_parameters[0],
        map1.seed ^ 0x4d31_585f_4752_4144,
        homeostasis,
        plasticity,
        resource,
    );
    let stability_within_bounds = all_results
        .iter()
        .flat_map(|row| &row.rule_results)
        .all(|row| row.saturation_fraction <= config.maximum_saturation_fraction && row.finite);
    let optimizer_effective = absolute.mean_training_loss_reduction.mean
        >= config.minimum_loss_reduction
        && absolute.mean_training_loss_reduction.lower95 > 0.0;
    let task_specificity_passed = paired_effects.absolute_over_shuffled_behavior.mean
        >= config.minimum_oracle_advantage
        && paired_effects.absolute_over_shuffled_behavior.lower95 > 0.0
        && paired_effects
            .absolute_over_shuffled_target_probability
            .lower95
            > 0.0;
    let reachable = |row: &M1XControlSummary| {
        row.mean_novel_behavior_accuracy.mean >= config.minimum_novel_behavior_accuracy
            && row.mean_novel_target_probability.mean >= config.minimum_novel_target_probability
            && row.mean_minimum_novel_accuracy.mean >= config.minimum_single_novel_accuracy
    };
    let envelope_reachability_passed = reachable(envelope);
    let absolute_reachability_passed = reachable(absolute)
        && paired_effects.absolute_over_frozen_behavior.mean >= config.minimum_oracle_advantage
        && paired_effects.absolute_over_frozen_behavior.lower95 > 0.0
        && task_specificity_passed;
    let base_acceptance = M1XAcceptanceReport {
        m1_task_carrier_and_readout_frozen: config.m1_protocol.reference_parameter_ids
            == M1ExperimentConfig::default().reference_parameter_ids
            && config.m1_protocol.phase_trial_count
                == M1ExperimentConfig::default().phase_trial_count
            && config.m1_protocol.evaluation_trial_count
                == M1ExperimentConfig::default().evaluation_trial_count,
        diagnostic_is_offline_only: true,
        development_and_confirmation_seeds_disjoint: development_seeds
            .iter()
            .all(|seed| !confirmation_seeds.contains(seed)),
        development_learning_rate_scan_complete: development_seed_results.len()
            == config.learning_rates.len() * carrier_parameters.len() * development_seeds.len(),
        confirmation_controls_complete: confirmation_seed_results.len()
            == M1XControl::all().len() * carrier_parameters.len() * confirmation_seeds.len(),
        all_four_rules_complete: all_results.iter().all(|row| {
            row.rule_results.len() == M1Rule::UNIQUE.len()
                && row
                    .rule_results
                    .iter()
                    .zip(M1Rule::UNIQUE)
                    .all(|(result, rule)| result.rule == rule)
        }),
        selected_learning_rate_from_development_only: config
            .learning_rates
            .contains(&selected_learning_rate),
        exact_gradient_optimizer_verified,
        matched_search_budgets: true,
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
        optimizer_effective,
        task_specificity_passed,
        envelope_reachability_passed,
        absolute_reachability_passed,
        stage_passed: false,
        passed: false,
    };
    let protocol_complete = base_acceptance.m1_task_carrier_and_readout_frozen
        && base_acceptance.diagnostic_is_offline_only
        && base_acceptance.development_and_confirmation_seeds_disjoint
        && base_acceptance.development_learning_rate_scan_complete
        && base_acceptance.confirmation_controls_complete
        && base_acceptance.all_four_rules_complete
        && base_acceptance.selected_learning_rate_from_development_only
        && base_acceptance.exact_gradient_optimizer_verified
        && base_acceptance.matched_search_budgets
        && base_acceptance.action_readout_remained_frozen
        && base_acceptance.topology_and_connection_budget_preserved
        && base_acceptance.finite_outputs;
    let acceptance = M1XAcceptanceReport {
        stage_passed: protocol_complete,
        passed: protocol_complete,
        ..base_acceptance
    };
    let decision = if !protocol_complete || !finite_outputs || !stability_within_bounds {
        M1XDecision::DynamicsUnstable
    } else if absolute_reachability_passed && envelope_reachability_passed {
        M1XDecision::ReachableWithinCarrierEnvelope
    } else if absolute_reachability_passed {
        M1XDecision::ReachableOnlyAtAbsoluteBounds
    } else if optimizer_effective && task_specificity_passed {
        M1XDecision::FixedSubspaceUnreached
    } else {
        M1XDecision::OptimizationInconclusive
    };
    let conclusions = conclusions(
        &development_learning_rate_summaries,
        selected_learning_rate,
        &confirmation_summaries,
        paired_effects,
        decision,
    );
    Ok(M1XResult {
        version: "adaptive-mechanism/m1x-v0.9-fixed-subspace-reachability".into(),
        config,
        diagnostic_contract: vec![
            "the oracle exists only in isolated clones and may read target labels; no oracle state, label, gradient, or optimized weight is written back to the online system".into(),
            "nodes, sensory weights, recurrent topology, 48 adjustable recurrent slots, A-trained action readout, task streams, and evaluation budget remain frozen".into(),
            "the norm-envelope oracle projects each target pair to its carrier reference norm; the absolute-bound oracle changes the same 48 values only and clamps them to the existing absolute weight limit".into(),
            "true-target and shuffled-target oracles use identical iterations, restarts, training trials, optimizer, and bounds; frozen weights use the same evaluation streams".into(),
            "development selects one learning rate; independent confirmation cannot change optimizer settings, controls, thresholds, or the decision order".into(),
            "failure means the preregistered high-information search did not reach the threshold; it is empirical evidence, not a proof of mathematical impossibility".into(),
        ],
        carrier_parameters,
        development_seed_results,
        development_learning_rate_summaries,
        selected_learning_rate,
        confirmation_seed_results,
        confirmation_summaries,
        paired_effects,
        decision,
        acceptance,
        conclusions,
    })
}

impl M1XControl {
    pub const fn all() -> [Self; 4] {
        [
            Self::FrozenWeights,
            Self::NormEnvelopeOracle,
            Self::AbsoluteBoundOracle,
            Self::ShuffledTargetOracle,
        ]
    }
}

fn validate_config(config: M1XConfig) -> Result<(), EmbodiedError> {
    if config
        .learning_rates
        .iter()
        .any(|value| !value.is_finite() || *value <= 0.0)
        || config
            .learning_rates
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
        || config.oracle_iterations == 0
        || config.oracle_training_trials < 4
        || config.oracle_restart_count == 0
        || !config.restart_jitter.is_finite()
        || config.restart_jitter < 0.0
        || [
            config.minimum_novel_behavior_accuracy,
            config.minimum_novel_target_probability,
            config.minimum_single_novel_accuracy,
            config.minimum_oracle_advantage,
            config.minimum_loss_reduction,
            config.maximum_saturation_fraction,
        ]
        .into_iter()
        .any(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
    {
        return Err(EmbodiedError::InvalidExperiment);
    }
    Ok(())
}

fn summarize_learning_rate(
    results: &[M1XSeedResult],
    learning_rate: f64,
) -> M1XLearningRateSummary {
    let rows = results
        .iter()
        .filter(|row| row.learning_rate == learning_rate)
        .collect::<Vec<_>>();
    M1XLearningRateSummary {
        learning_rate,
        run_count: rows.len(),
        mean_novel_behavior_accuracy: mean_interval(&novel_values(&rows, |row| {
            row.final_behavior_accuracy
        })),
        mean_novel_argmax_accuracy: mean_interval(&novel_values(&rows, |row| {
            row.final_argmax_accuracy
        })),
        mean_novel_target_probability: mean_interval(&novel_values(&rows, |row| {
            row.final_target_probability
        })),
        mean_minimum_novel_accuracy: mean_interval(
            &rows
                .iter()
                .map(|row| {
                    row.rule_results[1..]
                        .iter()
                        .map(|rule| rule.final_behavior_accuracy)
                        .fold(1.0, f64::min)
                })
                .collect::<Vec<_>>(),
        ),
        mean_training_loss_reduction: mean_interval(&novel_values(&rows, |row| {
            row.initial_training_loss - row.final_training_loss
        })),
        mean_saturation_fraction: mean_interval(&novel_values(&rows, |row| {
            row.saturation_fraction
        })),
        finite_fraction: rows.iter().filter(|row| row.finite).count() as f64
            / rows.len().max(1) as f64,
    }
}

fn learning_rate_rank(summary: &M1XLearningRateSummary) -> f64 {
    summary.mean_novel_target_probability.mean
        + summary.mean_novel_behavior_accuracy.mean
        + 0.5 * summary.mean_minimum_novel_accuracy.mean
        - summary.mean_saturation_fraction.mean
}

fn summarize_control(results: &[M1XSeedResult], control: M1XControl) -> M1XControlSummary {
    let rows = results
        .iter()
        .filter(|row| row.control == control)
        .collect::<Vec<_>>();
    let history = rows
        .iter()
        .map(|row| row.rule_results[0].final_behavior_accuracy)
        .collect::<Vec<_>>();
    M1XControlSummary {
        control,
        learning_rate: rows.first().map_or(0.0, |row| row.learning_rate),
        run_count: rows.len(),
        mean_history_behavior_accuracy: mean_interval(&history),
        mean_novel_behavior_accuracy: mean_interval(&novel_values(&rows, |row| {
            row.final_behavior_accuracy
        })),
        mean_novel_argmax_accuracy: mean_interval(&novel_values(&rows, |row| {
            row.final_argmax_accuracy
        })),
        mean_novel_target_probability: mean_interval(&novel_values(&rows, |row| {
            row.final_target_probability
        })),
        mean_minimum_novel_accuracy: mean_interval(
            &rows
                .iter()
                .map(|row| {
                    row.rule_results[1..]
                        .iter()
                        .map(|rule| rule.final_behavior_accuracy)
                        .fold(1.0, f64::min)
                })
                .collect::<Vec<_>>(),
        ),
        mean_training_loss_reduction: mean_interval(&novel_values(&rows, |row| {
            row.initial_training_loss - row.final_training_loss
        })),
        mean_relative_weight_drift: mean_interval(&novel_values(&rows, |row| {
            row.mean_relative_weight_drift
        })),
        mean_saturation_fraction: mean_interval(&novel_values(&rows, |row| {
            row.saturation_fraction
        })),
        finite_fraction: rows.iter().filter(|row| row.finite).count() as f64
            / rows.len().max(1) as f64,
    }
}

fn novel_values(rows: &[&M1XSeedResult], metric: fn(&M1XRuleResult) -> f64) -> Vec<f64> {
    rows.iter()
        .map(|row| row.rule_results[1..].iter().map(metric).sum::<f64>() / 3.0)
        .collect()
}

fn paired_effects(results: &[M1XSeedResult]) -> M1XPairedEffects {
    let paired = |left: M1XControl, right: M1XControl, metric: fn(&M1XRuleResult) -> f64| {
        let mut values = Vec::new();
        for row in results.iter().filter(|row| row.control == left) {
            let other = results
                .iter()
                .find(|candidate| {
                    candidate.control == right
                        && candidate.parameter_id == row.parameter_id
                        && candidate.seed == row.seed
                })
                .expect("paired M1-X control");
            values.push(
                row.rule_results[1..]
                    .iter()
                    .zip(&other.rule_results[1..])
                    .map(|(a, b)| metric(a) - metric(b))
                    .sum::<f64>()
                    / 3.0,
            );
        }
        mean_interval(&values)
    };
    M1XPairedEffects {
        absolute_over_frozen_behavior: paired(
            M1XControl::AbsoluteBoundOracle,
            M1XControl::FrozenWeights,
            |row| row.final_behavior_accuracy,
        ),
        absolute_over_shuffled_behavior: paired(
            M1XControl::AbsoluteBoundOracle,
            M1XControl::ShuffledTargetOracle,
            |row| row.final_behavior_accuracy,
        ),
        absolute_over_frozen_target_probability: paired(
            M1XControl::AbsoluteBoundOracle,
            M1XControl::FrozenWeights,
            |row| row.final_target_probability,
        ),
        absolute_over_shuffled_target_probability: paired(
            M1XControl::AbsoluteBoundOracle,
            M1XControl::ShuffledTargetOracle,
            |row| row.final_target_probability,
        ),
        absolute_over_envelope_behavior: paired(
            M1XControl::AbsoluteBoundOracle,
            M1XControl::NormEnvelopeOracle,
            |row| row.final_behavior_accuracy,
        ),
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
    development: &[M1XLearningRateSummary],
    selected_learning_rate: f64,
    confirmation: &[M1XControlSummary],
    effects: M1XPairedEffects,
    decision: M1XDecision,
) -> Vec<String> {
    let dev = development
        .iter()
        .map(|row| {
            format!(
                "{:.3}: {:.1}% / {:.1}%",
                row.learning_rate,
                row.mean_novel_behavior_accuracy.mean * 100.0,
                row.mean_novel_target_probability.mean * 100.0,
            )
        })
        .collect::<Vec<_>>()
        .join(" · ");
    let summary = |control| {
        confirmation
            .iter()
            .find(|row| row.control == control)
            .expect("control summary")
    };
    let frozen = summary(M1XControl::FrozenWeights);
    let envelope = summary(M1XControl::NormEnvelopeOracle);
    let absolute = summary(M1XControl::AbsoluteBoundOracle);
    let shuffled = summary(M1XControl::ShuffledTargetOracle);
    vec![
        format!(
            "开发学习率扫描（B/C/D 行为 / 目标概率）为 {dev}；冻结选择 {selected_learning_rate:.3}。"
        ),
        format!(
            "独立确认 B/C/D 行为：冻结 {:.1}%，范数包络 oracle {:.1}%，绝对边界 oracle {:.1}%，乱序目标 oracle {:.1}%。",
            frozen.mean_novel_behavior_accuracy.mean * 100.0,
            envelope.mean_novel_behavior_accuracy.mean * 100.0,
            absolute.mean_novel_behavior_accuracy.mean * 100.0,
            shuffled.mean_novel_behavior_accuracy.mean * 100.0,
        ),
        format!(
            "绝对边界 oracle 相对冻结行为 {:+.2} pp [{:+.2}, {:+.2}]，相对乱序目标 {:+.2} pp [{:+.2}, {:+.2}]。",
            effects.absolute_over_frozen_behavior.mean * 100.0,
            effects.absolute_over_frozen_behavior.lower95 * 100.0,
            effects.absolute_over_frozen_behavior.upper95 * 100.0,
            effects.absolute_over_shuffled_behavior.mean * 100.0,
            effects.absolute_over_shuffled_behavior.lower95 * 100.0,
            effects.absolute_over_shuffled_behavior.upper95 * 100.0,
        ),
        format!("M1-X 正式决策为 {decision:?}。"),
        match decision {
            M1XDecision::ReachableWithinCarrierEnvelope => "固定拓扑与固定读出在当前范数包络内即可表达新规则；下一瓶颈明确位于在线局部信用路由。".into(),
            M1XDecision::ReachableOnlyAtAbsoluteBounds => "同一 48 权重子空间只有放宽到载体绝对边界后才可达；当前稳态范数包络是独立瓶颈，在线信用仍未获验证。".into(),
            M1XDecision::FixedSubspaceUnreached => "高信息 oracle 在预注册预算内仍未使固定 48 权重与固定读出达到新规则门槛；证据转向当前可塑子空间/读出接口，而非继续堆叠局部信用启发式。".into(),
            M1XDecision::OptimizationInconclusive => "优化或任务特异性对照不足，不能把未达门槛归因于可塑子空间；应先增强或审计离线搜索。".into(),
            M1XDecision::DynamicsUnstable => "诊断出现非有限值、过饱和或协议不完整，不能解释可达性。".into(),
        },
    ]
}
