//! Source builder functions for commands_catalog. Extracted to respect the 250-line cap.
#![allow(clippy::too_many_lines)]

use crate::commands_catalog::{DataSource, FieldMeta};

pub(super) fn f(name: &str, ty: &str, desc: &str) -> FieldMeta {
    FieldMeta { name: name.into(), ty: ty.into(), description: desc.into() }
}

pub(super) fn sensor_source() -> DataSource {
    DataSource {
        id: "sensor".into(),
        name: "Hardware sensor stream".into(),
        kind: "sensor".into(),
        location: "COM port (in-memory stream)".into(),
        retention: "in-memory, no persistence".into(),
        fields: vec![
            f("mm", "u32", "Raw distance reading in millimeters."),
            f("cm", "f32", "Distance converted to centimeters."),
            f("timestamp", "string", "ISO 8601 UTC timestamp of the reading."),
        ],
        sample_row: Some(
            r#"{"mm":820,"cm":82.0,"timestamp":"2026-05-16T09:14:22Z"}"#.into(),
        ),
        description:
            "VL53L1X ToF sensor stream over USB serial (baud 115200). Drives state detection."
                .into(),
    }
}

pub(super) fn sqlite_sessions_source() -> DataSource {
    DataSource {
        id: "sqlite_sessions".into(),
        name: "Sessions database".into(),
        kind: "sqlite".into(),
        location: "{app_data_dir}/desk.db (table: sessions)".into(),
        retention: "no limit (manual backups in backups/)".into(),
        fields: vec![
            f("id", "i64", "Primary key."),
            f("started_at", "string", "RFC3339 timestamp of session start."),
            f("ended_at", "string", "RFC3339 timestamp of session end."),
            f("state", "string", "Sitting | Standing | Walking | Away."),
            f("duration_seconds", "i64", "Total session length in seconds."),
            f("sitting_seconds", "i64", "Sitting time accumulated in this session."),
            f("standing_seconds", "i64", "Standing time accumulated in this session."),
            f("position_changes", "i64", "State transitions in this session."),
            f("session_limit_secs", "i64", "Active sit limit at session time."),
            f("date_local", "string", "Local YYYY-MM-DD bucket."),
        ],
        sample_row: Some(
            r#"{"id":42,"started_at":"2026-05-16T08:00:00Z","ended_at":"2026-05-16T08:42:11Z","state":"Sitting","duration_seconds":2531,"sitting_seconds":2531,"standing_seconds":0,"position_changes":0,"session_limit_secs":2400,"date_local":"2026-05-16"}"#
                .into(),
        ),
        description: "Persistent history of completed sessions, written by db_sessions.".into(),
    }
}

pub(super) fn snapshots_source() -> DataSource {
    DataSource {
        id: "snapshots".into(),
        name: "Minute snapshots".into(),
        kind: "json".into(),
        location: "{app_data_dir}/logs/YYYY-MM-DD/HH-MM.json".into(),
        retention: "7 days (auto-pruned on startup)".into(),
        fields: vec![
            f("ts", "string", "RFC3339 timestamp."),
            f("state", "string", "Sitting | Standing | Walking | Away."),
            f("sitting_seconds", "i64", "Sitting seconds in current session."),
            f("standing_seconds", "i64", "Standing seconds in current session."),
            f("break_seconds", "i64", "Accumulated break seconds."),
            f("session_limit_secs", "i64", "Active sit limit."),
            f("stand_limit_secs", "i64", "Active stand limit."),
            f("desk_height_cm", "f32", "Desk height in cm."),
            f("position_changes", "i64", "Position changes today."),
            f("sitting_seconds_total", "i64", "Sitting seconds today (cumulative)."),
            f("idle_secs", "i64", "Seconds since last input."),
            f("away_bout_secs", "i64", "Current away bout length."),
            f("continuous_computer_secs", "i64", "Time at computer (sitting + standing)."),
            f("current_session_secs", "i64", "Length of active session."),
            f("standing_session_secs", "i64", "Length of active standing session."),
            f("connected", "bool", "Sensor connected at sample time."),
            f("port", "string", "COM port name."),
            f("version", "string", "App version that wrote the file."),
            f("metrics", "array", "Array of derived metric snapshots."),
        ],
        sample_row: Some(
            r#"{"ts":"2026-05-16T09:14:00Z","state":"Sitting","sitting_seconds":840,"standing_seconds":0,"desk_height_cm":72.0,"connected":true,"port":"COM3","version":"0.4.0"}"#
                .into(),
        ),
        description: "Per-minute JSON state snapshots written by snapshot_logger.".into(),
    }
}

