//! CommunicationPolicy — evaluates session state once/sec and returns [`Signals`]
//! for each UI channel. Stateless helpers live in [`communication_policy_helpers`].

use std::cell::Cell;
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::communication_types::{NotifySignal, Signals, ReminderDraft, ReminderReason, ReminderCategory, RetirementReason};
use crate::session_types::{DeskState, NotificationEvent};
use crate::communication_profile::{CommunicationProfile, EscalationStep};
use crate::communication_policy_helpers as helpers;
use crate::ergonomic_profile::ErgonomicProfile;

/// Restart-safe policy state. Live policy continues to use monotonic instants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedNotificationPolicy {
    pub profile_id: String,
    pub saved_at_utc: DateTime<Utc>,
    pub snoozed_until_utc: Option<DateTime<Utc>>,
    pub snooze_index: usize,
    pub last_notify_at_utc: Option<DateTime<Utc>>,
    pub notify_count: u32,
}

/// Snapshot of session data passed to [`CommunicationPolicy::evaluate`] each cycle.
///
/// Built by the engine — `SessionManager::policy_input` — not by whoever calls
/// [`CommunicationPolicy::evaluate`]. Adapters must not derive `elapsed_secs`
/// or the `standing_lap*` trio themselves: two derivations of the same value
/// drift apart without ever failing (E020-T05).
pub struct PolicyInput {
    pub state: DeskState,
    pub elapsed_secs: i64,
    pub sensor_connected: bool,
    /// Freshness is independent of physical serial connection state.
    pub observation_fresh: bool,
    pub standing_lap_progress: f32,
    pub standing_lap: u32,
    pub standing_lap_flash: bool,
    pub continuous_computer_secs: i64,
}

/// Raw session facts used by the single policy owner for one-shot notifications.
/// These are measurements and persisted suppression state, not decisions.
pub struct SessionNotificationFacts {
    pub state: DeskState,
    pub now: chrono::DateTime<chrono::Utc>,
    /// Whether time-derived policy decisions have coherent sensor coverage.
    pub observation_fresh: bool,
    pub last_position_change_at: Option<chrono::DateTime<chrono::Utc>>,
    pub sitting_seconds_total: i64,
    pub standing_seconds: i64,
    pub standing_target_secs: i64,
    pub inactivity_fired: bool,
    pub posture_balance_fired: bool,
    pub standing_target_fired: bool,
    pub praise_fired: bool,
}

/// Stateful policy engine that converts session snapshots into UI signals.
///
/// Call [`evaluate`] once per second. Call [`dismiss`] when the user acknowledges
/// an alert, and [`on_position_changed`] on every sitting↔standing transition.
pub struct CommunicationPolicy {
    comm_profile: CommunicationProfile,
    ergo_profile: ErgonomicProfile,
    snoozed_until: Option<Instant>,
    snooze_index: usize,
    disconnect_notified: bool,
    last_notify_time: Cell<Option<Instant>>,
    notify_count: Cell<u32>,
    screen_break_nudge_fired: bool,
    last_reminder: Option<ReminderDraft>,
    reminder_occurrence: Cell<u64>,
}

impl CommunicationPolicy {
    /// Create a new policy engine with the given profiles.
    pub fn new(comm: CommunicationProfile, ergo: ErgonomicProfile) -> Self {
        Self {
            comm_profile: comm,
            ergo_profile: ergo,
            snoozed_until: None,
            snooze_index: 0,
            disconnect_notified: false,
            last_notify_time: Cell::new(None),
            notify_count: Cell::new(0),
            screen_break_nudge_fired: false,
            last_reminder: None,
            reminder_occurrence: Cell::new(0),
        }
    }

    /// Evaluate current session state and return UI signals (called ~1/sec).
    pub fn evaluate(&mut self, input: &PolicyInput) -> Signals {
        self.evaluate_with_stand_limit_fired(
            input,
            self.ergo_profile.limits.sitting_secs as i64,
            input.elapsed_secs,
            false,
        )
    }

