//! session_tests_timers_live.rs — T042: Live time counting and snapshot tests.
//!
//! State-transition fixtures feed coherent instants through `on_reading_at`;
//! ordinary duration snapshots use the system clock, while quality-boundary
//! snapshots use the explicit instant under test.

#[cfg(test)]
mod timer_live_tests {
    use chrono::{TimeZone, Utc};

    use crate::session_manager::SessionManager;
    use crate::session_types::*;

    fn feed(m: &mut SessionManager, mm: i32) -> crate::session::ReadingResult {
        let start = m.state.last_tick_ts.unwrap_or_else(|| Utc::now() - chrono::Duration::seconds(5));
        let mut result = crate::session::ReadingResult::default();
        for second in 0..=DEBOUNCE_COUNT {
            result = m.on_reading_at(mm, true, start + chrono::Duration::seconds(second as i64));
        }
        result
    }

    fn transition_to_sitting(m: &mut SessionManager) {
        feed(m, 800);
        assert_eq!(m.state.state, DeskState::Sitting);
    }

    fn feed_candidate(m: &mut SessionManager, mm: i32, confirmed_at: chrono::DateTime<Utc>) {
        let first = confirmed_at - chrono::Duration::seconds(DEBOUNCE_COUNT as i64);
        for offset in 0..=DEBOUNCE_COUNT {
            m.on_reading_at(mm, true, first + chrono::Duration::seconds(offset as i64));
        }
    }

    fn feed_regular(
        m: &mut SessionManager,
        mm: i32,
        from: chrono::DateTime<Utc>,
        to: chrono::DateTime<Utc>,
    ) {
        let mut at = from + chrono::Duration::seconds(60);
        while at < to {
            m.on_reading_at(mm, true, at);
            at += chrono::Duration::seconds(60);
        }
        if m.state.last_tick_ts != Some(to) {
            m.on_reading_at(mm, true, to);
        }
    }

    fn transition_and_observe(
        m: &mut SessionManager,
        mm: i32,
        duration_secs: i64,
        observed_until: chrono::DateTime<Utc>,
    ) {
        let confirmed_at = observed_until - chrono::Duration::seconds(duration_secs);
        feed_candidate(m, mm, confirmed_at);
        feed_regular(m, mm, confirmed_at, observed_until);
    }

    #[test]
    fn live_projections_stop_at_the_last_observation_coverage_boundary() {
        let t0 = Utc.timestamp_opt(1_800_000_000, 0).unwrap();
        let mut m = SessionManager::new();
        m.state.state = DeskState::Sitting;
        m.state.sitting_seconds = 60;
        m.state.sitting_seconds_total = 100;
        m.state.sitting_started = Some(t0 - chrono::Duration::seconds(100));
        m.state.last_tick_ts = Some(t0);
        m.history_last_reading = Some(t0);
        m.history_span_started = Some(t0 - chrono::Duration::seconds(100));

        // The freshness window is not observed duration: every projection
        // stops at t0 even while the latest sample is still considered fresh.
        let t0_plus_300 = t0 + chrono::Duration::seconds(300);
        assert_eq!(m.get_live_sitting_seconds(t0), 160);
        assert_eq!(m.get_live_sitting_seconds_total(t0), 200);
        assert_eq!(m.get_live_sitting_seconds(t0_plus_300), 160);
        assert_eq!(m.get_live_sitting_seconds_total(t0_plus_300), 200);
        assert!(m.observation_is_fresh_at(t0_plus_300));

        let stale = t0 + chrono::Duration::seconds(301);
        assert_eq!(m.get_live_sitting_seconds(stale), 160);
        assert_eq!(m.get_live_sitting_seconds_total(stale), 200);
        assert!(!m.observation_is_fresh_at(stale));
        assert!(m.policy_input(stale, true).sensor_connected);
        assert!(!m.policy_input(stale, true).observation_fresh);
        assert_eq!(m.snapshot_at(stale).observation_quality, None);
    }

