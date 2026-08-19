use neuro_engine::{
    GateEControllerKind, GateEExperimentConfig, HIDDEN_COUNT, run_gate_e_experiment,
};

fn quick_config() -> GateEExperimentConfig {
    GateEExperimentConfig {
        model_seed_count: 2,
        training_episodes: 80,
        evaluation_episodes: 16,
        curve_window: 20,
        ..GateEExperimentConfig::default()
    }
}

#[test]
fn gate_e_is_exactly_deterministic_and_contains_no_wall_clock_field() {
    let first = run_gate_e_experiment(quick_config()).expect("first Gate E run");
    let second = run_gate_e_experiment(quick_config()).expect("second Gate E run");
    assert_eq!(first, second);
    let json = serde_json::to_string(&first).expect("serialize Gate E");
    assert!(!json.contains("elapsed"));
    assert!(!json.contains("runtime"));
}

#[test]
fn invalid_lif_and_damage_parameters_are_rejected() {
    for config in [
        GateEExperimentConfig {
            lif_threshold_mv: 0.0,
            ..quick_config()
        },
        GateEExperimentConfig {
            lif_membrane_time_constant_ms: f64::NAN,
            ..quick_config()
        },
        GateEExperimentConfig {
            lif_spike_trace_decay: 1.0,
            ..quick_config()
        },
        GateEExperimentConfig {
            damage_fraction: 0.75,
            ..quick_config()
        },
    ] {
        assert!(run_gate_e_experiment(config).is_err());
    }
    let mut mismatched_decay = quick_config();
    mismatched_decay.controller.hidden_leak = 0.80;
    assert!(run_gate_e_experiment(mismatched_decay).is_err());
}

#[test]
fn trainable_and_fixed_connection_budgets_are_equal() {
    let result = run_gate_e_experiment(quick_config()).expect("Gate E run");
    assert_eq!(result.controller_budgets.len(), 3);
    for budget in &result.controller_budgets {
        assert_eq!(budget.sensor_count, 12);
        assert_eq!(budget.state_unit_count, 24);
        assert_eq!(budget.action_count, 4);
        assert_eq!(budget.fixed_input_weight_count, 288);
        assert_eq!(budget.trainable_action_weight_count, 96);
    }
    let continuous = result
        .controller_budgets
        .iter()
        .find(|budget| budget.controller == GateEControllerKind::ContinuousState)
        .expect("continuous budget");
    let lif = result
        .controller_budgets
        .iter()
        .find(|budget| budget.controller == GateEControllerKind::LifSpiking)
        .expect("LIF budget");
    assert_eq!(continuous.conceptual_dynamic_scalar_count, HIDDEN_COUNT * 2);
    assert_eq!(
        continuous.conceptual_state_bytes,
        lif.conceptual_state_bytes
    );
    assert!(lif.implementation_dynamic_state_bytes > continuous.implementation_dynamic_state_bytes);
}

#[test]
fn lif_trace_has_timestamped_spikes_and_eight_cue_free_steps() {
    let result = run_gate_e_experiment(quick_config()).expect("Gate E run");
    let trace = result
        .traces
        .iter()
        .find(|trace| trace.controller == GateEControllerKind::LifSpiking)
        .expect("LIF trace");
    assert_eq!(trace.delay_steps, 8);
    assert!(trace.frames[0].visible_cue.is_some());
    assert!(trace.frames[1].visible_cue.is_some());
    let decision = trace
        .frames
        .iter()
        .position(|frame| frame.branch_choice.is_some())
        .expect("branch decision");
    assert!(decision >= 10);
    assert!(
        trace.frames[2..10]
            .iter()
            .all(|frame| frame.visible_cue.is_none())
    );
    assert!(
        trace
            .frames
            .iter()
            .any(|frame| frame.spikes.iter().any(|spike| *spike))
    );
    assert!(trace.frames.iter().all(|frame| {
        frame.features.iter().all(|value| value.is_finite())
            && frame
                .membrane_potentials_mv
                .iter()
                .all(|value| value.is_finite())
    }));
}

#[test]
fn frozen_gate_e_protocol_passes_equal_budget_behavior_activity_and_damage_acceptance() {
    let result =
        run_gate_e_experiment(GateEExperimentConfig::default()).expect("frozen Gate E experiment");
    assert_eq!(result.version, "embodied-learning/v1.5-lif-comparison");
    assert_eq!(result.config.seed, 0x4741_5445_5f45_0201);
    assert_eq!(result.config.memory_delays, [4, 8, 12]);
    assert_eq!(result.config.lif_spike_trace_decay, 0.86);
    assert_eq!(result.config.controller.hidden_leak, 0.86);
    assert!(result.acceptance.budgets_matched);
    assert!(result.acceptance.lif_behavior_learnable);
    assert!(result.acceptance.lif_improves_at_least_one_axis);
    assert!(result.acceptance.lif_activity_valid_and_sparse);
    assert!(result.acceptance.damage_protocol_complete);
    assert!(result.acceptance.deterministic);
    assert!(result.acceptance.passed);

    let lif = result
        .reports
        .iter()
        .find(|report| {
            report.delay_steps == 8 && report.controller == GateEControllerKind::LifSpiking
        })
        .expect("LIF delay eight report");
    assert_eq!(lif.activity.finite_state_fraction.mean, 1.0);
    assert!(lif.activity.emitted_spikes_per_step.mean > 0.0);
    assert!(lif.activity.emitted_spikes_per_step.mean / HIDDEN_COUNT as f64 <= 0.50);
    assert!(lif.damaged_metrics.is_some());

    for (controller, cost) in &result.aggregate_costs {
        if *controller == GateEControllerKind::LifSpiking {
            assert_eq!(cost.input_events, cost.environment_steps * HIDDEN_COUNT);
            assert!(cost.emitted_spikes > 0);
        } else {
            assert_eq!(cost.input_events, 0);
            assert_eq!(cost.emitted_spikes, 0);
        }
    }
}
