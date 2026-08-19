//! Deterministic embodied-learning experiments and retained neural simulation
//! primitives for Neuro Engine.

pub mod adaptive_mechanism;
pub mod embodied;
pub mod experiment;
pub mod gate_a;
pub mod gate_b;
pub mod gate_c;
pub mod gate_d;
pub mod gate_e;
pub mod gate_f;
pub mod learnability_map;
pub mod lif;
pub mod map1;
pub mod map2;
pub mod mechanism_m0;
pub mod metrics;
pub mod network;

pub use adaptive_mechanism::{
    ActivityAdjustment, ActivityObservation, AdjustableVariable, AdjustmentMechanism,
    HomeostasisAdjustment, HomeostasisObservation, MechanismStateContract, PlasticityAdjustment,
    PlasticityObservation, REFERENCE_MECHANISM_ID, REFERENCE_SUBSTRATE_VERSION,
    ReferenceSubstrateManifest, VariableContract, reference_substrate_manifest,
};
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
pub use gate_d::{
    GateDAcceptanceReport, GateDBehaviorTrace, GateDControllerBudget, GateDControllerKind,
    GateDControllerReport, GateDExperimentConfig, GateDExperimentResult, GateDMetricIntervals,
    GateDMetricPoint, GateDPairedEffect, GateDSampleEfficiency, GateDSeedMetric,
    GateDStabilityIntervals, GateDStabilityPoint, GateDTraceFrame, GateDTrainingCurvePoint,
    run_gate_d_experiment,
};
pub use gate_e::{
    GateEAcceptanceReport, GateEActivityIntervals, GateEActivityPoint, GateEBehaviorTrace,
    GateEControllerBudget, GateEControllerKind, GateEControllerReport, GateECostPoint,
    GateEExperimentConfig, GateEExperimentResult, GateEMetricIntervals, GateEMetricPoint,
    GateEPairedEffect, GateERuntimeBenchmarkPoint, GateESampleEfficiency, GateESeedMetric,
    GateETraceFrame, GateETrainingCurvePoint, benchmark_gate_e_runtime, run_gate_e_experiment,
};
pub use gate_f::{
    GateFAcceptanceReport, GateFAdaptationSummary, GateFBehaviorTrace, GateFCheckpointReport,
    GateFControllerBudget, GateFControllerKind, GateFExperimentConfig, GateFExperimentResult,
    GateFMetricIntervals, GateFMetricPoint, GateFPairedEffect, GateFPhase, GateFRule,
    GateFSeedMetric, GateFTraceFrame, GateFWeightIntervals, GateFWeightPoint,
    run_gate_f_experiment,
};
pub use learnability_map::{
    ContinuousSeedResult, ContinuousStreamConfig, Map0AcceptanceReport, Map0Control,
    Map0ControlSummary, Map0DynamicsMetrics, Map0ExperimentConfig, Map0ExperimentResult,
    Map0Interval, Map0ParameterPoint, Map0ParameterSummary, Map0ProbeMetrics, Map0RegionClass,
    Map0SeedResult, Map0Thresholds, run_continuous_seed_with_adjustment_mechanism,
    run_map_seed_with_adjustment_mechanism, run_map0_experiment,
};
pub use lif::{
    BatchResult, EventId, InputPolarity, LifNeuron, LifParameters, ModelError, NeuronId,
    NeuronSnapshot, SimDuration, SimTime, SimulationTrace, SpikeEvent, TimedInput, simulate_neuron,
};
pub use map1::{
    Map1AcceptanceReport, Map1ExperimentConfig, Map1ExperimentResult, Map1MechanismComparison,
    run_map1_experiment,
};
pub use map2::{
    Map2AAcceptanceReport, Map2AExperimentConfig, Map2AExperimentResult, Map2AMechanismComparison,
    Map2BAcceptanceReport, Map2BExperimentConfig, Map2BExperimentResult, Map2BMechanismComparison,
    Map2CAcceptanceReport, Map2CExperimentConfig, Map2CExperimentResult, Map2CParameterSummary,
    run_map2a_experiment, run_map2b_experiment, run_map2c_experiment,
};
pub use mechanism_m0::{
    FrozenArtifact, M0CapabilityResult, M0InterfaceAudit, MechanismM0Release, mechanism_m0_release,
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
