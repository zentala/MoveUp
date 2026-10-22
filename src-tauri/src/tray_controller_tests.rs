//! tray_controller_tests.rs — Unit tests for tooltip formatting and standing progress.

use crate::tray::tray_menu_item_ids;
use crate::tray_helpers::{build_tooltip_label, format_duration};
use crate::session::DeskState;

// ─── Tray menu composition tests ──────────────────────────────────────────────

#[test]
fn tray_menu_includes_open_analyst_item() {
    assert!(tray_menu_item_ids().contains(&"open-analyst"));
}

#[test]
fn tray_menu_open_analyst_between_settings_and_quit() {
    let ids = tray_menu_item_ids();
    let settings = ids.iter().position(|&id| id == "settings").expect("settings present");
    let analyst = ids.iter().position(|&id| id == "open-analyst").expect("open-analyst present");
    let quit = ids.iter().position(|&id| id == "quit").expect("quit present");
    assert!(settings < analyst, "settings must come before open-analyst");
    assert!(analyst < quit, "open-analyst must come before quit");
}

#[test]
fn format_duration_zero() {
    assert_eq!(format_duration(0), "00:00");
}

#[test]
fn format_duration_one_minute() {
    assert_eq!(format_duration(60), "01:00");
}

#[test]
fn format_duration_mixed() {
    assert_eq!(format_duration(754), "12:34");
}

#[test]
fn format_duration_negative_clamps() {
    assert_eq!(format_duration(-10), "00:00");
}

#[test]
fn tooltip_sitting_shows_sitting_seconds() {
    let label = build_tooltip_label(72.0, &DeskState::Sitting, 754, 0, 0, 0.0);
    assert_eq!(label, "\u{2195} 72 cm \u{2014} Sitting (12:34) +0");
}

#[test]
fn tooltip_standing_shows_standing_seconds() {
    let label = build_tooltip_label(114.0, &DeskState::Standing, 120, 452, 452, 38.0);
    assert_eq!(label, "\u{2195} 114 cm \u{2014} Standing (07:32) +38");
}

#[test]
fn tooltip_walking_shows_break_seconds() {
    let label = build_tooltip_label(114.0, &DeskState::Walking, 300, 200, 95, -8.0);
    assert_eq!(label, "\u{2195} 114 cm \u{2014} Walking (01:35) -8");
}

#[test]
fn tooltip_away_shows_zero() {
    let label = build_tooltip_label(0.0, &DeskState::Away, 500, 200, 100, 0.0);
    assert_eq!(label, "\u{2195} 0 cm \u{2014} Away (00:00) +0");
}

// ─── PolicyInput is built by the engine (E020-T05) ───────────────────────────
//
// These used to recompute the lap arithmetic inline and assert the result
// against itself — they passed no matter what `tray_controller` did. They now
// drive `SessionManager::policy_input`, the one place the values are derived.

mod policy_input {
    use chrono::{DateTime, Duration, TimeZone, Utc};

    use crate::communication_policy::CommunicationPolicy;
    use crate::communication_profile::{CommunicationProfile, EscalationStep};
    use crate::communication_types::NotifySignal;
    use crate::ergonomic_profile::ErgonomicProfile;
    use crate::session_manager::SessionManager;
    use crate::session_types::DeskState;

    const TARGET: i64 = 900;

