//! hourly_break_tracker.rs — Tracks per-clock-hour Away breaks for KPI.
//!
//! This is the third and most isolated of the three things called "break" in
//! this app. It answers one question per clock hour — *did this hour contain at
//! least 5 continuous minutes Away?* — and feeds the HourlyBreakCoverage KPI.
//! It never changes what the user is told to do.
//!
//! The other two live in [`crate::session_breaks`]:
//! [`SessionManager::apply_break_credit`](crate::session_manager::SessionManager::apply_break_credit)
//! (Session Break Credit, ADR 008 — a countdown against credited sitting time)
//! and
//! [`SessionManager::apply_day_break_reset`](crate::session_manager::SessionManager::apply_day_break_reset)
//! (Day Break Credit, ADR 009 — a one-shot reset of notification flags). See
//! that module's header for the full comparison.
//!
//! Three differences worth keeping in mind before merging any of them:
//!
//! - **Trigger.** This tracker is driven per tick from `session_reading.rs`;
//!   the other two fire once, on a return to sitting.
//! - **Unit.** This counts *clock hours*, they count *seconds*.
//! - **Away only.** Standing counts as a break for credit, but not here —
//!   [`Self::tick_active`] treats Standing as active and resets the counter.
//!
//! Log lines from this module carry the `[break:hourly]` prefix; credit lines
//! carry `[break:credit]` and `[break:day]`.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Duration, Timelike, Utc};
use std::collections::HashMap;

/// AWAY_BREAK_THRESHOLD_SECS: minimum continuous Away seconds to count as a break.
const AWAY_BREAK_THRESHOLD_SECS: i64 = 300; // 5 minutes

/// Runtime identity for one concrete local calendar-hour interval. The
/// persisted maps intentionally retain their compatible 0–23 keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LocalHourInterval {
    pub(crate) hour: u8,
    pub(crate) starts_at_utc: DateTime<Utc>,
}

impl LocalHourInterval {
    /// Maps an injected UTC instant through an explicit fixed local offset.
    /// Keeping the UTC start instant distinguishes repeated hours at a fold
    /// and supports offsets that are not whole hours.
    pub(crate) fn from_offset(at: DateTime<Utc>, offset_seconds: i32) -> Self {
        let local = at.naive_utc() + Duration::seconds(offset_seconds as i64);
        let hour = local.hour() as u8;
        let local_start = local.date().and_hms_opt(hour as u32, 0, 0)
            .expect("valid local hour boundary");
        let starts_at_utc = DateTime::<Utc>::from_naive_utc_and_offset(
            local_start - Duration::seconds(offset_seconds as i64), Utc,
        );
        Self { hour, starts_at_utc }
    }
}

/// Tracks per-clock-hour Away breaks for the HourlyBreakCoverage KPI.
/// Binary check per hour: was there ≥5 continuous minutes of Away?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyBreakTracker {
    /// Map of clock hour (0-23) to whether a break was taken in that hour.
    pub(crate) hours_with_break: HashMap<u8, bool>,
    /// Map of clock hour (0-23) to whether the user was active in that hour.
    pub(crate) hours_active: HashMap<u8, bool>,
    /// Current continuous Away duration in the current hour (seconds).
    pub(crate) current_away_secs: i64,
    /// Runtime-only hour for that duration; the persisted count alone cannot
    /// identify which local-hour interval it belongs to after a restart.
    #[serde(skip)]
    current_away_hour: Option<u8>,
    /// Concrete interval identity prevents repeated local clock hours from
    /// sharing one in-memory Away threshold.
    #[serde(skip)]
    current_away_interval_start: Option<DateTime<Utc>>,
    /// Sub-second elapsed Away time is retained between sensor observations;
    /// persisted compatibility stays in whole seconds.
    #[serde(skip)]
    current_away_subsec_nanos: u32,
}

impl HourlyBreakTracker {
    pub fn new() -> Self {
        Self {
            hours_with_break: HashMap::new(),
            hours_active: HashMap::new(),
            current_away_secs: 0,
            current_away_hour: None,
            current_away_interval_start: None,
            current_away_subsec_nanos: 0,
        }
    }

    /// Call every tick when the user is Away.
    /// `hour` is the injected local calendar hour (0-23); this method reads no
    /// clock or timezone. Each call represents one known Away second.
    ///
    /// Logs `[break:hourly]` once, on the tick where the hour first becomes
    /// covered — not on every tick past the threshold.
    pub fn tick_away(&mut self, hour: u8) {
        self.add_known_away_nanos(hour, None, 1_000_000_000);
    }

    fn add_known_away_nanos(
        &mut self,
        hour: u8,
        interval_start: Option<DateTime<Utc>>,
        nanos: i64,
    ) {
        if self.current_away_hour != Some(hour)
            || self.current_away_interval_start != interval_start
        {
            self.current_away_secs = 0;
            self.current_away_subsec_nanos = 0;
            self.current_away_hour = Some(hour);
            self.current_away_interval_start = interval_start;
        }
        self.hours_active.insert(hour, true);
        let accumulated = self.current_away_subsec_nanos as i64 + nanos;
        self.current_away_secs += accumulated / 1_000_000_000;
        self.current_away_subsec_nanos = (accumulated % 1_000_000_000) as u32;
        if self.current_away_secs >= AWAY_BREAK_THRESHOLD_SECS {
            let first = self.hours_with_break.insert(hour, true).is_none();
            if first {
                log::info!(
                    "[break:hourly] hour {} covered after {}s continuous Away",
                    hour, self.current_away_secs
                );
            }
        }
    }