    /// Evaluates signals using the active runtime sitting limit and accepts
    /// the standing maximum through the same threshold decision. The caller
    /// supplies the persisted one-shot flag.
    pub fn evaluate_with_stand_limit_fired(
        &mut self,
        input: &PolicyInput,
        sitting_limit_secs: i64,
        standing_max_elapsed_secs: i64,
        stand_limit_fired: bool,
    ) -> Signals {
        self.evaluate_with_stand_limit_fired_at(
            input,
            sitting_limit_secs,
            standing_max_elapsed_secs,
            stand_limit_fired,
            Instant::now(),
        )
    }

    /// Evaluates with the caller's monotonic instant so reminder timing is deterministic.
    pub fn evaluate_with_stand_limit_fired_at(
        &mut self,
        input: &PolicyInput,
        sitting_limit_secs: i64,
        standing_max_elapsed_secs: i64,
        stand_limit_fired: bool,
        now: Instant,
    ) -> Signals {
        self.last_reminder = None;
        if !input.sensor_connected {
            let (sigs, fired) = helpers::disconnected_signals(&self.comm_profile, self.disconnect_notified);
            if fired {
                self.disconnect_notified = true;
                let body = match &sigs.notify { Some(NotifySignal::Toast(body) | NotifySignal::Popup(body)) => body.clone(), _ => unreachable!("fired disconnect includes a notification") };
                self.last_reminder = Some(ReminderDraft {
                    occurrence: self.next_reminder_occurrence(), reason: ReminderReason::DeviceDisconnected,
                    category: ReminderCategory::Warning, title: "Sensor disconnected".into(), body,
                    elapsed_secs: None, limit_secs: None, position_anchor: None, observation_day: None,
                });
            }
            return sigs;
        }

        if !input.observation_fresh {
            return Signals::default();
        }

        if matches!(input.state, DeskState::Away | DeskState::Walking) {
            return helpers::inactive_signals(&self.comm_profile.inactive);
        }

        // Check snooze — if still active, return baseline only
        if let Some(until) = self.snoozed_until {
            if now < until {
                return self.make_baseline(input, sitting_limit_secs);
            } else {
                self.snoozed_until = None;
            }
        }

        let (limit_secs, steps) = match input.state {
            DeskState::Sitting => (sitting_limit_secs, self.comm_profile.escalation.sitting.clone()),
            DeskState::Standing => (
                self.ergo_profile.limits.standing_max_secs as i64,
                self.comm_profile.escalation.standing.clone(),
            ),
            _ => return self.make_baseline(input, sitting_limit_secs),
        };

        let elapsed_secs = if input.state == DeskState::Standing {
            standing_max_elapsed_secs
        } else {
            input.elapsed_secs
        };
        let offset = elapsed_secs - limit_secs;
        let active_step = Self::active_step_index(&steps, offset).map(|idx| (idx, &steps[idx]));

        let notification_elapsed_secs = if input.state == DeskState::Standing {
            standing_max_elapsed_secs
        } else {
            input.elapsed_secs
        };
        let mut signals = match active_step {
            Some((idx, step)) => self.step_to_signals(
                step,
                steps[..=idx]
                    .iter()
                    .rev()
                    .find_map(|active| active.notify.as_deref()),
                input,
                sitting_limit_secs,
                notification_elapsed_secs,
                now,
            ),
            None => self.make_baseline(input, sitting_limit_secs),
        };

        self.maybe_apply_screen_nudge(input, &mut signals);
        let standing_max = self.ergo_profile.limits.standing_max_secs as i64;
        if input.state == DeskState::Standing
            && standing_max > 0
            && standing_max_elapsed_secs >= standing_max
            && !stand_limit_fired
        {
            signals.accepted_events.push(NotificationEvent::StandLimitReached);
        }
        signals
    }

