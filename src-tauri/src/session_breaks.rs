//! session_breaks.rs — Break credit, state exit, and notification logic.
//!
//! # "Break" means three different things
//!
//! They are deliberately kept apart: they measure different quantities, fire on
//! different clocks, and one is not derivable from the others. Log lines carry a
//! distinct prefix each, so a log read tells you which one acted.
//!
//! | Concept | Lives in | Measures | Log prefix |
//! |---|---|---|---|
//! | Session Break Credit | [`SessionManager::apply_break_credit`] (this file) | a countdown: seconds of break cancel `multiplier ×` seconds of credited sitting (ADR 008) | `[break:credit]` |
//! | Day Break Credit | [`SessionManager::apply_day_break_reset`] (this file) | a one-shot reset: a sleep-length break clears notification flags and `daily_score` (ADR 009) | `[break:day]` |
//! | Hourly break coverage | [`crate::hourly_break_tracker::HourlyBreakTracker`] | a per-clock-hour boolean: did this hour contain ≥5 continuous minutes Away? (KPI only) | `[break:hourly]` |
//!
//! Session and Day credit share an entry point — `apply_break_credit` calls
//! `apply_day_break_reset` — because both are triggered by the same event, a
//! return to sitting. They stay separate functions because the thresholds,
//! the state they touch, and the reasons to change them have nothing in common.
//! Hourly coverage is driven by the per-tick reading loop instead
//! (`session_reading.rs`) and never sees a break-end event at all.

use chrono::DateTime;
use chrono::Utc;

use crate::session_types::*;
use crate::session_manager::SessionManager;

impl SessionManager {
    /// Handles state exit logic, returning any completed session.
    pub(crate) fn handle_state_exit(
        &mut self,
        candidate: &DeskState,
        now: DateTime<Utc>,
    ) -> Option<CompletedSession> {
        // History closes on EVERY confirmed state change. Preserve break time
        // through that same last observed instant before the history lower bound
        // moves forward; otherwise a continuing break across posture changes
        // would lose its already observed prefix. Legacy mixed-break rows cannot
        // be disaggregated here.
        let break_checkpoint = if self.state.state != DeskState::Sitting
            && self.state.break_started.is_some()
        {
            self.known_projection_end(now)
                .map(|_| self.get_live_break_seconds(now))
        } else {
            None
        };
        let completed_session = self.close_observed_boundary(now);
        if let Some(observed_break_secs) = break_checkpoint {
            self.observed_break_secs = observed_break_secs;
            self.state.break_seconds = observed_break_secs;
        }

        match &self.state.state {
            DeskState::Sitting => {
                if let Some(started) = self.state.sitting_started.take() {
                    let elapsed = (now - started).num_seconds().max(0);
                    self.state.sitting_seconds += elapsed;
                    self.state.sitting_seconds_total += elapsed;
                    self.state.last_sitting_secs = elapsed;
                }
                if *candidate != DeskState::Sitting {
                    self.sleep_break_credit = 0;
                    self.observed_break_secs = 0;
                    self.state.break_started = Some(now);
                    self.state.break_seconds = 0;
                    self.alert_fired = false;
                    self.stand_alert_fired = false;
                    self.state.last_position_change_at = Some(now);
                    self.state.last_break_credit = BreakCredit::None;
                    if *candidate == DeskState::Standing {
                        self.state.standing_session_started = Some(now);
                        self.state.standing_bout_started = Some(now);
                    }
                }
            }
            DeskState::Standing => {
                if *candidate != DeskState::Standing {
                    self.state.standing_session_secs = 0;
                    self.state.standing_session_started = None;
                    self.state.lap_bonus_awarded_for_lap = 0;
                    if *candidate == DeskState::Sitting {
                        // Finalize break: apply credit based on total break duration.
                        if self.state.break_started.is_some() {
                            let break_dur = self.get_live_break_seconds(now);
                            let break_started = self.state.break_started.expect("checked above");
                            self.state.break_started = None;
                            self.state.last_break_secs = break_dur;
                            self.apply_current_break_credit(break_dur, break_started, now);
                            self.sleep_break_credit = 0;
                            self.observed_break_secs = 0;
                            self.state.break_seconds = 0;
                        }
                        self.state.sitting_started = Some(now);
                    }
                    // Standing → Walking/Away: break continues, keep break_started.
                    self.state.last_position_change_at = Some(now);
                }
            }
            DeskState::Walking | DeskState::Away => {
                if *candidate == DeskState::Sitting {
                    // Standing time already committed on Standing→Away transition.
                    // Only apply break credit based on total break duration.
                    if self.state.break_started.is_some() {
                        let break_dur = self.get_live_break_seconds(now);
                        let break_started = self.state.break_started.expect("checked above");
                        self.state.break_started = None;
                        self.state.last_break_secs = break_dur;
                        self.apply_current_break_credit(break_dur, break_started, now);
                        self.sleep_break_credit = 0;
                        self.observed_break_secs = 0;
                        self.state.break_seconds = 0;
                    }
                    if self.state.away_bout_secs >= 300 {
                        self.state.continuous_computer_secs = 0;
                    }
                    self.state.away_bout_secs = 0;
                    self.state.sitting_started = Some(now);
                    self.state.last_position_change_at = Some(now);
                }
                // Away/Walking → Standing: start standing bout and break timer
                if *candidate == DeskState::Standing {
                    if self.state.away_bout_secs >= 300 {
                        self.state.continuous_computer_secs = 0;
                    }
                    self.state.away_bout_secs = 0;
                    self.state.standing_bout_started = Some(now);
                    self.state.standing_session_started = Some(now);
                    if self.state.break_started.is_none() {
                        self.sleep_break_credit = 0;
                        self.observed_break_secs = 0;
                        self.state.break_started = Some(now);
                        self.state.break_seconds = 0;
                    }
                    self.state.last_position_change_at = Some(now);
                }
            }
        }

        completed_session
    }

