//! Shared types/merges into generated WebSocket types for the roasted-rs daemon server

use crate::ws::gaggimate::{
    MachineMode, ProcessStatus, Status, SystemPhase, SystemState, WarningState,
};

/// State snapshot from Gaggimate
#[derive(Default, Clone, PartialEq, Debug)]
pub struct GaggimateState {
    /// Current or last process
    pub process: ProcessStatus,
    /// Current temperature, in Celsius
    pub current_temperature: f64,
    /// Target temperature, in Celsius.
    pub target_temperature: f64,
    /// Current pressure, in Bar.
    pub current_pressure: f64,
    /// Current flow, in grams per second.
    pub current_flow: f64,
    /// Current scale weight in grams. firmware reports zero when disconnected.
    /// Check `scale_connected` before interpreting this as a measurement.
    pub current_weight: f64,
    /// Bluetooth scale weight in grams. currently mirrors `current_weight`.
    /// Negative readings are possible after removing a tared cup.
    pub bluetooth_weight: f64,
    /// Whether the Bluetooth scale is connected. absent retains prior state.
    pub scale_connected: bool,
    /// Display system state.
    pub system_state: SystemState,
    /// Machine warnings; an empty list is a real update.
    pub warnings: Vec<WarningState>,
    /// Target pressure in Bar.
    pub target_pressure: f64,
    /// Selected operating mode; brew mode does not imply an active shot.
    pub machine_mode: MachineMode,
    /// Selected profile label.
    pub profile_label: String,
    // I'm making an assumption that the correct "clear/default" value
    // for gaggimate "bool" is `false`
    /// Pressure capability.
    pub pressure_capable: bool,
    /// Dimming capability.
    pub dimming_capable: bool,
}

/// State snapshot about the overall Gaggimate system.
pub struct GaggimateSystemState {
    /// Current temperature.
    pub phase: SystemPhase,
    /// Display message
    pub message: Option<String>,
    /// Controller error code if any
    pub error_code: Option<i64>,
}

/// Merge two types
/// Implement on the canonical version of the type for a
/// instant in time. `Self` should be the post-merge type.
pub trait Merge<T> {
    /// Conduct the merge, mutating `self`
    fn merge(&mut self, other: T);
}

impl<T, U> Merge<Box<T>> for U
where
    U: Merge<T>,
{
    fn merge(&mut self, other: Box<T>) {
        self.merge(*other)
    }
}

impl Merge<Status> for GaggimateState {
    fn merge(&mut self, other: Status) {
        self.process.merge(other.process);
        self.current_temperature.merge(other.current_temperature);
        self.target_temperature.merge(other.target_temperature);
        self.current_pressure.merge(other.current_pressure);
        self.current_flow.merge(other.current_flow);
        self.current_weight.merge(other.current_weight);
        self.bluetooth_weight.merge(other.bluetooth_weight);
        self.scale_connected.merge(other.scale_connected);
        self.system_state.merge(other.system_state);
        self.warnings.merge(other.warnings);
        self.target_pressure.merge(other.target_pressure);
        self.machine_mode.merge(other.machine_mode);
        self.profile_label.merge(other.profile_label);
        self.pressure_capable.merge(other.pressure_capable);
        self.dimming_capable.merge(other.dimming_capable);
    }
}

impl Merge<SystemState> for GaggimateSystemState {
    fn merge(&mut self, other: SystemState) {
        self.phase.merge(other.phase);
        self.message.merge(other.message.map(|s| (!s.is_empty()).then_some(s)));
        self.error_code.merge(other.error_code.map(|e| (e != 0).then_some(e)));
    }
}