    /// Selects session-event candidates using thresholds and persisted one-shot
    /// flags. [`resolve_session_notifications`](Self::resolve_session_notifications)
    /// applies the shared sitting-reminder cooldown and finite budget before dispatch.
    pub fn decide_session_notifications(
        &self,
        facts: &SessionNotificationFacts,
        periodic: bool,
    ) -> Vec<NotificationEvent> {
        if !self.comm_profile.notifications_enabled {
            return Vec::new();
        }
        if !facts.observation_fresh {
            return Vec::new();
        }
        if matches!(facts.state, DeskState::Away | DeskState::Walking) {
            return Vec::new();
        }
        let sitting_reminders_exhausted = facts.state == DeskState::Sitting
            && self.notify_count.get() as usize
                >= crate::communication_profile::MAX_NOTIFY_REMINDERS;
        let mut events = Vec::new();
        let candidates = if periodic {
            vec![(NotificationEvent::Inactivity, facts.inactivity_fired || sitting_reminders_exhausted),
                (NotificationEvent::PostureBalance, facts.posture_balance_fired || sitting_reminders_exhausted),
                (NotificationEvent::StandingTargetReached, facts.standing_target_fired)]
        } else if facts.state == DeskState::Standing {
            vec![(NotificationEvent::Praise, facts.praise_fired)]
        } else { Vec::new() };
        for (event, fired) in candidates {
            if !fired && self.session_reason_current(&event, facts) { events.push(event); }
        }
        events
    }

    /// Resolves session-event candidates into executable notifications.
    /// Sitting inactivity and posture-balance events consume the same cooldown
    /// and four-reminder budget as the sitting escalation ladder. Other event
    /// classes keep their own thresholds. NotificationService only logs and
    /// performs the selected side effects.
    pub fn resolve_session_notifications(
        &self,
        facts: &SessionNotificationFacts,
        periodic: bool,
    ) -> Vec<crate::notification_service::NotificationIntent> {
        self.resolve_session_notifications_at(facts, periodic, Instant::now())
    }

    /// Resolves session events on the same monotonic timeline used by
    /// [`evaluate_with_stand_limit_fired_at`](Self::evaluate_with_stand_limit_fired_at).
    pub fn resolve_session_notifications_at(
        &self,
        facts: &SessionNotificationFacts,
        periodic: bool,
        now: Instant,
    ) -> Vec<crate::notification_service::NotificationIntent> {
        let events = self.decide_session_notifications(facts, periodic);
        let events: Vec<_> = events
            .into_iter()
            .filter(|event| {
                if facts.state != DeskState::Sitting
                    || !matches!(event, NotificationEvent::Inactivity | NotificationEvent::PostureBalance)
                {
                    return true;
                }
                if self.is_snoozed_at(now) {
                    return false;
                }
                let Some(cooldown) = self.notify_cooldown_secs() else {
                    return false;
                };
                if self.last_notify_time.get().is_some_and(|last| {
                    now.checked_duration_since(last).unwrap_or_default()
                        < Duration::from_secs(cooldown)
                }) {
                    return false;
                }
                self.last_notify_time.set(Some(now));
                self.notify_count.set(self.notify_count.get() + 1);
                true
            })
            .collect();
        let ordinary_events: Vec<_> = events.iter()
            .filter(|event| !matches!(event, NotificationEvent::StandLimitReached))
            .cloned().collect();
        let mut intents = crate::notification_service::NotificationService::build_intents(
            &ordinary_events, facts.sitting_seconds_total, facts.standing_seconds,
        );
        if events.iter().any(|event| matches!(event, NotificationEvent::StandLimitReached)) {
            intents.push(crate::notification_service::NotificationService::stand_limit_intent());
        }
        crate::notification_service::NotificationService::resolve_channels(
            &mut intents, &self.comm_profile.notification_backend,
        );
        for intent in &mut intents {
            let (reason, category, elapsed, limit) = match intent.event {
                NotificationEvent::Inactivity => (ReminderReason::Inactivity, ReminderCategory::Warning,
                    facts.last_position_change_at.map(|at| (facts.now - at).num_seconds()),
                    Some(i64::from(self.comm_profile.periodic_notifications.inactivity_after_mins) * 60)),
                NotificationEvent::PostureBalance => (ReminderReason::PostureBalance, ReminderCategory::Warning, None, None),
                NotificationEvent::Praise => (ReminderReason::Praise, ReminderCategory::Success,
                    Some(facts.standing_seconds), Some(facts.standing_target_secs)),
                NotificationEvent::StandingTargetReached => (ReminderReason::StandingTarget, ReminderCategory::Success,
                    Some(facts.standing_seconds), Some(facts.standing_target_secs)),
                NotificationEvent::StandLimitReached => (ReminderReason::StandingLimit, ReminderCategory::Urgent, None, None),
            };
            if reason == ReminderReason::Inactivity {
                intent.title = format!("No position change in {} minutes", self.comm_profile.periodic_notifications.inactivity_after_mins);
            }
            intent.reminder = Some(ReminderDraft { occurrence: self.next_reminder_occurrence(), reason, category, title: intent.title.clone(), body: intent.body.clone(),
                elapsed_secs: elapsed, limit_secs: limit,
                observation_day: Some(facts.now.with_timezone(&chrono::Local).date_naive()),
                position_anchor: if reason == ReminderReason::Inactivity {
                    facts.last_position_change_at.map(|at| at.to_rfc3339())
                } else { None },
            });
        }
        intents
    }

