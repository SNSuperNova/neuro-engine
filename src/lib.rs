//! Deterministic embodied-learning experiments and retained neural simulation
//! primitives for Neuro Engine.

pub mod embodied;
pub mod experiment;
pub mod gate_a;
pub mod gate_b;
pub mod gate_c;
pub mod lif;
pub mod metrics;
pub mod network;

pub use embodied::{
    ACTION_COUNT, AdaptiveController, AgentAction, ArenaConfig, BehaviorFrame, BehaviorTrace,
    ControllerConfig, EmbodiedAcceptanceReport, EmbodiedError, EmbodiedExperimentConfig,
    EmbodiedExperimentResult, EpisodeSummary, EvaluationReport, GridPosition, HIDDEN_COUNT,
    Heading, PlasticityReport, RewardBreakdown, RewardConfig, SENSOR_COUNT, SensorConfig,
    TrainingCurvePoint, run_embodied_experiment,
};

pub use experiment::{
    Gate2AcceptanceCriteria, Gate2AcceptanceReport, Gate2ExperimentConfig, Gate2ExperimentResult,
    Gate2Summary, GeneratedNetworkConfig, generate_network, local_stimulus_inputs,
    pacemaker_inputs, run_gate2_experiment,
};
pub use gate_a::{
    ConfidenceInterval, GateAAcceptanceReport, GateACurvePoint, GateAExperimentConfig,
    GateAExperimentResult, GateAMetricIntervals, GateAMetricPoint, GateASampleEfficiency,
    GateASeedRun, GateAVariantReport, run_gate_a_experiment,
};
pub use gate_b::{
    ForkSide, GATE_B_SENSOR_COUNT, GateBAcceptanceReport, GateBBehaviorTrace, GateBCondition,
    GateBConfidenceInterval, GateBControllerBudget, GateBControllerKind, GateBControllerReport,
    GateBExperimentConfig, GateBExperimentResult, GateBMetricIntervals, GateBMetricPoint,
    GateBPairedEffect, GateBRobustnessConfig, GateBRobustnessFactor, GateBRobustnessPoint,
    GateBRobustnessResult, GateBSampleEfficiency, GateBSeedMetric, GateBTraceFrame,
    GateBTrainingCurvePoint, GateBTrialSummary, run_gate_b_experiment,
    run_gate_b_robustness_experiment,
};
pub use gate_c::{
    GATE_C_SENSOR_COUNT, GateCAcceptanceReport, GateCBehaviorTrace, GateCCellReport,
    GateCConfidenceInterval, GateCControllerBudget, GateCExperimentConfig, GateCExperimentResult,
    GateCMetricIntervals, GateCMetricPoint, GateCPairedEffect, GateCPhase, GateCSampleEfficiency,
    GateCSeedMetric, GateCTraceFrame, GateCTrainingCurvePoint, GateCTrialSummary,
    run_gate_c_experiment,
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
