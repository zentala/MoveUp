//! session_tests_sleep.rs — Sleep/suspend gap detection and break credit tests.
//!
//! Readings are injected at explicit instants (E020-T01), so the gap the engine
//! sees is exactly the gap the test intends. The old form backdated
//! `last_tick_ts` against one `Utc::now()` and then let `on_reading` read a
//! second, later one, which is why some assertions here were inequalities.

#[test]
fn e027_sleep_credit_with_positive_load_is_applied_once_for_whole_break() {
    use crate::session_types::DeskState;
    let start = chrono::DateTime::parse_from_rfc3339("2026-10-09T10:00:00Z")
        .unwrap().with_timezone(&chrono::Utc);
    for (gaps, end, observed_break_secs) in [
        (vec![(200, true), (800, false)], 850, 150),
        (vec![(200, true), (550, true), (800, false)], 850, 400),
        // The duplicate timestamp adds no known break time before the later outage.
        (vec![(200, true), (200, true), (900, false)], 950, 150),
    ] {
        let mut m = crate::session_manager::SessionManager::new();
        m.state.sitting_seconds = 10000;
        let mut events = vec![(0, 800, true), (100, 1200, true)];
        events.extend(gaps.iter().map(|(t, active)| (*t, 1200, *active)));
        events.push((end, 800, true));
        for (t, mm, active) in events {
            let candidate_start = start + chrono::Duration::seconds(
                t - crate::session_types::DEBOUNCE_COUNT as i64);
            let first_observation = m.state.last_tick_ts
                .filter(|last| (candidate_start - *last).num_seconds()
                    > crate::session_types::SLEEP_GAP_THRESHOLD_SECS)
                .map(|_| start + chrono::Duration::seconds(t))
                .unwrap_or(candidate_start);
            for offset in 0..=crate::session_types::DEBOUNCE_COUNT {
                m.on_reading_at(mm, active,
                    first_observation + chrono::Duration::seconds(offset as i64));
            }
        }
        assert_eq!(m.current_state(), DeskState::Sitting);
        assert_eq!(m.state.last_break_secs, observed_break_secs,
            "gaps={gaps:?} end={end} observed break mismatch");
        assert_eq!(m.state.sitting_seconds, 10100 - observed_break_secs * 2);
        assert_eq!(m.state.sitting_seconds_total, 100);
        assert_eq!(m.state.last_break_credit, crate::session_types::BreakCredit::Partial);
    }
}

#[test]
fn e027_gap_closes_forty_minutes_of_observed_sitting_without_crediting_gap() {
    use chrono::{Duration, TimeZone, Utc};
    use crate::session_manager::SessionManager;
    use crate::session_types::{BreakCredit, DEBOUNCE_COUNT};

    let start = Utc.with_ymd_and_hms(2026, 10, 9, 10, 0, 0).unwrap();
    let mut m = SessionManager::new();
    for offset in 0..=DEBOUNCE_COUNT {
        m.on_reading_at(800, true, start - Duration::seconds(DEBOUNCE_COUNT as i64) + Duration::seconds(offset as i64));
    }
    assert_eq!(m.current_state(), crate::session_types::DeskState::Sitting);
    for minute in 1..=40 {
        m.on_reading_at(800, true, start + Duration::minutes(minute));
    }
    let last_observed = start + Duration::minutes(40);
    m.on_reading_at(800, true, last_observed + Duration::minutes(10));

    assert_eq!(m.state.sitting_seconds_total, 40 * 60,
        "the observed prefix must be committed before the unknown gap");
    assert_eq!(m.state.sitting_seconds, 40 * 60,
        "an unexplained gap must not earn sitting credit");
    assert_eq!(m.state.last_break_credit, BreakCredit::None);
    assert_eq!(m.pending_history.len(), 1);
    assert_eq!(m.pending_history[0].span.duration_secs, 40 * 60);
}

