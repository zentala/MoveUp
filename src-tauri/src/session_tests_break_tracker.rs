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

/// A known Away interval split across local hours must be allocated to each
/// hour separately; elapsed coverage cannot carry its threshold over 09:00.
#[test]
fn away_coverage_does_not_carry_threshold_across_injected_local_hour() {
    let mut tracker = crate::hourly_break_tracker::HourlyBreakTracker::new();

    for _ in 0..180 {
        tracker.tick_away(8); // injected local interval 08:57–09:00
    }
    for _ in 0..120 {
        tracker.tick_away(9); // injected local interval 09:00–09:02
    }

    assert_eq!(tracker.hours_with_break(), 0,
        "neither local-hour intersection reaches the five-minute threshold");
}

/// A continuous Away interval earns coverage independently in each local hour.
#[test]
fn long_away_requires_known_coverage_in_each_injected_local_hour() {
    let mut tracker = crate::hourly_break_tracker::HourlyBreakTracker::new();

    for _ in 0..300 {
        tracker.tick_away(8);
    }
    assert_eq!(tracker.hours_with_break(), 1);

    tracker.tick_away(9);
    assert_eq!(tracker.hours_with_break(), 1,
        "the first tick of a new local hour cannot inherit prior-hour coverage");

    for _ in 0..299 {
        tracker.tick_away(9);
    }
    assert_eq!(tracker.hours_with_break(), 2,
        "the next hour is covered only after its own five observed minutes");
}

#[test]
fn reading_path_allocates_irregular_and_duplicate_observations_by_injected_zone() {
    use chrono::{Duration, TimeZone, Utc};

    let start = Utc.with_ymd_and_hms(2026, 6, 1, 6, 0, 0).unwrap();
    let mut m = crate::session_manager::SessionManager::new();
    let local_hour = |at: chrono::DateTime<Utc>| {
        crate::hourly_break_tracker::LocalHourInterval::from_offset(at, 9 * 60 * 60)
    };

    for second in -5..=0 {
        m.on_reading_at_with_local_hour(800, false,
            start + Duration::seconds(second), local_hour);
    }
    for offset in [30, 30, 155, 300] {
        m.on_reading_at_with_local_hour(
            800, false, start + Duration::seconds(offset), local_hour,
        );
    }

    assert_eq!(m.hourly_break_tracker.hours_with_break(), 1,
        "the known 300-second interval qualifies despite irregular cadence and a duplicate");
    assert_eq!(m.hourly_break_tracker.hours_active(), 1,
        "the explicitly injected +09:00 zone maps these readings into hour 15");
}

#[test]
fn repeated_local_hour_fold_starts_a_distinct_away_interval() {
    use chrono::{DateTime, TimeZone, Utc};

    let start = Utc.with_ymd_and_hms(2026, 10, 25, 0, 54, 0).unwrap();
    let fold = Utc.with_ymd_and_hms(2026, 10, 25, 1, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 10, 25, 1, 6, 0).unwrap();
    let local_hour = |at: DateTime<Utc>| {
        let offset = if at < fold { 2 * 60 * 60 } else { 60 * 60 };
        crate::hourly_break_tracker::LocalHourInterval::from_offset(at, offset)
    };
    let mut tracker = crate::hourly_break_tracker::HourlyBreakTracker::new();

    // Both six-minute portions display local hour 02, but the clock fold
    // places them in different concrete local-hour intervals.
    tracker.record_known_interval(start, end, true, local_hour);

    assert_eq!(tracker.hours_active(), 1);
    assert_eq!(tracker.hours_with_break(), 1,
        "the persisted 0–23 map still has one key for displayed hour 02");
    assert_eq!(tracker.current_away_secs, 360,
        "the second local 02:00 interval starts its own Away threshold");
}

