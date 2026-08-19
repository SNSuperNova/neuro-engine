use neuro_engine::{
    M1Rule, StructuralTimescaleConfig, StructuralTimescaleDecision,
    run_structural_timescale_diagnostic,
};

fn quick_config() -> StructuralTimescaleConfig {
    let mut config = StructuralTimescaleConfig::default();
    config.m2c_protocol.m1_protocol.development_seed_count = 1;
    config.m2c_protocol.m1_protocol.confirmation_seed_count = 2;
    config.forward_training_horizons = [0, 4, 8, 16];
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
fn timescale_diagnostic_is_exactly_deterministic() {
    let first = run_structural_timescale_diagnostic(quick_config()).expect("first M2C-T");
    let second = run_structural_timescale_diagnostic(quick_config()).expect("second M2C-T");
    assert_eq!(first, second);
}

#[test]
fn timescale_diagnostic_pairs_every_candidate_at_every_horizon() {
    let result = run_structural_timescale_diagnostic(quick_config()).expect("M2C-T");
    assert_eq!(result.confirmation_horizon_summaries.len(), 4);
    assert_eq!(result.confirmation_phase_summaries.len(), 20);
    assert_eq!(result.confirmation_horizon_comparisons.len(), 3);
    for row in &result.confirmation_seed_results {
        assert_eq!(row.checkpoints.len(), 18);
        assert_eq!(
            row.horizon_metrics.map(|metric| metric.training_horizon),
            [0, 4, 8, 16]
        );
        for checkpoint in &row.checkpoints {
            assert_eq!(checkpoint.candidate_count, 17);
            assert_eq!(checkpoint.candidates.len(), 17);
            assert_eq!(
                checkpoint
                    .horizon_metrics
                    .map(|metric| metric.training_horizon),
                [0, 4, 8, 16]
            );
            assert_eq!(
                checkpoint
                    .horizon_metrics
                    .map(|metric| metric.selected_source_unit),
                [checkpoint.horizon_metrics[0].selected_source_unit; 4]
            );
            for horizon_index in 0..4 {
                let metric = checkpoint.horizon_metrics[horizon_index];
                assert!(metric.selected_regret >= -1e-12);
                assert!(
                    checkpoint
                        .candidates
                        .iter()
                        .all(|candidate| metric.oracle_benefit + 1e-12
                            >= candidate.benefit_over_no_swap[horizon_index])
                );
            }
        }
        assert_eq!(
            row.checkpoints
                .iter()
                .filter(|checkpoint| checkpoint.rule == M1Rule::A)
                .count(),
            6
        );
    }
    assert!(
        result
            .acceptance
            .all_legal_candidates_enumerated_at_every_horizon
    );
    assert!(
        result
            .acceptance
            .same_checkpoint_clones_used_across_horizons
    );
    assert!(result.acceptance.cumulative_training_prefix_frozen);
    assert!(result.acceptance.evaluation_probes_frozen_across_horizons);
}

#[test]
fn timescale_diagnostic_preserves_live_stream_and_completes_protocol() {
    let result = run_structural_timescale_diagnostic(quick_config()).expect("M2C-T");
    assert!(result.acceptance.diagnostic_is_offline_only);
    assert!(result.acceptance.topology_unchanged_in_main_stream);
    assert!(result.acceptance.action_readout_remained_frozen);
    assert!(result.acceptance.stage_passed);
    assert!(result.acceptance.passed);
    assert!(matches!(
        result.decision,
        StructuralTimescaleDecision::DelayedStructuralEffect
            | StructuralTimescaleDecision::CreditSignalInsufficient
            | StructuralTimescaleDecision::RecoveryOnlySignal
            | StructuralTimescaleDecision::SingleEdgeFreedomInsufficient
            | StructuralTimescaleDecision::DiagnosticUnstable
    ));
}

#[test]
fn invalid_timescale_protocol_is_rejected() {
    for config in [
        StructuralTimescaleConfig {
            forward_training_horizons: [1, 4, 8, 16],
            ..quick_config()
        },
        StructuralTimescaleConfig {
            forward_training_horizons: [0, 8, 4, 16],
            ..quick_config()
        },
        StructuralTimescaleConfig {
            forward_training_horizons: [0, 4, 4, 16],
            ..quick_config()
        },
        StructuralTimescaleConfig {
            evaluation_trial_count: 0,
            ..quick_config()
        },
    ] {
        assert!(run_structural_timescale_diagnostic(config).is_err());
    }
}

#[test]
fn published_m2c_timescale_diagnostic_is_frozen() {
    let result = run_structural_timescale_diagnostic(StructuralTimescaleConfig::default())
        .expect("formal M2C-T");
    assert_eq!(
        serde_json::to_string_pretty(&result).expect("M2C-T JSON"),
        include_str!("../app/public/structural-timescale-v0.5.json")
    );
    assert_eq!(
        result
            .confirmation_horizon_summaries
            .iter()
            .map(|row| row.training_horizon)
            .collect::<Vec<_>>(),
        vec![0, 32, 128, 256]
    );
    assert_eq!(
        result.confirmation_horizon_summaries[0].checkpoint_count,
        864
    );
    assert_eq!(
        result.confirmation_horizon_summaries[0].candidate_evaluation_count,
        14688
    );
    assert!(result.acceptance.passed);
    assert!(result.acceptance.horizon_32_replicates_v04);

    let old: serde_json::Value = serde_json::from_str(include_str!(
        "../app/public/structural-diagnostic-v0.4.json"
    ))
    .expect("v0.4 JSON");
    let new = serde_json::to_value(&result).expect("M2C-T value");
    for split in ["developmentSeedResults", "confirmationSeedResults"] {
        let old_runs = old[split].as_array().expect("v0.4 runs");
        let new_runs = new[split].as_array().expect("M2C-T runs");
        for (old_run, new_run) in old_runs.iter().zip(new_runs) {
            for (old_checkpoint, new_checkpoint) in old_run["checkpoints"]
                .as_array()
                .expect("v0.4 checkpoints")
                .iter()
                .zip(
                    new_run["checkpoints"]
                        .as_array()
                        .expect("M2C-T checkpoints"),
                )
            {
                assert_eq!(
                    old_checkpoint["noSwapPostHorizonAccuracy"],
                    new_checkpoint["noSwapAccuracies"][1]
                );
                for (old_candidate, new_candidate) in old_checkpoint["candidates"]
                    .as_array()
                    .expect("v0.4 candidates")
                    .iter()
                    .zip(
                        new_checkpoint["candidates"]
                            .as_array()
                            .expect("M2C-T candidates"),
                    )
                {
                    assert_eq!(old_candidate["sourceUnit"], new_candidate["sourceUnit"]);
                    let old_benefit = old_candidate["benefitOverNoSwap"]
                        .as_f64()
                        .expect("v0.4 benefit");
                    let new_benefit = new_candidate["benefitOverNoSwap"][1]
                        .as_f64()
                        .expect("M2C-T benefit");
                    assert!((old_benefit - new_benefit).abs() <= f64::EPSILON);
                }
            }
        }
    }
}
