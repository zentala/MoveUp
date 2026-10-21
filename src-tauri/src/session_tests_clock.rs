//! session_tests_clock.rs — Clock injection (E020-T01).
//!
//! Every test here drives the engine through its `_at` entry points, so no
//! assertion depends on when the suite runs or on the machine's time zone.
//! Two things are under test:
//!
//! 1. **Purity** — `on_reading_at` derives all of its time from the `now` it
//!    is handed, so an identical replay yields identical counters.
//! 2. **D3, the local calendar day** — the daily reset compares LOCAL days, not
//!    UTC ones. In Warsaw (UTC+1/+2) a UTC comparison rolls the day over at
//!    01:00 or 02:00 local, mid-workday, and again fails to roll over at local
//!    midnight. Both directions are asserted below.

#[cfg(test)]
mod clock_tests {
    use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
    use chrono_tz::America::Sao_Paulo;
    use chrono_tz::Europe::Warsaw;
    use chrono_tz::Tz;

    use crate::session_daily::local_date_of;
    use crate::session_manager::SessionManager;
    use crate::session_types::*;

    /// The UTC instant of a local wall-clock time in `tz`.
    ///
    /// `earliest()` picks the first of the two instants an ambiguous local time
    /// maps to on a fall-back day — which is the reading a clock on the wall
    /// shows first, and the one a replay would hit first.
    fn utc_at(tz: Tz, y: i32, m: u32, d: u32, hh: u32, mm: u32) -> DateTime<Utc> {
        tz.with_ymd_and_hms(y, m, d, hh, mm, 0)
            .earliest()
            .expect("local time exists in this zone")
            .with_timezone(&Utc)
    }

    fn local_day(tz: Tz, now: DateTime<Utc>) -> NaiveDate {
        now.with_timezone(&tz).date_naive()
    }

    #[test]
    fn e027_midnight_closes_active_sitting_and_standing_at_explicit_boundary() {
        let boundary = Utc.with_ymd_and_hms(2026, 10, 10, 0, 0, 0).unwrap();
        let day = boundary.date_naive();
        let start = boundary - Duration::minutes(40);

        for active in [DeskState::Sitting, DeskState::Standing] {
            let mut m = SessionManager::new();
            m.state.state = active.clone();
            m.state.sitting_started = (active == DeskState::Sitting).then_some(start);
            m.state.standing_bout_started = (active == DeskState::Standing).then_some(start);
            m.state.standing_session_started = (active == DeskState::Standing).then_some(start);
            m.state.sitting_seconds = 2_400;
            m.state.sitting_seconds_total = 2_400;
            m.state.standing_seconds = 2_400;
            m.state.last_tick_ts = Some(boundary);
            m.history_span_started = Some(start);
            m.history_last_reading = Some(boundary);
            m.last_reset_date = day - Duration::days(1);
            m.last_reset_check = boundary - Duration::seconds(120);

            assert!(m.check_daily_reset_at_boundary(boundary, day, boundary));
            assert_eq!(m.state.state, active);
            assert_eq!(m.pending_history.len(), 1,
                "the pre-boundary observed span must be queued once");
            assert_eq!(m.pending_history[0].span.duration_secs, 2_400);
            assert_eq!(m.pending_history[0].state, active);
            if active == DeskState::Sitting {
                assert_eq!(m.state.sitting_started, Some(boundary));
            } else {
                assert_eq!(m.state.standing_bout_started, Some(boundary));
                assert_eq!(m.state.standing_session_started, Some(boundary));
            }
        }
    }

    #[test]
    fn e027_active_postures_split_at_explicit_23_and_25_hour_local_midnights() {
        let cases = [
            (DeskState::Sitting, utc_at(Warsaw, 2026, 3, 29, 0, 0),
                utc_at(Warsaw, 2026, 3, 30, 0, 0), 23 * 60 * 60),
            (DeskState::Standing, utc_at(Warsaw, 2026, 10, 25, 0, 0),
                utc_at(Warsaw, 2026, 10, 26, 0, 0), 25 * 60 * 60),
        ];

        for (posture, start, boundary, expected_secs) in cases {
            assert_eq!((boundary - start).num_seconds(), expected_secs);
            let mut m = SessionManager::new();
            m.state.state = posture.clone();
            m.state.sitting_started = (posture == DeskState::Sitting).then_some(start);
            m.state.standing_bout_started = (posture == DeskState::Standing).then_some(start);
            m.state.standing_session_started = (posture == DeskState::Standing).then_some(start);
            m.history_span_started = Some(start);
            m.history_last_reading = Some(boundary);
            m.state.last_tick_ts = m.history_last_reading;
            let old_day = start.with_timezone(&Warsaw).date_naive();
            let new_day = boundary.with_timezone(&Warsaw).date_naive();
            m.last_reset_date = old_day;
            m.last_reset_check = boundary - Duration::seconds(120);

            assert!(m.check_daily_reset_at_boundary(boundary, new_day, boundary));
            assert_eq!(m.pending_history.len(), 1);
            assert_eq!(m.pending_history[0].state, posture);
            assert_eq!(m.pending_history[0].span.duration_secs, expected_secs);
            assert_eq!(m.history_span_started, Some(boundary));
            if posture == DeskState::Sitting {
                assert_eq!(m.state.sitting_started, Some(boundary));
            } else {
                assert_eq!(m.state.standing_bout_started, Some(boundary));
                assert_eq!(m.state.standing_session_started, Some(boundary));
            }
        }
    }