    /// Applies Session Break Credit when returning to sitting (ADR 008).
    ///
    /// Each second of break cancels `break_credit_multiplier` seconds of sitting.
    /// E.g., with multiplier 2.0: 10 min break cancels 20 min sitting.
    /// Breaks under `break_min_secs` get no credit.
    ///
    /// Thresholds are read from [`SessionManager::limits`] — the live ergonomic
    /// profile, refreshed every tick — not from `SessionState`, so a profile
    /// edited on disk applies to the next credit without an app restart.
    ///
    /// A break long enough also triggers the separate Day Break Credit reset,
    /// delegated to [`SessionManager::apply_day_break_reset`]. A break too short
    /// to earn session credit returns before that call — it cannot reach the
    /// far higher day threshold anyway. See this module's header for how both
    /// relate to [`crate::hourly_break_tracker::HourlyBreakTracker`].
    pub fn apply_break_credit(&mut self, break_secs: i64) {
        self.apply_break_credit_after(break_secs, 0, None);
    }

    /// Applies only the additional credit beyond the already-applied prefix.
    /// Captures this application with its own rule and source timestamps.
    pub(crate) fn apply_current_break_credit(
        &mut self,
        break_secs: i64,
        break_started_at: DateTime<Utc>,
        applied_at: DateTime<Utc>,
    ) {
        let before = self.state.sitting_seconds;
        self.apply_break_credit_after(
            break_secs,
            self.sleep_break_credit,
            Some((break_started_at, applied_at)),
        );
        self.sleep_break_credit += before - self.state.sitting_seconds;
    }

