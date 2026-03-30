//! session_breaks.rs — Break credit, state exit, and notification logic.

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
        let mut completed_session = None;

        match &self.state.state {
            DeskState::Sitting => {
                if let Some(started) = self.state.sitting_started.take() {
                    let elapsed = (now - started).num_seconds().max(0);
                    self.state.sitting_seconds += elapsed;
                    self.state.sitting_seconds_total += elapsed;
                    self.state.last_sitting_secs = elapsed;
                    if *candidate != DeskState::Sitting {
                        completed_session = Some(CompletedSession {
                            started_at: started.to_rfc3339(),
                            ended_at: now.to_rfc3339(),
                            duration_secs: elapsed,
                        });
                    }
                }
                if *candidate != DeskState::Sitting {
                    self.state.current_session_secs = 0;
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
                    // Commit actual standing time from this bout.
                    if let Some(sb) = self.state.standing_bout_started.take() {
                        let standing_dur = (now - sb).num_seconds().max(0);
                        self.state.standing_seconds += standing_dur;
                    }
                    if *candidate == DeskState::Sitting {
                        // Finalize break: apply credit based on total break duration.
                        if let Some(bs) = self.state.break_started.take() {
                            let break_dur = (now - bs).num_seconds().max(0);
                            self.state.last_break_secs = break_dur;
                            self.apply_break_credit(break_dur);
                            self.state.current_session_secs = 0;
                            self.state.break_seconds = 0;
                            completed_session = Some(CompletedSession {
                                started_at: bs.to_rfc3339(),
                                ended_at: now.to_rfc3339(),
                                duration_secs: break_dur,
                            });
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
                    if let Some(bs) = self.state.break_started.take() {
                        let break_dur = (now - bs).num_seconds().max(0);
                        self.state.last_break_secs = break_dur;
                        self.apply_break_credit(break_dur);
                        self.state.current_session_secs = 0;
                        self.state.break_seconds = 0;
                        completed_session = Some(CompletedSession {
                            started_at: bs.to_rfc3339(),
                            ended_at: now.to_rfc3339(),
                            duration_secs: break_dur,
                        });
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
                        self.state.break_started = Some(now);
                        self.state.break_seconds = 0;
                    }
                    self.state.last_position_change_at = Some(now);
                }
            }
        }

        completed_session
    }

    /// Applies proportional break credit when returning to sitting.
    ///
    /// Each second of break cancels `break_credit_multiplier` seconds of sitting.
    /// E.g., with multiplier 2.0: 10 min break cancels 20 min sitting.
    /// Breaks under `break_min_secs` get no credit.
    /// Parameters come from the ergonomic profile (configurable per profile).
    pub fn apply_break_credit(&mut self, break_secs: i64) {
        let min_secs = self.state.break_min_secs;
        let multiplier = self.state.break_credit_multiplier;
        if break_secs < min_secs {
            self.state.last_break_credit = BreakCredit::None;
            return;
        }
        let credit = (break_secs as f64 * multiplier as f64) as i64;
        let before = self.state.sitting_seconds;
        self.state.sitting_seconds = (before - credit).max(0);
        if self.state.sitting_seconds == 0 {
            self.state.last_break_credit = BreakCredit::Full;
        } else {
            self.state.last_break_credit = BreakCredit::Partial;
        }
    }

    /// Checks notification conditions and returns events that should fire.
    /// Suppresses all notifications when user is Away (no point nagging an empty desk).
    pub fn check_notification_conditions(
        &mut self,
        comm: &crate::communication_profile::CommunicationProfile,
    ) -> Vec<NotificationEvent> {
        if self.state.state == DeskState::Away || self.state.state == DeskState::Walking {
            return Vec::new();
        }
        let pn = &comm.periodic_notifications;
        let now = Utc::now();
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
            if self.state.sitting_seconds > self.state.standing_seconds * 2 {
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

    /// Computes live break seconds: elapsed since break_started.
    pub(crate) fn get_live_break_seconds(&self, now: DateTime<Utc>) -> i64 {
        if self.state.state != DeskState::Sitting {
            if let Some(started) = self.state.break_started {
                return (now - started).num_seconds().max(0);
            }
        }
        self.state.break_seconds
    }
}