    /// A fixed instant, so nothing here depends on when the suite runs.
    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 1, 9, 0, 0).unwrap()
    }

    fn observe(
        manager: &mut SessionManager,
        height_mm: i32,
        active: bool,
        from: DateTime<Utc>,
        through: DateTime<Utc>,
    ) {
        let mut at = from;
        while at <= through {
            manager.on_reading_at(height_mm, active, at);
            at += Duration::seconds(1);
        }
    }

    fn sitting(prior_credited_secs: i64, current_bout_secs: i64) -> SessionManager {
        let mut manager = SessionManager::new();
        manager.state.sitting_seconds = prior_credited_secs;
        observe(
            &mut manager,
            800,
            true,
            now() - Duration::seconds(current_bout_secs + 5),
            now(),
        );
        assert_eq!(manager.state.state, DeskState::Sitting);
        manager
    }

    /// Standing for `bout` seconds, with `standing_today` already banked.
    fn standing(bout: i64, standing_today: i64) -> SessionManager {
        let mut m = SessionManager::new();
        m.state.stand_limit_secs = TARGET;
        m.state.standing_seconds = standing_today - bout;
        observe(
            &mut m,
            1200,
            true,
            now() - Duration::seconds(bout + 5),
            now(),
        );
        assert_eq!(m.state.state, DeskState::Standing);
        m
    }

    #[test]
    fn standing_progress_at_zero() {
        let input = standing(0, 0).policy_input(now(), true);
        assert_eq!(input.elapsed_secs, 0);
        assert!(input.standing_lap_progress.abs() < 0.01);
        assert_eq!(input.standing_lap, 0);
        assert!(!input.standing_lap_flash);
    }

    #[test]
    fn standing_progress_at_half() {
        let input = standing(450, 450).policy_input(now(), true);
        assert_eq!(input.elapsed_secs, 450, "standing elapsed is the current bout");
        assert!((input.standing_lap_progress - 0.5).abs() < 0.01);
        assert_eq!(input.standing_lap, 0);
        assert!(!input.standing_lap_flash);
    }

    #[test]
    fn standing_progress_at_target_fires_flash() {
        let input = standing(TARGET, TARGET).policy_input(now(), true);
        assert!(input.standing_lap_progress.abs() < 0.01, "wraps back to 0");
        assert_eq!(input.standing_lap, 1, "one lap completed today");
        assert!(input.standing_lap_flash, "crossing the target flashes");
    }

    #[test]
    fn standing_new_session_after_sit_no_immediate_flash() {
        // 300 s into a fresh bout, with a full lap already banked earlier today.
        let input = standing(300, 1200).policy_input(now(), true);
        assert!((input.standing_lap_progress - 0.333).abs() < 0.01);
        assert_eq!(input.standing_lap, 1, "yesterday's lap still counts today");
        assert!(!input.standing_lap_flash, "a new bout must not re-flash");
    }

    /// empty — a profile with no standing target must not divide by zero.
    #[test]
    fn zero_stand_limit_yields_no_lap() {
        let mut m = standing(450, 450);
        m.state.stand_limit_secs = 0;
        let input = m.policy_input(now(), true);
        assert!(input.standing_lap_progress.abs() < f32::EPSILON);
        assert_eq!(input.standing_lap, 0);
        assert!(!input.standing_lap_flash);
    }

    /// happy — sitting reports the CREDITED counter, the one E015 made canonical.
    #[test]
    fn sitting_elapsed_is_the_credited_counter() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Sitting;
        m.state.sitting_seconds = 600;
        observe(
            &mut m,
            800,
            true,
            now() - Duration::seconds(125),
            now(),
        );

        let input = m.policy_input(now(), true);
        assert_eq!(input.elapsed_secs, 720);
        assert_eq!(input.standing_lap, 0, "not standing — no lap");
    }

    /// Standing maximum uses actual standing time, not the longer break clock
    /// that continues through Away and Walking.
    #[test]
    fn standing_maximum_uses_standing_session_duration() {
        let mut m = SessionManager::new();
        m.state.stand_limit_secs = 600;
        let break_start = now() - Duration::seconds(3600);
        observe(&mut m, 1200, true, break_start - Duration::seconds(5), break_start);
        observe(&mut m, 1200, true, break_start + Duration::seconds(1), break_start + Duration::seconds(5));
        observe(&mut m, 1200, false, break_start + Duration::seconds(6), now() - Duration::seconds(305));
        observe(&mut m, 1200, true, now() - Duration::seconds(305), now());
        assert_eq!(m.state.state, DeskState::Standing);

        let input = m.policy_input(now(), true);
        let standing_secs = m.get_live_standing_session_secs(now());
        assert_eq!(input.elapsed_secs, 3600, "break progress clock remains unchanged");
        assert_eq!(standing_secs, 300, "maximum uses actual standing time");
        assert_eq!(input.standing_lap, 0);

        let mut ergonomic = ErgonomicProfile::default();
        ergonomic.limits.standing_max_secs = 600;
        let mut policy = CommunicationPolicy::new(CommunicationProfile::default(), ergonomic);
        let signals = policy.evaluate_with_stand_limit_fired(&input, m.state.session_limit_secs, standing_secs, false);
        assert!(signals.accepted_events.is_empty(), "300 standing seconds is below the 600 second maximum despite the longer break clock");
        assert!(signals.notify.is_none());
    }

    /// nil — neither sitting nor standing: no elapsed time to escalate on.
    #[test]
    fn away_has_no_elapsed_time() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Away;
        m.state.sitting_seconds = 3000;

        let input = m.policy_input(now(), true);
        assert_eq!(input.elapsed_secs, 0);
        assert_eq!(input.state, DeskState::Away);
    }

    /// The one fact the engine cannot know is passed in, not derived.
    #[test]
    fn sensor_connectivity_comes_from_the_caller() {
        let m = SessionManager::new();
        assert!(!m.policy_input(now(), false).sensor_connected);
        assert!(m.policy_input(now(), true).sensor_connected);
    }

    fn sitting_signal_for_limits(profile_limit_secs: u32, user_limit_minutes: u32, elapsed_secs: i64) -> Option<NotifySignal> {
        let mut session = sitting(0, elapsed_secs);
        session.set_limit_minutes(user_limit_minutes);

        let mut ergonomic = ErgonomicProfile::default();
        ergonomic.limits.sitting_secs = profile_limit_secs;
        session.set_ergo_profile(&ergonomic);
        assert_eq!(
            session.state.session_limit_secs,
            i64::from(user_limit_minutes) * 60,
            "profile reload must preserve the runtime session limit",
        );

        let mut communication = CommunicationProfile::default();
        communication.escalation.sitting = vec![EscalationStep {
            at: 0,
            tray: "none".into(),
            overlay: "hidden".into(),
            popup_header: "neutral".into(),
            notify: Some("toast".into()),
        }];
        let input = session.policy_input(now(), true);
        CommunicationPolicy::new(communication, ergonomic)
            .evaluate_with_stand_limit_fired(&input, session.state.session_limit_secs, 0, false)
            .notify
    }

    #[test]
    fn sitting_user_limit_below_profile_fires_at_runtime_limit() {
        assert!(matches!(sitting_signal_for_limits(2_400, 30, 1_800), Some(NotifySignal::Toast(_))));
    }

    #[test]
    fn sitting_user_limit_above_profile_waits_for_runtime_limit() {
        assert!(sitting_signal_for_limits(1_800, 40, 2_100).is_none());
    }

    #[test]
    fn zero_sitting_user_limit_keeps_immediate_alert_semantics() {
        assert!(matches!(sitting_signal_for_limits(1_800, 0, 0), Some(NotifySignal::Toast(_))));
    }

    #[test]
    fn policy_notification_dispatch_logs_channels_and_webhook_once_then_suppresses() {
        use crate::notification_service::{NotificationChannel, NotificationService};

        let mut session = sitting(0, 1_800);
        session.set_limit_minutes(30);

        let mut ergonomic = ErgonomicProfile::default();
        ergonomic.limits.sitting_secs = 2_400;
        session.set_ergo_profile(&ergonomic);
        let input = session.policy_input(now(), true);

        for (notify_type, expected_channel, expected_webhook) in [
            ("toast", NotificationChannel::Toast, true),
            ("popup", NotificationChannel::Popup, false),
        ] {
            let mut communication = CommunicationProfile::default();
            communication.escalation.sitting = vec![EscalationStep {
                at: 0,
                tray: "none".into(),
                overlay: "hidden".into(),
                popup_header: "neutral".into(),
                notify: Some(notify_type.into()),
            }];
            let mut policy = CommunicationPolicy::new(communication, ergonomic.clone());
            let first = policy.evaluate_with_stand_limit_fired(
                &input,
                session.state.session_limit_secs,
                0,
                false,
            );
            let second = policy.evaluate_with_stand_limit_fired(
                &input,
                session.state.session_limit_secs,
                0,
                false,
            );
            assert!(first.notify.is_some());
            assert!(second.notify.is_none(), "the accepted sitting step is one-shot");

            let mut log_count = 0;
            let mut channels = Vec::new();
            let mut webhook_bodies = Vec::new();
            for notify in [first.notify.as_ref(), second.notify.as_ref()] {
                crate::tray_controller::dispatch_notification_with(
                    notify,
                    |signal| NotificationService::dispatch_policy_signal_with(
                        signal,
                        || log_count += 1,
                        |channel, title, body| channels.push((channel, title.to_string(), body.to_string())),
                    ),
                    |signal| {
                        if let NotifySignal::Toast(body) = signal {
                            webhook_bodies.push(body.clone());
                        }
                    },
                );
            }

            assert_eq!(log_count, 1);
            assert_eq!(channels.len(), 1);
            assert_eq!(channels[0].0, expected_channel);
            assert_eq!(channels[0].1, "Smart Desk");
            assert_eq!(webhook_bodies.len(), usize::from(expected_webhook));
            if expected_webhook {
                assert_eq!(webhook_bodies, vec![channels[0].2.clone()]);
            }
        }
    }

    #[test]
    fn resolved_intent_dispatch_uses_toast_popup_both_and_suppression_is_silent() {
        use crate::notification_service::{NotificationChannel, NotificationService};
        use crate::session_types::NotificationEvent;

        let mut facts = crate::communication_policy::SessionNotificationFacts {
            state: DeskState::Standing,
            now: now(),
            observation_fresh: true,
            last_position_change_at: None,
            sitting_seconds_total: 0,
            standing_seconds: 600,
            standing_target_secs: 600,
            inactivity_fired: false,
            posture_balance_fired: false,
            standing_target_fired: false,
            praise_fired: false,
        };

        for (backend, expected_channels) in [
            ("toast", vec![NotificationChannel::Toast]),
            ("popup", vec![NotificationChannel::Popup]),
            ("both", vec![NotificationChannel::Toast, NotificationChannel::Popup]),
        ] {
            let mut communication = CommunicationProfile::default();
            communication.notification_backend = backend.into();
            let policy = CommunicationPolicy::new(communication, ErgonomicProfile::default());
            let intents = policy.resolve_session_notifications(&facts, true);
            assert_eq!(intents.len(), 1);
            assert!(matches!(intents[0].event, NotificationEvent::StandingTargetReached));

            let mut logs = Vec::new();
            let mut executed_channels = Vec::new();
            NotificationService::dispatch_with(
                &intents,
                |line| logs.push(line.to_string()),
                |channels, _title, _body| executed_channels.extend_from_slice(channels),
            );
            assert_eq!(logs, vec!["NOTIF standing_target_reached"]);
            assert_eq!(executed_channels, expected_channels);

            facts.standing_target_fired = true;
            let suppressed = policy.resolve_session_notifications(&facts, true);
            assert!(suppressed.is_empty());
            logs.clear();
            executed_channels.clear();
            NotificationService::dispatch_with(
                &suppressed,
                |line| logs.push(line.to_string()),
                |channels, _title, _body| executed_channels.extend_from_slice(channels),
            );
            assert!(logs.is_empty());
            assert!(executed_channels.is_empty());
            facts.standing_target_fired = false;
        }
    }
}

