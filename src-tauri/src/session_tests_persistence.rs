//! Tests for session_persistence.rs — flag + credit persistence across restarts.

use crate::session_persistence::PersistedSessionState;
use crate::session_manager::SessionManager;
use crate::db_sessions::TodayTotals;

#[test]
fn from_session_captures_all_flags() {
    let mut m = SessionManager::new();
    m.alert_fired = true;
    m.notify_inactivity_fired = true;
    m.state.sitting_seconds = 500;
    m.state.daily_score = 3.5;

    let p = PersistedSessionState::from_session(&m);
    assert!(p.alert_fired);
    assert!(p.notify_inactivity_fired);
    assert!(!p.stand_alert_fired);
    assert_eq!(p.sitting_seconds, 500);
    assert!((p.daily_score - 3.5).abs() < f32::EPSILON);
}

#[test]
fn load_restores_credited_sitting_seconds() {
    let mut m = SessionManager::new();
    m.load_today_totals(&TodayTotals { sitting_secs: 2400, standing_secs: 720, position_changes: 2 });
    assert_eq!(m.state.sitting_seconds, 2400);

    let persisted = PersistedSessionState {
        date_local: chrono::Local::now().format("%Y-%m-%d").to_string(),
        sitting_seconds: 960,
        daily_score: 2.0_f32,
        alert_fired: true,
        stand_alert_fired: false,
        notify_inactivity_fired: false,
        notify_posture_balance_fired: false,
        praise_halfway_fired_today: false,
        standing_target_reached_fired: false,
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

    let persisted = PersistedSessionState {
        date_local: chrono::Local::now().format("%Y-%m-%d").to_string(),
        sitting_seconds: 2000,
        daily_score: 0.0,
        alert_fired: false,
        stand_alert_fired: false,
        notify_inactivity_fired: false,
        notify_posture_balance_fired: false,
        praise_halfway_fired_today: false,
        standing_target_reached_fired: false,
    };
    m.load_persisted_state(&persisted);

    assert_eq!(m.state.sitting_seconds, 1000);
}

#[test]
fn stale_date_is_detected() {
    let state = PersistedSessionState {
        date_local: "2020-01-01".to_string(),
        sitting_seconds: 100,
        daily_score: 0.0,
        alert_fired: true,
        stand_alert_fired: false,
        notify_inactivity_fired: false,
        notify_posture_balance_fired: false,
        praise_halfway_fired_today: false,
        standing_target_reached_fired: false,
    };
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    assert_ne!(state.date_local, today);
}

/// Full scenario: sit 40min → stand 12min → credit → persist → restart → restore → sit 5min.
#[test]
fn round_trip_sit_stand_restart_sit() {
    use chrono::{Duration, Utc};
    use crate::session_types::DeskState;

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

    let persisted = PersistedSessionState::from_session(&m);
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

    let stale = PersistedSessionState {
        date_local: "2020-01-01".to_string(),
        sitting_seconds: 5000,
        daily_score: 10.0,
        alert_fired: true,
        stand_alert_fired: true,
        notify_inactivity_fired: true,
        notify_posture_balance_fired: true,
        praise_halfway_fired_today: true,
        standing_target_reached_fired: true,
    };

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    assert_ne!(stale.date_local, today);
    assert!(!m.alert_fired);
    assert!(!m.notify_inactivity_fired);
    assert_eq!(m.state.sitting_seconds, 0);
}

/// Serde round-trip: from_session → JSON → deserialize matches.
#[test]
fn serde_round_trip() {
    let mut m = SessionManager::new();
    m.alert_fired = true;
    m.state.sitting_seconds = 1234;
    m.state.daily_score = 4.2;
    m.notify_posture_balance_fired = true;

    let original = PersistedSessionState::from_session(&m);
    let json = serde_json::to_value(&original).unwrap();
    let restored: PersistedSessionState = serde_json::from_value(json).unwrap();

    assert_eq!(original.sitting_seconds, restored.sitting_seconds);
    assert_eq!(original.alert_fired, restored.alert_fired);
    assert_eq!(original.notify_posture_balance_fired, restored.notify_posture_balance_fired);
    assert!((original.daily_score - restored.daily_score).abs() < f32::EPSILON);
}
