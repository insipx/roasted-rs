// @generated automatically by Diesel CLI.

diesel::table! {
    gaggimate_status_frames (id) {
        id -> Nullable<Integer>,
        shot_id -> Binary,
        process_activity -> Nullable<Text>,
        process_phase -> Nullable<Text>,
        process_label -> Nullable<Text>,
        process_elapsed_ms -> Nullable<BigInt>,
        process_utility -> Nullable<Integer>,
        process_target_type -> Nullable<Text>,
        process_phase_target -> Nullable<Double>,
        process_phase_progress -> Nullable<Double>,
        current_temperature -> Double,
        target_temperature -> Double,
        current_pressure -> Double,
        current_flow -> Double,
        current_weight -> Double,
        bluetooth_weight -> Double,
        scale_connected -> Bool,
        system_phase -> Text,
        message -> Nullable<Text>,
        error_code -> Nullable<BigInt>,
        target_pressure -> Double,
        machine_mode -> Text,
        profile_label -> Text,
        pressure_capable -> Bool,
        dimming_capable -> Bool,
    }
}