mod notification_intent_count {
    use crate::communication_policy::{CommunicationPolicy, PolicyInput};
    use crate::communication_profile::{CommunicationProfile, EscalationStep};
    use crate::communication_types::NotifySignal;
    use crate::ergonomic_profile::ErgonomicProfile;
    use crate::session::DeskState;

    fn input(state: DeskState, elapsed_secs: i64, sensor_connected: bool) -> PolicyInput {
        PolicyInput {
            state,
            elapsed_secs,
            sensor_connected,
            observation_fresh: true,
            standing_lap_progress: 0.0,
            standing_lap: 0,
            standing_lap_flash: false,
            continuous_computer_secs: 0,
        }
    }

    fn policy() -> CommunicationPolicy {
        let mut comm = CommunicationProfile::default();
        comm.escalation.sitting = vec![EscalationStep {
            at: 0,
            tray: "red".into(),
            overlay: "red".into(),
            popup_header: "red".into(),
            notify: Some("toast".into()),
        }];
        comm.snooze.notify_cooldowns_secs = vec![0];
        let mut ergo = ErgonomicProfile::default();
        ergo.limits.sitting_secs = 2400;
        CommunicationPolicy::new(comm, ergo)
    }

    #[test]
    fn first_uninterrupted_sit_limit_resolves_once() {
        let mut policy = policy();
        let input = input(DeskState::Sitting, 2400, true);

        let first = policy.evaluate(&input);
        let repeated_tick = policy.evaluate(&input);

        assert!(matches!(first.notify, Some(NotifySignal::Toast(_))));
        assert_eq!(usize::from(first.notify.is_some()), 1);
        assert_eq!(usize::from(repeated_tick.notify.is_some()), 0);
    }