#[test]
fn half_hour_zone_hour_boundary_uses_local_interval_not_utc_hour() {
    use chrono::{DateTime, TimeZone, Utc};

    let start = Utc.with_ymd_and_hms(2026, 6, 1, 12, 25, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 6, 1, 12, 35, 0).unwrap();
    let local_hour = |at: DateTime<Utc>| {
        crate::hourly_break_tracker::LocalHourInterval::from_offset(at, 5 * 60 * 60 + 30 * 60)
    };
    let mut tracker = crate::hourly_break_tracker::HourlyBreakTracker::new();

    tracker.record_known_interval(start, end, true, local_hour);

    assert_eq!(tracker.hours_active(), 2,
        "the +05:30 local-hour boundary occurs at 12:30 UTC");
    assert_eq!(tracker.hours_with_break(), 2,
        "each five-minute local-hour intersection is independently covered");
    assert_eq!(tracker.current_away_secs, 300);
}

#[test]
fn reading_path_accumulates_subsecond_known_away_intervals() {
    use chrono::{Duration, TimeZone, Utc};

    let start = Utc.with_ymd_and_hms(2026, 6, 1, 6, 0, 0).unwrap();
    let mut m = crate::session_manager::SessionManager::new();
    let local_hour = |at: chrono::DateTime<Utc>| {
        crate::hourly_break_tracker::LocalHourInterval::from_offset(at, 9 * 60 * 60)
    };

    for second in -5..=0 {
        m.on_reading_at_with_local_hour(800, false,
            start + Duration::seconds(second), local_hour);
    }
    for half_seconds in 1..=600 {
        let at = start + Duration::milliseconds(half_seconds * 500);
        m.on_reading_at_with_local_hour(800, false, at, local_hour);
        if half_seconds % 17 == 0 {
            m.on_reading_at_with_local_hour(800, false, at, local_hour);
        }
    }

    assert_eq!(m.hourly_break_tracker.hours_with_break(), 1,
        "600 known half-second intervals total 300 seconds despite duplicate observations");
    assert_eq!(m.hourly_break_tracker.current_away_secs, 300);
    assert_eq!(m.hourly_break_tracker.hours_active(), 1,
        "the explicit +09:00 mapping keeps the full interval in local hour 15");
}

#[test]
fn reading_path_splits_hour_boundary_and_does_not_credit_unknown_gap() {
    use chrono::{Duration, TimeZone, Utc};

    let start = Utc.with_ymd_and_hms(2026, 6, 1, 6, 57, 0).unwrap();
    let mut m = crate::session_manager::SessionManager::new();
    let local_hour = |at: chrono::DateTime<Utc>| {
        crate::hourly_break_tracker::LocalHourInterval::from_offset(at, 9 * 60 * 60)
    };

    for second in -5..=0 {
        m.on_reading_at_with_local_hour(800, false,
            start + Duration::seconds(second), local_hour);
    }
    for offset in [30, 180, 180, 300] {
        m.on_reading_at_with_local_hour(
            800, false, start + Duration::seconds(offset), local_hour,
        );
    }
    assert_eq!(m.hourly_break_tracker.hours_with_break(), 0,
        "the known interval contributes 180s to hour 15 and 120s to hour 16");
    assert_eq!(m.hourly_break_tracker.hours_active(), 2);

    let resumed = start + Duration::seconds(601);
    m.on_reading_at_with_local_hour(800, false, resumed, local_hour);
    assert_eq!(m.hourly_break_tracker.current_away_secs, 0,
        "the 301-second unexplained gap breaks continuity without credit");
    assert_eq!(m.hourly_break_tracker.hours_with_break(), 0);

    for second in 1..=5 {
        m.on_reading_at_with_local_hour(800, false,
            resumed + Duration::seconds(second), local_hour);
    }
    m.on_reading_at_with_local_hour(800, false, resumed + Duration::seconds(299), local_hour);
    assert_eq!(m.hourly_break_tracker.hours_with_break(), 0,
        "299 known post-gap seconds remain below the threshold");
    m.on_reading_at_with_local_hour(800, false, resumed + Duration::seconds(300), local_hour);
    assert_eq!(m.hourly_break_tracker.hours_with_break(), 1,
        "coverage follows 300 known seconds in the injected hour 16");
}

