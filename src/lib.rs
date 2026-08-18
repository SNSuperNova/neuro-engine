//! Deterministic, event-driven primitives for Neuro Engine.
//!
//! Gate 0 intentionally contains only a single-neuron LIF model. Network
//! scheduling, synapses, storage, and rendering belong to later gates.

pub mod experiment;
pub mod lif;
pub mod metrics;
pub mod network;
pub mod phase2;
pub mod playback;

pub use experiment::{
    Gate2AcceptanceCriteria, Gate2AcceptanceReport, Gate2ExperimentConfig, Gate2ExperimentResult,
    Gate2Summary, GeneratedNetworkConfig, Pattern3x3, PatternStimulusSchedule,
    Phase2ExperimentConfig, Phase2ExperimentResult, Phase2Summary, generate_network,
    local_stimulus_inputs, pacemaker_inputs, pattern_stimulus_inputs, run_gate2_experiment,
    run_phase2_experiment,
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
    ActivityRegulatorConfig, EventLog, ExternalInput, ExternalInputKind, InputOrigin, InputRecord,
    LogEvent, NetworkDefinition, NetworkError, NetworkRun, NetworkSpike, NeuronPolarity,
    NeuronSpec, Position3, PropagationSpec, RegulationRecord, SimulationLimits, SpikeId, SynapseId,
    SynapseSpec, simulate_network, simulate_network_with_limits, simulate_network_with_regulator,
};
pub use phase2::{
    AblationComparison, AccuracyReport, ActivityRegulatorReport, FunctionalGroupReport,
    FunctionalNeuronScore, InputStabilityEvaluation, Phase2AnalysisError, Phase2HypothesisReport,
    Phase2PatternId, Phase2ProtocolConfig, Phase2ProtocolReport, Phase2ProtocolResult,
    Phase2RawEvent, Phase2StabilityCriteria, Phase2TrialArtifact, ReadoutReport,
    SeedExperimentReport, StructureSeedScreen, TrialMetadata, TrialResponse, TrialSplit,
    degree_preserving_connection_shuffle, evaluate_phase2_input_stability, run_phase2_protocol,
    run_phase2_trial_artifact, screen_phase2_structure_seeds, silence_neurons,
};
pub use playback::{
    ArrivalOrigin, PlaybackArrival, PlaybackBundle, PlaybackChunk, PlaybackDataset, PlaybackError,
    PlaybackInFlight, PlaybackMetricSample, PlaybackNeuron, PlaybackNeuronSample, PlaybackSpike,
    PlaybackSynapse, build_playback_dataset,
};
