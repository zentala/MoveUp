//! session_tests_scenario_table.rs — one regression table over the DTO (E020-T06).
//!
//! Every other `session_tests_*` file reaches into `SessionManager` and asserts
//! on internal counters. This one drives the engine only through
//! `on_reading_at` / `check_daily_reset_at` and asserts only on
//! [`SessionStateDto`] — the surface the UI actually consumes. A refactor that
//! keeps the internals green while changing what the user sees fails here.
//!
//! Each row names the pre-existing test it supersedes. **None of those tests
//! were deleted or inverted**: E020-T06's write set is this file plus `lib.rs`,
//! so the older tests stay exactly as they are and this table sits over them as
//! a second, DTO-level net. Where a row asserts behaviour an older test does
//! not cover at all, the row says so.
//!
//! Time is injected everywhere. Rows that must assert a *live* DTO field anchor
//! their timeline so the last event lands on `Utc::now()` (`Anchor::EndsNow`);
//! rows that need a specific calendar date anchor to a fixed instant
//! (`Anchor::Fixed`) and assert only fields the DTO does not extrapolate in the
//! state the row ends in.

#[cfg(test)]
mod scenario_table {
    use chrono::{DateTime, Duration, TimeZone, Utc};
    use chrono_tz::Europe::Warsaw;

    use crate::session_manager::SessionManager;
    use crate::session_types::*;

    const SIT_MM: i32 = 800;
    const STAND_MM: i32 = 1200;
    /// Seconds between readings inside a hold. Below `SLEEP_GAP_THRESHOLD_SECS`,
    /// so holding a position never looks to the engine like a suspend.
    const HOLD_STEP_SECS: i64 = 240;
    /// A hold confirms its state on its `DEBOUNCE_COUNT`-th reading, i.e. this
    /// many seconds after the hold's `from`. Every hold pays the same lag, so
    /// the *differences* the table asserts are exact.
    const CONFIRM_LAG: i64 = DEBOUNCE_COUNT as i64 - 1;

    /// One step of a scenario. Offsets are seconds from the scenario anchor.
    enum Ev {
        /// Hold `mm` from `from` to `to`; the state confirms at
        /// `from + CONFIRM_LAG`.
        Hold { from: i64, to: i64, mm: i32, active: bool },
        /// A single reading — used to open a gap on purpose.
        Read { at: i64, mm: i32 },
        /// A daily-reset check against the Warsaw local calendar day.
        Reset { at: i64 },
    }

    impl Ev {
        fn end(&self) -> i64 {
            match self {
                Ev::Hold { to, .. } => *to,
                Ev::Read { at, .. } | Ev::Reset { at } => *at,
            }
        }
    }

    enum Anchor {
        /// Place the anchor so the last event lands on `Utc::now()`.
        EndsNow,
        Fixed(DateTime<Utc>),
    }

    #[derive(Default)]
    struct Expect {
        state: Option<DeskState>,
        limit_used_secs: Option<i64>,
        sitting_seconds_total: Option<i64>,
        standing_seconds: Option<i64>,
        position_changes: Option<u32>,
        credit: Option<BreakCredit>,
        resets: Option<usize>,
        /// `(limit_used_secs, sitting_seconds_total)` sampled immediately before
        /// the first `Reset` event.
        before_reset: Option<(i64, i64)>,
        /// Allowed drift on live-extrapolated fields, in seconds.
        tol: i64,
    }

    struct Case {
        name: &'static str,
        supersedes: &'static str,
        anchor: Anchor,
        events: Vec<Ev>,
        expect: Expect,
    }

    struct Run {
        dto: SessionStateDto,
        credit: BreakCredit,
        resets: usize,
        before_reset: Option<(i64, i64)>,
    }

    fn hold(m: &mut SessionManager, anchor: DateTime<Utc>, from: i64, to: i64, mm: i32, active: bool) {
        assert!(to >= from + CONFIRM_LAG, "a hold must be long enough to debounce");
        let mut offsets: Vec<i64> = (0..=CONFIRM_LAG).map(|i| from + i).collect();
        let mut t = from + HOLD_STEP_SECS;
        while t < to {
            offsets.push(t);
            t += HOLD_STEP_SECS;
        }
        if *offsets.last().expect("debounce readings exist") != to {
            offsets.push(to);
        }
        for o in offsets {
            let _ = m.on_reading_at(mm, active, anchor + Duration::seconds(o));
        }
    }

