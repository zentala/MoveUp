//! Unit tests for [`CommunicationPolicy`].
//!
//! Tests cover all priority branches: disconnected, inactive, escalation steps,
//! baseline, snooze/dismiss, and notification de-duplication.

#[cfg(test)]
mod tests {
    use crate::communication_policy::{CommunicationPolicy, PolicyInput, SessionNotificationFacts};
    use crate::communication_profile::{CommunicationProfile, EscalationStep};
    use crate::communication_types::{
        NotifySignal, OverlaySignal, PopupSignal, TraySignal,
    };
    use crate::ergonomic_profile::ErgonomicProfile;
    use crate::session_types::DeskState;

    fn default_policy() -> CommunicationPolicy {
        CommunicationPolicy::new(CommunicationProfile::default(), ErgonomicProfile::default())
    }

    fn sitting_input(elapsed_secs: i64) -> PolicyInput {
        PolicyInput {
            state: DeskState::Sitting,
            elapsed_secs,
            sensor_connected: true,
            standing_lap_progress: 0.0,
            standing_lap: 0,
            standing_lap_flash: false,
            continuous_computer_secs: 0,
        }
    }

    fn standing_input(elapsed_secs: i64, progress: f32) -> PolicyInput {
        PolicyInput {
            state: DeskState::Standing,
            elapsed_secs,
            sensor_connected: true,
            standing_lap_progress: progress,
            standing_lap: 0,
            standing_lap_flash: false,
            continuous_computer_secs: 0,
        }
    }

    // ── 1. Sitting baseline ───────────────────────────────────────────────────

    #[test]
    fn sitting_baseline_within_limit() {
        let mut p = default_policy();
        // 500s elapsed, limit 2400s (default) — well within limit
        let sig = p.evaluate(&sitting_input(500));
        assert_eq!(sig.tray, TraySignal::None);
        assert!(matches!(sig.overlay, OverlaySignal::Neutral { .. }));
        assert_eq!(sig.popup, PopupSignal::Neutral);
        assert!(sig.notify.is_none());
    }

    // ── 2. Sitting warning (escalation at=-600) ───────────────────────────────

    #[test]
    fn sitting_warning_at_minus600() {
        let mut p = default_policy();
        // Default limit=2400, step at=-600 triggers at 2400-600=1800s
        let sig = p.evaluate(&sitting_input(1800));
        assert_eq!(sig.tray, TraySignal::Yellow);
        assert!(matches!(sig.overlay, OverlaySignal::Yellow { .. }));
    }

    // ── 3. Sitting at limit (at=0) → Red + Toast ──────────────────────────────

    #[test]
    fn sitting_at_limit_fires_toast() {
        let mut p = default_policy();
        let sig = p.evaluate(&sitting_input(2400));
        assert_eq!(sig.tray, TraySignal::Red);
        assert!(matches!(sig.overlay, OverlaySignal::Red { .. }));
        assert!(matches!(sig.notify, Some(NotifySignal::Toast(_))));
    }

    // ── 4. Sitting overdue (at=+300) → Blink + PulseRed, visual-only ────────

    #[test]
    fn sitting_overdue_blink_visual_only() {
        let mut p = default_policy();
        // Advance past at=0 first to record that notify step
        let _ = p.evaluate(&sitting_input(2400));
        // Now at limit+300 = 2700s — visual escalation, no second notification
        let sig = p.evaluate(&sitting_input(2700));
        assert!(matches!(sig.tray, TraySignal::Blink(_)));
        assert!(matches!(sig.overlay, OverlaySignal::PulseRed { .. }));
        assert!(sig.notify.is_none(), "overdue step should be visual-only, no notification");
    }

    // ── 5. Standing baseline ──────────────────────────────────────────────────

    #[test]
    fn standing_baseline_shows_progress() {
        let mut p = default_policy();
        // 300s elapsed, standing_max=5400 — well within limit
        let sig = p.evaluate(&standing_input(300, 0.25));
        assert_eq!(sig.tray, TraySignal::None);
        assert!(matches!(sig.overlay, OverlaySignal::Progress { progress, .. } if (progress - 0.25).abs() < 0.001));
    }