    #[test]
    fn e027_reset_after_unobserved_midnight_closes_only_through_last_reading() {
        let boundary = utc_at(Warsaw, 2026, 10, 10, 0, 0);
        let last_reading = boundary - Duration::seconds(10);
        let start = boundary - Duration::minutes(40);
        let previous_day = boundary.with_timezone(&Warsaw).date_naive() - Duration::days(1);
        let reset_day = boundary.with_timezone(&Warsaw).date_naive();
        let mut m = SessionManager::new();
        m.state.state = DeskState::Sitting;
        m.state.sitting_started = Some(start);
        m.history_span_started = Some(start);
        m.history_last_reading = Some(last_reading);
        m.state.last_tick_ts = Some(last_reading);
        m.last_reset_date = previous_day;
        m.last_reset_check = boundary - Duration::seconds(120);

        assert!(m.check_daily_reset_at_boundary(
            boundary + Duration::seconds(30), reset_day, boundary,
        ));
        assert_eq!(m.pending_history.len(), 1);
        assert_eq!(m.pending_history[0].span.duration_secs, 2_390);
        assert_eq!(m.pending_history[0].span.ended_at, last_reading.to_rfc3339());
        assert_eq!(last_reading.with_timezone(&Warsaw).date_naive(), previous_day);
        assert_eq!(m.state.sitting_seconds_total, 0,
            "daily counters reset after the previous day's known prefix is queued");
        assert_eq!(m.state.sitting_started, None);
        assert_eq!(m.history_span_started, None,
            "no post-midnight coverage exists until a new reading arrives");

        let resumed = boundary + Duration::seconds(40);
        for _ in 0..DEBOUNCE_COUNT {
            m.on_reading_at(800, true, resumed);
        }
        assert_eq!(m.history_span_started, Some(resumed));
        assert_eq!(m.state.sitting_started, Some(resumed));
        assert_eq!(m.pending_history.len(), 1,
            "the unobserved prefix to midnight must not be synthesized");
    }

    #[test]
    fn e027_at_reset_uses_injected_local_day_without_process_timezone() {
        let now = utc_at(Warsaw, 2026, 10, 10, 0, 30);
        let injected_day = local_date_of(now) + Duration::days(2);
        assert_ne!(injected_day, local_date_of(now),
            "replayed local day deliberately differs from this process's local calendar day");
        let mut m = SessionManager::new();
        m.last_reset_date = injected_day - Duration::days(1);
        m.last_reset_check = now - Duration::seconds(120);

        assert!(m.check_daily_reset_at(now, injected_day));
        assert_eq!(m.last_reset_date, injected_day);
    }

    #[test]
    fn e027_nonexistent_midnight_uses_first_valid_injected_zone_instant() {
        let day = NaiveDate::from_ymd_opt(2018, 11, 4).unwrap();
        let boundary = crate::session_daily::local_day_start_utc(&Sao_Paulo, day).unwrap();
        let local_boundary = boundary.with_timezone(&Sao_Paulo);
        assert_eq!(local_boundary.date_naive(), day);
        assert_eq!(local_boundary.time(), chrono::NaiveTime::from_hms_opt(1, 0, 0).unwrap());

        let now = boundary + Duration::seconds(30);
        let start = boundary - Duration::seconds(30);
        let mut m = SessionManager::new();
        m.state.state = DeskState::Standing;
        m.state.standing_bout_started = Some(start);
        m.state.standing_session_started = Some(start);
        m.history_span_started = Some(start);
        m.history_last_reading = Some(now);
        m.state.last_tick_ts = Some(now);
        m.last_reset_date = day - Duration::days(1);
        m.last_reset_check = now - Duration::seconds(120);

        assert!(m.check_daily_reset_at_boundary(now, day, boundary));
        assert_eq!(m.pending_history.len(), 1);
        assert_eq!(m.pending_history[0].span.started_at, start.to_rfc3339());
        assert_eq!(m.pending_history[0].span.ended_at, boundary.to_rfc3339());
        assert_eq!(m.pending_history[0].span.duration_secs, 30);
        assert_eq!(m.history_span_started, Some(boundary));
    }