    /// Record a user dismiss — start snooze timer, advance index, reset notify count.
    pub fn dismiss(&mut self) {
        self.dismiss_at(Instant::now());
    }

    pub fn dismiss_at(&mut self, now: Instant) {
        let durations = &self.comm_profile.snooze.durations_mins;
        let idx = self.snooze_index.min(durations.len().saturating_sub(1));
        let mins = durations.get(idx).copied().unwrap_or(5);
        self.snoozed_until = Some(now + Duration::from_secs(mins as u64 * 60));
        self.snooze_index = (self.snooze_index + 1).min(durations.len().saturating_sub(1));
        self.last_notify_time.set(None);
        self.notify_count.set(0);
    }

    /// Snooze for an explicit number of minutes (voice intent, E021-T06).
    ///
    /// Differs from [`dismiss`](Self::dismiss) in two ways, both deliberate:
    /// the length comes from the user rather than from the profile's
    /// escalation ladder, and `snooze_index` is left alone — a dictated
    /// "drzemka 5" is not an escalation step, so it must not push the next
    /// dismissal further out.
    ///
    /// A request of zero minutes is ignored rather than treated as "unsnooze",
    /// because the two are different asks and the parser never produces zero.
    pub fn snooze_for(&mut self, minutes: u16) {
        self.snooze_for_at(minutes, Instant::now());
    }

    pub fn snooze_for_at(&mut self, minutes: u16, now: Instant) {
        if minutes == 0 {
            return;
        }
        self.snoozed_until = Some(now + Duration::from_secs(u64::from(minutes) * 60));
        self.last_notify_time.set(None);
        self.notify_count.set(0);
    }

    /// Whether escalation is currently silenced. An expired snooze reads as
    /// `false` without needing an [`evaluate`](Self::evaluate) tick first.
    pub fn is_snoozed(&self) -> bool {
        self.is_snoozed_at(Instant::now())
    }

    pub fn is_snoozed_at(&self, now: Instant) -> bool {
        self.snoozed_until.is_some_and(|until| now < until)
    }

