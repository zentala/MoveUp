//! tray_icon.rs — T010 Dynamic tray icon by session state.
//!
//! This module provides the public interface for T010 — dynamic tray icon selection
//! based on session progress and sitting state.

use crate::session::DeskState;

// ─── Icon selection (T010) ────────────────────────────────────────────────────

/// Selects the appropriate PNG tray icon name based on sitting state and progress.
///
/// Returns the icon file name (without `.png` extension):
/// - `tray-ok` (green) — sitting, <60% of session limit
/// - `tray-warn` (amber) — sitting, 60-85% of limit
/// - `tray-alert` (red) — sitting, >85% of limit
/// - `tray-idle` (gray) — standing, walking, or away
pub fn icon_for_state_and_progress(state: DeskState, ratio: f32) -> &'static str {
    const THRESHOLD_YELLOW: f32 = 0.60;
    const THRESHOLD_RED: f32 = 0.85;

    match state {
        DeskState::Sitting => {
            if ratio >= THRESHOLD_RED {
                "tray-alert"
            } else if ratio >= THRESHOLD_YELLOW {
                "tray-warn"
            } else {
                "tray-ok"
            }
        }
        _ => "tray-idle", // Standing, Walking, Away
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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
    fn standing_returns_idle() {
        assert_eq!(icon_for_state_and_progress(DeskState::Standing, 0.5), "tray-idle");
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
    fn idle_state_returns_idle_regardless_of_progress() {
        assert_eq!(icon_for_state_and_progress(DeskState::Standing, 0.0), "tray-idle");
        assert_eq!(icon_for_state_and_progress(DeskState::Standing, 1.0), "tray-idle");
    }
}
