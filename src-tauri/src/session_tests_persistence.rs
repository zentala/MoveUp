//! Tests for session_persistence.rs — one versioned snapshot of the engine.
//!
//! Covers the four paths a snapshot can take: a v1 round trip that carries the
//! whole `SessionState`, a v0 (unversioned) file being migrated on read, a
//! corrupt value, and a stale-date file.

use crate::db_sessions::TodayTotals;
use crate::session_manager::SessionManager;
use crate::session_persistence::{
    PersistedEngineState, PersistedSessionState, ENGINE_SCHEMA_VERSION,
};

/// A v0 snapshot as written by every build up to v0.6.0: no `schema_version`,
/// no `engine`, no hourly-tracker keys.
fn legacy_v0_json(date_local: &str, sitting_seconds: i64) -> serde_json::Value {
    serde_json::json!({
        "date_local": date_local,
        "sitting_seconds": sitting_seconds,
        "daily_score": 2.0,
        "alert_fired": true,
        "stand_alert_fired": false,
        "notify_inactivity_fired": false,
        "notify_posture_balance_fired": false,
        "praise_halfway_fired_today": false,
        "standing_target_reached_fired": false
    })
}

fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

#[test]
fn from_session_captures_all_flags() {
    let mut m = SessionManager::new();
    m.alert_fired = true;
    m.notify_inactivity_fired = true;
    m.state.sitting_seconds = 500;
    m.state.daily_score = 3.5;

    let p = PersistedEngineState::from_session(&m, None);
    assert_eq!(p.schema_version, ENGINE_SCHEMA_VERSION);
    assert!(p.alert_fired);
    assert!(p.notify_inactivity_fired);
    assert!(!p.stand_alert_fired);
    assert_eq!(p.sitting_seconds, 500);
    assert!((p.daily_score - 3.5).abs() < f32::EPSILON);
}

/// The point of the unified snapshot: a `SessionState` field nobody mirrored
/// by hand still survives a restart.
#[test]
fn engine_snapshot_carries_unmirrored_state_fields() {
    let mut m = SessionManager::new();
    m.state.longest_computer_session_secs = 4321;
    m.state.away_bout_secs = 77;
    m.state.lap_bonus_awarded_for_lap = 3;
    m.state.last_sitting_secs = 654;

    let p = PersistedEngineState::from_session(&m, None);
    let engine = p.engine.as_ref().expect("v1 snapshot must carry the engine");
    assert_eq!(engine.longest_computer_session_secs, 4321);

    let mut m2 = SessionManager::new();
    m2.load_persisted_state(&p);
    assert_eq!(m2.state.longest_computer_session_secs, 4321);
    assert_eq!(m2.state.away_bout_secs, 77);
    assert_eq!(m2.state.lap_bonus_awarded_for_lap, 3);
    assert_eq!(m2.state.last_sitting_secs, 654);
}

/// Restoring must not credit the time the app spent shut down as sitting time.
#[test]
fn restore_drops_live_timer_anchors() {
    use crate::session_types::DeskState;
    let mut m = SessionManager::new();
    m.state.state = DeskState::Sitting;
    m.state.sitting_started = Some(chrono::Utc::now() - chrono::Duration::hours(9));
    m.state.last_tick_ts = Some(chrono::Utc::now());

    let p = PersistedEngineState::from_session(&m, None);
    let mut m2 = SessionManager::new();
    m2.load_persisted_state(&p);

    assert!(m2.state.sitting_started.is_none());
    assert!(m2.state.last_tick_ts.is_none());
    assert!(m2.state.standing_bout_started.is_none());
}

#[test]
fn load_restores_credited_sitting_seconds() {
    let mut m = SessionManager::new();
    m.load_today_totals(&TodayTotals { sitting_secs: 2400, standing_secs: 720, position_changes: 2 });
    assert_eq!(m.state.sitting_seconds, 2400);

    let persisted = PersistedEngineState {
        date_local: today(),
        sitting_seconds: 960,
        daily_score: 2.0_f32,
        alert_fired: true,
        ..PersistedEngineState::empty_for_today()
    };
    m.load_persisted_state(&persisted);

    assert_eq!(m.state.sitting_seconds, 960);
    assert!((m.state.daily_score - 2.0).abs() < f32::EPSILON);
    assert!(m.alert_fired);
    assert!(!m.stand_alert_fired);
}

