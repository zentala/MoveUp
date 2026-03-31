//! Tests for HourlyBreakTracker persistence across app restarts.

use crate::session_manager::SessionManager;
use crate::session_persistence::PersistedSessionState;

/// Break tracker persists and restores correctly across restart.
#[test]
fn break_tracker_persists_across_restart() {
    let mut m = SessionManager::new();
    // Simulate: active in hour 9, then 5-min break in hour 10.
    m.hourly_break_tracker.tick_active(9);
    for _ in 0..300 {
        m.hourly_break_tracker.tick_away(10);
    }
    assert_eq!(m.hourly_break_tracker.hours_with_break(), 1);
    assert_eq!(m.hourly_break_tracker.hours_active(), 2);

    // Persist.
    let persisted = PersistedSessionState::from_session(&m, None);
    assert_eq!(persisted.hours_with_break.len(), 1);
    assert_eq!(persisted.hours_active.len(), 2);

    // Simulate restart: new session, load DB totals, then restore persisted.
    let mut m2 = SessionManager::new();
    assert_eq!(m2.state.hourly_breaks_covered, 0);
    m2.load_persisted_state(&persisted);

    assert_eq!(m2.state.hourly_breaks_covered, 1);
    assert_eq!(m2.state.hourly_breaks_active, 2);
    assert_eq!(m2.hourly_break_tracker.hours_with_break(), 1);
    assert_eq!(m2.hourly_break_tracker.hours_active(), 2);
}

/// Backward compatibility: JSON without break tracker fields deserializes with defaults.
#[test]
fn backward_compat_no_tracker_fields() {
    let json = serde_json::json!({
        "date_local": chrono::Local::now().format("%Y-%m-%d").to_string(),
        "sitting_seconds": 100,
        "daily_score": 1.0,
        "alert_fired": false,
        "stand_alert_fired": false,
        "notify_inactivity_fired": false,
        "notify_posture_balance_fired": false,
        "praise_halfway_fired_today": false,
        "standing_target_reached_fired": false
    });
    let state: PersistedSessionState = serde_json::from_value(json).unwrap();
    assert!(state.hours_with_break.is_empty());
    assert!(state.hours_active.is_empty());
    assert_eq!(state.current_away_secs, 0);
}
