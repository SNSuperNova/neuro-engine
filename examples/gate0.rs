use neuro_engine::{
    EventId, LifNeuron, LifParameters, NeuronId, SimDuration, SimTime, TimedInput, simulate_neuron,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parameters = LifParameters {
        rest_potential_mv: -70.0,
        reset_potential_mv: -68.0,
        threshold_mv: -55.0,
        membrane_time_constant_ms: 10.0,
        refractory_period: SimDuration::from_micros(2_000),
    };
    let neuron = LifNeuron::new(NeuronId(1), parameters, -70.0, SimTime::ZERO)?;
    let inputs = vec![
        TimedInput::excitatory(EventId(1), SimTime::from_micros(1_000), 8.0)?,
        TimedInput::excitatory(EventId(2), SimTime::from_micros(1_000), 8.0)?,
        TimedInput::excitatory(EventId(3), SimTime::from_micros(3_000), 13.0)?,
    ];

    let trace = simulate_neuron(neuron, &inputs)?;
    for batch in trace.batches {
        println!(
            "t={:>5} us  integrated={:>7.3} mV  after={:>7.3} mV  spike={}",
            batch.time.as_micros(),
            batch.integrated_potential_mv,
            batch.membrane_potential_after_mv,
            batch.spike.is_some()
        );
    }

    Ok(())
}
