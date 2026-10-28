//! Cooldown and anti-spam tests for [`CommunicationPolicy`].
//!
//! Split from `communication_policy_tests.rs` to stay under 250 lines.
//! Tests cover: escalating cooldown, max notification limit, dismiss reset,
//! and rapid re-fire blocking.

#[cfg(test)]
mod tests {
    use crate::communication_policy::{CommunicationPolicy, PolicyInput};
    use crate::communication_profile::{CommunicationProfile, EscalationStep};
    use crate::communication_types::NotifySignal;
    use crate::ergonomic_profile::ErgonomicProfile;
    use crate::session_types::DeskState;
    use std::time::{Duration, Instant};

    fn default_policy() -> CommunicationPolicy {
        CommunicationPolicy::new(CommunicationProfile::default(), ErgonomicProfile::default())
    }

    fn sitting_input(elapsed_secs: i64) -> PolicyInput {
        PolicyInput {
            state: DeskState::Sitting,
            elapsed_secs,
            sensor_connected: true,
            observation_fresh: true,
            standing_lap_progress: 0.0,
            standing_lap: 0,
            standing_lap_flash: false,
            continuous_computer_secs: 0,
        }
    }

    // ── 1. Notification count increments and resets ──────────────────────────

    #[test]
    fn sitting_baseline_numeric_during_snooze_uses_runtime_limit() {
        let now = Instant::now();
        let mut policy = default_policy();
        policy.snooze_for_at(5, now);
        let signals = policy.evaluate_with_stand_limit_fired_at(
            &sitting_input(319), 1200, 0, false, now,
        );
        assert!(signals.notify.is_none());
        assert!(matches!(signals.overlay,
            crate::communication_types::OverlaySignal::Neutral { progress }
                if (progress - 0.26583334).abs() < 0.000001),
            "snooze must silence escalation without losing credited sitting load");
    }

    #[test]
    fn notify_count_increments_on_fire() {
        let mut p = default_policy();
        let sig = p.evaluate(&sitting_input(2400));
        assert!(sig.notify.is_some(), "first notification should fire");

        p.on_position_changed();
        let sig2 = p.evaluate(&sitting_input(2400));
        assert!(sig2.notify.is_some(), "notification should fire again after reset");
    }

    // ── 2. Max notifications reached → silence ──────────────────────────────

    #[test]
    fn max_notifications_cooldown_blocks_rapid_steps() {
        // Build a profile with many close notify steps
        let mut comm = CommunicationProfile::default();
        comm.escalation.sitting = (0..6).map(|i| EscalationStep {
            at: -1000 + (i * 10),
            tray: "red".to_string(),
            overlay: "red".to_string(),
            popup_header: "red".to_string(),
            notify: Some("toast".to_string()),
        }).collect();
        let mut p = CommunicationPolicy::new(comm, ErgonomicProfile::default());

        // Fire step 0 — count becomes 1
        let s = p.evaluate(&sitting_input(1400));
        assert!(s.notify.is_some(), "1st fire");

        // Steps 1-4 are blocked by cooldown (300s hasn't passed).
        let s = p.evaluate(&sitting_input(1410));
        assert!(s.notify.is_none(), "blocked by 300s cooldown");
    }

    #[test]
    fn policy_never_emits_a_fifth_reminder_even_if_profile_has_five_entries() {
        let mut comm = CommunicationProfile::default();
        comm.snooze.notify_cooldowns_secs = vec![0; 5];
        comm.escalation.sitting = (0..5).map(|i| EscalationStep {
            at: i * 10,
            tray: "red".to_string(),
            overlay: "red".to_string(),
            popup_header: "red".to_string(),
            notify: Some("toast".to_string()),
        }).collect();
        let mut policy = CommunicationPolicy::new(comm, ErgonomicProfile::default());

        let emitted = (0..5)
            .filter(|step| policy.evaluate(&sitting_input(2400 + step * 10)).notify.is_some())
            .count();

        assert_eq!(emitted, 4, "the accepted maximum is four reminders");
    }

