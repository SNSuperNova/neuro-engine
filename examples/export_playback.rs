use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use neuro_engine::{
    ArrivalOrigin, Gate2ExperimentConfig, Pattern3x3, Phase2ExperimentConfig, PlaybackDataset,
    SimDuration, build_playback_dataset, run_gate2_experiment, run_phase2_experiment,
};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CompactBundle {
    version: u32,
    topology: CompactTopology,
    branches: Vec<CompactBranch>,
}

#[derive(Serialize)]
struct CompactTopology {
    neurons: Vec<[f64; 10]>,
    synapses: Vec<[f64; 5]>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CompactBranch {
    label: String,
    comparison_base_index: Option<usize>,
    pattern_cells: Option<[bool; 9]>,
    input_neuron_ids: Vec<u32>,
    stimulus_window_ms: Option<[f64; 2]>,
    start_ms: f64,
    end_ms: f64,
    event_digest: String,
    pacemaker_neuron_ids: Vec<u32>,
    spikes: Vec<[f64; 3]>,
    arrivals: Vec<[f64; 9]>,
    samples: Vec<[f64; 4]>,
    metrics: Vec<[f64; 3]>,
    flights: Vec<[f64; 7]>,
}

impl CompactTopology {
    fn from_dataset(dataset: &PlaybackDataset) -> Self {
        Self {
            neurons: dataset
                .neurons
                .iter()
                .map(|neuron| {
                    [
                        f64::from(neuron.id),
                        f64::from(neuron.polarity == "inhibitory"),
                        f64::from(neuron.position[0]),
                        f64::from(neuron.position[1]),
                        f64::from(neuron.position[2]),
                        f64::from(neuron.rest_potential_mv),
                        f64::from(neuron.reset_potential_mv),
                        f64::from(neuron.threshold_mv),
                        f64::from(neuron.refractory_period_ms),
                        f64::from(neuron.membrane_time_constant_ms),
                    ]
                })
                .collect(),
            synapses: dataset
                .synapses
                .iter()
                .map(|synapse| {
                    [
                        f64::from(synapse.id),
                        f64::from(synapse.source),
                        f64::from(synapse.target),
                        f64::from(synapse.magnitude_mv),
                        f64::from(synapse.delay_ms),
                    ]
                })
                .collect(),
        }
    }
}

impl CompactBranch {
    fn from_dataset(
        dataset: PlaybackDataset,
        comparison_base_index: Option<usize>,
        pattern_cells: Option<[bool; 9]>,
        input_neuron_ids: Vec<u32>,
        stimulus_window_ms: Option<[f64; 2]>,
    ) -> Self {
        let spikes = dataset
            .chunks
            .iter()
            .flat_map(|chunk| &chunk.spike_events)
            .map(|spike| [spike.id as f64, f64::from(spike.neuron_id), spike.time_ms])
            .collect();
        let arrivals = dataset
            .chunks
            .iter()
            .flat_map(|chunk| &chunk.arrival_events)
            .filter(|arrival| !matches!(arrival.origin, ArrivalOrigin::Synaptic { .. }))
            .map(|arrival| {
                let (kind, first, second) = match arrival.origin {
                    ArrivalOrigin::Initialization { event_id } => (0.0, event_id as f64, 0.0),
                    ArrivalOrigin::Pacemaker { event_id } => (1.0, event_id as f64, 0.0),
                    ArrivalOrigin::Stimulus { event_id } => (2.0, event_id as f64, 0.0),
                    ArrivalOrigin::Synaptic {
                        spike_id,
                        synapse_id,
                        source,
                    } => (
                        3.0,
                        spike_id as f64,
                        f64::from(synapse_id) * 1_000.0 + f64::from(source),
                    ),
                };
                [
                    arrival.sequence as f64,
                    f64::from(arrival.target),
                    arrival.time_ms,
                    f64::from(arrival.polarity == "inhibitory"),
                    f64::from(arrival.magnitude_mv),
                    kind,
                    first,
                    second,
                    f64::from(arrival.ignored_during_refractory),
                ]
            })
            .collect();

        // The typed Rust contract retains every state transition. The packaged
        // viewer keeps the latest sample in each 10 ms bin, which is enough for
        // display interpolation while substantially reducing startup memory.
        let mut sample_bins = BTreeMap::new();
        for sample in dataset
            .chunks
            .iter()
            .flat_map(|chunk| &chunk.neuron_samples)
        {
            let bin = (sample.time_ms / 10.0).floor() as u64;
            sample_bins.insert((sample.neuron_id, bin), *sample);
        }
        let mut samples = sample_bins
            .into_values()
            .map(|sample| {
                [
                    f64::from(sample.neuron_id),
                    sample.time_ms,
                    f64::from(sample.membrane_potential_mv),
                    sample.refractory_until_ms.unwrap_or(-1.0),
                ]
            })
            .collect::<Vec<_>>();
        samples.sort_by(|left, right| {
            left[1]
                .total_cmp(&right[1])
                .then(left[0].total_cmp(&right[0]))
        });
        let metrics = dataset
            .chunks
            .iter()
            .flat_map(|chunk| &chunk.metric_samples)
            .map(|sample| {
                [
                    sample.time_ms,
                    f64::from(sample.spike_count),
                    f64::from(sample.active_neuron_count),
                ]
            })
            .collect();
        let mut unique_flights = BTreeMap::new();
        for flight in dataset
            .chunks
            .iter()
            .flat_map(|chunk| &chunk.in_flight_intervals)
        {
            unique_flights.insert((flight.spike_id, flight.synapse_id), *flight);
        }
        let flights = unique_flights
            .into_values()
            .map(|flight| {
                [
                    flight.spike_id as f64,
                    f64::from(flight.synapse_id),
                    f64::from(flight.source),
                    f64::from(flight.target),
                    flight.send_time_ms,
                    flight.arrival_time_ms,
                    f64::from(flight.polarity == "inhibitory"),
                ]
            })
            .collect();

        Self {
            label: dataset.label,
            comparison_base_index,
            pattern_cells,
            input_neuron_ids,
            stimulus_window_ms,
            start_ms: dataset.start_ms,
            end_ms: dataset.end_ms,
            event_digest: dataset.event_digest,
            pacemaker_neuron_ids: dataset.pacemaker_neuron_ids,
            spikes,
            arrivals,
            samples,
            metrics,
            flights,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("app/public/experiment-001-v1.json"));
    let experiment = run_gate2_experiment(Gate2ExperimentConfig::default())?;
    let phase2 = run_phase2_experiment(Phase2ExperimentConfig::default())?;
    if experiment.definition != phase2.definition {
        return Err("phase 2 playback must share the frozen baseline topology".into());
    }
    let chunk_duration = SimDuration::from_micros(250_000);
    let datasets = vec![
        build_playback_dataset(
            "continuous pacemaker · control",
            &experiment.definition,
            &experiment.stimulus_control_run,
            chunk_duration,
        )?,
        build_playback_dataset(
            "continuous pacemaker · local stimulus",
            &experiment.definition,
            &experiment.stimulus_variant_run,
            chunk_duration,
        )?,
        build_playback_dataset(
            "pacemaker withdrawal at 1500 ms",
            &experiment.definition,
            &experiment.driven_withdrawal_run,
            chunk_duration,
        )?,
        build_playback_dataset(
            "phase 2 · 3×3 blank control",
            &phase2.definition,
            &phase2.control_run,
            chunk_duration,
        )?,
        build_playback_dataset(
            "phase 2 · 3×3 center cross",
            &phase2.definition,
            &phase2.pattern_run,
            chunk_duration,
        )?,
    ];
    let phase2_input_ids = phase2
        .input_neuron_ids
        .iter()
        .map(|id| id.0)
        .collect::<Vec<_>>();
    let phase2_window = Some([
        phase2.pattern_start.as_micros() as f64 / 1_000.0,
        phase2.pattern_end.as_micros() as f64 / 1_000.0,
    ]);
    let topology = CompactTopology::from_dataset(&datasets[0]);
    let branches = datasets
        .into_iter()
        .enumerate()
        .map(|(index, dataset)| match index {
            0 => CompactBranch::from_dataset(dataset, None, None, Vec::new(), None),
            1 => CompactBranch::from_dataset(dataset, Some(0), None, Vec::new(), None),
            2 => CompactBranch::from_dataset(dataset, None, None, Vec::new(), None),
            3 => CompactBranch::from_dataset(
                dataset,
                None,
                Some(Pattern3x3::BLANK.cells),
                phase2_input_ids.clone(),
                phase2_window,
            ),
            4 => CompactBranch::from_dataset(
                dataset,
                Some(3),
                Some(phase2.pattern.cells),
                phase2_input_ids.clone(),
                phase2_window,
            ),
            _ => unreachable!("all playback branches have explicit metadata"),
        })
        .collect();
    let bundle = CompactBundle {
        version: 1,
        topology,
        branches,
    };
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let writer = BufWriter::new(File::create(&output)?);
    serde_json::to_writer(writer, &bundle)?;
    println!("wrote {}", output.display());
    Ok(())
}
