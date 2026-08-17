//! Deterministic, event-driven primitives for Neuro Engine.
//!
//! Gate 0 intentionally contains only a single-neuron LIF model. Network
//! scheduling, synapses, storage, and rendering belong to later gates.

pub mod experiment;
pub mod lif;
pub mod metrics;
pub mod network;
pub mod playback;

pub use experiment::{
    Gate2AcceptanceCriteria, Gate2AcceptanceReport, Gate2ExperimentConfig, Gate2ExperimentResult,
    Gate2Summary, GeneratedNetworkConfig, generate_network, local_stimulus_inputs,
    pacemaker_inputs, run_gate2_experiment,
};
pub use lif::{
    BatchResult, EventId, InputPolarity, LifNeuron, LifParameters, ModelError, NeuronId,
    NeuronSnapshot, SimDuration, SimTime, SimulationTrace, SpikeEvent, TimedInput, simulate_neuron,
};
pub use metrics::{
    MetricsConfig, MetricsError, NetworkMetrics, TrajectoryDifference, compare_spike_trajectories,
    compute_network_metrics,
};
pub use network::{
    EventLog, ExternalInput, ExternalInputKind, InputOrigin, InputRecord, LogEvent,
    NetworkDefinition, NetworkError, NetworkRun, NetworkSpike, NeuronPolarity, NeuronSpec,
    Position3, PropagationSpec, SimulationLimits, SpikeId, SynapseId, SynapseSpec,
    simulate_network, simulate_network_with_limits,
};
pub use playback::{
    ArrivalOrigin, PlaybackArrival, PlaybackBundle, PlaybackChunk, PlaybackDataset, PlaybackError,
    PlaybackInFlight, PlaybackMetricSample, PlaybackNeuron, PlaybackNeuronSample, PlaybackSpike,
    PlaybackSynapse, build_playback_dataset,
};
