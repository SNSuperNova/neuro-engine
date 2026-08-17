use neuro_engine::{
    EventId, ExternalInput, ExternalInputKind, Gate2AcceptanceCriteria, Gate2ExperimentConfig,
    InputOrigin, InputPolarity, NeuronId, NeuronPolarity, SimTime, generate_network,
    run_gate2_experiment, simulate_network,
};

#[test]
fn generated_network_has_the_frozen_size_balance_and_spatial_delays() {
    let config = Gate2ExperimentConfig::default();
    let first = generate_network(config.network).unwrap();
    let second = generate_network(config.network).unwrap();

    assert_eq!(first, second);
    assert_eq!(first.neurons().len(), 100);
    assert_eq!(
        first
            .neurons()
            .iter()
            .filter(|neuron| neuron.polarity == NeuronPolarity::Excitatory)
            .count(),
        80
    );
    assert_eq!(
        first
            .neurons()
            .iter()
            .filter(|neuron| neuron.polarity == NeuronPolarity::Inhibitory)
            .count(),
        20
    );
    assert_eq!(first.synapses().len(), 80 * 9 + 20 * 32);
    assert!(first.synapses().iter().all(|synapse| {
        synapse.propagation.path_length > 0.0
            && synapse.propagation.resolved_delay().unwrap().as_micros() > 0
    }));
}

#[test]
fn three_hundred_neuron_network_runs_at_the_gate2_upper_bound() {
    let mut config = Gate2ExperimentConfig::default().network;
    config.neuron_count = 300;
    config.inhibitory_neuron_count = 60;
    let definition = generate_network(config).unwrap();
    let initial = ExternalInput::new(
        EventId(1),
        NeuronId(0),
        SimTime::from_micros(1_000),
        InputPolarity::Excitatory,
        16.0,
        ExternalInputKind::Initialization,
    );
    let run = simulate_network(&definition, &[initial], SimTime::from_micros(20_000)).unwrap();

    assert_eq!(definition.neurons().len(), 300);
    assert_eq!(run.final_states.len(), 300);
    assert!(run.event_log.spikes().count() >= 1);
}

#[test]
fn experiment_001_v1_is_deterministic_and_meets_frozen_acceptance() {
    let config = Gate2ExperimentConfig::default();
    let first = run_gate2_experiment(config).unwrap();
    let second = run_gate2_experiment(config).unwrap();
    let acceptance = Gate2AcceptanceCriteria::experiment_001_v1().evaluate(&first.summary);

    assert!(acceptance.passed, "violations: {:?}", acceptance.violations);
    assert_eq!(first.summary, second.summary);
    assert_eq!(
        first.driven_withdrawal_run.event_log,
        second.driven_withdrawal_run.event_log
    );
}

#[test]
fn controls_distinguish_pacemaker_support_from_recurrent_self_maintenance() {
    let result = run_gate2_experiment(Gate2ExperimentConfig::default()).unwrap();
    let summary = result.summary;

    assert!(summary.no_drive.mean_firing_rate_hz < 0.5);
    assert!(summary.driven.mean_firing_rate_hz >= 4.0);
    assert!(summary.driven.active_neuron_fraction >= 0.45);
    assert!(summary.driven.excitatory_arrival_count > 0);
    assert!(summary.driven.inhibitory_arrival_count > 0);
    assert_eq!(summary.withdrawal.total_spikes, 0);
    assert!(summary.withdrawal_last_spike_after_stop_ms.is_none());
}

#[test]
fn local_stimulus_changes_the_later_population_trajectory() {
    let result = run_gate2_experiment(Gate2ExperimentConfig::default()).unwrap();
    let trajectory = &result.summary.stimulus_trajectory;

    assert!(trajectory.mean_rms_spike_difference >= 0.25);
    assert!(trajectory.maximum_rms_spike_difference > trajectory.mean_rms_spike_difference);
    assert!(trajectory.changed_bin_fraction >= 0.50);
    assert_ne!(
        result.summary.stimulus_control_digest,
        result.summary.stimulus_variant_digest
    );
    assert_eq!(
        result
            .stimulus_variant_run
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
        8
    );
}