    /// Captures policy timing as bounded wall-clock instants for restart.
    pub fn persisted_state_at(
        &self,
        wall_now: DateTime<Utc>,
        monotonic_now: Instant,
    ) -> PersistedNotificationPolicy {
        let max_age = Duration::from_secs(self.max_persisted_snooze_secs());
        let snoozed_until_utc = self.snoozed_until.and_then(|until| {
            let remaining = until.checked_duration_since(monotonic_now)?;
            if remaining > max_age { return None; }
            wall_now.checked_add_signed(chrono::Duration::from_std(remaining).ok()?)
        });
        let last_notify_at_utc = self.last_notify_time.get().and_then(|last| {
            let age = monotonic_now.checked_duration_since(last)?;
            if age > max_age { return None; }
            wall_now.checked_sub_signed(chrono::Duration::from_std(age).ok()?)
        });
        PersistedNotificationPolicy {
            profile_id: self.comm_profile.id.clone(),
            saved_at_utc: wall_now,
            snoozed_until_utc,
            snooze_index: self.snooze_index.min(self.comm_profile.snooze.durations_mins.len().saturating_sub(1)),
            last_notify_at_utc,
            notify_count: self.notify_count.get().min(crate::communication_profile::MAX_NOTIFY_REMINDERS as u32),
        }
    }

    /// Restores only state belonging to the currently selected profile.
    /// Clock rollback expires active silence and starts a fresh cooldown so it
    /// cannot create indefinite snooze or an immediate repeat burst.
    pub fn restore_persisted_state_at(
        &mut self,
        persisted: &PersistedNotificationPolicy,
        wall_now: DateTime<Utc>,
        monotonic_now: Instant,
    ) -> bool {
        if persisted.profile_id != self.comm_profile.id { return false; }

        let max_age = Duration::from_secs(self.max_persisted_snooze_secs());
        let max_age_chrono = chrono::Duration::from_std(max_age).expect("bounded snooze fits chrono duration");
        let max_index = self.comm_profile.snooze.durations_mins.len().saturating_sub(1);
        self.snooze_index = persisted.snooze_index.min(max_index);
        let count = persisted.notify_count.min(crate::communication_profile::MAX_NOTIFY_REMINDERS as u32);
        self.notify_count.set(count);

        if wall_now < persisted.saved_at_utc {
            self.snoozed_until = None;
            let rollback_count = count.max(1)
                .min(crate::communication_profile::MAX_NOTIFY_REMINDERS as u32);
            self.notify_count.set(rollback_count);
            self.last_notify_time.set(Some(monotonic_now));
            return true;
        }

        self.snoozed_until = persisted.snoozed_until_utc.and_then(|until| {
            let total = until.signed_duration_since(persisted.saved_at_utc);
            if total <= chrono::Duration::zero() || total > max_age_chrono || until <= wall_now {
                return None;
            }
            let remaining = until.signed_duration_since(wall_now).to_std().ok()?;
            if remaining > max_age { return None; }
            monotonic_now.checked_add(remaining)
        });

        let last = persisted.last_notify_at_utc.and_then(|notified| {
            if notified > persisted.saved_at_utc || notified > wall_now { return None; }
            let age = wall_now.signed_duration_since(notified).to_std().ok()?;
            if age > max_age { return None; }
            monotonic_now.checked_sub(age)
        });
        self.last_notify_time.set(if count > 0 && last.is_none() { Some(monotonic_now) } else { last });
        true
    }

    /// Bounds restart conversion to the active profile's actual dismiss
    /// schedule or the voice route's accepted maximum, whichever is longer.
    fn max_persisted_snooze_secs(&self) -> u64 {
        let profile_max = self.comm_profile.snooze.durations_mins.iter().copied().max().unwrap_or(0);
        u64::from(profile_max.max(u32::from(crate::voice_intent::MAX_SNOOZE_MINS)))
            .saturating_mul(60)
    }

    /// Reset snooze state and notification tracking on position change.
    pub fn on_position_changed(&mut self) {
        self.snoozed_until = None;
        self.snooze_index = 0;
        self.last_notify_time.set(None);
        self.notify_count.set(0);
    }

    /// Clear the disconnect-notified flag when the sensor reconnects.
    pub fn on_sensor_connected(&mut self) {
        self.disconnect_notified = false;
    }

