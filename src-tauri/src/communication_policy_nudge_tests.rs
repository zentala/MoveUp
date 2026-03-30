//! Screen break nudge tests for [`CommunicationPolicy`].
//!
//! Tests verify the nudge fires when standing past the continuous computer
//! time limit, fires only once, resets on position change, and respects
//! the disabled / zero-threshold guards.

#[cfg(test)]
mod tests {
    use crate::communication_policy::{CommunicationPolicy, PolicyInput};
    use crate::communication_profile::CommunicationProfile;
    use crate::communication_types::NotifySignal;
    use crate::ergonomic_profile::ErgonomicProfile;
    use crate::session_types::DeskState;

    fn test_policy() -> CommunicationPolicy {
        let mut ergo = ErgonomicProfile::default();
        ergo.limits.max_continuous_computer_secs = 60; // 60s for testing
        let comm = CommunicationProfile::default();
        CommunicationPolicy::new(comm, ergo)
    }

    fn standing_input(computer_secs: i64) -> PolicyInput {
        PolicyInput {
            state: DeskState::Standing,
            elapsed_secs: 30,
            sensor_connected: true,
            standing_lap_progress: 0.5,
            standing_lap: 0,
            standing_lap_flash: false,
            continuous_computer_secs: computer_secs,
        }
    }

    fn sitting_input(computer_secs: i64) -> PolicyInput {
        PolicyInput {
            state: DeskState::Sitting,
            elapsed_secs: 30,
            sensor_connected: true,
            standing_lap_progress: 0.0,
            standing_lap: 0,
            standing_lap_flash: false,
            continuous_computer_secs: computer_secs,
        }
    }

    #[test]
    fn nudge_fires_when_standing_past_computer_limit() {
        let mut policy = test_policy();
        let signals = policy.evaluate(&standing_input(65));
        assert!(signals.notify.is_some(), "nudge toast should fire");
    }

    #[test]
    fn nudge_does_not_fire_when_sitting() {
        let mut policy = test_policy();
        let signals = policy.evaluate(&sitting_input(65));
        // At 30s elapsed with 2400s sitting limit, no escalation step active
        assert!(signals.notify.is_none(), "sitting has own escalation, no nudge");
    }

    #[test]
    fn nudge_fires_only_once() {
        let mut policy = test_policy();
        let s1 = policy.evaluate(&standing_input(65));
        assert!(s1.notify.is_some(), "first call fires nudge");
        let s2 = policy.evaluate(&standing_input(70));
        assert!(s2.notify.is_none(), "second call should NOT fire");
    }

    #[test]
    fn nudge_resets_on_position_change() {
        let mut policy = test_policy();
        policy.evaluate(&standing_input(65)); // fires nudge
        policy.on_position_changed();
        let s2 = policy.evaluate(&standing_input(65));
        assert!(s2.notify.is_some(), "should fire again after position change");
    }

    #[test]
    fn nudge_does_not_fire_below_threshold() {
        let mut policy = test_policy();
        let signals = policy.evaluate(&standing_input(50)); // below 60s threshold
        assert!(signals.notify.is_none());
    }

    #[test]
    fn nudge_disabled_when_max_is_zero() {
        let mut ergo = ErgonomicProfile::default();
        ergo.limits.max_continuous_computer_secs = 0; // disabled
        let comm = CommunicationProfile::default();
        let mut policy = CommunicationPolicy::new(comm, ergo);
        let signals = policy.evaluate(&standing_input(9999));
        assert!(signals.notify.is_none());
    }

    #[test]
    fn nudge_toast_contains_message_text() {
        let mut policy = test_policy();
        let signals = policy.evaluate(&standing_input(65));
        match signals.notify {
            Some(NotifySignal::Toast(msg)) => {
                assert!(!msg.is_empty(), "toast message should not be empty");
            }
            _ => panic!("expected Toast variant"),
        }
    }

    #[test]
    fn nudge_disabled_in_comm_profile() {
        let mut ergo = ErgonomicProfile::default();
        ergo.limits.max_continuous_computer_secs = 60;
        let mut comm = CommunicationProfile::default();
        comm.screen_break_nudge.enabled = false;
        let mut policy = CommunicationPolicy::new(comm, ergo);
        let signals = policy.evaluate(&standing_input(65));
        assert!(signals.notify.is_none(), "nudge disabled in comm profile");
    }
}