#[test]
fn reading_path_rejects_fractional_gap_over_the_known_interval_limit() {
    use chrono::{Duration, TimeZone, Utc};

    let start = Utc.with_ymd_and_hms(2026, 6, 1, 6, 0, 0).unwrap();
    let mut m = crate::session_manager::SessionManager::new();
    let local_hour = |at: chrono::DateTime<Utc>| {
        crate::hourly_break_tracker::LocalHourInterval::from_offset(at, 9 * 60 * 60)
    };

    for second in -5..=0 {
        m.on_reading_at_with_local_hour(800, false,
            start + Duration::seconds(second), local_hour);
    }
    let resumed = start + Duration::milliseconds(300_500);
    m.on_reading_at_with_local_hour(800, false, resumed, local_hour);
    assert_eq!(m.hourly_break_tracker.current_away_secs, 0,
        "a 300.5-second observation gap breaks continuity without credit");
    assert_eq!(m.hourly_break_tracker.hours_with_break(), 0);

    for millis in (500..=2_500).step_by(500) {
        m.on_reading_at_with_local_hour(800, false,
            resumed + Duration::milliseconds(millis), local_hour);
    }
    m.on_reading_at_with_local_hour(800, false,
        resumed + Duration::milliseconds(299_500), local_hour);
    assert_eq!(m.hourly_break_tracker.hours_with_break(), 0,
        "299.5 known post-gap seconds remain below threshold");
    m.on_reading_at_with_local_hour(800, false,
        resumed + Duration::seconds(300), local_hour);
    assert_eq!(m.hourly_break_tracker.hours_with_break(), 1,
        "the next 0.5 known seconds complete exactly 300 seconds of coverage");
}

#[test]
fn reading_path_does_not_credit_unconfirmed_cold_start_candidates() {
    use chrono::{Duration, TimeZone, Utc};

    let start = Utc.with_ymd_and_hms(2026, 6, 1, 6, 0, 0).unwrap();
    let mut m = crate::session_manager::SessionManager::new();
    let read = |m: &mut crate::session_manager::SessionManager, offset: i64, mm: i32| {
        m.on_reading_at_with_local_hour(mm, true, start + Duration::seconds(offset),
            |at| crate::hourly_break_tracker::LocalHourInterval::from_offset(at, 0));
    };

    read(&mut m, 0, 800);
    read(&mut m, 1, 800);
    read(&mut m, 2, 800);
    read(&mut m, 3, 1200); // A changed candidate restarts the five-second run.
    read(&mut m, 4, 800);
    for second in 5..=8 { read(&mut m, second, 800); }

    assert_eq!(m.current_state(), crate::session_types::DeskState::Away,
        "the final Sitting candidate is not confirmed before five elapsed seconds");
    assert_eq!(m.history_span_started, None,
        "unconfirmed cold-start time remains outside observed history");
    assert_eq!(m.state.away_bout_secs, 0,
        "the default Away state cannot claim an unknown cold-start interval");
    assert_eq!(m.state.continuous_computer_secs, 0);
    assert_eq!(m.state.position_changes, 0);
    assert_eq!(m.hourly_break_tracker.current_away_secs, 0,
        "hourly coverage cannot infer Away from the unconfirmed default state");
    assert_eq!(m.hourly_break_tracker.hours_active(), 0);

    read(&mut m, 9, 800);
    assert_eq!(m.current_state(), crate::session_types::DeskState::Sitting);
    assert_eq!(m.history_span_started, Some(start + Duration::seconds(9)),
        "cold-start history begins when the candidate is confirmed");
    assert_eq!(m.state.away_bout_secs, 0);
    assert_eq!(m.state.position_changes, 0,
        "confirmation from default Away is not a posture change");

    read(&mut m, 10, 800);
    assert_eq!(m.state.continuous_computer_secs, 1,
        "computer time begins with the first known interval after confirmation");
    assert_eq!(m.state.sitting_seconds_total, 0,
        "unconfirmed candidate time is not added to raw Sitting history");
    assert_eq!(m.hourly_break_tracker.current_away_secs, 0);
    assert_eq!(m.hourly_break_tracker.hours_active(), 1);
}

