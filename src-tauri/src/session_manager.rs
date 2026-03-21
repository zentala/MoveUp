//! session_manager.rs — SessionManager struct and state machine logic.
//!
//! Owns `SessionState` and drives sit/stand transitions, break credits,
//! alert firing, and notification conditions.

use chrono::{DateTime, NaiveDate, Utc};
use log::info;

use crate::session_types::*;

// ─── SessionManager ──────────────────────────────────────────────────────────

/// Owns `SessionState` and drives state transitions.
pub struct SessionManager {
    pub state: SessionState,
    /// Pending candidate state (needs `debounce_count` confirmations).
    pub(crate) pending_state: Option<DeskState>,
    pub(crate) pending_count: u8,
    /// Calibrated sitting desk height in cm (default 72.0).
    pub sitting_height_cm: f32,
    /// Calibrated standing desk height in cm (default 105.0).
    pub standing_height_cm: f32,
    /// Desk surface thickness in cm to subtract from raw sensor reading.
    pub desk_thickness_cm: f32,
    /// Whether `should_alert()` has been armed for the current sitting session.
    pub(crate) alert_fired: bool,
    /// Whether a standing alert has been armed for the current standing stint.
    pub(crate) stand_alert_fired: bool,
    /// Last date when daily reset was performed (T009).
    pub last_reset_date: NaiveDate,
    /// Last time when check_daily_reset() was called (T009).
    pub last_reset_check: DateTime<Utc>,
    /// Notification debounce flag: inactivity alert fired today (T003).
    pub notify_inactivity_fired: bool,
    /// Notification debounce flag: posture balance alert fired today (T003).
    pub notify_posture_balance_fired: bool,
    /// Notification debounce flag: praise message fired today (T003).
    pub praise_halfway_fired_today: bool,
}

impl SessionManager {
    /// Creates a manager with default calibration values.
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            state: SessionState {
                state: DeskState::Away,
                sitting_started: None,
                sitting_seconds: 0,
                standing_seconds: 0,
                break_started: None,
                break_seconds: 0,
                session_limit_secs: DEFAULT_SESSION_LIMIT_SECS,
                stand_limit_secs: 0,
                desk_height_cm: 0.0,
                last_position_change_at: None,
                position_changes: 0,
                last_break_secs: 0,
                last_sitting_secs: 0,
                last_break_credit: BreakCredit::None,
            },
            pending_state: None,
            pending_count: 0,
            sitting_height_cm: 72.0,
            standing_height_cm: 105.0,
            desk_thickness_cm: 3.0,
            alert_fired: false,
            stand_alert_fired: false,
            last_reset_date: now.date_naive(),
            last_reset_check: now,
            notify_inactivity_fired: false,
            notify_posture_balance_fired: false,
            praise_halfway_fired_today: false,
        }
    }

    /// Creates a manager initialized from AppConfig.
    pub fn new_from_config(config: &crate::config::AppConfig) -> Self {
        let now = Utc::now();
        Self {
            state: SessionState {
                state: DeskState::Away,
                sitting_started: None,
                sitting_seconds: 0,
                standing_seconds: 0,
                break_started: None,
                break_seconds: 0,
                session_limit_secs: config.sit_limit_mins as i64 * 60,
                stand_limit_secs: config.standing_target_mins as i64 * 60,
                desk_height_cm: 0.0,
                last_position_change_at: None,
                position_changes: 0,
                last_break_secs: 0,
                last_sitting_secs: 0,
                last_break_credit: BreakCredit::None,
            },
            pending_state: None,
            pending_count: 0,
            sitting_height_cm: config.sitting_mm as f32 / 10.0,
            standing_height_cm: config.standing_mm as f32 / 10.0,
            desk_thickness_cm: config.desk_thickness_mm as f32 / 10.0,
            alert_fired: false,
            stand_alert_fired: false,
            last_reset_date: now.date_naive(),
            last_reset_check: now,
            notify_inactivity_fired: false,
            notify_posture_balance_fired: false,
            praise_halfway_fired_today: false,
        }
    }

    /// Seeds today's totals from SQLite so in-memory counters survive restarts.
    pub fn load_today_totals(&mut self, sitting_secs: i64, standing_secs: i64) {
        self.state.sitting_seconds = sitting_secs;
        self.state.standing_seconds = standing_secs;
        info!(
            "seeded today totals: sitting={}s standing={}s",
            sitting_secs, standing_secs
        );
    }

    /// Updates the sitting session limit (minutes -> seconds).
    pub fn set_limit_minutes(&mut self, minutes: u32) {
        self.state.session_limit_secs = minutes as i64 * 60;
    }

    /// Updates the standing target (minutes -> seconds).
    pub fn set_stand_limit_minutes(&mut self, minutes: u32) {
        self.state.stand_limit_secs = minutes as i64 * 60;
    }

    /// Returns a snapshot of the current session state as a DTO.
    pub fn snapshot(&self) -> SessionStateDto {
        let now = Utc::now();
        let live_sitting = self.get_live_sitting_seconds(now);
        SessionStateDto {
            state: self.state.state.clone(),
            sitting_seconds: live_sitting,
            standing_seconds: self.state.standing_seconds,
            break_seconds: self.state.break_seconds,
            session_limit_secs: self.state.session_limit_secs,
            stand_limit_secs: self.state.stand_limit_secs,
            desk_height_cm: self.state.desk_height_cm,
            position_changes: self.state.position_changes,
            limit_used_secs: self.compute_limit_used(now),
        }
    }

    /// Compute how many seconds of sitting limit have been consumed.
    /// Break credit is already subtracted from `sitting_seconds` in `apply_break_credit()`,
    /// so limit_used = live sitting seconds.
    fn compute_limit_used(&self, now: chrono::DateTime<Utc>) -> i64 {
        self.get_live_sitting_seconds(now)
    }

    /// Computes live sitting seconds: committed + elapsed since sitting_started.
    pub(crate) fn get_live_sitting_seconds(&self, now: DateTime<Utc>) -> i64 {
        if self.state.state == DeskState::Sitting {
            if let Some(started) = self.state.sitting_started {
                let elapsed = (now - started).num_seconds().max(0);
                return self.state.sitting_seconds + elapsed;
            }
        }
        self.state.sitting_seconds
    }

    /// Returns the current desk state.
    pub fn current_state(&self) -> DeskState {
        self.state.state.clone()
    }

    /// Checks if a new day has begun and resets daily counters.
    /// Only checks every 60 seconds to avoid overhead.
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
            self.last_reset_date = today;
            return true;
        }
        false
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

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
