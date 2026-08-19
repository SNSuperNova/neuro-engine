use neuro_engine::{GateDControllerKind, GateDExperimentConfig, run_gate_d_experiment};

fn quick_config() -> GateDExperimentConfig {
    GateDExperimentConfig {
        model_seed_count: 2,
        training_episodes: 80,
        evaluation_episodes: 16,
        curve_window: 20,
        ..GateDExperimentConfig::default()
    }
}

#[test]
fn gate_d_report_is_exactly_deterministic_and_excludes_wall_clock_time() {
    let first = run_gate_d_experiment(quick_config()).expect("first Gate D run");
    let second = run_gate_d_experiment(quick_config()).expect("second Gate D run");
    assert_eq!(first, second);
    assert_eq!(first.reports.len(), 12);
    assert_eq!(first.traces.len(), 3);
    let json = serde_json::to_string(&first).expect("serialize Gate D");
    assert!(!json.contains("runtimeMilliseconds"));
    assert!(!json.contains("elapsed"));
}

#[test]
fn invalid_dynamics_and_controller_parameters_are_rejected() {
    let mut invalid_radius = quick_config();
    invalid_radius.recurrent_spectral_radius = 1.0;
    assert!(run_gate_d_experiment(invalid_radius).is_err());

    let mut invalid_controller = quick_config();
    invalid_controller.controller.adaptation_strength = f64::NAN;
    assert!(run_gate_d_experiment(invalid_controller).is_err());
}

#[test]
fn recurrent_controls_match_weight_budget_multiset_and_spectrum() {
    let result = run_gate_d_experiment(quick_config()).expect("Gate D run");
    let none = result
        .controller_budgets
        .iter()
        .find(|budget| budget.controller == GateDControllerKind::NoRecurrence)
        .expect("no recurrence budget");
    let shuffled = result
        .controller_budgets
        .iter()
        .find(|budget| budget.controller == GateDControllerKind::ShuffledRecurrence)
        .expect("shuffled budget");
    let structured = result
        .controller_budgets
        .iter()
        .find(|budget| budget.controller == GateDControllerKind::StructuredRecurrence)
        .expect("structured budget");

    for budget in &result.controller_budgets {
        assert_eq!(budget.sensor_count, 12);
        assert_eq!(budget.state_unit_count, 24);
        assert_eq!(budget.action_count, 4);
        assert_eq!(budget.fixed_input_weight_count, 12 * 24);
        assert_eq!(budget.trainable_action_weight_count, 24 * 4);
        assert_eq!(budget.allocated_recurrent_weight_count, 24 * 24);
    }
    assert_eq!(none.active_recurrent_weight_count, 0);
    assert_eq!(shuffled.active_recurrent_weight_count, 24 * 6);
    assert_eq!(
        shuffled.active_recurrent_weight_count,
        structured.active_recurrent_weight_count
    );
    assert_eq!(
        shuffled.excitatory_edge_count,
        structured.excitatory_edge_count
    );
    assert_eq!(
        shuffled.inhibitory_edge_count,
        structured.inhibitory_edge_count
    );
    assert_eq!(
        shuffled.recurrent_weight_digest,
        structured.recurrent_weight_digest
    );
    assert!((shuffled.estimated_spectral_radius - 0.15).abs() <= 0.03);
    assert!(
        (shuffled.estimated_spectral_radius - structured.estimated_spectral_radius).abs() <= 1e-9
    );
}

#[test]
fn delay_eight_traces_show_two_cue_steps_then_eight_cue_free_steps() {
    let result = run_gate_d_experiment(quick_config()).expect("Gate D run");
    for trace in &result.traces {
        assert_eq!(trace.delay_steps, 8);
        assert_eq!(trace.frames[0].visible_cue, Some(trace.presented_cue));
        assert_eq!(trace.frames[1].visible_cue, Some(trace.presented_cue));
        let decision = trace
            .frames
            .iter()
            .position(|frame| frame.branch_choice.is_some())
            .expect("branch decision");
        assert!(decision >= 10, "decision frame was {decision}");
        assert!(
            trace.frames[2..decision]
                .iter()
                .all(|frame| frame.visible_cue.is_none())
        );
        assert_eq!(trace.frames[2].delay_steps_remaining, 8);
        assert_eq!(trace.frames[9].delay_steps_remaining, 1);
    }
}

#[test]
fn frozen_gate_d_protocol_passes_behavior_control_and_stability_acceptance() {
    let result =
        run_gate_d_experiment(GateDExperimentConfig::default()).expect("frozen Gate D experiment");
    assert_eq!(result.version, "embodied-learning/v1.4-recurrent-dynamics");
    assert_eq!(result.config.seed, 0x4741_5445_5f44_0201);
    assert_eq!(result.config.memory_delays, [4, 6, 8, 12]);
    assert_eq!(result.config.recurrent_in_degree, 6);
    assert_eq!(result.config.recurrent_spectral_radius, 0.15);
    assert!(result.acceptance.structured_beats_shuffled_at_delay_eight);
    assert!(result.acceptance.structured_delay_eight_learnable);
    assert!(result.acceptance.structured_extends_reliable_boundary);
    assert!(result.acceptance.shuffled_does_not_match_primary_gain);
    assert!(result.acceptance.structured_state_stable);
    assert!(result.acceptance.budgets_and_recurrent_controls_matched);
    assert!(result.acceptance.deterministic);
    assert!(result.acceptance.passed);

    let structured = result
        .reports
        .iter()
        .find(|report| {
            report.delay_steps == 8
                && report.controller == GateDControllerKind::StructuredRecurrence
        })
        .expect("structured delay eight report");
    assert_eq!(structured.stability.finite_state_fraction.mean, 1.0);
    assert!((0.05..=0.90).contains(&structured.stability.mean_absolute_activity.mean));
    assert!(structured.stability.saturated_unit_fraction.mean <= 0.25);
    assert!(structured.stability.silent_unit_fraction.mean <= 0.60);
    assert!(structured.stability.population_synchrony.mean <= 0.90);
}