#[test]
fn reading_path_accumulates_computer_counters_by_known_elapsed_time() {
    use chrono::{Duration, TimeZone, Utc};
    use crate::session_types::DeskState;

    let start = Utc.with_ymd_and_hms(2026, 6, 1, 9, 0, 0).unwrap();
    let known_sitting = || {
        let mut m = crate::session_manager::SessionManager::new();
        m.state.state = DeskState::Sitting;
        m.state.sitting_started = Some(start);
        m.history_span_started = Some(start);
        m.history_last_reading = Some(start);
        m.state.last_tick_ts = Some(start);
        m.state.last_accumulate_ts = Some(start);
        m
    };
    let mut ten_second = known_sitting();
    for second in (10..=60).step_by(10) {
        ten_second.on_reading_at_with_local_hour(800, true,
            start + Duration::seconds(second), |at| {
                crate::hourly_break_tracker::LocalHourInterval::from_offset(at, 0)
            });
    }
    assert_eq!(ten_second.state.continuous_computer_secs, 60,
        "six known ten-second intervals represent one minute, not six samples");
    assert_eq!(ten_second.state.longest_computer_session_secs, 60);
    assert_eq!(ten_second.state.away_bout_secs, 0);

    let mut jittered = known_sitting();
    let mut elapsed_millis = 0;
    for delta_millis in [400, 700, 900].into_iter().cycle().take(90) {
        elapsed_millis += delta_millis;
        jittered.on_reading_at_with_local_hour(800, true,
            start + Duration::milliseconds(elapsed_millis), |at| {
                crate::hourly_break_tracker::LocalHourInterval::from_offset(at, 0)
            });
    }
    assert_eq!(elapsed_millis, 60_000);
    assert_eq!(jittered.state.continuous_computer_secs, 60,
        "jittered known intervals totaling one minute retain all whole seconds");
    assert_eq!(jittered.state.longest_computer_session_secs, 60);
    assert_eq!(jittered.state.away_bout_secs, 0);
}

#[test]
fn reading_path_excludes_unknown_time_duplicates_and_rewinds_from_away_counters() {
    use chrono::{Duration, TimeZone, Utc};
    use crate::session_types::DeskState;

    let start = Utc.with_ymd_and_hms(2026, 6, 1, 9, 0, 0).unwrap();
    let mut m = crate::session_manager::SessionManager::new();
    m.state.state = DeskState::Away;
    m.state.continuous_computer_secs = 120;
    m.state.position_changes = 7;
    m.history_span_started = Some(start);
    m.history_last_reading = Some(start);
    m.state.last_tick_ts = Some(start);
    m.state.last_accumulate_ts = Some(start);

    for second in (10..=60).step_by(10) {
        m.on_reading_at_with_local_hour(800, false,
            start + Duration::seconds(second), |at| {
                crate::hourly_break_tracker::LocalHourInterval::from_offset(at, 0)
            });
    }
    assert_eq!(m.state.away_bout_secs, 60);
    assert_eq!(m.state.continuous_computer_secs, 120);

    m.on_reading_at_with_local_hour(800, false, start + Duration::seconds(60),
        |at| crate::hourly_break_tracker::LocalHourInterval::from_offset(at, 0));
    m.on_reading_at_with_local_hour(800, false, start + Duration::seconds(59),
        |at| crate::hourly_break_tracker::LocalHourInterval::from_offset(at, 0));
    assert_eq!(m.state.away_bout_secs, 60,
        "duplicate and backwards instants add no elapsed time");

    m.on_reading_at_with_local_hour(800, false, start + Duration::seconds(361),
        |at| crate::hourly_break_tracker::LocalHourInterval::from_offset(at, 0));
    assert_eq!(m.state.away_bout_secs, 0,
        "the unknown outage breaks reset continuity without erasing prior break history");
    assert_eq!(m.state.continuous_computer_secs, 120);
    assert_eq!(m.state.position_changes, 7);

    for second in 362..=366 {
        m.on_reading_at_with_local_hour(800, false, start + Duration::seconds(second),
            |at| crate::hourly_break_tracker::LocalHourInterval::from_offset(at, 0));
    }
    assert_eq!(m.history_span_started, Some(start + Duration::seconds(361)),
        "confirmed resumed Away history begins at its first sample");
    assert_eq!(m.state.away_bout_secs, 5,
        "only the five seconds in the confirmed resumed candidate follow the reset");
    assert_eq!(m.state.continuous_computer_secs, 120);
    assert_eq!(m.state.position_changes, 7);
}

