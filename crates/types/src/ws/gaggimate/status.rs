use serde::{Deserialize, Serialize};

use super::*;

/// Partial telemetry/state frame, including firmware process telemetry.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Status {
    /// Current or last process.
    #[serde(default, skip_serializing_if = "Patch::is_absent")]
    pub process: Patch<ProcessStatus>,
    /// Current temperature.
    #[serde(rename = "ct", default, skip_serializing_if = "Patch::is_absent")]
    pub current_temperature: Patch<f64>,
    /// Target temperature.
    #[serde(rename = "tt", default, skip_serializing_if = "Patch::is_absent")]
    pub target_temperature: Patch<f64>,
    /// Current pressure.
    #[serde(rename = "pr", default, skip_serializing_if = "Patch::is_absent")]
    pub current_pressure: Patch<f64>,
    /// Current flow.
    #[serde(rename = "fl", default, skip_serializing_if = "Patch::is_absent")]
    pub current_flow: Patch<f64>,
    /// Current scale weight in grams; firmware reports zero when disconnected.
    /// Check `scale_connected` before interpreting this as a measurement.
    #[serde(rename = "cw", default, skip_serializing_if = "Patch::is_absent")]
    pub current_weight: Patch<f64>,
    /// Bluetooth scale weight in grams; currently mirrors `current_weight`.
    /// Negative readings are possible after removing a tared cup.
    #[serde(rename = "bw", default, skip_serializing_if = "Patch::is_absent")]
    pub bluetooth_weight: Patch<f64>,
    /// Whether the Bluetooth scale is connected; absent retains prior state.
    #[serde(rename = "bc", default, skip_serializing_if = "Patch::is_absent")]
    pub scale_connected: Patch<bool>,
    /// Display system state.
    #[serde(rename = "sys", default, skip_serializing_if = "Patch::is_absent")]
    pub system_state: Patch<SystemState>,
    /// Machine warnings; an empty list is a real update.
    #[serde(rename = "warn", default, skip_serializing_if = "Patch::is_absent")]
    pub warnings: Patch<Vec<WarningState>>,
    /// Target pressure.
    #[serde(rename = "pt", default, skip_serializing_if = "Patch::is_absent")]
    pub target_pressure: Patch<f64>,
    /// Selected operating mode; brew mode does not imply an active shot.
    #[serde(rename = "m", default, skip_serializing_if = "Patch::is_absent")]
    pub machine_mode: Patch<MachineMode>,
    /// Selected profile label.
    #[serde(rename = "p", default, skip_serializing_if = "Patch::is_absent")]
    pub profile_label: Patch<String>,
    /// Pressure capability.
    #[serde(rename = "cp", default, skip_serializing_if = "Patch::is_absent")]
    pub pressure_capable: Patch<bool>,
    /// Dimming capability.
    #[serde(rename = "cd", default, skip_serializing_if = "Patch::is_absent")]
    pub dimming_capable: Patch<bool>,
}
