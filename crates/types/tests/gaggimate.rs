use roasted_types::ws::gaggimate::{
    MachineMode, Message, Patch, ProcessActivity, Status, WarningLevel,
};
use serde_json::json;

#[test]
fn status_preserves_absent_null_and_zero() {
    let message: Message =
        serde_json::from_value(json!({"tp": "evt:status", "ct": null, "pr": 0.0})).unwrap();
    let Message::Status(status) = message else {
        panic!("expected status");
    };
    assert_eq!(
        (status.current_temperature, status.target_temperature, status.current_pressure),
        (Patch::Null, Patch::Absent, Patch::Value(0.0))
    );
}

#[test]
fn status_round_trip_keeps_false_empty_and_null_values() {
    let original = json!({
        "tp": "evt:status", "pr": 0.0, "cp": false, "p": "", "warn": [],
        "ct": null, "sys": {"s": "ready", "m": "", "c": 0}
    });
    let message: Message = serde_json::from_value(original.clone()).unwrap();
    assert_eq!(serde_json::to_value(message).unwrap(), original);
}

#[test]
fn absent_status_fields_are_not_serialized_as_null() {
    assert_eq!(serde_json::to_value(Status::default()).unwrap(), json!({}));
}

#[test]
fn machine_modes_use_firmware_numeric_values() {
    for (wire, mode) in [
        (0, MachineMode::Standby),
        (1, MachineMode::Brew),
        (2, MachineMode::Steam),
        (3, MachineMode::HotWater),
        (4, MachineMode::Grind),
    ] {
        let original = json!({"tp": "evt:status", "m": wire});
        let message: Message = serde_json::from_value(original.clone()).unwrap();
        let Message::Status(ref status) = message else { panic!("expected status") };
        assert_eq!(status.machine_mode, Patch::Value(mode));
        assert_eq!(serde_json::to_value(message).unwrap(), original);
    }
}

#[test]
fn machine_mode_rejects_unknown_numbers_and_non_numeric_values() {
    for invalid in [json!(5), json!(-1), json!("brew"), json!(true)] {
        assert!(serde_json::from_value::<MachineMode>(invalid).is_err());
    }
}

#[test]
fn scale_readings_round_trip_including_cup_removal_and_disconnect() {
    for original in [
        json!({"tp": "evt:status", "cw": 38.09, "bw": 38.09, "bc": true}),
        json!({"tp": "evt:status", "cw": -76.5, "bw": -76.5, "bc": true}),
        json!({"tp": "evt:status", "cw": 0.0, "bw": 0.0, "bc": false}),
        json!({"tp": "evt:status", "cw": null, "bw": null, "bc": null}),
    ] {
        let message: Message = serde_json::from_value(original.clone()).unwrap();
        assert_eq!(serde_json::to_value(message).unwrap(), original);
    }
}

#[test]
fn scale_telemetry_does_not_imply_connection_state() {
    let status: Status = serde_json::from_value(json!({"cw": 0.0, "bw": null})).unwrap();
    assert_eq!(
        (status.current_weight, status.bluetooth_weight, status.scale_connected),
        (Patch::Value(0.0), Patch::Null, Patch::Absent)
    );
}

#[test]
fn process_snapshots_round_trip_active_phases_and_fractional_volume() {
    for phase in ["infusion", "brew", "grind"] {
        let original = json!({"tp": "evt:status", "process": {
            "a": 1, "s": phase, "l": "Running", "e": 1500,
            "tt": "volumetric", "pt": 36.5, "pp": 2.25
        }});
        let message: Message = serde_json::from_value(original.clone()).unwrap();
        assert_eq!(serde_json::to_value(message).unwrap(), original);
    }
}

#[test]
fn process_utility_flag_preserves_unknown_false_and_true() {
    for utility in [None, Some(0), Some(1)] {
        let mut original = json!({"process": {
            "a": 1, "s": "brew", "l": "Running", "e": 0,
            "tt": "time", "pt": 1000, "pp": 0
        }});
        if let Some(value) = utility {
            original["process"]["u"] = json!(value);
        }
        let status: Status = serde_json::from_value(original).unwrap();
        let Patch::Value(process) = status.process else { panic!("expected process") };
        assert_eq!(process.utility.map(|value| *value), utility);
    }
}

#[test]
fn process_patch_distinguishes_missing_from_cleared() {
    let absent: Status = serde_json::from_value(json!({"pr": 0})).unwrap();
    let cleared: Status = serde_json::from_value(json!({"process": null})).unwrap();
    assert_eq!((absent.process, cleared.process), (Patch::Absent, Patch::Null));
}

#[test]
fn process_activity_requires_zero_or_one_integer() {
    for invalid in [json!(true), json!(2), json!("1")] {
        assert!(serde_json::from_value::<ProcessActivity>(invalid).is_err());
    }
}

#[test]
fn activity_only_process_does_not_require_brew_details() {
    for activity in [0, 1] {
        let original = json!({"tp": "evt:status", "process": {"a": activity}});
        let message: Message = serde_json::from_value(original.clone()).unwrap();
        assert_eq!(serde_json::to_value(message).unwrap(), original);
    }
}

#[test]
fn process_without_activity_is_not_assumed_inactive() {
    assert!(
        serde_json::from_value::<Message>(json!({
            "tp": "evt:status", "process": {}
        }))
        .is_err()
    );
}

#[test]
fn invalid_status_measurement_is_rejected() {
    assert!(
        serde_json::from_value::<Message>(json!({
            "tp": "evt:status", "pr": "high"
        }))
        .is_err()
    );
}

#[test]
fn missing_message_type_is_rejected() {
    assert!(serde_json::from_value::<Message>(json!({"pr": 8.5})).is_err());
}

#[test]
fn warning_severity_uses_json_integers() {
    let warning: WarningLevel = serde_json::from_str("2").unwrap();
    assert_eq!(warning, WarningLevel::Error);
    assert!(serde_json::from_str::<WarningLevel>("3").is_err());
    assert!(serde_json::from_str::<WarningLevel>("\"error\"").is_err());
}

#[test]
fn unsigned_domain_types_preserve_wire_ranges_and_deref() {
    use roasted_types::ws::gaggimate::{ElapsedMs, UtilityFlag};

    let elapsed: ElapsedMs = serde_json::from_value(json!(u64::MAX)).unwrap();
    let utility: UtilityFlag = serde_json::from_value(json!(u8::MAX)).unwrap();
    let elapsed_ref: &u64 = &elapsed;
    let utility_ref: &u8 = &utility;
    assert_eq!(*elapsed_ref, u64::MAX);
    assert_eq!(*utility_ref, u8::MAX);
    assert_eq!(serde_json::to_value(elapsed).unwrap(), json!(u64::MAX));
    assert_eq!(serde_json::to_value(utility).unwrap(), json!(u8::MAX));
    assert!(serde_json::from_value::<ElapsedMs>(json!(-1)).is_err());
    assert!(serde_json::from_value::<UtilityFlag>(json!(-1)).is_err());
    assert!(serde_json::from_value::<UtilityFlag>(json!(256)).is_err());
}
