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
