use neuro_engine::{M1F_GAIN_COUNT, M1FConfig, M1FControl, M1FDecision, run_m1f_experiment};

fn quick_config() -> M1FConfig {
    let mut config = M1FConfig::default();
    config.m1_protocol.development_seed_count = 1;
    config.m1_protocol.confirmation_seed_count = 1;
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
fn m1f_is_exactly_deterministic() {
    let first = run_m1f_experiment(quick_config()).expect("first M1-F");
    let second = run_m1f_experiment(quick_config()).expect("second M1-F");
    assert_eq!(first, second);
}

#[test]
fn m1f_selects_gain_only_on_development_runs() {
    let result = run_m1f_experiment(quick_config()).expect("M1-F");
    assert_eq!(result.development_gain_summaries.len(), M1F_GAIN_COUNT);
    assert!(
        result
            .config
            .formation_gains
            .contains(&result.selected_formation_gain)
    );
    let best = result
        .development_gain_summaries
        .iter()
        .max_by(|left, right| {
            left.mean_novel_final_accuracy
                .mean
                .total_cmp(&right.mean_novel_final_accuracy.mean)
                .then_with(|| right.formation_gain.total_cmp(&left.formation_gain))
        })
        .expect("development winner");
    assert_eq!(best.formation_gain, result.selected_formation_gain);
    assert!(result.acceptance.development_gain_scan_complete);
    assert!(result.acceptance.selected_gain_from_development_only);
}

#[test]
fn m1f_keeps_candidate_local_and_every_budget_frozen() {
    let result = run_m1f_experiment(quick_config()).expect("M1-F");
    assert!(result.acceptance.candidate_observation_scope_local);
    assert!(
        result
            .acceptance
            .target_rule_and_readout_hidden_from_candidate
    );
    assert!(result.acceptance.paired_trial_and_perturbation_streams);
    assert!(result.acceptance.action_readout_remained_frozen);
    assert!(result.acceptance.topology_and_connection_budget_preserved);
    assert!(result.acceptance.finite_outputs);
    assert!(result.acceptance.stage_passed);
    assert!(result.acceptance.passed);
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
    for control in [
        M1FControl::RewardLocalBaseline,
        M1FControl::NodePerturbation,
        M1FControl::RandomConsequence,
        M1FControl::FrozenAdjustment,
    ] {
        assert_eq!(
            result
                .confirmation_summaries
                .iter()
                .filter(|row| row.control == control)
                .count(),
            1
        );
    }
    assert!(matches!(
        result.decision,
        M1FDecision::CandidateAccepted
            | M1FDecision::DynamicsUnstable
            | M1FDecision::ConsequenceIndependent
            | M1FDecision::HistoryOnlyFormation
            | M1FDecision::BehaviorWithoutFormationEvidence
            | M1FDecision::HistoryDegraded
            | M1FDecision::NoBehaviorBenefit
    ));
}

#[test]
fn invalid_m1f_protocol_is_rejected() {
    for config in [
        M1FConfig {
            formation_gains: [0.025, 0.05, 0.20, 0.10],
            ..quick_config()
        },
        M1FConfig {
            perturbation_scale: 0.0,
            ..quick_config()
        },
        M1FConfig {
            minimum_novel_accuracy: 1.1,
            ..quick_config()
        },
    ] {
        assert!(run_m1f_experiment(config).is_err());
    }
}

#[test]
fn published_m1f_result_is_frozen() {
    let result = run_m1f_experiment(M1FConfig::default()).expect("formal M1-F");
    assert_eq!(
        serde_json::to_string_pretty(&result.published()).expect("published M1-F JSON"),
        include_str!("../app/public/rule-formation-v0.8.json")
    );
    assert_eq!(result.selected_formation_gain, 0.20);
    assert_eq!(result.decision, M1FDecision::ConsequenceIndependent);
    assert!(!result.acceptance.mechanism_accepted);
    assert!(!result.acceptance.consequence_specificity_passed);
    assert!(!result.acceptance.novel_behavior_threshold_passed);
    assert!(!result.acceptance.target_probability_formation_passed);
    assert!(!result.acceptance.history_preserved);
    assert!(result.acceptance.stage_passed);
    assert!(result.acceptance.passed);

    let baseline = result
        .confirmation_summaries
        .iter()
        .find(|row| row.control == M1FControl::RewardLocalBaseline)
        .expect("M1 baseline");
    assert!((baseline.mean_history_final_accuracy.mean - 0.857638888888889).abs() <= 1e-12);
    assert!((baseline.mean_novel_final_accuracy.mean - 0.5533854166666667).abs() <= 1e-12);
}