    #[test]
    fn stale_connected_observation_preserves_facts_and_waits_for_coherent_resume() {
        let t0 = Utc.timestamp_opt(1_800_000_000, 0).unwrap();
        let mut m = SessionManager::new();
        m.state.state = DeskState::Sitting;
        m.state.sitting_seconds = 3_000;
        m.state.sitting_seconds_total = 3_000;
        m.state.sitting_started = Some(t0);
        m.state.last_position_change_at = Some(t0);
        m.state.last_tick_ts = Some(t0);
        m.history_last_reading = Some(t0);
        m.history_span_started = Some(t0);
        let stale = t0 + chrono::Duration::seconds(301);

        assert!(!m.observation_is_fresh_at(stale));
        let facts = m.notification_facts(stale);
        assert_eq!(facts.sitting_seconds_total, 3_000);
        assert_eq!(facts.standing_seconds, 0);
        assert_eq!(facts.last_position_change_at, Some(t0));
        assert!(m.policy_input(stale, true).sensor_connected);

        // A real sample refreshes observation freshness without changing the
        // physical-connected input or inventing an Away transition.
        let resumed = stale + chrono::Duration::seconds(1);
        let sitting_before_resume = m.state.sitting_seconds;
        m.on_reading_at(800, true, resumed);
        assert!(!m.observation_is_fresh_at(resumed));
        assert_eq!(m.state.state, DeskState::Sitting);
        assert_eq!(m.history_span_started, None);
        assert_eq!(m.state.sitting_seconds, sitting_before_resume);
        assert_eq!(m.get_live_sitting_seconds(resumed), sitting_before_resume);
        assert_eq!(m.get_live_sitting_seconds_total(resumed), sitting_before_resume);
        feed(&mut m, 800);
        let confirmed_at = m.state.last_tick_ts.unwrap();
        assert!(m.observation_is_fresh_at(confirmed_at));
        assert!(m.history_span_started.is_some());
        assert_eq!(m.get_live_sitting_seconds(confirmed_at), sitting_before_resume + 5);
        assert_eq!(m.get_live_sitting_seconds_total(confirmed_at), sitting_before_resume + 5);
        assert!(m.policy_input(confirmed_at, true).observation_fresh);
    }

    #[test]
    fn standing_and_break_projections_freeze_after_the_observation_window() {
        let t0 = Utc.timestamp_opt(1_800_000_000, 0).unwrap();
        let mut m = SessionManager::new();
        m.state.state = DeskState::Standing;
        m.state.standing_seconds = 10;
        let start = t0 - chrono::Duration::seconds(100);
        m.state.standing_bout_started = Some(start);
        m.state.standing_session_started = Some(start);
        m.state.break_started = Some(start);
        m.state.break_seconds = 40;
        m.observed_break_secs = 40;
        m.state.last_tick_ts = Some(t0);
        m.history_last_reading = Some(t0);
        m.history_span_started = Some(start);

        let boundary = t0 + chrono::Duration::seconds(300);
        assert_eq!(m.get_live_standing_seconds(boundary), 110);
        assert_eq!(m.get_live_standing_session_secs(boundary), 100);
        assert_eq!(m.get_live_break_seconds(boundary), 140);

        let stale = boundary + chrono::Duration::seconds(1);
        assert_eq!(m.get_live_standing_seconds(stale), 110);
        assert_eq!(m.get_live_standing_session_secs(stale), 100);
        assert_eq!(m.get_live_break_seconds(stale), 140);
        assert!(!m.observation_is_fresh_at(stale));
    }

    #[test]
    fn cold_start_and_clock_rollback_have_unknown_policy_elapsed() {
        let t0 = Utc.timestamp_opt(1_800_000_000, 0).unwrap();
        let m = SessionManager::new();
        assert!(!m.observation_is_fresh_at(t0));
        assert_eq!(m.snapshot_at(t0).observation_quality, None);

        let mut m = SessionManager::new();
        m.state.state = DeskState::Sitting;
        m.state.sitting_started = Some(t0);
        m.state.last_tick_ts = Some(t0);
        m.history_last_reading = Some(t0);
        assert!(!m.observation_is_fresh_at(t0 - chrono::Duration::seconds(1)));
    }

    #[test]
    fn unknown_standing_projection_preserves_recorded_session_seconds() {
        let t0 = Utc.timestamp_opt(1_800_000_000, 0).unwrap();
        let mut m = SessionManager::new();
        m.state.state = DeskState::Standing;
        m.state.standing_session_secs = 300;
        m.state.standing_session_started = Some(t0 - chrono::Duration::seconds(300));

        assert_eq!(m.get_live_standing_session_secs(t0), 300);
        assert!(!m.observation_is_fresh_at(t0));
        assert_eq!(m.snapshot_at(t0).observation_quality, None);
    }

    // ─── Time Counting (tests 12–16) ─────────────────────────────────────

