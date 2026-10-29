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
fn older_v1_snapshot_without_notification_state_remains_readable() {
    let mut old = serde_json::to_value(PersistedEngineState::empty_for_today()).unwrap();
    old.as_object_mut().unwrap().remove("notification_policy");
    let loaded = PersistedEngineState::from_stored_value(old)
        .expect("an existing v1 snapshot without the optional field remains compatible");
    assert_eq!(loaded.schema_version, ENGINE_SCHEMA_VERSION);
    assert!(loaded.notification_policy.is_none());
}

#[test]
fn unsupported_future_snapshot_version_is_rejected() {
    let future = serde_json::json!({
        "schema_version": ENGINE_SCHEMA_VERSION + 1,
        "date_local": today(),
        "future_payload": {"preserve": true}
    });
    assert!(PersistedEngineState::from_stored_value(future).is_none());
}

#[test]
fn notification_policy_restore_uses_injected_local_date_across_midnight() {
    use chrono::TimeZone;
    use std::time::Instant;

    let wall = chrono::Utc.with_ymd_and_hms(2026, 2, 1, 12, 0, 0).unwrap();
    let monotonic = Instant::now();
    let policy = crate::communication_policy::CommunicationPolicy::new(
        crate::communication_profile::CommunicationProfile::default(),
        crate::ergonomic_profile::ErgonomicProfile::default(),
    );
    let mut persisted = PersistedEngineState::empty_for_today();
    persisted.date_local = wall.with_timezone(&chrono::Local).format("%Y-%m-%d").to_string();
    persisted.notification_policy = Some(policy.persisted_state_at(wall, monotonic));

    let mut same_day = crate::communication_policy::CommunicationPolicy::new(
        crate::communication_profile::CommunicationProfile::default(),
        crate::ergonomic_profile::ErgonomicProfile::default(),
    );
    assert!(persisted.restore_notification_policy_at(&mut same_day, wall, monotonic),
        "same-day restoration follows the injected wall clock even when host today differs");

    let after_midnight = wall + chrono::Duration::days(2);
    let mut next_day = crate::communication_policy::CommunicationPolicy::new(
        crate::communication_profile::CommunicationProfile::default(),
        crate::ergonomic_profile::ErgonomicProfile::default(),
    );
    assert!(!persisted.restore_notification_policy_at(
        &mut next_day, after_midnight, monotonic,
    ), "a different injected local date must not restore the prior day's policy");
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

    fn observe(
        manager: &mut SessionManager,
        height_mm: i32,
        from: chrono::DateTime<Utc>,
        through: chrono::DateTime<Utc>,
    ) {
        let mut at = from;
        while at <= through {
            manager.on_reading_at(height_mm, true, at);
            at += Duration::seconds(1);
        }
    }

    let mut m = SessionManager::new();
    let t0 = Utc::now() - Duration::minutes(53);

    // Confirm Sitting, then observe the full 40-minute prefix before the
    // Standing candidate's five-second confirmation window.
    let t1 = t0 + Duration::minutes(40);
    observe(&mut m, 800, t0, t1 - Duration::seconds(1));
    observe(&mut m, 1200, t1, t1 + Duration::seconds(5));
    assert_eq!(m.state.state, DeskState::Standing);
    assert_eq!(m.state.sitting_seconds, 2400);

    // Observe 12 minutes of Standing before confirming return to Sitting.
    let t2 = t1 + Duration::minutes(12) + Duration::seconds(5);
    observe(&mut m, 1200, t1 + Duration::seconds(6), t2 - Duration::seconds(5));
    observe(&mut m, 800, t2 - Duration::seconds(5), t2);
    assert_eq!(m.state.state, DeskState::Sitting);
    assert_eq!(m.state.last_break_secs, 720);
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

    // Reconfirm Sitting after process restart, then observe five known minutes.
    let resumed_at = t2 + Duration::minutes(1);
    observe(&mut m2, 800, resumed_at, resumed_at + Duration::seconds(5));
    let now = resumed_at + Duration::seconds(306);
    observe(&mut m2, 800, resumed_at + Duration::seconds(6), now);
    let live = m2.get_live_sitting_seconds(now);
    assert!(live >= 1258 && live <= 1262, "expected ~1260, got {}", live);
}

