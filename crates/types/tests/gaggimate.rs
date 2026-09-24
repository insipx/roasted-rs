use roasted_types::gaggimate::{Message, Patch, Profile, Pump, Status, WarningLevel};
use serde_json::{Value, json};

#[test]
fn status_preserves_absent_null_and_zero() {
    let message: Message =
        serde_json::from_value(json!({"tp": "evt:status", "ct": null, "pr": 0.0})).unwrap();
    let Message::Status(status) = message else {
        panic!("expected status");
    };
    assert_eq!(
        (status.ct, status.tt, status.pr),
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
fn invalid_status_measurement_is_rejected() {
    assert!(
        serde_json::from_value::<Message>(json!({
            "tp": "evt:status", "pr": "high"
        }))
        .is_err()
    );
}

#[test]
fn future_status_fields_do_not_break_known_measurements() {
    let message: Message = serde_json::from_value(json!({
        "tp": "evt:status", "pr": 8.5, "futureSensor": {"value": 42}
    }))
    .unwrap();
    assert!(matches!(message, Message::Status(status) if status.pr == Patch::Value(8.5)));
}

#[test]
fn unknown_message_types_are_explicitly_ignored() {
    let message: Message = serde_json::from_value(json!({
        "tp": "evt:future", "payload": {"value": 42}
    }))
    .unwrap();
    assert_eq!(message, Message::Unknown);
}

#[test]
fn missing_required_fields_are_rejected() {
    for value in [
        json!({"pr": 8.5}),
        json!({"tp": "evt:ota-progress", "phase": 0}),
        json!({"tp": "evt:autotune-result"}),
        json!({"tp": "req:profiles:load"}),
        json!({"tp": "req:profiles:save"}),
        json!({"tp": "req:profiles:reorder"}),
        json!({"tp": "evt:brew:confirm"}),
    ] {
        assert!(
            serde_json::from_value::<Message>(value.clone()).is_err(),
            "{value}"
        );
    }
}

#[test]
fn warning_severity_uses_json_integers() {
    let warning: WarningLevel = serde_json::from_str("2").unwrap();
    assert_eq!(warning, WarningLevel::Error);
    assert!(serde_json::from_str::<WarningLevel>("3").is_err());
    assert!(serde_json::from_str::<WarningLevel>("\"error\"").is_err());
}

fn profile() -> Value {
    json!({
        "label": "Nine bar", "type": "pro", "description": "",
        "_notes": {"bean": "house", "tags": ["test"]},
        "phases": [{
            "name": "Brew", "phase": "brew", "valve": 1, "duration": 30.0,
            "pump": {"target": "pressure", "pressure": 9.0, "flow": -1.0},
            "transition": {"type": "ease-in-out", "duration": 2.0, "adaptive": false},
            "targets": [{"type": "volumetric", "value": 36.0, "operator": "gte"}]
        }]
    })
}

#[test]
fn advanced_profiles_round_trip_including_annotations() {
    let original = profile();
    let profile: Profile = serde_json::from_value(original.clone()).unwrap();
    assert_eq!(serde_json::to_value(profile).unwrap(), original);
}

#[test]
fn simple_pump_is_an_integer_not_a_boolean() {
    let mut original = profile();
    original["phases"][0]["pump"] = json!(100);
    let profile: Profile = serde_json::from_value(original.clone()).unwrap();
    assert_eq!(profile.phases[0].pump, Pump::Percent(100));
    original["phases"][0]["pump"] = json!(true);
    assert!(serde_json::from_value::<Profile>(original).is_err());
}

#[test]
fn every_documented_message_round_trips_its_wire_tag_and_payload() {
    let profile = profile();
    let messages = [
        json!({"tp": "evt:status", "fl": 2.1, "m": 0}),
        json!({"tp": "res:ota-settings", "latestVersion": "1", "displayVersion": "1",
            "controllerVersion": "1", "hardware": "pro", "displayUpdateAvailable": false,
            "controllerUpdateAvailable": false, "channel": "stable", "updating": false}),
        json!({"tp": "evt:ota-progress", "phase": 3, "progress": 50}),
        json!({"tp": "evt:autotune-result", "pid": "1,2,3"}),
        json!({"tp": "res:profiles:list", "rid": "1", "profiles": [profile]}),
        json!({"tp": "res:profiles:load", "rid": "1", "profile": profile}),
        json!({"tp": "res:profiles:save", "rid": "1", "profile": profile}),
        json!({"tp": "res:profiles:delete", "rid": "1", "error": "missing"}),
        json!({"tp": "res:profiles:select", "rid": "1"}),
        json!({"tp": "res:profiles:favorite", "rid": "1"}),
        json!({"tp": "res:profiles:unfavorite", "rid": "1"}),
        json!({"tp": "res:profiles:reorder", "rid": "1"}),
        json!({"tp": "evt:brew:confirm", "warn": [{"k": "scaleConnected", "l": 2}]}),
        json!({"tp": "evt:brew:confirm:cancel"}),
        json!({"tp": "res:flush:start", "rid": "1", "success": false}),
        json!({"tp": "res:flush:stop", "rid": "1", "success": true}),
        json!({"tp": "req:ota-settings", "update": false, "channel": "stable"}),
        json!({"tp": "req:ota-start", "cp": "controller"}),
        json!({"tp": "req:autotune-start", "time": 60, "samples": 10}),
        json!({"tp": "req:profiles:list", "rid": "1"}),
        json!({"tp": "req:profiles:load", "rid": "1", "id": "9bar"}),
        json!({"tp": "req:profiles:save", "rid": "1", "profile": profile}),
        json!({"tp": "req:profiles:delete", "rid": "1", "id": "9bar"}),
        json!({"tp": "req:profiles:select", "rid": "1", "id": "9bar"}),
        json!({"tp": "req:profiles:favorite", "rid": "1", "id": "9bar"}),
        json!({"tp": "req:profiles:unfavorite", "rid": "1", "id": "9bar"}),
        json!({"tp": "req:profiles:reorder", "rid": "1", "order": ["9bar"]}),
        json!({"tp": "req:process:activate", "ignoreWarnings": false}),
        json!({"tp": "req:brew:confirm:cancel"}),
        json!({"tp": "req:flush:start", "rid": "1"}),
        json!({"tp": "req:flush:stop", "rid": "1"}),
    ];
    for original in messages {
        let message: Message = serde_json::from_value(original.clone()).unwrap();
        assert_eq!(serde_json::to_value(message).unwrap(), original);
    }
}
