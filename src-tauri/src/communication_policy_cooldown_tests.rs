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
        }
    }

    // ── 1. Notification count increments and resets ──────────────────────────

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
