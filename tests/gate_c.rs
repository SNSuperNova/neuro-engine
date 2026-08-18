use neuro_engine::{GateCExperimentConfig, GateCPhase, run_gate_c_experiment};

fn quick_config() -> GateCExperimentConfig {
    GateCExperimentConfig {
        model_seed_count: 2,
        training_episodes: 40,
        evaluation_episodes: 8,
        curve_window: 10,
        ..GateCExperimentConfig::default()
    }
}

#[test]
fn gate_c_report_is_exactly_deterministic_and_equal_budget() {
    let first = run_gate_c_experiment(quick_config()).expect("first deterministic run");
    let second = run_gate_c_experiment(quick_config()).expect("second deterministic run");
    assert_eq!(first, second);
    assert_eq!(first.reports.len(), 17);
    assert_eq!(first.controller_budget.sensor_count, 12);
    assert_eq!(first.controller_budget.feature_count, 24);
    assert_eq!(first.controller_budget.action_count, 4);
    assert_eq!(first.controller_budget.fixed_input_weight_count, 24 * 12);
    assert_eq!(
        first.controller_budget.trainable_action_weight_count,
        24 * 4
    );
    let json = serde_json::to_string(&first).expect("serialize report");
    assert!(!json.contains("runtime"));
    assert!(!json.contains("elapsed"));
}

#[test]
fn delayed_trace_hides_cue_and_delivers_real_energy_after_eight_waiting_steps() {
    let result = run_gate_c_experiment(GateCExperimentConfig::default()).expect("full Gate C run");
    let trace = result
        .traces
        .iter()
        .find(|trace| {
            trace.reward_delay_steps == 8
                && trace.eligibility_decay == 0.88
                && !trace.cue_randomized
        })
        .expect("current trace at delay eight");
    let choice_index = trace
        .frames
        .iter()
        .position(|frame| frame.branch_choice.is_some())
        .expect("branch choice");
    let choice_frame = &trace.frames[choice_index];
    assert_eq!(choice_frame.phase_before, GateCPhase::Junction);
    assert_eq!(choice_frame.visible_cue, Some(trace.presented_cue));
    assert_eq!(trace.frames.len() - choice_index - 1, 8);
    assert!(
        trace.frames[choice_index + 1..]
            .iter()
            .all(|frame| frame.phase_before == GateCPhase::Waiting && frame.visible_cue.is_none())
    );
    assert!(trace.summary.correct_choice);
    assert!(trace.summary.energy_paid_out);
    assert!(trace.frames.last().expect("outcome frame").energy_paid_out);
    assert!(trace.frames.last().expect("outcome frame").reward > 0.0);
    let energy_delta =
        (trace.summary.final_energy - result.config.initial_energy) / result.config.food_energy;
    assert!((energy_delta - trace.summary.total_reward).abs() < 1e-12);
}

#[test]
fn frozen_gate_c_protocol_passes_every_control() {
    let result =
        run_gate_c_experiment(GateCExperimentConfig::default()).expect("frozen Gate C experiment");
    assert_eq!(result.version, "embodied-learning/v1.3-credit-assignment");
    assert!(result.acceptance.immediate_reward_learnable);
    assert!(result.acceptance.current_trace_beats_zero_at_delay_eight);
    assert!(result.acceptance.current_trace_reaches_seventy_percent);
    assert!(result.acceptance.zero_trace_degrades_with_delay);
    assert!(result.acceptance.randomized_control_at_chance);
    assert!(result.acceptance.left_right_and_branch_consistent);
    assert!(result.acceptance.deterministic);
    assert!(result.acceptance.passed);

    let report = |delay, decay, randomized| {
        result
            .reports
            .iter()
            .find(|report| {
                report.reward_delay_steps == delay
                    && report.eligibility_decay == decay
                    && report.cue_randomized == randomized
            })
            .expect("canonical report")
    };
    for decay in [0.0, 0.50, 0.88, 0.97] {
        assert!(report(0, decay, false).metrics.correct_choice_fraction.mean >= 0.80);
    }
    let current = report(8, 0.88, false);
    assert!(current.metrics.correct_choice_fraction.mean >= 0.70);
    assert!(current.metrics.branch_choice_fraction.mean >= 0.95);
    assert!(current.metrics.left_target_accuracy.mean >= 0.70);
    assert!(current.metrics.right_target_accuracy.mean >= 0.70);
    let randomized = report(4, 0.88, true).metrics.correct_choice_fraction;
    assert!(randomized.lower95 <= 0.50 && randomized.upper95 >= 0.50);

    let primary = result
        .paired_effects
        .iter()
        .find(|effect| effect.id == "delay-8-current-vs-zero")
        .expect("primary effect")
        .effect
        .correct_choice_fraction;
    assert!(primary.mean >= 0.15);
    assert!(primary.lower95 > 0.0);
    let zero_degradation = result
        .paired_effects
        .iter()
        .find(|effect| effect.id == "zero-trace-immediate-vs-delay-8")
        .expect("zero trace delay effect")
        .effect
        .correct_choice_fraction;
    assert!(zero_degradation.lower95 > 0.0);
}