#[test]
fn load_ignores_higher_persisted_sitting() {
    let mut m = SessionManager::new();
    m.load_today_totals(&TodayTotals { sitting_secs: 1000, standing_secs: 500, position_changes: 1 });

    let persisted = PersistedEngineState {
        sitting_seconds: 2000,
        ..PersistedEngineState::empty_for_today()
    };
    m.load_persisted_state(&persisted);

    assert_eq!(m.state.sitting_seconds, 1000);
}

/// The DB saw sessions this process did not, so it wins on raw daily totals
/// even when the snapshot carries a full engine.
#[test]
fn db_totals_win_over_stale_engine_totals() {
    let mut m = SessionManager::new();
    m.state.standing_seconds = 100;
    m.state.position_changes = 1;
    m.state.sitting_seconds_total = 200;
    let persisted = PersistedEngineState::from_session(&m, None);

    let mut m2 = SessionManager::new();
    m2.load_today_totals(&TodayTotals { sitting_secs: 900, standing_secs: 800, position_changes: 6 });
    m2.load_persisted_state(&persisted);

    assert_eq!(m2.state.standing_seconds, 800);
    assert_eq!(m2.state.position_changes, 6);
    assert_eq!(m2.state.sitting_seconds_total, 900);
}

// ─── Migration: v0 (unversioned) → v1 ────────────────────────────────────────

#[test]
fn v0_json_migrates_to_current_version() {
    let state = PersistedEngineState::from_stored_value(legacy_v0_json(&today(), 640))
        .expect("a v0 snapshot must still be readable");

    assert_eq!(state.schema_version, ENGINE_SCHEMA_VERSION);
    assert!(state.engine.is_none(), "v0 files carry no engine");
    assert_eq!(state.sitting_seconds, 640);
    assert!(state.alert_fired);
    assert!(state.hours_with_break.is_empty());
    assert_eq!(state.current_away_secs, 0);
}

/// A migrated v0 snapshot must restore exactly what it used to restore.
#[test]
fn migrated_v0_still_restores_credit_and_flags() {
    let state = PersistedEngineState::from_stored_value(legacy_v0_json(&today(), 640)).unwrap();
    let mut m = SessionManager::new();
    m.load_today_totals(&TodayTotals { sitting_secs: 1800, standing_secs: 300, position_changes: 2 });
    m.load_persisted_state(&state);

    assert_eq!(m.state.sitting_seconds, 640);
    assert!(m.alert_fired);
    assert!((m.state.daily_score - 2.0).abs() < f32::EPSILON);
}

#[test]
fn corrupt_value_is_rejected_not_silently_defaulted() {
    let corrupt = serde_json::json!({ "schema_version": "not-a-number" });
    assert!(PersistedEngineState::from_stored_value(corrupt).is_none());
}

#[test]
fn stale_date_is_detected() {
    let state = PersistedEngineState::from_stored_value(legacy_v0_json("2020-01-01", 100)).unwrap();
    assert_ne!(state.date_local, today());
}

/// Historical alias still resolves — `commands.rs` and `setup_helpers.rs`
/// import it by that name.
#[test]
fn legacy_type_alias_is_the_same_type() {
    let via_alias: PersistedSessionState = PersistedEngineState::empty_for_today();
    assert_eq!(via_alias.schema_version, ENGINE_SCHEMA_VERSION);
}

// ─── Full scenarios ──────────────────────────────────────────────────────────