    fn apply_break_credit_after(
        &mut self,
        break_secs: i64,
        already_applied: i64,
        source_times: Option<(DateTime<Utc>, DateTime<Utc>)>,
    ) {
        if source_times.is_none() {
            // A newer application without producer timestamps supersedes older
            // explanation data; never leave a stale, plausible-looking record.
            self.state.last_break_credit_facts = None;
        }
        let min_secs = self.limits.break_min_secs as i64;
        let multiplier = self.limits.break_credit_multiplier.clamp(0.0, 10.0);
        let observed_break_secs = break_secs.max(0);
        let already_applied = already_applied.max(0);
        let current_before = self.state.sitting_seconds;
        if break_secs < min_secs {
            self.state.last_break_credit = BreakCredit::None;
            if let Some((break_started_at, applied_at)) = source_times {
                self.record_break_credit_application(crate::session_types::LastBreakCreditFacts {
                    break_started_at: break_started_at.to_rfc3339(),
                    observed_break_secs,
                    minimum_break_secs: min_secs,
                    multiplier,
                    requested_credit_secs: 0,
                    prior_applied_credit_secs: already_applied,
                    sitting_before_secs: current_before,
                    applied_credit_secs: 0,
                    sitting_after_secs: current_before,
                    applied_at: applied_at.to_rfc3339(),
                    no_credit_reason: Some("break_below_minimum".to_string()),
                });
            }
            log::debug!(
                "[break:credit] {}s break < {}s minimum — no credit",
                break_secs, min_secs
            );
            return;
        }
        let credit = (observed_break_secs as f64 * multiplier as f64) as i64;
        let requested_now = (credit - already_applied).max(0);
        self.state.sitting_seconds = (current_before - requested_now).max(0);
        let applied_credit = current_before.saturating_sub(self.state.sitting_seconds);
        let no_credit_reason = if applied_credit > 0 {
            None
        } else if current_before == 0 && requested_now > 0 {
            Some("balance_already_zero".to_string())
        } else if already_applied > 0 && requested_now == 0 {
            Some("already_credited_prefix".to_string())
        } else if multiplier == 0.0 {
            Some("zero_multiplier".to_string())
        } else if credit == 0 {
            Some("requested_credit_rounded_to_zero".to_string())
        } else {
            None
        };
        if let Some((break_started_at, applied_at)) = source_times {
            self.record_break_credit_application(crate::session_types::LastBreakCreditFacts {
                break_started_at: break_started_at.to_rfc3339(),
                observed_break_secs,
                minimum_break_secs: min_secs,
                multiplier,
                requested_credit_secs: requested_now,
                prior_applied_credit_secs: already_applied,
                sitting_before_secs: current_before,
                applied_credit_secs: applied_credit,
                sitting_after_secs: self.state.sitting_seconds,
                applied_at: applied_at.to_rfc3339(),
                no_credit_reason,
            });
        }
        if self.state.sitting_seconds == 0 {
            self.state.last_break_credit = BreakCredit::Full;
        } else {
            self.state.last_break_credit = BreakCredit::Partial;
        }
        log::info!(
            "[break:credit] {}s observed break x{} = {}s target; {}s already applied; applied {}s now — sitting {}s -> {}s ({:?})",
            break_secs, multiplier, credit, already_applied, applied_credit, current_before, self.state.sitting_seconds,
            self.state.last_break_credit
        );

        self.apply_day_break_reset(break_secs);
    }

    fn record_break_credit_application(&mut self, fact: crate::session_types::LastBreakCreditFacts) {
        let applications = self.state.last_break_credit_facts.get_or_insert_with(Vec::new);
        if applications.last().map(|previous| previous.break_started_at.as_str())
            == Some(fact.break_started_at.as_str()) {
            applications.push(fact);
        } else {
            *applications = vec![fact];
        }
    }

    /// Applies Day Break Credit: a sleep-length break clears the motivational
    /// slate (ADR 009).
    ///
    /// This is NOT the countdown of [`SessionManager::apply_break_credit`] — it
    /// subtracts no seconds. It resets the four notification flags and
    /// `daily_score`, so a user returning after a night gets a fresh start
    /// instead of yesterday's exhausted reminders. Daily KPI counters
    /// (`sitting_seconds_total`, `standing_seconds`, `position_changes`, the
    /// hourly-coverage pair) are deliberately preserved: they describe the
    /// calendar day and are reset only by the daily rollover in
    /// `session_daily.rs`.
    ///
    /// Sets the one-shot `day_break_applied` flag, drained by
    /// `serial_periodic_reset::log_break_credit` to write its own event-log line.
    ///
    /// A `day_break_min_secs` of 0 disables the feature entirely.
    pub fn apply_day_break_reset(&mut self, break_secs: i64) {
        let day_min = self.limits.day_break_min_secs as i64;
        if day_min == 0 || break_secs < day_min {
            return;
        }
        log::info!(
            "[break:day] {}s break >= {}s threshold — resetting notification flags and score",
            break_secs, day_min
        );
        self.notify_inactivity_fired = false;
        self.notify_posture_balance_fired = false;
        self.praise_halfway_fired_today = false;
        self.standing_target_reached_fired = false;
        self.state.daily_score = 0.0;
        self.day_break_applied = true;
    }

