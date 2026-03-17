//! tray_controller.rs — Wires `desk:state-changed` events to tray + overlay.
//!
//! Listens for [`StateChangedPayload`] events emitted by `serial.rs` and
//! updates the tray icon colour/tooltip and the overlay progress bar to
//! reflect the current session state.

use log::info;
use tauri::{AppHandle, Listener};

use crate::{
    colors::color_for_progress,
    session::{DeskState, StateChangedPayload},
    tray,
};

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
    use crate::commands::AppState;
    use tauri::Manager;

    // Read session limit and overlay from managed state.
    let (session_limit_secs, overlay) = {
        let app_state = app.state::<AppState>();
        let session_limit_secs = app_state
            .session
            .lock()
            .unwrap()
            .snapshot()
            .session_limit_secs;
        let overlay = app_state.overlay.clone();
        (session_limit_secs, overlay)
    };

    let sitting_secs = payload.sitting_seconds;
    let progress = if session_limit_secs > 0 {
        sitting_secs as f32 / session_limit_secs as f32
    } else {
        0.0
    };

    let (r, g, b, _css_color) = color_for_progress(progress);

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

    // Update WinAPI overlay
    if payload.state == DeskState::Sitting {
        info!("→ Showing overlay, progress: {:.0}%", progress * 100.0);
        overlay.update(progress, (r, g, b));
        overlay.show();
    } else {
        info!("→ Hiding overlay (state: {:?})", payload.state);
        overlay.hide();
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

}
