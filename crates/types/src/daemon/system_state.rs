use super::*;

/// State snapshot about the overall Gaggimate system.
#[derive(Default, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "diesel", derive(diesel::Insertable))]
#[cfg_attr(feature = "diesel", diesel(table_name = roasted_db_schema::schema::gaggimate_status_frames))]
#[cfg_attr(feature = "diesel", diesel(check_for_backend(diesel::sqlite::Sqlite)))]
pub struct GaggimateSystemState {
    ///  Current phase.
    #[cfg_attr(feature = "diesel", diesel(column_name = "system_phase"))]
    pub phase: SystemPhase,
    /// Display message
    pub message: Option<String>,
    /// Controller error code if any
    pub error_code: Option<i64>,
}

