use neuro_engine::{
    M1Rule, RepresentationCapacityConfig, RepresentationCapacityDecision,
    run_representation_capacity_diagnostic,
};

fn quick_config() -> RepresentationCapacityConfig {
    let mut config = RepresentationCapacityConfig::default();
    config.m1_protocol.development_seed_count = 1;
    config.m1_protocol.confirmation_seed_count = 1;
    config.probe_training_trials = 32;
    config.probe_evaluation_trials = 32;
    config.shuffled_label_repeats = 16;
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
fn representation_diagnostic_is_exactly_deterministic() {
    let first = run_representation_capacity_diagnostic(quick_config()).expect("first M1-R");
    let second = run_representation_capacity_diagnostic(quick_config()).expect("second M1-R");
    assert_eq!(first, second);
}

#[test]
fn representation_diagnostic_keeps_probes_offline_and_budgets_matched() {
    let result = run_representation_capacity_diagnostic(quick_config()).expect("M1-R");
    assert!(result.acceptance.diagnostic_is_offline_only);
    assert!(result.acceptance.main_stream_unchanged);
    assert!(result.acceptance.action_readout_remained_frozen);
    assert!(result.acceptance.topology_and_connection_budget_preserved);
    assert!(
        result
            .acceptance
            .target_credit_uses_matched_trials_and_variables
    );
    assert!(result.acceptance.finite_outputs);
    assert!(result.acceptance.stage_passed);
    assert!(result.acceptance.passed);
    for row in result
        .development_seed_results
        .iter()
        .chain(&result.confirmation_seed_results)
    {
        assert_eq!(row.rule_results.len(), 4);
        for (rule_result, rule) in row.rule_results.iter().zip(M1Rule::UNIQUE) {
            assert_eq!(rule_result.rule, rule);
            assert_eq!(rule_result.adjustable_connection_count, 48);
            assert_eq!(
                rule_result.action_readout_digest_before,
                rule_result.reward_local_action_readout_digest_after
            );
            assert_eq!(
                rule_result.action_readout_digest_before,
                rule_result.target_directed_action_readout_digest_after
            );
            assert_eq!(
                rule_result.topology_digest_before,
                rule_result.reward_local_topology_digest_after
            );
            assert_eq!(
                rule_result.topology_digest_before,
                rule_result.target_directed_topology_digest_after
            );
        }
    }
}

#[test]
fn representation_diagnostic_validates_positive_and_negative_probe_controls() {
    let result = run_representation_capacity_diagnostic(quick_config()).expect("M1-R");
    assert!(result.acceptance.raw_sensor_linearity_control_valid);
    assert!(result.acceptance.shuffled_label_controls_valid);
    assert!(
        result.confirmation_summaries[0]
            .raw_sensor_probe_accuracy
            .mean
            >= 0.95
    );
    assert!(
        result.confirmation_summaries[1]
            .raw_sensor_probe_accuracy
            .mean
            >= 0.95
    );
    assert!(
        result.confirmation_summaries[2]
            .raw_sensor_probe_accuracy
            .mean
            <= 0.60
    );
    assert!(
        result.confirmation_summaries[3]
            .raw_sensor_probe_accuracy
            .mean
            <= 0.60
    );
    assert!(matches!(
        result.decision,
        RepresentationCapacityDecision::BaselineCapabilityPresent
            | RepresentationCapacityDecision::CreditRoutingBottleneck
            | RepresentationCapacityDecision::LearnedRepresentationReadoutBottleneck
            | RepresentationCapacityDecision::LatentSymbolCodeWithoutRuleFormation
            | RepresentationCapacityDecision::RepresentationInsufficient
            | RepresentationCapacityDecision::InconclusiveBoundary
            | RepresentationCapacityDecision::DiagnosticUnstable
    ));
}

#[test]
fn invalid_representation_protocol_is_rejected() {
    for config in [
        RepresentationCapacityConfig {
            probe_training_trials: 24,
            ..quick_config()
        },
        RepresentationCapacityConfig {
            probe_evaluation_trials: 0,
            ..quick_config()
        },
        RepresentationCapacityConfig {
            probe_ridge: 0.0,
            ..quick_config()
        },
        RepresentationCapacityConfig {
            shuffled_label_repeats: 0,
            ..quick_config()
        },
    ] {
        assert!(run_representation_capacity_diagnostic(config).is_err());
    }
}

#[test]
fn published_m1_representation_diagnostic_is_frozen() {
    let result = run_representation_capacity_diagnostic(RepresentationCapacityConfig::default())
        .expect("formal M1-R");
    assert_eq!(
        serde_json::to_string_pretty(&result.published()).expect("published M1-R JSON"),
        include_str!("../app/public/representation-capacity-v0.7.json")
    );
    assert_eq!(
        result.decision,
        RepresentationCapacityDecision::LatentSymbolCodeWithoutRuleFormation
    );
    assert!(result.acceptance.latent_novel_code_accessible);
    assert!(!result.acceptance.reward_local_representation_formed);
    assert!(result.acceptance.alternative_readout_rescues_novel_rules);
    assert!(!result.acceptance.target_directed_credit_rescues_novel_rules);
    assert!(result.acceptance.stage_passed);
    assert!(result.acceptance.passed);

    let expected_m1_single_rule = [
        0.857638888888889,
        0.529947916666667,
        0.547743055555555,
        0.582465277777778,
    ];
    for (summary, expected) in result
        .confirmation_summaries
        .iter()
        .zip(expected_m1_single_rule)
    {
        assert!((summary.reward_local_behavior_accuracy.mean - expected).abs() <= 1e-12);
    }
}
