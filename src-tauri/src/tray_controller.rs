//! Wires `desk:state-changed` + `desk:distance` events to tray tooltip,
//! overlay progress bar, and [`AlertManager`] state machine.

use tauri::{AppHandle, Listener, Manager};

use crate::{
    alert_manager::AlertAction,
    colors::{color_for_progress, color_for_standing},
    commands::AppState,
    session::{DeskState, StateChangedPayload},
    tray,
    ws_broadcaster::{self, DisplayEvent},
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

    // Device disconnected: show gray tray icon with "disconnected" tooltip
    let handle3 = app.clone();
    app.listen("desk:device-lost", move |_event| {
        let _ = tray::update_tray(&handle3, "Desk — sensor disconnected", DeskState::Away, 0.0);
    });

    let handle4 = app.clone();
    app.listen("desk:device-missing", move |_event| {
        let _ = tray::update_tray(&handle4, "Desk — no sensor found", DeskState::Away, 0.0);
    });
}

// ─── Private helpers ──────────────────────────────────────────────────────────

/// Reacts to a state-change event by updating tray, overlay, and alerts.
fn on_state_changed(app: &AppHandle, payload: &StateChangedPayload) {

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

    // Broadcast state change to remote display clients
    let app_state = app.state::<AppState>();
    ws_broadcaster::broadcast_event(
        &app_state.ws_tx,
        &DisplayEvent::StateChanged(
            serde_json::to_value(payload).unwrap_or_default(),
        ),
    );

    // Refresh today_cache from DB on state transitions (new session row may exist)
    refresh_today_cache(app);
}

/// Updates overlay progress on every sensor reading and drives the alert state machine.
fn update_overlay_progress(app: &AppHandle) {

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
    let now = chrono::Utc::now();
    let mut raw_state = session.state.clone();
    raw_state.sitting_seconds_total = session.get_live_sitting_seconds_total(now);
    raw_state.standing_seconds = session.get_live_standing_seconds(now);
    drop(session); // Release lock before calling tray/overlay

    // Broadcast session + metrics to remote display clients (~1/s).
    // All data is from memory — zero DB access in the hot path.
    {
        let config_guard = app_state.config.lock().unwrap();
        let config = config_guard.as_ref().cloned().unwrap_or_default();
        drop(config_guard);
        let metrics = crate::metrics::MetricEngine::with_defaults()
            .compute_all(&raw_state, &config);
        let today = app_state.today_cache.lock().unwrap().clone();
        let remote_state = ws_broadcaster::RemoteDisplayState {
            session: snapshot.clone(),
            metrics,
            today,
        };
        ws_broadcaster::broadcast_event(
            &app_state.ws_tx,
            &DisplayEvent::Snapshot(remote_state),
        );
    }

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

// Tooltip formatting and today-cache refresh moved to tray_helpers.rs
use crate::tray_helpers::{build_tooltip_label, refresh_today_cache};

// Tests moved to tray_controller_tests.rs