    #[test]
    fn short_noncredited_break_return_has_one_policy_intent_per_sitting_stint() {
        let mut policy = policy();
        let sitting_at_limit = input(DeskState::Sitting, 2400, true);

        let first_stint = policy.evaluate(&sitting_at_limit);
        policy.on_position_changed(); // Sitting -> short break.
        policy.on_position_changed(); // Short break -> Sitting; credited load is unchanged.
        let returned_stint = policy.evaluate(&sitting_at_limit);

        assert!(matches!(first_stint.notify, Some(NotifySignal::Toast(_))));
        assert!(matches!(returned_stint.notify, Some(NotifySignal::Toast(_))));
        assert_eq!(
            usize::from(first_stint.notify.is_some()) + usize::from(returned_stint.notify.is_some()),
            2,
            "each sitting stint resolves one intent; no second producer is included"
        );
    }

    #[test]
    fn sensor_disconnect_intent_is_one_shot_and_reconnect_rearms_it() {
        let mut policy = policy();
        let disconnected = input(DeskState::Sitting, 0, false);

        let first_loss = policy.evaluate(&disconnected);
        let repeated_loss_tick = policy.evaluate(&disconnected);
        policy.on_sensor_connected();
        let connected_tick = policy.evaluate(&input(DeskState::Sitting, 0, true));
        let second_loss = policy.evaluate(&disconnected);

        assert!(matches!(first_loss.notify, Some(NotifySignal::Toast(_))));
        assert_eq!(usize::from(repeated_loss_tick.notify.is_some()), 0);
        assert_eq!(usize::from(connected_tick.notify.is_some()), 0);
        assert!(matches!(second_loss.notify, Some(NotifySignal::Toast(_))));
        assert_eq!(
            usize::from(first_loss.notify.is_some()) + usize::from(second_loss.notify.is_some()),
            2
        );
    }
}