    /// Checks notification conditions using the system clock.
    ///
    /// Impure boundary; the logic lives in
    /// [`SessionManager::check_notification_conditions_at`].
    pub fn check_notification_conditions(
        &mut self,
        comm: &crate::communication_profile::CommunicationProfile,
    ) -> Vec<NotificationEvent> {
        self.check_notification_conditions_at(comm, Utc::now())
    }

    /// Checks notification conditions at `now` and returns events that should fire.
    /// Suppresses all notifications when user is Away (no point nagging an empty desk).
    pub fn check_notification_conditions_at(
        &mut self,
        comm: &crate::communication_profile::CommunicationProfile,
        now: DateTime<Utc>,
    ) -> Vec<NotificationEvent> {
        if self.state.state == DeskState::Away || self.state.state == DeskState::Walking {
            return Vec::new();
        }
        let pn = &comm.periodic_notifications;
        let mut events = Vec::new();
        if pn.inactivity_enabled && !self.notify_inactivity_fired {
            if let Some(last_change) = self.state.last_position_change_at {
                if (now - last_change).num_seconds() >= 60 * 60 {
                    self.notify_inactivity_fired = true;
                    events.push(NotificationEvent::Inactivity);
                }
            }
        }
        if pn.posture_balance_enabled
            && !self.notify_posture_balance_fired
        {
            // Both sides raw (E015-T03). `sitting_seconds` is credited: a user
            // who takes breaks drives it down, so comparing it against raw
            // standing time made the notification unreachable for exactly the
            // people whose balance it reports.
            if self.state.sitting_seconds_total
                >= self.limits.posture_balance_min_sitting_secs as i64
                && self.state.sitting_seconds_total > self.state.standing_seconds * 2
            {
                self.notify_posture_balance_fired = true;
                events.push(NotificationEvent::PostureBalance);
            }
        }
        if !self.standing_target_reached_fired
            && self.state.stand_limit_secs > 0
            && self.state.standing_seconds >= self.state.stand_limit_secs
        {
            self.standing_target_reached_fired = true;
            events.push(NotificationEvent::StandingTargetReached);
        }
        events
    }

    /// Checks if praise-halfway notification should fire.
    pub fn should_send_praise_halfway(
        &mut self,
        comm: &crate::communication_profile::CommunicationProfile,
    ) -> bool {
        if comm.periodic_notifications.praise_halfway_enabled
            && !self.praise_halfway_fired_today
            && self.state.stand_limit_secs > 0
            && self.state.standing_seconds >= self.state.stand_limit_secs / 2
        {
            self.praise_halfway_fired_today = true;
            return true;
        }
        false
    }

    /// Checks if a stand limit alert should fire.
    pub fn should_stand_alert(&mut self) -> bool {
        if self.state.state == DeskState::Standing
            && !self.stand_alert_fired
            && self.state.stand_limit_secs > 0
            && self.state.break_seconds >= self.state.stand_limit_secs
        {
            self.stand_alert_fired = true;
            return true;
        }
        false
    }

    /// Computes known break seconds, excluding observation gaps.
    pub(crate) fn get_live_break_seconds(&self, now: DateTime<Utc>) -> i64 {
        if self.state.state != DeskState::Sitting {
            if let Some(started) = self.state.break_started {
                return self.observed_break_secs + self.known_elapsed_since(started, now);
            }
        }
        self.state.break_seconds.max(self.observed_break_secs)
    }
}