    #[test]
    fn e015_t042_12_sitting_5min_live_credited_secs() {
        let mut m = SessionManager::new();
        let now = Utc::now();
        transition_and_observe(&mut m, 800, 300, now);
        let live = m.get_live_sitting_seconds(now);
        assert!(live >= 299 && live <= 301, "expected ~300, got {}", live);
    }

    #[test]
    fn t042_13_standing_5min_live_break_secs() {
        let mut m = SessionManager::new();
        let now = Utc::now();
        transition_and_observe(&mut m, 1200, 300, now);
        let live = m.get_live_break_seconds(now);
        assert!(live >= 299 && live <= 301, "expected ~300, got {}", live);
    }

    #[test]
    fn e015_t042_14_snapshot_after_60s_sitting() {
        let mut m = SessionManager::new();
        transition_and_observe(&mut m, 800, 60, Utc::now());
        let snap = m.snapshot();
        assert!(snap.sitting_seconds >= 59 && snap.sitting_seconds <= 61);
        // The UI timer reads limit_used_secs — one field, same value.
        assert_eq!(snap.limit_used_secs, snap.sitting_seconds);
        assert_eq!(snap.break_seconds, 0);
    }

    #[test]
    fn e015_t042_15_snapshot_after_30s_standing() {
        let mut m = SessionManager::new();
        m.state.sitting_seconds = 900;
        transition_and_observe(&mut m, 1200, 30, Utc::now());
        let snap = m.snapshot();
        assert!(snap.break_seconds >= 29 && snap.break_seconds <= 31);
        // Standing freezes the credited counter; it does not zero it.
        assert_eq!(snap.limit_used_secs, 900);
    }

    #[test]
    fn t042_15b_snapshot_standing_seconds_includes_live() {
        let mut m = SessionManager::new();
        m.state.standing_seconds = 600; // 10 min from earlier
        transition_and_observe(&mut m, 1200, 120, Utc::now());
        let snap = m.snapshot();
        // standing_seconds should include base (600) + live standing bout (120)
        assert!(
            snap.standing_seconds >= 719 && snap.standing_seconds <= 721,
            "expected ~720, got {}",
            snap.standing_seconds
        );
    }

    #[test]
    fn t042_15c_snapshot_standing_seconds_frozen_while_sitting() {
        let mut m = SessionManager::new();
        m.state.standing_seconds = 600;
        transition_to_sitting(&mut m);
        m.state.sitting_started = Some(Utc::now() - chrono::Duration::seconds(60));
        let snap = m.snapshot();
        // While sitting, standing_seconds should NOT grow
        assert_eq!(snap.standing_seconds, 600);
    }