    /// Call every tick when the user is NOT Away (Sitting/Standing/Walking).
    /// Resets the continuous Away counter.
    /// `hour` is the injected local calendar hour (0-23); this method reads no
    /// clock or timezone.
    pub fn tick_active(&mut self, hour: u8) {
        self.hours_active.insert(hour, true);
        self.current_away_secs = 0;
        self.current_away_hour = None;
        self.current_away_interval_start = None;
        self.current_away_subsec_nanos = 0;
    }

    /// Allocates a known observation interval to each explicit local-hour
    /// bucket it intersects. Duplicate instants add no time; callers omit
    /// intervals across unknown gaps.
    pub fn record_known_interval<F>(
        &mut self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        away: bool,
        local_hour_at: F,
    ) where
        F: Fn(DateTime<Utc>) -> LocalHourInterval,
    {
        let mut cursor = start;
        while cursor < end {
            let window_end = (cursor + Duration::seconds(1)).min(end);
            let interval = local_hour_at(cursor);
            let last_in_window = window_end - Duration::nanoseconds(1);
            let end_interval = local_hour_at(last_in_window);
            let segment_end = if end_interval == interval {
                window_end
            } else {
                // Hour changes are rare; locate the exact injected boundary so
                // fractional cadence cannot leak time into the adjacent hour.
                let mut low = 0i64;
                let mut high = (window_end - cursor).num_nanoseconds().unwrap();
                while high - low > 1 {
                    let middle = low + (high - low) / 2;
                    let at = cursor + Duration::nanoseconds(middle);
                    if local_hour_at(at) == interval {
                        low = middle;
                    } else {
                        high = middle;
                    }
                }
                cursor + Duration::nanoseconds(high)
            };
            let nanos = (segment_end - cursor).num_nanoseconds().unwrap_or(0).max(0);
            if away {
                self.add_known_away_nanos(
                    interval.hour, Some(interval.starts_at_utc), nanos,
                );
            } else {
                self.tick_active(interval.hour);
            }
            cursor = segment_end;
        }
    }

    /// Unknown time breaks Away continuity without earning coverage.
    pub fn break_away_continuity(&mut self) {
        self.current_away_secs = 0;
        self.current_away_hour = None;
        self.current_away_interval_start = None;
        self.current_away_subsec_nanos = 0;
    }

    /// Number of completed work hours that had a break.
    pub fn hours_with_break(&self) -> u8 {
        self.hours_with_break.values().filter(|&&v| v).count() as u8
    }

    /// Number of hours where the user was active (for denominator).
    pub fn hours_active(&self) -> u8 {
        self.hours_active.len() as u8
    }

    /// Restores tracker from persisted state (app restart within same day).
    pub fn restore(
        hours_with_break: HashMap<u8, bool>,
        hours_active: HashMap<u8, bool>,
        current_away_secs: i64,
    ) -> Self {
        Self { hours_with_break, hours_active, current_away_secs, current_away_hour: None,
            current_away_interval_start: None,
            current_away_subsec_nanos: 0 }
    }

    /// Reset for a new day.
    pub fn reset(&mut self) {
        self.hours_with_break.clear();
        self.hours_active.clear();
        self.current_away_secs = 0;
        self.current_away_hour = None;
        self.current_away_interval_start = None;
        self.current_away_subsec_nanos = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tracker_is_empty() {
        let t = HourlyBreakTracker::new();
        assert_eq!(t.hours_with_break(), 0);
        assert_eq!(t.hours_active(), 0);
    }

    #[test]
    fn tick_active_marks_hour() {
        let mut t = HourlyBreakTracker::new();
        t.tick_active(9);
        assert_eq!(t.hours_active(), 1);
    }

    #[test]
    fn tick_away_marks_break_after_threshold() {
        let mut t = HourlyBreakTracker::new();
        for _ in 0..300 {
            t.tick_away(10);
        }
        assert_eq!(t.hours_with_break(), 1);
    }

    #[test]
    fn tick_away_not_enough() {
        let mut t = HourlyBreakTracker::new();
        for _ in 0..299 {
            t.tick_away(10);
        }
        assert_eq!(t.hours_with_break(), 0);
    }

    #[test]
    fn tick_active_resets_away_counter() {
        let mut t = HourlyBreakTracker::new();
        for _ in 0..200 {
            t.tick_away(10);
        }
        t.tick_active(10);
        for _ in 0..200 {
            t.tick_away(10);
        }
        assert_eq!(t.hours_with_break(), 0);
    }

    #[test]
    fn covered_hour_stays_covered_past_threshold() {
        let mut t = HourlyBreakTracker::new();
        for _ in 0..600 {
            t.tick_away(10);
        }
        assert_eq!(t.hours_with_break(), 1, "extra ticks must not double-count");
        assert_eq!(t.hours_active(), 1);
    }

    #[test]
    fn breaks_are_tracked_per_clock_hour() {
        let mut t = HourlyBreakTracker::new();
        for _ in 0..300 {
            t.tick_away(10);
        }
        t.tick_active(11);
        for _ in 0..300 {
            t.tick_away(11);
        }
        assert_eq!(t.hours_with_break(), 2);
        assert_eq!(t.hours_active(), 2);
    }

    #[test]
    fn reset_clears_all() {
        let mut t = HourlyBreakTracker::new();
        t.tick_active(9);
        for _ in 0..300 {
            t.tick_away(10);
        }
        assert!(t.hours_active() > 0);
        t.reset();
        assert_eq!(t.hours_with_break(), 0);
        assert_eq!(t.hours_active(), 0);
        assert_eq!(t.current_away_secs, 0);
    }

}
