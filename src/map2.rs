use serde::Serialize;

use crate::learnability_map::{
    ContinuousResource, ContinuousSeedResult, ContinuousStreamConfig, DualTimescaleHomeostasis,
    HomeostasisMechanism, Map0Control, Map0ControlSummary, Map0Interval, Map0SeedResult,
    PlasticityMechanism, ResourceMechanism, SoftBoundedPlasticity, normalized_distance,
    result_is_finite, run_continuous_seed_with_mechanisms, run_map_seed_with_mechanisms,
    run_map_seed_with_plasticity, seed_partition, summarize, wilson_interval,
};
use crate::map1::{
    Map1ExperimentConfig, parameter_points, protocol_config, rank, same_probe_protocol,
    select_candidates, stable_region, validate_config as validate_map1_config,
};
use crate::{EmbodiedError, Map0ParameterPoint, Map0ParameterSummary};

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Map2AExperimentConfig {
    pub map1_protocol: Map1ExperimentConfig,
    pub soft_bound_scale: f64,
    pub minimum_mean_weight_drift_reduction: f64,
    pub maximum_accuracy_degradation: f64,
}

impl Default for Map2AExperimentConfig {
    fn default() -> Self {
        Self {
            map1_protocol: Map1ExperimentConfig::default(),
            soft_bound_scale: 1.0,
            minimum_mean_weight_drift_reduction: 0.25,
            maximum_accuracy_degradation: 0.02,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Map2AMechanismComparison {
    pub parameter_id: usize,
    pub soft_bounded_formation_probability: Map0Interval,
    pub additive_formation_probability: Map0Interval,
    pub formation_probability_change: f64,
    pub mean_probe_score_change: f64,
    pub repeated_reversal_accuracy_change: f64,
    pub perturbation_recovery_gain_change: f64,
    pub mean_weight_drift_change: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Map2AAcceptanceReport {
    pub only_plasticity_mechanism_changed: bool,
    pub development_and_confirmation_seeds_disjoint: bool,
    pub development_runs_complete: bool,
    pub confirmation_runs_complete: bool,
    pub additive_controls_complete: bool,
    pub causal_controls_complete: bool,
    pub causal_intervention_detected: bool,
    pub finite_outputs: bool,
    pub mean_weight_drift_reduced: bool,
    pub repeated_reversal_or_recovery_preserved: bool,
    pub stable_region_found: bool,
    pub stage_passed: bool,
    pub passed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Map2AExperimentResult {
    pub version: String,
    pub config: Map2AExperimentConfig,
    pub development_parameters: Vec<Map0ParameterPoint>,
    pub development_seed_results: Vec<Map0SeedResult>,
    pub development_summaries: Vec<Map0ParameterSummary>,
    pub confirmation_parameter_ids: Vec<usize>,
    pub confirmation_seed_results: Vec<Map0SeedResult>,
    pub confirmation_summaries: Vec<Map0ParameterSummary>,
    pub additive_seed_results: Vec<Map0SeedResult>,
    pub additive_summaries: Vec<Map0ParameterSummary>,
    pub mechanism_comparisons: Vec<Map2AMechanismComparison>,
    pub control_seed_results: Vec<Map0SeedResult>,
    pub control_summaries: Vec<Map0ControlSummary>,
    pub stable_region_parameter_ids: Vec<usize>,
    pub acceptance: Map2AAcceptanceReport,
    pub conclusions: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Map2BExperimentConfig {
    pub map2a_protocol: Map2AExperimentConfig,
    pub initial_resource: f64,
    pub supply_rate: f64,
    pub maintenance_cost: f64,
    pub activity_cost: f64,
    pub plasticity_cost: f64,
    pub minimum_modulation: f64,
    pub minimum_no_supply_resource_drop: f64,
    pub minimum_no_supply_probe_score_drop: f64,
    pub maximum_probe_score_degradation: f64,
    pub maximum_weight_drift_increase: f64,
}

impl Default for Map2BExperimentConfig {
    fn default() -> Self {
        Self {
            map2a_protocol: Map2AExperimentConfig::default(),
            initial_resource: 0.75,
            supply_rate: 0.007,
            maintenance_cost: 0.001,
            activity_cost: 0.012,
            plasticity_cost: 0.05,
            minimum_modulation: 0.60,
            minimum_no_supply_resource_drop: 0.20,
            minimum_no_supply_probe_score_drop: 0.10,
            maximum_probe_score_degradation: 0.02,
            maximum_weight_drift_increase: 0.05,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Map2BMechanismComparison {
    pub parameter_id: usize,
    pub continuous_resource_formation_probability: Map0Interval,
    pub no_resource_formation_probability: Map0Interval,
    pub formation_probability_change: f64,
    pub mean_probe_score_change: f64,
    pub repeated_reversal_accuracy_change: f64,
    pub perturbation_recovery_gain_change: f64,
    pub mean_weight_drift_change: f64,
    pub mean_resource_level_change: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Map2BAcceptanceReport {
    pub only_resource_mechanism_added: bool,
    pub development_and_confirmation_seeds_disjoint: bool,
    pub development_runs_complete: bool,
    pub confirmation_runs_complete: bool,
    pub no_resource_controls_complete: bool,
    pub no_supply_controls_complete: bool,
    pub causal_controls_complete: bool,
    pub causal_intervention_detected: bool,
    pub finite_outputs: bool,
    pub resource_dynamically_engaged: bool,
    pub no_supply_lowers_resource: bool,
    pub no_supply_lowers_probe_score: bool,
    pub probe_score_preserved: bool,
    pub weight_stability_preserved: bool,
    pub stable_region_found: bool,
    pub stage_passed: bool,
    pub passed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Map2BExperimentResult {
    pub version: String,
    pub config: Map2BExperimentConfig,
    pub development_parameters: Vec<Map0ParameterPoint>,
    pub development_seed_results: Vec<Map0SeedResult>,
    pub development_summaries: Vec<Map0ParameterSummary>,
    pub confirmation_parameter_ids: Vec<usize>,
    pub confirmation_seed_results: Vec<Map0SeedResult>,
    pub confirmation_summaries: Vec<Map0ParameterSummary>,
    pub no_resource_seed_results: Vec<Map0SeedResult>,
    pub no_resource_summaries: Vec<Map0ParameterSummary>,
    pub no_supply_seed_results: Vec<Map0SeedResult>,
    pub no_supply_summaries: Vec<Map0ParameterSummary>,
    pub mechanism_comparisons: Vec<Map2BMechanismComparison>,
    pub control_seed_results: Vec<Map0SeedResult>,
    pub control_summaries: Vec<Map0ControlSummary>,
    pub stable_region_parameter_ids: Vec<usize>,
    pub acceptance: Map2BAcceptanceReport,
    pub conclusions: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Map2CExperimentConfig {
    pub map2b_protocol: Map2BExperimentConfig,
    pub stream_trial_count: usize,
    pub minimum_change_gap: usize,
    pub change_probability: f64,
    pub settling_window: usize,
    pub exploration: f64,
    pub minimum_rule_changes: usize,
    pub minimum_overall_accuracy: f64,
    pub minimum_settled_accuracy: f64,
    pub maximum_relative_weight_drift: f64,
    pub minimum_resource_level: f64,
    pub minimum_formation_probability: f64,
    pub minimum_region_size: usize,
    pub maximum_reset_advantage: f64,
    pub minimum_no_supply_accuracy_drop: f64,
    pub minimum_frozen_accuracy_drop: f64,
}

impl Default for Map2CExperimentConfig {
    fn default() -> Self {
        Self {
            map2b_protocol: Map2BExperimentConfig::default(),
            stream_trial_count: 720,
            minimum_change_gap: 30,
            change_probability: 0.025,
            settling_window: 20,
            exploration: 0.12,
            minimum_rule_changes: 5,
            minimum_overall_accuracy: 0.60,
            minimum_settled_accuracy: 0.65,
            maximum_relative_weight_drift: 1.0,
            minimum_resource_level: 0.05,
            minimum_formation_probability: 0.50,
            minimum_region_size: 3,
            maximum_reset_advantage: 0.05,
            minimum_no_supply_accuracy_drop: 0.10,
            minimum_frozen_accuracy_drop: 0.10,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Map2CParameterSummary {
    pub parameters: Map0ParameterPoint,
    pub control: Map0Control,
    pub seed_count: usize,
    pub formation_probability: Map0Interval,
    pub mean_rule_change_count: f64,
    pub mean_state_reset_count: f64,
    pub mean_overall_accuracy: f64,
    pub mean_post_change_accuracy: f64,
    pub mean_settled_accuracy: f64,
    pub mean_recovery_gain: f64,
    pub mean_resource_level: f64,
    pub mean_minimum_resource_level: f64,
    pub mean_relative_weight_drift: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Map2CAcceptanceReport {
    pub only_continuous_protocol_changed: bool,
    pub development_and_confirmation_seeds_disjoint: bool,
    pub development_runs_complete: bool,
    pub confirmation_runs_complete: bool,
    pub reset_controls_complete: bool,
    pub no_supply_controls_complete: bool,
    pub frozen_controls_complete: bool,
    pub finite_outputs: bool,
    pub continuous_stream_has_no_state_resets: bool,
    pub reset_control_does_not_explain_performance: bool,
    pub no_supply_reduces_accuracy: bool,
    pub frozen_plasticity_reduces_accuracy: bool,
    pub stable_region_found: bool,
    pub stage_passed: bool,
    pub passed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Map2CExperimentResult {
    pub version: String,
    pub config: Map2CExperimentConfig,
    pub development_parameters: Vec<Map0ParameterPoint>,
    pub development_seed_results: Vec<ContinuousSeedResult>,
    pub development_summaries: Vec<Map2CParameterSummary>,
    pub confirmation_parameter_ids: Vec<usize>,
    pub confirmation_seed_results: Vec<ContinuousSeedResult>,
    pub confirmation_summaries: Vec<Map2CParameterSummary>,
    pub reset_seed_results: Vec<ContinuousSeedResult>,
    pub reset_summaries: Vec<Map2CParameterSummary>,
    pub no_supply_seed_results: Vec<ContinuousSeedResult>,
    pub no_supply_summaries: Vec<Map2CParameterSummary>,
    pub frozen_seed_results: Vec<ContinuousSeedResult>,
    pub frozen_summaries: Vec<Map2CParameterSummary>,
    pub stable_region_parameter_ids: Vec<usize>,
    pub acceptance: Map2CAcceptanceReport,
    pub conclusions: Vec<String>,
}

pub fn run_map2a_experiment(
    config: Map2AExperimentConfig,
) -> Result<Map2AExperimentResult, EmbodiedError> {
    validate_config(config)?;
    let map1 = config.map1_protocol;
    let protocol = protocol_config(map1);
    let homeostasis = HomeostasisMechanism::DualTimescale(DualTimescaleHomeostasis {
        activity_target: map1.activity_target,
        activity_ema_rate: map1.activity_ema_rate,
        excitability_adjustment_rate: map1.excitability_adjustment_rate,
        weight_norm_relaxation_rate: map1.weight_norm_relaxation_rate,
        minimum_excitability_gain: map1.minimum_excitability_gain,
        maximum_excitability_gain: map1.maximum_excitability_gain,
    });
    let soft_bounded = PlasticityMechanism::SoftBounded(SoftBoundedPlasticity {
        bound_scale: config.soft_bound_scale,
    });
    let parameters = parameter_points(map1);
    let development_seeds = seed_partition(map1.seed ^ 0x4445_5601, map1.development_seed_count);
    let confirmation_seeds =
        seed_partition(map1.seed ^ 0x434f_4e46_0101, map1.confirmation_seed_count);

    let mut development_seed_results = Vec::new();
    for point in &parameters {
        for seed in &development_seeds {
            development_seed_results.push(run_map_seed_with_plasticity(
                protocol,
                *point,
                *seed,
                Map0Control::Baseline,
                homeostasis,
                soft_bounded,
            ));
        }
    }
    let development_summaries = summarize(
        &parameters,
        &development_seed_results,
        Map0Control::Baseline,
    );
    let confirmation_parameter_ids = select_candidates(map1, &development_summaries);
    let confirmation_parameters = confirmation_parameter_ids
        .iter()
        .map(|id| parameters[*id])
        .collect::<Vec<_>>();

    let mut confirmation_seed_results = Vec::new();
    let mut additive_seed_results = Vec::new();
    let mut control_seed_results = Vec::new();
    for point in &confirmation_parameters {
        for seed in &confirmation_seeds {
            confirmation_seed_results.push(run_map_seed_with_plasticity(
                protocol,
                *point,
                *seed,
                Map0Control::Baseline,
                homeostasis,
                soft_bounded,
            ));
            additive_seed_results.push(run_map_seed_with_plasticity(
                protocol,
                *point,
                *seed,
                Map0Control::AdditivePlasticity,
                homeostasis,
                PlasticityMechanism::Additive,
            ));
        }
        for control in Map0Control::CAUSAL_CONTROLS {
            for seed in &confirmation_seeds {
                control_seed_results.push(run_map_seed_with_plasticity(
                    protocol,
                    *point,
                    *seed,
                    control,
                    homeostasis,
                    soft_bounded,
                ));
            }
        }
    }

    let confirmation_summaries = summarize(
        &confirmation_parameters,
        &confirmation_seed_results,
        Map0Control::Baseline,
    );
    let additive_summaries = summarize(
        &confirmation_parameters,
        &additive_seed_results,
        Map0Control::AdditivePlasticity,
    );
    let mechanism_comparisons = comparisons(&confirmation_summaries, &additive_summaries);
    let mut control_summaries = Vec::new();
    for point in &confirmation_parameters {
        let baseline = confirmation_summaries
            .iter()
            .find(|summary| summary.parameters.id == point.id)
            .expect("Map 2A confirmation baseline");
        for control in Map0Control::CAUSAL_CONTROLS {
            let summary = summarize(&[*point], &control_seed_results, control)
                .into_iter()
                .next()
                .expect("Map 2A control summary");
            control_summaries.push(Map0ControlSummary {
                control,
                parameter_id: point.id,
                formation_probability: summary.formation_probability,
                mean_probe_score: summary.mean_probe_score,
                formation_probability_change_from_baseline: summary.formation_probability.mean
                    - baseline.formation_probability.mean,
                mean_probe_score_change_from_baseline: summary.mean_probe_score
                    - baseline.mean_probe_score,
            });
        }
    }

    let stable_region_parameter_ids = stable_region(map1, &confirmation_summaries);
    let causal_intervention_detected = control_summaries.iter().any(|summary| {
        summary.formation_probability_change_from_baseline
            <= -protocol.thresholds.minimum_causal_probability_drop
            || summary.mean_probe_score_change_from_baseline
                <= -protocol.thresholds.minimum_causal_probe_score_drop
    });
    let mean_change = |f: fn(&Map2AMechanismComparison) -> f64| {
        mechanism_comparisons.iter().map(f).sum::<f64>() / mechanism_comparisons.len().max(1) as f64
    };
    let mean_weight_drift_reduced = mean_change(|item| item.mean_weight_drift_change)
        <= -config.minimum_mean_weight_drift_reduction;
    let repeated_reversal_or_recovery_preserved =
        mean_change(|item| item.repeated_reversal_accuracy_change)
            >= -config.maximum_accuracy_degradation
            || mean_change(|item| item.perturbation_recovery_gain_change)
                >= -config.maximum_accuracy_degradation;
    let finite_outputs = development_seed_results
        .iter()
        .chain(&confirmation_seed_results)
        .chain(&additive_seed_results)
        .chain(&control_seed_results)
        .all(result_is_finite);
    let seeds_disjoint = development_seeds
        .iter()
        .all(|seed| !confirmation_seeds.contains(seed));
    let stable_region_found = stable_region_parameter_ids.len()
        >= protocol.thresholds.minimum_region_size
        && causal_intervention_detected;
    let stage_passed =
        mean_weight_drift_reduced && repeated_reversal_or_recovery_preserved && finite_outputs;
    let mut acceptance = Map2AAcceptanceReport {
        only_plasticity_mechanism_changed: same_probe_protocol(map1.probe_protocol, protocol),
        development_and_confirmation_seeds_disjoint: seeds_disjoint,
        development_runs_complete: development_seed_results.len()
            == map1.development_config_count * map1.development_seed_count,
        confirmation_runs_complete: confirmation_seed_results.len()
            == confirmation_parameter_ids.len() * map1.confirmation_seed_count,
        additive_controls_complete: additive_seed_results.len()
            == confirmation_parameter_ids.len() * map1.confirmation_seed_count,
        causal_controls_complete: control_seed_results.len()
            == confirmation_parameter_ids.len()
                * Map0Control::CAUSAL_CONTROLS.len()
                * map1.confirmation_seed_count,
        causal_intervention_detected,
        finite_outputs,
        mean_weight_drift_reduced,
        repeated_reversal_or_recovery_preserved,
        stable_region_found,
        stage_passed,
        passed: false,
    };
    acceptance.passed = acceptance.only_plasticity_mechanism_changed
        && acceptance.development_and_confirmation_seeds_disjoint
        && acceptance.development_runs_complete
        && acceptance.confirmation_runs_complete
        && acceptance.additive_controls_complete
        && acceptance.causal_controls_complete
        && acceptance.finite_outputs;
    let conclusions = conclusions(
        &development_summaries,
        &confirmation_summaries,
        &mechanism_comparisons,
        &stable_region_parameter_ids,
        acceptance,
    );
    Ok(Map2AExperimentResult {
        version: "learnability-map/v0.3-soft-bounded-plasticity".to_owned(),
        config,
        development_parameters: parameters,
        development_seed_results,
        development_summaries,
        confirmation_parameter_ids,
        confirmation_seed_results,
        confirmation_summaries,
        additive_seed_results,
        additive_summaries,
        mechanism_comparisons,
        control_seed_results,
        control_summaries,
        stable_region_parameter_ids,
        acceptance,
        conclusions,
    })
}

pub fn run_map2b_experiment(
    config: Map2BExperimentConfig,
) -> Result<Map2BExperimentResult, EmbodiedError> {
    validate_map2b_config(config)?;
    let map2a = config.map2a_protocol;
    let map1 = map2a.map1_protocol;
    let protocol = protocol_config(map1);
    let homeostasis = dual_homeostasis(map1);
    let plasticity = PlasticityMechanism::SoftBounded(SoftBoundedPlasticity {
        bound_scale: map2a.soft_bound_scale,
    });
    let resource = resource_mechanism(config, config.supply_rate);
    let no_supply = resource_mechanism(config, 0.0);
    let parameters = parameter_points(map1);
    let development_seeds = seed_partition(map1.seed ^ 0x4445_5601, map1.development_seed_count);
    let confirmation_seeds =
        seed_partition(map1.seed ^ 0x434f_4e46_0101, map1.confirmation_seed_count);

    let mut development_seed_results = Vec::new();
    for point in &parameters {
        for seed in &development_seeds {
            development_seed_results.push(run_map_seed_with_mechanisms(
                protocol,
                *point,
                *seed,
                Map0Control::Baseline,
                homeostasis,
                plasticity,
                resource,
            ));
        }
    }
    let development_summaries = summarize(
        &parameters,
        &development_seed_results,
        Map0Control::Baseline,
    );
    let confirmation_parameter_ids = select_candidates(map1, &development_summaries);
    let confirmation_parameters = confirmation_parameter_ids
        .iter()
        .map(|id| parameters[*id])
        .collect::<Vec<_>>();

    let mut confirmation_seed_results = Vec::new();
    let mut no_resource_seed_results = Vec::new();
    let mut no_supply_seed_results = Vec::new();
    let mut control_seed_results = Vec::new();
    for point in &confirmation_parameters {
        for seed in &confirmation_seeds {
            confirmation_seed_results.push(run_map_seed_with_mechanisms(
                protocol,
                *point,
                *seed,
                Map0Control::Baseline,
                homeostasis,
                plasticity,
                resource,
            ));
            no_resource_seed_results.push(run_map_seed_with_mechanisms(
                protocol,
                *point,
                *seed,
                Map0Control::NoResourceAccounting,
                homeostasis,
                plasticity,
                ResourceMechanism::Disabled,
            ));
            no_supply_seed_results.push(run_map_seed_with_mechanisms(
                protocol,
                *point,
                *seed,
                Map0Control::NoResourceSupply,
                homeostasis,
                plasticity,
                no_supply,
            ));
        }
        for control in Map0Control::CAUSAL_CONTROLS {
            for seed in &confirmation_seeds {
                control_seed_results.push(run_map_seed_with_mechanisms(
                    protocol,
                    *point,
                    *seed,
                    control,
                    homeostasis,
                    plasticity,
                    resource,
                ));
            }
        }
    }
    let confirmation_summaries = summarize(
        &confirmation_parameters,
        &confirmation_seed_results,
        Map0Control::Baseline,
    );
    let no_resource_summaries = summarize(
        &confirmation_parameters,
        &no_resource_seed_results,
        Map0Control::NoResourceAccounting,
    );
    let no_supply_summaries = summarize(
        &confirmation_parameters,
        &no_supply_seed_results,
        Map0Control::NoResourceSupply,
    );
    let mechanism_comparisons =
        resource_comparisons(&confirmation_summaries, &no_resource_summaries);
    let mut control_summaries = Vec::new();
    for point in &confirmation_parameters {
        let baseline = confirmation_summaries
            .iter()
            .find(|summary| summary.parameters.id == point.id)
            .expect("Map 2B confirmation baseline");
        for control in Map0Control::CAUSAL_CONTROLS {
            let summary = summarize(&[*point], &control_seed_results, control)
                .into_iter()
                .next()
                .expect("Map 2B control summary");
            control_summaries.push(Map0ControlSummary {
                control,
                parameter_id: point.id,
                formation_probability: summary.formation_probability,
                mean_probe_score: summary.mean_probe_score,
                formation_probability_change_from_baseline: summary.formation_probability.mean
                    - baseline.formation_probability.mean,
                mean_probe_score_change_from_baseline: summary.mean_probe_score
                    - baseline.mean_probe_score,
            });
        }
    }

    let stable_region_parameter_ids = stable_region(map1, &confirmation_summaries);
    let causal_intervention_detected = control_summaries.iter().any(|summary| {
        summary.formation_probability_change_from_baseline
            <= -protocol.thresholds.minimum_causal_probability_drop
            || summary.mean_probe_score_change_from_baseline
                <= -protocol.thresholds.minimum_causal_probe_score_drop
    });
    let mean_comparison = |f: fn(&Map2BMechanismComparison) -> f64| {
        mechanism_comparisons.iter().map(f).sum::<f64>() / mechanism_comparisons.len().max(1) as f64
    };
    let mean_summary = |rows: &[Map0ParameterSummary], f: fn(&Map0ParameterSummary) -> f64| {
        rows.iter().map(f).sum::<f64>() / rows.len().max(1) as f64
    };
    let resource_mean = mean_summary(&confirmation_summaries, |row| row.mean_resource_level);
    let no_supply_mean = mean_summary(&no_supply_summaries, |row| row.mean_resource_level);
    let supplied_probe_score = mean_summary(&confirmation_summaries, |row| row.mean_probe_score);
    let no_supply_probe_score = mean_summary(&no_supply_summaries, |row| row.mean_probe_score);
    let resource_dynamically_engaged = (0.05..0.98).contains(&resource_mean);
    let no_supply_lowers_resource =
        resource_mean - no_supply_mean >= config.minimum_no_supply_resource_drop;
    let no_supply_lowers_probe_score =
        supplied_probe_score - no_supply_probe_score >= config.minimum_no_supply_probe_score_drop;
    let probe_score_preserved = mean_comparison(|item| item.mean_probe_score_change)
        >= -config.maximum_probe_score_degradation;
    let weight_stability_preserved = mean_comparison(|item| item.mean_weight_drift_change)
        <= config.maximum_weight_drift_increase;
    let finite_outputs = development_seed_results
        .iter()
        .chain(&confirmation_seed_results)
        .chain(&no_resource_seed_results)
        .chain(&no_supply_seed_results)
        .chain(&control_seed_results)
        .all(result_is_finite);
    let seeds_disjoint = development_seeds
        .iter()
        .all(|seed| !confirmation_seeds.contains(seed));
    let stable_region_found = stable_region_parameter_ids.len()
        >= protocol.thresholds.minimum_region_size
        && causal_intervention_detected;
    let stage_passed = resource_dynamically_engaged
        && no_supply_lowers_resource
        && no_supply_lowers_probe_score
        && probe_score_preserved
        && weight_stability_preserved
        && finite_outputs;
    let mut acceptance = Map2BAcceptanceReport {
        only_resource_mechanism_added: same_probe_protocol(map1.probe_protocol, protocol)
            && map2a.soft_bound_scale == 1.0,
        development_and_confirmation_seeds_disjoint: seeds_disjoint,
        development_runs_complete: development_seed_results.len()
            == map1.development_config_count * map1.development_seed_count,
        confirmation_runs_complete: confirmation_seed_results.len()
            == confirmation_parameter_ids.len() * map1.confirmation_seed_count,
        no_resource_controls_complete: no_resource_seed_results.len()
            == confirmation_parameter_ids.len() * map1.confirmation_seed_count,
        no_supply_controls_complete: no_supply_seed_results.len()
            == confirmation_parameter_ids.len() * map1.confirmation_seed_count,
        causal_controls_complete: control_seed_results.len()
            == confirmation_parameter_ids.len()
                * Map0Control::CAUSAL_CONTROLS.len()
                * map1.confirmation_seed_count,
        causal_intervention_detected,
        finite_outputs,
        resource_dynamically_engaged,
        no_supply_lowers_resource,
        no_supply_lowers_probe_score,
        probe_score_preserved,
        weight_stability_preserved,
        stable_region_found,
        stage_passed,
        passed: false,
    };
    acceptance.passed = acceptance.only_resource_mechanism_added
        && acceptance.development_and_confirmation_seeds_disjoint
        && acceptance.development_runs_complete
        && acceptance.confirmation_runs_complete
        && acceptance.no_resource_controls_complete
        && acceptance.no_supply_controls_complete
        && acceptance.causal_controls_complete
        && acceptance.finite_outputs;
    let conclusions = resource_conclusions(
        &development_summaries,
        &confirmation_summaries,
        &mechanism_comparisons,
        resource_mean,
        no_supply_mean,
        &stable_region_parameter_ids,
        acceptance,
    );
    Ok(Map2BExperimentResult {
        version: "learnability-map/v0.4-continuous-resource".to_owned(),
        config,
        development_parameters: parameters,
        development_seed_results,
        development_summaries,
        confirmation_parameter_ids,
        confirmation_seed_results,
        confirmation_summaries,
        no_resource_seed_results,
        no_resource_summaries,
        no_supply_seed_results,
        no_supply_summaries,
        mechanism_comparisons,
        control_seed_results,
        control_summaries,
        stable_region_parameter_ids,
        acceptance,
        conclusions,
    })
}

pub fn run_map2c_experiment(
    config: Map2CExperimentConfig,
) -> Result<Map2CExperimentResult, EmbodiedError> {
    validate_map2c_config(config)?;
    let map2b = config.map2b_protocol;
    let map2a = map2b.map2a_protocol;
    let map1 = map2a.map1_protocol;
    let protocol = protocol_config(map1);
    let homeostasis = dual_homeostasis(map1);
    let plasticity = PlasticityMechanism::SoftBounded(SoftBoundedPlasticity {
        bound_scale: map2a.soft_bound_scale,
    });
    let resource = resource_mechanism(map2b, map2b.supply_rate);
    let no_supply = resource_mechanism(map2b, 0.0);
    let stream = continuous_stream(config, false);
    let reset_stream = continuous_stream(config, true);
    let parameters = parameter_points(map1);
    let development_seeds = seed_partition(map1.seed ^ 0x4445_5601, map1.development_seed_count);
    let confirmation_seeds =
        seed_partition(map1.seed ^ 0x434f_4e46_0101, map1.confirmation_seed_count);

    let mut development_seed_results = Vec::new();
    for point in &parameters {
        for seed in &development_seeds {
            development_seed_results.push(run_continuous_seed_with_mechanisms(
                protocol,
                *point,
                *seed,
                Map0Control::Baseline,
                homeostasis,
                plasticity,
                resource,
                stream,
            ));
        }
    }
    let development_summaries = continuous_summaries(
        config,
        &parameters,
        &development_seed_results,
        Map0Control::Baseline,
    );
    let confirmation_parameter_ids = select_continuous_candidates(config, &development_summaries);
    let confirmation_parameters = confirmation_parameter_ids
        .iter()
        .map(|id| parameters[*id])
        .collect::<Vec<_>>();

    let mut confirmation_seed_results = Vec::new();
    let mut reset_seed_results = Vec::new();
    let mut no_supply_seed_results = Vec::new();
    let mut frozen_seed_results = Vec::new();
    for point in &confirmation_parameters {
        for seed in &confirmation_seeds {
            confirmation_seed_results.push(run_continuous_seed_with_mechanisms(
                protocol,
                *point,
                *seed,
                Map0Control::Baseline,
                homeostasis,
                plasticity,
                resource,
                stream,
            ));
            reset_seed_results.push(run_continuous_seed_with_mechanisms(
                protocol,
                *point,
                *seed,
                Map0Control::ResetBetweenTrials,
                homeostasis,
                plasticity,
                resource,
                reset_stream,
            ));
            no_supply_seed_results.push(run_continuous_seed_with_mechanisms(
                protocol,
                *point,
                *seed,
                Map0Control::NoResourceSupply,
                homeostasis,
                plasticity,
                no_supply,
                stream,
            ));
            frozen_seed_results.push(run_continuous_seed_with_mechanisms(
                protocol,
                *point,
                *seed,
                Map0Control::FrozenPlasticity,
                homeostasis,
                plasticity,
                resource,
                stream,
            ));
        }
    }
    let confirmation_summaries = continuous_summaries(
        config,
        &confirmation_parameters,
        &confirmation_seed_results,
        Map0Control::Baseline,
    );
    let reset_summaries = continuous_summaries(
        config,
        &confirmation_parameters,
        &reset_seed_results,
        Map0Control::ResetBetweenTrials,
    );
    let no_supply_summaries = continuous_summaries(
        config,
        &confirmation_parameters,
        &no_supply_seed_results,
        Map0Control::NoResourceSupply,
    );
    let frozen_summaries = continuous_summaries(
        config,
        &confirmation_parameters,
        &frozen_seed_results,
        Map0Control::FrozenPlasticity,
    );
    let stable_region_parameter_ids = continuous_stable_region(config, &confirmation_summaries);
    let mean_accuracy = |rows: &[Map2CParameterSummary]| {
        rows.iter()
            .map(|row| row.mean_overall_accuracy)
            .sum::<f64>()
            / rows.len().max(1) as f64
    };
    let continuous_accuracy = mean_accuracy(&confirmation_summaries);
    let reset_accuracy = mean_accuracy(&reset_summaries);
    let no_supply_accuracy = mean_accuracy(&no_supply_summaries);
    let frozen_accuracy = mean_accuracy(&frozen_summaries);
    let finite_outputs = development_seed_results
        .iter()
        .chain(&confirmation_seed_results)
        .chain(&reset_seed_results)
        .chain(&no_supply_seed_results)
        .chain(&frozen_seed_results)
        .all(|row| row.finite);
    let no_resets = development_seed_results
        .iter()
        .chain(&confirmation_seed_results)
        .chain(&no_supply_seed_results)
        .chain(&frozen_seed_results)
        .all(|row| row.state_reset_count == 0);
    let seeds_disjoint = development_seeds
        .iter()
        .all(|seed| !confirmation_seeds.contains(seed));
    let reset_control_does_not_explain_performance =
        reset_accuracy - continuous_accuracy <= config.maximum_reset_advantage;
    let no_supply_reduces_accuracy =
        continuous_accuracy - no_supply_accuracy >= config.minimum_no_supply_accuracy_drop;
    let frozen_plasticity_reduces_accuracy =
        continuous_accuracy - frozen_accuracy >= config.minimum_frozen_accuracy_drop;
    let stable_region_found = stable_region_parameter_ids.len() >= config.minimum_region_size;
    let stage_passed = stable_region_found
        && reset_control_does_not_explain_performance
        && no_supply_reduces_accuracy
        && frozen_plasticity_reduces_accuracy
        && finite_outputs
        && no_resets;
    let mut acceptance = Map2CAcceptanceReport {
        only_continuous_protocol_changed: same_probe_protocol(map1.probe_protocol, protocol)
            && map2a.soft_bound_scale == 1.0
            && same_resource_protocol(map2b, Map2BExperimentConfig::default()),
        development_and_confirmation_seeds_disjoint: seeds_disjoint,
        development_runs_complete: development_seed_results.len()
            == map1.development_config_count * map1.development_seed_count,
        confirmation_runs_complete: confirmation_seed_results.len()
            == confirmation_parameter_ids.len() * map1.confirmation_seed_count,
        reset_controls_complete: reset_seed_results.len()
            == confirmation_parameter_ids.len() * map1.confirmation_seed_count,
        no_supply_controls_complete: no_supply_seed_results.len()
            == confirmation_parameter_ids.len() * map1.confirmation_seed_count,
        frozen_controls_complete: frozen_seed_results.len()
            == confirmation_parameter_ids.len() * map1.confirmation_seed_count,
        finite_outputs,
        continuous_stream_has_no_state_resets: no_resets,
        reset_control_does_not_explain_performance,
        no_supply_reduces_accuracy,
        frozen_plasticity_reduces_accuracy,
        stable_region_found,
        stage_passed,
        passed: false,
    };
    acceptance.passed = acceptance.only_continuous_protocol_changed
        && acceptance.development_and_confirmation_seeds_disjoint
        && acceptance.development_runs_complete
        && acceptance.confirmation_runs_complete
        && acceptance.reset_controls_complete
        && acceptance.no_supply_controls_complete
        && acceptance.frozen_controls_complete
        && acceptance.finite_outputs
        && acceptance.continuous_stream_has_no_state_resets;
    let conclusions = continuous_conclusions(
        &development_summaries,
        &confirmation_summaries,
        continuous_accuracy,
        reset_accuracy,
        no_supply_accuracy,
        frozen_accuracy,
        &stable_region_parameter_ids,
        acceptance,
    );
    Ok(Map2CExperimentResult {
        version: "learnability-map/v0.5-continuous-stream".to_owned(),
        config,
        development_parameters: parameters,
        development_seed_results,
        development_summaries,
        confirmation_parameter_ids,
        confirmation_seed_results,
        confirmation_summaries,
        reset_seed_results,
        reset_summaries,
        no_supply_seed_results,
        no_supply_summaries,
        frozen_seed_results,
        frozen_summaries,
        stable_region_parameter_ids,
        acceptance,
        conclusions,
    })
}

fn continuous_stream(
    config: Map2CExperimentConfig,
    reset_between_trials: bool,
) -> ContinuousStreamConfig {
    ContinuousStreamConfig {
        trial_count: config.stream_trial_count,
        minimum_change_gap: config.minimum_change_gap,
        change_probability: config.change_probability,
        settling_window: config.settling_window,
        exploration: config.exploration,
        reset_between_trials,
    }
}

fn continuous_pass(config: Map2CExperimentConfig, row: &ContinuousSeedResult) -> bool {
    row.rule_change_count >= config.minimum_rule_changes
        && row.state_reset_count == 0
        && row.overall_accuracy >= config.minimum_overall_accuracy
        && row.settled_accuracy >= config.minimum_settled_accuracy
        && row.recovery_gain >= 0.0
        && row.mean_relative_weight_drift <= config.maximum_relative_weight_drift
        && row.mean_resource_level >= config.minimum_resource_level
        && row.finite
}

fn continuous_summaries(
    config: Map2CExperimentConfig,
    parameters: &[Map0ParameterPoint],
    results: &[ContinuousSeedResult],
    control: Map0Control,
) -> Vec<Map2CParameterSummary> {
    parameters
        .iter()
        .map(|parameters| {
            let rows = results
                .iter()
                .filter(|row| row.parameter_id == parameters.id && row.control == control)
                .collect::<Vec<_>>();
            let count = rows.len().max(1) as f64;
            let mean = |f: fn(&ContinuousSeedResult) -> f64| {
                rows.iter().map(|row| f(row)).sum::<f64>() / count
            };
            let successes = rows
                .iter()
                .filter(|row| continuous_pass(config, row))
                .count();
            Map2CParameterSummary {
                parameters: *parameters,
                control,
                seed_count: rows.len(),
                formation_probability: wilson_interval(successes, rows.len()),
                mean_rule_change_count: mean(|row| row.rule_change_count as f64),
                mean_state_reset_count: mean(|row| row.state_reset_count as f64),
                mean_overall_accuracy: mean(|row| row.overall_accuracy),
                mean_post_change_accuracy: mean(|row| row.post_change_accuracy),
                mean_settled_accuracy: mean(|row| row.settled_accuracy),
                mean_recovery_gain: mean(|row| row.recovery_gain),
                mean_resource_level: mean(|row| row.mean_resource_level),
                mean_minimum_resource_level: mean(|row| row.minimum_resource_level),
                mean_relative_weight_drift: mean(|row| row.mean_relative_weight_drift),
            }
        })
        .collect()
}

fn continuous_rank(summary: &Map2CParameterSummary) -> f64 {
    summary.formation_probability.mean * 2.0
        + summary.mean_overall_accuracy
        + summary.mean_settled_accuracy
        + summary.mean_recovery_gain.max(0.0)
}

fn select_continuous_candidates(
    config: Map2CExperimentConfig,
    summaries: &[Map2CParameterSummary],
) -> Vec<usize> {
    let mut ranked = summaries.iter().collect::<Vec<_>>();
    ranked.sort_by(|left, right| continuous_rank(right).total_cmp(&continuous_rank(left)));
    let Some(best) = ranked.first().copied() else {
        return Vec::new();
    };
    let protocol = protocol_config(config.map2b_protocol.map2a_protocol.map1_protocol);
    let mut selected = vec![best.parameters.id];
    let mut neighbors = summaries.iter().collect::<Vec<_>>();
    neighbors.sort_by(|left, right| {
        normalized_distance(protocol, best.parameters, left.parameters).total_cmp(
            &normalized_distance(protocol, best.parameters, right.parameters),
        )
    });
    for neighbor in neighbors
        .into_iter()
        .filter(|summary| summary.parameters.id != best.parameters.id)
        .take(config.minimum_region_size.saturating_sub(1))
    {
        selected.push(neighbor.parameters.id);
    }
    for summary in ranked {
        if selected.len()
            == config
                .map2b_protocol
                .map2a_protocol
                .map1_protocol
                .confirmation_candidate_count
        {
            break;
        }
        if !selected.contains(&summary.parameters.id) {
            selected.push(summary.parameters.id);
        }
    }
    selected
}

fn continuous_stable_region(
    config: Map2CExperimentConfig,
    summaries: &[Map2CParameterSummary],
) -> Vec<usize> {
    let protocol = protocol_config(config.map2b_protocol.map2a_protocol.map1_protocol);
    let eligible = summaries
        .iter()
        .filter(|summary| {
            summary.formation_probability.mean >= config.minimum_formation_probability
        })
        .collect::<Vec<_>>();
    let mut best = Vec::new();
    for start in &eligible {
        let mut component = vec![start.parameters.id];
        let mut cursor = 0;
        while cursor < component.len() {
            let current = summaries
                .iter()
                .find(|summary| summary.parameters.id == component[cursor])
                .expect("Map 2C component summary");
            for candidate in &eligible {
                if !component.contains(&candidate.parameters.id)
                    && normalized_distance(protocol, current.parameters, candidate.parameters)
                        <= 0.55
                {
                    component.push(candidate.parameters.id);
                }
            }
            cursor += 1;
        }
        if component.len() > best.len() {
            best = component;
        }
    }
    best.sort_unstable();
    best
}

fn continuous_conclusions(
    development: &[Map2CParameterSummary],
    confirmation: &[Map2CParameterSummary],
    continuous_accuracy: f64,
    reset_accuracy: f64,
    no_supply_accuracy: f64,
    frozen_accuracy: f64,
    stable_region: &[usize],
    acceptance: Map2CAcceptanceReport,
) -> Vec<String> {
    let mut output = Vec::new();
    if let Some(best) = development
        .iter()
        .max_by(|left, right| continuous_rank(left).total_cmp(&continuous_rank(right)))
    {
        output.push(format!(
            "Map 2C 开发扫描最佳配置 {} 的连续形成概率为 {:.1}%，总体准确率 {:.1}%，稳定期准确率 {:.1}%。",
            best.parameters.id,
            best.formation_probability.mean * 100.0,
            best.mean_overall_accuracy * 100.0,
            best.mean_settled_accuracy * 100.0,
        ));
    }
    if let Some(best) = confirmation
        .iter()
        .max_by(|left, right| continuous_rank(left).total_cmp(&continuous_rank(right)))
    {
        output.push(format!(
            "独立确认最佳配置 {} 的连续形成概率为 {:.1}%（Wilson 95% 区间 {:.1}%–{:.1}%）。",
            best.parameters.id,
            best.formation_probability.mean * 100.0,
            best.formation_probability.lower95 * 100.0,
            best.formation_probability.upper95 * 100.0,
        ));
    }
    output.push(format!(
        "连续基线总体准确率 {:.1}%；逐试次重置 {:.1}%，关闭供能 {:.1}%，冻结可塑性 {:.1}%。",
        continuous_accuracy * 100.0,
        reset_accuracy * 100.0,
        no_supply_accuracy * 100.0,
        frozen_accuracy * 100.0,
    ));
    output.push(if acceptance.stage_passed {
        "Map 2C 达到预注册连续运行标准，可以冻结 Map 2 并进入 Scale 0。".to_owned()
    } else {
        "Map 2C 未达到预注册连续运行标准；这是有效的负结果，停止在 Map 2，不进入 Scale 0。"
            .to_owned()
    });
    if stable_region.is_empty() {
        output.push("无重置连续流中没有找到跨相邻参数可复现的稳定区域。".to_owned());
    } else {
        output.push(format!(
            "无重置连续流中找到由 {} 个确认配置组成的稳定区域。",
            stable_region.len()
        ));
    }
    output
}

fn comparisons(
    bounded: &[Map0ParameterSummary],
    additive: &[Map0ParameterSummary],
) -> Vec<Map2AMechanismComparison> {
    bounded
        .iter()
        .map(|current| {
            let old = additive
                .iter()
                .find(|summary| summary.parameters.id == current.parameters.id)
                .expect("Map 2A additive summary");
            Map2AMechanismComparison {
                parameter_id: current.parameters.id,
                soft_bounded_formation_probability: current.formation_probability,
                additive_formation_probability: old.formation_probability,
                formation_probability_change: current.formation_probability.mean
                    - old.formation_probability.mean,
                mean_probe_score_change: current.mean_probe_score - old.mean_probe_score,
                repeated_reversal_accuracy_change: current.mean_repeated_reversal_accuracy
                    - old.mean_repeated_reversal_accuracy,
                perturbation_recovery_gain_change: current.mean_perturbation_recovery_gain
                    - old.mean_perturbation_recovery_gain,
                mean_weight_drift_change: current.mean_relative_weight_drift
                    - old.mean_relative_weight_drift,
            }
        })
        .collect()
}

fn resource_comparisons(
    resource: &[Map0ParameterSummary],
    disabled: &[Map0ParameterSummary],
) -> Vec<Map2BMechanismComparison> {
    resource
        .iter()
        .map(|current| {
            let old = disabled
                .iter()
                .find(|summary| summary.parameters.id == current.parameters.id)
                .expect("Map 2B no-resource summary");
            Map2BMechanismComparison {
                parameter_id: current.parameters.id,
                continuous_resource_formation_probability: current.formation_probability,
                no_resource_formation_probability: old.formation_probability,
                formation_probability_change: current.formation_probability.mean
                    - old.formation_probability.mean,
                mean_probe_score_change: current.mean_probe_score - old.mean_probe_score,
                repeated_reversal_accuracy_change: current.mean_repeated_reversal_accuracy
                    - old.mean_repeated_reversal_accuracy,
                perturbation_recovery_gain_change: current.mean_perturbation_recovery_gain
                    - old.mean_perturbation_recovery_gain,
                mean_weight_drift_change: current.mean_relative_weight_drift
                    - old.mean_relative_weight_drift,
                mean_resource_level_change: current.mean_resource_level - old.mean_resource_level,
            }
        })
        .collect()
}

fn dual_homeostasis(config: Map1ExperimentConfig) -> HomeostasisMechanism {
    HomeostasisMechanism::DualTimescale(DualTimescaleHomeostasis {
        activity_target: config.activity_target,
        activity_ema_rate: config.activity_ema_rate,
        excitability_adjustment_rate: config.excitability_adjustment_rate,
        weight_norm_relaxation_rate: config.weight_norm_relaxation_rate,
        minimum_excitability_gain: config.minimum_excitability_gain,
        maximum_excitability_gain: config.maximum_excitability_gain,
    })
}

fn resource_mechanism(config: Map2BExperimentConfig, supply_rate: f64) -> ResourceMechanism {
    ResourceMechanism::Continuous(ContinuousResource {
        initial_level: config.initial_resource,
        supply_rate,
        maintenance_cost: config.maintenance_cost,
        activity_cost: config.activity_cost,
        plasticity_cost: config.plasticity_cost,
        minimum_modulation: config.minimum_modulation,
    })
}

fn same_resource_protocol(left: Map2BExperimentConfig, right: Map2BExperimentConfig) -> bool {
    left.initial_resource == right.initial_resource
        && left.supply_rate == right.supply_rate
        && left.maintenance_cost == right.maintenance_cost
        && left.activity_cost == right.activity_cost
        && left.plasticity_cost == right.plasticity_cost
        && left.minimum_modulation == right.minimum_modulation
        && left.minimum_no_supply_resource_drop == right.minimum_no_supply_resource_drop
        && left.minimum_no_supply_probe_score_drop == right.minimum_no_supply_probe_score_drop
        && left.maximum_probe_score_degradation == right.maximum_probe_score_degradation
        && left.maximum_weight_drift_increase == right.maximum_weight_drift_increase
}

fn resource_conclusions(
    development: &[Map0ParameterSummary],
    confirmation: &[Map0ParameterSummary],
    comparisons: &[Map2BMechanismComparison],
    resource_mean: f64,
    no_supply_mean: f64,
    stable_region: &[usize],
    acceptance: Map2BAcceptanceReport,
) -> Vec<String> {
    let mut output = Vec::new();
    if let Some(best) = development
        .iter()
        .max_by(|left, right| rank(left).total_cmp(&rank(right)))
    {
        output.push(format!(
            "Map 2B 开发扫描最佳配置 {} 的形成概率为 {:.1}%，平均探针得分为 {:.1}%。",
            best.parameters.id,
            best.formation_probability.mean * 100.0,
            best.mean_probe_score * 100.0,
        ));
    }
    if let Some(best) = confirmation
        .iter()
        .max_by(|left, right| rank(left).total_cmp(&rank(right)))
    {
        output.push(format!(
            "独立确认最佳配置 {} 的形成概率为 {:.1}%（Wilson 95% 区间 {:.1}%–{:.1}%）。",
            best.parameters.id,
            best.formation_probability.mean * 100.0,
            best.formation_probability.lower95 * 100.0,
            best.formation_probability.upper95 * 100.0,
        ));
    }
    let mean = |f: fn(&Map2BMechanismComparison) -> f64| {
        comparisons.iter().map(f).sum::<f64>() / comparisons.len().max(1) as f64
    };
    output.push(format!(
        "持续供能相对关闭资源核算的平均探针变化为 {:+.1} 个百分点，权重漂移变化 {:+.3}；平均资源 {:.3}，关闭供能后 {:.3}。",
        mean(|item| item.mean_probe_score_change) * 100.0,
        mean(|item| item.mean_weight_drift_change),
        resource_mean,
        no_supply_mean,
    ));
    output.push(if acceptance.stage_passed {
        "Map 2B 达到预注册资源闭环标准，可以冻结机制并进入 Map 2C。".to_owned()
    } else {
        "Map 2B 未达到预注册资源闭环标准，按基座审计规则停止，不进入 Map 2C。".to_owned()
    });
    if stable_region.is_empty() {
        output.push("当前仍没有满足 Map 0 形成条件的连续稳定区域。".to_owned());
    } else {
        output.push(format!(
            "当前找到由 {} 个确认配置组成的连续稳定区域。",
            stable_region.len()
        ));
    }
    output
}

fn conclusions(
    development: &[Map0ParameterSummary],
    confirmation: &[Map0ParameterSummary],
    comparisons: &[Map2AMechanismComparison],
    stable_region: &[usize],
    acceptance: Map2AAcceptanceReport,
) -> Vec<String> {
    let mut output = Vec::new();
    if let Some(best) = development
        .iter()
        .max_by(|left, right| rank(left).total_cmp(&rank(right)))
    {
        output.push(format!(
            "Map 2A 开发扫描最佳配置 {} 的形成概率为 {:.1}%，平均探针得分为 {:.1}%。",
            best.parameters.id,
            best.formation_probability.mean * 100.0,
            best.mean_probe_score * 100.0,
        ));
    }
    if let Some(best) = confirmation
        .iter()
        .max_by(|left, right| rank(left).total_cmp(&rank(right)))
    {
        output.push(format!(
            "独立确认最佳配置 {} 的形成概率为 {:.1}%（Wilson 95% 区间 {:.1}%–{:.1}%）。",
            best.parameters.id,
            best.formation_probability.mean * 100.0,
            best.formation_probability.lower95 * 100.0,
            best.formation_probability.upper95 * 100.0,
        ));
    }
    let mean = |f: fn(&Map2AMechanismComparison) -> f64| {
        comparisons.iter().map(f).sum::<f64>() / comparisons.len().max(1) as f64
    };
    output.push(format!(
        "相同配置和确认种子下，软边界相对加性更新的平均探针变化为 {:+.1} 个百分点，重复反转变化 {:+.1} 个百分点，恢复增益变化 {:+.1} 个百分点，权重漂移变化 {:+.3}。",
        mean(|item| item.mean_probe_score_change) * 100.0,
        mean(|item| item.repeated_reversal_accuracy_change) * 100.0,
        mean(|item| item.perturbation_recovery_gain_change) * 100.0,
        mean(|item| item.mean_weight_drift_change),
    ));
    output.push(if acceptance.stage_passed {
        "Map 2A 达到预注册的学习—漂移权衡标准，可以冻结机制并进入 Map 2B。".to_owned()
    } else {
        "Map 2A 未达到预注册权衡标准，按基座审计规则停止，不进入 Map 2B。".to_owned()
    });
    if stable_region.is_empty() {
        output.push("当前仍没有满足 Map 0 形成条件的连续稳定区域。".to_owned());
    } else {
        output.push(format!(
            "当前找到由 {} 个确认配置组成的连续稳定区域。",
            stable_region.len()
        ));
    }
    output
}

fn validate_config(config: Map2AExperimentConfig) -> Result<(), EmbodiedError> {
    validate_map1_config(config.map1_protocol)?;
    if !config.soft_bound_scale.is_finite()
        || config.soft_bound_scale <= 0.0
        || !config.minimum_mean_weight_drift_reduction.is_finite()
        || config.minimum_mean_weight_drift_reduction <= 0.0
        || !config.maximum_accuracy_degradation.is_finite()
        || !(0.0..0.1).contains(&config.maximum_accuracy_degradation)
    {
        return Err(EmbodiedError::InvalidExperiment);
    }
    Ok(())
}

fn validate_map2b_config(config: Map2BExperimentConfig) -> Result<(), EmbodiedError> {
    validate_config(config.map2a_protocol)?;
    let finite = [
        config.initial_resource,
        config.supply_rate,
        config.maintenance_cost,
        config.activity_cost,
        config.plasticity_cost,
        config.minimum_modulation,
        config.minimum_no_supply_resource_drop,
        config.minimum_no_supply_probe_score_drop,
        config.maximum_probe_score_degradation,
        config.maximum_weight_drift_increase,
    ]
    .into_iter()
    .all(f64::is_finite);
    if !finite
        || !(0.0..=1.0).contains(&config.initial_resource)
        || config.supply_rate <= 0.0
        || config.maintenance_cost < 0.0
        || config.activity_cost <= 0.0
        || config.plasticity_cost < 0.0
        || !(0.0..1.0).contains(&config.minimum_modulation)
        || !(0.0..1.0).contains(&config.minimum_no_supply_resource_drop)
        || !(0.0..0.5).contains(&config.minimum_no_supply_probe_score_drop)
        || !(0.0..0.1).contains(&config.maximum_probe_score_degradation)
        || !(0.0..0.5).contains(&config.maximum_weight_drift_increase)
    {
        return Err(EmbodiedError::InvalidExperiment);
    }
    Ok(())
}

fn validate_map2c_config(config: Map2CExperimentConfig) -> Result<(), EmbodiedError> {
    validate_map2b_config(config.map2b_protocol)?;
    let finite = [
        config.change_probability,
        config.exploration,
        config.minimum_overall_accuracy,
        config.minimum_settled_accuracy,
        config.maximum_relative_weight_drift,
        config.minimum_resource_level,
        config.minimum_formation_probability,
        config.maximum_reset_advantage,
        config.minimum_no_supply_accuracy_drop,
        config.minimum_frozen_accuracy_drop,
    ]
    .into_iter()
    .all(f64::is_finite);
    if !finite
        || config.stream_trial_count < 2 * config.minimum_change_gap
        || config.minimum_change_gap == 0
        || !(0.0..1.0).contains(&config.change_probability)
        || config.settling_window == 0
        || config.settling_window >= config.minimum_change_gap
        || !(0.0..0.5).contains(&config.exploration)
        || config.minimum_rule_changes == 0
        || !(0.5..1.0).contains(&config.minimum_overall_accuracy)
        || !(0.5..1.0).contains(&config.minimum_settled_accuracy)
        || config.maximum_relative_weight_drift <= 0.0
        || !(0.0..1.0).contains(&config.minimum_resource_level)
        || !(0.0..=1.0).contains(&config.minimum_formation_probability)
        || config.minimum_region_size == 0
        || config.minimum_region_size
            > config
                .map2b_protocol
                .map2a_protocol
                .map1_protocol
                .confirmation_candidate_count
        || !(0.0..0.25).contains(&config.maximum_reset_advantage)
        || !(0.0..0.5).contains(&config.minimum_no_supply_accuracy_drop)
        || !(0.0..0.5).contains(&config.minimum_frozen_accuracy_drop)
    {
        return Err(EmbodiedError::InvalidExperiment);
    }
    Ok(())
}
