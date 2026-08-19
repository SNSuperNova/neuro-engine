use serde::Serialize;

use crate::adaptive_mechanism::{
    REFERENCE_MECHANISM_ID, ReferenceSubstrateManifest, reference_substrate_manifest,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrozenArtifact {
    pub file: String,
    pub sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M0InterfaceAudit {
    pub all_adjustable_variables_declared: bool,
    pub read_and_write_permissions_declared: bool,
    pub update_budgets_declared: bool,
    pub freeze_modes_declared: bool,
    pub task_labels_not_exposed: bool,
    pub readout_state_not_exposed: bool,
    pub output_writes_not_representable: bool,
    pub boundary_enforces_finite_bounded_writes: bool,
    pub candidate_runner_uses_closed_task_loop: bool,
    pub map2_exact_json_regression_tested: bool,
    pub candidate_interface_disabled_by_default: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M0CapabilityResult {
    pub map2c_continuous_accuracy: f64,
    pub reset_control_accuracy: f64,
    pub no_supply_accuracy: f64,
    pub frozen_plasticity_accuracy: f64,
    pub stable_region_parameter_ids: Vec<usize>,
    pub interpretation: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MechanismM0Release {
    pub version: String,
    pub carrier_state: ReferenceSubstrateManifest,
    pub adjustment_mechanism: String,
    pub interface_audit: M0InterfaceAudit,
    pub frozen_map2_artifacts: Vec<FrozenArtifact>,
    pub capability_result: M0CapabilityResult,
    pub verification_command: String,
}

pub fn mechanism_m0_release() -> MechanismM0Release {
    MechanismM0Release {
        version: "adaptive-mechanism/m0-v0.1".into(),
        carrier_state: reference_substrate_manifest(),
        adjustment_mechanism: REFERENCE_MECHANISM_ID.into(),
        interface_audit: M0InterfaceAudit {
            all_adjustable_variables_declared: true,
            read_and_write_permissions_declared: true,
            update_budgets_declared: true,
            freeze_modes_declared: true,
            task_labels_not_exposed: true,
            readout_state_not_exposed: true,
            output_writes_not_representable: true,
            boundary_enforces_finite_bounded_writes: true,
            candidate_runner_uses_closed_task_loop: true,
            map2_exact_json_regression_tested: true,
            candidate_interface_disabled_by_default: true,
        },
        frozen_map2_artifacts: vec![
            FrozenArtifact {
                file: "map2a-v0.3.json".into(),
                sha256: "28a6319ef533583622e56c1895776380e4c277f7ee98f32af4dc41db13d920cb".into(),
            },
            FrozenArtifact {
                file: "map2b-v0.4.json".into(),
                sha256: "d07fbd50964293bc48150399600fb91e97e6304a9826b50ce6f977b0a8af4636".into(),
            },
            FrozenArtifact {
                file: "map2c-v0.5.json".into(),
                sha256: "78a9c9f118f83978761d5e7538cf021da9b1d625bf3a4210a4880a37961ac08f".into(),
            },
        ],
        capability_result: M0CapabilityResult {
            map2c_continuous_accuracy: 0.661,
            reset_control_accuracy: 0.676,
            no_supply_accuracy: 0.557,
            frozen_plasticity_accuracy: 0.530,
            stable_region_parameter_ids: vec![0, 16, 20, 21, 29, 45],
            interpretation: "Map 2 is retained as a bounded continuous-adaptation reference, not evidence of general learning or a biologically complete neuron.".into(),
        },
        verification_command: "cargo test --all-targets --release".into(),
    }
}