#[test]
fn e027_positive_sitting_load_standing_gap_return_credits_observed_break_once() {
    use chrono::{Duration, TimeZone, Utc};
    use crate::session_manager::SessionManager;
    use crate::session_types::{BreakCredit, DEBOUNCE_COUNT};

    let start = Utc.with_ymd_and_hms(2026, 10, 9, 10, 0, 0).unwrap();
    let mut m = SessionManager::new();
    m.state.sitting_seconds = 10_000;
    for offset in 0..=DEBOUNCE_COUNT {
        m.on_reading_at(800, true, start - Duration::seconds(DEBOUNCE_COUNT as i64) + Duration::seconds(offset as i64));
    }
    for i in 0..=DEBOUNCE_COUNT {
        m.on_reading_at(1200, true, start + Duration::seconds(100 + i as i64));
    }
    assert_eq!(m.current_state(), crate::session_types::DeskState::Standing);

    for i in 1..=60 {
        m.on_reading_at(1200, true, start + Duration::seconds(100 + DEBOUNCE_COUNT as i64 + i as i64));
    }
    let resumed = start + Duration::seconds(100 + DEBOUNCE_COUNT as i64 + 60 + 600);
    // The first post-gap sample is the actual resume instant. Start the new
    // candidate there and let its fifth elapsed second confirm at resumed + 5.
    m.on_reading_at(1200, true, resumed);
    for offset in 0..=DEBOUNCE_COUNT {
        m.on_reading_at(800, true, resumed + Duration::seconds(offset as i64));
    }

    assert_eq!(m.current_state(), crate::session_types::DeskState::Sitting);
    assert_eq!(m.state.sitting_seconds_total, 105,
        "the confirmed pre-standing Sitting prefix is retained as raw time");
    assert_eq!(m.state.last_break_secs, 65,
        "sixty observed Standing seconds plus five post-resume seconds qualify; the unknown gap is excluded");
    assert_eq!(m.state.sitting_seconds, 10_105 - 130,
        "the sixty-five observed break seconds earn exactly one proportional credit");
    assert_eq!(m.state.last_break_credit, BreakCredit::Partial);
}
#[cfg(test)]
mod tests {
    use chrono::{DateTime, Duration, TimeZone, Utc};

    use crate::session_manager::SessionManager;
    use crate::session_types::*;

