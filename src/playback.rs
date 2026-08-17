use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter};

use serde::Serialize;

use crate::lif::{LifNeuron, NeuronId, SimDuration, SimTime, TimedInput};
use crate::network::{
    ExternalInputKind, InputOrigin, NetworkDefinition, NetworkRun, NeuronPolarity,
};

const PLAYBACK_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackBundle {
    pub version: u32,
    pub datasets: Vec<PlaybackDataset>,
}

impl PlaybackBundle {
    pub fn new(datasets: Vec<PlaybackDataset>) -> Self {
        Self {
            version: PLAYBACK_VERSION,
            datasets,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackDataset {
    pub version: u32,
    pub label: String,
    pub start_ms: f64,
    pub end_ms: f64,
    pub event_digest: String,
    pub neurons: Vec<PlaybackNeuron>,
    pub synapses: Vec<PlaybackSynapse>,
    pub pacemaker_neuron_ids: Vec<u32>,
    pub chunks: Vec<PlaybackChunk>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackNeuron {
    pub id: u32,
    pub polarity: &'static str,
    pub position: [f32; 3],
    pub rest_potential_mv: f32,
    pub reset_potential_mv: f32,
    pub threshold_mv: f32,
    pub membrane_time_constant_ms: f32,
    pub refractory_period_ms: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackSynapse {
    pub id: u32,
    pub source: u32,
    pub target: u32,
    pub magnitude_mv: f32,
    pub delay_ms: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackChunk {
    pub start_ms: f64,
    pub end_ms: f64,
    pub spike_events: Vec<PlaybackSpike>,
    pub arrival_events: Vec<PlaybackArrival>,
    pub neuron_samples: Vec<PlaybackNeuronSample>,
    pub metric_samples: Vec<PlaybackMetricSample>,
    pub in_flight_intervals: Vec<PlaybackInFlight>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackSpike {
    pub id: u64,
    pub neuron_id: u32,
    pub time_ms: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackArrival {
    pub sequence: u64,
    pub target: u32,
    pub time_ms: f64,
    pub polarity: &'static str,
    pub magnitude_mv: f32,
    pub origin: ArrivalOrigin,
    pub ignored_during_refractory: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ArrivalOrigin {
    Initialization {
        event_id: u64,
    },
    Pacemaker {
        event_id: u64,
    },
    Stimulus {
        event_id: u64,
    },
    Synaptic {
        spike_id: u64,
        synapse_id: u32,
        source: u32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackNeuronSample {
    pub neuron_id: u32,
    pub time_ms: f64,
    pub membrane_potential_mv: f32,
    pub refractory_until_ms: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackMetricSample {
    pub time_ms: f64,
    pub spike_count: u32,
    pub active_neuron_count: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackInFlight {
    pub spike_id: u64,
    pub synapse_id: u32,
    pub source: u32,
    pub target: u32,
    pub send_time_ms: f64,
    pub arrival_time_ms: f64,
    pub polarity: &'static str,
}

pub fn build_playback_dataset(
    label: impl Into<String>,
    definition: &NetworkDefinition,
    run: &NetworkRun,
    chunk_duration: SimDuration,
) -> Result<PlaybackDataset, PlaybackError> {
    if chunk_duration.as_micros() == 0 {
        return Err(PlaybackError::ZeroChunkDuration);
    }

    let neurons = definition
        .neurons()
        .iter()
        .map(|neuron| PlaybackNeuron {
            id: neuron.id.0,
            polarity: polarity_name(neuron.polarity),
            position: [
                neuron.position.x as f32,
                neuron.position.y as f32,
                neuron.position.z as f32,
            ],
            rest_potential_mv: neuron.parameters.rest_potential_mv as f32,
            reset_potential_mv: neuron.parameters.reset_potential_mv as f32,
            threshold_mv: neuron.parameters.threshold_mv as f32,
            membrane_time_constant_ms: neuron.parameters.membrane_time_constant_ms as f32,
            refractory_period_ms: micros_to_ms(neuron.parameters.refractory_period.as_micros())
                as f32,
        })
        .collect();
    let synapses = definition
        .synapses()
        .iter()
        .map(|synapse| {
            Ok(PlaybackSynapse {
                id: synapse.id.0,
                source: synapse.source.0,
                target: synapse.target.0,
                magnitude_mv: synapse.magnitude_mv as f32,
                delay_ms: micros_to_ms(synapse.propagation.resolved_delay()?.as_micros()) as f32,
            })
        })
        .collect::<Result<Vec<_>, crate::network::NetworkError>>()?;

    let spikes = run
        .event_log
        .spikes()
        .map(|spike| PlaybackSpike {
            id: spike.id.0,
            neuron_id: spike.neuron_id.0,
            time_ms: micros_to_ms(spike.time.as_micros()),
        })
        .collect::<Vec<_>>();
    let arrivals = run
        .event_log
        .inputs()
        .map(|input| PlaybackArrival {
            sequence: input.sequence.0,
            target: input.target.0,
            time_ms: micros_to_ms(input.time.as_micros()),
            polarity: input_polarity_name(input.polarity),
            magnitude_mv: input.magnitude_mv as f32,
            origin: match input.origin {
                InputOrigin::External {
                    external_event_id,
                    kind: ExternalInputKind::Initialization,
                } => ArrivalOrigin::Initialization {
                    event_id: external_event_id.0,
                },
                InputOrigin::External {
                    external_event_id,
                    kind: ExternalInputKind::Pacemaker,
                } => ArrivalOrigin::Pacemaker {
                    event_id: external_event_id.0,
                },
                InputOrigin::External {
                    external_event_id,
                    kind: ExternalInputKind::Stimulus,
                } => ArrivalOrigin::Stimulus {
                    event_id: external_event_id.0,
                },
                InputOrigin::Synaptic {
                    spike_id,
                    synapse_id,
                    source,
                } => ArrivalOrigin::Synaptic {
                    spike_id: spike_id.0,
                    synapse_id: synapse_id.0,
                    source: source.0,
                },
            },
            ignored_during_refractory: input.ignored_during_refractory,
        })
        .collect::<Vec<_>>();

    let pacemaker_neuron_ids = arrivals
        .iter()
        .filter(|arrival| matches!(arrival.origin, ArrivalOrigin::Pacemaker { .. }))
        .map(|arrival| arrival.target)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let samples = reconstruct_samples(definition, run)?;
    let in_flight = reconstruct_in_flight(definition, &spikes, &arrivals)?;
    let metrics = build_metric_samples(definition, &spikes, run.start_time, run.end_time);
    let chunks = chunk_events(
        run.start_time,
        run.end_time,
        chunk_duration,
        &spikes,
        &arrivals,
        &samples,
        &metrics,
        &in_flight,
    );

    Ok(PlaybackDataset {
        version: PLAYBACK_VERSION,
        label: label.into(),
        start_ms: micros_to_ms(run.start_time.as_micros()),
        end_ms: micros_to_ms(run.end_time.as_micros()),
        event_digest: format!("{:016x}", run.event_log.stable_digest()),
        neurons,
        synapses,
        pacemaker_neuron_ids,
        chunks,
    })
}

fn reconstruct_samples(
    definition: &NetworkDefinition,
    run: &NetworkRun,
) -> Result<Vec<PlaybackNeuronSample>, PlaybackError> {
    let mut models = definition
        .neurons()
        .iter()
        .map(|spec| {
            Ok((
                spec.id,
                LifNeuron::new(
                    spec.id,
                    spec.parameters,
                    spec.initial_potential_mv,
                    definition.start_time(),
                )?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, crate::lif::ModelError>>()?;
    let mut samples = definition
        .neurons()
        .iter()
        .map(|spec| PlaybackNeuronSample {
            neuron_id: spec.id.0,
            time_ms: micros_to_ms(definition.start_time().as_micros()),
            membrane_potential_mv: spec.initial_potential_mv as f32,
            refractory_until_ms: None,
        })
        .collect::<Vec<_>>();

    let inputs = run.event_log.inputs().collect::<Vec<_>>();
    let mut cursor = 0;
    while cursor < inputs.len() {
        let time = inputs[cursor].time;
        let target = inputs[cursor].target;
        let mut end = cursor + 1;
        while end < inputs.len() && inputs[end].time == time && inputs[end].target == target {
            end += 1;
        }
        let batch = inputs[cursor..end]
            .iter()
            .map(|input| {
                TimedInput::new(
                    input.sequence,
                    input.time,
                    input.polarity,
                    input.magnitude_mv,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let model = models
            .get_mut(&target)
            .ok_or(PlaybackError::UnknownNeuron(target))?;
        let result = model.apply_batch(time, &batch)?;
        samples.push(PlaybackNeuronSample {
            neuron_id: target.0,
            time_ms: micros_to_ms(time.as_micros()),
            membrane_potential_mv: result.membrane_potential_after_mv as f32,
            refractory_until_ms: model
                .snapshot()
                .refractory_until
                .map(|value| micros_to_ms(value.as_micros())),
        });
        cursor = end;
    }
    samples.sort_by(|left, right| {
        left.time_ms
            .total_cmp(&right.time_ms)
            .then(left.neuron_id.cmp(&right.neuron_id))
    });
    Ok(samples)
}

fn reconstruct_in_flight(
    definition: &NetworkDefinition,
    spikes: &[PlaybackSpike],
    arrivals: &[PlaybackArrival],
) -> Result<Vec<PlaybackInFlight>, PlaybackError> {
    let spike_time = spikes
        .iter()
        .map(|spike| (spike.id, spike.time_ms))
        .collect::<BTreeMap<_, _>>();
    let synapses = definition
        .synapses()
        .iter()
        .map(|synapse| (synapse.id.0, synapse))
        .collect::<BTreeMap<_, _>>();
    let polarity = definition
        .neurons()
        .iter()
        .map(|neuron| (neuron.id.0, polarity_name(neuron.polarity)))
        .collect::<BTreeMap<_, _>>();

    arrivals
        .iter()
        .filter_map(|arrival| match arrival.origin {
            ArrivalOrigin::Synaptic {
                spike_id,
                synapse_id,
                source,
            } => Some((arrival, spike_id, synapse_id, source)),
            _ => None,
        })
        .map(|(arrival, spike_id, synapse_id, source)| {
            let synapse = synapses
                .get(&synapse_id)
                .ok_or(PlaybackError::UnknownSynapse(synapse_id))?;
            Ok(PlaybackInFlight {
                spike_id,
                synapse_id,
                source,
                target: synapse.target.0,
                send_time_ms: *spike_time
                    .get(&spike_id)
                    .ok_or(PlaybackError::UnknownSpike(spike_id))?,
                arrival_time_ms: arrival.time_ms,
                polarity: polarity[&source],
            })
        })
        .collect()
}

fn build_metric_samples(
    definition: &NetworkDefinition,
    spikes: &[PlaybackSpike],
    start: SimTime,
    end: SimTime,
) -> Vec<PlaybackMetricSample> {
    let bin_micros = 10_000_u64;
    let bin_count = (end.as_micros() - start.as_micros()).div_ceil(bin_micros) as usize;
    let mut counts = vec![0_u32; bin_count];
    let mut active = vec![BTreeSet::<u32>::new(); bin_count];
    for spike in spikes {
        let time_micros = (spike.time_ms * 1_000.0).round() as u64;
        if time_micros >= start.as_micros() && time_micros < end.as_micros() {
            let index = ((time_micros - start.as_micros()) / bin_micros) as usize;
            counts[index] += 1;
            active[index].insert(spike.neuron_id);
        }
    }
    counts
        .into_iter()
        .zip(active)
        .enumerate()
        .map(|(index, (spike_count, neurons))| PlaybackMetricSample {
            time_ms: micros_to_ms(start.as_micros() + index as u64 * bin_micros),
            spike_count,
            active_neuron_count: neurons.len().min(definition.neurons().len()) as u32,
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn chunk_events(
    start: SimTime,
    end: SimTime,
    duration: SimDuration,
    spikes: &[PlaybackSpike],
    arrivals: &[PlaybackArrival],
    samples: &[PlaybackNeuronSample],
    metrics: &[PlaybackMetricSample],
    in_flight: &[PlaybackInFlight],
) -> Vec<PlaybackChunk> {
    let count = (end.as_micros() - start.as_micros())
        .div_ceil(duration.as_micros())
        .max(1) as usize;
    (0..count)
        .map(|index| {
            let chunk_start = start.as_micros() + index as u64 * duration.as_micros();
            let chunk_end = (chunk_start + duration.as_micros()).min(end.as_micros());
            let start_ms = micros_to_ms(chunk_start);
            let end_ms = micros_to_ms(chunk_end);
            let within = |time: f64| {
                time >= start_ms && (time < end_ms || (index + 1 == count && time <= end_ms))
            };
            PlaybackChunk {
                start_ms,
                end_ms,
                spike_events: spikes
                    .iter()
                    .copied()
                    .filter(|event| within(event.time_ms))
                    .collect(),
                arrival_events: arrivals
                    .iter()
                    .copied()
                    .filter(|event| within(event.time_ms))
                    .collect(),
                neuron_samples: samples
                    .iter()
                    .copied()
                    .filter(|sample| within(sample.time_ms))
                    .collect(),
                metric_samples: metrics
                    .iter()
                    .copied()
                    .filter(|sample| within(sample.time_ms))
                    .collect(),
                in_flight_intervals: in_flight
                    .iter()
                    .copied()
                    .filter(|event| {
                        event.send_time_ms <= end_ms && event.arrival_time_ms >= start_ms
                    })
                    .collect(),
            }
        })
        .collect()
}

fn polarity_name(polarity: NeuronPolarity) -> &'static str {
    match polarity {
        NeuronPolarity::Excitatory => "excitatory",
        NeuronPolarity::Inhibitory => "inhibitory",
    }
}

fn input_polarity_name(polarity: crate::lif::InputPolarity) -> &'static str {
    match polarity {
        crate::lif::InputPolarity::Excitatory => "excitatory",
        crate::lif::InputPolarity::Inhibitory => "inhibitory",
    }
}

fn micros_to_ms(micros: u64) -> f64 {
    micros as f64 / 1_000.0
}

#[derive(Clone, Debug, PartialEq)]
pub enum PlaybackError {
    ZeroChunkDuration,
    UnknownNeuron(NeuronId),
    UnknownSynapse(u32),
    UnknownSpike(u64),
    Model(crate::lif::ModelError),
    Network(crate::network::NetworkError),
}

impl Display for PlaybackError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroChunkDuration => {
                formatter.write_str("playback chunk duration must be positive")
            }
            Self::UnknownNeuron(id) => {
                write!(formatter, "playback references unknown neuron {}", id.0)
            }
            Self::UnknownSynapse(id) => {
                write!(formatter, "playback references unknown synapse {id}")
            }
            Self::UnknownSpike(id) => write!(formatter, "playback references unknown spike {id}"),
            Self::Model(error) => Display::fmt(error, formatter),
            Self::Network(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for PlaybackError {}

impl From<crate::lif::ModelError> for PlaybackError {
    fn from(value: crate::lif::ModelError) -> Self {
        Self::Model(value)
    }
}

impl From<crate::network::NetworkError> for PlaybackError {
    fn from(value: crate::network::NetworkError) -> Self {
        Self::Network(value)
    }
}
