use neuro_engine::{
    EmbodiedExperimentConfig, GateAExperimentConfig, RewardConfig, SensorConfig,
    run_embodied_experiment, run_gate_a_experiment,
};

fn quick_gate_a_config() -> GateAExperimentConfig {
    GateAExperimentConfig {
        model_seed_count: 3,
        training_episodes: 120,
        evaluation_episodes: 12,
        curve_window: 20,
        ..GateAExperimentConfig::default()
    }
}

#[test]
fn gate_a_scan_is_deterministic_and_contains_every_canonical_variant() {
    let first = run_gate_a_experiment(quick_gate_a_config()).unwrap();
    let second = run_gate_a_experiment(quick_gate_a_config()).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.version, "embodied-learning/v1.1-reward-audit");
    assert_eq!(first.variants.len(), 9);
    assert!(
        first
            .variants
            .iter()
            .any(|variant| variant.id == "no-distance-shaping")
    );
    assert!(
        first
            .variants
            .iter()
            .any(|variant| variant.id == "no-food-direction")
    );
    let variant = |id: &str| first.variants.iter().find(|item| item.id == id).unwrap();
    assert_eq!(
        variant("no-distance-shaping")
            .reward
            .distance_progress_weight,
        0.0
    );
    assert_eq!(
        variant("direction-precision-0.50")
            .sensors
            .food_direction_precision,
        0.5
    );
    assert_eq!(
        variant("direction-noise-0.15").sensors.food_direction_noise,
        0.15
    );
    assert_eq!(
        variant("direction-dropout-0.35")
            .sensors
            .food_direction_dropout,
        0.35
    );
    assert!(!variant("no-food-direction").sensors.food_direction_enabled);
    assert!(
        first
            .variants
            .iter()
            .all(|variant| variant.seed_runs.len() == 3)
    );
    assert!(first.variants.iter().all(|variant| {
        variant.sample_efficiency.food_threshold == 4.0
            && variant.sample_efficiency.reached_seed_count <= 3
    }));
}

#[test]
fn reward_components_and_food_direction_controls_are_observable() {
    let result = run_embodied_experiment(EmbodiedExperimentConfig {
        reward: RewardConfig {
            distance_progress_weight: 0.0,
            ..RewardConfig::default()
        },
        sensors: SensorConfig {
            food_direction_enabled: false,
            ..SensorConfig::default()
        },
        training_episodes: 120,
        evaluation_episodes: 12,
        curve_window: 20,
        ..EmbodiedExperimentConfig::default()
    })
    .unwrap();

    for frame in result.traces.iter().flat_map(|trace| &trace.frames) {
        assert_eq!(&frame.sensors[3..7], &[0.0; 4]);
        assert_eq!(frame.reward_breakdown.distance_progress, 0.0);
        assert!(
            (frame.reward
                - frame.reward_breakdown.energy_delta
                - frame.reward_breakdown.distance_progress)
                .abs()
                < 1e-12
        );
    }
    for report in &result.evaluations {
        assert_eq!(report.mean_reward_breakdown.distance_progress, 0.0);
        assert!(
            (report.mean_reward_breakdown.total
                - report.mean_reward_breakdown.energy_delta
                - report.mean_reward_breakdown.distance_progress)
                .abs()
                < 1e-9
        );
    }
}

#[test]
fn canonical_gate_a_protocol_passes_statistical_acceptance() {
    let result = run_gate_a_experiment(GateAExperimentConfig {
        model_seed_count: 8,
        training_episodes: 240,
        evaluation_episodes: 24,
        curve_window: 20,
        ..GateAExperimentConfig::default()
    })
    .unwrap();
    assert!(result.acceptance.passed, "{:#?}", result.acceptance);
    let no_shaping = result
        .variants
        .iter()
        .find(|variant| variant.id == "no-distance-shaping")
        .unwrap();
    assert!(no_shaping.paired_effect.foods_eaten.lower95 > 0.0);
}
