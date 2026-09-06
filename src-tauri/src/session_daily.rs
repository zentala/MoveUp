//! session_daily.rs — Daily reset, score accumulation, and alert logic.

use chrono::{DateTime, Local, NaiveDate, Utc};
use log::info;

use crate::session_types::*;
use crate::session_manager::SessionManager;

/// The calendar day `now` falls on **in the machine's local time zone** (D3).
///
/// Every other date-keyed surface in this app — `session_persistence.rs`,
/// `db_sessions.rs` — keys on the local day, so the daily reset must too.
/// Deriving it from UTC fired the reset at 01:00 or 02:00 local in Warsaw.
///
/// This is the second impure boundary of the engine (it reads the system time
/// zone). The reset functions take the resulting date as an explicit argument
/// so a test can pass any zone's date without touching the machine's.
pub fn local_date_of(now: DateTime<Utc>) -> NaiveDate {
    now.with_timezone(&Local).date_naive()
}

impl SessionManager {
    /// Returns true if a daily reset would occur on the next `check_daily_reset` call.
    /// Does NOT mutate state — safe to call for pre-reset telemetry.
    ///
    /// Impure boundary; the logic lives in [`SessionManager::needs_daily_reset_at`].
    pub fn needs_daily_reset(&self) -> bool {
        let now = Utc::now();
        self.needs_daily_reset_at(now, local_date_of(now))
    }

    /// Returns true if [`SessionManager::check_daily_reset_at`] would reset at
    /// `now`, given `today_local` as the current LOCAL calendar day (D3).
    pub fn needs_daily_reset_at(&self, now: DateTime<Utc>, today_local: NaiveDate) -> bool {
        if (now - self.last_reset_check).num_seconds() < 60 {
            return false;
        }
        self.last_reset_date < today_local
    }

    /// Checks if a new day has begun and resets daily counters (throttled to 60s).
    ///
    /// Impure boundary; the logic lives in [`SessionManager::check_daily_reset_at`].
    pub fn check_daily_reset(&mut self) -> bool {
        let now = Utc::now();
        self.check_daily_reset_at(now, local_date_of(now))
    }

    /// Resets daily counters when `today_local` is past `last_reset_date`
    /// (throttled to one check per 60s of `now`).
    ///
    /// `today_local` is the current **local** calendar day, passed in rather
    /// than derived here (D3) so callers replaying a fixed timeline — and the
    /// DST test — control both the instant and the day boundary.
    pub fn check_daily_reset_at(&mut self, now: DateTime<Utc>, today_local: NaiveDate) -> bool {
        let today = today_local;
        if (now - self.last_reset_check).num_seconds() < 60 {
            return false;
        }
        self.last_reset_check = now;
        if self.last_reset_date < today {
            info!("daily reset: new day detected, resetting in-memory counters");
            self.state.sitting_seconds = 0;
            self.state.sitting_seconds_total = 0;
            self.state.standing_seconds = 0;
            self.state.position_changes = 0;
            self.state.last_break_secs = 0;
            self.state.last_sitting_secs = 0;
            self.state.last_break_credit = BreakCredit::None;
            self.alert_fired = false;
            self.stand_alert_fired = false;
            self.notify_inactivity_fired = false;
            self.notify_posture_balance_fired = false;
            self.praise_halfway_fired_today = false;
            self.standing_target_reached_fired = false;
            self.state.daily_score = 0.0;
            self.state.standing_session_secs = 0;
            self.state.standing_session_started = None;
            self.state.lap_bonus_awarded_for_lap = 0;
            self.state.continuous_computer_secs = 0;
            self.state.longest_computer_session_secs = 0;
            self.state.away_bout_secs = 0;
            self.state.first_reading_at = None;
            self.state.last_tick_ts = None;
            self.state.last_accumulate_ts = None;
            self.state.standing_bout_started = None;
            self.state.hourly_breaks_covered = 0;
            self.state.hourly_breaks_active = 0;
            self.hourly_break_tracker.reset();
            self.last_reset_date = today;
            return true;
        }
        false
    }

    /// Accumulates score every tick (~1s) using the system clock.
    ///
    /// Impure boundary; the logic lives in
    /// [`SessionManager::accumulate_score_tick_at`].
    pub fn accumulate_score_tick(&mut self, ergo: &crate::ergonomic_profile::ErgonomicProfile) {
        self.accumulate_score_tick_at(ergo, Utc::now());
    }

    /// Accumulates score for the tick at `now`. Called from serial_periodic
    /// after `on_reading_at`, with the same `now`.
    ///
    /// Caller must gate this behind `last_accumulate_ran` to ensure 1 Hz rate.
    pub fn accumulate_score_tick_at(
        &mut self,
        ergo: &crate::ergonomic_profile::ErgonomicProfile,
        now: DateTime<Utc>,
    ) {
        match self.state.state {
            DeskState::Sitting => {
                self.state.daily_score += ergo.scoring.pts_sitting_per_min / 60.0;
            }
            DeskState::Standing => {
                self.state.daily_score += ergo.scoring.pts_standing_per_min / 60.0;

                // Compute live standing session duration from timestamp.
                let live_standing_secs = self.state.standing_session_started
                    .map(|s| (now - s).num_seconds().max(0))
                    .unwrap_or(self.state.standing_session_secs);
                self.state.standing_session_secs = live_standing_secs;

                // Lap bonus: award for each target multiple crossed.
                let target_secs = ergo.limits.standing_target_secs as i64;
                if target_secs > 0 {
                    let current_lap = live_standing_secs / target_secs;
                    let awarded = self.state.lap_bonus_awarded_for_lap as i64;
                    if current_lap > awarded {
                        let missed = current_lap - awarded;
                        self.state.daily_score += ergo.scoring.pts_session_bonus * missed as f32;
                        self.state.lap_bonus_awarded_for_lap = current_lap as u32;
                    }
                }
            }
            _ => {} // Walking / Away: neutral
        }
    }

    /// Returns `true` (once per sitting stint) when sitting limit is reached.
    pub fn should_alert(&mut self) -> bool {
        if self.state.state == DeskState::Sitting
            && !self.alert_fired
            && self.state.sitting_seconds >= self.state.session_limit_secs
        {
            self.alert_fired = true;
            return true;
        }
        false
    }
}
