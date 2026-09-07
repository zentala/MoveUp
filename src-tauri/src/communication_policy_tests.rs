//! Unit tests for [`CommunicationPolicy`].
//!
//! Tests cover all priority branches: disconnected, inactive, escalation steps,
//! baseline, snooze/dismiss, and notification de-duplication.

#[cfg(test)]
mod tests {
    use crate::communication_policy::{CommunicationPolicy, PolicyInput};
    use crate::communication_profile::CommunicationProfile;
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
}
