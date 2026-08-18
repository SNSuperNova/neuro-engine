use neuro_engine::{
    GateBCondition, GateBControllerKind, GateBExperimentConfig, run_gate_b_experiment,
};

fn quick_config() -> GateBExperimentConfig {
    GateBExperimentConfig {
        model_seed_count: 2,
        training_episodes: 40,
        evaluation_episodes: 8,
        curve_window: 10,
        ..GateBExperimentConfig::default()
    }
}

#[test]
fn report_is_exactly_deterministic_and_excludes_wall_clock_time() {
    let first = run_gate_b_experiment(quick_config()).expect("first deterministic run");
    let second = run_gate_b_experiment(quick_config()).expect("second deterministic run");
    assert_eq!(first, second);

    let json = serde_json::to_string(&first).expect("serialize report");
    assert!(!json.contains("runtimeMilliseconds"));
    assert!(!json.contains("elapsed"));
}

#[test]
fn all_controllers_have_the_same_observations_actions_and_trainable_budget() {
    let result = run_gate_b_experiment(quick_config()).expect("Gate B run");
    assert_eq!(result.controller_budgets.len(), 3);
    let reference = result.controller_budgets[0];
    assert_eq!(reference.sensor_count, 12);
    assert_eq!(reference.state_unit_count, 24);
    assert_eq!(reference.action_count, 4);
    assert_eq!(reference.fixed_input_weight_count, 24 * 12);
    assert_eq!(reference.trainable_action_weight_count, 24 * 4);
    for budget in &result.controller_budgets[1..] {
        assert_eq!(budget.sensor_count, reference.sensor_count);
        assert_eq!(budget.state_unit_count, reference.state_unit_count);
        assert_eq!(budget.action_count, reference.action_count);
        assert_eq!(
            budget.fixed_input_weight_count,
            reference.fixed_input_weight_count
        );
        assert_eq!(
            budget.trainable_action_weight_count,
            reference.trainable_action_weight_count
        );
    }
}

#[test]
fn delayed_trace_has_two_cue_steps_then_four_cue_free_steps_before_choice() {
    let result = run_gate_b_experiment(quick_config()).expect("Gate B run");
    let trace = result
        .traces
        .iter()
        .find(|trace| {
            trace.condition == GateBCondition::DelayedCue
                && trace.controller == GateBControllerKind::LeakyState
        })
        .expect("delayed leaky trace");

    assert_eq!(trace.frames[0].visible_cue, Some(trace.presented_cue));
    assert_eq!(trace.frames[1].visible_cue, Some(trace.presented_cue));
    let decision = trace
        .frames
        .iter()
        .position(|frame| frame.branch_choice.is_some())
        .expect("branch decision");
    assert!(decision >= 6, "decision frame was {decision}");
    assert_eq!(trace.frames[decision].position_before.x, 4);
    assert_eq!(trace.frames[decision].position_before.y, 2);
    assert!(
        trace.frames[2..=decision]
            .iter()
            .all(|frame| frame.visible_cue.is_none())
    );
    assert_eq!(trace.frames[0].position.y, 7);
    assert_eq!(trace.frames[5].position.y, 2);
}

#[test]
fn visible_control_repeats_the_cue_on_the_branch_decision() {
    let result = run_gate_b_experiment(quick_config()).expect("Gate B run");
    let trace = result
        .traces
        .iter()
        .find(|trace| trace.condition == GateBCondition::CueVisibleAtFork)
        .expect("visible-control trace");
    let decision = trace
        .frames
        .iter()
        .find(|frame| frame.branch_choice.is_some())
        .expect("branch decision");
    assert_eq!(decision.visible_cue, Some(trace.presented_cue));
}

#[test]
fn frozen_full_protocol_passes_every_gate_b_control() {
    let result =
        run_gate_b_experiment(GateBExperimentConfig::default()).expect("frozen Gate B experiment");
    assert_eq!(result.version, "embodied-learning/v1.2-state-necessity");
    assert_eq!(result.config.model_seed_count, 12);
    assert_eq!(result.config.training_episodes, 1_200);
    assert_eq!(result.config.evaluation_episodes, 200);
    assert!(result.acceptance.visible_control_learnable);
    assert!(result.acceptance.randomized_control_at_chance);
    assert!(result.acceptance.leaky_beats_state_reset);
    assert!(result.acceptance.leaky_reaches_seventy_percent);
    assert!(result.acceptance.history_shuffle_hurts);
    assert!(result.acceptance.left_right_consistent);
    assert!(result.acceptance.deterministic);
    assert!(result.acceptance.passed);

    for controller in [
        GateBControllerKind::Stateless,
        GateBControllerKind::StateReset,
        GateBControllerKind::LeakyState,
    ] {
        let visible = result
            .reports
            .iter()
            .find(|report| {
                report.condition == GateBCondition::CueVisibleAtFork
                    && report.controller == controller
            })
            .expect("visible report");
        assert!(visible.metrics.correct_choice_fraction.mean >= 0.80);

        let randomized = result
            .reports
            .iter()
            .find(|report| {
                report.condition == GateBCondition::CueRandomized && report.controller == controller
            })
            .expect("randomized report");
        let interval = randomized.metrics.correct_choice_fraction;
        assert!(interval.lower95 <= 0.50 && interval.upper95 >= 0.50);
        assert_eq!(randomized.seed_metrics.len(), 12);
    }

    let delayed_leaky = result
        .reports
        .iter()
        .find(|report| {
            report.condition == GateBCondition::DelayedCue
                && report.controller == GateBControllerKind::LeakyState
        })
        .expect("delayed leaky report");
    assert!(delayed_leaky.metrics.correct_choice_fraction.mean >= 0.70);
    assert!(delayed_leaky.metrics.left_target_accuracy.mean >= 0.70);
    assert!(delayed_leaky.metrics.right_target_accuracy.mean >= 0.70);
    assert!(
        (delayed_leaky.metrics.left_target_accuracy.mean
            - delayed_leaky.metrics.right_target_accuracy.mean)
            .abs()
            <= 0.15
    );

    let primary = result
        .paired_effects
        .iter()
        .find(|effect| effect.id == "leaky-vs-state-reset")
        .expect("primary paired effect")
        .effect
        .correct_choice_fraction;
    assert!(primary.mean >= 0.15);
    assert!(primary.lower95 > 0.0);

    let shuffled = result
        .paired_effects
        .iter()
        .find(|effect| effect.id == "delayed-vs-history-shuffled")
        .expect("history-shuffle effect")
        .effect
        .correct_choice_fraction;
    assert!(shuffled.lower95 > 0.0);

    assert!(
        result
            .traces
            .iter()
            .any(|trace| trace.condition == GateBCondition::HistoryShuffled)
    );
}
