use roasted_types::gaggimate::Message;

macro_rules! telemetry_snapshot {
    ($name:ident, $fixture:literal) => {
        #[test]
        fn $name() {
            let message: Message = serde_json::from_str(include_str!($fixture))
                .expect("fixture must decode as a GaggiMate message");
            insta::assert_debug_snapshot!(message);
        }
    };
}

telemetry_snapshot!(live_idle, "fixtures/gaggimate/live-idle.json");
telemetry_snapshot!(active_infusion, "fixtures/gaggimate/active-infusion.json");
telemetry_snapshot!(active_extraction, "fixtures/gaggimate/active-extraction.json");
telemetry_snapshot!(finished_shot, "fixtures/gaggimate/finished-shot.json");
telemetry_snapshot!(activity_only, "fixtures/gaggimate/activity-only.json");
telemetry_snapshot!(cup_removed, "fixtures/gaggimate/cup-removed.json");
telemetry_snapshot!(scale_disconnected, "fixtures/gaggimate/scale-disconnected.json");
telemetry_snapshot!(partial_update, "fixtures/gaggimate/partial-update.json");
telemetry_snapshot!(explicit_nulls, "fixtures/gaggimate/explicit-nulls.json");
telemetry_snapshot!(unrelated_message, "fixtures/gaggimate/unrelated-message.json");