    /// A fixed instant, so nothing here depends on when the suite runs.
    fn base() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 1, 9, 0, 0).unwrap()
    }

    /// Feeds readings across `count` elapsed seconds from `at`; returns the
    /// instant after the endpoint sample.
    fn advance_ticks(
        m: &mut SessionManager,
        mm: i32,
        active: bool,
        at: DateTime<Utc>,
        count: i64,
    ) -> DateTime<Utc> {
        for i in 0..=count {
            let _ = m.on_reading_at(mm, active, at + Duration::seconds(i));
        }
        at + Duration::seconds(count + 1)
    }

    /// Transitions manager to Sitting state via debounced readings.
    fn transition_to_sitting(m: &mut SessionManager, at: DateTime<Utc>) -> DateTime<Utc> {
        let next = advance_ticks(m, 800, true, at, DEBOUNCE_COUNT as i64);
        assert_eq!(m.state.state, DeskState::Sitting);
        next
    }

    /// Transitions manager to Standing state via debounced readings.
    fn transition_to_standing(m: &mut SessionManager, at: DateTime<Utc>) -> DateTime<Utc> {
        let next = advance_ticks(m, 1200, true, at, DEBOUNCE_COUNT as i64);
        assert_eq!(m.state.state, DeskState::Standing);
        next
    }

    // ── Inflation-prevention (original tests) ──────────────────────────────

    /// Regression: computer sleep inflates sitting_seconds.
    #[test]
    fn sleep_does_not_inflate_sitting_seconds() {
        let mut m = SessionManager::new();
        let now = base();

        transition_to_sitting(&mut m, now - Duration::hours(9));

        m.state.sitting_started = Some(now - Duration::hours(8));
        m.state.sitting_seconds = 600;
        m.state.sitting_seconds_total = 600;
        m.state.last_tick_ts = Some(now - Duration::hours(8));

        advance_ticks(&mut m, 800, false, now, DEBOUNCE_COUNT as i64);
        assert_eq!(m.state.state, DeskState::Away);

        assert!(
            m.state.sitting_seconds < 700,
            "sitting_seconds should be capped, got {} (sleep inflation!)",
            m.state.sitting_seconds
        );
    }

    /// Same test but for standing: sleep shouldn't inflate standing_seconds.
    #[test]
    fn sleep_does_not_inflate_standing_seconds() {
        let mut m = SessionManager::new();
        let now = base();

        let t = transition_to_sitting(&mut m, now - Duration::hours(9));
        m.state.sitting_started = Some(now);
        transition_to_standing(&mut m, t);
        assert_eq!(m.state.state, DeskState::Standing);

        m.state.standing_bout_started = Some(now - Duration::hours(8));
        m.state.standing_seconds = 300;
        m.state.last_tick_ts = Some(now - Duration::hours(8));

        advance_ticks(&mut m, 1200, false, now, DEBOUNCE_COUNT as i64);
        assert_eq!(m.state.state, DeskState::Away);

        assert!(
            m.state.standing_seconds < 400,
            "standing_seconds should be capped, got {} (sleep inflation!)",
            m.state.standing_seconds
        );
    }

    // ── Unknown sleep gaps preserve only the previously observed prefix ────

    /// Unknown time cannot earn break credit; the known sitting prefix remains.
    #[test]
    fn sleep_gap_preserves_only_observed_sitting_prefix() {
        let mut m = SessionManager::new();
        let now = base();
        transition_to_sitting(&mut m, now - Duration::hours(3));

        m.state.sitting_seconds = 2400; // 40 min sitting
        m.state.sitting_seconds_total = 2400;
        let last_observed = now - Duration::hours(2);
        m.state.last_tick_ts = Some(last_observed);
        let known_prefix = (last_observed - m.state.sitting_started.unwrap()).num_seconds();

        let _ = m.on_reading_at(800, true, now);

        assert_eq!(
            m.state.sitting_seconds, 2400 + known_prefix,
            "the known Sitting prefix is preserved and the unknown gap excluded (got {})",
            m.state.sitting_seconds
        );
        assert_eq!(
            m.state.last_break_credit,
            BreakCredit::None,
            "unknown time must not earn break credit"
        );
    }

    /// A 10-minute sleep gap earns no credit because its duration and posture
    /// were not observed.
    #[test]
    fn sleep_gap_does_not_credit_unknown_sitting_time() {
        let mut m = SessionManager::new();
        let now = base();
        transition_to_sitting(&mut m, now - Duration::hours(1));

        m.state.sitting_seconds = 2400; // 40 min sitting
        m.state.sitting_seconds_total = 2400;
        let last_observed = now - Duration::minutes(10);
        m.state.last_tick_ts = Some(last_observed);
        let known_prefix = (last_observed - m.state.sitting_started.unwrap()).num_seconds();

        let _ = m.on_reading_at(800, true, now);

        assert_eq!(
            m.state.sitting_seconds, 2400 + known_prefix,
            "only the observed prefix is counted (got {})",
            m.state.sitting_seconds
        );
        assert_eq!(
            m.state.last_break_credit,
            BreakCredit::None,
            "unknown time must not earn break credit"
        );
    }

    /// Changing the credit multiplier cannot make an unknown gap creditable.
    #[test]
    fn sleep_gap_multiplier_does_not_credit_unknown_time() {
        let mut m = SessionManager::new();
        let now = base();
        transition_to_sitting(&mut m, now - Duration::hours(1));

        m.state.sitting_seconds = 2400; // 40 min sitting
        m.state.sitting_seconds_total = 2400;
        m.limits.break_credit_multiplier = 1.0; // slower reset
        let last_observed = now - Duration::minutes(10);
        m.state.last_tick_ts = Some(last_observed);
        let known_prefix = (last_observed - m.state.sitting_started.unwrap()).num_seconds();

        let _ = m.on_reading_at(800, true, now);

        assert_eq!(
            m.state.sitting_seconds, 2400 + known_prefix,
            "the known Sitting prefix is preserved and the unknown gap excluded (got {})",
            m.state.sitting_seconds
        );
        assert_eq!(m.state.last_break_credit, BreakCredit::None);
    }

    /// A Standing gap resumes at the actual reading without crediting the gap.
    #[test]
    fn sleep_gap_while_standing_rewinds_without_break_credit() {
        let mut m = SessionManager::new();
        let now = base();
        let t = transition_to_sitting(&mut m, now - Duration::hours(1));
        m.state.sitting_seconds = 0; // just transitioned from credit
        transition_to_standing(&mut m, t);

        // Keep the posture anchor within the known prefix before the last sample.
        m.state.standing_bout_started = Some(now - Duration::minutes(40));
        let sitting_before_gap = m.state.sitting_seconds;
        m.state.last_tick_ts = Some(now - Duration::minutes(30));

        for second in 0..=DEBOUNCE_COUNT {
            let _ = m.on_reading_at(1200, true, now + Duration::seconds(second as i64));
        }

        assert_eq!(m.state.sitting_seconds, sitting_before_gap,
            "an unknown Standing gap cannot alter sitting credit");
        assert_eq!(m.state.last_break_credit, BreakCredit::None);
        assert_eq!(
            m.state.standing_bout_started,
            Some(now),
            "the Standing anchor resumes at the actual reading"
        );
    }

    /// Raw sitting includes the known prefix, but not unknown time or credit.
    #[test]
    fn sleep_gap_preserves_raw_sitting_prefix_without_credit() {
        let mut m = SessionManager::new();
        let now = base();
        transition_to_sitting(&mut m, now - Duration::hours(3));

        m.state.sitting_seconds = 2400;
        m.state.sitting_seconds_total = 2400;
        let last_observed = now - Duration::hours(2);
        m.state.last_tick_ts = Some(last_observed);
        let known_prefix = (last_observed - m.state.sitting_started.unwrap()).num_seconds();

        let _ = m.on_reading_at(800, true, now);

        assert_eq!(
            m.state.sitting_seconds, 2400 + known_prefix,
            "credited sitting preserves only its known prefix (got {})",
            m.state.sitting_seconds
        );
        assert_eq!(
            m.state.sitting_seconds_total, 2400 + known_prefix,
            "raw sitting includes only the known prefix (got {})",
            m.state.sitting_seconds_total
        );
        assert_eq!(m.state.last_break_credit, BreakCredit::None);
    }
}