pub(super) fn events_log_source() -> DataSource {
    DataSource {
        id: "events_log".into(),
        name: "Event log".into(),
        kind: "log".into(),
        location: "{app_data_dir}/logs/YYYY-MM-DD/events.log".into(),
        retention: "7 days (auto-pruned on startup)".into(),
        fields: vec![
            f("time", "string", "Local HH:MM:SS timestamp."),
            f("type", "string", "STATE | CREDIT | NOTIF | DEVICE | ALERT | RESET | START | AUTOSTART."),
            f("details", "string", "Free-form event payload (event-type dependent)."),
        ],
        sample_row: Some(
            "09:14:22 STATE Sitting -> Standing (height 108cm, debounce 5)".into(),
        ),
        description:
            "Append-only text log of state transitions, notifications, device events.".into(),
    }
}

pub(super) fn profiles_ergonomic_source() -> DataSource {
    DataSource {
        id: "profiles_ergonomic".into(),
        name: "Ergonomic profiles".into(),
        kind: "config".into(),
        location: "{app_data_dir}/profiles/ergonomic/*.json".into(),
        retention: "persistent (hot-reloadable)".into(),
        fields: vec![
            f("id", "string", "Profile identifier (filename without extension)."),
            f("name", "string", "Human label shown in UI."),
            f("session_limit_secs", "i64", "Sit session limit before alert."),
            f("standing_max_secs", "i64", "Stand session limit before alert."),
            f("break_min_secs", "i64", "Minimum seconds to qualify as a break."),
            f("break_credit_multiplier", "f32", "Seconds of sitting cancelled per break second."),
            f("scoring", "object", "Per-minute point rules and bonuses."),
            f("kpi_thresholds", "object", "Daily targets and color bands."),
        ],
        sample_row: Some(
            r#"{"id":"standard","name":"Standard","session_limit_secs":2400,"standing_max_secs":5400,"break_min_secs":60,"break_credit_multiplier":2.0}"#
                .into(),
        ),
        description:
            "User-editable ergonomic policy files. Built-ins: standard, strict, relaxed, demo."
                .into(),
    }
}

pub(super) fn profiles_communication_source() -> DataSource {
    DataSource {
        id: "profiles_communication".into(),
        name: "Communication profiles".into(),
        kind: "config".into(),
        location: "{app_data_dir}/profiles/communication/*.json".into(),
        retention: "persistent (hot-reloadable)".into(),
        fields: vec![
            f("id", "string", "Profile identifier (filename without extension)."),
            f("name", "string", "Human label shown in UI."),
            f("escalation", "array", "Ordered escalation steps (yellow/red/blink/pulse/toast)."),
            f("snooze", "object", "Notification cooldown schedule (e.g. [0,300,900,1800])."),
            f("channels", "object", "Per-channel toggles (tray, overlay, toast, popup)."),
            f("messages", "object", "Templated strings for each notification type."),
        ],
        sample_row: Some(
            r#"{"id":"default","name":"Default","snooze":{"notify_cooldowns_secs":[0,300,900,1800]}}"#
                .into(),
        ),
        description:
            "User-editable notification policy. Built-ins: default, aggressive, gentle, silent, demo."
                .into(),
    }
}

pub(super) fn store_source() -> DataSource {
    DataSource {
        id: "store".into(),
        name: "Tauri store".into(),
        kind: "config".into(),
        location: "tauri-plugin-store (key-value JSON on disk)".into(),
        retention: "persistent until explicit reset".into(),
        fields: vec![
            f("sitting_mm", "u32", "Calibrated sitting distance in mm."),
            f("standing_mm", "u32", "Calibrated standing distance in mm."),
            f("desk_thickness_mm", "u32", "Desk thickness offset in mm."),
            f("persisted_session_state", "object", "Session state across restarts (date-guarded)."),
            f("reset_after", "string", "RFC3339 timestamp of next scheduled reset."),
            f("active_ergonomic_profile", "string", "ID of currently active ergonomic profile."),
            f("active_communication_profile", "string", "ID of currently active comm profile."),
        ],
        sample_row: Some(
            r#"{"sitting_mm":820,"standing_mm":480,"desk_thickness_mm":25,"active_ergonomic_profile":"standard"}"#
                .into(),
        ),
        description: "Key-value store for calibration, active profiles, and session persistence."
            .into(),
    }
}

pub(super) fn remote_ws_source() -> DataSource {
    DataSource {
        id: "remote_ws".into(),
        name: "Remote WebSocket broadcast".into(),
        kind: "websocket".into(),
        location: "WebSocket on 0.0.0.0:3390 (in-memory channel, capacity 64)".into(),
        retention: "in-memory, dropped on disconnect".into(),
        fields: vec![
            f("type", "string", "snapshot | desk:state-changed | desk:device-connected | desk:device-lost | desk:daily-reset | heartbeat."),
            f("payload", "object", "Message-type-specific payload (see ws_broadcaster.rs)."),
            f("ts", "string", "Server-side RFC3339 timestamp."),
        ],
        sample_row: Some(
            r#"{"type":"desk:state-changed","payload":{"from":"Sitting","to":"Standing"},"ts":"2026-05-16T09:14:22Z"}"#
                .into(),
        ),
        description:
            "Live broadcast of desk events to remote dashboards (phone, browser kiosk)."
                .into(),
    }
}
