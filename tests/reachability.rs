use neuro_engine::{
    M1X_LEARNING_RATE_COUNT, M1XConfig, M1XControl, M1XDecision, run_m1x_experiment,
};

fn quick_config() -> M1XConfig {
    let mut config = M1XConfig::default();
    config.m1_protocol.development_seed_count = 1;
    config.m1_protocol.confirmation_seed_count = 1;
    config.oracle_iterations = 20;
    config.oracle_training_trials = 8;
    config.oracle_restart_count = 1;
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
fn m1x_is_exactly_deterministic() {
    let first = run_m1x_experiment(quick_config()).expect("first M1-X");
    let second = run_m1x_experiment(quick_config()).expect("second M1-X");
    assert_eq!(first, second);
}

#[test]
fn m1x_selects_learning_rate_only_from_development() {
    let result = run_m1x_experiment(quick_config()).expect("M1-X");
    assert_eq!(
        result.development_learning_rate_summaries.len(),
        M1X_LEARNING_RATE_COUNT
    );
    assert!(
        result
            .config
            .learning_rates
            .contains(&result.selected_learning_rate)
    );
    assert!(result.acceptance.development_learning_rate_scan_complete);
    assert!(
        result
            .acceptance
            .selected_learning_rate_from_development_only
    );
}

#[test]
fn m1x_keeps_the_live_interface_and_every_budget_frozen() {
    let result = run_m1x_experiment(quick_config()).expect("M1-X");
    assert!(result.acceptance.diagnostic_is_offline_only);
    assert!(result.acceptance.exact_gradient_optimizer_verified);
    assert!(result.acceptance.matched_search_budgets);
    assert!(result.acceptance.action_readout_remained_frozen);
    assert!(result.acceptance.topology_and_connection_budget_preserved);
    assert!(result.acceptance.finite_outputs);
    assert!(result.acceptance.stage_passed);
    assert!(result.acceptance.passed);
    assert_eq!(result.confirmation_summaries.len(), 4);
    for control in M1XControl::all() {
        assert!(
            result
                .confirmation_summaries
                .iter()
                .any(|row| row.control == control)
        );
    }
    for row in result
        .development_seed_results
        .iter()
        .chain(&result.confirmation_seed_results)
    {
        assert_eq!(row.rule_results.len(), 4);
        for rule in &row.rule_results {
            assert_eq!(
                rule.action_readout_digest_before,
                rule.action_readout_digest_after
            );
            assert_eq!(rule.topology_digest_before, rule.topology_digest_after);
            assert_eq!(rule.adjustable_connection_count, 48);
            assert!(rule.finite);
        }
    }
}

#[test]
fn invalid_m1x_protocol_is_rejected() {
    let mut config = quick_config();
    config.oracle_restart_count = 0;
    assert!(run_m1x_experiment(config).is_err());
}

#[test]
fn published_m1x_result_is_frozen() {
    let result = run_m1x_experiment(M1XConfig::default()).expect("formal M1-X");
    let published = serde_json::to_string_pretty(&result.published()).expect("published JSON");
    assert_eq!(
        published,
        include_str!("../app/public/reachability-v0.9.json")
    );
    assert_eq!(result.selected_learning_rate, 0.08);
    assert_eq!(result.decision, M1XDecision::ReachableOnlyAtAbsoluteBounds);
    assert!(result.acceptance.absolute_reachability_passed);
    assert!(!result.acceptance.envelope_reachability_passed);
    assert!(result.acceptance.optimizer_effective);
    assert!(result.acceptance.task_specificity_passed);
    let absolute = result
        .confirmation_summaries
        .iter()
        .find(|row| row.control == M1XControl::AbsoluteBoundOracle)
        .expect("absolute summary");
    let envelope = result
        .confirmation_summaries
        .iter()
        .find(|row| row.control == M1XControl::NormEnvelopeOracle)
        .expect("envelope summary");
    assert_eq!(
        absolute.mean_novel_behavior_accuracy.mean,
        0.9725115740740743
    );
    assert_eq!(
        absolute.mean_novel_target_probability.mean,
        0.9691440831244306
    );
    assert_eq!(
        envelope.mean_minimum_novel_accuracy.mean,
        0.5577256944444444
    );
}