    // ── 6. Standing warning (at=-300, triggers at max-300) ───────────────────

    #[test]
    fn standing_warning_step() {
        let mut p = default_policy();
        // standing_max=5400, at=-300 triggers at 5100s
        let sig = p.evaluate(&standing_input(5100, 0.9));
        assert_eq!(sig.tray, TraySignal::Yellow);
        assert!(matches!(sig.overlay, OverlaySignal::Yellow { .. }));
    }

    // ── 7. Standing at limit → Red + Toast ───────────────────────────────────

    #[test]
    fn standing_at_limit_fires_toast() {
        let mut p = default_policy();
        let sig = p.evaluate(&standing_input(5400, 1.0));
        assert_eq!(sig.tray, TraySignal::Red);
        assert!(matches!(sig.overlay, OverlaySignal::Red { .. }));
        assert!(matches!(sig.notify, Some(NotifySignal::Toast(_))));
    }

    // ── 8. Disconnected → Gray blink + toast once ────────────────────────────

    #[test]
    fn disconnected_fires_toast_only_once() {
        let mut p = default_policy();
        let input = PolicyInput {
            state: DeskState::Sitting,
            elapsed_secs: 0,
            sensor_connected: false,
            standing_lap_progress: 0.0,
            standing_lap: 0,
            standing_lap_flash: false,
            continuous_computer_secs: 0,
        };
        let sig1 = p.evaluate(&input);
        assert!(matches!(sig1.tray, TraySignal::Blink(_)));
        assert_eq!(sig1.popup, PopupSignal::Gray);
        assert!(sig1.notify.is_some());

        let sig2 = p.evaluate(&input);
        assert!(sig2.notify.is_none(), "toast should not re-fire on second call");
    }

    #[test]
    fn disconnected_none_channel_has_no_notification_and_keeps_visual_state() {
        let mut comm = CommunicationProfile::default();
        comm.disconnected.notify_once = None;
        let mut p = CommunicationPolicy::new(comm, ErgonomicProfile::default());
        let input = PolicyInput {
            state: DeskState::Sitting,
            elapsed_secs: 0,
            sensor_connected: false,
            standing_lap_progress: 0.0,
            standing_lap: 0,
            standing_lap_flash: false,
            continuous_computer_secs: 0,
        };

        let signals = p.evaluate(&input);

        assert!(signals.notify.is_none(), "None must not become an empty toast");
        assert!(matches!(signals.tray, TraySignal::Blink(_)));
        assert_eq!(signals.popup, PopupSignal::Gray);
    }

    #[test]
    fn silent_builtin_keeps_visual_state_without_emitting_a_notification() {
        let profile: CommunicationProfile =
            serde_json::from_str(include_str!("../profiles/communication/silent.json")).unwrap();
        let mut policy = CommunicationPolicy::new(profile, ErgonomicProfile::default());

        let input = sitting_input(3600);
        assert!(input.sensor_connected, "exercise the connected sensor path");
        let signals = policy.evaluate(&input);

        assert!(signals.notify.is_none());
        assert!(matches!(signals.overlay, OverlaySignal::Neutral { .. }));
    }

    fn standing_session_facts() -> SessionNotificationFacts {
        SessionNotificationFacts {
            state: DeskState::Standing,
            now: chrono::Utc::now(),
            last_position_change_at: Some(chrono::Utc::now() - chrono::Duration::hours(2)),
            sitting_seconds_total: 7200,
            standing_seconds: 7200,
            standing_target_secs: 3600,
            inactivity_fired: false,
            posture_balance_fired: false,
            standing_target_fired: false,
            praise_fired: false,
        }
    }

