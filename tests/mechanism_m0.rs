use std::collections::BTreeSet;

use neuro_engine::{
    ActivityAdjustment, ActivityObservation, AdjustableVariable, AdjustmentMechanism,
    HomeostasisAdjustment, HomeostasisObservation, Map0Control, Map0ExperimentConfig,
    Map0ParameterPoint, MechanismStateContract, PlasticityAdjustment, PlasticityObservation,
    REFERENCE_SUBSTRATE_VERSION, mechanism_m0_release, reference_substrate_manifest,
    run_map_seed_with_adjustment_mechanism,
};

#[derive(Clone)]
struct InvalidWrites;

impl AdjustmentMechanism for InvalidWrites {
    fn mechanism_id(&self) -> &'static str {
        "test/invalid-writes"
    }
    fn mechanism_state_contract(&self) -> MechanismStateContract {
        MechanismStateContract {
            scalar_state_per_unit: 0,
            value_range: "none".into(),
            update_budget: "zero".into(),
            freeze_mode: "not applicable".into(),
        }
    }
    fn initial_resource(&self) -> f64 {
        f64::INFINITY
    }
    fn excitability_bounds(&self) -> [f64; 2] {
        [0.5, 1.5]
    }
    fn adjust_activity(&mut self, _: ActivityObservation) -> ActivityAdjustment {
        ActivityAdjustment {
            activity: f64::NAN,
            resource_level: f64::NEG_INFINITY,
        }
    }
    fn adjust_plasticity(&mut self, _: PlasticityObservation) -> PlasticityAdjustment {
        PlasticityAdjustment {
            recurrent_weight: f64::INFINITY,
            resource_level: f64::NAN,
        }
    }
    fn adjust_homeostasis(&mut self, _: HomeostasisObservation) -> HomeostasisAdjustment {
        HomeostasisAdjustment {
            activity_ema: f64::NAN,
            excitability_gain: f64::INFINITY,
            recurrent_weights: [f64::NAN, f64::NEG_INFINITY],
        }
    }
}

#[test]
fn reference_substrate_declares_every_adjustable_variable_once() {
    let manifest = reference_substrate_manifest();
    assert_eq!(manifest.version, REFERENCE_SUBSTRATE_VERSION);
    assert_eq!(manifest.hidden_unit_count, 24);
    assert_eq!(manifest.sensory_channel_count, 4);
    assert_eq!(manifest.action_count, 2);
    assert_eq!(manifest.adjustable_variables.len(), 6);
    let variables = manifest
        .adjustable_variables
        .iter()
        .map(|contract| format!("{:?}", contract.variable))
        .collect::<BTreeSet<_>>();
    assert_eq!(variables.len(), 6);
    for expected in [
        AdjustableVariable::ActivityState,
        AdjustableVariable::SensoryWeights,
        AdjustableVariable::RecurrentWeights,
        AdjustableVariable::Excitability,
        AdjustableVariable::Resource,
        AdjustableVariable::MechanismState,
    ] {
        assert!(variables.contains(&format!("{expected:?}")));
    }
    assert!(manifest.adjustable_variables.iter().all(|contract| {
        !contract.read_scope.is_empty()
            && !contract.update_permission.is_empty()
            && !contract.value_range.is_empty()
            && !contract.update_budget.is_empty()
            && !contract.freeze_mode.is_empty()
    }));
}

#[test]
fn adjustment_interface_forbids_task_and_output_bypass() {
    let release = mechanism_m0_release();
    assert!(release.interface_audit.task_labels_not_exposed);
    assert!(release.interface_audit.readout_state_not_exposed);
    assert!(release.interface_audit.output_writes_not_representable);
    assert!(
        release
            .interface_audit
            .candidate_runner_uses_closed_task_loop
    );
    assert!(
        release
            .carrier_state
            .forbidden_observations
            .iter()
            .any(|item| item.contains("target"))
    );
    assert!(
        release
            .carrier_state
            .forbidden_writes
            .iter()
            .any(|item| item.contains("readout"))
    );
}

#[test]
fn m0_release_freezes_map2_hashes_and_capability_summary() {
    let release = mechanism_m0_release();
    assert_eq!(release.frozen_map2_artifacts.len(), 3);
    assert!(
        release
            .frozen_map2_artifacts
            .iter()
            .all(|artifact| artifact.sha256.len() == 64)
    );
    assert_eq!(
        release.capability_result.stable_region_parameter_ids,
        [0, 16, 20, 21, 29, 45]
    );
    assert!(release.interface_audit.map2_exact_json_regression_tested);
}

#[test]
fn interface_boundary_rejects_non_finite_candidate_writes() {
    let config = Map0ExperimentConfig {
        pretraining_episodes: 4,
        adaptation_episodes: 4,
        evaluation_episodes: 4,
        threshold_check_interval: 2,
        ..Map0ExperimentConfig::default()
    };
    let result = run_map_seed_with_adjustment_mechanism(
        config,
        Map0ParameterPoint {
            id: 0,
            recurrent_gain: 0.5,
            internal_learning_rate: 0.01,
            homeostasis_strength: 0.1,
            exploration_rate: 0.1,
        },
        7,
        Map0Control::Baseline,
        InvalidWrites,
    );
    assert!(result.dynamics.finite_activity_fraction == 1.0);
    assert!(result.dynamics.maximum_absolute_weight.is_finite());
    assert!(result.dynamics.mean_resource_level.is_finite());
}
