//! session_daily.rs — Daily reset, score accumulation, and alert logic.

use chrono::Utc;
use log::info;

use crate::session_types::*;
use crate::session_manager::SessionManager;

impl SessionManager {
    /// Returns true if a daily reset would occur on the next `check_daily_reset` call.
    /// Does NOT mutate state — safe to call for pre-reset telemetry.
    pub fn needs_daily_reset(&self) -> bool {
        let now = Utc::now();
        let today = now.date_naive();
        if (now - self.last_reset_check).num_seconds() < 60 {
            return false;
        }
        self.last_reset_date < today
    }

    /// Checks if a new day has begun and resets daily counters (throttled to 60s).
    pub fn check_daily_reset(&mut self) -> bool {
        let now = Utc::now();
        let today = now.date_naive();
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

    /// Accumulates score every tick (~1s). Called from serial_periodic after on_reading.
    ///
    /// Caller must gate this behind `last_accumulate_ran` to ensure 1 Hz rate.
    pub fn accumulate_score_tick(&mut self, ergo: &crate::ergonomic_profile::ErgonomicProfile) {
        match self.state.state {
            DeskState::Sitting => {
                self.state.daily_score += ergo.scoring.pts_sitting_per_min / 60.0;
            }
            DeskState::Standing => {
                self.state.daily_score += ergo.scoring.pts_standing_per_min / 60.0;

                // Compute live standing session duration from timestamp.
                let live_standing_secs = self.state.standing_session_started
                    .map(|s| (Utc::now() - s).num_seconds().max(0))
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
