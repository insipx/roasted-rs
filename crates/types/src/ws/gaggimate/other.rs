//! Types that are simply their definitions, no manual implementations.
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use super::*;

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

/// Operating modes from the firmware's `src/display/core/constants.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr, Default)]
#[cfg_attr(feature = "diesel", derive(diesel::AsExpression, diesel::FromSqlRow))]
#[cfg_attr(feature = "diesel", diesel(sql_type = diesel::sql_types::Text))]
#[repr(u8)]
pub enum MachineMode {
    /// Machine on standby.
    #[default]
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

/// Numeric controller activity reported within a process snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr, Default)]
#[repr(u8)]
#[cfg_attr(feature = "diesel", derive(diesel::AsExpression, diesel::FromSqlRow))]
#[cfg_attr(feature = "diesel", diesel(sql_type = diesel::sql_types::Text))]
pub enum ProcessActivity {
    /// Controller is inactive; a finished process may still be reported.
    #[default]
    Inactive = 0,
    /// Controller is active.
    Active = 1,
}

/// Live process phase, distinct from the profile's phase type names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "diesel", derive(diesel::AsExpression, diesel::FromSqlRow))]
#[cfg_attr(feature = "diesel", diesel(sql_type = diesel::sql_types::Text))]
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
#[cfg_attr(feature = "diesel", derive(diesel::AsExpression, diesel::FromSqlRow))]
#[cfg_attr(feature = "diesel", diesel(sql_type = diesel::sql_types::Text))]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "diesel", derive(diesel::AsExpression, diesel::FromSqlRow))]
#[cfg_attr(feature = "diesel", diesel(sql_type = diesel::sql_types::Text))]
pub enum SystemPhase {
    /// Starting.
    Starting,
    /// Waiting.
    #[default]
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
