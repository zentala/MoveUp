use chrono::{DateTime, NaiveDate, Utc};
use log::info;

use crate::height_stabilizer::HeightStabilizer;
use crate::hourly_break_tracker::HourlyBreakTracker;
use crate::session_types::*;

/// Owns `SessionState` and drives state transitions.
pub struct SessionManager {
    pub state: SessionState,
    pub(crate) pending_state: Option<DeskState>,
    pub(crate) pending_count: u8,
    pub sitting_height_cm: f32,
    pub standing_height_cm: f32,
    pub desk_thickness_cm: f32,
    pub(crate) alert_fired: bool,
    pub(crate) stand_alert_fired: bool,
    pub last_reset_date: NaiveDate,
    pub last_reset_check: DateTime<Utc>,
    pub notify_inactivity_fired: bool,
    pub notify_posture_balance_fired: bool,
    pub praise_halfway_fired_today: bool,
    pub standing_target_reached_fired: bool,
    /// Smooths raw sensor readings for stable UI display.
    pub(crate) height_stabilizer: HeightStabilizer,
    /// Set to `true` by `accumulate_ongoing` when it actually executes (not throttled).
    /// Callers use this to gate per-second work like `accumulate_score_tick`.
    pub(crate) last_accumulate_ran: bool,
    /// Tracks per-clock-hour Away breaks for KPI metric.
    pub(crate) hourly_break_tracker: HourlyBreakTracker,
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
                daily_score: 0.0,
                standing_session_secs: 0,
                standing_session_started: None,
                lap_bonus_awarded_for_lap: 0,
                current_session_secs: 0,
                continuous_computer_secs: 0,
                longest_computer_session_secs: 0,
                away_bout_secs: 0,
                first_reading_at: None,
                last_tick_ts: None,
                last_accumulate_ts: None,
                standing_bout_started: None,
                hourly_breaks_covered: 0,
                hourly_breaks_active: 0,
                sitting_seconds_total: 0,
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
            standing_target_reached_fired: false,
            height_stabilizer: HeightStabilizer::new(),
            last_accumulate_ran: false,
            hourly_break_tracker: HourlyBreakTracker::new(),
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
                daily_score: 0.0,
                standing_session_secs: 0,
                standing_session_started: None,
                lap_bonus_awarded_for_lap: 0,
                current_session_secs: 0,
                continuous_computer_secs: 0,
                longest_computer_session_secs: 0,
                away_bout_secs: 0,
                first_reading_at: None,
                last_tick_ts: None,
                last_accumulate_ts: None,
                standing_bout_started: None,
                hourly_breaks_covered: 0,
                hourly_breaks_active: 0,
                sitting_seconds_total: 0,
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
            standing_target_reached_fired: false,
            height_stabilizer: HeightStabilizer::new(),
            last_accumulate_ran: false,
            hourly_break_tracker: HourlyBreakTracker::new(),
        }
    }

    /// Seeds today's totals from SQLite (in-memory counters survive restarts).
    pub fn load_today_totals(&mut self, totals: &crate::db_sessions::TodayTotals) {
        self.state.sitting_seconds = totals.sitting_secs;
        self.state.sitting_seconds_total = totals.sitting_secs;
        self.state.standing_seconds = totals.standing_secs;
        self.state.position_changes = totals.position_changes;
        // current_session_secs stays 0: no active session after restart.
        self.state.current_session_secs = 0;
        info!(
            "seeded today totals: sitting={}s standing={}s changes={}",
            totals.sitting_secs, totals.standing_secs, totals.position_changes
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
        let live_current = self.get_live_current_session_secs(now);
        let live_break = self.get_live_break_seconds(now);
        let live_standing = self.get_live_standing_seconds(now);
        SessionStateDto {
            state: self.state.state.clone(),
            sitting_seconds: live_sitting,
            standing_seconds: live_standing,
            break_seconds: live_break,
            session_limit_secs: self.state.session_limit_secs,
            stand_limit_secs: self.state.stand_limit_secs,
            desk_height_cm: self.state.desk_height_cm,
            position_changes: self.state.position_changes,
            limit_used_secs: self.compute_limit_used(now),
            daily_score: self.state.daily_score,
            standing_session_secs: self.get_live_standing_session_secs(now),
            current_session_secs: live_current,
            continuous_computer_secs: self.state.continuous_computer_secs,
            longest_computer_session_secs: self.state.longest_computer_session_secs,
            sitting_seconds_total: self.get_live_sitting_seconds_total(now),
        }
    }

    /// Computes live standing seconds: accumulated + current standing bout elapsed.
    /// Only counts actual Standing time, not Away time.
    pub(crate) fn get_live_standing_seconds(&self, now: DateTime<Utc>) -> i64 {
        let base = self.state.standing_seconds;
        if self.state.state == DeskState::Standing {
            if let Some(started) = self.state.standing_bout_started {
                return base + (now - started).num_seconds().max(0);
            }
        }
        base
    }

    fn compute_limit_used(&self, now: chrono::DateTime<Utc>) -> i64 {
        self.get_live_sitting_seconds(now)
    }
    /// Computes live current session seconds: committed + elapsed since sitting_started.
    pub(crate) fn get_live_current_session_secs(&self, now: DateTime<Utc>) -> i64 {
        if self.state.state == DeskState::Sitting {
            if let Some(started) = self.state.sitting_started {
                let elapsed = (now - started).num_seconds().max(0);
                return self.state.current_session_secs + elapsed;
            }
        }
        self.state.current_session_secs
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
    /// Computes live raw sitting seconds (never reduced by break credit).
    /// Used by standing_pct metric for accurate KPI calculation.
    pub(crate) fn get_live_sitting_seconds_total(&self, now: DateTime<Utc>) -> i64 {
        if self.state.state == DeskState::Sitting {
            if let Some(started) = self.state.sitting_started {
                let elapsed = (now - started).num_seconds().max(0);
                return self.state.sitting_seconds_total + elapsed;
            }
        }
        self.state.sitting_seconds_total
    }
    /// Computes live standing session seconds from timestamp.
    pub(crate) fn get_live_standing_session_secs(&self, now: DateTime<Utc>) -> i64 {
        if self.state.state == DeskState::Standing {
            if let Some(started) = self.state.standing_session_started {
                return (now - started).num_seconds().max(0);
            }
        }
        self.state.standing_session_secs
    }

    /// Returns the current desk state.
    pub fn current_state(&self) -> DeskState {
        self.state.state.clone()
    }
}
