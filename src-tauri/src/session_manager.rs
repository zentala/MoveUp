use chrono::{DateTime, NaiveDate, Utc};
use log::info;

use crate::communication_policy::PolicyInput;
use crate::ergonomic_profile::{ErgonomicProfile, Limits};
use crate::height_stabilizer::HeightStabilizer;
use crate::hourly_break_tracker::HourlyBreakTracker;
use crate::session_types::*;

/// Owns `SessionState` and drives state transitions.
pub struct SessionManager {
    pub state: SessionState,
    /// Ergonomic limits currently in force — configuration, never state.
    ///
    /// Every engine function that needs a threshold reads it from here rather
    /// than from [`SessionState`], and adapters refresh it from the active
    /// profile on each tick ([`SessionManager::set_limits`]). Before E020-T02
    /// these values were copied into `SessionState` at construction and never
    /// re-read, so editing an ergonomic profile on disk changed nothing until
    /// the app restarted.
    pub limits: Limits,
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
    /// Set to `true` by `apply_break_credit` when a day-level break credit fires.
    /// Consumed by the periodic tick to log the event, then cleared.
    pub(crate) day_break_applied: bool,
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
            // Every other field starts at its `Default` — an Away manager with
            // zeroed counters. Only the limits differ from that.
            state: SessionState {
                session_limit_secs: DEFAULT_SESSION_LIMIT_SECS,
                ..SessionState::default()
            },
            limits: Limits::default(),
            pending_state: None,
            pending_count: 0,
            sitting_height_cm: 72.0,
            standing_height_cm: 105.0,
            desk_thickness_cm: 3.0,
            alert_fired: false,
            stand_alert_fired: false,
            last_reset_date: crate::session_daily::local_date_of(now),
            last_reset_check: now,
            notify_inactivity_fired: false,
            notify_posture_balance_fired: false,
            praise_halfway_fired_today: false,
            standing_target_reached_fired: false,
            day_break_applied: false,
            height_stabilizer: HeightStabilizer::new(),
            last_accumulate_ran: false,
            hourly_break_tracker: HourlyBreakTracker::new(),
        }
    }

    /// Creates a manager initialized from AppConfig (calibration) and ErgonomicProfile (limits).
    pub fn new_from_config(
        config: &crate::config::AppConfig,
        ergo: &crate::ergonomic_profile::ErgonomicProfile,
    ) -> Self {
        let now = Utc::now();
        Self {
            state: SessionState {
                session_limit_secs: ergo.limits.sitting_secs as i64,
                stand_limit_secs: ergo.limits.standing_target_secs as i64,
                ..SessionState::default()
            },
            limits: ergo.limits.clone(),
            pending_state: None,
            pending_count: 0,
            sitting_height_cm: config.sitting_mm as f32 / 10.0,
            standing_height_cm: config.standing_mm as f32 / 10.0,
            desk_thickness_cm: config.desk_thickness_mm as f32 / 10.0,
            alert_fired: false,
            stand_alert_fired: false,
            last_reset_date: crate::session_daily::local_date_of(now),
            last_reset_check: now,
            notify_inactivity_fired: false,
            notify_posture_balance_fired: false,
            praise_halfway_fired_today: false,
            standing_target_reached_fired: false,
            day_break_applied: false,
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
        info!(
            "seeded today totals: sitting={}s standing={}s changes={}",
            totals.sitting_secs, totals.standing_secs, totals.position_changes
        );
    }

    /// Replaces the ergonomic limits in force, taking effect on the next tick.
    ///
    /// Adapters call this with the profile they already loaded for the tick
    /// (`serial_periodic.rs`) or right after a hot-reload
    /// (`profile_reload.rs`), so a profile edited on disk changes break credit,
    /// PostureBalance and the computer-time reset without an app restart.
    /// `session_limit_secs`/`stand_limit_secs` are deliberately untouched — the
    /// user can override those at runtime through settings.
    pub fn set_limits(&mut self, limits: &Limits) {
        self.limits = limits.clone();
    }

    /// Replaces the ergonomic limits from a whole profile.
    pub fn set_ergo_profile(&mut self, ergo: &ErgonomicProfile) {
        self.set_limits(&ergo.limits);
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
            secs_since_last_break: self.get_secs_since_last_break(now),
            continuous_computer_secs: self.state.continuous_computer_secs,
            longest_computer_session_secs: self.state.longest_computer_session_secs,
            sitting_seconds_total: self.get_live_sitting_seconds_total(now),
            idle_secs: self.state.idle_secs,
            away_bout_secs: self.state.away_bout_secs,
            max_continuous_computer_secs: self.limits.max_continuous_computer_secs as i64,
        }
    }

    /// Builds this tick's [`PolicyInput`] for the communication policy.
    ///
    /// The engine owns every derived value here (E020-T05). Until then
    /// `tray_controller` re-derived `elapsed_secs` and the standing-lap trio
    /// from snapshot fields, so a second consumer could silently disagree with
    /// the first. Adapters now pass in only what the engine cannot know:
    /// whether the sensor is connected.
    pub fn policy_input(&self, now: DateTime<Utc>, sensor_connected: bool) -> PolicyInput {
        let target = self.state.stand_limit_secs;
        let lapping = self.state.state == DeskState::Standing && target > 0;
        let bout = self.get_live_break_seconds(now);
        PolicyInput {
            state: self.state.state.clone(),
            elapsed_secs: match self.state.state {
                DeskState::Sitting => self.get_live_sitting_seconds(now),
                DeskState::Standing => bout,
                _ => 0,
            },
            sensor_connected,
            standing_lap_progress: if lapping {
                (bout % target) as f32 / target as f32
            } else {
                0.0
            },
            standing_lap: if lapping {
                (self.get_live_standing_seconds(now) / target) as u32
            } else {
                0
            },
            standing_lap_flash: lapping && bout / target > 0,
            continuous_computer_secs: self.state.continuous_computer_secs,
        }
    }

    fn compute_limit_used(&self, now: chrono::DateTime<Utc>) -> i64 {
        self.get_live_sitting_seconds(now)
    }

    /// Returns the current desk state.
    pub fn current_state(&self) -> DeskState {
        self.state.state.clone()
    }
}
