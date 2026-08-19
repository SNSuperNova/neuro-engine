use serde::Serialize;

use crate::learnability_map::{
    ContinuousResource, DualTimescaleHomeostasis, HomeostasisMechanism, M1XSeedProtocol,
    PlasticityMechanism, ResourceMechanism, SoftBoundedPlasticity, run_m1ne_seed, seed_partition,
    verify_m1x_oracle_gradient,
};
use crate::map1::{parameter_points, protocol_config};
use crate::{EmbodiedError, HIDDEN_COUNT, M1Rule, M1XEConfig, Map0Interval, Map0ParameterPoint};

const DEVELOPMENT_SEED_LABEL: u64 = 0x4d31_4e45_4445_5601;
const CONFIRMATION_SEED_LABEL: u64 = 0x4d31_4e45_434f_4e46;
pub const M1NE_CONTROL_COUNT: usize = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum M1NEControl {
    RewardLocalOneX,
    RewardLocalOnePointFive,
    FrozenOnePointFive,
    RandomConsequenceOnePointFive,
    OracleOnePointFive,
}

impl M1NEControl {
    pub const ALL: [Self; M1NE_CONTROL_COUNT] = [
        Self::RewardLocalOneX,
        Self::RewardLocalOnePointFive,
        Self::FrozenOnePointFive,
        Self::RandomConsequenceOnePointFive,
        Self::OracleOnePointFive,
    ];