#[cfg(test)]
mod notification_policy_route_tests {
    use chrono::Utc;
    use crate::{
        communication_policy::{CommunicationPolicy, PolicyInput, SessionNotificationFacts},
        communication_profile::CommunicationProfile,
        communication_types::NotifySignal,
        ergonomic_profile::ErgonomicProfile,
        session_types::{DeskState, NotificationEvent},
    };

    #[test]
    fn one_policy_resolves_target_maximum_suppression_and_channels() {
        let mut comm = CommunicationProfile::default();
        comm.notification_backend = "popup".into();
        let mut policy = CommunicationPolicy::new(comm, ErgonomicProfile::default());
        let mut facts = SessionNotificationFacts {
            state: DeskState::Standing,
            now: Utc::now(),
            observation_fresh: true,
            last_position_change_at: None,
            sitting_seconds_total: 0,
            standing_seconds: 600,
            standing_target_secs: 600,
            inactivity_fired: false,
            posture_balance_fired: false,
            standing_target_fired: false,
            praise_fired: false,
        };

        let target = policy.resolve_session_notifications(&facts, true);
        assert_eq!(target.len(), 1);
        assert!(matches!(&target[0].event, NotificationEvent::StandingTargetReached));
        assert_eq!(target[0].channels, vec![crate::notification_service::NotificationChannel::Popup]);

        let below_maximum = policy.resolve_session_notifications(&facts, false);
        assert!(below_maximum.iter().all(|intent| !matches!(&intent.event, NotificationEvent::StandLimitReached)));
        let max_secs = ErgonomicProfile::default().limits.standing_max_secs as i64;
        let maximum = policy.evaluate_with_stand_limit_fired(&PolicyInput {
            state: DeskState::Standing,
            elapsed_secs: max_secs,
            sensor_connected: true,
            observation_fresh: true,
            standing_lap_progress: 1.0,
            standing_lap: 0,
            standing_lap_flash: false,
            continuous_computer_secs: 0,
        }, ErgonomicProfile::default().limits.sitting_secs as i64, max_secs, false);
        assert!(matches!(maximum.notify, Some(NotifySignal::Toast(_))));
        assert!(matches!(maximum.accepted_events.as_slice(), [NotificationEvent::StandLimitReached]));
        facts.praise_fired = true;
        let after_maximum = policy.resolve_session_notifications(&facts, false);
        assert!(after_maximum.iter().all(|intent| !matches!(&intent.event, NotificationEvent::StandLimitReached)));
        assert_eq!(usize::from(maximum.notify.is_some()), 1);

        facts.standing_target_fired = true;
        assert!(policy.resolve_session_notifications(&facts, true).is_empty());
    }

