#![allow(clippy::field_reassign_with_default)]

use neuro_engine::{
    M1CD_CHECKPOINT_COUNT, M1CD_COMPONENT_COUNT, M1CD_CONTROL_COUNT, M1CDConfig, M1CDControl,
    M1CDDecision, run_m1cd_experiment,
};

fn quick_config() -> M1CDConfig {
    let mut config = M1CDConfig::default();
    config
        .m1ne_protocol
        .m1xe_protocol
        .m1x_protocol
        .m1_protocol
        .development_seed_count = 1;
    config
        .m1ne_protocol
        .m1xe_protocol
        .m1x_protocol
        .m1_protocol
        .confirmation_seed_count = 1;
    config.diagnostic_trial_count = 4;
    config.oracle_trial_count = 4;
    config
}

#[test]
fn m1cd_is_exactly_deterministic() {
    assert_eq!(
        run_m1cd_experiment(quick_config()).expect("first M1-CD"),
        run_m1cd_experiment(quick_config()).expect("second M1-CD")
    );
}

#[test]
fn m1cd_pairs_all_controls_and_components_at_frozen_checkpoints() {
    let result = run_m1cd_experiment(quick_config()).expect("M1-CD");
    assert_eq!(result.development_summaries.len(), M1CD_CONTROL_COUNT);
    assert_eq!(result.confirmation_summaries.len(), M1CD_CONTROL_COUNT);
    for seed in result
        .development_seed_results
        .iter()
        .chain(&result.confirmation_seed_results)
    {
        assert_eq!(seed.norm_multiplier, 1.5);
        assert_eq!(seed.rule_results.len(), 4);
        for rule in &seed.rule_results {
            assert_eq!(rule.checkpoints.len(), M1CD_CHECKPOINT_COUNT);
            assert_eq!(rule.adjustable_connection_count, 48);
            assert_eq!(
                rule.action_readout_digest_before,
                rule.action_readout_digest_after
            );
            assert_eq!(rule.topology_digest_before, rule.topology_digest_after);
            for checkpoint in &rule.checkpoints {
                assert_eq!(checkpoint.controls.len(), M1CD_CONTROL_COUNT);
                for control in M1CDControl::ALL {
                    let row = checkpoint
                        .controls
                        .iter()
                        .find(|row| row.control == control)
                        .expect("paired control");
                    assert_eq!(row.components.len(), M1CD_COMPONENT_COUNT);
                    assert!(row.finite);
                }
            }
        }
    }
    assert!(result.acceptance.checkpoints_complete);
    assert!(result.acceptance.controls_paired_from_identical_state);
    assert!(result.acceptance.oracle_used_only_as_offline_ruler);
    assert!(result.acceptance.shuffled_eligibility_preserves_multiset);
    assert!(result.acceptance.stage_passed);
}

#[test]
fn invalid_m1cd_protocol_is_rejected() {
    let mut altered_online_rule = quick_config();
    altered_online_rule
        .m1ne_protocol
        .m1xe_protocol
        .m1x_protocol
        .m1_protocol
        .map2_protocol
        .exploration = 0.5;
    for config in [
        M1CDConfig {
            diagnostic_trial_count: 3,
            ..quick_config()
        },
        M1CDConfig {
            checkpoints: [0, 60, 60, 180, 240],
            ..quick_config()
        },
        M1CDConfig {
            minimum_sign_agreement: 1.1,
            ..quick_config()
        },
        altered_online_rule,
    ] {
        assert!(run_m1cd_experiment(config).is_err());
    }
}

#[test]
fn m1cd_decision_is_a_registered_diagnostic_outcome() {
    let result = run_m1cd_experiment(quick_config()).expect("M1-CD");
    assert!(matches!(
        result.decision,
        M1CDDecision::LocalCreditDirectionAdequate
            | M1CDDecision::AlignedButWeak
            | M1CDDecision::HomeostasisCancelsCredit
            | M1CDDecision::EligibilityUninformative
            | M1CDDecision::ConsequenceSignalUninformative
            | M1CDDecision::DirectionMostlyMisaligned
            | M1CDDecision::DynamicsUnstable
    ));
}

#[test]
fn formal_m1cd_result_matches_the_frozen_release_artifact() {
    let result = run_m1cd_experiment(M1CDConfig::default()).expect("formal M1-CD");
    assert_eq!(
        serde_json::to_string_pretty(&result.published()).expect("published M1-CD JSON"),
        include_str!("../app/public/credit-decomposition-v1.2.json")
    );
    assert_eq!(result.decision, M1CDDecision::EligibilityUninformative);
    assert!(result.acceptance.stage_passed);
    assert!(result.acceptance.passed);
    assert_eq!(result.confirmation_online_summary.independent_run_count, 48);
    assert_eq!(
        result.confirmation_online_summary.novel_final_behavior.mean,
        0.5781250000000001
    );
    let true_summary = result
        .confirmation_summaries
        .iter()
        .find(|row| row.control == M1CDControl::TrueConsequence)
        .expect("true consequence");
    assert_eq!(
        true_summary
            .component(neuro_engine::M1CDComponent::TotalUpdate)
            .cosine_alignment
            .mean,
        0.19069678723463832
    );
    assert_eq!(
        result
            .confirmation_effects
            .true_over_shuffled_total_cosine
            .mean,
        -0.020070802026743547
    );
}
