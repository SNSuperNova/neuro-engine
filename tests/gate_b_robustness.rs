use neuro_engine::{
    GateBRobustnessConfig, GateBRobustnessFactor, run_gate_b_robustness_experiment,
};

fn quick_config() -> GateBRobustnessConfig {
    GateBRobustnessConfig {
        model_seed_count: 2,
        training_episodes: 40,
        evaluation_episodes: 8,
        curve_window: 10,
        ..GateBRobustnessConfig::default()
    }
}

#[test]
fn robustness_report_is_deterministic_and_contains_the_frozen_scan() {
    let first = run_gate_b_robustness_experiment(quick_config()).expect("first boundary run");
    let second = run_gate_b_robustness_experiment(quick_config()).expect("second boundary run");
    assert_eq!(first, second);
    assert_eq!(first.points.len(), 11);
    assert_eq!(
        first
            .points
            .iter()
            .filter(|point| point.id == "delay-4")
            .count(),
        1
    );
    assert_eq!(
        first
            .points
            .iter()
            .filter(|point| point.factor == GateBRobustnessFactor::DelaySteps)
            .map(|point| point.delay_steps)
            .collect::<Vec<_>>(),
        vec![2, 4, 6, 8, 12]
    );
    assert!(
        !serde_json::to_string(&first)
            .expect("serialize")
            .contains("runtime")
    );
}

#[test]
fn frozen_boundary_exposes_memory_and_input_sensitivity() {
    let result = run_gate_b_robustness_experiment(GateBRobustnessConfig::default())
        .expect("full robustness scan");
    assert_eq!(result.reliable_memory_boundary_steps, Some(4));
    let accuracy = |id: &str| {
        result
            .points
            .iter()
            .find(|point| point.id == id)
            .expect("frozen point")
            .report
            .metrics
            .correct_choice_fraction
    };
    assert!(accuracy("delay-2").mean >= 0.70);
    assert!(accuracy("delay-4").mean >= 0.70);
    assert!(accuracy("delay-4").lower95 > 0.50);
    assert!(accuracy("delay-6").mean < 0.70);
    assert!(accuracy("delay-12").lower95 <= 0.50);
    assert!(accuracy("cue-scale-1.50").mean > accuracy("cue-scale-0.60").mean);
    assert!(accuracy("persistent-scale-0.10").mean > accuracy("persistent-scale-0.65").mean);
}