    #[test]
    fn t042_15d_state_changed_payload_uses_live_break_seconds() {
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);
        // Transition to standing — break_seconds should be 0 (just started)
        let result = feed(&mut m, 1200);
        if let Some(payload) = &result.state_change {
            // Break just started, so break_seconds should be ~0
            assert!(
                payload.break_seconds <= 1,
                "expected ~0, got {}",
                payload.break_seconds
            );
        }
    }

    #[test]
    fn t042_15e_standing_to_sitting_creates_completed_session() {
        let mut m = SessionManager::new();
        let now = Utc::now();
        transition_and_observe(&mut m, 1200, 295, now - chrono::Duration::seconds(5));
        feed_candidate(&mut m, 800, now);
        // The transition should have produced a completed session
        // We can't access the result directly from transition_to_sitting helper,
        // so verify via state: standing_seconds should have accumulated
        assert!(
            m.state.standing_seconds >= 299,
            "expected ~300, got {}",
            m.state.standing_seconds
        );
    }

    #[test]
    fn e015_t042_16_full_cycle_sit_stand_sit_timer_is_credited_never_reset() {
        let mut m = SessionManager::new();
        let now = Utc::now();

        // Phase 1: observe five minutes of sitting before the Standing transition.
        let sitting_started = now - chrono::Duration::seconds(420);
        feed_candidate(&mut m, 800, sitting_started);
        feed_regular(&mut m, 800, sitting_started, now - chrono::Duration::seconds(125));
        let standing_started = now - chrono::Duration::seconds(120);
        feed_candidate(&mut m, 1200, standing_started);
        let sitting_at_transition = m.get_live_sitting_seconds(standing_started);
        assert!(sitting_at_transition >= 299 && sitting_at_transition <= 301);
        assert_eq!(m.get_live_break_seconds(standing_started), 0);

        // Phase 2: observe two minutes of Standing (credit = 120s * 2.0).
        feed_regular(&mut m, 1200, standing_started, now - chrono::Duration::seconds(5));
        let standing_sample_at = now - chrono::Duration::seconds(5);
        let standing_break_at_sample = m.get_live_break_seconds(standing_sample_at);
        assert!(standing_break_at_sample >= 114 && standing_break_at_sample <= 116);
        // While standing the timer is frozen at what was consumed, not zeroed.
        assert!(
            m.get_live_sitting_seconds(standing_sample_at) >= 299
                && m.get_live_sitting_seconds(standing_sample_at) <= 301,
            "credited sitting should remain near 300 while standing"
        );
        // standing_seconds should include live break
        assert!(
            m.get_live_standing_seconds(standing_sample_at) >= 114
                && m.get_live_standing_seconds(standing_sample_at) <= 116,
            "standing seconds should stop at the last actual sample"
        );

        // Phase 3: Sit again — 120s break * 2.0 = 240s credit < ~300s sitting → Partial.
        // This is the assertion E015 inverted: the old suite demanded ~0 here,
        // which is the reset the user complained about. The engine credits, it
        // does not reset, so the timer must show 300 − 240 ≈ 60.
        feed_candidate(&mut m, 800, now);
        let s3 = m.snapshot();
        assert_eq!(m.state.last_break_credit, BreakCredit::Partial);
        assert_eq!(m.state.last_break_secs, 120);
        assert!(
            s3.limit_used_secs >= 55 && s3.limit_used_secs <= 70,
            "expected ~60 (300 - 120*2.0), got {}",
            s3.limit_used_secs
        );
    }

    #[test]
    fn standing_to_away_checkpoint_then_gap_counts_only_new_known_break_tail() {
        let started = Utc.timestamp_opt(1_800_000_000, 0).unwrap();
        let standing_through = started + chrono::Duration::seconds(55);
        let away_started = started + chrono::Duration::seconds(60);
        let away_through = away_started + chrono::Duration::seconds(30);
        let resumed = away_through + chrono::Duration::seconds(600);
        let mut m = SessionManager::new();
        m.state.sitting_seconds = 10_000;

        transition_and_observe(&mut m, 1200, 55, standing_through);
        for offset in 0..=DEBOUNCE_COUNT {
            m.on_reading_at(
                1200,
                false,
                standing_through + chrono::Duration::seconds(offset as i64),
            );
        }
        assert_eq!(m.state.state, DeskState::Away);
        assert_eq!(m.get_live_break_seconds(away_started), 60);

        for second in 1..=30 {
            m.on_reading_at(
                1200,
                false,
                away_started + chrono::Duration::seconds(second),
            );
        }
        assert_eq!(m.get_live_break_seconds(away_through), 90);

        for offset in 0..=DEBOUNCE_COUNT {
            m.on_reading_at(
                800,
                true,
                resumed + chrono::Duration::seconds(offset as i64),
            );
        }

        assert_eq!(m.state.state, DeskState::Sitting);
        assert_eq!(m.state.last_break_secs, 90,
            "the checkpointed Standing60 plus known Away30 count once; the gap is excluded");
        assert_eq!(m.state.last_break_credit, BreakCredit::Partial);
        assert_eq!(m.get_live_sitting_seconds_total(resumed + chrono::Duration::seconds(5)), 5,
            "only the five confirmed post-resume Sitting seconds join the raw total");
        assert_eq!(m.get_live_sitting_seconds(resumed + chrono::Duration::seconds(5)), 9_825,
            "the measured 90-second break earns one 180-second credit");
    }

    #[test]
    fn shutdown_history_queue_then_same_manager_resume_preserves_known_standing_prefixes() {
        let started = Utc.timestamp_opt(1_800_000_000, 0).unwrap();
        let shutdown_at = started + chrono::Duration::seconds(120);
        let resumed_until = shutdown_at + chrono::Duration::seconds(60);
        let mut m = SessionManager::new();

        transition_and_observe(&mut m, 1200, 120, shutdown_at);
        assert_eq!(m.state.state, DeskState::Standing);
        assert_eq!(m.get_live_standing_session_secs(shutdown_at), 120);
        assert_eq!(m.get_live_break_seconds(shutdown_at), 120);
        assert_eq!(m.get_live_standing_seconds(shutdown_at), 120);

        // Exercise the in-memory history queue and same-manager continuation.
        // This does not run or claim native lifecycle cancellation.
        m.queue_shutdown_history(shutdown_at);
        assert_eq!(m.history_span_started, Some(shutdown_at));
        assert_eq!(m.state.standing_seconds, 120);

        for second in 1..=60 {
            m.on_reading_at(
                1200,
                true,
                shutdown_at + chrono::Duration::seconds(second),
            );
        }

        assert_eq!(m.get_live_standing_session_secs(resumed_until), 180,
            "the live standing lap retains the flushed prefix plus the known resumed tail");
        assert_eq!(m.get_live_break_seconds(resumed_until), 180,
            "the break retains the flushed prefix plus the known resumed tail");
        assert_eq!(m.get_live_standing_seconds(resumed_until), 180,
            "raw Standing retains measured time across the queue boundary");
    }

    #[test]
    fn fresh_shutdown_flush_uses_last_reading_and_retry_does_not_extend_history() {
        let t0 = Utc.timestamp_opt(1_800_000_000, 0).unwrap();
        let started = t0 - chrono::Duration::seconds(60);
        let flush_at = t0 + chrono::Duration::seconds(120);
        let mut m = SessionManager::new();
        m.state.state = DeskState::Sitting;
        m.state.sitting_seconds = 600;
        m.state.sitting_seconds_total = 900;
        m.state.sitting_started = Some(started);
        m.state.last_tick_ts = Some(t0);
        m.history_last_reading = Some(t0);
        m.history_span_started = Some(started);

        let span = m.flush_current_session_at(flush_at).unwrap();
        assert_eq!(span.ended_at, t0.to_rfc3339(),
            "freshness is not evidence for a shutdown tail without another reading");
        assert_eq!(span.duration_secs, 60);

        m.queue_shutdown_history(flush_at);
        assert_eq!(m.pending_history.len(), 1);
        assert_eq!(m.pending_history[0].span.ended_at, t0.to_rfc3339());
        assert_eq!(m.pending_history[0].span.duration_secs, 60);
        assert_eq!(m.state.sitting_seconds, 660);
        assert_eq!(m.state.sitting_seconds_total, 960);
        assert_eq!(m.state.sitting_started, Some(t0));

        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::init_schema(&conn).unwrap();
        crate::serial_periodic_reset::persist_pending_history(&mut m, &conn, |_, _| {}).unwrap();
        m.queue_shutdown_history(flush_at + chrono::Duration::seconds(30));
        crate::serial_periodic_reset::persist_pending_history(&mut m, &conn, |_, _| {}).unwrap();

        let (count, duration, ended_at): (i64, i64, String) = conn.query_row(
            "SELECT COUNT(*), SUM(duration_seconds), MAX(ended_at) FROM sessions",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).unwrap();
        assert_eq!((count, duration, ended_at.as_str()), (1, 60, t0.to_rfc3339().as_str()),
            "repeated flush persistence must retain only the one known prefix");
        assert_eq!(m.state.sitting_seconds, 660);
        assert_eq!(m.state.sitting_seconds_total, 960);
    }

    #[test]
    fn repeated_counter_only_shutdown_queues_keep_legacy_segments() {
        let started = Utc.timestamp_opt(1_800_000_000, 0).unwrap();
        let first_shutdown = started + chrono::Duration::seconds(60);
        let second_shutdown = first_shutdown + chrono::Duration::seconds(30);
        let mut m = SessionManager::new();
        m.state.state = DeskState::Sitting;
        m.state.sitting_started = Some(started);

        m.queue_shutdown_history(first_shutdown);
        assert_eq!(m.pending_history.len(), 1);
        assert_eq!(m.pending_history[0].span.duration_secs, 60);
        assert_eq!(m.history_span_started, None,
            "a counter-only fixture must not be relabeled as observation-backed");

        m.queue_shutdown_history(second_shutdown);
        assert_eq!(m.pending_history.len(), 2);
        assert_eq!(m.pending_history[1].span.started_at, first_shutdown.to_rfc3339());
        assert_eq!(m.pending_history[1].span.ended_at, second_shutdown.to_rfc3339());
        assert_eq!(m.pending_history[1].span.duration_secs, 30);
        assert_eq!(m.state.sitting_seconds, 90);
        assert_eq!(m.state.sitting_seconds_total, 90);
    }
}
