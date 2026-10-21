//! session_daily.rs — Daily reset, score accumulation, and alert logic.

use chrono::{DateTime, Local, LocalResult, NaiveDate, TimeZone, Utc};
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

/// Resolves the first valid local instant of a calendar day without changing
/// the process timezone. The result is passed into the pure reset operation.
pub(crate) fn local_day_start_utc<Tz: TimeZone>(tz: &Tz, day: NaiveDate) -> Option<DateTime<Utc>> {
    // Some zones advance their clocks at midnight. Search wall-clock seconds
    // until this date's first representable instant instead of substituting
    // the later time at which the periodic reset happened to run.
    for seconds in 0..86_400 {
        let local = day.and_hms_opt(0, 0, 0)? + chrono::Duration::seconds(seconds);
        match tz.from_local_datetime(&local) {
            LocalResult::Single(value) => return Some(value.with_timezone(&Utc)),
            LocalResult::Ambiguous(first, second) => {
                return Some(first.min(second).with_timezone(&Utc));
            }
            LocalResult::None => {}
        }
    }
    None
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
        let today = local_date_of(now);
        let Some(boundary) = local_day_start_utc(&Local, today) else {
            log::warn!("daily reset deferred: local date {today} has no valid instant");
            return false;
        };
        self.check_daily_reset_at_boundary(now, today, boundary)
    }

    /// Resets daily counters when `today_local` is past `last_reset_date`
    /// (throttled to one check per 60s of `now`).
    ///
    /// `today_local` is the current **local** calendar day, passed in rather
    /// than derived here (D3) so callers replaying a fixed timeline — and the
    /// DST test — control both the instant and the day boundary.
    pub fn check_daily_reset_at(&mut self, now: DateTime<Utc>, today_local: NaiveDate) -> bool {
        self.check_daily_reset_at_boundary(now, today_local, now)
    }

    /// Pure reset operation using the explicit instant at the start of the
    /// local day. Tests can supply DST-aware instants without changing TZ.
    pub(crate) fn check_daily_reset_at_boundary(
        &mut self,
        now: DateTime<Utc>,
        today_local: NaiveDate,
        day_boundary: DateTime<Utc>,
    ) -> bool {
        self.check_daily_reset_at_boundary_inner(now, today_local, day_boundary, false)
    }

    /// Applies a changed-day rollover immediately before processing `now` as
    /// a sensor reading. That reading extends known coverage only when the
    /// previous observation is within the normal gap threshold.
    pub(crate) fn check_daily_reset_at_observation(
        &mut self,
        now: DateTime<Utc>,
        today_local: NaiveDate,
        day_boundary: DateTime<Utc>,
    ) -> bool {
        if self.last_reset_date >= today_local {
            return false;
        }
        let last_observation = self.history_last_reading.into_iter()
            .chain(self.state.last_tick_ts).max();
        if last_observation.is_some_and(|last| now < last) {
            return false;
        }
        if last_observation.is_some_and(|last| {
            (now - last).num_seconds() <= SLEEP_GAP_THRESHOLD_SECS
        }) {
            self.history_last_reading = Some(now);
        }
        self.check_daily_reset_at_boundary_inner(now, today_local, day_boundary, true)
    }

    fn check_daily_reset_at_boundary_inner(
        &mut self,
        now: DateTime<Utc>,
        today_local: NaiveDate,
        day_boundary: DateTime<Utc>,
        changed_day_bypasses_throttle: bool,
    ) -> bool {
        let today = today_local;
        if !changed_day_bypasses_throttle && (now - self.last_reset_check).num_seconds() < 60 {
            return false;
        }
        self.last_reset_check = now;
        if self.last_reset_date < today {
            info!("daily reset: new day detected, resetting in-memory counters");
            let boundary = day_boundary.min(now);
            let last_observation = self.history_last_reading.or(self.state.last_tick_ts);
            let active_state = self.current_state();
            if let Some(span) = self.close_observed_boundary(boundary) {
                self.queue_history(active_state, span, None);
            }
            self.sleep_break_credit = 0;
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
            if last_observation.is_some_and(|last| last >= boundary) {
                let current_span_start = self.history_span_started
                    .filter(|started| *started > boundary)
                    .unwrap_or(boundary);
                self.open_observed_boundary(current_span_start);
            } else {
                self.history_span_started = None;
            }
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