    fn assert_no_dispatched_notifications(policy: &CommunicationPolicy, periodic: bool) {
        let facts = standing_session_facts();
        let intents = policy.resolve_session_notifications(&facts, periodic);
        let mut logged = Vec::new();
        let mut dispatched_channels = 0;

        crate::notification_service::NotificationService::dispatch_with(
            &intents,
            |line| logged.push(line.to_string()),
            |channels, _, _| dispatched_channels += channels.len(),
        );

        assert!(logged.is_empty(), "Silent must not log intents: {logged:?}");
        assert_eq!(dispatched_channels, 0, "Silent must not call notification channels");
    }

    #[test]
    fn silent_builtin_suppresses_periodic_session_intents_at_dispatch() {
        let profile: CommunicationProfile =
            serde_json::from_str(include_str!("../profiles/communication/silent.json")).unwrap();
        let policy = CommunicationPolicy::new(profile, ErgonomicProfile::default());

        assert_no_dispatched_notifications(&policy, true);
    }

    #[test]
    fn silent_builtin_suppresses_nonperiodic_session_intents_at_dispatch() {
        let mut profile: CommunicationProfile =
            serde_json::from_str(include_str!("../profiles/communication/silent.json")).unwrap();
        profile.periodic_notifications.praise_halfway_enabled = true;
        let policy = CommunicationPolicy::new(profile, ErgonomicProfile::default());

        assert_no_dispatched_notifications(&policy, false);
    }

    #[test]
    fn inactivity_notification_uses_the_profile_duration_and_injected_time() {
        let mut profile = CommunicationProfile::default();
        profile.periodic_notifications.inactivity_enabled = true;
        profile.periodic_notifications.inactivity_after_mins = 7;
        let policy = CommunicationPolicy::new(profile, ErgonomicProfile::default());
        let changed_at = chrono::Utc::now();
        let mut facts = standing_session_facts();
        facts.standing_target_secs = 0;
        facts.last_position_change_at = Some(changed_at);
        facts.now = changed_at + chrono::Duration::minutes(6);

        assert!(policy.decide_session_notifications(&facts, true).is_empty());

        facts.now = changed_at + chrono::Duration::minutes(7);
        let notifications = policy.decide_session_notifications(&facts, true);
        assert_eq!(notifications.len(), 1);
        assert!(matches!(
            notifications.first(),
            Some(crate::session_types::NotificationEvent::Inactivity)
        ));
    }

