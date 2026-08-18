use neuro_engine::{
    ACTION_COUNT, EmbodiedExperimentConfig, HIDDEN_COUNT, SENSOR_COUNT, run_embodied_experiment,
};

fn quick_config() -> EmbodiedExperimentConfig {
    EmbodiedExperimentConfig {
        training_episodes: 160,
        evaluation_episodes: 12,
        curve_window: 20,
        ..EmbodiedExperimentConfig::default()
    }
}

#[test]
fn embodied_experiment_is_deterministic() {
    let first = run_embodied_experiment(quick_config()).unwrap();
    let second = run_embodied_experiment(quick_config()).unwrap();
    assert_eq!(first, second);
}

#[test]
fn closed_loop_exposes_sensors_actions_energy_and_behavior() {
    let result = run_embodied_experiment(quick_config()).unwrap();
    assert_eq!(result.sensor_labels.len(), SENSOR_COUNT);
    assert_eq!(result.action_labels.len(), ACTION_COUNT);
    assert_eq!(
        result.plasticity.lesioned_hidden_units.len(),
        HIDDEN_COUNT / 4
    );
    assert!(result.traces.iter().all(|trace| !trace.frames.is_empty()));
    assert!(
        result
            .traces
            .iter()
            .flat_map(|trace| &trace.frames)
            .all(|frame| {
                frame.sensors.len() == SENSOR_COUNT
                    && frame.action_probabilities.len() == ACTION_COUNT
                    && frame.hidden_activity.len() == HIDDEN_COUNT
                    && frame.energy >= 0.0
            })
    );
}

#[test]
fn plasticity_changes_persistent_action_weights() {
    let result = run_embodied_experiment(quick_config()).unwrap();
    assert!(result.plasticity.changed_weight_count > 0);
    assert!(result.plasticity.root_mean_square_change > 0.0);
    assert_eq!(
        result.plasticity.total_weight_count,
        ACTION_COUNT * HIDDEN_COUNT
    );
}

#[test]
fn frozen_v1_protocol_passes_behavioral_and_causal_acceptance() {
    let result = run_embodied_experiment(EmbodiedExperimentConfig::default()).unwrap();
    assert!(result.acceptance.passed, "{:#?}", result.acceptance);
    let report = |label: &str| {
        result
            .evaluations
            .iter()
            .find(|report| report.label == label)
            .unwrap()
    };
    assert_eq!(report("learned").completion_fraction, 1.0);
    assert!(
        report("learned").mean_foods_eaten > report("learning-disabled").mean_foods_eaten + 6.0
    );
    assert!(report("learned").mean_foods_eaten > report("shuffled").mean_foods_eaten + 6.0);
    assert!(report("learned").mean_foods_eaten > report("lesioned").mean_foods_eaten + 6.0);
}
