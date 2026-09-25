//! Read-only GaggiMate shot telemetry on `/ws`.
//!
//! Status fields follow upstream revision `001a475105cdfa11c54e2c688046f8159dda8d3e`:
//! <https://github.com/jniebuhr/gaggimate/blob/001a475105cdfa11c54e2c688046f8159dda8d3e/docs/websocket-api.yaml>.
//! Process telemetry supplements that schema from upstream
//! `src/display/plugins/WebSocketHandler.cpp` (`publishTelemetry`).
//!
//!
//! ```
//! use roasted_types::gaggimate::{Message, Patch};
//! let message: Message = serde_json::from_str(r#"{"tp":"evt:status","pr":8.5}"#)?;
//! if let Message::Status(status) = message {
//!     assert_eq!(status.current_pressure, Patch::Value(8.5));
//!     assert_eq!(status.current_temperature, Patch::Absent);
//! }
//! # Ok::<(), serde_json::Error>(())
//! ```

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// A field in a partial status update.
///
/// Use `#[serde(default, skip_serializing_if = "Patch::is_absent")]` on fields.
/// Ordinary `Option<T>` would collapse missing and null into the same value.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Patch<T> {
    /// No update; retain the previous value.
    #[default]
    Absent,
    /// Explicit JSON null; clear the previous value.
    Null,
    /// Replace the previous value, including zero, false, or an empty collection.
    Value(T),
}

impl<T> Patch<T> {
    /// Whether this field was omitted from the update.
    pub fn is_absent(&self) -> bool {
        matches!(self, Self::Absent)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Patch<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Option::<T>::deserialize(deserializer).map(|value| value.map_or(Self::Null, Self::Value))
    }
}

impl<T: Serialize> Serialize for Patch<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Absent => Err(serde::ser::Error::custom(
                "absent patch fields must be omitted by the containing struct",
            )),
            Self::Null => serializer.serialize_none(),
            Self::Value(value) => value.serialize(serializer),
        }
    }
}

/// Incoming telemetry identified by the JSON `tp` field.
/// Unrelated API messages are ignored.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "tp")]
pub enum Message {
    /// `evt:status`.
    #[serde(rename = "evt:status")]
    Status(Box<Status>),
    /// An unrecognized message type; its payload is ignored and cannot be serialized.
    #[serde(other, skip_serializing)]
    Unknown,
}

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

/// Operating modes from the firmware's `src/display/core/constants.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum MachineMode {
    /// Machine on standby.
    Standby = 0,
    /// Brewing mode, including idle time between shots.
    Brew = 1,
    /// Steam mode.
    Steam = 2,
    /// Hot-water mode.
    HotWater = 3,
    /// Grinding mode.
    Grind = 4,
}

/// Process snapshot emitted by the firmware, including the last finished process.
///
/// Only activity is emitted for processes other than brewing and grinding.
/// A snapshot replaces the previous process object; missing details must not
/// inherit a previous brew's phase or elapsed time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessStatus {
    /// Controller activity, encoded as integer 0 or 1 rather than a JSON boolean.
    #[serde(rename = "a")]
    pub activity: ProcessActivity,
    /// Current phase category; infusion and brew belong to the same shot.
    #[serde(rename = "s", default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<ProcessPhase>,
    /// Display label; do not use this user-facing text for lifecycle detection.
    #[serde(rename = "l", default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Elapsed process time in milliseconds.
    #[serde(rename = "e", default, skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: Option<u64>,
    /// Utility profile flag (0 or 1); absent on older firmware and grind processes.
    #[serde(rename = "u", default, skip_serializing_if = "Option::is_none")]
    pub utility: Option<u8>,
    /// Whether phase progress is measured by time or volume.
    #[serde(rename = "tt", default, skip_serializing_if = "Option::is_none")]
    pub target_type: Option<ProcessTarget>,
    /// Phase target: milliseconds for time, volume for volumetric mode.
    #[serde(rename = "pt", default, skip_serializing_if = "Option::is_none")]
    pub phase_target: Option<f64>,
    /// Phase progress in the same units as the target.
    #[serde(rename = "pp", default, skip_serializing_if = "Option::is_none")]
    pub phase_progress: Option<f64>,
}

/// Numeric controller activity reported within a process snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum ProcessActivity {
    /// Controller is inactive; a finished process may still be reported.
    Inactive = 0,
    /// Controller is active.
    Active = 1,
}

/// Live process phase, distinct from the profile's phase type names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProcessPhase {
    /// Puck saturation before the main extraction.
    Infusion,
    /// Main extraction.
    Brew,
    /// Grinding, not a shot.
    Grind,
}

/// Measurement used for live process progress and target values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProcessTarget {
    /// Elapsed milliseconds.
    Time,
    /// Measured volume.
    Volumetric,
}

/// Display state carried within a status frame.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SystemState {
    /// System phase.
    #[serde(rename = "s", default, skip_serializing_if = "Patch::is_absent")]
    pub phase: Patch<SystemPhase>,
    /// Display message; empty when ready.
    #[serde(rename = "m", default, skip_serializing_if = "Patch::is_absent")]
    pub message: Patch<String>,
    /// Controller error code; zero means none.
    #[serde(rename = "c", default, skip_serializing_if = "Patch::is_absent")]
    pub error_code: Patch<i64>,
}

/// Documented display system phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SystemPhase {
    /// Starting.
    Starting,
    /// Waiting.
    Waiting,
    /// Ready.
    Ready,
    /// Updating.
    Updating,
    /// Autotuning.
    Autotuning,
    /// Mismatch.
    Mismatch,
    /// Error.
    Error,
}

/// Machine warning carried in status telemetry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WarningState {
    /// Warning category.
    #[serde(rename = "k")]
    pub category: WarningKey,
    /// Configured severity, encoded as an integer.
    #[serde(rename = "l")]
    pub severity: WarningLevel,
    /// Whether the warning is active; context determines the omitted value.
    #[serde(rename = "a", default, skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}

/// Documented warning categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WarningKey {
    /// Water.
    Water,
    /// Flush.
    Flush,
    /// Switch.
    Switch,
    /// ScaleConnected.
    ScaleConnected,
    /// ScaleBattery.
    ScaleBattery,
    /// Temperature.
    Temperature,
}

/// Warning severity on the JSON wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum WarningLevel {
    /// 0: ignore.
    Ignore = 0,
    /// 1: warn.
    Warn = 1,
    /// 2: error.
    Error = 2,
}