    #[test]
    fn sitting_session_events_and_ladder_share_four_notification_budget() {
        use crate::notification_service::NotificationService;

        let mut profile = CommunicationProfile::default();
        profile.snooze.notify_cooldowns_secs = vec![0; 4];
        profile.escalation.sitting = vec![EscalationStep {
            at: 0,
            tray: "yellow".into(),
            overlay: "pulse_red".into(),
            popup_header: "red".into(),
            notify: Some("toast".into()),
        }];
        profile.periodic_notifications.inactivity_enabled = true;
        profile.periodic_notifications.inactivity_after_mins = 1;
        profile.periodic_notifications.posture_balance_enabled = true;
        let mut ergonomic = ErgonomicProfile::default();
        ergonomic.limits.posture_balance_min_sitting_secs = 1;
        let mut policy = CommunicationPolicy::new(profile, ergonomic);
        let reminder_now = std::time::Instant::now();
        let mut log_count = 0;
        let mut channel_count = 0;
        let mut webhook_count = 0;

        for _ in 0..2 {
            let signals = policy.evaluate_with_stand_limit_fired_at(
                &sitting_input(2700), 2400, 0, false, reminder_now,
            );
            assert!(matches!(signals.notify, Some(NotifySignal::Toast(_))));
            crate::tray_controller::dispatch_notification_with(
                signals.notify.as_ref(),
                |signal| NotificationService::dispatch_policy_signal_with(
                    signal,
                    || log_count += 1,
                    |_channel, _title, _body| channel_count += 1,
                ),
                |signal| {
                    if matches!(signal, NotifySignal::Toast(_)) {
                        webhook_count += 1;
                    }
                },
            );
        }

        let now = chrono::Utc::now();
        let facts = SessionNotificationFacts {
            state: DeskState::Sitting,
            now,
            last_position_change_at: Some(now - chrono::Duration::minutes(2)),
            sitting_seconds_total: 7200,
            standing_seconds: 0,
            standing_target_secs: 0,
            inactivity_fired: false,
            posture_balance_fired: false,
            standing_target_fired: false,
            praise_fired: false,
        };
        let session_intents = policy.resolve_session_notifications_at(&facts, true, reminder_now);
        assert_eq!(session_intents.len(), 2, "both sitting session producers are eligible before exhaustion");
        NotificationService::dispatch_with(
            &session_intents,
            |_line| log_count += 1,
            |channels, _title, _body| channel_count += channels.len(),
        );

        let exhausted_ladder = policy.evaluate_with_stand_limit_fired_at(
            &sitting_input(2700), 2400, 0, false, reminder_now,
        );
        assert!(exhausted_ladder.notify.is_none(), "the mixed producer total must exhaust the fourth slot");
        crate::tray_controller::dispatch_notification_with(
            exhausted_ladder.notify.as_ref(),
            |signal| NotificationService::dispatch_policy_signal_with(
                signal,
                || log_count += 1,
                |_channel, _title, _body| channel_count += 1,
            ),
            |_signal| webhook_count += 1,
        );

        let exhausted_session = policy.resolve_session_notifications_at(&facts, true, reminder_now);
        assert!(exhausted_session.is_empty(), "session producers must stay silent after the shared budget");
        NotificationService::dispatch_with(
            &exhausted_session,
            |_line| log_count += 1,
            |channels, _title, _body| channel_count += channels.len(),
        );

        let mut standing_facts = SessionNotificationFacts {
            state: DeskState::Standing,
            now: facts.now,
            last_position_change_at: facts.last_position_change_at,
            sitting_seconds_total: facts.sitting_seconds_total,
            standing_seconds: 60,
            standing_target_secs: 60,
            inactivity_fired: true,
            posture_balance_fired: true,
            standing_target_fired: false,
            praise_fired: false,
        };
        let standing_target = policy.decide_session_notifications(&standing_facts, true);
        assert!(matches!(
            standing_target.as_slice(),
            [crate::session_types::NotificationEvent::StandingTargetReached]
        ));
        let mut standing_profile = policy.comm_profile().clone();
        standing_profile.periodic_notifications.praise_halfway_enabled = true;
        policy.set_comm_profile(standing_profile);
        standing_facts.standing_target_fired = true;
        standing_facts.standing_seconds = 30;
        let praise = policy.decide_session_notifications(&standing_facts, false);
        assert!(matches!(
            praise.as_slice(),
            [crate::session_types::NotificationEvent::Praise]
        ));

        assert_eq!(log_count, 4, "only four sitting notification events are logged");
        assert_eq!(channel_count, 4, "only four notification channels are dispatched");
        assert_eq!(webhook_count, 2, "only the two ladder toasts mirror to the webhook");
    }

    #[test]
    fn sitting_session_events_share_snooze_and_configured_schedule_gates() {
        let base_now = std::time::Instant::now();
        let utc_now = chrono::Utc::now();
        let facts = SessionNotificationFacts {
            state: DeskState::Sitting,
            now: utc_now,
            last_position_change_at: Some(utc_now - chrono::Duration::minutes(2)),
            sitting_seconds_total: 7200,
            standing_seconds: 0,
            standing_target_secs: 0,
            inactivity_fired: false,
            posture_balance_fired: false,
            standing_target_fired: false,
            praise_fired: false,
        };

        let mut short_schedule = CommunicationProfile::default();
        short_schedule.snooze.notify_cooldowns_secs = vec![0; 2];
        short_schedule.periodic_notifications.inactivity_enabled = true;
        short_schedule.periodic_notifications.inactivity_after_mins = 1;
        let mut short_policy = CommunicationPolicy::new(short_schedule, ErgonomicProfile::default());
        for _ in 0..2 {
            let signal = short_policy.evaluate_with_stand_limit_fired_at(
                &sitting_input(2700), 2400, 0, false, base_now,
            );
            assert!(signal.notify.is_some());
        }
        assert_eq!(short_policy.decide_session_notifications(&facts, true).len(), 1);
        assert!(short_policy
            .resolve_session_notifications_at(&facts, true, base_now)
            .is_empty(), "a two-entry profile schedule cannot emit a third reminder");

        let mut snooze_profile = CommunicationProfile::default();
        snooze_profile.periodic_notifications.inactivity_enabled = true;
        snooze_profile.periodic_notifications.inactivity_after_mins = 1;
        let mut snoozed_policy = CommunicationPolicy::new(snooze_profile, ErgonomicProfile::default());
        snoozed_policy.snooze_for_at(5, base_now);
        assert!(snoozed_policy
            .resolve_session_notifications_at(&facts, true, base_now + std::time::Duration::from_secs(299))
            .is_empty(), "sitting session events obey an active snooze");
        let after_snooze = snoozed_policy.resolve_session_notifications_at(
            &facts, true, base_now + std::time::Duration::from_secs(300),
        );
        assert!(matches!(
            after_snooze.first().map(|intent| &intent.event),
            Some(crate::session_types::NotificationEvent::Inactivity)
        ));
    }