    fn run(case: &Case) -> Run {
        let last = case.events.iter().map(Ev::end).max().expect("scenario has events");
        let anchor = match case.anchor {
            Anchor::EndsNow => Utc::now() - Duration::seconds(last),
            Anchor::Fixed(t) => t,
        };

        let mut m = SessionManager::new();
        // The manager dates itself from the wall clock at construction; a
        // scenario replaying another day has to say which day it starts on.
        m.last_reset_date = anchor.with_timezone(&Warsaw).date_naive();
        m.last_reset_check = anchor - Duration::seconds(120);

        let mut resets = 0usize;
        let mut before_reset = None;
        for ev in &case.events {
            match *ev {
                Ev::Hold { from, to, mm, active } => hold(&mut m, anchor, from, to, mm, active),
                Ev::Read { at, mm } => {
                    let _ = m.on_reading_at(mm, true, anchor + Duration::seconds(at));
                }
                Ev::Reset { at } => {
                    let now = anchor + Duration::seconds(at);
                    if before_reset.is_none() {
                        let d = m.snapshot();
                        before_reset = Some((d.limit_used_secs, d.sitting_seconds_total));
                    }
                    // Clear the 60 s throttle so the row tests the day
                    // comparison, not the throttle (that is covered by
                    // session_tests_clock::daily_reset_stays_throttled_across_the_rollover).
                    m.last_reset_check = now - Duration::seconds(120);
                    if m.check_daily_reset_at(now, now.with_timezone(&Warsaw).date_naive()) {
                        resets += 1;
                    }
                }
            }
        }

        Run { dto: m.snapshot(), credit: m.state.last_break_credit.clone(), resets, before_reset }
    }

    fn near(case: &Case, field: &str, actual: i64, expected: i64) {
        assert!(
            (actual - expected).abs() <= case.expect.tol,
            "{}: {} = {}, expected {} (+/-{}); supersedes {}",
            case.name, field, actual, expected, case.expect.tol, case.supersedes
        );
    }

    fn check(case: &Case) {
        let run = run(case);
        let e = &case.expect;
        if let Some(ref s) = e.state {
            assert_eq!(&run.dto.state, s, "{}: state", case.name);
        }
        if let Some(ref c) = e.credit {
            assert_eq!(&run.credit, c, "{}: break credit", case.name);
        }
        if let Some(v) = e.limit_used_secs {
            near(case, "limit_used_secs", run.dto.limit_used_secs, v);
        }
        if let Some(v) = e.sitting_seconds_total {
            near(case, "sitting_seconds_total", run.dto.sitting_seconds_total, v);
        }
        if let Some(v) = e.standing_seconds {
            near(case, "standing_seconds", run.dto.standing_seconds, v);
        }
        if let Some(v) = e.position_changes {
            assert_eq!(run.dto.position_changes, v, "{}: position_changes", case.name);
        }
        if let Some(v) = e.resets {
            assert_eq!(run.resets, v, "{}: daily resets fired", case.name);
        }
        if let Some((used, total)) = e.before_reset {
            let got = run.before_reset.expect("row declares a pre-reset sample but fires no Reset");
            assert_eq!(got.0, used, "{}: limit_used_secs before reset", case.name);
            assert_eq!(got.1, total, "{}: sitting_seconds_total before reset", case.name);
        }
    }