    /// Hot-reload the communication profile.
    pub fn set_comm_profile(&mut self, p: CommunicationProfile) {
        if self.comm_profile.id != p.id {
            self.snoozed_until = None;
            self.snooze_index = 0;
            self.last_notify_time.set(None);
            self.notify_count.set(0);
        }
        self.comm_profile = p;
    }

    /// Hot-reload the ergonomic profile.
    pub fn set_ergo_profile(&mut self, p: ErgonomicProfile) {
        self.ergo_profile = p;
    }

    /// Read access to the current communication profile.
    pub fn comm_profile(&self) -> &CommunicationProfile {
        &self.comm_profile
    }

    /// Read access to the current ergonomic profile.
    pub fn ergo_profile(&self) -> &ErgonomicProfile {
        &self.ergo_profile
    }

    pub(crate) fn take_reminder(&mut self) -> Option<ReminderDraft> { self.last_reminder.take() }

    fn next_reminder_occurrence(&self) -> u64 {
        let next = self.reminder_occurrence.get().checked_add(1).expect("reminder occurrence exhausted");
        self.reminder_occurrence.set(next);
        next
    }

    pub(crate) fn reminder_snooze_minutes(&self) -> Result<u16, String> {
        let durations = &self.comm_profile.snooze.durations_mins;
        let index = self.snooze_index.min(durations.len().saturating_sub(1));
        let minutes = durations.get(index).copied().unwrap_or(5);
        u16::try_from(minutes).ok().filter(|minutes| *minutes > 0)
            .ok_or_else(|| "unsupported_snooze_duration".into())
    }

    fn active_step_index(steps: &[EscalationStep], offset: i64) -> Option<usize> {
        steps.iter().rposition(|step| offset >= step.at)
    }

    /// Shared predicates for acceptance and retirement; one definition per reason.
    fn session_reason_current(&self, event: &NotificationEvent, facts: &SessionNotificationFacts) -> bool {
        let config = &self.comm_profile.periodic_notifications;
        match event {
            NotificationEvent::Inactivity => config.inactivity_enabled && facts.last_position_change_at.is_some_and(|at|
                (facts.now - at).num_seconds() >= i64::from(config.inactivity_after_mins) * 60),
            NotificationEvent::PostureBalance => config.posture_balance_enabled
                && facts.sitting_seconds_total >= self.ergo_profile.limits.posture_balance_min_sitting_secs as i64
                && facts.sitting_seconds_total > facts.standing_seconds * 2,
            NotificationEvent::StandingTargetReached => facts.standing_target_secs > 0 && facts.standing_seconds >= facts.standing_target_secs,
            NotificationEvent::Praise => config.praise_halfway_enabled && facts.standing_target_secs > 0
                && facts.standing_seconds >= facts.standing_target_secs / 2,
            NotificationEvent::StandLimitReached => false,
        }
    }