    #[test]
    fn all_builtin_profiles_follow_their_real_first_sitting_notification_step() {
        for (name, json) in [
            ("default", include_str!("../profiles/communication/default.json")),
            ("aggressive", include_str!("../profiles/communication/aggressive.json")),
            ("gentle", include_str!("../profiles/communication/gentle.json")),
            ("silent", include_str!("../profiles/communication/silent.json")),
            ("demo", include_str!("../profiles/communication/demo.json")),
        ] {
            let profile: CommunicationProfile = serde_json::from_str(json).unwrap_or_else(|error| {
                panic!("{name} profile should deserialize: {error}")
            });
            let expected = profile
                .escalation
                .sitting
                .iter()
                .rev()
                .find(|step| step.at <= 0)
                .and_then(|step| step.notify.clone());
            let mut policy = CommunicationPolicy::new(profile, ErgonomicProfile::default());

            let signals = policy.evaluate(&sitting_input(2400));

            match (expected.as_deref(), signals.notify) {
                (None, None) => {}
                (Some("toast"), Some(NotifySignal::Toast(_))) => {}
                (Some("popup"), Some(NotifySignal::Popup(_))) => {}
                (expected, actual) => panic!("{name}: configured {expected:?}, produced {actual:?}"),
            }
        }
    }

    #[test]
    fn explicit_none_suppresses_only_notification_not_escalation_visuals() {
        let mut profile = CommunicationProfile::default();
        profile.escalation.sitting = vec![EscalationStep {
            at: 0,
            tray: "red".into(),
            overlay: "pulse_red".into(),
            popup_header: "red".into(),
            notify: None,
        }];
        let mut policy = CommunicationPolicy::new(profile, ErgonomicProfile::default());

        let signals = policy.evaluate(&sitting_input(2400));

        assert!(signals.notify.is_none());
        assert_eq!(signals.tray, TraySignal::Red);
        assert!(matches!(signals.overlay, OverlaySignal::PulseRed { .. }));
        assert_eq!(signals.popup, PopupSignal::Red);
    }

    #[test]
    fn popup_minutes_are_interpolated_from_the_policy_input() {
        let mut profile = CommunicationProfile::default();
        profile.snooze.notify_cooldowns_secs = vec![0, 0];
        profile.escalation.sitting = vec![EscalationStep {
            at: 0,
            tray: "red".into(),
            overlay: "red".into(),
            popup_header: "red".into(),
            notify: Some("popup".into()),
        }];
        profile.messages.sitting_overdue_popup = "Sitting for {minutes} minutes".into();
        profile.escalation.standing = vec![EscalationStep {
            at: 0,
            tray: "red".into(),
            overlay: "red".into(),
            popup_header: "red".into(),
            notify: Some("popup".into()),
        }];
        profile.messages.standing_overdue_popup = "Standing for {minutes} minutes".into();
        let mut policy = CommunicationPolicy::new(profile, ErgonomicProfile::default());
        let now = std::time::Instant::now();

        let sitting = policy.evaluate_with_stand_limit_fired_at(
            &sitting_input(2705), 2400, 0, false, now,
        );
        assert_eq!(sitting.notify, Some(NotifySignal::Popup("Sitting for 45 minutes".into())));

        let standing = PolicyInput {
            state: DeskState::Standing,
            elapsed_secs: 30,
            sensor_connected: true,
            standing_lap_progress: 0.5,
            standing_lap: 0,
            standing_lap_flash: false,
            continuous_computer_secs: 0,
        };
        let standing = policy.evaluate_with_stand_limit_fired_at(
            &standing, 2400, 5400, false, now,
        );
        assert_eq!(standing.notify, Some(NotifySignal::Popup("Standing for 90 minutes".into())));
    }

