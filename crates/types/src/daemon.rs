//! Shared types for the roasted-rs daemon server

use crate::ws::gaggimate::{MachineMode, ProcessStatus, SystemPhase, SystemState, WarningState};

/// Merged/Accumulated state from Gaggimate
pub struct GaggimateState {
    /// Current or last process
    pub process: ProcessStatus,
    /// Current temperature.
    pub current_temperature: f64,
    /// Target temperature.
    pub target_temperature: f64,
    /// Current pressure.
    pub current_pressure: f64,
    /// Current flow.
    pub current_flow: f64,
    /// Current scale weight in grams; firmware reports zero when disconnected.
    /// Check `scale_connected` before interpreting this as a measurement.
    pub current_weight: f64,
    /// Bluetooth scale weight in grams; currently mirrors `current_weight`.
    /// Negative readings are possible after removing a tared cup.
    pub bluetooth_weight: f64,
    /// Whether the Bluetooth scale is connected; absent retains prior state.
    pub scale_connected: bool,
    /// Display system state.
    pub system_state: SystemState,
    /// Machine warnings; an empty list is a real update.
    pub warnings: Vec<WarningState>,
    /// Target pressure.
    pub target_pressure: f64,
    /// Selected operating mode; brew mode does not imply an active shot.
    pub machine_mode: MachineMode,
    /// Selected profile label.
    pub profile_label: String,
    /// Pressure capability.
    pub pressure_capable: bool,
    /// Dimming capability.
    pub dimming_capable: bool,
}

/// State about the overall gaggimate system.
pub struct GaggimateSystemState {
    /// Current temperature.
    pub phase: SystemPhase,
    /// Display message
    pub message: Option<String>,
    /// Controller error code if any
    pub error_code: Option<i64>,
}
