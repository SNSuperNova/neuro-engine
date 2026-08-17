use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// Integer simulation time in microseconds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimTime(u64);

impl SimTime {
    pub const ZERO: Self = Self(0);

    pub const fn from_micros(micros: u64) -> Self {
        Self(micros)
    }

    pub const fn as_micros(self) -> u64 {
        self.0
    }

    pub fn checked_add(self, duration: SimDuration) -> Result<Self, ModelError> {
        self.0
            .checked_add(duration.0)
            .map(Self)
            .ok_or(ModelError::TimeOverflow)
    }
}

/// Integer simulation duration in microseconds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimDuration(u64);

impl SimDuration {
    pub const fn from_micros(micros: u64) -> Self {
        Self(micros)
    }

    pub const fn as_micros(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NeuronId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputPolarity {
    Excitatory,
    Inhibitory,
}

/// A validated input event. Magnitudes are always non-negative; polarity
/// determines whether the voltage delta is positive or negative.
#[derive(Clone, Debug, PartialEq)]
pub struct TimedInput {
    id: EventId,
    time: SimTime,
    polarity: InputPolarity,
    magnitude_mv: f64,
}

impl TimedInput {
    pub fn new(
        id: EventId,
        time: SimTime,
        polarity: InputPolarity,
        magnitude_mv: f64,
    ) -> Result<Self, ModelError> {
        if !magnitude_mv.is_finite() || magnitude_mv < 0.0 {
            return Err(ModelError::InvalidInputMagnitude { magnitude_mv });
        }

        Ok(Self {
            id,
            time,
            polarity,
            magnitude_mv,
        })
    }

    pub fn excitatory(id: EventId, time: SimTime, magnitude_mv: f64) -> Result<Self, ModelError> {
        Self::new(id, time, InputPolarity::Excitatory, magnitude_mv)
    }

    pub fn inhibitory(id: EventId, time: SimTime, magnitude_mv: f64) -> Result<Self, ModelError> {
        Self::new(id, time, InputPolarity::Inhibitory, magnitude_mv)
    }

    pub const fn id(&self) -> EventId {
        self.id
    }

    pub const fn time(&self) -> SimTime {
        self.time
    }

    pub const fn polarity(&self) -> InputPolarity {
        self.polarity
    }

    pub const fn magnitude_mv(&self) -> f64 {
        self.magnitude_mv
    }

    fn signed_delta_mv(&self) -> f64 {
        match self.polarity {
            InputPolarity::Excitatory => self.magnitude_mv,
            InputPolarity::Inhibitory => -self.magnitude_mv,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LifParameters {
    pub rest_potential_mv: f64,
    pub reset_potential_mv: f64,
    pub threshold_mv: f64,
    pub membrane_time_constant_ms: f64,
    pub refractory_period: SimDuration,
}

impl LifParameters {
    pub fn validate(self) -> Result<Self, ModelError> {
        for (name, value) in [
            ("rest_potential_mv", self.rest_potential_mv),
            ("reset_potential_mv", self.reset_potential_mv),
            ("threshold_mv", self.threshold_mv),
            ("membrane_time_constant_ms", self.membrane_time_constant_ms),
        ] {
            if !value.is_finite() {
                return Err(ModelError::NonFiniteParameter { name, value });
            }
        }

        if self.membrane_time_constant_ms <= 0.0 {
            return Err(ModelError::NonPositiveTimeConstant {
                value_ms: self.membrane_time_constant_ms,
            });
        }
        if self.refractory_period.as_micros() == 0 {
            return Err(ModelError::ZeroRefractoryPeriod);
        }
        if self.rest_potential_mv >= self.threshold_mv {
            return Err(ModelError::PotentialNotBelowThreshold {
                name: "rest_potential_mv",
                value_mv: self.rest_potential_mv,
                threshold_mv: self.threshold_mv,
            });
        }
        if self.reset_potential_mv >= self.threshold_mv {
            return Err(ModelError::PotentialNotBelowThreshold {
                name: "reset_potential_mv",
                value_mv: self.reset_potential_mv,
                threshold_mv: self.threshold_mv,
            });
        }

        Ok(self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NeuronSnapshot {
    pub neuron_id: NeuronId,
    pub time: SimTime,
    pub membrane_potential_mv: f64,
    pub refractory_until: Option<SimTime>,
}

impl NeuronSnapshot {
    pub fn is_refractory(&self) -> bool {
        self.refractory_until.is_some_and(|end| self.time < end)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpikeEvent {
    pub neuron_id: NeuronId,
    pub time: SimTime,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BatchResult {
    pub time: SimTime,
    pub input_event_ids: Vec<EventId>,
    pub potential_before_input_mv: f64,
    pub applied_delta_mv: f64,
    pub integrated_potential_mv: f64,
    pub membrane_potential_after_mv: f64,
    pub ignored_input_count: usize,
    pub spike: Option<SpikeEvent>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SimulationTrace {
    pub batches: Vec<BatchResult>,
    pub spikes: Vec<SpikeEvent>,
    pub final_state: NeuronSnapshot,
}

#[derive(Clone, Debug)]
pub struct LifNeuron {
    id: NeuronId,
    parameters: LifParameters,
    membrane_potential_mv: f64,
    last_update_time: SimTime,
    last_input_time: Option<SimTime>,
    refractory_until: Option<SimTime>,
}

impl LifNeuron {
    pub fn new(
        id: NeuronId,
        parameters: LifParameters,
        initial_potential_mv: f64,
        start_time: SimTime,
    ) -> Result<Self, ModelError> {
        let parameters = parameters.validate()?;
        if !initial_potential_mv.is_finite() {
            return Err(ModelError::NonFiniteInitialPotential {
                value_mv: initial_potential_mv,
            });
        }
        if initial_potential_mv >= parameters.threshold_mv {
            return Err(ModelError::InitialPotentialAtOrAboveThreshold {
                value_mv: initial_potential_mv,
                threshold_mv: parameters.threshold_mv,
            });
        }

        Ok(Self {
            id,
            parameters,
            membrane_potential_mv: initial_potential_mv,
            last_update_time: start_time,
            last_input_time: None,
            refractory_until: None,
        })
    }

    pub const fn id(&self) -> NeuronId {
        self.id
    }

    pub const fn parameters(&self) -> LifParameters {
        self.parameters
    }

    pub fn snapshot(&self) -> NeuronSnapshot {
        NeuronSnapshot {
            neuron_id: self.id,
            time: self.last_update_time,
            membrane_potential_mv: self.membrane_potential_mv,
            refractory_until: self.refractory_until,
        }
    }

    /// Advances the neuron analytically without inventing simulation ticks.
    /// During the absolute refractory period the voltage is held at reset.
    pub fn advance_to(&mut self, time: SimTime) -> Result<NeuronSnapshot, ModelError> {
        if time < self.last_update_time {
            return Err(ModelError::TimeWentBackwards {
                current: self.last_update_time,
                requested: time,
            });
        }

        let decay_start = match self.refractory_until {
            Some(end) if time < end => {
                self.last_update_time = time;
                return Ok(self.snapshot());
            }
            Some(end) => {
                self.refractory_until = None;
                end
            }
            None => self.last_update_time,
        };

        let elapsed_micros = time.as_micros() - decay_start.as_micros();
        let elapsed_ms = elapsed_micros as f64 / 1_000.0;
        let decay = (-elapsed_ms / self.parameters.membrane_time_constant_ms).exp();
        self.membrane_potential_mv = self.parameters.rest_potential_mv
            + (self.membrane_potential_mv - self.parameters.rest_potential_mv) * decay;
        self.last_update_time = time;

        Ok(self.snapshot())
    }

    /// Applies every input arriving at exactly `time` as one deterministic
    /// batch, then performs at most one threshold check.
    pub fn apply_batch(
        &mut self,
        time: SimTime,
        inputs: &[TimedInput],
    ) -> Result<BatchResult, ModelError> {
        let mut next = self.clone();
        let result = next.apply_batch_in_place(time, inputs)?;
        *self = next;
        Ok(result)
    }

    fn apply_batch_in_place(
        &mut self,
        time: SimTime,
        inputs: &[TimedInput],
    ) -> Result<BatchResult, ModelError> {
        if inputs.is_empty() {
            return Err(ModelError::EmptyInputBatch);
        }
        if self.last_input_time == Some(time) {
            return Err(ModelError::DuplicateInputBatchTime(time));
        }

        let mut ordered = inputs.iter().collect::<Vec<_>>();
        ordered.sort_by_key(|input| input.id());

        let mut previous_id = None;
        for input in &ordered {
            if input.time() != time {
                return Err(ModelError::InputTimeDoesNotMatchBatch {
                    event_id: input.id(),
                    input_time: input.time(),
                    batch_time: time,
                });
            }
            if previous_id == Some(input.id()) {
                return Err(ModelError::DuplicateEventId(input.id()));
            }
            previous_id = Some(input.id());
        }

        self.advance_to(time)?;
        self.last_input_time = Some(time);
        let potential_before_input_mv = self.membrane_potential_mv;
        let input_event_ids = ordered.iter().map(|input| input.id()).collect::<Vec<_>>();

        if self.refractory_until.is_some_and(|end| time < end) {
            return Ok(BatchResult {
                time,
                input_event_ids,
                potential_before_input_mv,
                applied_delta_mv: 0.0,
                integrated_potential_mv: potential_before_input_mv,
                membrane_potential_after_mv: potential_before_input_mv,
                ignored_input_count: ordered.len(),
                spike: None,
            });
        }

        let applied_delta_mv = ordered
            .iter()
            .fold(0.0, |sum, input| sum + input.signed_delta_mv());
        if !applied_delta_mv.is_finite() {
            return Err(ModelError::NonFiniteInputSum);
        }
        let integrated_potential_mv = potential_before_input_mv + applied_delta_mv;
        if !integrated_potential_mv.is_finite() {
            return Err(ModelError::NonFiniteIntegratedPotential);
        }
        self.membrane_potential_mv = integrated_potential_mv;

        let spike = if integrated_potential_mv >= self.parameters.threshold_mv {
            let refractory_until = time.checked_add(self.parameters.refractory_period)?;
            let spike = SpikeEvent {
                neuron_id: self.id,
                time,
            };
            self.membrane_potential_mv = self.parameters.reset_potential_mv;
            self.refractory_until = Some(refractory_until);
            Some(spike)
        } else {
            None
        };

        Ok(BatchResult {
            time,
            input_event_ids,
            potential_before_input_mv,
            applied_delta_mv,
            integrated_potential_mv,
            membrane_potential_after_mv: self.membrane_potential_mv,
            ignored_input_count: 0,
            spike,
        })
    }
}

/// Runs a canonical single-neuron trace. Input order supplied by the caller is
/// irrelevant: events are sorted by `(time, event_id)` and grouped by time.
pub fn simulate_neuron(
    mut neuron: LifNeuron,
    inputs: &[TimedInput],
) -> Result<SimulationTrace, ModelError> {
    let mut ids = BTreeSet::new();
    for input in inputs {
        if !ids.insert(input.id()) {
            return Err(ModelError::DuplicateEventId(input.id()));
        }
        if input.time() < neuron.last_update_time {
            return Err(ModelError::InputBeforeSimulationStart {
                event_id: input.id(),
                input_time: input.time(),
                start_time: neuron.last_update_time,
            });
        }
    }

    let mut ordered = inputs.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|input| (input.time(), input.id()));

    let mut batches = Vec::new();
    let mut spikes = Vec::new();
    let mut start = 0;
    while start < ordered.len() {
        let time = ordered[start].time();
        let mut end = start + 1;
        while end < ordered.len() && ordered[end].time() == time {
            end += 1;
        }

        let batch_inputs = ordered[start..end]
            .iter()
            .map(|input| (*input).clone())
            .collect::<Vec<_>>();
        let result = neuron.apply_batch(time, &batch_inputs)?;
        if let Some(spike) = result.spike {
            spikes.push(spike);
        }
        batches.push(result);
        start = end;
    }

    Ok(SimulationTrace {
        batches,
        spikes,
        final_state: neuron.snapshot(),
    })
}

#[derive(Clone, Debug, PartialEq)]
pub enum ModelError {
    NonFiniteParameter {
        name: &'static str,
        value: f64,
    },
    NonPositiveTimeConstant {
        value_ms: f64,
    },
    ZeroRefractoryPeriod,
    PotentialNotBelowThreshold {
        name: &'static str,
        value_mv: f64,
        threshold_mv: f64,
    },
    NonFiniteInitialPotential {
        value_mv: f64,
    },
    InitialPotentialAtOrAboveThreshold {
        value_mv: f64,
        threshold_mv: f64,
    },
    InvalidInputMagnitude {
        magnitude_mv: f64,
    },
    TimeWentBackwards {
        current: SimTime,
        requested: SimTime,
    },
    TimeOverflow,
    NonFiniteInputSum,
    NonFiniteIntegratedPotential,
    EmptyInputBatch,
    DuplicateInputBatchTime(SimTime),
    DuplicateEventId(EventId),
    InputTimeDoesNotMatchBatch {
        event_id: EventId,
        input_time: SimTime,
        batch_time: SimTime,
    },
    InputBeforeSimulationStart {
        event_id: EventId,
        input_time: SimTime,
        start_time: SimTime,
    },
}

impl Display for ModelError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteParameter { name, value } => {
                write!(formatter, "parameter {name} must be finite, got {value}")
            }
            Self::NonPositiveTimeConstant { value_ms } => write!(
                formatter,
                "membrane time constant must be positive, got {value_ms} ms"
            ),
            Self::ZeroRefractoryPeriod => {
                formatter.write_str("refractory period must be greater than zero")
            }
            Self::PotentialNotBelowThreshold {
                name,
                value_mv,
                threshold_mv,
            } => write!(
                formatter,
                "{name} ({value_mv} mV) must be below threshold ({threshold_mv} mV)"
            ),
            Self::NonFiniteInitialPotential { value_mv } => {
                write!(
                    formatter,
                    "initial potential must be finite, got {value_mv} mV"
                )
            }
            Self::InitialPotentialAtOrAboveThreshold {
                value_mv,
                threshold_mv,
            } => write!(
                formatter,
                "initial potential ({value_mv} mV) must be below threshold ({threshold_mv} mV)"
            ),
            Self::InvalidInputMagnitude { magnitude_mv } => write!(
                formatter,
                "input magnitude must be finite and non-negative, got {magnitude_mv} mV"
            ),
            Self::TimeWentBackwards { current, requested } => write!(
                formatter,
                "cannot move simulation time from {} us back to {} us",
                current.as_micros(),
                requested.as_micros()
            ),
            Self::TimeOverflow => formatter.write_str("simulation time overflow"),
            Self::NonFiniteInputSum => {
                formatter.write_str("the deterministic sum of input magnitudes is not finite")
            }
            Self::NonFiniteIntegratedPotential => {
                formatter.write_str("membrane potential became non-finite after applying inputs")
            }
            Self::EmptyInputBatch => formatter.write_str(
                "input batches must contain at least one event; use advance_to for sampling",
            ),
            Self::DuplicateInputBatchTime(time) => write!(
                formatter,
                "inputs at {} us were already processed; simultaneous inputs must share one batch",
                time.as_micros()
            ),
            Self::DuplicateEventId(id) => write!(formatter, "duplicate event id {}", id.0),
            Self::InputTimeDoesNotMatchBatch {
                event_id,
                input_time,
                batch_time,
            } => write!(
                formatter,
                "event {} occurs at {} us but batch occurs at {} us",
                event_id.0,
                input_time.as_micros(),
                batch_time.as_micros()
            ),
            Self::InputBeforeSimulationStart {
                event_id,
                input_time,
                start_time,
            } => write!(
                formatter,
                "event {} occurs at {} us before simulation start {} us",
                event_id.0,
                input_time.as_micros(),
                start_time.as_micros()
            ),
        }
    }
}

impl Error for ModelError {}
