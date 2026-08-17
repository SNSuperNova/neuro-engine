use neuro_engine::{
    EventId, ExternalInput, ExternalInputKind, InputPolarity, LifParameters, NetworkDefinition,
    NetworkError, NeuronId, NeuronPolarity, NeuronSpec, Position3, PropagationSpec, SimDuration,
    SimTime, SimulationLimits, SynapseId, SynapseSpec, simulate_network,
    simulate_network_with_limits,
};

fn parameters() -> LifParameters {
    LifParameters {
        rest_potential_mv: -70.0,
        reset_potential_mv: -68.0,
        threshold_mv: -55.0,
        membrane_time_constant_ms: 10.0,
        refractory_period: SimDuration::from_micros(2_000),
    }
}

fn ring_network(neuron_count: u32) -> NetworkDefinition {
    let neurons = (0..neuron_count)
        .map(|id| NeuronSpec {
            id: NeuronId(id),
            polarity: NeuronPolarity::Excitatory,
            position: Position3::new(f64::from(id), 0.0, 0.0),
            parameters: parameters(),
            initial_potential_mv: -70.0,
        })
        .collect::<Vec<_>>();
    let synapses = (0..neuron_count)
        .map(|source| SynapseSpec {
            id: SynapseId(source),
            source: NeuronId(source),
            target: NeuronId((source + 1) % neuron_count),
            magnitude_mv: 15.0,
            propagation: PropagationSpec {
                path_length: 1.0 + f64::from(source % 3) * 0.25,
                conduction_velocity_units_per_ms: 1.0,
                synaptic_delay: SimDuration::from_micros(0),
            },
        })
        .collect::<Vec<_>>();
    NetworkDefinition::new(SimTime::ZERO, neurons, synapses).unwrap()
}

#[test]
fn ten_neuron_cycle_propagates_and_keeps_multiple_spikes_in_flight() {
    let mut definition = ring_network(10);
    let mut synapses = definition.synapses().to_vec();
    let next_id = synapses.len() as u32;
    synapses.push(SynapseSpec {
        id: SynapseId(next_id),
        source: NeuronId(0),
        target: NeuronId(5),
        magnitude_mv: 15.0,
        propagation: PropagationSpec {
            path_length: 8.0,
            conduction_velocity_units_per_ms: 1.0,
            synaptic_delay: SimDuration::from_micros(0),
        },
    });
    definition = NetworkDefinition::new(
        definition.start_time(),
        definition.neurons().to_vec(),
        synapses,
    )
    .unwrap();

    let initial = ExternalInput::new(
        EventId(100),
        NeuronId(0),
        SimTime::from_micros(1_000),
        InputPolarity::Excitatory,
        15.0,
        ExternalInputKind::Initialization,
    );
    let run = simulate_network(&definition, &[initial], SimTime::from_micros(30_000)).unwrap();
    let fired = run
        .event_log
        .spikes()
        .map(|spike| spike.neuron_id)
        .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(fired.len(), 10);
    assert!(run.event_log.spikes().count() > 10, "the ring should cycle");
    assert!(run.max_in_flight_inputs >= 2);
}

#[test]
fn fifty_neuron_cycle_propagates_across_the_gate1_upper_bound() {
    let definition = ring_network(50);
    let initial = ExternalInput::new(
        EventId(1),
        NeuronId(0),
        SimTime::from_micros(1_000),
        InputPolarity::Excitatory,
        15.0,
        ExternalInputKind::Initialization,
    );
    let run = simulate_network(&definition, &[initial], SimTime::from_micros(80_000)).unwrap();
    let fired = run
        .event_log
        .spikes()
        .map(|spike| spike.neuron_id)
        .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(fired.len(), 50);
}

#[test]
fn simultaneous_external_inputs_trigger_only_one_spike() {
    let definition = ring_network(10);
    let time = SimTime::from_micros(1_000);
    let inputs = [
        ExternalInput::new(
            EventId(2),
            NeuronId(0),
            time,
            InputPolarity::Excitatory,
            8.0,
            ExternalInputKind::Initialization,
        ),
        ExternalInput::new(
            EventId(1),
            NeuronId(0),
            time,
            InputPolarity::Excitatory,
            8.0,
            ExternalInputKind::Initialization,
        ),
    ];
    let run = simulate_network(&definition, &inputs, time).unwrap();

    assert_eq!(run.event_log.inputs().count(), 2);
    assert_eq!(run.event_log.spikes().count(), 1);
}

