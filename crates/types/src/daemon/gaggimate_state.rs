use crate::{
    daemon::GaggimateSystemState,
    ws::gaggimate::{MachineMode, ProcessStatus, WarningState},
};

/// State snapshot from Gaggimate
#[derive(Default, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "diesel", derive(diesel::Insertable))]
#[cfg_attr(feature = "diesel", diesel(table_name = roasted_db_schema::schema::gaggimate_status_frames))]
#[cfg_attr(feature = "diesel", diesel(check_for_backend(diesel::sqlite::Sqlite)))]
pub struct GaggimateState {
    /// Current or last process
    #[cfg_attr(feature = "diesel", diesel(embed))]
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
    #[cfg_attr(feature = "diesel", diesel(embed))]
    pub system_state: GaggimateSystemState,
    // use a separate table
    /// Machine warnings; an empty list is a real update.
    #[cfg_attr(feature = "diesel", diesel(skip_insertion))]
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
