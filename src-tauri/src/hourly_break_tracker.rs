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
use std::collections::HashMap;

/// AWAY_BREAK_THRESHOLD_SECS: minimum continuous Away seconds to count as a break.
const AWAY_BREAK_THRESHOLD_SECS: i64 = 300; // 5 minutes

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
}

impl HourlyBreakTracker {
    pub fn new() -> Self {
        Self {
            hours_with_break: HashMap::new(),
            hours_active: HashMap::new(),
            current_away_secs: 0,
        }
    }

    /// Call every tick when the user is Away.
    /// `hour` is the current clock hour (0-23).
    ///
    /// Logs `[break:hourly]` once, on the tick where the hour first becomes
    /// covered — not on every tick past the threshold.
    pub fn tick_away(&mut self, hour: u8) {
        self.hours_active.insert(hour, true);
        self.current_away_secs += 1;
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
    /// `hour` is the current clock hour (0-23).
    pub fn tick_active(&mut self, hour: u8) {
        self.hours_active.insert(hour, true);
        self.current_away_secs = 0;
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
        Self { hours_with_break, hours_active, current_away_secs }
    }

    /// Reset for a new day.
    pub fn reset(&mut self) {
        self.hours_with_break.clear();
        self.hours_active.clear();
        self.current_away_secs = 0;
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
