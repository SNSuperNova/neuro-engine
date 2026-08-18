use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::lif::{
    EventId, InputPolarity, LifNeuron, LifParameters, ModelError, NeuronId, NeuronSnapshot,
    SimDuration, SimTime, TimedInput,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Position3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NeuronPolarity {
    Excitatory,
    Inhibitory,
}

impl NeuronPolarity {
    const fn as_input_polarity(self) -> InputPolarity {
        match self {
            Self::Excitatory => InputPolarity::Excitatory,
            Self::Inhibitory => InputPolarity::Inhibitory,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NeuronSpec {
    pub id: NeuronId,
    pub polarity: NeuronPolarity,
    pub position: Position3,
    pub parameters: LifParameters,
    pub initial_potential_mv: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SynapseId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PropagationSpec {
    pub path_length: f64,
    pub conduction_velocity_units_per_ms: f64,
    pub synaptic_delay: SimDuration,
}

impl PropagationSpec {
    pub fn resolved_delay(self) -> Result<SimDuration, NetworkError> {
        if !self.path_length.is_finite() || self.path_length < 0.0 {
            return Err(NetworkError::InvalidPathLength(self.path_length));
        }
        if !self.conduction_velocity_units_per_ms.is_finite()
            || self.conduction_velocity_units_per_ms <= 0.0
        {
            return Err(NetworkError::InvalidConductionVelocity(
                self.conduction_velocity_units_per_ms,
            ));
        }

        let travel_micros =
            (self.path_length / self.conduction_velocity_units_per_ms * 1_000.0).round();
        if !(0.0..=u64::MAX as f64).contains(&travel_micros) {
            return Err(NetworkError::PropagationDelayOverflow);
        }
        let total = (travel_micros as u64)
            .checked_add(self.synaptic_delay.as_micros())
            .ok_or(NetworkError::PropagationDelayOverflow)?;
        if total == 0 {
            return Err(NetworkError::ZeroPropagationDelay);
        }
        Ok(SimDuration::from_micros(total))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SynapseSpec {
    pub id: SynapseId,
    pub source: NeuronId,
    pub target: NeuronId,
    pub magnitude_mv: f64,
    pub propagation: PropagationSpec,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExternalInputKind {
    Initialization,
    Pacemaker,
    Stimulus,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExternalInput {
    pub id: EventId,
    pub target: NeuronId,
    pub time: SimTime,
    pub polarity: InputPolarity,
    pub magnitude_mv: f64,
    pub kind: ExternalInputKind,
}

impl ExternalInput {
    pub const fn new(
        id: EventId,
        target: NeuronId,
        time: SimTime,
        polarity: InputPolarity,
        magnitude_mv: f64,
        kind: ExternalInputKind,
    ) -> Self {
        Self {
            id,
            target,
            time,
            polarity,
            magnitude_mv,
            kind,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NetworkDefinition {
    start_time: SimTime,
    neurons: Vec<NeuronSpec>,
    synapses: Vec<SynapseSpec>,
}

impl NetworkDefinition {
    pub fn new(
        start_time: SimTime,
        mut neurons: Vec<NeuronSpec>,
        mut synapses: Vec<SynapseSpec>,
    ) -> Result<Self, NetworkError> {
        if neurons.is_empty() {
            return Err(NetworkError::EmptyNetwork);
        }
        neurons.sort_by_key(|neuron| neuron.id);
        synapses.sort_by_key(|synapse| synapse.id);

        let mut neuron_ids = BTreeSet::new();
        for neuron in &neurons {
            if !neuron_ids.insert(neuron.id) {
                return Err(NetworkError::DuplicateNeuronId(neuron.id));
            }
            if !neuron.position.is_finite() {
                return Err(NetworkError::InvalidPosition(neuron.id));
            }
            LifNeuron::new(
                neuron.id,
                neuron.parameters,
                neuron.initial_potential_mv,
                start_time,
            )
            .map_err(NetworkError::Model)?;
        }

        let mut synapse_ids = BTreeSet::new();
        for synapse in &synapses {
            if !synapse_ids.insert(synapse.id) {
                return Err(NetworkError::DuplicateSynapseId(synapse.id));
            }
            if !neuron_ids.contains(&synapse.source) {
                return Err(NetworkError::UnknownSourceNeuron {
                    synapse_id: synapse.id,
                    neuron_id: synapse.source,
                });
            }
            if !neuron_ids.contains(&synapse.target) {
                return Err(NetworkError::UnknownTargetNeuron {
                    synapse_id: synapse.id,
                    neuron_id: synapse.target,
                });
            }
            if !synapse.magnitude_mv.is_finite() || synapse.magnitude_mv <= 0.0 {
                return Err(NetworkError::InvalidSynapseMagnitude {
                    synapse_id: synapse.id,
                    magnitude_mv: synapse.magnitude_mv,
                });
            }
            synapse.propagation.resolved_delay().map_err(|reason| {
                NetworkError::InvalidPropagation {
                    synapse_id: synapse.id,
                    reason: Box::new(reason),
                }
            })?;
        }

        Ok(Self {
            start_time,
            neurons,
            synapses,
        })
    }

    pub const fn start_time(&self) -> SimTime {
        self.start_time
    }

    pub fn neurons(&self) -> &[NeuronSpec] {
        &self.neurons
    }

    pub fn synapses(&self) -> &[SynapseSpec] {
        &self.synapses
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpikeId(pub u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputOrigin {
    External {
        external_event_id: EventId,
        kind: ExternalInputKind,
    },
    Synaptic {
        spike_id: SpikeId,
        synapse_id: SynapseId,
        source: NeuronId,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct InputRecord {
    pub sequence: EventId,
    pub time: SimTime,
    pub target: NeuronId,
    pub polarity: InputPolarity,
    pub magnitude_mv: f64,
    pub origin: InputOrigin,
    pub ignored_during_refractory: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NetworkSpike {
    pub id: SpikeId,
    pub neuron_id: NeuronId,
    pub time: SimTime,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegulationRecord {
    pub episode_id: u64,
    pub time: SimTime,
    pub trigger_neuron_id: NeuronId,
    pub unique_spike_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LogEvent {
    Input(InputRecord),
    Spike(NetworkSpike),
    Regulation(RegulationRecord),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EventLog {
    pub events: Vec<LogEvent>,
}

impl EventLog {
    pub fn inputs(&self) -> impl Iterator<Item = &InputRecord> {
        self.events.iter().filter_map(|event| match event {
            LogEvent::Input(input) => Some(input),
            LogEvent::Spike(_) | LogEvent::Regulation(_) => None,
        })
    }

    pub fn spikes(&self) -> impl Iterator<Item = &NetworkSpike> {
        self.events.iter().filter_map(|event| match event {
            LogEvent::Input(_) | LogEvent::Regulation(_) => None,
            LogEvent::Spike(spike) => Some(spike),
        })
    }

    pub fn regulations(&self) -> impl Iterator<Item = &RegulationRecord> {
        self.events.iter().filter_map(|event| match event {
            LogEvent::Regulation(record) => Some(record),
            LogEvent::Input(_) | LogEvent::Spike(_) => None,
        })
    }

    /// Stable FNV-1a digest over the canonical event representation.
    pub fn stable_digest(&self) -> u64 {
        let mut digest = 0xcbf2_9ce4_8422_2325_u64;
        mix_u64(&mut digest, 1); // digest format version
        for event in &self.events {
            match event {
                LogEvent::Input(input) => {
                    mix_u64(&mut digest, 1);
                    mix_u64(&mut digest, input.sequence.0);
                    mix_u64(&mut digest, input.time.as_micros());
                    mix_u64(&mut digest, u64::from(input.target.0));
                    mix_u64(
                        &mut digest,
                        match input.polarity {
                            InputPolarity::Excitatory => 1,
                            InputPolarity::Inhibitory => 2,
                        },
                    );
                    mix_u64(&mut digest, input.magnitude_mv.to_bits());
                    match input.origin {
                        InputOrigin::External {
                            external_event_id,
                            kind,
                        } => {
                            mix_u64(&mut digest, 1);
                            mix_u64(&mut digest, external_event_id.0);
                            mix_u64(
                                &mut digest,
                                match kind {
                                    ExternalInputKind::Initialization => 1,
                                    ExternalInputKind::Pacemaker => 2,
                                    ExternalInputKind::Stimulus => 3,
                                },
                            );
                        }
                        InputOrigin::Synaptic {
                            spike_id,
                            synapse_id,
                            source,
                        } => {
                            mix_u64(&mut digest, 2);
                            mix_u64(&mut digest, spike_id.0);
                            mix_u64(&mut digest, u64::from(synapse_id.0));
                            mix_u64(&mut digest, u64::from(source.0));
                        }
                    }
                    mix_u64(&mut digest, u64::from(input.ignored_during_refractory));
                }
                LogEvent::Spike(spike) => {
                    mix_u64(&mut digest, 2);
                    mix_u64(&mut digest, spike.id.0);
                    mix_u64(&mut digest, u64::from(spike.neuron_id.0));
                    mix_u64(&mut digest, spike.time.as_micros());
                }
                LogEvent::Regulation(record) => {
                    mix_u64(&mut digest, 3);
                    mix_u64(&mut digest, record.episode_id);
                    mix_u64(&mut digest, record.time.as_micros());
                    mix_u64(&mut digest, u64::from(record.trigger_neuron_id.0));
                    mix_u64(&mut digest, record.unique_spike_count as u64);
                }
            }
        }
        digest
    }
}

fn mix_u64(digest: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        *digest ^= u64::from(byte);
        *digest = digest.wrapping_mul(0x0000_0100_0000_01b3);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NetworkRun {
    pub start_time: SimTime,
    pub end_time: SimTime,
    pub final_states: Vec<NeuronSnapshot>,
    pub event_log: EventLog,
    pub max_in_flight_inputs: usize,
    pub in_flight_inputs_at_end: usize,
    pub regulation_episode_count: usize,
    pub suppressed_propagation_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SimulationLimits {
    pub maximum_processed_inputs: usize,
    pub maximum_queued_inputs: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActivityRegulatorConfig {
    pub window: SimDuration,
    pub unique_spike_threshold: usize,
    pub cooldown: SimDuration,
}

impl Default for ActivityRegulatorConfig {
    fn default() -> Self {
        Self {
            window: SimDuration::from_micros(10_000),
            unique_spike_threshold: 18,
            cooldown: SimDuration::from_micros(5_000),
        }
    }
}

impl Default for SimulationLimits {
    fn default() -> Self {
        Self {
            maximum_processed_inputs: 10_000_000,
            maximum_queued_inputs: 1_000_000,
        }
    }
}

#[derive(Clone, Debug)]
struct ScheduledInput {
    sequence: EventId,
    time: SimTime,
    target: NeuronId,
    polarity: InputPolarity,
    magnitude_mv: f64,
    origin: InputOrigin,
}

pub fn simulate_network(
    definition: &NetworkDefinition,
    external_inputs: &[ExternalInput],
    end_time: SimTime,
) -> Result<NetworkRun, NetworkError> {
    simulate_network_internal(
        definition,
        external_inputs,
        end_time,
        SimulationLimits::default(),
        None,
    )
}

pub fn simulate_network_with_regulator(
    definition: &NetworkDefinition,
    external_inputs: &[ExternalInput],
    end_time: SimTime,
    regulator: ActivityRegulatorConfig,
) -> Result<NetworkRun, NetworkError> {
    simulate_network_internal(
        definition,
        external_inputs,
        end_time,
        SimulationLimits::default(),
        Some(regulator),
    )
}

pub fn simulate_network_with_limits(
    definition: &NetworkDefinition,
    external_inputs: &[ExternalInput],
    end_time: SimTime,
    limits: SimulationLimits,
) -> Result<NetworkRun, NetworkError> {
    simulate_network_internal(definition, external_inputs, end_time, limits, None)
}

fn simulate_network_internal(
    definition: &NetworkDefinition,
    external_inputs: &[ExternalInput],
    end_time: SimTime,
    limits: SimulationLimits,
    regulator: Option<ActivityRegulatorConfig>,
) -> Result<NetworkRun, NetworkError> {
    if end_time < definition.start_time {
        return Err(NetworkError::EndBeforeStart {
            start_time: definition.start_time,
            end_time,
        });
    }
    if limits.maximum_processed_inputs == 0 || limits.maximum_queued_inputs == 0 {
        return Err(NetworkError::InvalidSimulationLimits);
    }
    if let Some(config) = regulator
        && (config.window.as_micros() == 0
            || config.unique_spike_threshold == 0
            || config.cooldown.as_micros() == 0)
    {
        return Err(NetworkError::InvalidActivityRegulator);
    }

    let mut id_to_index = BTreeMap::new();
    let mut neurons = Vec::with_capacity(definition.neurons.len());
    for (index, spec) in definition.neurons.iter().enumerate() {
        id_to_index.insert(spec.id, index);
        neurons.push(
            LifNeuron::new(
                spec.id,
                spec.parameters,
                spec.initial_potential_mv,
                definition.start_time,
            )
            .map_err(NetworkError::Model)?,
        );
    }

    let mut outgoing = vec![Vec::new(); neurons.len()];
    for synapse in &definition.synapses {
        let source_index = id_to_index[&synapse.source];
        outgoing[source_index].push(synapse);
    }

    let mut external = external_inputs.iter().collect::<Vec<_>>();
    external.sort_by_key(|input| (input.time, input.target, input.id));
    let mut external_ids = BTreeSet::new();
    let mut queue = BTreeMap::<SimTime, Vec<ScheduledInput>>::new();
    let mut next_sequence = 0_u64;
    for input in external {
        if !external_ids.insert(input.id) {
            return Err(NetworkError::DuplicateExternalEventId(input.id));
        }
        if !id_to_index.contains_key(&input.target) {
            return Err(NetworkError::UnknownExternalTarget {
                event_id: input.id,
                neuron_id: input.target,
            });
        }
        if input.time < definition.start_time {
            return Err(NetworkError::ExternalInputBeforeStart {
                event_id: input.id,
                input_time: input.time,
                start_time: definition.start_time,
            });
        }
        TimedInput::new(input.id, input.time, input.polarity, input.magnitude_mv)
            .map_err(NetworkError::Model)?;

        let sequence = allocate_event_id(&mut next_sequence)?;
        queue.entry(input.time).or_default().push(ScheduledInput {
            sequence,
            time: input.time,
            target: input.target,
            polarity: input.polarity,
            magnitude_mv: input.magnitude_mv,
            origin: InputOrigin::External {
                external_event_id: input.id,
                kind: input.kind,
            },
        });
    }

    let mut queued_count = external_inputs.len();
    if queued_count > limits.maximum_queued_inputs {
        return Err(NetworkError::QueuedInputLimitExceeded {
            limit: limits.maximum_queued_inputs,
        });
    }
    let mut in_flight_input_count = 0_usize;
    let mut max_in_flight_inputs = 0_usize;
    let mut processed_input_count = 0_usize;
    let mut next_spike_id = 0_u64;
    let mut event_log = EventLog::default();
    let mut recent_spikes = VecDeque::<(SimTime, NeuronId)>::new();
    let mut regulation_until = None::<SimTime>;
    let mut next_regulation_episode = 0_u64;
    let mut suppressed_propagation_count = 0_usize;

    while let Some((time, mut scheduled)) = queue.pop_first() {
        if time > end_time {
            queue.insert(time, scheduled);
            break;
        }
        queued_count -= scheduled.len();
        in_flight_input_count -= scheduled
            .iter()
            .filter(|input| matches!(input.origin, InputOrigin::Synaptic { .. }))
            .count();
        processed_input_count = processed_input_count.checked_add(scheduled.len()).ok_or(
            NetworkError::ProcessedInputLimitExceeded {
                limit: limits.maximum_processed_inputs,
            },
        )?;
        if processed_input_count > limits.maximum_processed_inputs {
            return Err(NetworkError::ProcessedInputLimitExceeded {
                limit: limits.maximum_processed_inputs,
            });
        }
        scheduled.sort_by_key(|input| (input.target, input.sequence));

        let mut start = 0;
        while start < scheduled.len() {
            let target = scheduled[start].target;
            let mut end = start + 1;
            while end < scheduled.len() && scheduled[end].target == target {
                end += 1;
            }

            let inputs = scheduled[start..end]
                .iter()
                .map(|input| {
                    TimedInput::new(
                        input.sequence,
                        input.time,
                        input.polarity,
                        input.magnitude_mv,
                    )
                })
                .collect::<Result<Vec<_>, _>>()
                .map_err(NetworkError::Model)?;
            let neuron_index = id_to_index[&target];
            let result = neurons[neuron_index]
                .apply_batch(time, &inputs)
                .map_err(NetworkError::Model)?;
            let ignored = result.ignored_input_count > 0;

            for input in &scheduled[start..end] {
                event_log.events.push(LogEvent::Input(InputRecord {
                    sequence: input.sequence,
                    time: input.time,
                    target: input.target,
                    polarity: input.polarity,
                    magnitude_mv: input.magnitude_mv,
                    origin: input.origin.clone(),
                    ignored_during_refractory: ignored,
                }));
            }

            if result.spike.is_some() {
                let spike_id = allocate_spike_id(&mut next_spike_id)?;
                let spike = NetworkSpike {
                    id: spike_id,
                    neuron_id: target,
                    time,
                };
                event_log.events.push(LogEvent::Spike(spike));

                let mut suppress_propagation = false;
                if let Some(regulator) = regulator {
                    recent_spikes.push_back((time, target));
                    while recent_spikes.front().is_some_and(|(spike_time, _)| {
                        time.as_micros().saturating_sub(spike_time.as_micros())
                            > regulator.window.as_micros()
                    }) {
                        recent_spikes.pop_front();
                    }
                    let unique_spikes = recent_spikes
                        .iter()
                        .map(|(_, neuron_id)| *neuron_id)
                        .collect::<BTreeSet<_>>()
                        .len();
                    suppress_propagation = regulation_until.is_some_and(|until| time < until);
                    if !suppress_propagation && unique_spikes >= regulator.unique_spike_threshold {
                        suppress_propagation = true;
                        regulation_until = Some(
                            time.checked_add(regulator.cooldown)
                                .map_err(NetworkError::Model)?,
                        );
                        event_log
                            .events
                            .push(LogEvent::Regulation(RegulationRecord {
                                episode_id: next_regulation_episode,
                                time,
                                trigger_neuron_id: target,
                                unique_spike_count: unique_spikes,
                            }));
                        next_regulation_episode = next_regulation_episode
                            .checked_add(1)
                            .ok_or(NetworkError::EventSequenceOverflow)?;
                        recent_spikes.clear();
                    }
                }

                if suppress_propagation {
                    suppressed_propagation_count += outgoing[neuron_index].len();
                } else {
                    let source_polarity = definition.neurons[neuron_index]
                        .polarity
                        .as_input_polarity();
                    for synapse in &outgoing[neuron_index] {
                        let arrival_time = time
                            .checked_add(synapse.propagation.resolved_delay().map_err(
                                |reason| NetworkError::InvalidPropagation {
                                    synapse_id: synapse.id,
                                    reason: Box::new(reason),
                                },
                            )?)
                            .map_err(NetworkError::Model)?;
                        let sequence = allocate_event_id(&mut next_sequence)?;
                        queue.entry(arrival_time).or_default().push(ScheduledInput {
                            sequence,
                            time: arrival_time,
                            target: synapse.target,
                            polarity: source_polarity,
                            magnitude_mv: synapse.magnitude_mv,
                            origin: InputOrigin::Synaptic {
                                spike_id,
                                synapse_id: synapse.id,
                                source: synapse.source,
                            },
                        });
                        queued_count += 1;
                        in_flight_input_count += 1;
                        if queued_count > limits.maximum_queued_inputs {
                            return Err(NetworkError::QueuedInputLimitExceeded {
                                limit: limits.maximum_queued_inputs,
                            });
                        }
                    }
                }
                max_in_flight_inputs = max_in_flight_inputs.max(in_flight_input_count);
            }

            start = end;
        }
    }

    for neuron in &mut neurons {
        neuron.advance_to(end_time).map_err(NetworkError::Model)?;
    }

    Ok(NetworkRun {
        start_time: definition.start_time,
        end_time,
        final_states: neurons.iter().map(LifNeuron::snapshot).collect(),
        event_log,
        max_in_flight_inputs,
        in_flight_inputs_at_end: in_flight_input_count,
        regulation_episode_count: next_regulation_episode as usize,
        suppressed_propagation_count,
    })
}

fn allocate_event_id(next: &mut u64) -> Result<EventId, NetworkError> {
    let id = *next;
    *next = next
        .checked_add(1)
        .ok_or(NetworkError::EventSequenceOverflow)?;
    Ok(EventId(id))
}

fn allocate_spike_id(next: &mut u64) -> Result<SpikeId, NetworkError> {
    let id = *next;
    *next = next
        .checked_add(1)
        .ok_or(NetworkError::SpikeSequenceOverflow)?;
    Ok(SpikeId(id))
}

#[derive(Clone, Debug, PartialEq)]
pub enum NetworkError {
    EmptyNetwork,
    DuplicateNeuronId(NeuronId),
    DuplicateSynapseId(SynapseId),
    InvalidPosition(NeuronId),
    UnknownSourceNeuron {
        synapse_id: SynapseId,
        neuron_id: NeuronId,
    },
    UnknownTargetNeuron {
        synapse_id: SynapseId,
        neuron_id: NeuronId,
    },
    InvalidSynapseMagnitude {
        synapse_id: SynapseId,
        magnitude_mv: f64,
    },
    InvalidPropagation {
        synapse_id: SynapseId,
        reason: Box<NetworkError>,
    },
    InvalidPathLength(f64),
    InvalidConductionVelocity(f64),
    PropagationDelayOverflow,
    ZeroPropagationDelay,
    DuplicateExternalEventId(EventId),
    UnknownExternalTarget {
        event_id: EventId,
        neuron_id: NeuronId,
    },
    ExternalInputBeforeStart {
        event_id: EventId,
        input_time: SimTime,
        start_time: SimTime,
    },
    EndBeforeStart {
        start_time: SimTime,
        end_time: SimTime,
    },
    EventSequenceOverflow,
    SpikeSequenceOverflow,
    InvalidSimulationLimits,
    InvalidActivityRegulator,
    ProcessedInputLimitExceeded {
        limit: usize,
    },
    QueuedInputLimitExceeded {
        limit: usize,
    },
    Model(ModelError),
}

impl Display for NetworkError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyNetwork => formatter.write_str("network must contain at least one neuron"),
            Self::DuplicateNeuronId(id) => write!(formatter, "duplicate neuron id {}", id.0),
            Self::DuplicateSynapseId(id) => write!(formatter, "duplicate synapse id {}", id.0),
            Self::InvalidPosition(id) => {
                write!(formatter, "neuron {} has a non-finite position", id.0)
            }
            Self::UnknownSourceNeuron {
                synapse_id,
                neuron_id,
            } => write!(
                formatter,
                "synapse {} references unknown source neuron {}",
                synapse_id.0, neuron_id.0
            ),
            Self::UnknownTargetNeuron {
                synapse_id,
                neuron_id,
            } => write!(
                formatter,
                "synapse {} references unknown target neuron {}",
                synapse_id.0, neuron_id.0
            ),
            Self::InvalidSynapseMagnitude {
                synapse_id,
                magnitude_mv,
            } => write!(
                formatter,
                "synapse {} magnitude must be finite and positive, got {} mV",
                synapse_id.0, magnitude_mv
            ),
            Self::InvalidPropagation { synapse_id, reason } => write!(
                formatter,
                "synapse {} has invalid propagation parameters: {}",
                synapse_id.0, reason
            ),
            Self::InvalidPathLength(length) => write!(
                formatter,
                "path length must be finite and non-negative, got {length}"
            ),
            Self::InvalidConductionVelocity(velocity) => write!(
                formatter,
                "conduction velocity must be finite and positive, got {velocity}"
            ),
            Self::PropagationDelayOverflow => formatter.write_str("propagation delay overflow"),
            Self::ZeroPropagationDelay => {
                formatter.write_str("total propagation delay must be greater than zero")
            }
            Self::DuplicateExternalEventId(id) => {
                write!(formatter, "duplicate external event id {}", id.0)
            }
            Self::UnknownExternalTarget {
                event_id,
                neuron_id,
            } => write!(
                formatter,
                "external event {} targets unknown neuron {}",
                event_id.0, neuron_id.0
            ),
            Self::ExternalInputBeforeStart {
                event_id,
                input_time,
                start_time,
            } => write!(
                formatter,
                "external event {} at {} us precedes network start {} us",
                event_id.0,
                input_time.as_micros(),
                start_time.as_micros()
            ),
            Self::EndBeforeStart {
                start_time,
                end_time,
            } => write!(
                formatter,
                "simulation end {} us precedes start {} us",
                end_time.as_micros(),
                start_time.as_micros()
            ),
            Self::EventSequenceOverflow => formatter.write_str("input event sequence overflow"),
            Self::SpikeSequenceOverflow => formatter.write_str("spike sequence overflow"),
            Self::InvalidSimulationLimits => {
                formatter.write_str("simulation limits must be greater than zero")
            }
            Self::InvalidActivityRegulator => {
                formatter.write_str("activity regulator parameters must be greater than zero")
            }
            Self::ProcessedInputLimitExceeded { limit } => {
                write!(formatter, "processed input limit exceeded ({limit})")
            }
            Self::QueuedInputLimitExceeded { limit } => {
                write!(formatter, "queued input limit exceeded ({limit})")
            }
            Self::Model(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for NetworkError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidPropagation { reason, .. } => Some(reason),
            Self::Model(error) => Some(error),
            _ => None,
        }
    }
}