    /// Drives `count` readings one second apart starting at `start`.
    /// Returns the instant after the last reading.
    fn feed(
        m: &mut SessionManager,
        mm: i32,
        active: bool,
        start: DateTime<Utc>,
        count: i64,
    ) -> DateTime<Utc> {
        for i in 0..count {
            let _ = m.on_reading_at(mm, active, start + Duration::seconds(i));
        }
        start + Duration::seconds(count)
    }

    // ── Purity ───────────────────────────────────────────────────────────────

    /// The same readings at the same instants must produce the same counters,
    /// no matter when the test runs. This is what clock injection buys.
    #[test]
    fn replay_at_fixed_instants_is_deterministic() {
        let start = Utc.with_ymd_and_hms(2026, 6, 1, 8, 0, 0).unwrap();

        let run = |base: DateTime<Utc>| {
            let mut m = SessionManager::new();
            let after = feed(&mut m, 800, true, base, DEBOUNCE_COUNT as i64);
            assert_eq!(m.state.state, DeskState::Sitting);
            // 4 minutes of sitting, then stand up. Kept under
            // SLEEP_GAP_THRESHOLD_SECS so this measures plain accumulation
            // rather than the sleep-gap path (covered in session_tests_sleep).
            let stand_at = after + Duration::seconds(4 * 60);
            for i in 0..DEBOUNCE_COUNT as i64 {
                let _ = m.on_reading_at(1200, true, stand_at + Duration::seconds(i));
            }
            (m.state.state.clone(), m.state.sitting_seconds)
        };

        let first = run(start);
        let second = run(start);
        assert_eq!(first, second, "replay must be reproducible");
        assert_eq!(first.0, DeskState::Standing);
        assert!(
            first.1 >= 4 * 60,
            "4 min of injected sitting must be committed on exit, got {}s",
            first.1
        );
    }

    /// `on_reading_at` must not consult the system clock: a reading dated a year
    /// in the past still lands a full sitting bout, with no sleep-gap credit
    /// against "now".
    #[test]
    fn on_reading_at_ignores_the_system_clock() {
        let long_ago = Utc.with_ymd_and_hms(2025, 1, 2, 9, 0, 0).unwrap();
        let mut m = SessionManager::new();
        let after = feed(&mut m, 800, true, long_ago, DEBOUNCE_COUNT as i64);
        assert_eq!(m.state.state, DeskState::Sitting);
        assert_eq!(
            m.state.last_tick_ts,
            Some(after - Duration::seconds(1)),
            "last tick must be the injected instant, not the wall clock"
        );
    }

    // ── D3: midnight rollover on the LOCAL calendar ──────────────────────────

    /// At 22:30 UTC on 2026-06-05 the Warsaw wall clock already reads
    /// 2026-06-06 00:30 — a new day for the user. The reset must fire.
    /// The UTC-day comparison this replaces would not fire until 02:00 local.
    #[test]
    fn daily_reset_fires_at_local_midnight_not_utc_midnight() {
        let now = Utc.with_ymd_and_hms(2026, 6, 5, 22, 30, 0).unwrap();
        let today_local = local_day(Warsaw, now);
        assert_eq!(
            today_local,
            NaiveDate::from_ymd_opt(2026, 6, 6).unwrap(),
            "precondition: local day is already the 6th while UTC says the 5th"
        );

        let mut m = SessionManager::new();
        m.state.sitting_seconds = 2400;
        m.state.standing_seconds = 3600;
        m.last_reset_date = NaiveDate::from_ymd_opt(2026, 6, 5).unwrap();
        m.last_reset_check = now - Duration::seconds(120);

        // Regression proof: on the pre-D3 behaviour — comparing UTC days — the
        // reset does not fire here at all. This assertion is what makes the
        // test fail if someone reinstates `now.date_naive()`.
        assert!(
            !m.needs_daily_reset_at(now, now.date_naive()),
            "the UTC-day comparison misses the local rollover"
        );

        assert!(m.needs_daily_reset_at(now, today_local));
        assert!(m.check_daily_reset_at(now, today_local), "reset must fire");
        assert_eq!(m.state.sitting_seconds, 0);
        assert_eq!(m.state.standing_seconds, 0);
        assert_eq!(m.last_reset_date, today_local);
    }