#[test]
fn unknown_gap_breaks_away_reset_continuity_but_preserves_known_break_time() {
    use chrono::{Duration, TimeZone, Utc};
    use crate::session_types::DeskState;

    let start = Utc.with_ymd_and_hms(2026, 6, 1, 9, 0, 0).unwrap();
    let local_hour = |at: chrono::DateTime<Utc>| {
        crate::hourly_break_tracker::LocalHourInterval::from_offset(at, 0)
    };
    let mut m = crate::session_manager::SessionManager::new();

    for second in 0..=5 {
        m.on_reading_at_with_local_hour(
            800, true, start + Duration::seconds(second), local_hour,
        );
    }
    assert_eq!(m.state.state, DeskState::Sitting);
    m.state.continuous_computer_secs = 50;

    for second in 6..=11 {
        m.on_reading_at_with_local_hour(
            800, false, start + Duration::seconds(second), local_hour,
        );
    }
    assert_eq!(m.state.state, DeskState::Away);
    for second in 12..=61 {
        m.on_reading_at_with_local_hour(
            800, false, start + Duration::seconds(second), local_hour,
        );
    }
    m.state.continuous_computer_secs = 50;
    assert_eq!(m.state.away_bout_secs, 50);

    m.limits.computer_break_reset_secs = 60;
    let resumed = start + Duration::seconds(362); // 301 seconds after the last sample.
    m.on_reading_at_with_local_hour(800, false, resumed, local_hour);
    for second in 1..=10 {
        m.on_reading_at_with_local_hour(
            800, false, resumed + Duration::seconds(second), local_hour,
        );
    }

    assert_eq!(m.state.continuous_computer_secs, 50,
        "ten confirmed Away seconds after Unknown cannot combine with the pre-gap fifty");
    assert_eq!(m.state.away_bout_secs, 10,
        "the continuous Away reset clock restarts after the unknown outage");
    assert_eq!(m.observed_break_secs, 50,
        "the known pre-gap break remains separately credited");
    assert_eq!(m.state.break_seconds, 60,
        "break history retains 50 known pre-gap plus 10 known resumed seconds");
}

#[test]
fn reading_path_resets_computer_time_once_after_elapsed_away_threshold() {
    use chrono::{Duration, TimeZone, Utc};
    use crate::session_types::DeskState;

    let start = Utc.with_ymd_and_hms(2026, 6, 1, 9, 0, 0).unwrap();
    let mut m = crate::session_manager::SessionManager::new();
    m.state.state = DeskState::Away;
    m.state.continuous_computer_secs = 120;
    m.state.position_changes = 7;
    m.limits.computer_break_reset_secs = 60;
    m.history_span_started = Some(start);
    m.history_last_reading = Some(start);
    m.state.last_tick_ts = Some(start);
    m.state.last_accumulate_ts = Some(start);

    for second in (10..=70).step_by(10) {
        m.on_reading_at_with_local_hour(800, false,
            start + Duration::seconds(second), |at| {
                crate::hourly_break_tracker::LocalHourInterval::from_offset(at, 0)
            });
    }
    assert_eq!(m.state.away_bout_secs, 70);
    assert_eq!(m.state.continuous_computer_secs, 0,
        "one elapsed 60-second Away threshold resets the computer counter");
    assert_eq!(m.state.position_changes, 8,
        "the reset counts as one position change and cannot repeat");
}
