use neuro_engine::{
    EventId, InputPolarity, LifNeuron, LifParameters, ModelError, NeuronId, SimDuration, SimTime,
    TimedInput, simulate_neuron,
};

const EPSILON: f64 = 1.0e-10;

fn parameters() -> LifParameters {
    LifParameters {
        rest_potential_mv: -70.0,
        reset_potential_mv: -68.0,
        threshold_mv: -55.0,
        membrane_time_constant_ms: 10.0,
        refractory_period: SimDuration::from_micros(2_000),
    }
}

fn neuron(initial_potential_mv: f64) -> LifNeuron {
    LifNeuron::new(
        NeuronId(7),
        parameters(),
        initial_potential_mv,
        SimTime::ZERO,
    )
    .unwrap()
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= EPSILON,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn resting_potential_remains_constant() {
    let mut neuron = neuron(-70.0);
    let snapshot = neuron.advance_to(SimTime::from_micros(50_000)).unwrap();
    assert_eq!(snapshot.membrane_potential_mv, -70.0);
}

#[test]
fn potential_above_rest_decays_analytically() {
    let mut neuron = neuron(-60.0);
    let snapshot = neuron.advance_to(SimTime::from_micros(10_000)).unwrap();
    assert_close(
        snapshot.membrane_potential_mv,
        -70.0 + 10.0 / std::f64::consts::E,
    );
}

#[test]
fn potential_below_rest_decays_toward_rest() {
    let mut neuron = neuron(-80.0);
    let snapshot = neuron.advance_to(SimTime::from_micros(10_000)).unwrap();
    assert_close(
        snapshot.membrane_potential_mv,
        -70.0 - 10.0 / std::f64::consts::E,
    );
}

#[test]
fn excitatory_and_inhibitory_inputs_change_voltage_in_opposite_directions() {
    let time = SimTime::from_micros(1_000);
    let inputs = [
        TimedInput::excitatory(EventId(1), time, 5.0).unwrap(),
        TimedInput::inhibitory(EventId(2), time, 2.0).unwrap(),
    ];
    let result = neuron(-70.0).apply_batch(time, &inputs).unwrap();

    assert_eq!(result.applied_delta_mv, 3.0);
    assert_eq!(result.integrated_potential_mv, -67.0);
    assert!(result.spike.is_none());
}

#[test]
fn subthreshold_input_does_not_fire() {
    let time = SimTime::from_micros(1_000);
    let input = TimedInput::excitatory(EventId(1), time, 14.9).unwrap();
    let result = neuron(-70.0).apply_batch(time, &[input]).unwrap();

    assert!(result.integrated_potential_mv < parameters().threshold_mv);
    assert!(result.spike.is_none());
}

#[test]
fn simultaneous_inputs_are_summed_before_one_threshold_check() {
    let time = SimTime::from_micros(1_000);
    let inputs = [
        TimedInput::excitatory(EventId(2), time, 8.0).unwrap(),
        TimedInput::excitatory(EventId(1), time, 8.0).unwrap(),
    ];
    let result = neuron(-70.0).apply_batch(time, &inputs).unwrap();

    assert_eq!(result.input_event_ids, vec![EventId(1), EventId(2)]);
    assert_eq!(result.integrated_potential_mv, -54.0);
    assert_eq!(result.spike.unwrap().time, time);
}

#[test]
fn a_second_batch_at_the_same_time_is_rejected() {
    let time = SimTime::from_micros(1_000);
    let mut neuron = neuron(-70.0);
    neuron
        .apply_batch(
            time,
            &[TimedInput::excitatory(EventId(1), time, 1.0).unwrap()],
        )
        .unwrap();

    assert!(matches!(
        neuron.apply_batch(
            time,
            &[TimedInput::excitatory(EventId(2), time, 1.0).unwrap()]
        ),
        Err(ModelError::DuplicateInputBatchTime(batch_time)) if batch_time == time
    ));
}

#[test]
fn firing_resets_voltage_and_enters_refractory_period() {
    let time = SimTime::from_micros(1_000);
    let input = TimedInput::excitatory(EventId(1), time, 15.0).unwrap();
    let result = neuron(-70.0).apply_batch(time, &[input]).unwrap();

    assert!(result.spike.is_some());
    assert_eq!(result.membrane_potential_after_mv, -68.0);
}

#[test]
fn input_before_refractory_boundary_is_ignored() {
    let spike_time = SimTime::from_micros(1_000);
    let ignored_time = SimTime::from_micros(2_999);
    let mut neuron = neuron(-70.0);
    neuron
        .apply_batch(
            spike_time,
            &[TimedInput::excitatory(EventId(1), spike_time, 15.0).unwrap()],
        )
        .unwrap();

    let result = neuron
        .apply_batch(
            ignored_time,
            &[TimedInput::excitatory(EventId(2), ignored_time, 100.0).unwrap()],
        )
        .unwrap();

    assert_eq!(result.ignored_input_count, 1);
    assert_eq!(result.applied_delta_mv, 0.0);
    assert_eq!(result.membrane_potential_after_mv, -68.0);
    assert!(result.spike.is_none());
}

#[test]
fn input_at_exact_refractory_boundary_is_applied() {
    let spike_time = SimTime::from_micros(1_000);
    let boundary = SimTime::from_micros(3_000);
    let mut neuron = neuron(-70.0);
    neuron
        .apply_batch(
            spike_time,
            &[TimedInput::excitatory(EventId(1), spike_time, 15.0).unwrap()],
        )
        .unwrap();

    let result = neuron
        .apply_batch(
            boundary,
            &[TimedInput::excitatory(EventId(2), boundary, 13.0).unwrap()],
        )
        .unwrap();

    assert_eq!(result.ignored_input_count, 0);
    assert_eq!(result.integrated_potential_mv, -55.0);
    assert!(result.spike.is_some());
}

#[test]
fn refractory_voltage_is_held_then_decays_from_boundary() {
    let spike_time = SimTime::from_micros(1_000);
    let mut neuron = neuron(-70.0);
    neuron
        .apply_batch(
            spike_time,
            &[TimedInput::excitatory(EventId(1), spike_time, 15.0).unwrap()],
        )
        .unwrap();

    let held = neuron.advance_to(SimTime::from_micros(2_000)).unwrap();
    assert_eq!(held.membrane_potential_mv, -68.0);
    assert!(held.is_refractory());

    let decayed = neuron.advance_to(SimTime::from_micros(13_000)).unwrap();
    assert_close(
        decayed.membrane_potential_mv,
        -70.0 + 2.0 / std::f64::consts::E,
    );
    assert!(!decayed.is_refractory());
}

#[test]
fn canonical_simulation_is_independent_of_caller_input_order() {
    let inputs = vec![
        TimedInput::inhibitory(EventId(4), SimTime::from_micros(5_000), 1.0).unwrap(),
        TimedInput::excitatory(EventId(2), SimTime::from_micros(1_000), 6.0).unwrap(),
        TimedInput::excitatory(EventId(1), SimTime::from_micros(1_000), 10.0).unwrap(),
        TimedInput::excitatory(EventId(3), SimTime::from_micros(3_000), 13.0).unwrap(),
    ];
    let mut reversed = inputs.clone();
    reversed.reverse();

    let first = simulate_neuron(neuron(-70.0), &inputs).unwrap();
    let second = simulate_neuron(neuron(-70.0), &reversed).unwrap();

    assert_eq!(first, second);
    assert_eq!(first.spikes.len(), 2);
}

#[test]
fn invalid_input_and_time_are_rejected() {
    assert!(matches!(
        TimedInput::new(EventId(1), SimTime::ZERO, InputPolarity::Excitatory, -1.0),
        Err(ModelError::InvalidInputMagnitude { .. })
    ));

    let mut neuron = neuron(-70.0);
    neuron.advance_to(SimTime::from_micros(1_000)).unwrap();
    assert!(matches!(
        neuron.advance_to(SimTime::from_micros(999)),
        Err(ModelError::TimeWentBackwards { .. })
    ));
}

#[test]
fn failed_batches_do_not_partially_mutate_the_neuron() {
    let time = SimTime::from_micros(u64::MAX);
    let mut neuron = neuron(-70.0);
    let original = neuron.snapshot();
    let overflow = [
        TimedInput::excitatory(EventId(1), time, f64::MAX).unwrap(),
        TimedInput::excitatory(EventId(2), time, f64::MAX).unwrap(),
    ];

    assert!(matches!(
        neuron.apply_batch(time, &overflow),
        Err(ModelError::NonFiniteInputSum)
    ));
    assert_eq!(neuron.snapshot(), original);

    let spike = TimedInput::excitatory(EventId(3), time, 15.0).unwrap();
    assert!(matches!(
        neuron.apply_batch(time, &[spike]),
        Err(ModelError::TimeOverflow)
    ));
    assert_eq!(neuron.snapshot(), original);
}