    /// The mirror image: 00:30 UTC on 2026-06-06 is still 2026-06-06 in Warsaw,
    /// but a UTC comparison sees a fresh day and would wipe the counters at
    /// 02:30 local — in the middle of a night shift.
    #[test]
    fn daily_reset_does_not_fire_after_utc_midnight_same_local_day() {
        let now = Utc.with_ymd_and_hms(2026, 6, 6, 0, 30, 0).unwrap();
        let today_local = local_day(Warsaw, now);

        let mut m = SessionManager::new();
        m.state.sitting_seconds = 2400;
        // Already reset for the local day that began 2.5 h ago.
        m.last_reset_date = today_local;
        m.last_reset_check = now - Duration::seconds(120);

        assert!(!m.needs_daily_reset_at(now, today_local));
        assert!(!m.check_daily_reset_at(now, today_local));
        assert_eq!(m.state.sitting_seconds, 2400, "counters must survive");
    }

    /// The 60-second throttle still applies once the day has rolled over.
    #[test]
    fn daily_reset_stays_throttled_across_the_rollover() {
        let now = Utc.with_ymd_and_hms(2026, 6, 5, 22, 30, 0).unwrap();
        let today_local = local_day(Warsaw, now);

        let mut m = SessionManager::new();
        m.state.sitting_seconds = 1000;
        m.last_reset_date = NaiveDate::from_ymd_opt(2026, 6, 5).unwrap();
        m.last_reset_check = now - Duration::seconds(30);

        assert!(!m.check_daily_reset_at(now, today_local), "throttled");
        assert_eq!(m.state.sitting_seconds, 1000);

        // 31 seconds later the throttle window has passed.
        let later = now + Duration::seconds(31);
        assert!(m.check_daily_reset_at(later, local_day(Warsaw, later)));
        assert_eq!(m.state.sitting_seconds, 0);
    }

    // ── DST transition days ──────────────────────────────────────────────────

    /// 2026-10-25 is 25 hours long in Warsaw (02:00 falls back to 01:00).
    /// The local day must not roll over anywhere inside it — a UTC comparison
    /// rolls at 01:00 local (after the fall-back) and resets mid-morning.
    #[test]
    fn dst_fall_back_day_holds_one_local_day_for_25_hours() {
        let day = NaiveDate::from_ymd_opt(2026, 10, 25).unwrap();
        let start = utc_at(Warsaw, 2026, 10, 25, 0, 0);
        let next_midnight = utc_at(Warsaw, 2026, 10, 26, 0, 0);
        let span_hours = (next_midnight - start).num_hours();
        assert_eq!(span_hours, 25, "precondition: fall-back day is 25 h long");

        let mut m = SessionManager::new();
        m.last_reset_date = day;
        m.last_reset_check = start;

        for h in 0..span_hours {
            let now = start + Duration::hours(h);
            let today_local = local_day(Warsaw, now);
            assert_eq!(
                today_local, day,
                "hour {h} of the fall-back day must still be {day}"
            );
            m.last_reset_check = now - Duration::seconds(120);
            assert!(
                !m.check_daily_reset_at(now, today_local),
                "no reset may fire inside the local day (hour {h})"
            );
        }

        // The first instant of the next local day does reset.
        m.last_reset_check = next_midnight - Duration::seconds(120);
        assert!(m.check_daily_reset_at(next_midnight, local_day(Warsaw, next_midnight)));
        assert_eq!(
            m.last_reset_date,
            NaiveDate::from_ymd_opt(2026, 10, 26).unwrap()
        );
    }

    /// 2026-03-29 is 23 hours long in Warsaw (02:00 springs to 03:00).
    /// A short day must still produce exactly one reset, at its end.
    #[test]
    fn dst_spring_forward_day_resets_once_at_its_end() {
        let day = NaiveDate::from_ymd_opt(2026, 3, 29).unwrap();
        let start = utc_at(Warsaw, 2026, 3, 29, 0, 0);
        let next_midnight = utc_at(Warsaw, 2026, 3, 30, 0, 0);
        assert_eq!(
            (next_midnight - start).num_hours(),
            23,
            "precondition: spring-forward day is 23 h long"
        );

        let mut m = SessionManager::new();
        m.last_reset_date = day;
        let mut resets = 0;
        let mut now = start;
        while now <= next_midnight {
            m.last_reset_check = now - Duration::seconds(120);
            if m.check_daily_reset_at(now, local_day(Warsaw, now)) {
                resets += 1;
            }
            now += Duration::minutes(30);
        }
        assert_eq!(resets, 1, "exactly one reset across a 23-hour day");
        assert_eq!(
            m.last_reset_date,
            NaiveDate::from_ymd_opt(2026, 3, 30).unwrap()
        );
    }

    /// `local_date_of` is the impure boundary the wrappers use. It must agree
    /// with an explicit conversion into the machine's own zone.
    #[test]
    fn local_date_of_matches_the_machine_zone() {
        let now = Utc::now();
        assert_eq!(local_date_of(now), now.with_timezone(&chrono::Local).date_naive());
    }
}
