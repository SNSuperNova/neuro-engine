#![allow(clippy::field_reassign_with_default)]

use neuro_engine::{
    M1XE_MULTIPLIER_COUNT, M1XEBoundKind, M1XEConfig, M1XEDecision, run_m1xe_experiment,
};

fn quick_config() -> M1XEConfig {
    let mut config = M1XEConfig::default();
    config.m1x_protocol.m1_protocol.development_seed_count = 1;
    config.m1x_protocol.m1_protocol.confirmation_seed_count = 1;
    config.m1x_protocol.oracle_iterations = 20;
    config.m1x_protocol.oracle_training_trials = 8;
    config.m1x_protocol.oracle_restart_count = 1;
    config
        .m1x_protocol
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
fn m1xe_is_exactly_deterministic() {
    let first = run_m1xe_experiment(quick_config()).expect("first M1-XE");
    let second = run_m1xe_experiment(quick_config()).expect("second M1-XE");
    assert_eq!(first, second);
}

#[test]
fn m1xe_scans_every_preregistered_multiplier_and_absolute_anchor() {
    let result = run_m1xe_experiment(quick_config()).expect("M1-XE");
    assert_eq!(result.development_summaries.len(), M1XE_MULTIPLIER_COUNT);
    assert_eq!(
        result
            .confirmation_summaries
            .iter()
            .filter(|row| row.bound_kind == M1XEBoundKind::ReferenceMultiplier)
            .count(),
        M1XE_MULTIPLIER_COUNT
    );
    assert_eq!(
        result
            .confirmation_summaries
            .iter()
            .filter(|row| row.bound_kind == M1XEBoundKind::AbsoluteBound)
            .count(),
        1
    );
    assert!(result.acceptance.development_scale_scan_complete);
    assert!(result.acceptance.confirmation_scale_scan_complete);
    assert!(result.acceptance.absolute_anchor_complete);
}

#[test]
fn m1xe_preserves_the_fixed_subspace_interface() {
    let result = run_m1xe_experiment(quick_config()).expect("M1-XE");
    assert!(result.acceptance.diagnostic_is_offline_only);
    assert!(result.acceptance.exact_gradient_optimizer_verified);
    assert!(result.acceptance.action_readout_remained_frozen);
    assert!(result.acceptance.topology_and_connection_budget_preserved);
    assert!(result.acceptance.finite_outputs);
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
fn invalid_m1xe_protocol_is_rejected() {
    let mut config = quick_config();
    config.norm_multipliers[1] = 1.4;
    assert!(run_m1xe_experiment(config).is_err());
}

#[test]
fn formal_m1xe_result_matches_the_frozen_release_artifact() {
    let result = run_m1xe_experiment(M1XEConfig::default()).expect("formal M1-XE");
    let published = serde_json::to_string_pretty(&result.published()).expect("serialize M1-XE");
    assert_eq!(
        published,
        include_str!("../app/public/reachability-envelope-v1.0.json")
    );
    assert_eq!(result.selected_development_multiplier, Some(1.5));
    assert_eq!(result.confirmed_minimum_multiplier, Some(1.5));
    assert_eq!(
        result.passing_confirmation_multipliers,
        vec![1.5, 2.0, 3.0, 4.0]
    );
    assert_eq!(
        result.decision,
        M1XEDecision::UniformEnvelopeBoundaryConfirmed
    );
    assert!(result.acceptance.passed);

    let boundary = result
        .confirmation_summaries
        .iter()
        .find(|row| row.norm_multiplier == Some(1.5))
        .expect("1.5x confirmation boundary");
    assert_eq!(boundary.run_count, 48);
    assert_eq!(
        boundary.mean_novel_behavior_accuracy.mean,
        0.8543113425925924
    );
    assert_eq!(
        boundary.mean_novel_target_probability.mean,
        0.859137441137808
    );
    assert_eq!(
        boundary.mean_minimum_novel_accuracy.mean,
        0.7083333333333335
    );
    assert!(boundary.capability_passed);

    let absolute = result
        .confirmation_summaries
        .iter()
        .find(|row| row.bound_kind == M1XEBoundKind::AbsoluteBound)
        .expect("absolute-bound anchor");
    assert_eq!(
        absolute.mean_novel_behavior_accuracy.mean,
        0.9681712962962964
    );
    assert_eq!(
        absolute.mean_novel_target_probability.mean,
        0.9649209657276036
    );
    assert!(absolute.capability_passed);
}