    pub(crate) fn reminder_obsolete_reason(&self, draft: &ReminderDraft, day: chrono::NaiveDate,
        input: &PolicyInput, facts: &SessionNotificationFacts, sitting_limit: i64, standing_elapsed: i64,
        now: Instant,
    ) -> Option<RetirementReason> {
        if draft.reason == ReminderReason::DeviceDisconnected {
            if !self.comm_profile.notifications_enabled
                || !matches!(self.comm_profile.disconnected.notify_once.as_deref(), Some("toast" | "popup")) {
                return Some(RetirementReason::ProfileDisabled);
            }
            return input.sensor_connected.then_some(RetirementReason::ContextChanged);
        }
        if !self.comm_profile.notifications_enabled || crate::notification_service::channels_for_backend(&self.comm_profile.notification_backend).is_empty() {
            return Some(RetirementReason::ProfileDisabled);
        }
        if !input.sensor_connected || !input.observation_fresh { return Some(RetirementReason::ObservationUnavailable); }
        if day != facts.now.with_timezone(&chrono::Local).date_naive() { return Some(RetirementReason::ContextChanged); }
        if self.is_snoozed_at(now) && (matches!(draft.reason, ReminderReason::SittingLimit | ReminderReason::StandingLimit | ReminderReason::ScreenBreak)
            || (input.state == DeskState::Sitting && matches!(draft.reason, ReminderReason::Inactivity | ReminderReason::PostureBalance))) {
            return Some(RetirementReason::ContextChanged);
        }
        let valid = match draft.reason {
            ReminderReason::SittingLimit | ReminderReason::StandingLimit => {
                let sitting = draft.reason == ReminderReason::SittingLimit;
                let (state, steps, elapsed, limit) = if sitting {
                    (DeskState::Sitting, &self.comm_profile.escalation.sitting, input.elapsed_secs, sitting_limit)
                } else { (DeskState::Standing, &self.comm_profile.escalation.standing, standing_elapsed,
                    self.ergo_profile.limits.standing_max_secs as i64) };
                input.state == state && Self::active_step_index(steps, elapsed.saturating_sub(limit)).is_some_and(|idx|
                    steps[..=idx].iter().any(|step| matches!(step.notify.as_deref(), Some("toast" | "popup" | "both"))))
            }
            ReminderReason::ScreenBreak => self.screen_break_current(input),
            ReminderReason::Inactivity => !matches!(input.state, DeskState::Away | DeskState::Walking)
                && draft.position_anchor == facts.last_position_change_at.map(|at| at.to_rfc3339())
                && self.session_reason_current(&NotificationEvent::Inactivity, facts),
            ReminderReason::PostureBalance => self.session_reason_current(&NotificationEvent::PostureBalance, facts),
            ReminderReason::Praise => self.session_reason_current(&NotificationEvent::Praise, facts),
            ReminderReason::StandingTarget => self.session_reason_current(&NotificationEvent::StandingTargetReached, facts),
            ReminderReason::Preview => true,
            ReminderReason::DeviceDisconnected => unreachable!("device reason handled before coverage guards"),
        };
        (!valid).then_some(RetirementReason::ContextChanged)
    }

    // ── private helpers ───────────────────────────────────────────────────────

    fn screen_break_current(&self, input: &PolicyInput) -> bool {
        input.state == DeskState::Standing && self.comm_profile.screen_break_nudge.enabled
            && !self.comm_profile.screen_break_nudge.messages.is_empty()
            && self.ergo_profile.limits.max_continuous_computer_secs > 0
            && input.continuous_computer_secs >= self.ergo_profile.limits.max_continuous_computer_secs as i64
    }

    /// Check if a screen break nudge should fire and apply it to signals.
    fn maybe_apply_screen_nudge(&mut self, input: &PolicyInput, signals: &mut Signals) {
        if input.continuous_computer_secs < self.ergo_profile.limits.max_continuous_computer_secs as i64 {
            self.screen_break_nudge_fired = false;
        }

        if self.screen_break_current(input)
            && signals.notify.is_none()
            && !self.screen_break_nudge_fired
            && self.comm_profile.notifications_enabled
            && !crate::notification_service::channels_for_backend(&self.comm_profile.notification_backend).is_empty()
        {
            if let Some(msg) = crate::screen_break_nudge::pick_nudge_message(
                &self.comm_profile.screen_break_nudge.messages,
            ) {
                signals.notify = Some(NotifySignal::Toast(msg));
                self.last_reminder = Some(ReminderDraft { occurrence: self.next_reminder_occurrence(), reason: ReminderReason::ScreenBreak, category: ReminderCategory::Warning,
                    title: "Take a screen break".into(), body: match &signals.notify { Some(NotifySignal::Toast(body)) => body.clone(), _ => unreachable!() },
                    elapsed_secs: Some(input.continuous_computer_secs), limit_secs: Some(self.ergo_profile.limits.max_continuous_computer_secs as i64), position_anchor: None, observation_day: None });
                self.screen_break_nudge_fired = true;
            }
        }
    }

