use neuro_engine::{
    Map0Control, Map2AExperimentConfig, Map2BExperimentConfig, Map2CExperimentConfig,
    run_map2a_experiment, run_map2b_experiment, run_map2c_experiment,
};

fn quick_config() -> Map2AExperimentConfig {
    let mut config = Map2AExperimentConfig::default();
    config.map1_protocol.development_config_count = 4;
    config.map1_protocol.development_seed_count = 2;
    config.map1_protocol.confirmation_seed_count = 2;
    config.map1_protocol.confirmation_candidate_count = 3;
    config.map1_protocol.probe_protocol.pretraining_episodes = 40;
    config.map1_protocol.probe_protocol.adaptation_episodes = 40;
    config.map1_protocol.probe_protocol.evaluation_episodes = 20;
    config.map1_protocol.probe_protocol.threshold_check_interval = 10;
    config
}

#[test]
fn map2a_is_exactly_deterministic() {
    let first = run_map2a_experiment(quick_config()).expect("first Map 2A run");
    let second = run_map2a_experiment(quick_config()).expect("second Map 2A run");
    assert_eq!(first, second);
}

#[test]
fn map2a_pairs_soft_bounded_and_additive_on_identical_worlds() {
    let config = quick_config();
    let result = run_map2a_experiment(config).expect("Map 2A run");
    assert_eq!(
        result.development_seed_results.len(),
        config.map1_protocol.development_config_count * config.map1_protocol.development_seed_count
    );
    assert_eq!(
        result.confirmation_seed_results.len(),
        result.additive_seed_results.len()
    );
    for (bounded, additive) in result
        .confirmation_seed_results
        .iter()
        .zip(&result.additive_seed_results)
    {
        assert_eq!(bounded.parameter_id, additive.parameter_id);
        assert_eq!(bounded.seed, additive.seed);
        assert_eq!(bounded.control, Map0Control::Baseline);
        assert_eq!(additive.control, Map0Control::AdditivePlasticity);
    }
    assert!(result.acceptance.only_plasticity_mechanism_changed);
    assert!(result.acceptance.additive_controls_complete);
    assert!(result.acceptance.passed);
}

#[test]
fn map2a_keeps_all_outputs_finite() {
    let result = run_map2a_experiment(quick_config()).expect("Map 2A run");
    assert!(result.acceptance.finite_outputs);
    assert!(
        result
            .development_seed_results
            .iter()
            .chain(&result.confirmation_seed_results)
            .chain(&result.additive_seed_results)
            .chain(&result.control_seed_results)
            .all(|row| row.dynamics.maximum_absolute_weight.is_finite()
                && row.dynamics.mean_relative_weight_drift.is_finite())
    );
}

#[test]
fn invalid_map2a_protocol_is_rejected() {
    for config in [
        Map2AExperimentConfig {
            soft_bound_scale: 0.0,
            ..quick_config()
        },
        Map2AExperimentConfig {
            minimum_mean_weight_drift_reduction: 0.0,
            ..quick_config()
        },
        Map2AExperimentConfig {
            maximum_accuracy_degradation: 0.2,
            ..quick_config()
        },
    ] {
        assert!(run_map2a_experiment(config).is_err());
    }
}

fn quick_resource_config() -> Map2BExperimentConfig {
    Map2BExperimentConfig {
        map2a_protocol: quick_config(),
        ..Map2BExperimentConfig::default()
    }
}

#[test]
fn map2b_is_deterministic_and_pairs_resource_controls() {
    let config = quick_resource_config();
    let first = run_map2b_experiment(config).expect("first Map 2B run");
    let second = run_map2b_experiment(config).expect("second Map 2B run");
    assert_eq!(first, second);
    assert_eq!(
        first.confirmation_seed_results.len(),
        first.no_resource_seed_results.len()
    );
    assert_eq!(
        first.confirmation_seed_results.len(),
        first.no_supply_seed_results.len()
    );
    for ((resource, disabled), no_supply) in first
        .confirmation_seed_results
        .iter()
        .zip(&first.no_resource_seed_results)
        .zip(&first.no_supply_seed_results)
    {
        assert_eq!(resource.parameter_id, disabled.parameter_id);
        assert_eq!(resource.parameter_id, no_supply.parameter_id);
        assert_eq!(resource.seed, disabled.seed);
        assert_eq!(resource.seed, no_supply.seed);
        assert_eq!(disabled.control, Map0Control::NoResourceAccounting);
        assert_eq!(no_supply.control, Map0Control::NoResourceSupply);
    }
    assert!(first.acceptance.passed);
    assert!(first.acceptance.no_resource_controls_complete);
    assert!(first.acceptance.no_supply_controls_complete);
}

