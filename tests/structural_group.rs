use neuro_engine::{
    StructuralGroupConfig, StructuralGroupDecision, run_structural_group_diagnostic,
};

fn quick_config() -> StructuralGroupConfig {
    let mut config = StructuralGroupConfig::default();
    config.m2c_protocol.m1_protocol.development_seed_count = 1;
    config.m2c_protocol.m1_protocol.confirmation_seed_count = 1;
    config.forward_training_trials = 8;
    config.evaluation_trial_count = 16;
    config.shuffled_ranking_count = 4;
    config
        .m2c_protocol
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
fn grouped_diagnostic_is_exactly_deterministic() {
    let first = run_structural_group_diagnostic(quick_config()).expect("first M2C-G");
    let second = run_structural_group_diagnostic(quick_config()).expect("second M2C-G");
    assert_eq!(first, second);
}

#[test]
fn grouped_diagnostic_balances_every_dimension_under_equal_search_budget() {
    let result = run_structural_group_diagnostic(quick_config()).expect("M2C-G");
    for row in &result.confirmation_seed_results {
        assert_eq!(row.checkpoints.len(), 18);
        for checkpoint in &row.checkpoints {
            assert_eq!(checkpoint.scales.len(), 3);
            for (scale, edge_count) in checkpoint.scales.iter().zip([1, 2, 4]) {
                assert_eq!(scale.edge_count, edge_count);
                assert_eq!(scale.target_units.len(), edge_count);
                assert_eq!(scale.target_evidence_covered.len(), edge_count);
                assert_eq!(scale.replaced_slots.len(), edge_count);
                assert_eq!(scale.bundle_count, 17);
                assert_eq!(scale.candidates.len(), 17);
                assert_eq!(scale.component_control_evaluation_count, edge_count * 17);
                assert!(scale.balanced_candidate_marginals);
                assert!(scale.clone_connection_budget_preserved);
                assert!(scale.selected_regret >= -1e-12);
                assert!(scale.candidates.iter().all(|candidate| {
                    candidate.source_units.len() == edge_count
                        && scale.budgeted_oracle_benefit + 1e-12 >= candidate.benefit_over_no_swap
                }));
                for dimension in 0..edge_count {
                    let mut sources = scale
                        .candidates
                        .iter()
                        .map(|candidate| candidate.source_units[dimension])
                        .collect::<Vec<_>>();
                    sources.sort_unstable();
                    sources.dedup();
                    assert_eq!(sources.len(), 17);
                }
            }
        }
    }
    assert!(result.acceptance.equal_bundle_budget_across_group_sizes);
    assert!(result.acceptance.balanced_candidate_marginals_complete);
    assert!(
        result
            .acceptance
            .grouped_target_evidence_coverage_sufficient
    );
    assert!(result.acceptance.component_controls_complete);
}

#[test]
fn grouped_diagnostic_is_offline_and_preserves_every_budget() {
    let result = run_structural_group_diagnostic(quick_config()).expect("M2C-G");
    assert!(result.acceptance.diagnostic_is_offline_only);
    assert!(result.acceptance.topology_unchanged_in_main_stream);
    assert!(result.acceptance.action_readout_remained_frozen);
    assert!(result.acceptance.clone_connection_budget_preserved);
    assert!(result.acceptance.stage_passed);
    assert!(result.acceptance.passed);
    assert!(matches!(
        result.decision,
        StructuralGroupDecision::GroupedFreedomAndCredit
            | StructuralGroupDecision::GroupedFreedomWithoutCredit
            | StructuralGroupDecision::HistoryDominatedGroupedEffect
            | StructuralGroupDecision::GroupedEffectNotBeyondSingle
            | StructuralGroupDecision::NoUsefulGroupedEffect
            | StructuralGroupDecision::DiagnosticUnstable
    ));
}

#[test]
fn invalid_grouped_protocol_is_rejected() {
    for config in [
        StructuralGroupConfig {
            group_sizes: [1, 3, 4],
            ..quick_config()
        },
        StructuralGroupConfig {
            bundle_budget: 16,
            ..quick_config()
        },
        StructuralGroupConfig {
            forward_training_trials: 0,
            ..quick_config()
        },
        StructuralGroupConfig {
            evaluation_trial_count: 0,
            ..quick_config()
        },
    ] {
        assert!(run_structural_group_diagnostic(config).is_err());
    }
}

#[test]
fn published_m2c_grouped_diagnostic_is_frozen() {
    let result =
        run_structural_group_diagnostic(StructuralGroupConfig::default()).expect("formal M2C-G");
    assert_eq!(
        serde_json::to_string_pretty(&result.published()).expect("published M2C-G JSON"),
        include_str!("../app/public/structural-group-v0.6.json")
    );
    assert!(result.acceptance.single_edge_scale_replicates_v04);
    assert!(result.acceptance.stage_passed);
    assert!(result.acceptance.passed);
    assert_eq!(
        result
            .confirmation_summaries
            .iter()
            .map(|row| row.edge_count)
            .collect::<Vec<_>>(),
        vec![1, 2, 4]
    );
    assert!(
        result
            .confirmation_summaries
            .iter()
            .all(|row| row.bundle_evaluation_count == 14_688)
    );

    let old: serde_json::Value = serde_json::from_str(include_str!(
        "../app/public/structural-diagnostic-v0.4.json"
    ))
    .expect("v0.4 JSON");
    let new = serde_json::to_value(&result).expect("M2C-G value");
    for split in ["developmentSeedResults", "confirmationSeedResults"] {
        for (old_run, new_run) in old[split]
            .as_array()
            .expect("v0.4 runs")
            .iter()
            .zip(new[split].as_array().expect("M2C-G runs"))
        {
            for (old_checkpoint, new_checkpoint) in old_run["checkpoints"]
                .as_array()
                .expect("v0.4 checkpoints")
                .iter()
                .zip(
                    new_run["checkpoints"]
                        .as_array()
                        .expect("M2C-G checkpoints"),
                )
            {
                let single = &new_checkpoint["scales"][0];
                assert_eq!(
                    old_checkpoint["noSwapPostHorizonAccuracy"],
                    single["noSwapPostHorizonAccuracy"]
                );
                for (old_candidate, new_candidate) in old_checkpoint["candidates"]
                    .as_array()
                    .expect("v0.4 candidates")
                    .iter()
                    .zip(single["candidates"].as_array().expect("M2C-G candidates"))
                {
                    assert_eq!(old_candidate["sourceUnit"], new_candidate["sourceUnits"][0]);
                    let old_benefit = old_candidate["benefitOverNoSwap"]
                        .as_f64()
                        .expect("v0.4 benefit");
                    let new_benefit = new_candidate["benefitOverNoSwap"]
                        .as_f64()
                        .expect("M2C-G benefit");
                    assert!((old_benefit - new_benefit).abs() <= f64::EPSILON);
                }
            }
        }
    }
}
