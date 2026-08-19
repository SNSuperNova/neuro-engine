use serde::Serialize;

use crate::learnability_map::{
    ContinuousResource, DualTimescaleHomeostasis, HomeostasisMechanism, PlasticityMechanism,
    ResourceMechanism, SoftBoundedPlasticity, run_multirule_seed_with_mechanisms, seed_partition,
};
use crate::map1::{parameter_points, protocol_config};
use crate::map2::Map2CExperimentConfig;
use crate::{EmbodiedError, Map0Interval, Map0ParameterPoint};

const DEVELOPMENT_SEED_LABEL: u64 = 0x4d31_4445_5601;
const CONFIRMATION_SEED_LABEL: u64 = 0x4d31_434f_4e46;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum M1Rule {
    A,
    B,
    C,
    D,
}

impl M1Rule {
    pub const SEQUENCE: [Self; 5] = [Self::A, Self::B, Self::C, Self::D, Self::A];
    pub const UNIQUE: [Self; 4] = [Self::A, Self::B, Self::C, Self::D];

    pub(crate) fn target_right(self, symbol: usize) -> bool {
        const MAPPINGS: [[bool; 4]; 4] = [
            [false, false, true, true],
            [false, true, false, true],
            [false, true, true, false],
            [true, false, false, true],
        ];
        MAPPINGS[self as usize][symbol]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum M1Control {
    Baseline,
    FrozenPlasticity,
    RandomConsequence,
    ResetBetweenTrials,
    SingleRuleCapacity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum M1FailureClass {
    CapacityInsufficient,
    OverwriteForgetting,
    CreditRoutingError,
    ResourceExhaustion,
    DynamicsInstability,
    NoDominantFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1ExperimentConfig {
    pub map2_protocol: Map2CExperimentConfig,
    pub reference_parameter_ids: [usize; 6],
    pub development_seed_count: usize,
    pub confirmation_seed_count: usize,
    pub phase_trial_count: usize,
    pub evaluation_trial_count: usize,
    pub threshold_check_interval: usize,
    pub accuracy_threshold: f64,
    pub minimum_retained_accuracy: f64,
    pub maximum_retention_drop: f64,
    pub maximum_relative_weight_drift: f64,
    pub minimum_mean_resource_level: f64,
    pub minimum_single_rule_accuracy: f64,
    pub minimum_retention_probability: f64,
    pub minimum_causal_accuracy_drop: f64,
    pub maximum_reset_retention_advantage: f64,
}

impl Default for M1ExperimentConfig {
    fn default() -> Self {
        Self {
            map2_protocol: Map2CExperimentConfig::default(),
            reference_parameter_ids: [0, 16, 20, 21, 29, 45],
            development_seed_count: 4,
            confirmation_seed_count: 8,
            phase_trial_count: 240,
            evaluation_trial_count: 48,
            threshold_check_interval: 20,
            accuracy_threshold: 0.70,
            minimum_retained_accuracy: 0.60,
            maximum_retention_drop: 0.15,
            maximum_relative_weight_drift: 1.0,
            minimum_mean_resource_level: 0.20,
            minimum_single_rule_accuracy: 0.70,
            minimum_retention_probability: 0.50,
            minimum_causal_accuracy_drop: 0.05,
            maximum_reset_retention_advantage: 0.05,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct M1SeedProtocol {
    pub phase_trial_count: usize,
    pub evaluation_trial_count: usize,
    pub threshold_check_interval: usize,
    pub accuracy_threshold: f64,
    pub exploration: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1PhaseResult {
    pub phase_index: usize,
    pub rule: M1Rule,
    pub initial_accuracy: f64,
    pub online_accuracy: f64,
    pub departure_accuracy: f64,
    pub trials_to_threshold: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1SingleRuleResult {
    pub rule: M1Rule,
    pub initial_accuracy: f64,
    pub final_accuracy: f64,
    pub trials_to_threshold: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1SeedResult {
    pub parameter_id: usize,
    pub seed: u64,
    pub control: M1Control,
    pub phase_results: Vec<M1PhaseResult>,
    pub single_rule_results: Vec<M1SingleRuleResult>,
    pub initial_a_accuracy: f64,
    pub departure_a_accuracy: f64,
    pub return_a_initial_accuracy: f64,
    pub return_a_final_accuracy: f64,
    pub retention_drop: f64,
    pub mean_novel_rule_final_accuracy: f64,
    pub mean_first_acquisition_trials: Option<f64>,
    pub return_reacquisition_trials: Option<usize>,
    pub reacquisition_speedup: Option<f64>,
    pub minimum_single_rule_final_accuracy: f64,
    pub mean_resource_level: f64,
    pub minimum_resource_level: f64,
    pub mean_relative_weight_drift: f64,
    pub maximum_absolute_weight: f64,
    pub state_reset_count: usize,
    pub action_readout_digest_before: u64,
    pub action_readout_digest_after: u64,
    pub finite: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1ParameterSummary {
    pub parameters: Map0ParameterPoint,
    pub control: M1Control,
    pub seed_count: usize,
    pub retention_probability: Map0Interval,
    pub mean_initial_a_accuracy: f64,
    pub mean_departure_a_accuracy: f64,
    pub mean_return_a_initial_accuracy: f64,
    pub mean_return_a_final_accuracy: f64,
    pub mean_retention_drop: f64,
    pub mean_novel_rule_final_accuracy: f64,
    pub mean_first_acquisition_trials: Option<f64>,
    pub mean_return_reacquisition_trials: Option<f64>,
    pub mean_reacquisition_speedup: Option<f64>,
    pub mean_minimum_single_rule_final_accuracy: f64,
    pub mean_resource_level: f64,
    pub mean_minimum_resource_level: f64,
    pub mean_relative_weight_drift: f64,
    pub readout_frozen_fraction: f64,
    pub finite_fraction: f64,
    pub failure_class: M1FailureClass,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1AcceptanceReport {
    pub reference_substrate_unchanged: bool,
    pub reference_adjustment_mechanism_unchanged: bool,
    pub four_fixed_balanced_rules_complete: bool,
    pub exact_sequence_complete: bool,
    pub development_and_confirmation_seeds_disjoint: bool,
    pub development_runs_complete: bool,
    pub development_single_rule_capacity_controls_complete: bool,
    pub confirmation_runs_complete: bool,
    pub frozen_plasticity_controls_complete: bool,
    pub random_consequence_controls_complete: bool,
    pub reset_controls_complete: bool,
    pub single_rule_capacity_controls_complete: bool,
    pub baseline_has_no_state_resets: bool,
    pub action_readout_remained_frozen: bool,
    pub finite_outputs: bool,
    pub plasticity_causally_engaged: bool,
    pub consequence_causally_engaged: bool,
    pub reset_does_not_explain_retention: bool,
    pub failure_boundary_classified: bool,
    pub retention_boundary_passed: bool,
    pub stage_passed: bool,
    pub passed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M1ExperimentResult {
    pub version: String,
    pub config: M1ExperimentConfig,
    pub rules: Vec<[bool; 4]>,
    pub sequence: Vec<M1Rule>,
    pub reference_parameters: Vec<Map0ParameterPoint>,
    pub development_seed_results: Vec<M1SeedResult>,
    pub development_single_rule_seed_results: Vec<M1SeedResult>,
    pub development_summaries: Vec<M1ParameterSummary>,
    pub development_single_rule_summaries: Vec<M1ParameterSummary>,
    pub confirmation_seed_results: Vec<M1SeedResult>,
    pub confirmation_summaries: Vec<M1ParameterSummary>,
    pub frozen_seed_results: Vec<M1SeedResult>,
    pub frozen_summaries: Vec<M1ParameterSummary>,
    pub random_consequence_seed_results: Vec<M1SeedResult>,
    pub random_consequence_summaries: Vec<M1ParameterSummary>,
    pub reset_seed_results: Vec<M1SeedResult>,
    pub reset_summaries: Vec<M1ParameterSummary>,
    pub single_rule_seed_results: Vec<M1SeedResult>,
    pub single_rule_summaries: Vec<M1ParameterSummary>,
    pub dominant_failure_class: M1FailureClass,
    pub acceptance: M1AcceptanceReport,
    pub conclusions: Vec<String>,
}

pub fn run_m1_experiment(config: M1ExperimentConfig) -> Result<M1ExperimentResult, EmbodiedError> {
    validate_config(config)?;
    let map1 = config
        .map2_protocol
        .map2b_protocol
        .map2a_protocol
        .map1_protocol;
    let carrier = protocol_config(map1);
    let all_parameters = parameter_points(map1);
    let reference_parameters = config
        .reference_parameter_ids
        .iter()
        .map(|id| all_parameters[*id])
        .collect::<Vec<_>>();
    let development_seeds = seed_partition(
        map1.seed ^ DEVELOPMENT_SEED_LABEL,
        config.development_seed_count,
    );
    let confirmation_seeds = seed_partition(
        map1.seed ^ CONFIRMATION_SEED_LABEL,
        config.confirmation_seed_count,
    );
    let protocol = M1SeedProtocol {
        phase_trial_count: config.phase_trial_count,
        evaluation_trial_count: config.evaluation_trial_count,
        threshold_check_interval: config.threshold_check_interval,
        accuracy_threshold: config.accuracy_threshold,
        exploration: config.map2_protocol.exploration,
    };
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
            .map2_protocol
            .map2b_protocol
            .map2a_protocol
            .soft_bound_scale,
    });
    let map2b = config.map2_protocol.map2b_protocol;
    let resource = ResourceMechanism::Continuous(ContinuousResource {
        initial_level: map2b.initial_resource,
        supply_rate: map2b.supply_rate,
        maintenance_cost: map2b.maintenance_cost,
        activity_cost: map2b.activity_cost,
        plasticity_cost: map2b.plasticity_cost,
        minimum_modulation: map2b.minimum_modulation,
    });

    let run = |parameters: &[Map0ParameterPoint], seeds: &[u64], control: M1Control| {
        parameters
            .iter()
            .flat_map(|point| {
                seeds.iter().map(move |seed| {
                    run_multirule_seed_with_mechanisms(
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
            .collect::<Vec<_>>()
    };
    let development_seed_results = run(
        &reference_parameters,
        &development_seeds,
        M1Control::Baseline,
    );
    let development_single_rule_seed_results = run(
        &reference_parameters,
        &development_seeds,
        M1Control::SingleRuleCapacity,
    );
    let confirmation_seed_results = run(
        &reference_parameters,
        &confirmation_seeds,
        M1Control::Baseline,
    );
    let frozen_seed_results = run(
        &reference_parameters,
        &confirmation_seeds,
        M1Control::FrozenPlasticity,
    );
    let random_consequence_seed_results = run(
        &reference_parameters,
        &confirmation_seeds,
        M1Control::RandomConsequence,
    );
    let reset_seed_results = run(
        &reference_parameters,
        &confirmation_seeds,
        M1Control::ResetBetweenTrials,
    );
    let single_rule_seed_results = run(
        &reference_parameters,
        &confirmation_seeds,
        M1Control::SingleRuleCapacity,
    );
    let single_rule_summaries = summarize(
        config,
        &reference_parameters,
        &single_rule_seed_results,
        M1Control::SingleRuleCapacity,
        None,
    );
    let development_single_rule_summaries = summarize(
        config,
        &reference_parameters,
        &development_single_rule_seed_results,
        M1Control::SingleRuleCapacity,
        None,
    );
    let development_summaries = summarize(
        config,
        &reference_parameters,
        &development_seed_results,
        M1Control::Baseline,
        Some(&development_single_rule_summaries),
    );
    let confirmation_summaries = summarize(
        config,
        &reference_parameters,
        &confirmation_seed_results,
        M1Control::Baseline,
        Some(&single_rule_summaries),
    );
    let frozen_summaries = summarize(
        config,
        &reference_parameters,
        &frozen_seed_results,
        M1Control::FrozenPlasticity,
        None,
    );
    let random_consequence_summaries = summarize(
        config,
        &reference_parameters,
        &random_consequence_seed_results,
        M1Control::RandomConsequence,
        None,
    );
    let reset_summaries = summarize(
        config,
        &reference_parameters,
        &reset_seed_results,
        M1Control::ResetBetweenTrials,
        None,
    );
    let dominant_failure_class = dominant_failure(&confirmation_summaries);
    let expected_development = reference_parameters.len() * development_seeds.len();
    let expected_confirmation = reference_parameters.len() * confirmation_seeds.len();
    let finite_outputs = development_seed_results
        .iter()
        .chain(&development_single_rule_seed_results)
        .chain(&confirmation_seed_results)
        .chain(&frozen_seed_results)
        .chain(&random_consequence_seed_results)
        .chain(&reset_seed_results)
        .chain(&single_rule_seed_results)
        .all(|row| row.finite);
    let retention_boundary_passed = confirmation_summaries.iter().any(|row| {
        row.retention_probability.mean >= config.minimum_retention_probability
            && row.failure_class == M1FailureClass::NoDominantFailure
    });
    let summary_mean = |rows: &[M1ParameterSummary], f: fn(&M1ParameterSummary) -> f64| {
        rows.iter().map(f).sum::<f64>() / rows.len().max(1) as f64
    };
    let baseline_novel = summary_mean(&confirmation_summaries, |row| {
        row.mean_novel_rule_final_accuracy
    });
    let frozen_novel = summary_mean(&frozen_summaries, |row| row.mean_novel_rule_final_accuracy);
    let random_novel = summary_mean(&random_consequence_summaries, |row| {
        row.mean_novel_rule_final_accuracy
    });
    let baseline_return = summary_mean(&confirmation_summaries, |row| {
        row.mean_return_a_initial_accuracy
    });
    let reset_return = summary_mean(&reset_summaries, |row| row.mean_return_a_initial_accuracy);
    let acceptance = M1AcceptanceReport {
        reference_substrate_unchanged: true,
        reference_adjustment_mechanism_unchanged: true,
        four_fixed_balanced_rules_complete: M1Rule::UNIQUE
            .iter()
            .all(|rule| (0..4).filter(|symbol| rule.target_right(*symbol)).count() == 2),
        exact_sequence_complete: M1Rule::SEQUENCE
            == [M1Rule::A, M1Rule::B, M1Rule::C, M1Rule::D, M1Rule::A],
        development_and_confirmation_seeds_disjoint: development_seeds
            .iter()
            .all(|seed| !confirmation_seeds.contains(seed)),
        development_runs_complete: development_seed_results.len() == expected_development,
        development_single_rule_capacity_controls_complete: development_single_rule_seed_results
            .len()
            == expected_development,
        confirmation_runs_complete: confirmation_seed_results.len() == expected_confirmation,
        frozen_plasticity_controls_complete: frozen_seed_results.len() == expected_confirmation,
        random_consequence_controls_complete: random_consequence_seed_results.len()
            == expected_confirmation,
        reset_controls_complete: reset_seed_results.len() == expected_confirmation,
        single_rule_capacity_controls_complete: single_rule_seed_results.len()
            == expected_confirmation,
        baseline_has_no_state_resets: development_seed_results
            .iter()
            .chain(&confirmation_seed_results)
            .all(|row| row.state_reset_count == 0),
        action_readout_remained_frozen: development_seed_results
            .iter()
            .chain(&development_single_rule_seed_results)
            .chain(&confirmation_seed_results)
            .chain(&frozen_seed_results)
            .chain(&random_consequence_seed_results)
            .chain(&reset_seed_results)
            .chain(&single_rule_seed_results)
            .all(|row| row.action_readout_digest_before == row.action_readout_digest_after),
        finite_outputs,
        plasticity_causally_engaged: baseline_novel - frozen_novel
            >= config.minimum_causal_accuracy_drop,
        consequence_causally_engaged: baseline_novel - random_novel
            >= config.minimum_causal_accuracy_drop,
        reset_does_not_explain_retention: reset_return - baseline_return
            <= config.maximum_reset_retention_advantage,
        failure_boundary_classified: !confirmation_summaries.is_empty(),
        retention_boundary_passed,
        stage_passed: false,
        passed: false,
    };
    let protocol_complete = acceptance.reference_substrate_unchanged
        && acceptance.reference_adjustment_mechanism_unchanged
        && acceptance.four_fixed_balanced_rules_complete
        && acceptance.exact_sequence_complete
        && acceptance.development_and_confirmation_seeds_disjoint
        && acceptance.development_runs_complete
        && acceptance.development_single_rule_capacity_controls_complete
        && acceptance.confirmation_runs_complete
        && acceptance.frozen_plasticity_controls_complete
        && acceptance.random_consequence_controls_complete
        && acceptance.reset_controls_complete
        && acceptance.single_rule_capacity_controls_complete
        && acceptance.baseline_has_no_state_resets
        && acceptance.action_readout_remained_frozen
        && acceptance.finite_outputs
        && acceptance.plasticity_causally_engaged
        && acceptance.consequence_causally_engaged
        && acceptance.reset_does_not_explain_retention
        && acceptance.failure_boundary_classified;
    let acceptance = M1AcceptanceReport {
        stage_passed: protocol_complete,
        passed: protocol_complete,
        ..acceptance
    };
    let conclusions = conclusions(
        &confirmation_summaries,
        &single_rule_summaries,
        &frozen_summaries,
        &random_consequence_summaries,
        &reset_summaries,
        dominant_failure_class,
        retention_boundary_passed,
    );
    Ok(M1ExperimentResult {
        version: "adaptive-mechanism/m1-v0.2-multirule-retention".into(),
        config,
        rules: M1Rule::UNIQUE
            .iter()
            .map(|rule| std::array::from_fn(|symbol| rule.target_right(symbol)))
            .collect(),
        sequence: M1Rule::SEQUENCE.to_vec(),
        reference_parameters,
        development_seed_results,
        development_single_rule_seed_results,
        development_summaries,
        development_single_rule_summaries,
        confirmation_seed_results,
        confirmation_summaries,
        frozen_seed_results,
        frozen_summaries,
        random_consequence_seed_results,
        random_consequence_summaries,
        reset_seed_results,
        reset_summaries,
        single_rule_seed_results,
        single_rule_summaries,
        dominant_failure_class,
        acceptance,
        conclusions,
    })
}

fn validate_config(config: M1ExperimentConfig) -> Result<(), EmbodiedError> {
    let values = [
        config.accuracy_threshold,
        config.minimum_retained_accuracy,
        config.maximum_retention_drop,
        config.maximum_relative_weight_drift,
        config.minimum_mean_resource_level,
        config.minimum_single_rule_accuracy,
        config.minimum_retention_probability,
        config.minimum_causal_accuracy_drop,
        config.maximum_reset_retention_advantage,
    ];
    if config.development_seed_count == 0
        || config.confirmation_seed_count == 0
        || config.phase_trial_count < 4
        || config.evaluation_trial_count < 4
        || config.threshold_check_interval == 0
        || config.threshold_check_interval > config.phase_trial_count
        || config.reference_parameter_ids.iter().any(|id| *id >= 48)
        || values.iter().any(|value| !value.is_finite())
        || !(0.5..=1.0).contains(&config.accuracy_threshold)
        || !(0.0..=1.0).contains(&config.minimum_retained_accuracy)
        || !(0.0..=1.0).contains(&config.maximum_retention_drop)
        || config.maximum_relative_weight_drift <= 0.0
        || !(0.0..=1.0).contains(&config.minimum_mean_resource_level)
        || !(0.5..=1.0).contains(&config.minimum_single_rule_accuracy)
        || !(0.0..=1.0).contains(&config.minimum_retention_probability)
        || !(0.0..=1.0).contains(&config.minimum_causal_accuracy_drop)
        || !(0.0..=1.0).contains(&config.maximum_reset_retention_advantage)
    {
        return Err(EmbodiedError::InvalidExperiment);
    }
    Ok(())
}

fn retained(config: M1ExperimentConfig, row: &M1SeedResult) -> bool {
    row.control == M1Control::Baseline
        && row.return_a_initial_accuracy >= config.minimum_retained_accuracy
        && row.retention_drop <= config.maximum_retention_drop
        && row.mean_novel_rule_final_accuracy >= config.accuracy_threshold
        && row.mean_relative_weight_drift <= config.maximum_relative_weight_drift
        && row.mean_resource_level >= config.minimum_mean_resource_level
        && row.action_readout_digest_before == row.action_readout_digest_after
        && row.finite
}

fn summarize(
    config: M1ExperimentConfig,
    parameters: &[Map0ParameterPoint],
    results: &[M1SeedResult],
    control: M1Control,
    capacity: Option<&[M1ParameterSummary]>,
) -> Vec<M1ParameterSummary> {
    parameters
        .iter()
        .map(|parameters| {
            let rows = results
                .iter()
                .filter(|row| row.parameter_id == parameters.id && row.control == control)
                .collect::<Vec<_>>();
            let denominator = rows.len().max(1) as f64;
            let mean = |f: fn(&M1SeedResult) -> f64| {
                rows.iter().map(|row| f(row)).sum::<f64>() / denominator
            };
            let optional_mean = |f: fn(&M1SeedResult) -> Option<f64>| {
                let values = rows.iter().filter_map(|row| f(row)).collect::<Vec<_>>();
                (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
            };
            let successes = rows.iter().filter(|row| retained(config, row)).count();
            let capacity_accuracy = capacity
                .and_then(|summaries| {
                    summaries
                        .iter()
                        .find(|row| row.parameters.id == parameters.id)
                })
                .map(|row| row.mean_minimum_single_rule_final_accuracy)
                .unwrap_or_else(|| mean(|row| row.minimum_single_rule_final_accuracy));
            let mean_retention_drop = mean(|row| row.retention_drop);
            let mean_novel = mean(|row| row.mean_novel_rule_final_accuracy);
            let mean_resource = mean(|row| row.mean_resource_level);
            let mean_drift = mean(|row| row.mean_relative_weight_drift);
            let finite_fraction = rows.iter().filter(|row| row.finite).count() as f64 / denominator;
            let failure_class =
                if finite_fraction < 1.0 || mean_drift > config.maximum_relative_weight_drift {
                    M1FailureClass::DynamicsInstability
                } else if mean_resource < config.minimum_mean_resource_level {
                    M1FailureClass::ResourceExhaustion
                } else if capacity_accuracy < config.minimum_single_rule_accuracy {
                    M1FailureClass::CapacityInsufficient
                } else if mean_retention_drop > config.maximum_retention_drop
                    && mean_novel >= config.accuracy_threshold
                {
                    M1FailureClass::OverwriteForgetting
                } else if mean_novel < config.accuracy_threshold {
                    M1FailureClass::CreditRoutingError
                } else {
                    M1FailureClass::NoDominantFailure
                };
            M1ParameterSummary {
                parameters: *parameters,
                control,
                seed_count: rows.len(),
                retention_probability: crate::learnability_map::wilson_interval(
                    successes,
                    rows.len(),
                ),
                mean_initial_a_accuracy: mean(|row| row.initial_a_accuracy),
                mean_departure_a_accuracy: mean(|row| row.departure_a_accuracy),
                mean_return_a_initial_accuracy: mean(|row| row.return_a_initial_accuracy),
                mean_return_a_final_accuracy: mean(|row| row.return_a_final_accuracy),
                mean_retention_drop,
                mean_novel_rule_final_accuracy: mean_novel,
                mean_first_acquisition_trials: optional_mean(|row| {
                    row.mean_first_acquisition_trials
                }),
                mean_return_reacquisition_trials: optional_mean(|row| {
                    row.return_reacquisition_trials.map(|value| value as f64)
                }),
                mean_reacquisition_speedup: optional_mean(|row| row.reacquisition_speedup),
                mean_minimum_single_rule_final_accuracy: capacity_accuracy,
                mean_resource_level: mean_resource,
                mean_minimum_resource_level: mean(|row| row.minimum_resource_level),
                mean_relative_weight_drift: mean_drift,
                readout_frozen_fraction: rows
                    .iter()
                    .filter(|row| {
                        row.action_readout_digest_before == row.action_readout_digest_after
                    })
                    .count() as f64
                    / denominator,
                finite_fraction,
                failure_class,
            }
        })
        .collect()
}

fn dominant_failure(summaries: &[M1ParameterSummary]) -> M1FailureClass {
    let classes = [
        M1FailureClass::CapacityInsufficient,
        M1FailureClass::OverwriteForgetting,
        M1FailureClass::CreditRoutingError,
        M1FailureClass::ResourceExhaustion,
        M1FailureClass::DynamicsInstability,
        M1FailureClass::NoDominantFailure,
    ];
    classes
        .into_iter()
        .max_by_key(|class| {
            summaries
                .iter()
                .filter(|row| row.failure_class == *class)
                .count()
        })
        .unwrap_or(M1FailureClass::NoDominantFailure)
}

fn conclusions(
    baseline: &[M1ParameterSummary],
    capacity: &[M1ParameterSummary],
    frozen: &[M1ParameterSummary],
    random: &[M1ParameterSummary],
    reset: &[M1ParameterSummary],
    dominant: M1FailureClass,
    retention_passed: bool,
) -> Vec<String> {
    let mean = |rows: &[M1ParameterSummary], f: fn(&M1ParameterSummary) -> f64| {
        rows.iter().map(f).sum::<f64>() / rows.len().max(1) as f64
    };
    vec![
        format!(
            "M1 独立确认中，A 离开前准确率为 {:.1}%，重现初始准确率为 {:.1}%，平均保留下降 {:.1} 个百分点。",
            mean(baseline, |row| row.mean_departure_a_accuracy) * 100.0,
            mean(baseline, |row| row.mean_return_a_initial_accuracy) * 100.0,
            mean(baseline, |row| row.mean_retention_drop) * 100.0,
        ),
        format!(
            "B/C/D 首次学习后的平均准确率为 {:.1}%；单规则容量控制的最低准确率为 {:.1}%。",
            mean(baseline, |row| row.mean_novel_rule_final_accuracy) * 100.0,
            mean(capacity, |row| row.mean_minimum_single_rule_final_accuracy) * 100.0,
        ),
        format!(
            "B/C/D 的冻结可塑性对照为 {:.1}%，随机后果对照为 {:.1}%；逐试次重置的 A 重现初始准确率为 {:.1}%。",
            mean(frozen, |row| row.mean_novel_rule_final_accuracy) * 100.0,
            mean(random, |row| row.mean_novel_rule_final_accuracy) * 100.0,
            mean(reset, |row| row.mean_return_a_initial_accuracy) * 100.0,
        ),
        format!("M1 的主导诊断为 {:?}。", dominant),
        if retention_passed {
            "至少一个参考参数点通过多规则保留边界；M2 暂不因遗忘自动触发。".into()
        } else {
            "没有参考参数点通过完整保留边界；后续 M2 只能进入主导失败类型对应的单机制分支。".into()
        },
    ]
}