    // ── 9. Away → all none/hidden ─────────────────────────────────────────────

    #[test]
    fn away_state_returns_hidden_signals() {
        let mut p = default_policy();
        let input = PolicyInput {
            state: DeskState::Away,
            elapsed_secs: 100,
            sensor_connected: true,
            standing_lap_progress: 0.0,
            standing_lap: 0,
            standing_lap_flash: false,
            continuous_computer_secs: 0,
        };
        let sig = p.evaluate(&input);
        assert_eq!(sig.tray, TraySignal::None);
        assert_eq!(sig.overlay, OverlaySignal::Hidden);
    }

    // ── 10. Dismiss + snooze → baseline during snooze ────────────────────────

    #[test]
    fn dismiss_returns_baseline_during_snooze() {
        let mut p = default_policy();
        // Trigger an alert
        let _ = p.evaluate(&sitting_input(2400));
        // User dismisses
        p.dismiss();
        // Immediately after dismiss, should get baseline (snoozed)
        let sig = p.evaluate(&sitting_input(2400));
        assert_eq!(sig.tray, TraySignal::None, "should be baseline (snoozed)");
        assert!(matches!(sig.overlay, OverlaySignal::Neutral { .. }));
    }

    // ── 11. Position changed → resets snooze ─────────────────────────────────

    #[test]
    fn position_changed_resets_snooze_state() {
        let mut p = default_policy();
        let _ = p.evaluate(&sitting_input(2400));
        p.dismiss();
        p.on_position_changed();
        // After position change, snooze should be cleared, escalation active again
        let sig = p.evaluate(&sitting_input(2400));
        assert!(matches!(sig.notify, Some(NotifySignal::Toast(_))),
            "notification should fire again after position change");
    }

    // ── 12. Notification fires once per step ─────────────────────────────────

    #[test]
    fn notification_fires_once_per_step() {
        let mut p = default_policy();
        let sig1 = p.evaluate(&sitting_input(2400));
        assert!(sig1.notify.is_some(), "first evaluation should fire notify");

        let sig2 = p.evaluate(&sitting_input(2400));
        assert!(sig2.notify.is_none(), "second evaluation at same step must NOT re-fire");
    }

    // ── 13. sensor_connected restores disconnect flag ─────────────────────────

    #[test]
    fn on_sensor_connected_clears_disconnect_flag() {
        let mut p = default_policy();
        let disconnected_input = PolicyInput {
            state: DeskState::Sitting,
            elapsed_secs: 0,
            sensor_connected: false,
            standing_lap_progress: 0.0,
            standing_lap: 0,
            standing_lap_flash: false,
            continuous_computer_secs: 0,
        };
        // Fire disconnect toast
        let sig1 = p.evaluate(&disconnected_input);
        assert!(sig1.notify.is_some());

        // Reconnect
        p.on_sensor_connected();

        // Disconnect again — should fire toast again
        let sig2 = p.evaluate(&disconnected_input);
        assert!(sig2.notify.is_some(), "toast should fire again after reconnect+disconnect");
    }

    // ── 14. Walking state → inactive signals ────────────────────────────────
    // Cooldown tests (14-17) moved to communication_policy_cooldown_tests.rs

