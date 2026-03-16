//! tray_controller.rs — Wires `desk:state-changed` events to tray + overlay.
//!
//! Listens for [`StateChangedPayload`] events emitted by `serial.rs` and
//! updates the tray icon colour/tooltip and the overlay progress bar to
//! reflect the current session state.

use log::info;
use tauri::{AppHandle, Listener};

use crate::{
    overlay,
    session::{DeskState, StateChangedPayload},
    tray,
};

// ─── Colour thresholds ────────────────────────────────────────────────────────

/// 0–60 % of session limit → green.
const THRESHOLD_YELLOW: f32 = 0.60;
/// 60–85 % → yellow.
const THRESHOLD_RED: f32 = 0.85;

// ─── Public API ───────────────────────────────────────────────────────────────

/// Registers the `desk:state-changed` event listener.
///
/// Call once from `lib.rs` setup.
pub fn setup(app: &AppHandle) {
    let handle = app.clone();
    app.listen("desk:state-changed", move |event| {
        if let Ok(payload) = serde_json::from_str::<StateChangedPayload>(event.payload()) {
            on_state_changed(&handle, &payload);
        }
    });
}

// ─── Private helpers ──────────────────────────────────────────────────────────

/// Reacts to a state-change event by updating tray and overlay.
fn on_state_changed(app: &AppHandle, payload: &StateChangedPayload) {
    // Read session limit from the managed state.
    let session_limit_secs = {
        use crate::commands::AppState;
        use tauri::Manager;
        app.state::<AppState>()
            .session
            .lock()
            .unwrap()
            .snapshot()
            .session_limit_secs
    };

    let sitting_secs = payload.sitting_seconds;
    let progress = if session_limit_secs > 0 {
        sitting_secs as f32 / session_limit_secs as f32
    } else {
        0.0
    };

    let (_r, _g, _b, css_color) = color_for_progress(progress);

    // Build tooltip label: "↕ 72.3 cm — Sitting (12:34)"
    let state_str = match payload.state {
        DeskState::Sitting => "Sitting",
        DeskState::Standing => "Standing",
        DeskState::Walking => "Walking",
        DeskState::Away => "Away",
    };
    let label = format!(
        "↕ {:.1} cm — {} ({})",
        payload.desk_height_cm,
        state_str,
        format_duration(sitting_secs),
    );

    let _ = tray::update_tray(app, &label, payload.state.clone(), progress);

    if payload.state == DeskState::Sitting {
        info!("→ Showing overlay, emitting progress: {} color: {}", progress, css_color);
        overlay::update_overlay(app, progress, css_color);
        overlay::show_overlay(app);
    } else {
        info!("→ Hiding overlay (state: {:?})", payload.state);
        overlay::hide_overlay(app);
    }
}

/// Returns (r, g, b, css_hex) for a given progress ratio.
fn color_for_progress(progress: f32) -> (u8, u8, u8, &'static str) {
    if progress < THRESHOLD_YELLOW {
        (76, 175, 80, "#4caf50") // green
    } else if progress < THRESHOLD_RED {
        (255, 193, 7, "#ffc107") // yellow
    } else {
        (244, 67, 54, "#f44336") // red
    }
}

/// Formats a duration in seconds as `"MM:SS"`.
fn format_duration(secs: i64) -> String {
    let secs = secs.max(0);
    let minutes = secs / 60;
    let seconds = secs % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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
    fn color_green_below_60_percent() {
        let (r, g, b, css) = color_for_progress(0.3);
        assert_eq!((r, g, b), (76, 175, 80));
        assert_eq!(css, "#4caf50");
    }

    #[test]
    fn color_yellow_between_60_and_85() {
        let (r, g, b, css) = color_for_progress(0.7);
        assert_eq!((r, g, b), (255, 193, 7));
        assert_eq!(css, "#ffc107");
    }

    #[test]
    fn color_red_above_85_percent() {
        let (r, g, b, css) = color_for_progress(0.9);
        assert_eq!((r, g, b), (244, 67, 54));
        assert_eq!(css, "#f44336");
    }
}