    #[test]
    fn accepted_intentless_maximum_persists_once_without_dispatch() {
        let mut comm = CommunicationProfile::default();
        for step in &mut comm.escalation.standing { step.notify = None; }
        let mut policy = CommunicationPolicy::new(comm, ErgonomicProfile::default());
        let max_secs = ErgonomicProfile::default().limits.standing_max_secs as i64;
        let input = PolicyInput {
            state: DeskState::Standing,
            elapsed_secs: max_secs,
            sensor_connected: true,
            observation_fresh: true,
            standing_lap_progress: 1.0,
            standing_lap: 0,
            standing_lap_flash: false,
            continuous_computer_secs: 0,
        };

        let accepted = policy.evaluate_with_stand_limit_fired(
            &input,
            ErgonomicProfile::default().limits.sitting_secs as i64,
            max_secs,
            false,
        );
        assert_eq!(accepted.accepted_events.len(), 1);
        assert!(accepted.notify.is_none(), "accepted event has zero notification dispatches");

        let mut session = crate::session_manager::SessionManager::new();
        session.record_notification_events(&accepted.accepted_events);
        let persisted = crate::session_persistence::PersistedEngineState::from_session(&session, None);
        assert!(persisted.stand_alert_fired);

        let repeated = policy.evaluate_with_stand_limit_fired(
            &input,
            ErgonomicProfile::default().limits.sitting_secs as i64,
            max_secs,
            persisted.stand_alert_fired,
        );
        assert!(repeated.accepted_events.is_empty());
        assert!(repeated.notify.is_none());
    }
}