    #[test]
    fn configured_cooldowns_use_injected_time_and_end_in_silence() {
        let mut comm = CommunicationProfile::default();
        comm.snooze.notify_cooldowns_secs = vec![0, 300, 900, 1800];
        comm.escalation.sitting = (0..5).map(|i| EscalationStep {
            at: i * 10,
            tray: "red".to_string(),
            overlay: "pulse_red".to_string(),
            popup_header: "red".to_string(),
            notify: Some("toast".to_string()),
        }).collect();
        let mut policy = CommunicationPolicy::new(comm, ErgonomicProfile::default());
        let start = Instant::now();
        let evaluate = |policy: &mut CommunicationPolicy, step: i64, elapsed: u64| {
            policy.evaluate_with_stand_limit_fired_at(
                &sitting_input(2400 + step * 10),
                2400,
                0,
                false,
                start + Duration::from_secs(elapsed),
            )
        };

        assert!(evaluate(&mut policy, 0, 0).notify.is_some());
        assert!(evaluate(&mut policy, 1, 299).notify.is_none());
        assert!(evaluate(&mut policy, 1, 300).notify.is_some());
        assert!(evaluate(&mut policy, 2, 1199).notify.is_none());
        assert!(evaluate(&mut policy, 2, 1200).notify.is_some());
        assert!(evaluate(&mut policy, 3, 2999).notify.is_none());
        assert!(evaluate(&mut policy, 3, 3000).notify.is_some());
        let final_signals = evaluate(&mut policy, 4, 20_000);
        assert!(final_signals.notify.is_none(), "four reminders end in silence");
        assert!(matches!(final_signals.overlay, crate::communication_types::OverlaySignal::PulseRed { .. }));
    }

    #[test]
    fn injected_dismiss_snooze_expiry_and_position_reset_are_deterministic() {
        let mut policy = default_policy();
        let start = Instant::now();
        let evaluate = |policy: &mut CommunicationPolicy, elapsed: u64| {
            policy.evaluate_with_stand_limit_fired_at(
                &sitting_input(2400), 2400, 0, false, start + Duration::from_secs(elapsed),
            )
        };

        assert!(evaluate(&mut policy, 0).notify.is_some());
        policy.dismiss_at(start);
        assert!(policy.is_snoozed_at(start + Duration::from_secs(299)));
        assert!(evaluate(&mut policy, 299).notify.is_none());
        assert!(evaluate(&mut policy, 300).notify.is_some());

        policy.snooze_for_at(2, start + Duration::from_secs(400));
        assert!(policy.is_snoozed_at(start + Duration::from_secs(519)));
        assert!(!policy.is_snoozed_at(start + Duration::from_secs(520)));
        assert!(evaluate(&mut policy, 520).notify.is_some());

        policy.on_position_changed();
        assert!(evaluate(&mut policy, 521).notify.is_some(), "position reset starts a fresh schedule");
    }

    #[test]
    fn valid_profile_reload_uses_new_schedule_without_resetting_session_limit() {
        let mut policy = default_policy();
        let start = Instant::now();
        let input = sitting_input(2400);
        assert!(policy.evaluate_with_stand_limit_fired_at(&input, 2400, 0, false, start).notify.is_some());

        let mut reloaded = CommunicationProfile::default();
        reloaded.snooze.notify_cooldowns_secs = vec![0, 120];
        policy.set_comm_profile(reloaded);

        let at_119 = start + Duration::from_secs(119);
        let at_120 = start + Duration::from_secs(120);
        assert!(policy.evaluate_with_stand_limit_fired_at(&input, 2400, 0, false, at_119).notify.is_none());
        assert!(policy.evaluate_with_stand_limit_fired_at(&input, 2400, 0, false, at_120).notify.is_some());
    }

    #[test]
    fn same_id_profile_reload_preserves_an_active_snooze() {
        let mut policy = default_policy();
        let start = Instant::now();
        policy.snooze_for_at(5, start);
        let mut reloaded = CommunicationProfile::default();
        reloaded.name = "Edited default".into();

        policy.set_comm_profile(reloaded);

        assert!(policy.is_snoozed_at(start + Duration::from_secs(299)));
        let signals = policy.evaluate_with_stand_limit_fired_at(
            &sitting_input(2400), 2400, 0, false, start + Duration::from_secs(299),
        );
        assert!(signals.notify.is_none(), "same-id reload must preserve the active snooze");
    }

    #[test]
    fn same_id_reload_uses_the_new_ladder_channel_on_a_visual_only_step() {
        let mut policy = default_policy();
        let start = Instant::now();
        let input = sitting_input(2700);
        assert!(matches!(
            policy.evaluate_with_stand_limit_fired_at(&input, 2400, 0, false, start).notify,
            Some(NotifySignal::Toast(_))
        ));

        let mut reloaded = CommunicationProfile::default();
        reloaded.escalation.sitting = vec![
            EscalationStep {
                at: 0,
                tray: "yellow".into(),
                overlay: "yellow".into(),
                popup_header: "yellow".into(),
                notify: Some("popup".into()),
            },
            EscalationStep {
                at: 300,
                tray: "red".into(),
                overlay: "pulse_red".into(),
                popup_header: "red".into(),
                notify: None,
            },
        ];
        policy.set_comm_profile(reloaded);

        let signals = policy.evaluate_with_stand_limit_fired_at(
            &input,
            2400,
            0,
            false,
            start + Duration::from_secs(300),
        );

        assert!(matches!(signals.notify, Some(NotifySignal::Popup(_))));
    }