    /// Warsaw local wall-clock time as a UTC instant.
    fn warsaw(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> DateTime<Utc> {
        Warsaw
            .with_ymd_and_hms(y, mo, d, h, mi, 0)
            .earliest()
            .expect("local time exists")
            .with_timezone(&Utc)
    }

    fn cases() -> Vec<Case> {
        let mul = SessionManager::new().limits.break_credit_multiplier as f64;
        assert!(
            (120.0 * mul) < 1800.0,
            "row 1 assumes a 2-minute break earns partial, not full, credit"
        );
        let credited = 1800 - (120.0 * mul) as i64;

        vec![
            // Row 1 — the E015 complaint, restated on the DTO: returning to
            // sitting must not zero the timer, it must subtract the earned
            // credit. Supersedes nothing (both kept): it is the table's version
            // of session_tests_e015_credited::e015_sit30_stand2_sit_dto_shows_credited_value
            // and session_tests_break_credit's partial-credit cases.
            Case {
                name: "sit30_stand2_sit_shows_credited_value",
                supersedes: "session_tests_e015_credited::e015_sit30_stand2_sit_dto_shows_credited_value (kept)",
                anchor: Anchor::EndsNow,
                events: vec![
                    Ev::Hold { from: 0, to: 1800, mm: SIT_MM, active: true },
                    Ev::Hold { from: 1800, to: 1920, mm: STAND_MM, active: true },
                    Ev::Hold { from: 1920, to: 1920 + CONFIRM_LAG, mm: SIT_MM, active: true },
                ],
                expect: Expect {
                    state: Some(DeskState::Sitting),
                    credit: Some(BreakCredit::Partial),
                    limit_used_secs: Some(credited),
                    sitting_seconds_total: Some(1800),
                    standing_seconds: Some(120),
                    position_changes: Some(2),
                    tol: 3,
                    ..Default::default()
                },
            },
            // Row 2 — a suspend mid-bout. The gap is credited as a break and the
            // start timestamps are rewound, so the DTO shows a fresh session
            // rather than hours of phantom sitting.
            //
            // NOTE, and it is not what the older tests assert: because neither
            // counter is committed until the bout ends, the 40 minutes sat
            // BEFORE the suspend disappear from `sitting_seconds_total` too —
            // the raw daily KPI that break credit is never supposed to touch.
            // session_tests_sleep::sleep_gap_preserves_sitting_seconds_total
            // only covers already-committed time, so it stays green. This row
            // pins today's behaviour; see the T06 report for the defect note.
            Case {
                name: "sleep_gap_credits_the_bout_and_rewinds",
                supersedes: "session_tests_sleep::sleep_gap_full_credit_resets_sitting (kept)",
                anchor: Anchor::EndsNow,
                events: vec![
                    Ev::Hold { from: 0, to: 2400, mm: SIT_MM, active: true },
                    Ev::Read { at: 2400 + 7200, mm: SIT_MM },
                ],
                expect: Expect {
                    state: Some(DeskState::Sitting),
                    credit: Some(BreakCredit::Full),
                    limit_used_secs: Some(1),
                    sitting_seconds_total: Some(1),
                    tol: 3,
                    ..Default::default()
                },
            },
            // Row 3 — midnight on the LOCAL calendar (D3). The scenario runs the
            // half hour before Warsaw midnight, then checks the reset one second
            // into the new local day. Everything the DTO reports goes to zero.
            Case {
                name: "local_midnight_rollover_zeroes_the_dto",
                supersedes: "session_tests_clock::daily_reset_fires_at_local_midnight_not_utc_midnight (kept)",
                anchor: Anchor::Fixed(warsaw(2026, 6, 5, 23, 0)),
                events: vec![
                    Ev::Hold { from: 0, to: 1800, mm: SIT_MM, active: true },
                    Ev::Hold { from: 1800, to: 1800 + CONFIRM_LAG, mm: STAND_MM, active: true },
                    Ev::Reset { at: 3600 },
                ],
                expect: Expect {
                    state: Some(DeskState::Standing),
                    resets: Some(1),
                    before_reset: Some((1800, 1800)),
                    limit_used_secs: Some(0),
                    sitting_seconds_total: Some(0),
                    position_changes: Some(0),
                    tol: 0,
                    ..Default::default()
                },
            },
            // Row 4 — the Warsaw fall-back day. Sitting from 00:30 to 03:30 local
            // is FOUR hours of real time, because 02:00 happens twice. The engine
            // measures instants, so the DTO must report 14400 s, not 10800. The
            // day must not roll over inside those 25 hours, and must roll over
            // exactly once at the next local midnight.
            Case {
                name: "dst_fall_back_day_counts_instants_not_wall_clock",
                supersedes: "session_tests_clock::dst_fall_back_day_holds_one_local_day_for_25_hours (kept)",
                anchor: Anchor::Fixed(warsaw(2026, 10, 25, 0, 30)),
                events: vec![
                    Ev::Hold { from: 0, to: 4 * 3600, mm: SIT_MM, active: true },
                    Ev::Hold { from: 4 * 3600, to: 4 * 3600 + CONFIRM_LAG, mm: STAND_MM, active: true },
                    // Still 25 October in Warsaw — no reset.
                    Ev::Reset { at: 20 * 3600 },
                    // 26 October 00:00 local = 23:00 UTC on the 25th.
                    Ev::Reset { at: 88_200 },
                ],
                expect: Expect {
                    state: Some(DeskState::Standing),
                    resets: Some(1),
                    before_reset: Some((4 * 3600, 4 * 3600)),
                    limit_used_secs: Some(0),
                    sitting_seconds_total: Some(0),
                    tol: 0,
                    ..Default::default()
                },
            },
        ]
    }

    #[test]
    fn scenario_table_rows_hold() {
        for case in cases() {
            check(&case);
        }
    }

    /// The premise row 4 rests on: three hours on the Warsaw wall clock are four
    /// hours of real time on the fall-back day. If this ever stops holding, row
    /// 4 is asserting nothing.
    #[test]
    fn scenario_table_dst_precondition() {
        let start = warsaw(2026, 10, 25, 0, 30);
        let end = warsaw(2026, 10, 25, 3, 30);
        assert_eq!((end - start).num_hours(), 4, "fall-back day repeats 02:00");
    }
}
