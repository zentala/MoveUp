//! session_daily.rs — Daily reset, score accumulation, and alert logic.

use chrono::Utc;
use log::info;

use crate::session_types::*;
use crate::session_manager::SessionManager;

impl SessionManager {
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
            self.state.lap_bonus_awarded_for_lap = 0;
            self.state.current_session_secs = 0;
            self.last_reset_date = today;
            return true;
        }
        false
    }

    /// Accumulates score every tick (~1s). Called from serial_periodic after on_reading.
    pub fn accumulate_score_tick(&mut self, config: &crate::config::AppConfig) {
        match self.state.state {
            DeskState::Sitting => {
                self.state.daily_score += config.pts_sitting_per_min / 60.0;
            }
            DeskState::Standing => {
                self.state.daily_score += config.pts_standing_per_min / 60.0;
                self.state.standing_session_secs += 1;

                // Lap bonus: award when standing_session_secs crosses a target multiple.
                let target_secs = config.standing_target_mins as i64 * 60;
                if target_secs > 0 {
                    let current_lap = self.state.standing_session_secs / target_secs;
                    if current_lap > self.state.lap_bonus_awarded_for_lap as i64 {
                        self.state.daily_score += config.pts_session_bonus;
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
