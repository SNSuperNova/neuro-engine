//! Constrained adjustment-mechanism interface for the frozen Map 2 carrier.
//!
//! The interface intentionally exposes neither task labels nor action/readout
//! state. A mechanism can observe local carrier state and consequence signals,
//! then return writes to the explicitly permitted carrier variables only.

use serde::Serialize;

pub const REFERENCE_SUBSTRATE_VERSION: &str = "reference-substrate/v1";
pub const REFERENCE_MECHANISM_ID: &str = "map2-soft-bound-resource-modulation/v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AdjustableVariable {
    ActivityState,
    SensoryWeights,
    RecurrentWeights,
    Excitability,
    Resource,
    MechanismState,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VariableContract {
    pub variable: AdjustableVariable,
    pub read_scope: String,
    pub update_permission: String,
    pub value_range: String,
    pub update_budget: String,
    pub freeze_mode: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSubstrateManifest {
    pub version: String,
    pub hidden_unit_count: usize,
    pub sensory_channel_count: usize,
    pub action_count: usize,
    pub plastic_recurrent_connections_per_unit: usize,
    pub adjustable_variables: Vec<VariableContract>,
    pub development_seed_rule: String,
    pub confirmation_seed_rule: String,
    pub task_generation_rule: String,
    pub control_interface: Vec<String>,
    pub forbidden_observations: Vec<String>,
    pub forbidden_writes: Vec<String>,
}

pub fn reference_substrate_manifest() -> ReferenceSubstrateManifest {
    use AdjustableVariable::*;
    let contract = |variable,
                    read_scope: &str,
                    update_permission: &str,
                    value_range: &str,
                    update_budget: &str,
                    freeze_mode: &str| VariableContract {
        variable,
        read_scope: read_scope.into(),
        update_permission: update_permission.into(),
        value_range: value_range.into(),
        update_budget: update_budget.into(),
        freeze_mode: freeze_mode.into(),
    };
    ReferenceSubstrateManifest {
        version: REFERENCE_SUBSTRATE_VERSION.into(),
        hidden_unit_count: 24,
        sensory_channel_count: 4,
        action_count: 2,
        plastic_recurrent_connections_per_unit: 2,
        adjustable_variables: vec![
            contract(
                ActivityState,
                "current unit plus its local raw activation and resource",
                "carrier transition; mechanism may only modulate the current unit",
                "[-1, 1]",
                "one write per unit per carrier step",
                "freeze adjustment action to identity; carrier transition remains active",
            ),
            contract(
                SensoryWeights,
                "incoming sensory weights of the current unit",
                "seeded initialization only in reference-substrate/v1",
                "[-1.2, 1.2] at initialization",
                "zero online writes",
                "fixed after deterministic initialization",
            ),
            contract(
                RecurrentWeights,
                "one eligible incoming recurrent connection",
                "mechanism action for that connection only",
                "soft bound derived from the unit reference norm",
                "at most 2 writes per unit per rewarded trial",
                "FrozenPlasticity returns the existing value",
            ),
            contract(
                Excitability,
                "current unit activity EMA and gain",
                "homeostatic action for the current unit only",
                "[minimum_excitability_gain, maximum_excitability_gain]",
                "at most one write per unit per rewarded trial",
                "NoHomeostasis returns the existing gain",
            ),
            contract(
                Resource,
                "current unit resource and local activity/plasticity cost",
                "activity and plasticity actions for the current unit only",
                "[0, 1]",
                "one write per unit step plus one per eligible weight update",
                "NoResourceAccounting fixes resource at 1; NoResourceSupply sets supply to 0",
            ),
            contract(
                MechanismState,
                "mechanism-owned local history only",
                "reserved; reference mechanism has no additional state",
                "must be declared and bounded by each candidate",
                "zero for the reference mechanism",
                "candidate-specific state reset/freeze must be paired",
            ),
        ],
        development_seed_rule: "seed_partition(protocol.seed XOR 0x4445_5601)".into(),
        confirmation_seed_rule: "seed_partition(protocol.seed XOR 0x434f_4e46_0101)".into(),
        task_generation_rule: "balanced binary cue; deterministic per-trial RNG; continuous rule changes use fixed minimum gap and seeded probability".into(),
        control_interface: vec![
            "FrozenPlasticity".into(),
            "RandomReward".into(),
            "NoHomeostasis".into(),
            "NoResourceAccounting".into(),
            "NoResourceSupply".into(),
            "ResetBetweenTrials".into(),
            "ShuffledStructure".into(),
        ],
        forbidden_observations: vec![
            "target label or correct action".into(),
            "action probabilities or readout logits".into(),
            "action readout weights".into(),
            "future task events".into(),
        ],
        forbidden_writes: vec![
            "action choice".into(),
            "action probabilities or logits".into(),
            "action readout weights".into(),
            "task generator, target label, or reward".into(),
        ],
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActivityObservation {
    pub unit: usize,
    pub raw_activity: f64,
    pub resource_level: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActivityAdjustment {
    pub activity: f64,
    pub resource_level: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlasticityObservation {
    pub target_unit: usize,
    pub source_unit: usize,
    pub current_weight: f64,
    pub proposed_delta: f64,
    pub reference_norm: f64,
    pub absolute_weight_limit: f64,
    pub resource_level: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlasticityAdjustment {
    pub recurrent_weight: f64,
    pub resource_level: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HomeostasisObservation {
    pub unit: usize,
    pub strength: f64,
    pub mean_absolute_activity: f64,
    pub activity_ema: f64,
    pub excitability_gain: f64,
    pub recurrent_weights: [f64; 2],
    pub reference_recurrent_norm: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HomeostasisAdjustment {
    pub activity_ema: f64,
    pub excitability_gain: f64,
    pub recurrent_weights: [f64; 2],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MechanismStateContract {
    pub scalar_state_per_unit: usize,
    pub value_range: String,
    pub update_budget: String,
    pub freeze_mode: String,
}

/// The only runtime injection point for adjustment mechanisms.
///
/// Observation types contain no label/readout fields and action types contain
/// no output fields, making direct task-output writes unrepresentable here.
pub trait AdjustmentMechanism: Clone {
    fn mechanism_id(&self) -> &'static str;
    fn mechanism_state_contract(&self) -> MechanismStateContract;
    fn initial_resource(&self) -> f64;
    fn excitability_bounds(&self) -> [f64; 2];
    fn adjust_activity(&mut self, observation: ActivityObservation) -> ActivityAdjustment;
    fn adjust_plasticity(&mut self, observation: PlasticityObservation) -> PlasticityAdjustment;
    fn adjust_homeostasis(&mut self, observation: HomeostasisObservation) -> HomeostasisAdjustment;
}
