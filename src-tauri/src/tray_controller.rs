//! Wires `desk:state-changed` + `desk:distance` events to tray tooltip,
//! overlay progress bar, and [`AlertManager`] state machine.

use tauri::{AppHandle, Listener};

use crate::{
    alert_manager::AlertAction,
    colors::{color_for_progress, color_for_standing},
    session::{DeskState, StateChangedPayload},
    tray,
};

// ─── Public API ───────────────────────────────────────────────────────────────

/// Registers `desk:state-changed` and `desk:distance` event listeners.
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

    // Read session snapshot, overlay, alert_manager, and alert_popup from managed state.
    let (snapshot, overlay, alert_manager, alert_popup) = {
        let app_state = app.state::<AppState>();
        let snap = app_state.session.lock().unwrap().snapshot();
        let overlay = app_state.overlay.clone();
        let alert_manager = app_state.alert_manager.clone();
        let alert_popup = app_state.alert_popup.clone();
        (snap, overlay, alert_manager, alert_popup)
    };

    let progress = if snapshot.session_limit_secs > 0 {
        payload.sitting_seconds as f32 / snapshot.session_limit_secs as f32
    } else {
        0.0
    };
    let (r, g, b, _css_color) = color_for_progress(progress);

    let label = build_tooltip_label(
        payload.desk_height_cm,
        &payload.state,
        payload.sitting_seconds,
        payload.standing_seconds,
        payload.break_seconds,
        snapshot.daily_score,
    );

    let _ = tray::update_tray(app, &label, payload.state.clone(), progress);

    // Update WinAPI overlay
    if payload.state == DeskState::Sitting {
        log::debug!("→ Showing overlay, progress: {:.0}%", progress * 100.0);
        overlay.clear_standing();
        overlay.update(progress, (r, g, b));
        overlay.show();
    } else {
        log::debug!("→ Hiding overlay (state: {:?})", payload.state);
        overlay.clear_standing();
        overlay.hide();
    }

    // Notify AlertManager of standing — resets escalation state
    if payload.state == DeskState::Standing {
        let actions = alert_manager.lock().unwrap().on_standing();
        execute_alert_actions(&actions, &overlay, &alert_popup);
    }
}

/// Updates overlay progress on every sensor reading and drives the alert state machine.
fn update_overlay_progress(app: &AppHandle) {
    use crate::commands::AppState;
    use tauri::Manager;

    let app_state = app.state::<AppState>();

    // Check if user dismissed the popup — triggers snooze logic in alert_manager
    if app_state.alert_popup.lock().unwrap().take_user_dismissed() {
        let actions = app_state.alert_manager.lock().unwrap().dismiss();
        let overlay = app_state.overlay.clone();
        let alert_popup = app_state.alert_popup.clone();
        execute_alert_actions(&actions, &overlay, &alert_popup);
    }

    let session = app_state.session.lock().unwrap();
    let snapshot = session.snapshot();
    drop(session); // Release lock before calling tray/overlay

    // Update tooltip every second regardless of state
    update_tooltip(app, &snapshot);

    // Standing mode: show gold bar filling over standing_target
    if snapshot.state == DeskState::Standing {
        let target_secs = snapshot.stand_limit_secs;
        if target_secs > 0 {
            // break_seconds = current standing session duration (resets on sit)
            let session_secs = snapshot.break_seconds;
            let session_lap = (session_secs / target_secs) as u32;
            let lap_progress = (session_secs % target_secs) as f32 / target_secs as f32;
            let total_laps = (snapshot.standing_seconds / target_secs) as u32;
            let (r, g, b) = color_for_standing(lap_progress);

            let overlay = app_state.overlay.clone();
            overlay.update(lap_progress, (r, g, b));
            overlay.update_standing(lap_progress, total_laps);
            overlay.show();
            overlay.maybe_flash_lap(session_lap);
        }
        return;
    }

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

/// Updates tray tooltip from a session snapshot (called every ~1s).
fn update_tooltip(app: &AppHandle, snapshot: &crate::session::SessionStateDto) {
    let label = build_tooltip_label(
        snapshot.desk_height_cm,
        &snapshot.state,
        snapshot.sitting_seconds,
        snapshot.standing_seconds,
        snapshot.break_seconds,
        snapshot.daily_score,
    );
    let _ = tray::update_tray_tooltip(app, &label);
}

/// Builds tooltip like `"↕ 72.3 cm — Sitting (12:34) +38"` with state-appropriate duration and score.
pub(crate) fn build_tooltip_label(
    desk_height_cm: f32,
    state: &DeskState,
    sitting_secs: i64,
    standing_secs: i64,
    break_secs: i64,
    daily_score: f32,
) -> String {
    let (state_str, duration_secs) = match state {
        DeskState::Sitting => ("Sitting", sitting_secs),
        DeskState::Standing => ("Standing", break_secs),
        DeskState::Walking => ("Walking", break_secs),
        DeskState::Away => ("Away", 0),
    };
    let score_str = if daily_score >= 0.0 {
        format!(" +{:.0}", daily_score)
    } else {
        format!(" {:.0}", daily_score)
    };
    format!(
        "\u{2195} {:.0} cm \u{2014} {} ({}){}", // ↕ and —
        desk_height_cm, state_str, format_duration(duration_secs), score_str,
    )
}

/// Formats a duration in seconds as `"MM:SS"`.
pub(crate) fn format_duration(secs: i64) -> String {
    let secs = secs.max(0);
    let minutes = secs / 60;
    let seconds = secs % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

// Tests moved to tray_controller_tests.rs