#[test]
fn caller_order_does_not_change_event_log_digest() {
    let definition = ring_network(10);
    let mut inputs = vec![
        ExternalInput::new(
            EventId(3),
            NeuronId(2),
            SimTime::from_micros(2_000),
            InputPolarity::Inhibitory,
            1.0,
            ExternalInputKind::Stimulus,
        ),
        ExternalInput::new(
            EventId(2),
            NeuronId(0),
            SimTime::from_micros(1_000),
            InputPolarity::Excitatory,
            8.0,
            ExternalInputKind::Initialization,
        ),
        ExternalInput::new(
            EventId(1),
            NeuronId(0),
            SimTime::from_micros(1_000),
            InputPolarity::Excitatory,
            8.0,
            ExternalInputKind::Initialization,
        ),
    ];
    let first = simulate_network(&definition, &inputs, SimTime::from_micros(30_000)).unwrap();
    inputs.reverse();
    let second = simulate_network(&definition, &inputs, SimTime::from_micros(30_000)).unwrap();

    assert_eq!(first.event_log, second.event_log);
    assert_eq!(
        first.event_log.stable_digest(),
        second.event_log.stable_digest()
    );
}

#[test]
fn a_spike_can_remain_in_flight_at_the_end_of_a_run() {
    let neurons = ring_network(10).neurons().to_vec();
    let synapse = SynapseSpec {
        id: SynapseId(0),
        source: NeuronId(0),
        target: NeuronId(1),
        magnitude_mv: 15.0,
        propagation: PropagationSpec {
            path_length: 100.0,
            conduction_velocity_units_per_ms: 1.0,
            synaptic_delay: SimDuration::from_micros(0),
        },
    };
    let definition = NetworkDefinition::new(SimTime::ZERO, neurons, vec![synapse]).unwrap();
    let initial = ExternalInput::new(
        EventId(1),
        NeuronId(0),
        SimTime::from_micros(1_000),
        InputPolarity::Excitatory,
        15.0,
        ExternalInputKind::Initialization,
    );
    let run = simulate_network(&definition, &[initial], SimTime::from_micros(10_000)).unwrap();

    assert_eq!(run.in_flight_inputs_at_end, 1);
}

#[test]
fn zero_delay_synapses_are_rejected_before_simulation() {
    let neurons = ring_network(10).neurons().to_vec();
    let synapse = SynapseSpec {
        id: SynapseId(0),
        source: NeuronId(0),
        target: NeuronId(1),
        magnitude_mv: 15.0,
        propagation: PropagationSpec {
            path_length: 0.0,
            conduction_velocity_units_per_ms: 1.0,
            synaptic_delay: SimDuration::from_micros(0),
        },
    };

    assert!(matches!(
        NetworkDefinition::new(SimTime::ZERO, neurons, vec![synapse]),
        Err(NetworkError::InvalidPropagation {
            synapse_id: SynapseId(0),
            reason
        }) if matches!(*reason, NetworkError::ZeroPropagationDelay)
    ));
}

#[test]
fn configured_event_limits_stop_runaway_work() {
    let definition = ring_network(10);
    let time = SimTime::from_micros(1_000);
    let inputs = [
        ExternalInput::new(
            EventId(1),
            NeuronId(0),
            time,
            InputPolarity::Excitatory,
            8.0,
            ExternalInputKind::Initialization,
        ),
        ExternalInput::new(
            EventId(2),
            NeuronId(0),
            time,
            InputPolarity::Excitatory,
            8.0,
            ExternalInputKind::Initialization,
        ),
    ];

    assert!(matches!(
        simulate_network_with_limits(
            &definition,
            &inputs,
            SimTime::from_micros(30_000),
            SimulationLimits {
                maximum_processed_inputs: 1,
                maximum_queued_inputs: 100,
            }
        ),
        Err(NetworkError::ProcessedInputLimitExceeded { limit: 1 })
    ));
}
