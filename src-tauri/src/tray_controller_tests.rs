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

    use crate::session_manager::SessionManager;
    use crate::session_types::DeskState;

    const TARGET: i64 = 900;

    /// A fixed instant, so nothing here depends on when the suite runs.
    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 1, 9, 0, 0).unwrap()
    }

    /// Standing for `bout` seconds, with `standing_today` already banked.
    fn standing(bout: i64, standing_today: i64) -> SessionManager {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Standing;
        m.state.stand_limit_secs = TARGET;
        m.state.standing_seconds = standing_today - bout;
        m.state.break_started = Some(now() - Duration::seconds(bout));
        m.state.standing_bout_started = Some(now() - Duration::seconds(bout));
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
        m.state.sitting_started = Some(now() - Duration::seconds(120));

        let input = m.policy_input(now(), true);
        assert_eq!(input.elapsed_secs, 720);
        assert_eq!(input.standing_lap, 0, "not standing — no lap");
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
}
