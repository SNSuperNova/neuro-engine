use neuro_engine::{GateFControllerKind, GateFExperimentConfig, GateFPhase, run_gate_f_experiment};

fn quick_config() -> GateFExperimentConfig {
    GateFExperimentConfig {
        model_seed_count: 2,
        pretraining_episodes: 80,
        adaptation_episodes: 40,
        evaluation_episodes: 20,
        checkpoints: [0, 5, 10, 20, 40],
        ..GateFExperimentConfig::default()
    }
}

#[test]
fn gate_f_is_exactly_deterministic() {
    let first = run_gate_f_experiment(quick_config()).expect("first Gate F run");
    let second = run_gate_f_experiment(quick_config()).expect("second Gate F run");
    assert_eq!(first, second);
}

#[test]
fn invalid_protocol_is_rejected() {
    for config in [
        GateFExperimentConfig {
            internal_learning_rate: 0.0,
            ..quick_config()
        },
        GateFExperimentConfig {
            reward_baseline_decay: 1.0,
            ..quick_config()
        },
        GateFExperimentConfig {
            homeostasis_strength: 1.1,
            ..quick_config()
        },
        GateFExperimentConfig {
            checkpoints: [0, 10, 5, 20, 40],
            ..quick_config()
        },
    ] {
        assert!(run_gate_f_experiment(config).is_err());
    }
}

#[test]
fn plastic_budgets_are_explicit_and_readout_stays_frozen() {
    let result = run_gate_f_experiment(quick_config()).expect("Gate F run");
    assert_eq!(result.controller_budgets.len(), 4);
    assert!(result.controller_budgets.iter().all(|budget| {
        budget.sensor_count == 12
            && budget.state_unit_count == 24
            && budget.action_count == 2
            && budget.allocated_input_weight_count == 288
            && budget.allocated_recurrent_weight_count == 576
            && budget.active_recurrent_weight_count == 144
            && budget.strong_plastic_candidate_count == 48
            && budget.weak_fixed_recurrent_count == 96
            && budget.allocated_internal_plastic_slot_count == 48
            && budget.frozen_action_weight_count == 48
    }));
    for metric in &result.seed_metrics {
        let initial = result
            .seed_metrics
            .iter()
            .find(|candidate| {
                candidate.seed == metric.seed
                    && candidate.controller == metric.controller
                    && candidate.phase == GateFPhase::BeforeChange
            })
            .expect("initial metric");
        assert_eq!(metric.action_weight_digest, initial.action_weight_digest);
        assert_eq!(metric.weights.finite_weight_fraction, 1.0);
    }
}

#[test]
fn traces_cover_each_controller_and_the_cue_free_delay() {
    let result = run_gate_f_experiment(quick_config()).expect("Gate F run");
    for kind in GateFControllerKind::ALL {
        let trace = result
            .traces
            .iter()
            .find(|trace| trace.controller == kind)
            .expect("controller trace");
        assert_eq!(
            trace.frames.len(),
            quick_config().cue_steps + quick_config().delay_steps
        );
        assert!(trace.frames[..2].iter().all(|frame| frame.cue_visible));
        assert!(trace.frames[2..].iter().all(|frame| !frame.cue_visible));
        assert!(
            trace
                .frames
                .iter()
                .flat_map(|frame| frame.hidden_activity)
                .all(f64::is_finite)
        );
    }
}

#[test]
fn frozen_gate_f_protocol_passes() {
    let result =
        run_gate_f_experiment(GateFExperimentConfig::default()).expect("frozen Gate F experiment");
    assert_eq!(result.version, "embodied-learning/v1.6-internal-plasticity");
    assert_eq!(result.task, "reversal-cue-fork/v1");
    assert!(result.acceptance.matched_start_and_plastic_budgets);
    assert!(result.acceptance.original_rule_learned);
    assert!(result.acceptance.sensory_plasticity_adapts);
    assert!(result.acceptance.recurrent_plasticity_adapts);
    assert!(result.acceptance.restored_rule_relearned);
    assert!(result.acceptance.readout_frozen_and_states_finite);
    assert!(result.acceptance.homeostasis_control_complete);
    assert!(result.acceptance.deterministic);
    assert!(result.acceptance.passed);
}
