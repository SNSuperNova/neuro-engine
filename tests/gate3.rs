use neuro_engine::{
    ArrivalOrigin, Gate2ExperimentConfig, SimDuration, build_playback_dataset, run_gate2_experiment,
};

fn dataset() -> neuro_engine::PlaybackDataset {
    let experiment = run_gate2_experiment(Gate2ExperimentConfig::default()).unwrap();
    build_playback_dataset(
        "test",
        &experiment.definition,
        &experiment.driven_withdrawal_run,
        SimDuration::from_micros(250_000),
    )
    .unwrap()
}

#[test]
fn playback_contract_contains_topology_events_samples_and_metrics() {
    let playback = dataset();
    assert_eq!(playback.version, 1);
    assert_eq!(playback.neurons.len(), 100);
    assert_eq!(playback.synapses.len(), 1_360);
    assert_eq!(playback.chunks.len(), 10);
    assert_eq!(playback.pacemaker_neuron_ids, vec![0, 1, 2, 3]);
    assert!(
        playback
            .chunks
            .iter()
            .any(|chunk| !chunk.spike_events.is_empty())
    );
    assert!(
        playback
            .chunks
            .iter()
            .any(|chunk| !chunk.arrival_events.is_empty())
    );
    assert!(
        playback
            .chunks
            .iter()
            .any(|chunk| !chunk.neuron_samples.is_empty())
    );
    assert!(
        playback
            .chunks
            .iter()
            .all(|chunk| !chunk.metric_samples.is_empty())
    );
}

#[test]
fn playback_reconstructs_external_sources_and_synaptic_flights() {
    let playback = dataset();
    let arrivals = playback
        .chunks
        .iter()
        .flat_map(|chunk| &chunk.arrival_events)
        .collect::<Vec<_>>();
    assert!(
        arrivals
            .iter()
            .any(|event| matches!(event.origin, ArrivalOrigin::Pacemaker { .. }))
    );
    assert!(
        arrivals
            .iter()
            .any(|event| matches!(event.origin, ArrivalOrigin::Synaptic { .. }))
    );
    let flights = playback
        .chunks
        .iter()
        .flat_map(|chunk| &chunk.in_flight_intervals)
        .collect::<Vec<_>>();
    assert!(!flights.is_empty());
    assert!(
        flights
            .iter()
            .all(|flight| flight.arrival_time_ms > flight.send_time_ms)
    );
}

#[test]
fn playback_export_is_deterministic_json() {
    let first = serde_json::to_vec(&dataset()).unwrap();
    let second = serde_json::to_vec(&dataset()).unwrap();
    assert_eq!(first, second);
}

#[test]
fn zero_chunk_duration_is_rejected() {
    let experiment = run_gate2_experiment(Gate2ExperimentConfig::default()).unwrap();
    let result = build_playback_dataset(
        "invalid",
        &experiment.definition,
        &experiment.driven_withdrawal_run,
        SimDuration::from_micros(0),
    );
    assert!(result.is_err());
}
