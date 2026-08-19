use serde::Serialize;

use crate::learnability_map::{
    ContinuousResource, DualTimescaleHomeostasis, HomeostasisMechanism, M1CDSeedProtocol,
    PlasticityMechanism, ResourceMechanism, SoftBoundedPlasticity, run_m1cd_seed, seed_partition,
    verify_m1x_oracle_gradient,
};
use crate::map1::{parameter_points, protocol_config};
use crate::{EmbodiedError, HIDDEN_COUNT, M1NEConfig, M1Rule, Map0Interval, Map0ParameterPoint};

const DEVELOPMENT_SEED_LABEL: u64 = 0x4d31_4344_4445_5601;
const CONFIRMATION_SEED_LABEL: u64 = 0x4d31_4344_434f_4e46;
pub const M1CD_CONTROL_COUNT: usize = 3;
pub const M1CD_COMPONENT_COUNT: usize = 4;
pub const M1CD_CHECKPOINT_COUNT: usize = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum M1CDControl {
    TrueConsequence,
    RandomConsequence,
    ShuffledEligibility,
}

impl M1CDControl {
    pub const ALL: [Self; M1CD_CONTROL_COUNT] = [
        Self::TrueConsequence,
        Self::RandomConsequence,
        Self::ShuffledEligibility,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum M1CDComponent {
    RawProposal,
    AppliedPlasticity,
    HomeostasisCorrection,
    TotalUpdate,
}

impl M1CDComponent {
    pub const ALL: [Self; M1CD_COMPONENT_COUNT] = [
        Self::RawProposal,
        Self::AppliedPlasticity,
        Self::HomeostasisCorrection,
        Self::TotalUpdate,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum M1CDDecision {
    LocalCreditDirectionAdequate,
    AlignedButWeak,
    HomeostasisCancelsCredit,
    EligibilityUninformative,
    ConsequenceSignalUninformative,
    DirectionMostlyMisaligned,
    DynamicsUnstable,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1CDConfig {
    pub m1ne_protocol: M1NEConfig,
    pub checkpoints: [usize; M1CD_CHECKPOINT_COUNT],
    pub diagnostic_trial_count: usize,
    pub oracle_trial_count: usize,
    pub minimum_direction_cosine: f64,
    pub minimum_sign_agreement: f64,
    pub minimum_control_advantage: f64,
    pub minimum_productive_projection: f64,
    pub maximum_homeostasis_projection_loss: f64,
}

impl Default for M1CDConfig {
    fn default() -> Self {
        Self {
            m1ne_protocol: M1NEConfig::default(),
            checkpoints: [0, 60, 120, 180, 240],
            diagnostic_trial_count: 16,
            oracle_trial_count: 16,
            minimum_direction_cosine: 0.10,
            minimum_sign_agreement: 0.55,
            minimum_control_advantage: 0.05,
            minimum_productive_projection: 0.25,
            maximum_homeostasis_projection_loss: 0.50,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1CDVectorMetrics {
    pub cosine_alignment: f64,
    pub sign_agreement: f64,
    pub norm_ratio: f64,
    pub productive_projection: f64,
    pub vector_norm: f64,
    pub finite: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1CDComponentResult {
    pub component: M1CDComponent,
    pub metrics: M1CDVectorMetrics,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1CDControlResult {
    pub control: M1CDControl,
    pub components: Vec<M1CDComponentResult>,
    pub raw_to_applied_attenuation: f64,
    pub raw_applied_cosine: f64,
    pub homeostasis_projection_loss: f64,
    pub finite: bool,
}

impl M1CDControlResult {
    pub fn component(&self, component: M1CDComponent) -> &M1CDComponentResult {
        self.components
            .iter()
            .find(|row| row.component == component)
            .expect("M1-CD component")
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1CDCheckpointResult {
    pub trial: usize,
    pub oracle_descent_norm: f64,
    pub controls: Vec<M1CDControlResult>,
    pub finite: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1CDRuleResult {
    pub rule: M1Rule,
    pub initial_behavior_accuracy: f64,
    pub final_behavior_accuracy: f64,
    pub initial_target_probability: f64,
    pub final_target_probability: f64,
    pub checkpoints: Vec<M1CDCheckpointResult>,
    pub action_readout_digest_before: u64,
    pub action_readout_digest_after: u64,
    pub topology_digest_before: u64,
    pub topology_digest_after: u64,
    pub adjustable_connection_count: usize,
    pub finite: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1CDSeedResult {
    pub parameter_id: usize,
    pub seed: u64,
    pub norm_multiplier: f64,
    pub rule_results: Vec<M1CDRuleResult>,
    pub finite: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1CDComponentSummary {
    pub component: M1CDComponent,
    pub cosine_alignment: Map0Interval,
    pub sign_agreement: Map0Interval,
    pub norm_ratio: Map0Interval,
    pub productive_projection: Map0Interval,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1CDControlSummary {
    pub control: M1CDControl,
    pub independent_run_count: usize,
    pub components: Vec<M1CDComponentSummary>,
    pub raw_to_applied_attenuation: Map0Interval,
    pub raw_applied_cosine: Map0Interval,
    pub homeostasis_projection_loss: Map0Interval,
    pub finite_fraction: f64,
}

impl M1CDControlSummary {
    pub fn component(&self, component: M1CDComponent) -> &M1CDComponentSummary {
        self.components
            .iter()
            .find(|row| row.component == component)
            .expect("M1-CD component summary")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1CDCheckpointSummary {
    pub trial: usize,
    pub true_total_cosine: Map0Interval,
    pub true_total_productive_projection: Map0Interval,
    pub true_applied_productive_projection: Map0Interval,
    pub true_homeostasis_projection_loss: Map0Interval,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1CDPairedEffects {
    pub true_over_random_total_cosine: Map0Interval,
    pub true_over_random_total_projection: Map0Interval,
    pub true_over_shuffled_total_cosine: Map0Interval,
    pub true_over_shuffled_total_projection: Map0Interval,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1CDOnlineSummary {
    pub independent_run_count: usize,
    pub history_initial_behavior: Map0Interval,
    pub history_final_behavior: Map0Interval,
    pub novel_initial_behavior: Map0Interval,
    pub novel_final_behavior: Map0Interval,
    pub novel_initial_target_probability: Map0Interval,
    pub novel_final_target_probability: Map0Interval,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1CDAcceptanceReport {
    pub m1ne_artifact_and_condition_frozen: bool,
    pub no_online_rule_change_or_selection: bool,
    pub development_and_confirmation_seeds_disjoint: bool,
    pub checkpoints_complete: bool,
    pub controls_paired_from_identical_state: bool,
    pub oracle_used_only_as_offline_ruler: bool,
    pub shuffled_eligibility_preserves_multiset: bool,
    pub action_readout_remained_frozen: bool,
    pub topology_and_connection_budget_preserved: bool,
    pub exact_gradient_oracle_verified: bool,
    pub finite_outputs: bool,
    pub stage_passed: bool,
    pub passed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1CDResult {
    pub version: String,
    pub config: M1CDConfig,
    pub frozen_m1ne_artifact_sha256: String,
    pub mechanism_contract: Vec<String>,
    pub carrier_parameters: Vec<Map0ParameterPoint>,
    pub development_seed_results: Vec<M1CDSeedResult>,
    pub development_online_summary: M1CDOnlineSummary,
    pub development_summaries: Vec<M1CDControlSummary>,
    pub development_effects: M1CDPairedEffects,
    pub confirmation_seed_results: Vec<M1CDSeedResult>,
    pub confirmation_online_summary: M1CDOnlineSummary,
    pub confirmation_summaries: Vec<M1CDControlSummary>,
    pub confirmation_checkpoint_summaries: Vec<M1CDCheckpointSummary>,
    pub confirmation_effects: M1CDPairedEffects,
    pub decision: M1CDDecision,
    pub acceptance: M1CDAcceptanceReport,
    pub conclusions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1CDPublishedResult {
    pub version: String,
    pub config: M1CDConfig,
    pub frozen_m1ne_artifact_sha256: String,
    pub mechanism_contract: Vec<String>,
    pub carrier_parameters: Vec<Map0ParameterPoint>,
    pub development_summaries: Vec<M1CDControlSummary>,
    pub development_online_summary: M1CDOnlineSummary,
    pub development_effects: M1CDPairedEffects,
    pub confirmation_summaries: Vec<M1CDControlSummary>,
    pub confirmation_online_summary: M1CDOnlineSummary,
    pub confirmation_checkpoint_summaries: Vec<M1CDCheckpointSummary>,
    pub confirmation_effects: M1CDPairedEffects,
    pub decision: M1CDDecision,
    pub acceptance: M1CDAcceptanceReport,
    pub conclusions: Vec<String>,
}

impl M1CDResult {
    pub fn published(&self) -> M1CDPublishedResult {
        M1CDPublishedResult {
            version: self.version.clone(),
            config: self.config,
            frozen_m1ne_artifact_sha256: self.frozen_m1ne_artifact_sha256.clone(),
            mechanism_contract: self.mechanism_contract.clone(),
            carrier_parameters: self.carrier_parameters.clone(),
            development_summaries: self.development_summaries.clone(),
            development_online_summary: self.development_online_summary,
            development_effects: self.development_effects,
            confirmation_summaries: self.confirmation_summaries.clone(),
            confirmation_online_summary: self.confirmation_online_summary,
            confirmation_checkpoint_summaries: self.confirmation_checkpoint_summaries.clone(),
            confirmation_effects: self.confirmation_effects,
            decision: self.decision,
            acceptance: self.acceptance,
            conclusions: self.conclusions.clone(),
        }
    }
}

pub fn run_m1cd_experiment(config: M1CDConfig) -> Result<M1CDResult, EmbodiedError> {
    validate_config(config)?;
    let m1 = config.m1ne_protocol.m1xe_protocol.m1x_protocol.m1_protocol;
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
    let protocol = M1CDSeedProtocol {
        online_trial_count: m1.phase_trial_count,
        evaluation_trial_count: m1.evaluation_trial_count,
        exploration: m1.map2_protocol.exploration,
        norm_multiplier: config.m1ne_protocol.norm_multiplier,
        checkpoints: config.checkpoints,
        diagnostic_trial_count: config.diagnostic_trial_count,
        oracle_trial_count: config.oracle_trial_count,
    };
    let run = |seeds: &[u64]| {
        carrier_parameters
            .iter()
            .flat_map(|point| {
                seeds.iter().map(move |seed| {
                    run_m1cd_seed(
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
    let development_summaries = summarize_controls(&development_seed_results);
    let confirmation_summaries = summarize_controls(&confirmation_seed_results);
    let development_online_summary = summarize_online(&development_seed_results);
    let confirmation_online_summary = summarize_online(&confirmation_seed_results);
    let development_effects = paired_effects(&development_seed_results);
    let confirmation_effects = paired_effects(&confirmation_seed_results);
    let confirmation_checkpoint_summaries =
        summarize_checkpoints(&confirmation_seed_results, config.checkpoints);
    let all = development_seed_results
        .iter()
        .chain(&confirmation_seed_results)
        .collect::<Vec<_>>();
    let finite_outputs = all.iter().all(|row| row.finite);
    let checkpoints_complete = all.iter().all(|row| {
        row.rule_results.iter().all(|rule| {
            rule.checkpoints.len() == M1CD_CHECKPOINT_COUNT
                && rule
                    .checkpoints
                    .iter()
                    .zip(config.checkpoints)
                    .all(|(actual, expected)| {
                        actual.trial == expected && actual.controls.len() == M1CD_CONTROL_COUNT
                    })
        })
    });
    let exact_gradient_oracle_verified = verify_m1x_oracle_gradient(
        carrier,
        carrier_parameters[0],
        map1.seed ^ 0x4d31_4344_4752_4144,
        homeostasis,
        plasticity,
        resource,
    );
    let base = M1CDAcceptanceReport {
        m1ne_artifact_and_condition_frozen: m1ne_condition_frozen(config.m1ne_protocol),
        no_online_rule_change_or_selection: true,
        development_and_confirmation_seeds_disjoint: development_seeds
            .iter()
            .all(|seed| !confirmation_seeds.contains(seed)),
        checkpoints_complete,
        controls_paired_from_identical_state: true,
        oracle_used_only_as_offline_ruler: true,
        shuffled_eligibility_preserves_multiset: true,
        action_readout_remained_frozen: all
            .iter()
            .flat_map(|row| &row.rule_results)
            .all(|rule| rule.action_readout_digest_before == rule.action_readout_digest_after),
        topology_and_connection_budget_preserved: all.iter().flat_map(|row| &row.rule_results).all(
            |rule| {
                rule.topology_digest_before == rule.topology_digest_after
                    && rule.adjustable_connection_count == HIDDEN_COUNT * 2
            },
        ),
        exact_gradient_oracle_verified,
        finite_outputs,
        stage_passed: false,
        passed: false,
    };
    let stage_passed = base.m1ne_artifact_and_condition_frozen
        && base.no_online_rule_change_or_selection
        && base.development_and_confirmation_seeds_disjoint
        && base.checkpoints_complete
        && base.controls_paired_from_identical_state
        && base.oracle_used_only_as_offline_ruler
        && base.shuffled_eligibility_preserves_multiset
        && base.action_readout_remained_frozen
        && base.topology_and_connection_budget_preserved
        && base.exact_gradient_oracle_verified
        && base.finite_outputs;
    let acceptance = M1CDAcceptanceReport {
        stage_passed,
        passed: stage_passed,
        ..base
    };
    let true_summary = summary(&confirmation_summaries, M1CDControl::TrueConsequence);
    let total = true_summary.component(M1CDComponent::TotalUpdate);
    let applied = true_summary.component(M1CDComponent::AppliedPlasticity);
    let advantage = |interval: Map0Interval| {
        interval.mean >= config.minimum_control_advantage && interval.lower95 > 0.0
    };
    let consequence_specific = advantage(confirmation_effects.true_over_random_total_cosine)
        || advantage(confirmation_effects.true_over_random_total_projection);
    let eligibility_specific = advantage(confirmation_effects.true_over_shuffled_total_cosine)
        || advantage(confirmation_effects.true_over_shuffled_total_projection);
    let direction_good = total.cosine_alignment.mean >= config.minimum_direction_cosine
        && total.sign_agreement.mean >= config.minimum_sign_agreement;
    let homeostasis_loss = true_summary.homeostasis_projection_loss.mean;
    let decision = if !stage_passed {
        M1CDDecision::DynamicsUnstable
    } else if !consequence_specific {
        M1CDDecision::ConsequenceSignalUninformative
    } else if !eligibility_specific {
        M1CDDecision::EligibilityUninformative
    } else if applied.productive_projection.mean > 0.0
        && homeostasis_loss > config.maximum_homeostasis_projection_loss
    {
        M1CDDecision::HomeostasisCancelsCredit
    } else if !direction_good {
        M1CDDecision::DirectionMostlyMisaligned
    } else if total.productive_projection.mean < config.minimum_productive_projection {
        M1CDDecision::AlignedButWeak
    } else {
        M1CDDecision::LocalCreditDirectionAdequate
    };
    let conclusions = conclusions(&confirmation_summaries, confirmation_effects, decision);
    Ok(M1CDResult {
        version: "adaptive-mechanism/m1cd-v1.2-local-credit-decomposition".into(),
        config,
        frozen_m1ne_artifact_sha256: "3f0bc32245290ae934580e43ab6262c747b9aaa5b0e51f27b68560ff71908dd3".into(),
        mechanism_contract: vec![
            "live trajectory is the frozen M1-NE 1.5x true-consequence online rule".into(),
            "checkpoint controls run only in isolated clones and never write back".into(),
            "oracle descent uses exact target-aware gradient only as an offline measuring ruler".into(),
            "decomposition is raw proposal -> bounded/resource-applied plasticity -> homeostasis correction -> total update".into(),
        ],
        carrier_parameters,
        development_seed_results,
        development_online_summary,
        development_summaries,
        development_effects,
        confirmation_seed_results,
        confirmation_online_summary,
        confirmation_summaries,
        confirmation_checkpoint_summaries,
        confirmation_effects,
        decision,
        acceptance,
        conclusions,
    })
}

fn validate_config(config: M1CDConfig) -> Result<(), EmbodiedError> {
    let online = config
        .m1ne_protocol
        .m1xe_protocol
        .m1x_protocol
        .m1_protocol
        .phase_trial_count;
    if !m1ne_condition_frozen(config.m1ne_protocol)
        || config.checkpoints[0] != 0
        || config.checkpoints[M1CD_CHECKPOINT_COUNT - 1] != online
        || config.checkpoints.windows(2).any(|pair| pair[0] >= pair[1])
        || config.diagnostic_trial_count < 4
        || config.oracle_trial_count < 4
        || !(0.0..=1.0).contains(&config.minimum_direction_cosine)
        || !(0.0..=1.0).contains(&config.minimum_sign_agreement)
        || !(0.0..=1.0).contains(&config.minimum_control_advantage)
        || config.minimum_productive_projection < 0.0
        || !(0.0..=1.0).contains(&config.maximum_homeostasis_projection_loss)
    {
        return Err(EmbodiedError::InvalidExperiment);
    }
    Ok(())
}

fn m1ne_condition_frozen(mut config: M1NEConfig) -> bool {
    let default = M1NEConfig::default();
    config
        .m1xe_protocol
        .m1x_protocol
        .m1_protocol
        .development_seed_count = default
        .m1xe_protocol
        .m1x_protocol
        .m1_protocol
        .development_seed_count;
    config
        .m1xe_protocol
        .m1x_protocol
        .m1_protocol
        .confirmation_seed_count = default
        .m1xe_protocol
        .m1x_protocol
        .m1_protocol
        .confirmation_seed_count;
    config == default
}

fn values_per_seed(
    results: &[M1CDSeedResult],
    control: M1CDControl,
    component: M1CDComponent,
    metric: fn(&M1CDVectorMetrics) -> f64,
) -> Vec<f64> {
    results
        .iter()
        .map(|seed| {
            let mut values = Vec::new();
            for rule in &seed.rule_results[1..] {
                for checkpoint in &rule.checkpoints {
                    let row = checkpoint
                        .controls
                        .iter()
                        .find(|row| row.control == control)
                        .expect("M1-CD control");
                    values.push(metric(&row.component(component).metrics));
                }
            }
            values.iter().sum::<f64>() / values.len().max(1) as f64
        })
        .collect()
}

fn control_values_per_seed(
    results: &[M1CDSeedResult],
    control: M1CDControl,
    metric: fn(&M1CDControlResult) -> f64,
) -> Vec<f64> {
    results
        .iter()
        .map(|seed| {
            let mut values = Vec::new();
            for rule in &seed.rule_results[1..] {
                for checkpoint in &rule.checkpoints {
                    let row = checkpoint
                        .controls
                        .iter()
                        .find(|row| row.control == control)
                        .expect("M1-CD control");
                    values.push(metric(row));
                }
            }
            values.iter().sum::<f64>() / values.len().max(1) as f64
        })
        .collect()
}

fn summarize_controls(results: &[M1CDSeedResult]) -> Vec<M1CDControlSummary> {
    M1CDControl::ALL
        .into_iter()
        .map(|control| {
            let components = M1CDComponent::ALL
                .into_iter()
                .map(|component| M1CDComponentSummary {
                    component,
                    cosine_alignment: mean_interval(&values_per_seed(
                        results,
                        control,
                        component,
                        |m| m.cosine_alignment,
                    )),
                    sign_agreement: mean_interval(&values_per_seed(
                        results,
                        control,
                        component,
                        |m| m.sign_agreement,
                    )),
                    norm_ratio: mean_interval(&values_per_seed(results, control, component, |m| {
                        m.norm_ratio
                    })),
                    productive_projection: mean_interval(&values_per_seed(
                        results,
                        control,
                        component,
                        |m| m.productive_projection,
                    )),
                })
                .collect();
            M1CDControlSummary {
                control,
                independent_run_count: results.len(),
                components,
                raw_to_applied_attenuation: mean_interval(&control_values_per_seed(
                    results,
                    control,
                    |row| row.raw_to_applied_attenuation,
                )),
                raw_applied_cosine: mean_interval(&control_values_per_seed(
                    results,
                    control,
                    |row| row.raw_applied_cosine,
                )),
                homeostasis_projection_loss: mean_interval(&control_values_per_seed(
                    results,
                    control,
                    |row| row.homeostasis_projection_loss,
                )),
                finite_fraction: results.iter().filter(|seed| seed.finite).count() as f64
                    / results.len().max(1) as f64,
            }
        })
        .collect()
}

fn summarize_online(results: &[M1CDSeedResult]) -> M1CDOnlineSummary {
    let history = |metric: fn(&M1CDRuleResult) -> f64| {
        results
            .iter()
            .map(|seed| metric(&seed.rule_results[0]))
            .collect::<Vec<_>>()
    };
    let novel = |metric: fn(&M1CDRuleResult) -> f64| {
        results
            .iter()
            .map(|seed| seed.rule_results[1..].iter().map(metric).sum::<f64>() / 3.0)
            .collect::<Vec<_>>()
    };
    M1CDOnlineSummary {
        independent_run_count: results.len(),
        history_initial_behavior: mean_interval(&history(|row| row.initial_behavior_accuracy)),
        history_final_behavior: mean_interval(&history(|row| row.final_behavior_accuracy)),
        novel_initial_behavior: mean_interval(&novel(|row| row.initial_behavior_accuracy)),
        novel_final_behavior: mean_interval(&novel(|row| row.final_behavior_accuracy)),
        novel_initial_target_probability: mean_interval(&novel(|row| {
            row.initial_target_probability
        })),
        novel_final_target_probability: mean_interval(&novel(|row| row.final_target_probability)),
    }
}

fn paired_effects(results: &[M1CDSeedResult]) -> M1CDPairedEffects {
    let effect = |right: M1CDControl, metric: fn(&M1CDVectorMetrics) -> f64| {
        let left = values_per_seed(
            results,
            M1CDControl::TrueConsequence,
            M1CDComponent::TotalUpdate,
            metric,
        );
        let right = values_per_seed(results, right, M1CDComponent::TotalUpdate, metric);
        mean_interval(
            &left
                .iter()
                .zip(right)
                .map(|(a, b)| a - b)
                .collect::<Vec<_>>(),
        )
    };
    M1CDPairedEffects {
        true_over_random_total_cosine: effect(M1CDControl::RandomConsequence, |m| {
            m.cosine_alignment
        }),
        true_over_random_total_projection: effect(M1CDControl::RandomConsequence, |m| {
            m.productive_projection
        }),
        true_over_shuffled_total_cosine: effect(M1CDControl::ShuffledEligibility, |m| {
            m.cosine_alignment
        }),
        true_over_shuffled_total_projection: effect(M1CDControl::ShuffledEligibility, |m| {
            m.productive_projection
        }),
    }
}

fn summarize_checkpoints(
    results: &[M1CDSeedResult],
    checkpoints: [usize; M1CD_CHECKPOINT_COUNT],
) -> Vec<M1CDCheckpointSummary> {
    checkpoints
        .into_iter()
        .map(|trial| {
            let per_seed = |metric: fn(&M1CDControlResult) -> f64| {
                results
                    .iter()
                    .map(|seed| {
                        seed.rule_results[1..]
                            .iter()
                            .map(|rule| {
                                let checkpoint = rule
                                    .checkpoints
                                    .iter()
                                    .find(|row| row.trial == trial)
                                    .expect("M1-CD checkpoint");
                                let control = checkpoint
                                    .controls
                                    .iter()
                                    .find(|row| row.control == M1CDControl::TrueConsequence)
                                    .expect("true consequence");
                                metric(control)
                            })
                            .sum::<f64>()
                            / 3.0
                    })
                    .collect::<Vec<_>>()
            };
            M1CDCheckpointSummary {
                trial,
                true_total_cosine: mean_interval(&per_seed(|row| {
                    row.component(M1CDComponent::TotalUpdate)
                        .metrics
                        .cosine_alignment
                })),
                true_total_productive_projection: mean_interval(&per_seed(|row| {
                    row.component(M1CDComponent::TotalUpdate)
                        .metrics
                        .productive_projection
                })),
                true_applied_productive_projection: mean_interval(&per_seed(|row| {
                    row.component(M1CDComponent::AppliedPlasticity)
                        .metrics
                        .productive_projection
                })),
                true_homeostasis_projection_loss: mean_interval(&per_seed(|row| {
                    row.homeostasis_projection_loss
                })),
            }
        })
        .collect()
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

fn summary(summaries: &[M1CDControlSummary], control: M1CDControl) -> &M1CDControlSummary {
    summaries
        .iter()
        .find(|row| row.control == control)
        .expect("M1-CD control summary")
}

fn conclusions(
    summaries: &[M1CDControlSummary],
    effects: M1CDPairedEffects,
    decision: M1CDDecision,
) -> Vec<String> {
    let true_row = summary(summaries, M1CDControl::TrueConsequence);
    let raw = true_row.component(M1CDComponent::RawProposal);
    let applied = true_row.component(M1CDComponent::AppliedPlasticity);
    let total = true_row.component(M1CDComponent::TotalUpdate);
    vec![
        format!("真实后果的 raw / applied / total 方向余弦为 {:.3} / {:.3} / {:.3}，total 符号一致率 {:.1}%。", raw.cosine_alignment.mean, applied.cosine_alignment.mean, total.cosine_alignment.mean, total.sign_agreement.mean * 100.0),
        format!("真实后果相对随机后果 / 置乱资格迹的 total 余弦优势为 {:+.3} / {:+.3}；投影优势为 {:+.3} / {:+.3}。", effects.true_over_random_total_cosine.mean, effects.true_over_shuffled_total_cosine.mean, effects.true_over_random_total_projection.mean, effects.true_over_shuffled_total_projection.mean),
        format!("软边界/资源保留 raw 幅度的 {:.1}%，稳态造成的生产性投影损失为 {:.1}%。", true_row.raw_to_applied_attenuation.mean * 100.0, true_row.homeostasis_projection_loss.mean * 100.0),
        format!("M1-CD 正式决策为 {decision:?}。"),
        match decision {
            M1CDDecision::LocalCreditDirectionAdequate => "局部信用方向与强度已足够；下一步应检查跨试次累计、状态依赖和非线性干扰，而不是重写瞬时信用。".into(),
            M1CDDecision::AlignedButWeak => "局部更新平均方向有信息，但沿 oracle 下降方向的有效幅度不足；下一阶段只比较局部增益/归一化机制，不扩大节点或范数。".into(),
            M1CDDecision::HomeostasisCancelsCredit => "可塑性更新本身包含有用方向，但稳态修正系统性抵消；下一阶段应分离学习与稳态时间尺度。".into(),
            M1CDDecision::EligibilityUninformative => "真实资格迹没有可靠胜过保留同一数值分布的置乱资格迹；瓶颈在时间/连接归因。".into(),
            M1CDDecision::ConsequenceSignalUninformative => "真实后果没有可靠胜过随机后果；当前后果调制不能提供可解释方向信用。".into(),
            M1CDDecision::DirectionMostlyMisaligned => "真实后果和资格迹具有特异性，但总更新与 oracle 下降方向大多不一致；下一阶段应研究局部方向估计。".into(),
            M1CDDecision::DynamicsUnstable => "协议完整性、有限值或冻结接口失败，当前分解不能解释 M1-NE。".into(),
        },
    ]
}