    #[test]
    fn same_id_reload_with_all_notification_steps_disabled_drops_cached_channel() {
        let mut policy = default_policy();
        let start = Instant::now();
        let input = sitting_input(2700);
        assert!(policy.evaluate_with_stand_limit_fired_at(&input, 2400, 0, false, start).notify.is_some());

        let mut reloaded = CommunicationProfile::default();
        reloaded.escalation.sitting = vec![EscalationStep {
            at: 0,
            tray: "red".into(),
            overlay: "pulse_red".into(),
            popup_header: "red".into(),
            notify: None,
        }];
        policy.set_comm_profile(reloaded);

        let signals = policy.evaluate_with_stand_limit_fired_at(
            &input,
            2400,
            0,
            false,
            start + Duration::from_secs(1800),
        );

        assert!(signals.notify.is_none(), "same-id reload must not reuse the old channel");
    }

    // ── 3. Position change fires fresh after max was reached ────────────────

    #[test]
    fn position_change_resets_after_max() {
        let mut p = default_policy();
        // Fire first notification
        let sig = p.evaluate(&sitting_input(2400));
        assert!(sig.notify.is_some());

        // Simulate "max reached" by resetting and firing multiple times
        for _ in 0..3 {
            p.on_position_changed();
            let _ = p.evaluate(&sitting_input(2400));
        }

        // After position change, count resets → fires again
        p.on_position_changed();
        let sig = p.evaluate(&sitting_input(2400));
        assert!(sig.notify.is_some(), "fresh after position change");
    }

    // ── 4. Dismiss resets notify_count for fresh schedule ───────────────────

    #[test]
    fn dismiss_resets_notify_count() {
        let mut p = default_policy();
        let sig = p.evaluate(&sitting_input(2400));
        assert!(sig.notify.is_some());

        p.dismiss();
        p.on_position_changed();

        let sig2 = p.evaluate(&sitting_input(2400));
        assert!(sig2.notify.is_some(), "fresh count after dismiss+reset");
    }

    // ── 5. Cooldown blocks rapid re-fire at same step ───────────────────────

    #[test]
    fn cooldown_blocks_same_step_refire() {
        let mut p = default_policy();
        let sig1 = p.evaluate(&sitting_input(2400));
        assert!(sig1.notify.is_some());

        let sig2 = p.evaluate(&sitting_input(2400));
        assert!(sig2.notify.is_none(), "same step must not re-fire");
    }

    // ── 6. Custom cooldown profile respected ────────────────────────────────

    #[test]
    fn empty_cooldown_means_no_notifications() {
        let mut comm = CommunicationProfile::default();
        comm.snooze.notify_cooldowns_secs = vec![];
        let mut p = CommunicationPolicy::new(comm, ErgonomicProfile::default());

        let sig = p.evaluate(&sitting_input(2400));
        assert!(sig.notify.is_none(), "empty cooldowns = silent profile");
    }

    #[test]
    fn single_cooldown_fires_once_then_silence() {
        let mut comm = CommunicationProfile::default();
        comm.snooze.notify_cooldowns_secs = vec![0];
        let mut p = CommunicationPolicy::new(comm, ErgonomicProfile::default());

        let sig1 = p.evaluate(&sitting_input(2400));
        assert!(matches!(sig1.notify, Some(NotifySignal::Toast(_))));

        // After one fire, no more entries in cooldown → silence
        p.on_position_changed();
        let _ = p.evaluate(&sitting_input(2400)); // fires (count=1)
        // Now count=1, cooldown vec has only index 0 → None
        // But position_changed resets count to 0, so it fires again.
        // Test the actual "exhausted" case without reset:
        let mut comm2 = CommunicationProfile::default();
        comm2.snooze.notify_cooldowns_secs = vec![0];
        comm2.escalation.sitting = (0..3).map(|i| EscalationStep {
            at: -100 + (i * 10),
            tray: "red".to_string(),
            overlay: "red".to_string(),
            popup_header: "red".to_string(),
            notify: Some("toast".to_string()),
        }).collect();
        let mut p2 = CommunicationPolicy::new(comm2, ErgonomicProfile::default());

        let s1 = p2.evaluate(&sitting_input(2300));
        assert!(s1.notify.is_some(), "first fire");

        // Step 1 — count is now 1, cooldown[1] doesn't exist → None
        let s2 = p2.evaluate(&sitting_input(2310));
        assert!(s2.notify.is_none(), "exhausted after 1 fire");
    }
}