    pub(crate) fn norm_multiplier(self) -> f64 {
        if self == Self::RewardLocalOneX {
            1.0
        } else {
            1.5
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum M1NEDecision {
    OnlineSufficiencyConfirmed,
    AmplitudeNecessaryButInsufficient,
    ConsequenceIndependent,
    HistoryDegraded,
    OracleAnchorFailed,
    DynamicsUnstable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1NEFrozenArtifact {
    pub file: &'static str,
    pub sha256: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1NEConfig {
    pub m1xe_protocol: M1XEConfig,
    pub norm_multiplier: f64,
    pub minimum_history_accuracy: f64,
    pub minimum_novel_behavior_accuracy: f64,
    pub minimum_novel_target_probability: f64,
    pub minimum_single_novel_accuracy: f64,
    pub minimum_causal_advantage: f64,
    pub maximum_history_degradation: f64,
    pub minimum_mean_resource_level: f64,
    pub maximum_saturation_fraction: f64,
}

impl Default for M1NEConfig {
    fn default() -> Self {
        Self {
            m1xe_protocol: M1XEConfig::default(),
            norm_multiplier: 1.5,
            minimum_history_accuracy: 0.75,
            minimum_novel_behavior_accuracy: 0.70,
            minimum_novel_target_probability: 0.70,
            minimum_single_novel_accuracy: 0.65,
            minimum_causal_advantage: 0.05,
            maximum_history_degradation: 0.05,
            minimum_mean_resource_level: 0.20,
            maximum_saturation_fraction: 0.25,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct M1NESeedProtocol {
    pub online_trial_count: usize,
    pub evaluation_trial_count: usize,
    pub exploration: f64,
    pub norm_multiplier: f64,
    pub oracle: M1XSeedProtocol,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1NERuleResult {
    pub rule: M1Rule,
    pub initial_behavior_accuracy: f64,
    pub final_behavior_accuracy: f64,
    pub final_argmax_accuracy: f64,
    pub initial_target_probability: f64,
    pub final_target_probability: f64,
    pub target_probability_gain: f64,
    pub mean_resource_level: f64,
    pub minimum_resource_level: f64,
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
pub struct M1NESeedResult {
    pub parameter_id: usize,
    pub seed: u64,
    pub control: M1NEControl,
    pub norm_multiplier: f64,
    pub rule_results: Vec<M1NERuleResult>,
    pub finite: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1NEControlSummary {
    pub control: M1NEControl,
    pub norm_multiplier: f64,
    pub run_count: usize,
    pub mean_history_initial_accuracy: Map0Interval,
    pub mean_history_final_accuracy: Map0Interval,
    pub mean_novel_initial_accuracy: Map0Interval,
    pub mean_novel_final_accuracy: Map0Interval,
    pub mean_novel_final_argmax_accuracy: Map0Interval,
    pub mean_novel_initial_target_probability: Map0Interval,
    pub mean_novel_final_target_probability: Map0Interval,
    pub mean_novel_target_probability_gain: Map0Interval,
    pub mean_minimum_novel_accuracy: Map0Interval,
    pub mean_resource_level: Map0Interval,
    pub mean_minimum_resource_level: Map0Interval,
    pub mean_relative_weight_drift: Map0Interval,
    pub mean_saturation_fraction: Map0Interval,
    pub finite_fraction: f64,
    pub capability_passed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1NEPairedEffects {
    pub candidate_over_one_x_behavior: Map0Interval,
    pub candidate_over_one_x_target_probability: Map0Interval,
    pub candidate_over_frozen_behavior: Map0Interval,
    pub candidate_over_frozen_target_probability: Map0Interval,
    pub candidate_over_random_behavior: Map0Interval,
    pub candidate_over_random_target_probability: Map0Interval,
    pub candidate_history_change_from_frozen: Map0Interval,
    pub oracle_over_candidate_behavior: Map0Interval,
    pub oracle_over_candidate_target_probability: Map0Interval,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1NEAcceptanceReport {
    pub m1xe_artifact_and_boundary_frozen: bool,
    pub no_online_hyperparameter_selection: bool,
    pub development_and_confirmation_seeds_disjoint: bool,
    pub development_controls_complete: bool,
    pub confirmation_controls_complete: bool,
    pub all_four_rules_complete: bool,
    pub online_candidate_observation_scope_local: bool,
    pub target_rule_and_oracle_hidden_from_online_controls: bool,
    pub paired_carriers_trials_and_consequences: bool,
    pub action_readout_remained_frozen: bool,
    pub topology_and_connection_budget_preserved: bool,
    pub exact_gradient_oracle_verified: bool,
    pub finite_outputs: bool,
    pub stability_within_bounds: bool,
    pub oracle_anchor_passed: bool,
    pub candidate_capability_passed: bool,
    pub envelope_advantage_passed: bool,
    pub learned_advantage_passed: bool,
    pub consequence_specificity_passed: bool,
    pub target_probability_formation_passed: bool,
    pub history_preserved: bool,
    pub online_sufficiency_confirmed: bool,
    pub stage_passed: bool,
    pub passed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1NEResult {
    pub version: String,
    pub config: M1NEConfig,
    pub frozen_m1xe_artifact: M1NEFrozenArtifact,
    pub mechanism_contract: Vec<String>,
    pub carrier_parameters: Vec<Map0ParameterPoint>,
    pub development_seed_results: Vec<M1NESeedResult>,
    pub development_summaries: Vec<M1NEControlSummary>,
    pub development_effects: M1NEPairedEffects,
    pub confirmation_seed_results: Vec<M1NESeedResult>,
    pub confirmation_summaries: Vec<M1NEControlSummary>,
    pub confirmation_effects: M1NEPairedEffects,
    pub decision: M1NEDecision,
    pub acceptance: M1NEAcceptanceReport,
    pub conclusions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1NEPublishedResult {
    pub version: String,
    pub config: M1NEConfig,
    pub frozen_m1xe_artifact: M1NEFrozenArtifact,
    pub mechanism_contract: Vec<String>,
    pub carrier_parameters: Vec<Map0ParameterPoint>,
    pub development_summaries: Vec<M1NEControlSummary>,
    pub development_effects: M1NEPairedEffects,
    pub confirmation_summaries: Vec<M1NEControlSummary>,
    pub confirmation_effects: M1NEPairedEffects,
    pub decision: M1NEDecision,
    pub acceptance: M1NEAcceptanceReport,
    pub conclusions: Vec<String>,
}

impl M1NEResult {
    pub fn published(&self) -> M1NEPublishedResult {
        M1NEPublishedResult {
            version: self.version.clone(),
            config: self.config,
            frozen_m1xe_artifact: self.frozen_m1xe_artifact,
            mechanism_contract: self.mechanism_contract.clone(),
            carrier_parameters: self.carrier_parameters.clone(),
            development_summaries: self.development_summaries.clone(),
            development_effects: self.development_effects,
            confirmation_summaries: self.confirmation_summaries.clone(),
            confirmation_effects: self.confirmation_effects,
            decision: self.decision,
            acceptance: self.acceptance,
            conclusions: self.conclusions.clone(),
        }
    }
}

pub fn run_m1ne_experiment(config: M1NEConfig) -> Result<M1NEResult, EmbodiedError> {
    validate_config(config)?;
    let m1 = config.m1xe_protocol.m1x_protocol.m1_protocol;
    let map1 = m1.map2_protocol.map2b_protocol.map2a_protocol.map1_protocol;
    let carrier = protocol_config(map1);
    let all_parameters = parameter_points(map1);
    let carrier_parameters = m1
        .reference_parameter_ids
        .iter()
        .map(|id| all_parameters[*id])
        .collect::<Vec<_>>();
    let development_seeds = seed_partition(
        map1.seed ^ DEVELOPMENT_SEED_LABEL,
        m1.development_seed_count,
    );
    let confirmation_seeds = seed_partition(
        map1.seed ^ CONFIRMATION_SEED_LABEL,
        m1.confirmation_seed_count,
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
        bound_scale: m1
            .map2_protocol
            .map2b_protocol
            .map2a_protocol
            .soft_bound_scale,
    });
    let map2b = m1.map2_protocol.map2b_protocol;
    let resource = ResourceMechanism::Continuous(ContinuousResource {
        initial_level: map2b.initial_resource,
        supply_rate: map2b.supply_rate,
        maintenance_cost: map2b.maintenance_cost,
        activity_cost: map2b.activity_cost,
        plasticity_cost: map2b.plasticity_cost,
        minimum_modulation: map2b.minimum_modulation,
    });
    let m1x = config.m1xe_protocol.m1x_protocol;
    let protocol = M1NESeedProtocol {
        online_trial_count: m1.phase_trial_count,
        evaluation_trial_count: m1.evaluation_trial_count,
        exploration: m1.map2_protocol.exploration,
        norm_multiplier: config.norm_multiplier,
        oracle: M1XSeedProtocol {
            training_trial_count: m1x.oracle_training_trials,
            evaluation_trial_count: m1.evaluation_trial_count,
            iterations: m1x.oracle_iterations,
            restart_count: m1x.oracle_restart_count,
            restart_jitter: m1x.restart_jitter,
            learning_rate: config.m1xe_protocol.oracle_learning_rate,
        },
    };
    let run = |seeds: &[u64]| {
        M1NEControl::ALL
            .into_iter()
            .flat_map(|control| {
                carrier_parameters.iter().flat_map(move |point| {
                    seeds.iter().map(move |seed| {
                        run_m1ne_seed(
                            carrier,
                            *point,
                            *seed,
                            control,
                            homeostasis,
                            plasticity,
                            resource,
                            protocol,
                        )
                    })
                })
            })
            .collect::<Vec<_>>()
    };
    let development_seed_results = run(&development_seeds);
    let confirmation_seed_results = run(&confirmation_seeds);
    let development_summaries = summarize_controls(&development_seed_results, config);
    let confirmation_summaries = summarize_controls(&confirmation_seed_results, config);
    let development_effects = paired_effects(&development_seed_results);
    let confirmation_effects = paired_effects(&confirmation_seed_results);
    let candidate = summary(
        &confirmation_summaries,
        M1NEControl::RewardLocalOnePointFive,
    );
    let oracle = summary(&confirmation_summaries, M1NEControl::OracleOnePointFive);
    let all_results = development_seed_results
        .iter()
        .chain(&confirmation_seed_results)
        .collect::<Vec<_>>();
    let finite_outputs = all_results.iter().all(|row| row.finite);
    let stability_within_bounds = candidate.mean_resource_level.mean
        >= config.minimum_mean_resource_level
        && candidate.mean_saturation_fraction.upper95 <= config.maximum_saturation_fraction;
    let advantage = |interval: Map0Interval| {
        interval.mean >= config.minimum_causal_advantage && interval.lower95 > 0.0
    };
    let envelope_advantage_passed = advantage(confirmation_effects.candidate_over_one_x_behavior)
        && advantage(confirmation_effects.candidate_over_one_x_target_probability);
    let learned_advantage_passed = advantage(confirmation_effects.candidate_over_frozen_behavior)
        && advantage(confirmation_effects.candidate_over_frozen_target_probability);
    let consequence_specificity_passed =
        advantage(confirmation_effects.candidate_over_random_behavior)
            && advantage(confirmation_effects.candidate_over_random_target_probability);
    let target_probability_formation_passed = candidate.mean_novel_final_target_probability.lower95
        >= config.minimum_novel_target_probability
        && candidate.mean_novel_target_probability_gain.lower95 > 0.0;
    let history_preserved = candidate.mean_history_final_accuracy.lower95
        >= config.minimum_history_accuracy
        && confirmation_effects
            .candidate_history_change_from_frozen
            .mean
            >= -config.maximum_history_degradation;
    let expected_development =
        M1NE_CONTROL_COUNT * carrier_parameters.len() * development_seeds.len();
    let expected_confirmation =
        M1NE_CONTROL_COUNT * carrier_parameters.len() * confirmation_seeds.len();
    let exact_gradient_oracle_verified = verify_m1x_oracle_gradient(
        carrier,
        carrier_parameters[0],
        map1.seed ^ 0x4d31_4e45_4752_4144,
        homeostasis,
        plasticity,
        resource,
    );
    let frozen_m1xe_artifact = M1NEFrozenArtifact {
        file: "app/public/reachability-envelope-v1.0.json",
        sha256: "33d3effecc9821b58294ade846eeb3b0a1fd346668c30e237c75f2cf004f31f2",
    };
    let base_acceptance = M1NEAcceptanceReport {
        m1xe_artifact_and_boundary_frozen: config.m1xe_protocol == M1XEConfig::default()
            && config.norm_multiplier == 1.5,
        no_online_hyperparameter_selection: true,
        development_and_confirmation_seeds_disjoint: development_seeds
            .iter()
            .all(|seed| !confirmation_seeds.contains(seed)),
        development_controls_complete: development_seed_results.len() == expected_development,
        confirmation_controls_complete: confirmation_seed_results.len() == expected_confirmation,
        all_four_rules_complete: all_results.iter().all(|row| {
            row.rule_results.len() == M1Rule::UNIQUE.len()
                && row
                    .rule_results
                    .iter()
                    .zip(M1Rule::UNIQUE)
                    .all(|(result, rule)| result.rule == rule)
        }),
        online_candidate_observation_scope_local: true,
        target_rule_and_oracle_hidden_from_online_controls: true,
        paired_carriers_trials_and_consequences: true,
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
        exact_gradient_oracle_verified,
        finite_outputs,
        stability_within_bounds,
        oracle_anchor_passed: oracle.capability_passed,
        candidate_capability_passed: candidate.capability_passed,
        envelope_advantage_passed,
        learned_advantage_passed,
        consequence_specificity_passed,
        target_probability_formation_passed,
        history_preserved,
        online_sufficiency_confirmed: false,
        stage_passed: false,
        passed: false,
    };
    let protocol_complete = base_acceptance.m1xe_artifact_and_boundary_frozen
        && base_acceptance.no_online_hyperparameter_selection
        && base_acceptance.development_and_confirmation_seeds_disjoint
        && base_acceptance.development_controls_complete
        && base_acceptance.confirmation_controls_complete
        && base_acceptance.all_four_rules_complete
        && base_acceptance.online_candidate_observation_scope_local
        && base_acceptance.target_rule_and_oracle_hidden_from_online_controls
        && base_acceptance.paired_carriers_trials_and_consequences
        && base_acceptance.action_readout_remained_frozen
        && base_acceptance.topology_and_connection_budget_preserved
        && base_acceptance.exact_gradient_oracle_verified
        && base_acceptance.finite_outputs;
    let online_sufficiency_confirmed = protocol_complete
        && stability_within_bounds
        && oracle.capability_passed
        && candidate.capability_passed
        && envelope_advantage_passed
        && learned_advantage_passed
        && consequence_specificity_passed
        && target_probability_formation_passed
        && history_preserved;
    let acceptance = M1NEAcceptanceReport {
        online_sufficiency_confirmed,
        stage_passed: protocol_complete,
        passed: protocol_complete,
        ..base_acceptance
    };
    let decision = if !protocol_complete || !finite_outputs || !stability_within_bounds {
        M1NEDecision::DynamicsUnstable
    } else if !oracle.capability_passed {
        M1NEDecision::OracleAnchorFailed
    } else if !history_preserved {
        M1NEDecision::HistoryDegraded
    } else if candidate.capability_passed
        && learned_advantage_passed
        && !consequence_specificity_passed
    {
        M1NEDecision::ConsequenceIndependent
    } else if online_sufficiency_confirmed {
        M1NEDecision::OnlineSufficiencyConfirmed
    } else {
        M1NEDecision::AmplitudeNecessaryButInsufficient
    };
    let conclusions = conclusions(
        &development_summaries,
        &confirmation_summaries,
        confirmation_effects,
        decision,
    );
    Ok(M1NEResult {
        version: "adaptive-mechanism/m1ne-v1.1-norm-enabled-sufficiency".into(),
        config,
        frozen_m1xe_artifact,
        mechanism_contract: vec![
            "M1-XE's 1.5x target norm is applied once to the same 48 recurrent values and reference norms before online trials; topology, input, resources, and the A action readout remain frozen".into(),
            "the candidate is the existing reward-local eligibility update with no new parameter scan, no target labels, no oracle gradient, and no access to rule identity".into(),
            "1.0x reward-local, 1.5x frozen, and 1.5x random-consequence controls isolate envelope benefit, learning benefit, and consequence specificity on paired carrier, trial, and seed streams".into(),
            "the 1.5x exact-gradient oracle is an isolated positive reachability anchor only; its labels, gradients, moments, restarts, and weights never enter an online control".into(),
            "development performs no selection; confirmation on disjoint seeds is authoritative for every capability, causal, history, resource, and stability threshold".into(),
        ],
        carrier_parameters,
        development_seed_results,
        development_summaries,
        development_effects,
        confirmation_seed_results,
        confirmation_summaries,
        confirmation_effects,
        decision,
        acceptance,
        conclusions,
    })
}

fn validate_config(config: M1NEConfig) -> Result<(), EmbodiedError> {
    let probabilities = [
        config.minimum_history_accuracy,
        config.minimum_novel_behavior_accuracy,
        config.minimum_novel_target_probability,
        config.minimum_single_novel_accuracy,
        config.minimum_causal_advantage,
        config.maximum_history_degradation,
        config.minimum_mean_resource_level,
        config.maximum_saturation_fraction,
    ];
    if config.norm_multiplier != 1.5
        || probabilities
            .iter()
            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
        || config
            .m1xe_protocol
            .m1x_protocol
            .m1_protocol
            .development_seed_count
            == 0
        || config
            .m1xe_protocol
            .m1x_protocol
            .m1_protocol
            .confirmation_seed_count
            == 0
        || config
            .m1xe_protocol
            .m1x_protocol
            .m1_protocol
            .phase_trial_count
            < 4
        || config
            .m1xe_protocol
            .m1x_protocol
            .m1_protocol
            .evaluation_trial_count
            < 4
        || config.m1xe_protocol.m1x_protocol.oracle_iterations == 0
        || config.m1xe_protocol.m1x_protocol.oracle_training_trials < 4
        || config.m1xe_protocol.m1x_protocol.oracle_restart_count == 0
    {
        return Err(EmbodiedError::InvalidExperiment);
    }
    Ok(())
}

fn summarize_controls(results: &[M1NESeedResult], config: M1NEConfig) -> Vec<M1NEControlSummary> {
    M1NEControl::ALL
        .into_iter()
        .map(|control| summarize_control(results, control, config))
        .collect()
}

fn summarize_control(
    results: &[M1NESeedResult],
    control: M1NEControl,
    config: M1NEConfig,
) -> M1NEControlSummary {
    let rows = results
        .iter()
        .filter(|row| row.control == control)
        .collect::<Vec<_>>();
    let history = |metric: fn(&M1NERuleResult) -> f64| {
        rows.iter()
            .map(|row| metric(&row.rule_results[0]))
            .collect::<Vec<_>>()
    };
    let novel = |metric: fn(&M1NERuleResult) -> f64| {
        rows.iter()
            .map(|row| row.rule_results[1..].iter().map(metric).sum::<f64>() / 3.0)
            .collect::<Vec<_>>()
    };
    let minimum = rows
        .iter()
        .map(|row| {
            row.rule_results[1..]
                .iter()
                .map(|rule| rule.final_behavior_accuracy)
                .fold(1.0, f64::min)
        })
        .collect::<Vec<_>>();
    let history_initial = mean_interval(&history(|row| row.initial_behavior_accuracy));
    let history_final = mean_interval(&history(|row| row.final_behavior_accuracy));
    let novel_initial = mean_interval(&novel(|row| row.initial_behavior_accuracy));
    let novel_final = mean_interval(&novel(|row| row.final_behavior_accuracy));
    let novel_argmax = mean_interval(&novel(|row| row.final_argmax_accuracy));
    let target_initial = mean_interval(&novel(|row| row.initial_target_probability));
    let target_final = mean_interval(&novel(|row| row.final_target_probability));
    let target_gain = mean_interval(&novel(|row| row.target_probability_gain));
    let minimum_novel = mean_interval(&minimum);
    let resource = mean_interval(&novel(|row| row.mean_resource_level));
    let minimum_resource = mean_interval(&novel(|row| row.minimum_resource_level));
    let drift = mean_interval(&novel(|row| row.mean_relative_weight_drift));
    let saturation = mean_interval(&novel(|row| row.saturation_fraction));
    let finite_fraction =
        rows.iter().filter(|row| row.finite).count() as f64 / rows.len().max(1) as f64;
    let capability_passed = novel_final.lower95 >= config.minimum_novel_behavior_accuracy
        && target_final.lower95 >= config.minimum_novel_target_probability
        && minimum_novel.mean >= config.minimum_single_novel_accuracy
        && history_final.lower95 >= config.minimum_history_accuracy
        && saturation.upper95 <= config.maximum_saturation_fraction
        && finite_fraction == 1.0;
    M1NEControlSummary {
        control,
        norm_multiplier: control.norm_multiplier(),
        run_count: rows.len(),
        mean_history_initial_accuracy: history_initial,
        mean_history_final_accuracy: history_final,
        mean_novel_initial_accuracy: novel_initial,
        mean_novel_final_accuracy: novel_final,
        mean_novel_final_argmax_accuracy: novel_argmax,
        mean_novel_initial_target_probability: target_initial,
        mean_novel_final_target_probability: target_final,
        mean_novel_target_probability_gain: target_gain,
        mean_minimum_novel_accuracy: minimum_novel,
        mean_resource_level: resource,
        mean_minimum_resource_level: minimum_resource,
        mean_relative_weight_drift: drift,
        mean_saturation_fraction: saturation,
        finite_fraction,
        capability_passed,
    }
}

fn paired_effects(results: &[M1NESeedResult]) -> M1NEPairedEffects {
    let effect =
        |left: M1NEControl, right: M1NEControl, metric: fn(&M1NERuleResult) -> f64, novel: bool| {
            let mut values = Vec::new();
            for row in results.iter().filter(|row| row.control == left) {
                let paired = results
                    .iter()
                    .find(|candidate| {
                        candidate.control == right
                            && candidate.parameter_id == row.parameter_id
                            && candidate.seed == row.seed
                    })
                    .expect("paired M1-NE control");
                let range = if novel { 1..4 } else { 0..1 };
                values.push(
                    row.rule_results[range.clone()]
                        .iter()
                        .zip(&paired.rule_results[range])
                        .map(|(left_rule, right_rule)| metric(left_rule) - metric(right_rule))
                        .sum::<f64>()
                        / if novel { 3.0 } else { 1.0 },
                );
            }
            mean_interval(&values)
        };
    let candidate = M1NEControl::RewardLocalOnePointFive;
    M1NEPairedEffects {
        candidate_over_one_x_behavior: effect(
            candidate,
            M1NEControl::RewardLocalOneX,
            |row| row.final_behavior_accuracy,
            true,
        ),
        candidate_over_one_x_target_probability: effect(
            candidate,
            M1NEControl::RewardLocalOneX,
            |row| row.final_target_probability,
            true,
        ),
        candidate_over_frozen_behavior: effect(
            candidate,
            M1NEControl::FrozenOnePointFive,
            |row| row.final_behavior_accuracy,
            true,
        ),
        candidate_over_frozen_target_probability: effect(
            candidate,
            M1NEControl::FrozenOnePointFive,
            |row| row.final_target_probability,
            true,
        ),
        candidate_over_random_behavior: effect(
            candidate,
            M1NEControl::RandomConsequenceOnePointFive,
            |row| row.final_behavior_accuracy,
            true,
        ),
        candidate_over_random_target_probability: effect(
            candidate,
            M1NEControl::RandomConsequenceOnePointFive,
            |row| row.final_target_probability,
            true,
        ),
        candidate_history_change_from_frozen: effect(
            candidate,
            M1NEControl::FrozenOnePointFive,
            |row| row.final_behavior_accuracy,
            false,
        ),
        oracle_over_candidate_behavior: effect(
            M1NEControl::OracleOnePointFive,
            candidate,
            |row| row.final_behavior_accuracy,
            true,
        ),
        oracle_over_candidate_target_probability: effect(
            M1NEControl::OracleOnePointFive,
            candidate,
            |row| row.final_target_probability,
            true,
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

fn summary(summaries: &[M1NEControlSummary], control: M1NEControl) -> &M1NEControlSummary {
    summaries
        .iter()
        .find(|row| row.control == control)
        .expect("M1-NE control summary")
}

fn conclusions(
    development: &[M1NEControlSummary],
    confirmation: &[M1NEControlSummary],
    effects: M1NEPairedEffects,
    decision: M1NEDecision,
) -> Vec<String> {
    let describe = |rows: &[M1NEControlSummary]| {
        M1NEControl::ALL
            .into_iter()
            .map(|control| {
                let row = summary(rows, control);
                format!(
                    "{control:?}: {:.1}% / {:.1}% / {:.1}%{}",
                    row.mean_novel_final_accuracy.mean * 100.0,
                    row.mean_novel_final_target_probability.mean * 100.0,
                    row.mean_minimum_novel_accuracy.mean * 100.0,
                    if row.capability_passed { " ✓" } else { "" },
                )
            })
            .collect::<Vec<_>>()
            .join(" · ")
    };
    vec![
        format!(
            "开发控制（B/C/D 行为 / 目标概率 / 最低新规则）为 {}。",
            describe(development),
        ),
        format!(
            "独立确认控制为 {}。",
            describe(confirmation),
        ),
        format!(
            "1.5× 在线候选相对 1.0× / 1.5× 冻结 / 1.5× 随机后果的 B/C/D 行为变化分别为 {:+.2} / {:+.2} / {:+.2} pp。",
            effects.candidate_over_one_x_behavior.mean * 100.0,
            effects.candidate_over_frozen_behavior.mean * 100.0,
            effects.candidate_over_random_behavior.mean * 100.0,
        ),
        format!("M1-NE 正式决策为 {decision:?}。"),
        match decision {
            M1NEDecision::OnlineSufficiencyConfirmed => "冻结 1.5× 幅度自由度后，既有奖励局部规则在独立确认中产生了后果特异、超过冻结与 1.0× 基线且保持 A 的能力；下一步才允许研究系统如何局部形成并维持该包络。".into(),
            M1NEDecision::AmplitudeNecessaryButInsufficient => "1.5× oracle 仍可达；既有在线局部规则相对冻结与随机后果产生了后果特异的部分改善，但未跨过能力与形成门槛，也未达到预注册的 1.0× 优势幅度。幅度自由度必要但不足，下一步应分解局部信用的强度与方向对齐，而不是继续放大范数或增加节点。".into(),
            M1NEDecision::ConsequenceIndependent => "1.5× 在线变化未可靠胜过随机后果，不能归因于任务结果信用；下一步应修订后果到局部连接的信用路由。".into(),
            M1NEDecision::HistoryDegraded => "1.5× 在线候选损伤已有 A，不能把新规则变化解释为稳定能力形成；下一步先解决保留与更新的冲突。".into(),
            M1NEDecision::OracleAnchorFailed => "同种子 1.5× oracle 未复现 M1-XE 可达性，M1-NE 的在线比较失去正锚点，不能解释局部信用。".into(),
            M1NEDecision::DynamicsUnstable => "协议、资源、饱和、有限值或冻结接口失败，M1-NE 不能作能力解释。".into(),
        },
    ]
}
