use neuro_engine::{Map0ExperimentConfig, Map0RegionClass, run_map0_experiment};

fn quick_config() -> Map0ExperimentConfig {
    Map0ExperimentConfig {
        development_config_count: 4,
        development_seed_count: 2,
        confirmation_seed_count: 2,
        confirmation_candidate_count: 3,
        pretraining_episodes: 40,
        adaptation_episodes: 40,
        evaluation_episodes: 20,
        threshold_check_interval: 10,
        ..Map0ExperimentConfig::default()
    }
}

#[test]
fn map0_is_exactly_deterministic() {
    let first = run_map0_experiment(quick_config()).expect("first Map 0 run");
    let second = run_map0_experiment(quick_config()).expect("second Map 0 run");
    assert_eq!(first, second);
}

#[test]
fn map0_keeps_development_confirmation_and_controls_explicit() {
    let config = quick_config();
    let result = run_map0_experiment(config).expect("Map 0 run");
    assert_eq!(
        result.development_seed_results.len(),
        config.development_config_count * config.development_seed_count
    );
    assert_eq!(
        result.confirmation_seed_results.len(),
        config.confirmation_candidate_count * config.confirmation_seed_count
    );
    assert_eq!(
        result.control_seed_results.len(),
        config.confirmation_candidate_count * 5 * config.confirmation_seed_count
    );
    assert!(
        result
            .acceptance
            .development_and_confirmation_seeds_disjoint
    );
    assert!(result.acceptance.causal_controls_complete);
    assert!(result.acceptance.passed);
}

#[test]
fn map0_samples_every_axis_within_the_frozen_ranges() {
    let config = quick_config();
    let result = run_map0_experiment(config).expect("Map 0 run");
    assert_eq!(result.development_parameters.len(), 4);
    for point in &result.development_parameters {
        assert!(
            (config.recurrent_gain_range[0]..config.recurrent_gain_range[1])
                .contains(&point.recurrent_gain)
        );
        assert!(
            (config.internal_learning_rate_range[0]..config.internal_learning_rate_range[1])
                .contains(&point.internal_learning_rate)
        );
        assert!(
            (config.homeostasis_strength_range[0]..config.homeostasis_strength_range[1])
                .contains(&point.homeostasis_strength)
        );
        assert!(
            (config.exploration_rate_range[0]..config.exploration_rate_range[1])
                .contains(&point.exploration_rate)
        );
    }
}

#[test]
fn formation_requires_the_learnable_stable_class() {
    let result = run_map0_experiment(quick_config()).expect("Map 0 run");
    for row in result
        .development_seed_results
        .iter()
        .chain(&result.confirmation_seed_results)
        .chain(&result.control_seed_results)
    {
        assert_eq!(row.formed, row.class == Map0RegionClass::LearnableStable);
        assert!(row.passed_probe_count <= 6);
    }
}

#[test]
fn invalid_map0_protocol_is_rejected() {
    for config in [
        Map0ExperimentConfig {
            development_seed_count: 1,
            ..quick_config()
        },
        Map0ExperimentConfig {
            confirmation_candidate_count: 2,
            ..quick_config()
        },
        Map0ExperimentConfig {
            adaptation_episodes: 41,
            ..quick_config()
        },
        Map0ExperimentConfig {
            homeostasis_strength_range: [-0.1, 1.0],
            ..quick_config()
        },
    ] {
        assert!(run_map0_experiment(config).is_err());
    }
}

#[test]
fn frozen_map0_protocol_completes_and_records_the_negative_boundary() {
    let config = Map0ExperimentConfig::default();
    let result = run_map0_experiment(config).expect("frozen Map 0 experiment");
    assert_eq!(result.version, "learnability-map/v0.1");
    assert_eq!(result.development_seed_results.len(), 64 * 8);
    assert_eq!(result.confirmation_seed_results.len(), 6 * 12);
    assert_eq!(result.control_seed_results.len(), 6 * 5 * 12);
    assert!(result.acceptance.passed);
    assert!(result.acceptance.causal_intervention_detected);
    assert!(!result.acceptance.stable_region_found);
    assert!(result.stable_region_parameter_ids.is_empty());
    assert!(
        result
            .confirmation_seed_results
            .iter()
            .all(|row| !row.formed)
    );
}
