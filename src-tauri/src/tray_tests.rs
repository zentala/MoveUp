//! tray_tests.rs — Unit tests for tray icon selection logic.

use crate::session::DeskState;
use crate::tray::icon_for_state_and_progress;

#[test]
fn sitting_below_60_percent_returns_ok() {
    assert_eq!(icon_for_state_and_progress(DeskState::Sitting, 0.3), "tray-ok");
}

#[test]
fn sitting_60_to_85_percent_returns_warn() {
    assert_eq!(icon_for_state_and_progress(DeskState::Sitting, 0.7), "tray-warn");
}

#[test]
fn sitting_above_85_percent_returns_alert() {
    assert_eq!(icon_for_state_and_progress(DeskState::Sitting, 0.9), "tray-alert");
}

#[test]
fn sitting_exactly_60_percent_returns_warn() {
    assert_eq!(icon_for_state_and_progress(DeskState::Sitting, 0.60), "tray-warn");
}

#[test]
fn sitting_exactly_85_percent_returns_alert() {
    assert_eq!(icon_for_state_and_progress(DeskState::Sitting, 0.85), "tray-alert");
}

#[test]
fn standing_returns_standing() {
    assert_eq!(icon_for_state_and_progress(DeskState::Standing, 0.5), "tray-standing");
}

#[test]
fn walking_returns_idle() {
    assert_eq!(icon_for_state_and_progress(DeskState::Walking, 0.5), "tray-idle");
}

#[test]
fn away_returns_idle() {
    assert_eq!(icon_for_state_and_progress(DeskState::Away, 0.5), "tray-idle");
}

#[test]
fn standing_returns_standing_regardless_of_progress() {
    assert_eq!(icon_for_state_and_progress(DeskState::Standing, 0.0), "tray-standing");
    assert_eq!(icon_for_state_and_progress(DeskState::Standing, 1.0), "tray-standing");
}