/// Sit 40min → stand 12min → credit → persist → restart → restore → sit 5min.
#[test]
fn round_trip_sit_stand_restart_sit() {
    use crate::session_types::DeskState;
    use chrono::{Duration, Utc};

    let mut m = SessionManager::new();
    let t0 = Utc::now() - Duration::minutes(52);

    // Sit 40 min.
    m.state.state = DeskState::Sitting;
    m.state.sitting_started = Some(t0);
    let t1 = t0 + Duration::minutes(40);
    m.handle_state_exit(&DeskState::Standing, t1);
    m.state.state = DeskState::Standing;
    assert_eq!(m.state.sitting_seconds, 2400);

    // Stand 12 min → credit = 720 * 2.0 = 1440.
    let t2 = t1 + Duration::minutes(12);
    m.state.standing_bout_started = Some(t1);
    m.handle_state_exit(&DeskState::Sitting, t2);
    m.state.state = DeskState::Sitting;
    m.state.sitting_started = Some(t2);
    assert_eq!(m.state.sitting_seconds, 960);

    let persisted = PersistedEngineState::from_session(&m, None);
    assert_eq!(persisted.sitting_seconds, 960);

    // --- Restart ---
    let mut m2 = SessionManager::new();
    m2.load_today_totals(&TodayTotals {
        sitting_secs: 2400,
        standing_secs: 720,
        position_changes: 2,
    });
    assert_eq!(m2.state.sitting_seconds, 2400);

    m2.load_persisted_state(&persisted);
    assert_eq!(m2.state.sitting_seconds, 960);

    // Sit 5 more min → live = 960 + 300 = 1260.
    let now = Utc::now();
    m2.state.state = DeskState::Sitting;
    m2.state.sitting_started = Some(now - Duration::minutes(5));
    let live = m2.get_live_sitting_seconds(now);
    assert!(live >= 1258 && live <= 1262, "expected ~1260, got {}", live);
}

/// Daily reset + stale persistence → flags stay cleared.
#[test]
fn daily_reset_clears_flags_even_with_stale_persistence() {
    let mut m = SessionManager::new();
    m.alert_fired = true;
    m.notify_inactivity_fired = true;
    m.standing_target_reached_fired = true;

    // Simulate daily reset.
    m.state.sitting_seconds = 0;
    m.alert_fired = false;
    m.stand_alert_fired = false;
    m.notify_inactivity_fired = false;
    m.notify_posture_balance_fired = false;
    m.praise_halfway_fired_today = false;
    m.standing_target_reached_fired = false;

    let stale = PersistedEngineState {
        date_local: "2020-01-01".to_string(),
        sitting_seconds: 5000,
        daily_score: 10.0,
        alert_fired: true,
        standing_target_reached_fired: true,
        ..PersistedEngineState::empty_for_today()
    };

    assert_ne!(stale.date_local, today());
    assert!(!m.alert_fired);
    assert!(!m.notify_inactivity_fired);
    assert_eq!(m.state.sitting_seconds, 0);
}

/// Serde round-trip through JSON: engine, tracker and flags all survive.
#[test]
fn serde_round_trip() {
    let mut m = SessionManager::new();
    m.alert_fired = true;
    m.state.sitting_seconds = 1234;
    m.state.daily_score = 4.2;
    m.notify_posture_balance_fired = true;
    for _ in 0..300 {
        m.hourly_break_tracker.tick_away(10);
    }
    m.hourly_break_tracker.tick_active(11);

    let original = PersistedEngineState::from_session(&m, None);
    let json = serde_json::to_value(&original).unwrap();
    let restored = PersistedEngineState::from_stored_value(json).unwrap();

    assert_eq!(original.schema_version, restored.schema_version);
    assert_eq!(original.sitting_seconds, restored.sitting_seconds);
    assert_eq!(original.alert_fired, restored.alert_fired);
    assert_eq!(original.notify_posture_balance_fired, restored.notify_posture_balance_fired);
    assert!((original.daily_score - restored.daily_score).abs() < f32::EPSILON);
    assert_eq!(original.hours_with_break, restored.hours_with_break);
    assert_eq!(original.hours_active, restored.hours_active);
    assert_eq!(original.current_away_secs, restored.current_away_secs);
    assert_eq!(
        restored.engine.map(|e| e.sitting_seconds),
        Some(1234),
        "the engine must round-trip through JSON, not just the mirrored scalars"
    );
}

// Break tracker persistence tests in session_tests_break_tracker.rs
