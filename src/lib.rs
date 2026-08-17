//! Deterministic, event-driven primitives for Neuro Engine.
//!
//! Gate 0 intentionally contains only a single-neuron LIF model. Network
//! scheduling, synapses, storage, and rendering belong to later gates.

pub mod lif;

pub use lif::{
    BatchResult, EventId, InputPolarity, LifNeuron, LifParameters, ModelError, NeuronId,
    NeuronSnapshot, SimDuration, SimTime, SimulationTrace, SpikeEvent, TimedInput, simulate_neuron,
};