    #[test]
    fn walking_state_is_treated_as_inactive() {
        let mut p = default_policy();
        let input = PolicyInput {
            state: DeskState::Walking,
            elapsed_secs: 500,
            sensor_connected: true,
            standing_lap_progress: 0.5,
            standing_lap: 1,
            standing_lap_flash: false,
            continuous_computer_secs: 0,
        };
        let sig = p.evaluate(&input);
        assert_eq!(sig.tray, TraySignal::None);
        assert_eq!(sig.overlay, OverlaySignal::Hidden);
        assert!(sig.notify.is_none());
    }

    // ── 15. Voice-driven snooze (E021-T06) ──────────────────────────────────

    /// A dictated snooze must silence escalation the same way a dismiss does —
    /// the sitting escalation is well past its limit here, so without the
    /// snooze there would be a notification.
    #[test]
    fn e021_t06_snooze_for_silences_escalation() {
        let mut reference = default_policy();
        assert!(
            reference.evaluate(&sitting_input(2400)).notify.is_some(),
            "precondition: this input escalates without a snooze"
        );

        let mut p = default_policy();
        p.snooze_for(5);
        assert!(p.is_snoozed());
        assert!(
            p.evaluate(&sitting_input(2400)).notify.is_none(),
            "a dictated snooze must silence the escalation"
        );
    }

    /// `snooze_for` must not advance the dismissal ladder — the user set the
    /// length themselves, so the next dismiss still gets the first duration.
    #[test]
    fn e021_t06_snooze_for_does_not_advance_the_dismiss_ladder() {
        let mut p = default_policy();
        p.snooze_for(60);
        p.on_position_changed();
        assert!(!p.is_snoozed(), "a position change clears any snooze");

        let mut reference = default_policy();
        reference.dismiss();
        p.dismiss();
        assert_eq!(
            p.is_snoozed(),
            reference.is_snoozed(),
            "the first dismiss after a voice snooze behaves like any first dismiss"
        );
    }

    #[test]
    fn e021_t06_snooze_for_zero_is_ignored() {
        let mut p = default_policy();
        p.snooze_for(0);
        assert!(!p.is_snoozed(), "zero minutes is not a snooze request");
    }

    #[test]
    fn persisted_dismiss_restores_only_remaining_silence_on_monotonic_clock() {
        use chrono::TimeZone;
        use std::time::{Duration, Instant};

        let wall = chrono::Utc.timestamp_opt(1_797_000_000, 0).unwrap();
        let monotonic = Instant::now();
        let mut source = default_policy();
        source.dismiss_at(monotonic);
        let persisted = source.persisted_state_at(wall, monotonic);

        let restored_at = monotonic + Duration::from_secs(20);
        let wall_at_restore = wall + chrono::Duration::seconds(20);
        let mut restored = default_policy();
        assert!(restored.restore_persisted_state_at(&persisted, wall_at_restore, restored_at));
        assert!(restored.is_snoozed_at(restored_at));
        assert!(restored.is_snoozed_at(restored_at + Duration::from_secs(279)));
        assert!(!restored.is_snoozed_at(restored_at + Duration::from_secs(280)));
    }

    #[test]
    fn persisted_deadline_expiration_and_clock_rollback_do_not_extend_silence_or_repeat() {
        use chrono::TimeZone;
        use std::time::{Duration, Instant};

        let wall = chrono::Utc.timestamp_opt(1_797_000_000, 0).unwrap();
        let monotonic = Instant::now();
        let mut source = default_policy();
        source.dismiss_at(monotonic);
        let expired = source.persisted_state_at(wall, monotonic);
        let mut after_expiry = default_policy();
        let after_expiry_at = monotonic + Duration::from_secs(301);
        assert!(after_expiry.restore_persisted_state_at(
            &expired, wall + chrono::Duration::seconds(301), after_expiry_at,
        ));
        assert!(!after_expiry.is_snoozed_at(after_expiry_at));

        let rollback_state = crate::communication_policy::PersistedNotificationPolicy {
            profile_id: "default".into(),
            saved_at_utc: wall,
            snoozed_until_utc: Some(wall + chrono::Duration::minutes(5)),
            snooze_index: 1,
            last_notify_at_utc: None,
            notify_count: 0,
        };
        let mut rolled_back = default_policy();
        let rollback_now = monotonic + Duration::from_secs(400);
        assert!(rolled_back.restore_persisted_state_at(
            &rollback_state, wall - chrono::Duration::seconds(1), rollback_now,
        ));
        assert!(!rolled_back.is_snoozed_at(rollback_now));
        assert_eq!(rolled_back.persisted_state_at(wall, rollback_now).notify_count, 1);
        assert!(rolled_back.evaluate_with_stand_limit_fired_at(
            &sitting_input(2400), 2400, 0, false, rollback_now,
        ).notify.is_none(), "rollback starts a cooldown instead of replaying immediately");
    }