#[test]
fn disabled_resource_is_neutral_and_no_supply_depletes() {
    let result = run_map2b_experiment(quick_resource_config()).expect("Map 2B run");
    assert!(result.no_resource_seed_results.iter().all(|row| {
        row.dynamics.mean_resource_level == 1.0
            && row.dynamics.minimum_resource_level == 1.0
            && row.dynamics.resource_constrained_fraction == 0.0
    }));
    let supplied = result
        .confirmation_seed_results
        .iter()
        .map(|row| row.dynamics.mean_resource_level)
        .sum::<f64>()
        / result.confirmation_seed_results.len() as f64;
    let unsupplied = result
        .no_supply_seed_results
        .iter()
        .map(|row| row.dynamics.mean_resource_level)
        .sum::<f64>()
        / result.no_supply_seed_results.len() as f64;
    assert!(supplied > unsupplied);
}

#[test]
fn invalid_map2b_resource_protocol_is_rejected() {
    for config in [
        Map2BExperimentConfig {
            supply_rate: 0.0,
            ..quick_resource_config()
        },
        Map2BExperimentConfig {
            minimum_modulation: 1.0,
            ..quick_resource_config()
        },
        Map2BExperimentConfig {
            activity_cost: -0.1,
            ..quick_resource_config()
        },
    ] {
        assert!(run_map2b_experiment(config).is_err());
    }
}

fn quick_stream_config() -> Map2CExperimentConfig {
    Map2CExperimentConfig {
        map2b_protocol: quick_resource_config(),
        stream_trial_count: 160,
        minimum_change_gap: 15,
        change_probability: 0.05,
        settling_window: 10,
        minimum_rule_changes: 2,
        minimum_region_size: 2,
        ..Map2CExperimentConfig::default()
    }
}

#[test]
fn map2c_is_deterministic_and_pairs_continuous_controls() {
    let config = quick_stream_config();
    let first = run_map2c_experiment(config).expect("first Map 2C run");
    let second = run_map2c_experiment(config).expect("second Map 2C run");
    assert_eq!(first, second);
    assert_eq!(
        first.confirmation_seed_results.len(),
        first.reset_seed_results.len()
    );
    assert_eq!(
        first.confirmation_seed_results.len(),
        first.no_supply_seed_results.len()
    );
    assert_eq!(
        first.confirmation_seed_results.len(),
        first.frozen_seed_results.len()
    );
    for (((baseline, reset), no_supply), frozen) in first
        .confirmation_seed_results
        .iter()
        .zip(&first.reset_seed_results)
        .zip(&first.no_supply_seed_results)
        .zip(&first.frozen_seed_results)
    {
        assert_eq!(baseline.parameter_id, reset.parameter_id);
        assert_eq!(baseline.parameter_id, no_supply.parameter_id);
        assert_eq!(baseline.parameter_id, frozen.parameter_id);
        assert_eq!(baseline.seed, reset.seed);
        assert_eq!(baseline.seed, no_supply.seed);
        assert_eq!(baseline.seed, frozen.seed);
        assert_eq!(reset.control, Map0Control::ResetBetweenTrials);
        assert_eq!(no_supply.control, Map0Control::NoResourceSupply);
        assert_eq!(frozen.control, Map0Control::FrozenPlasticity);
    }
    assert!(first.acceptance.passed);
}

#[test]
fn map2c_baseline_never_resets_state_and_outputs_are_finite() {
    let result = run_map2c_experiment(quick_stream_config()).expect("Map 2C run");
    assert!(result.acceptance.continuous_stream_has_no_state_resets);
    assert!(
        result
            .development_seed_results
            .iter()
            .chain(&result.confirmation_seed_results)
            .all(|row| row.state_reset_count == 0 && row.finite)
    );
    assert!(
        result
            .reset_seed_results
            .iter()
            .all(|row| row.state_reset_count == row.trial_count)
    );
}

#[test]
fn invalid_map2c_stream_protocol_is_rejected() {
    for config in [
        Map2CExperimentConfig {
            minimum_change_gap: 0,
            ..quick_stream_config()
        },
        Map2CExperimentConfig {
            settling_window: 15,
            ..quick_stream_config()
        },
        Map2CExperimentConfig {
            change_probability: 1.0,
            ..quick_stream_config()
        },
    ] {
        assert!(run_map2c_experiment(config).is_err());
    }
}

#[test]
fn published_map2_stage_decisions_are_frozen() {
    let map2a = run_map2a_experiment(Map2AExperimentConfig::default()).expect("formal Map 2A");
    assert_eq!(map2a.confirmation_parameter_ids, [38, 18, 2, 14, 26, 6]);
    assert!(map2a.acceptance.stage_passed);
    assert!(!map2a.acceptance.stable_region_found);

    let map2b = run_map2b_experiment(Map2BExperimentConfig::default()).expect("formal Map 2B");
    assert_eq!(map2b.confirmation_parameter_ids, [30, 10, 14, 34, 44, 22]);
    assert!(map2b.acceptance.stage_passed);
    assert!(!map2b.acceptance.stable_region_found);

    let map2c = run_map2c_experiment(Map2CExperimentConfig::default()).expect("formal Map 2C");
    assert_eq!(map2c.confirmation_parameter_ids, [0, 20, 29, 45, 16, 21]);
    assert_eq!(map2c.stable_region_parameter_ids, [0, 16, 20, 21, 29, 45]);
    assert!(map2c.acceptance.stage_passed);
    assert!(map2c.acceptance.passed);
}