    fn make_baseline(&self, input: &PolicyInput, sitting_limit_secs: i64) -> Signals {
        let sitting_progress = if sitting_limit_secs > 0 {
            (input.elapsed_secs as f32 / sitting_limit_secs as f32).min(1.5)
        } else {
            0.0
        };
        helpers::baseline_signals(
            &input.state,
            &self.comm_profile.baseline,
            sitting_progress,
            input.standing_lap_progress,
            input.standing_lap,
            input.standing_lap_flash,
        )
    }

    fn step_to_signals(
        &mut self,
        step: &EscalationStep,
        notify_type: Option<&str>,
        input: &PolicyInput,
        sitting_limit_secs: i64,
        notification_elapsed_secs: i64,
        now: Instant,
    ) -> Signals {
        let progress = match input.state {
            DeskState::Sitting if sitting_limit_secs > 0 => (input.elapsed_secs as f32 / sitting_limit_secs as f32).min(1.5),
            DeskState::Sitting => 0.0,
            DeskState::Standing => input.standing_lap_progress,
            _ => 0.0,
        };

        let tray = helpers::parse_tray_signal(&step.tray);
        let overlay = helpers::parse_overlay_signal(&step.overlay, progress);
        let popup = helpers::parse_popup_signal(&step.popup_header);
        let notify = self.build_step_notify(notify_type, input, notification_elapsed_secs, now);
        if let Some(NotifySignal::Toast(body) | NotifySignal::Popup(body)) = &notify {
            let sitting = input.state == DeskState::Sitting;
            self.last_reminder = Some(ReminderDraft {
                occurrence: self.next_reminder_occurrence(),
                reason: if sitting { ReminderReason::SittingLimit } else { ReminderReason::StandingLimit },
                category: if step.at < 0 { ReminderCategory::Warning } else { ReminderCategory::Urgent },
                title: if sitting { "Time to stand up" } else { "Time to change position" }.into(),
                body: body.clone(), elapsed_secs: Some(notification_elapsed_secs),
                limit_secs: Some(if sitting { sitting_limit_secs } else { self.ergo_profile.limits.standing_max_secs as i64 }),
                position_anchor: None,
                observation_day: None,
            });
        }

        Signals { tray, overlay, popup, notify, accepted_events: Vec::new() }
    }

    /// Cooldown before next notification, or `None` if all reminders exhausted.
    fn notify_cooldown_secs(&self) -> Option<u64> {
        let cooldowns = &self.comm_profile.snooze.notify_cooldowns_secs;
        let notify_count = self.notify_count.get();
        if notify_count as usize >= crate::communication_profile::MAX_NOTIFY_REMINDERS {
            return None;
        }
        cooldowns.get(notify_count as usize).copied()
    }

    fn build_step_notify(
        &mut self,
        notify_type: Option<&str>,
        input: &PolicyInput,
        notification_elapsed_secs: i64,
        now: Instant,
    ) -> Option<NotifySignal> {
        if !self.comm_profile.notifications_enabled {
            return None;
        }
        let notify_type = notify_type?;
        if !matches!(notify_type, "toast" | "popup" | "both")
            || crate::notification_service::channels_for_backend(&self.comm_profile.notification_backend).is_empty() {
            return None;
        }

        // The profile's first enabled step starts its reminder cadence. Later
        // visual-only steps retain that channel while the configured cooldown
        // schedule permits another reminder.
        let cooldown = self.notify_cooldown_secs()?;
        if let Some(last) = self.last_notify_time.get() {
            if now.checked_duration_since(last).unwrap_or_default() < Duration::from_secs(cooldown) {
                return None;
            }
        }

        self.last_notify_time.set(Some(now));
        self.notify_count.set(self.notify_count.get() + 1);

        let is_firm = self.snooze_index
            >= self.comm_profile.snooze.tone_shift_after_dismisses as usize;
        let msg = helpers::message_for_state(
            &input.state,
            notify_type,
            &self.comm_profile.messages,
            is_firm,
            notification_elapsed_secs,
        );
        Some(helpers::make_notify_signal(notify_type, &msg))
    }
}