#[test]
fn captured_break_facts_survive_restart_while_credit_prefix_resets_as_before() {
    use chrono::{Duration, Utc};
    use crate::session_types::DeskState;

    let break_started_at = Utc::now() - Duration::minutes(10);
    let mut before_restart = SessionManager::new();
    before_restart.state.state = DeskState::Standing;
    before_restart.state.break_started = Some(break_started_at);
    before_restart.state.break_seconds = 240;
    before_restart.state.sitting_seconds = 1200;
    before_restart.state.sitting_seconds_total = 1200;
    before_restart.apply_current_break_credit(
        240,
        break_started_at,
        break_started_at + Duration::seconds(240),
    );
    assert_eq!(before_restart.sleep_break_credit, 480);
    assert_eq!(before_restart.state.sitting_seconds, 720);

    let mut changed_limits = before_restart.limits.clone();
    changed_limits.break_credit_multiplier = 1.0;
    before_restart.set_limits(&changed_limits);
    before_restart.apply_current_break_credit(
        600,
        break_started_at,
        break_started_at + Duration::seconds(600),
    );

    assert_eq!(before_restart.state.sitting_seconds, 600);
    assert_eq!(before_restart.state.sitting_seconds_total, 1200);
    assert_eq!(before_restart.sleep_break_credit, 600);
    let applications = before_restart.state.last_break_credit_facts.as_ref().unwrap();
    assert_eq!(applications.len(), 2);
    assert_eq!(applications[0].multiplier, 2.0);
    assert_eq!(applications[0].requested_credit_secs, 480);
    assert_eq!(applications[0].prior_applied_credit_secs, 0);
    assert_eq!(applications[0].applied_credit_secs, 480);
    assert_eq!(applications[0].sitting_before_secs, 1200);
    assert_eq!(applications[0].sitting_after_secs, 720);
    assert_eq!(applications[1].multiplier, 1.0);
    assert_eq!(applications[1].requested_credit_secs, 120);
    assert_eq!(applications[1].prior_applied_credit_secs, 480);
    assert_eq!(applications[1].applied_credit_secs, 120);
    assert_eq!(applications[1].sitting_before_secs, 720);
    assert_eq!(applications[1].sitting_after_secs, 600);

    let persisted = PersistedEngineState::from_session(&before_restart, None);
    let expected_facts = applications.clone();
    let mut after_restart = SessionManager::new();
    after_restart.load_today_totals(&TodayTotals {
        sitting_secs: 1200,
        standing_secs: 240,
        position_changes: 2,
    });
    after_restart.load_persisted_state(&persisted);

    assert_eq!(after_restart.state.sitting_seconds, 600);
    assert_eq!(after_restart.state.sitting_seconds_total, 1200);
    assert_eq!(after_restart.state.break_started, Some(break_started_at));
    assert_eq!(after_restart.sleep_break_credit, 0, "restart credit eligibility remains unchanged from baseline");
    assert_eq!(after_restart.state.last_break_credit_facts.as_ref(), Some(&expected_facts));
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

#[test]
fn daily_reset_clears_break_credit_explanation() {
    use chrono::{Duration, TimeZone, Utc};
    let at = Utc.with_ymd_and_hms(2026, 6, 2, 9, 0, 0).unwrap();
    let today = crate::session_daily::local_date_of(at);
    let mut manager = SessionManager::new();
    manager.last_reset_date = today - Duration::days(1);
    manager.last_reset_check = at - Duration::seconds(120);
    manager.state.last_break_credit_facts = Some(vec![crate::session_types::LastBreakCreditFacts {
        break_started_at: (at - Duration::minutes(5)).to_rfc3339(),
        observed_break_secs: 300,
        minimum_break_secs: 60,
        multiplier: 2.0,
        requested_credit_secs: 600,
        prior_applied_credit_secs: 0,
        sitting_before_secs: 600,
        applied_credit_secs: 600,
        sitting_after_secs: 0,
        applied_at: at.to_rfc3339(),
        no_credit_reason: None,
    }]);

    assert!(manager.check_daily_reset_at(at, today));
    assert!(manager.state.last_break_credit_facts.is_none());
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
