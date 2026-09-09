#![allow(clippy::field_reassign_with_default)]

use neuro_engine::{M2CControl, M2CDecision, M2CExperimentConfig, run_m2c_experiment};

fn quick_config() -> M2CExperimentConfig {
    let mut config = M2CExperimentConfig::default();
    config.m1_protocol.development_seed_count = 1;
    config.m1_protocol.confirmation_seed_count = 2;
    config.m1_protocol.phase_trial_count = 40;
    config.m1_protocol.evaluation_trial_count = 16;
    config.m1_protocol.threshold_check_interval = 10;
    config.candidate_rewiring_intervals = [4, 8, 12, 20];
    config
        .m1_protocol
        .map2_protocol
        .map2b_protocol
        .map2a_protocol
        .map1_protocol
        .probe_protocol
        .pretraining_episodes = 80;
    config
}

#[test]
fn m2c_is_exactly_deterministic() {
    let first = run_m2c_experiment(quick_config()).expect("first M2C run");
    let second = run_m2c_experiment(quick_config()).expect("second M2C run");
    assert_eq!(first, second);
}

#[test]
fn m2c_preserves_connection_and_readout_budgets() {
    let result = run_m2c_experiment(quick_config()).expect("M2C run");
    assert!(result.acceptance.connection_budget_preserved);
    assert!(result.acceptance.action_readout_remained_frozen);
    assert!(result.confirmation_seed_results.iter().all(|row| {
        row.allocated_connection_count_before == 144
            && row.allocated_connection_count_after == 144
            && row.action_readout_digest_before == row.action_readout_digest_after
    }));
    assert!(result.confirmation_seed_results.iter().all(|row| {
        if matches!(
            row.control,
            M2CControl::LocalEvidenceRewiring | M2CControl::RandomRewiring
        ) {
            row.sequence_rewire_count > 0 && row.topology_digest_before != row.topology_digest_after
        } else {
            row.sequence_rewire_count == 0
                && row.topology_digest_before == row.topology_digest_after
        }
    }));
}

#[test]
fn m2c_pairs_local_random_weight_and_frozen_controls() {
    let config = quick_config();
    let result = run_m2c_experiment(config).expect("M2C run");
    let expected = config.m1_protocol.reference_parameter_ids.len()
        * config.m1_protocol.confirmation_seed_count;
    for control in M2CControl::CONFIRMATION {
        assert_eq!(
            result
                .confirmation_seed_results
                .iter()
                .filter(|row| row.control == control)
                .count(),
            expected
        );
    }
    assert!(result.acceptance.local_and_random_rewire_counts_matched);
    assert!(result.acceptance.confirmation_controls_complete);
}

#[test]
fn m2c_reports_a_valid_conditional_decision() {
    let result = run_m2c_experiment(quick_config()).expect("M2C run");
    assert!(result.acceptance.stage_passed);
    assert!(result.acceptance.passed);
    assert!(matches!(
        result.decision,
        M2CDecision::CandidateAccepted
            | M2CDecision::NoBenefitOverWeightOnly
            | M2CDecision::RandomRewiringEquivalent
            | M2CDecision::CapacityStillInsufficient
            | M2CDecision::RetentionStillInsufficient
            | M2CDecision::ResourceExhaustion
            | M2CDecision::DynamicsInstability
    ));
}

#[test]
fn invalid_m2c_protocol_is_rejected() {
    for config in [
        M2CExperimentConfig {
            candidate_rewiring_intervals: [0, 8, 16, 32],
            ..quick_config()
        },
        M2CExperimentConfig {
            candidate_evidence_decays: [1.0, 0.9],
            ..quick_config()
        },
        M2CExperimentConfig {
            minimum_single_rule_accuracy: 1.1,
            ..quick_config()
        },
    ] {
        assert!(run_m2c_experiment(config).is_err());
    }
}

#[test]
fn published_m2c_negative_boundary_is_frozen() {
    let result = run_m2c_experiment(M2CExperimentConfig::default()).expect("formal M2C run");
    assert_eq!(
        serde_json::to_string_pretty(&result).expect("M2C JSON"),
        include_str!("../app/public/mechanism-m2c-v0.3.json")
    );
    assert_eq!(result.decision, M2CDecision::NoBenefitOverWeightOnly);
    assert!(!result.acceptance.mechanism_accepted);
    assert!(result.acceptance.retention_boundary_passed);
    assert!(!result.acceptance.single_rule_capacity_boundary_passed);
    assert!(!result.acceptance.continuous_learning_boundary_passed);
    assert!(result.acceptance.stage_passed);
    assert!(result.acceptance.passed);
}
