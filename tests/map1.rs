#![allow(clippy::field_reassign_with_default)]

use neuro_engine::{Map0Control, Map0RegionClass, Map1ExperimentConfig, run_map1_experiment};

fn quick_config() -> Map1ExperimentConfig {
    let mut config = Map1ExperimentConfig::default();
    config.development_config_count = 4;
    config.development_seed_count = 2;
    config.confirmation_seed_count = 2;
    config.confirmation_candidate_count = 3;
    config.probe_protocol.pretraining_episodes = 40;
    config.probe_protocol.adaptation_episodes = 40;
    config.probe_protocol.evaluation_episodes = 20;
    config.probe_protocol.threshold_check_interval = 10;
    config
}

#[test]
fn map1_is_exactly_deterministic() {
    let first = run_map1_experiment(quick_config()).expect("first Map 1 run");
    let second = run_map1_experiment(quick_config()).expect("second Map 1 run");
    assert_eq!(first, second);
}

#[test]
fn map1_pairs_dual_and_reference_mechanisms_on_identical_worlds() {
    let config = quick_config();
    let result = run_map1_experiment(config).expect("Map 1 run");
    assert_eq!(
        result.development_seed_results.len(),
        config.development_config_count * config.development_seed_count
    );
    assert_eq!(
        result.confirmation_seed_results.len(),
        config.confirmation_candidate_count * config.confirmation_seed_count
    );
    assert_eq!(
        result.reference_norm_seed_results.len(),
        result.confirmation_seed_results.len()
    );
    assert_eq!(
        result.control_seed_results.len(),
        config.confirmation_candidate_count * 5 * config.confirmation_seed_count
    );
    for (dual, reference) in result
        .confirmation_seed_results
        .iter()
        .zip(&result.reference_norm_seed_results)
    {
        assert_eq!(dual.parameter_id, reference.parameter_id);
        assert_eq!(dual.seed, reference.seed);
        assert_eq!(dual.control, Map0Control::Baseline);
        assert_eq!(reference.control, Map0Control::ReferenceNormHomeostasis);
    }
    assert!(result.acceptance.only_homeostasis_mechanism_changed);
    assert!(
        result
            .acceptance
            .development_and_confirmation_seeds_disjoint
    );
    assert!(result.acceptance.reference_mechanism_controls_complete);
    assert!(result.acceptance.passed);
}

#[test]
fn map1_dual_timescale_state_is_finite_and_bounded() {
    let config = quick_config();
    let result = run_map1_experiment(config).expect("Map 1 run");
    for row in result
        .development_seed_results
        .iter()
        .chain(&result.confirmation_seed_results)
        .chain(&result.control_seed_results)
    {
        assert!(row.dynamics.mean_excitability_gain.is_finite());
        assert!(
            row.dynamics.minimum_excitability_gain
                >= config.minimum_excitability_gain - f64::EPSILON
        );
        assert!(
            row.dynamics.maximum_excitability_gain
                <= config.maximum_excitability_gain + f64::EPSILON
        );
    }
    assert!(result.reference_norm_seed_results.iter().all(|row| {
        row.dynamics.mean_excitability_gain == 1.0
            && row.dynamics.minimum_excitability_gain == 1.0
            && row.dynamics.maximum_excitability_gain == 1.0
    }));
}

#[test]
fn invalid_map1_protocol_is_rejected() {
    for config in [
        Map1ExperimentConfig {
            development_seed_count: 1,
            ..quick_config()
        },
        Map1ExperimentConfig {
            confirmation_candidate_count: 2,
            ..quick_config()
        },
        Map1ExperimentConfig {
            activity_ema_rate: 1.0,
            ..quick_config()
        },
        Map1ExperimentConfig {
            minimum_excitability_gain: 1.1,
            ..quick_config()
        },
    ] {
        assert!(run_map1_experiment(config).is_err());
    }
}

#[test]
fn frozen_map1_protocol_completes_and_rejects_the_mechanism() {
    let result =
        run_map1_experiment(Map1ExperimentConfig::default()).expect("frozen Map 1 experiment");
    assert_eq!(
        result.version,
        "learnability-map/v0.2-dual-timescale-homeostasis"
    );
    assert_eq!(result.development_seed_results.len(), 48 * 8);
    assert_eq!(result.confirmation_seed_results.len(), 6 * 12);
    assert_eq!(result.reference_norm_seed_results.len(), 6 * 12);
    assert_eq!(result.control_seed_results.len(), 6 * 5 * 12);
    assert!(result.acceptance.passed);
    assert!(result.acceptance.causal_intervention_detected);
    assert!(!result.acceptance.stable_region_found);
    assert!(!result.acceptance.mechanism_improves_tradeoff);
    assert!(result.stable_region_parameter_ids.is_empty());
    assert!(
        result
            .confirmation_seed_results
            .iter()
            .all(|row| !row.formed)
    );
    assert_eq!(
        result
            .confirmation_seed_results
            .iter()
            .filter(|row| row.class == Map0RegionClass::Unstable)
            .count(),
        59
    );
    assert!(
        result
            .reference_norm_seed_results
            .iter()
            .all(|row| { row.class == Map0RegionClass::TaskSpecialized && !row.formed })
    );
}
