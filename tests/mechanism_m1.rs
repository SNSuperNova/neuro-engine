use neuro_engine::{M1Control, M1ExperimentConfig, M1Rule, run_m1_experiment};

fn quick_config() -> M1ExperimentConfig {
    let mut config = M1ExperimentConfig::default();
    config.development_seed_count = 1;
    config.confirmation_seed_count = 2;
    config.phase_trial_count = 40;
    config.evaluation_trial_count = 16;
    config.threshold_check_interval = 10;
    config
        .map2_protocol
        .map2b_protocol
        .map2a_protocol
        .map1_protocol
        .probe_protocol
        .pretraining_episodes = 80;
    config
}

#[test]
fn m1_is_exactly_deterministic() {
    let first = run_m1_experiment(quick_config()).expect("first M1 run");
    let second = run_m1_experiment(quick_config()).expect("second M1 run");
    assert_eq!(first, second);
}

#[test]
fn m1_freezes_four_balanced_rules_and_exact_sequence() {
    let result = run_m1_experiment(quick_config()).expect("M1 run");
    assert_eq!(result.rules.len(), 4);
    assert!(
        result
            .rules
            .iter()
            .all(|rule| rule.iter().filter(|target| **target).count() == 2)
    );
    assert_eq!(
        result.sequence,
        [M1Rule::A, M1Rule::B, M1Rule::C, M1Rule::D, M1Rule::A]
    );
    assert!(result.acceptance.four_fixed_balanced_rules_complete);
    assert!(result.acceptance.exact_sequence_complete);
}

#[test]
fn m1_pairs_every_control_and_keeps_baseline_continuous() {
    let config = quick_config();
    let result = run_m1_experiment(config).expect("M1 run");
    let expected = config.reference_parameter_ids.len() * config.confirmation_seed_count;
    let expected_development = config.reference_parameter_ids.len() * config.development_seed_count;
    assert_eq!(
        result.development_single_rule_seed_results.len(),
        expected_development
    );
    assert_eq!(result.confirmation_seed_results.len(), expected);
    assert_eq!(result.frozen_seed_results.len(), expected);
    assert_eq!(result.random_consequence_seed_results.len(), expected);
    assert_eq!(result.reset_seed_results.len(), expected);
    assert_eq!(result.single_rule_seed_results.len(), expected);
    assert!(result.confirmation_seed_results.iter().all(|row| {
        row.control == M1Control::Baseline
            && row
                .phase_results
                .iter()
                .map(|phase| phase.rule)
                .collect::<Vec<_>>()
                == result.sequence
            && row.state_reset_count == 0
    }));
    assert!(
        result.reset_seed_results.iter().all(|row| {
            row.state_reset_count == config.phase_trial_count * result.sequence.len()
        })
    );
    assert!(
        result
            .development_single_rule_seed_results
            .iter()
            .all(|row| row.single_rule_results.len() == 4)
    );
    assert!(
        result
            .single_rule_seed_results
            .iter()
            .all(|row| row.single_rule_results.len() == 4)
    );
}

#[test]
fn m1_freezes_action_readout_and_classifies_the_boundary() {
    let result = run_m1_experiment(quick_config()).expect("M1 run");
    assert!(result.acceptance.action_readout_remained_frozen);
    assert!(result.acceptance.failure_boundary_classified);
    assert!(result.acceptance.stage_passed);
    assert!(result.acceptance.passed);
    assert!(result.confirmation_seed_results.iter().all(|row| {
        row.action_readout_digest_before == row.action_readout_digest_after && row.finite
    }));
}

#[test]
fn invalid_m1_protocol_is_rejected() {
    for config in [
        M1ExperimentConfig {
            phase_trial_count: 0,
            ..quick_config()
        },
        M1ExperimentConfig {
            accuracy_threshold: 0.4,
            ..quick_config()
        },
        M1ExperimentConfig {
            reference_parameter_ids: [0, 16, 20, 21, 29, 99],
            ..quick_config()
        },
    ] {
        assert!(run_m1_experiment(config).is_err());
    }
}

#[test]
fn published_m1_capacity_boundary_is_frozen() {
    let result = run_m1_experiment(M1ExperimentConfig::default()).expect("formal M1 run");
    assert_eq!(
        serde_json::to_string_pretty(&result).expect("M1 JSON"),
        include_str!("../app/public/mechanism-m1-v0.2.json")
    );
    assert_eq!(
        result.dominant_failure_class,
        neuro_engine::M1FailureClass::CapacityInsufficient
    );
    assert!(!result.acceptance.retention_boundary_passed);
    assert!(result.acceptance.plasticity_causally_engaged);
    assert!(result.acceptance.consequence_causally_engaged);
    assert!(result.acceptance.reset_does_not_explain_retention);
    assert!(result.acceptance.passed);
}
