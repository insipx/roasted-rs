use serde::{Deserialize, Serialize};

use super::*;

/// Process snapshot emitted by the firmware, including the last finished process.
///
/// Only activity is emitted for processes other than brewing and grinding.
/// A snapshot replaces the previous process object; missing details must not
/// inherit a previous brew's phase or elapsed time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "diesel", derive(diesel::Insertable))]
#[cfg_attr(feature = "diesel", diesel(table_name = roasted_db_schema::schema::gaggimate_status_frames))]
#[cfg_attr(feature = "diesel", diesel(check_for_backend(diesel::sqlite::Sqlite)))]
pub struct ProcessStatus {
    /// Controller activity, encoded as integer 0 or 1 rather than a JSON boolean.
    #[serde(rename = "a")]
    #[cfg_attr(feature = "diesel", diesel(column_name = "process_activity"))]
    activity: ProcessActivity,
    /// Current phase category; infusion and brew belong to the same shot.
    #[serde(rename = "s", default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "diesel", diesel(column_name = "process_phase"))]
    pub phase: Option<ProcessPhase>,
    /// Display label; do not use this user-facing text for lifecycle detection.
    #[serde(rename = "l", default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "diesel", diesel(column_name = "process_label"))]
    pub label: Option<String>,
    /// Elapsed process time in milliseconds.
    #[cfg_attr(feature = "diesel", diesel(column_name = "process_elapsed_ms"))]
    #[serde(rename = "e", default, skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: Option<ElapsedMs>,
    /// Utility profile flag (0 or 1); absent on older firmware and grind processes.
    #[cfg_attr(feature = "diesel", diesel(column_name = "process_utility"))]
    #[serde(rename = "u", default, skip_serializing_if = "Option::is_none")]
    pub utility: Option<UtilityFlag>,
    /// Whether phase progress is measured by time or volume.
    #[cfg_attr(feature = "diesel", diesel(column_name = "process_target_type"))]
    #[serde(rename = "tt", default, skip_serializing_if = "Option::is_none")]
    pub target_type: Option<ProcessTarget>,
    /// Phase target: milliseconds for time, volume for volumetric mode.
    #[cfg_attr(feature = "diesel", diesel(column_name = "process_phase_target"))]
    #[serde(rename = "pt", default, skip_serializing_if = "Option::is_none")]
    pub phase_target: Option<f64>,
    /// Phase progress in the same units as the target.
    #[cfg_attr(feature = "diesel", diesel(column_name = "process_phase_progress"))]
    #[serde(rename = "pp", default, skip_serializing_if = "Option::is_none")]
    pub phase_progress: Option<f64>,
}

impl ProcessStatus {
    /// Returns `true` if the status is `ProcessActivity::Active`
    pub fn is_active(&self) -> bool {
        self.activity == ProcessActivity::Active
    }
}