    #[test]
    fn persisted_notification_state_is_rejected_for_another_selected_profile() {
        use chrono::TimeZone;
        use std::time::Instant;

        let wall = chrono::Utc.timestamp_opt(1_797_000_000, 0).unwrap();
        let state = crate::communication_policy::PersistedNotificationPolicy {
            profile_id: "silent".into(),
            saved_at_utc: wall,
            snoozed_until_utc: Some(wall + chrono::Duration::minutes(5)),
            snooze_index: 1,
            last_notify_at_utc: None,
            notify_count: 0,
        };
        let mut policy = default_policy();
        assert!(!policy.restore_persisted_state_at(&state, wall, Instant::now()));
        assert!(!policy.is_snoozed_at(Instant::now()));
    }

    #[test]
    fn persisted_snooze_bound_preserves_voice_and_longer_profile_schedules() {
        use chrono::TimeZone;
        use std::time::{Duration, Instant};

        let wall = chrono::Utc.timestamp_opt(1_797_000_000, 0).unwrap();
        let monotonic = Instant::now();
        let voice_minutes = match crate::voice_intent::parse("snooze 9000") {
            crate::voice_intent::Intent::Snooze(minutes) => minutes,
            other => panic!("expected a parsed voice snooze, got {other:?}"),
        };
        assert_eq!(voice_minutes, crate::voice_intent::MAX_SNOOZE_MINS);
        let mut voice_policy = default_policy();
        voice_policy.snooze_for_at(voice_minutes, monotonic);
        let voice_state = voice_policy.persisted_state_at(wall, monotonic);
        assert_eq!(voice_state.snoozed_until_utc, Some(
            wall + chrono::Duration::minutes(i64::from(voice_minutes)),
        ));
        let mut voice_restored = default_policy();
        let voice_restore_at = monotonic + Duration::from_secs(60);
        assert!(voice_restored.restore_persisted_state_at(
            &voice_state, wall + chrono::Duration::minutes(1), voice_restore_at,
        ));
        assert!(voice_restored.is_snoozed_at(voice_restore_at + Duration::from_secs(178 * 60)));
        assert!(!voice_restored.is_snoozed_at(voice_restore_at + Duration::from_secs(179 * 60)));

        let mut long_profile = CommunicationProfile::default();
        long_profile.snooze.durations_mins = vec![10_000];
        let long_profile = long_profile;
        let mut profile_policy = CommunicationPolicy::new(
            long_profile.clone(), ErgonomicProfile::default(),
        );
        profile_policy.dismiss_at(monotonic);
        let profile_state = profile_policy.persisted_state_at(wall, monotonic);
        assert_eq!(profile_state.snoozed_until_utc, Some(
            wall + chrono::Duration::minutes(10_000),
        ));
        let mut profile_restored = CommunicationPolicy::new(
            long_profile, ErgonomicProfile::default(),
        );
        let profile_restore_at = monotonic + Duration::from_secs(60);
        assert!(profile_restored.restore_persisted_state_at(
            &profile_state, wall + chrono::Duration::minutes(1), profile_restore_at,
        ));
        assert!(profile_restored.is_snoozed_at(
            profile_restore_at + Duration::from_secs(9_998 * 60),
        ));
    }
}
