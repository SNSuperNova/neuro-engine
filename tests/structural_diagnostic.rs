use neuro_engine::{
    M1Rule, StructuralDiagnosticConfig, StructuralDiagnosticDecision, run_structural_diagnostic,
};

fn quick_config() -> StructuralDiagnosticConfig {
    let mut config = StructuralDiagnosticConfig::default();
    config.m2c_protocol.m1_protocol.development_seed_count = 1;
    config.m2c_protocol.m1_protocol.confirmation_seed_count = 2;
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
fn structural_diagnostic_is_exactly_deterministic() {
    let first = run_structural_diagnostic(quick_config()).expect("first diagnostic");
    let second = run_structural_diagnostic(quick_config()).expect("second diagnostic");
    assert_eq!(first, second);
}

#[test]
fn diagnostic_never_mutates_the_live_topology_or_readout() {
    let result = run_structural_diagnostic(quick_config()).expect("diagnostic");
    assert!(result.acceptance.diagnostic_is_offline_only);
    assert!(result.acceptance.topology_unchanged_in_main_stream);
    assert!(result.acceptance.action_readout_remained_frozen);
    assert!(
        result
            .confirmation_seed_results
            .iter()
            .all(
                |row| row.topology_digest_before == row.topology_digest_after
                    && row.action_readout_digest_before == row.action_readout_digest_after
            )
    );
}

#[test]
fn diagnostic_enumerates_every_legal_candidate_at_all_rewire_checkpoints() {
    let result = run_structural_diagnostic(quick_config()).expect("diagnostic");
    for row in &result.confirmation_seed_results {
        assert_eq!(row.checkpoints.len(), 18);
        assert_eq!(
            row.checkpoints
                .iter()
                .filter(|checkpoint| checkpoint.rule == M1Rule::A)
                .count(),
            6
        );
        assert_eq!(
            [M1Rule::B, M1Rule::C, M1Rule::D].map(|rule| row
                .checkpoints
                .iter()
                .filter(|checkpoint| checkpoint.rule == rule)
                .count()),
            [4, 4, 4]
        );
        assert!(row.checkpoints.iter().all(|checkpoint| {
            checkpoint.candidate_count == 17
                && checkpoint.candidates.len() == 17
                && checkpoint.selected_regret >= -1e-12
                && checkpoint.candidates.iter().all(|candidate| {
                    checkpoint.oracle_benefit + 1e-12 >= candidate.benefit_over_no_swap
                })
        }));
    }
    assert!(result.acceptance.all_checkpoints_complete);
    assert!(result.acceptance.all_legal_candidates_enumerated);
}

#[test]
fn diagnostic_reports_a_complete_causal_interpretation() {
    let result = run_structural_diagnostic(quick_config()).expect("diagnostic");
    assert!(result.acceptance.shuffled_ranking_controls_complete);
    assert!(result.acceptance.paired_horizons_and_randomness_frozen);
    assert!(result.acceptance.stage_passed);
    assert!(result.acceptance.passed);
    assert!(matches!(
        result.decision,
        StructuralDiagnosticDecision::EvidenceInformative
            | StructuralDiagnosticDecision::EvidenceRankingWeak
            | StructuralDiagnosticDecision::EvidenceNotInformative
            | StructuralDiagnosticDecision::CounterfactualEffectsFlat
            | StructuralDiagnosticDecision::DiagnosticUnstable
    ));
}

#[test]
fn invalid_structural_diagnostic_protocol_is_rejected() {
    for config in [
        StructuralDiagnosticConfig {
            forward_training_trials: 0,
            ..quick_config()
        },
        StructuralDiagnosticConfig {
            checkpoint_global_trials: [
                63, 128, 192, 256, 320, 384, 448, 512, 576, 640, 704, 768, 832, 896, 960, 1024,
                1088, 1152,
            ],
            ..quick_config()
        },
        StructuralDiagnosticConfig {
            shuffled_ranking_count: 0,
            ..quick_config()
        },
    ] {
        assert!(run_structural_diagnostic(config).is_err());
    }
}

#[test]
fn published_offline_structural_diagnostic_is_frozen() {
    let result = run_structural_diagnostic(StructuralDiagnosticConfig::default())
        .expect("formal structural diagnostic");
    assert_eq!(
        serde_json::to_string_pretty(&result).expect("diagnostic JSON"),
        include_str!("../app/public/structural-diagnostic-v0.4.json")
    );
    assert_eq!(
        result.decision,
        StructuralDiagnosticDecision::CounterfactualEffectsFlat
    );
    assert!(!result.acceptance.useful_counterfactual_swaps_exist);
    assert!(!result.acceptance.evidence_informative);
    assert!(result.acceptance.topology_unchanged_in_main_stream);
    assert!(result.acceptance.stage_passed);
    assert!(result.acceptance.passed);
    assert_eq!(result.confirmation_summary.checkpoint_count, 864);
    assert_eq!(
        result.confirmation_summary.candidate_evaluation_count,
        14688
    );
    assert!(
        result.confirmation_phase_summaries[4]
            .mean_spearman_correlation
            .lower95
            > 0.0
    );
}
