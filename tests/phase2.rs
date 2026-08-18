use neuro_engine::{
    ExternalInputKind, InputOrigin, Pattern3x3, PatternStimulusSchedule, Phase2ExperimentConfig,
    SimDuration, SimTime, generate_network, pattern_stimulus_inputs, run_phase2_experiment,
};

#[test]
fn pattern_encoder_targets_only_active_cells_over_a_bounded_window() {
    let config = Phase2ExperimentConfig::default();
    let definition = generate_network(config.network).unwrap();
    let inputs = pattern_stimulus_inputs(
        &definition,
        Pattern3x3::CENTER_CROSS,
        config.input_neuron_ids,
        PatternStimulusSchedule {
            start: SimTime::from_micros(1_000),
            end: SimTime::from_micros(5_000),
            interval: SimDuration::from_micros(1_000),
            magnitude_mv: 8.0,
            first_event_id: 50_000,
        },
    )
    .unwrap();

    assert_eq!(
        inputs.len(),
        4 * Pattern3x3::CENTER_CROSS.active_cell_count()
    );
    assert!(inputs.iter().all(|input| {
        input.time >= SimTime::from_micros(1_000)
            && input.time < SimTime::from_micros(5_000)
            && input.kind == ExternalInputKind::Stimulus
    }));
    let active_targets = Pattern3x3::CENTER_CROSS
        .cells
        .iter()
        .zip(config.input_neuron_ids)
        .filter_map(|(active, target)| active.then_some(target))
        .collect::<std::collections::BTreeSet<_>>();
    assert!(
        inputs
            .iter()
            .all(|input| active_targets.contains(&input.target))
    );
}

#[test]
fn phase2_pair_is_deterministic_and_differs_only_by_recorded_pattern_inputs() {
    let config = Phase2ExperimentConfig::default();
    let first = run_phase2_experiment(config).unwrap();
    let second = run_phase2_experiment(config).unwrap();

    assert_eq!(first.summary, second.summary);
    assert_eq!(first.control_run.event_log, second.control_run.event_log);
    assert_eq!(first.pattern_run.event_log, second.pattern_run.event_log);
    assert_ne!(first.summary.control_digest, first.summary.pattern_digest);
    assert_eq!(
        first.summary.pattern_input_event_count,
        14 * Pattern3x3::CENTER_CROSS.active_cell_count()
    );
    assert_eq!(
        first
            .control_run
            .event_log
            .inputs()
            .filter(|input| matches!(
                input.origin,
                InputOrigin::External {
                    kind: ExternalInputKind::Stimulus,
                    ..
                }
            ))
            .count(),
        0
    );
    assert_eq!(
        first
            .pattern_run
            .event_log
            .inputs()
            .filter(|input| matches!(
                input.origin,
                InputOrigin::External {
                    kind: ExternalInputKind::Stimulus,
                    ..
                }
            ))
            .count(),
        first.summary.pattern_input_event_count
    );
    assert!(first.summary.trajectory.changed_bin_fraction > 0.0);
}
