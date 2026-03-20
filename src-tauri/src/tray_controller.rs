//! tray_controller.rs — Wires `desk:state-changed` events to tray + overlay.
//!
//! Listens for [`StateChangedPayload`] events emitted by `serial.rs` and
//! updates the tray icon colour/tooltip and the overlay progress bar to
//! reflect the current session state.
//!
//! Also drives the [`AlertManager`] state machine and executes returned
//! [`AlertAction`]s (pulse bar, show/dismiss popup).

use log::info;
use tauri::{AppHandle, Listener};

use crate::{
    alert_manager::AlertAction,
    colors::color_for_progress,
    session::{DeskState, StateChangedPayload},
    tray,
};

// ─── Public API ───────────────────────────────────────────────────────────────

/// Registers event listeners for tray + overlay + alert updates.
///
/// - `desk:state-changed` — state transitions (show/hide overlay, update tray, alert reset)
/// - `desk:distance` — every sensor reading (update overlay progress + alert tick)
///
/// Call once from `lib.rs` setup.
pub fn setup(app: &AppHandle) {
    // State transitions: update tray icon + show/hide overlay + reset alerts
    let handle = app.clone();
    app.listen("desk:state-changed", move |event| {
        if let Ok(payload) = serde_json::from_str::<StateChangedPayload>(event.payload()) {
            on_state_changed(&handle, &payload);
        }
    });

    // Every sensor reading: update overlay progress + drive alert state machine
    let handle2 = app.clone();
    app.listen("desk:distance", move |_event| {
        update_overlay_progress(&handle2);
    });
}

// ─── Private helpers ──────────────────────────────────────────────────────────

/// Reacts to a state-change event by updating tray, overlay, and alerts.
fn on_state_changed(app: &AppHandle, payload: &StateChangedPayload) {
    use crate::commands::AppState;
    use tauri::Manager;

    // Read session limit, overlay, alert_manager, and alert_popup from managed state.
    let (session_limit_secs, overlay, alert_manager, alert_popup) = {
        let app_state = app.state::<AppState>();
        let session_limit_secs = app_state
            .session
            .lock()
            .unwrap()
            .snapshot()
            .session_limit_secs;
        let overlay = app_state.overlay.clone();
        let alert_manager = app_state.alert_manager.clone();
        let alert_popup = app_state.alert_popup.clone();
        (session_limit_secs, overlay, alert_manager, alert_popup)
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
        log::debug!("→ Showing overlay, progress: {:.0}%", progress * 100.0);
        overlay.update(progress, (r, g, b));
        overlay.show();
    } else {
        log::debug!("→ Hiding overlay (state: {:?})", payload.state);
        overlay.hide();
    }

    // Notify AlertManager of standing — resets escalation state
    if payload.state == DeskState::Standing {
        let actions = alert_manager.lock().unwrap().on_standing();
        execute_alert_actions(&actions, &overlay, &alert_popup);
    }
}

/// Updates overlay progress on every sensor reading, and drives the alert state machine.
///
/// Called from `desk:distance` listener — fires ~every second.
/// Only updates if state is Sitting; otherwise no-op.
fn update_overlay_progress(app: &AppHandle) {
    use crate::commands::AppState;
    use tauri::Manager;

    let app_state = app.state::<AppState>();
    let session = app_state.session.lock().unwrap();
    let snapshot = session.snapshot();

    if snapshot.state != DeskState::Sitting {
        return;
    }

    let progress = if snapshot.session_limit_secs > 0 {
        snapshot.sitting_seconds as f32 / snapshot.session_limit_secs as f32
    } else {
        0.0
    };

    let (r, g, b, _) = color_for_progress(progress);
    let overlay = app_state.overlay.clone();
    let alert_manager = app_state.alert_manager.clone();
    let alert_popup = app_state.alert_popup.clone();
    drop(session); // Release lock before calling overlay or alert_manager

    overlay.update(progress, (r, g, b));

    // Tick the alert state machine and execute any returned actions
    let actions = alert_manager.lock().unwrap().tick(progress);
    execute_alert_actions(&actions, &overlay, &alert_popup);
}

/// Executes a list of [`AlertAction`]s against the overlay and popup.
fn execute_alert_actions(
    actions: &[AlertAction],
    overlay: &crate::overlay_renderer::OverlayRenderer,
    alert_popup: &std::sync::Arc<std::sync::Mutex<crate::alert_popup::AlertPopup>>,
) {
    for action in actions {
        match action {
            AlertAction::PulseBar => overlay.set_variant(2),
            AlertAction::StopPulse => overlay.set_variant(0),
            AlertAction::ShowPopup(msg) => {
                alert_popup.lock().unwrap().show(msg.clone());
            }
            AlertAction::DismissPopup => {
                alert_popup.lock().unwrap().dismiss();
            }
            // Stages 3-5 not implemented — T017
            AlertAction::ExpandOverlay | AlertAction::FullScreenNudge => {}
        }
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
