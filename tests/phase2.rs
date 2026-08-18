use neuro_engine::{
    ActivityRegulatorConfig, EventId, ExternalInput, ExternalInputKind, InputOrigin, InputPolarity,
    NeuronId, Pattern3x3, PatternStimulusSchedule, Phase2ExperimentConfig, Phase2PatternId,
    Phase2ProtocolConfig, SimDuration, SimTime, degree_preserving_connection_shuffle,
    generate_network, pattern_stimulus_inputs, run_phase2_experiment, run_phase2_protocol,
    silence_neurons, simulate_network_with_regulator,
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
fn primary_patterns_have_equal_input_energy_and_distinct_geometry() {
    let patterns = Phase2PatternId::PRIMARY.map(Phase2PatternId::pattern);
    assert!(
        patterns
            .iter()
            .all(|pattern| pattern.active_cell_count() == 3)
    );
    let distinct = patterns
        .iter()
        .map(|pattern| pattern.cells)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(distinct.len(), patterns.len());
}

#[test]
fn connection_shuffle_preserves_exact_directed_degrees() {
    let definition = generate_network(Phase2ExperimentConfig::default().network).unwrap();
    let shuffled = degree_preserving_connection_shuffle(&definition, 42, 20_000).unwrap();
    let degrees = |synapses: &[neuro_engine::SynapseSpec]| {
        let mut incoming = std::collections::BTreeMap::new();
        let mut outgoing = std::collections::BTreeMap::new();
        for synapse in synapses {
            *incoming.entry(synapse.target).or_insert(0_usize) += 1;
            *outgoing.entry(synapse.source).or_insert(0_usize) += 1;
        }
        (incoming, outgoing)
    };
    assert_eq!(degrees(definition.synapses()), degrees(shuffled.synapses()));
    assert_ne!(definition.synapses(), shuffled.synapses());
}

#[test]
fn silencing_removes_every_incident_connection_without_renumbering_neurons() {
    let definition = generate_network(Phase2ExperimentConfig::default().network).unwrap();
    let silenced = [NeuronId(17), NeuronId(29)]
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    let result = silence_neurons(&definition, &silenced).unwrap();

    assert_eq!(definition.neurons(), result.neurons());
    assert!(result.synapses().iter().all(|synapse| {
        !silenced.contains(&synapse.source) && !silenced.contains(&synapse.target)
    }));
}

#[test]
fn propagation_gate_is_deterministic_and_records_suppressed_connections() {
    let definition = generate_network(Phase2ExperimentConfig::default().network).unwrap();
    let inputs = definition
        .neurons()
        .iter()
        .take(20)
        .enumerate()
        .map(|(index, neuron)| {
            ExternalInput::new(
                EventId(index as u64),
                neuron.id,
                SimTime::from_micros(1_000),
                InputPolarity::Excitatory,
                100.0,
                ExternalInputKind::Stimulus,
            )
        })
        .collect::<Vec<_>>();
    let regulator = ActivityRegulatorConfig {
        unique_spike_threshold: 2,
        ..ActivityRegulatorConfig::default()
    };
    let run = || {
        simulate_network_with_regulator(
            &definition,
            &inputs,
            SimTime::from_micros(20_000),
            regulator,
        )
        .unwrap()
    };
    let first = run();
    let second = run();

    assert_eq!(first.event_log, second.event_log);
    assert!(first.regulation_episode_count > 0);
    assert_eq!(
        first.regulation_episode_count,
        first.event_log.regulations().count()
    );
    assert!(first.suppressed_propagation_count > 0);
}

#[test]
fn quick_protocol_runs_the_complete_statistical_and_ablation_pipeline() {
    let config = Phase2ProtocolConfig {
        structure_seed_count: 1,
        trials_per_pattern: 6,
        training_trials_per_pattern: 4,
        single_pixel_trials: 1,
        functional_group_size: 3,
        label_permutation_count: 5,
        connection_swap_multiplier: 1,
        ..Phase2ProtocolConfig::default()
    };
    let result = run_phase2_protocol(config).unwrap();

    assert_eq!(result.report.seeds.len(), 1);
    assert_eq!(result.report.seeds[0].functional_groups.len(), 4);
    assert_eq!(result.report.seeds[0].ablations.len(), 4);
    assert_eq!(result.report.seeds[0].readout.prototype.total, 8);
    assert!(
        result
            .trials
            .iter()
            .all(|trial| !trial.neuron_ids.contains(&4))
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
