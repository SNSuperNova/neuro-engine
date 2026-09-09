#![allow(clippy::field_reassign_with_default)]

use neuro_engine::{
    M1NE_CONTROL_COUNT, M1NEConfig, M1NEControl, M1NEDecision, run_m1ne_experiment,
};

fn quick_config() -> M1NEConfig {
    let mut config = M1NEConfig::default();
    config
        .m1xe_protocol
        .m1x_protocol
        .m1_protocol
        .development_seed_count = 1;
    config
        .m1xe_protocol
        .m1x_protocol
        .m1_protocol
        .confirmation_seed_count = 1;
    config.m1xe_protocol.m1x_protocol.oracle_iterations = 20;
    config.m1xe_protocol.m1x_protocol.oracle_training_trials = 8;
    config.m1xe_protocol.m1x_protocol.oracle_restart_count = 1;
    config
        .m1xe_protocol
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
fn m1ne_is_exactly_deterministic() {
    assert_eq!(
        run_m1ne_experiment(quick_config()).expect("first M1-NE"),
        run_m1ne_experiment(quick_config()).expect("second M1-NE")
    );
}

#[test]
fn m1ne_runs_all_five_controls_without_online_parameter_selection() {
    let result = run_m1ne_experiment(quick_config()).expect("M1-NE");
    assert_eq!(result.development_summaries.len(), M1NE_CONTROL_COUNT);
    assert_eq!(result.confirmation_summaries.len(), M1NE_CONTROL_COUNT);
    for control in M1NEControl::ALL {
        assert_eq!(
            result
                .confirmation_summaries
                .iter()
                .filter(|row| row.control == control)
                .count(),
            1
        );
    }
    assert!(result.acceptance.no_online_hyperparameter_selection);
    assert!(result.acceptance.development_controls_complete);
    assert!(result.acceptance.confirmation_controls_complete);
}

#[test]
fn m1ne_preserves_the_fixed_subspace_and_online_information_boundary() {
    let result = run_m1ne_experiment(quick_config()).expect("M1-NE");
    assert!(result.acceptance.online_candidate_observation_scope_local);
    assert!(
        result
            .acceptance
            .target_rule_and_oracle_hidden_from_online_controls
    );
    assert!(result.acceptance.paired_carriers_trials_and_consequences);
    assert!(result.acceptance.action_readout_remained_frozen);
    assert!(result.acceptance.topology_and_connection_budget_preserved);
    assert!(result.acceptance.finite_outputs);
    for row in result
        .development_seed_results
        .iter()
        .chain(&result.confirmation_seed_results)
    {
        assert_eq!(row.rule_results.len(), 4);
        assert_eq!(
            row.norm_multiplier,
            if row.control == M1NEControl::RewardLocalOneX {
                1.0
            } else {
                1.5
            }
        );
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
    assert!(matches!(
        result.decision,
        M1NEDecision::OnlineSufficiencyConfirmed
            | M1NEDecision::AmplitudeNecessaryButInsufficient
            | M1NEDecision::ConsequenceIndependent
            | M1NEDecision::HistoryDegraded
            | M1NEDecision::OracleAnchorFailed
            | M1NEDecision::DynamicsUnstable
    ));
}

#[test]
fn invalid_m1ne_protocol_is_rejected() {
    for config in [
        M1NEConfig {
            norm_multiplier: 2.0,
            ..quick_config()
        },
        M1NEConfig {
            minimum_causal_advantage: -0.1,
            ..quick_config()
        },
        M1NEConfig {
            maximum_saturation_fraction: 1.1,
            ..quick_config()
        },
    ] {
        assert!(run_m1ne_experiment(config).is_err());
    }
}

#[test]
fn formal_m1ne_result_matches_the_frozen_release_artifact() {
    let result = run_m1ne_experiment(M1NEConfig::default()).expect("formal M1-NE");
    assert_eq!(
        serde_json::to_string_pretty(&result.published()).expect("published M1-NE JSON"),
        include_str!("../app/public/norm-enabled-sufficiency-v1.1.json")
    );
    assert_eq!(
        result.decision,
        M1NEDecision::AmplitudeNecessaryButInsufficient
    );
    assert!(result.acceptance.stage_passed);
    assert!(result.acceptance.passed);
    assert!(result.acceptance.oracle_anchor_passed);
    assert!(result.acceptance.learned_advantage_passed);
    assert!(result.acceptance.consequence_specificity_passed);
    assert!(result.acceptance.history_preserved);
    assert!(!result.acceptance.candidate_capability_passed);
    assert!(!result.acceptance.envelope_advantage_passed);
    assert!(!result.acceptance.target_probability_formation_passed);
    assert!(!result.acceptance.online_sufficiency_confirmed);

    let candidate = result
        .confirmation_summaries
        .iter()
        .find(|row| row.control == M1NEControl::RewardLocalOnePointFive)
        .expect("1.5x online candidate");
    assert_eq!(candidate.run_count, 48);
    assert_eq!(candidate.mean_novel_final_accuracy.mean, 0.5902777777777776);
    assert_eq!(
        candidate.mean_novel_final_target_probability.mean,
        0.5966023880223009
    );
    assert_eq!(
        candidate.mean_minimum_novel_accuracy.mean,
        0.4913194444444445
    );
    assert_eq!(
        result
            .confirmation_effects
            .candidate_over_frozen_behavior
            .mean,
        0.14134837962962965
    );
    assert_eq!(
        result
            .confirmation_effects
            .candidate_over_random_behavior
            .mean,
        0.10850694444444442
    );
    let oracle = result
        .confirmation_summaries
        .iter()
        .find(|row| row.control == M1NEControl::OracleOnePointFive)
        .expect("1.5x oracle anchor");
    assert_eq!(oracle.mean_novel_final_accuracy.mean, 0.8813657407407406);
    assert!(oracle.capability_passed);
}
