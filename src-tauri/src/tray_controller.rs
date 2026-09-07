//! Wires `desk:state-changed` + `desk:distance` events to tray tooltip,
//! overlay progress bar, and [`CommunicationPolicy`] signal engine.

use chrono::Utc;
use tauri::{AppHandle, Listener, Manager};

use crate::{
    colors::color_for_progress,
    commands::AppState,
    communication_policy::PolicyInput,
    communication_types::NotifySignal,
    notify_webhook::{Notification, WebhookNotifier},
    desk_events::{
        DESK_DEVICE_CONNECTED, DESK_DEVICE_LOST, DESK_DEVICE_MISSING, DESK_DISTANCE,
        DESK_STATE_CHANGED,
    },
    remote_display_state,
    session::{DeskState, StateChangedPayload},
    tray,
    tray_signal_exec,
    ws_broadcaster::{self, DisplayEvent},
};

// ─── Public API ───────────────────────────────────────────────────────────────

/// Registers `desk:state-changed` and `desk:distance` event listeners.
/// Call once from `lib.rs` setup.
pub fn setup(app: &AppHandle) {
    let handle = app.clone();
    app.listen(DESK_STATE_CHANGED, move |event| {
        if let Ok(payload) = serde_json::from_str::<StateChangedPayload>(event.payload()) {
            on_state_changed(&handle, &payload);
        }
    });

    let handle2 = app.clone();
    app.listen(DESK_DISTANCE, move |_event| {
        update_from_policy(&handle2);
    });

    let handle3 = app.clone();
    app.listen(DESK_DEVICE_LOST, move |_event| {
        let _ = tray::update_tray(&handle3, "Desk \u{2014} sensor disconnected", DeskState::Away, 0.0);
    });

    let handle4 = app.clone();
    app.listen(DESK_DEVICE_MISSING, move |_event| {
        let _ = tray::update_tray(&handle4, "Desk \u{2014} no sensor found", DeskState::Away, 0.0);
    });

    let handle5 = app.clone();
    app.listen(DESK_DEVICE_CONNECTED, move |_event| {
        let app_state = handle5.state::<AppState>();
        app_state.comm_policy.lock().unwrap_or_else(|e| e.into_inner()).on_sensor_connected();
    });
}

// ─── Private helpers ──────────────────────────────────────────────────────────

/// Reacts to a state-change event: notify policy of position change, then evaluate.
fn on_state_changed(app: &AppHandle, payload: &StateChangedPayload) {
    let app_state = app.state::<AppState>();

    app_state.comm_policy.lock().unwrap_or_else(|e| e.into_inner()).on_position_changed();

    let snapshot = app_state.session.lock().unwrap_or_else(|e| e.into_inner()).snapshot();
    let overlay = app_state.overlay.clone();

    let progress = if snapshot.session_limit_secs > 0 {
        payload.limit_used_secs as f32 / snapshot.session_limit_secs as f32
    } else {
        0.0
    };

    let label = build_tooltip_label(
        payload.desk_height_cm,
        &payload.state,
        payload.limit_used_secs,
        payload.standing_seconds,
        payload.break_seconds,
        snapshot.daily_score,
    );
    let _ = tray::update_tray(app, &label, payload.state.clone(), progress);

    if payload.state == DeskState::Sitting {
        let (r, g, b, _) = color_for_progress(progress);
        overlay.clear_standing();
        overlay.update(progress, (r, g, b));
        overlay.show();
    } else {
        overlay.clear_standing();
        overlay.hide();
    }

    ws_broadcaster::broadcast_event(
        &app_state.ws_tx,
        &DisplayEvent::StateChanged(
            serde_json::to_value(payload).unwrap_or_default(),
        ),
    );

    refresh_today_cache(app);
}

/// Evaluates CommunicationPolicy each tick (~1/s) and executes returned signals.
fn update_from_policy(app: &AppHandle) {
    let app_state = app.state::<AppState>();

    if app_state.alert_popup.lock().unwrap_or_else(|e| e.into_inner()).take_user_dismissed() {
        app_state.comm_policy.lock().unwrap_or_else(|e| e.into_inner()).dismiss();
    }

    let is_connected = app_state.conn.connected_port.lock().unwrap_or_else(|e| e.into_inner()).is_some();

    let session = app_state.session.lock().unwrap_or_else(|e| e.into_inner());
    // Dismiss alert popup when user stands (popup is no longer relevant).
    if session.state.state == DeskState::Standing {
        app_state.alert_popup.lock().unwrap_or_else(|e| e.into_inner()).dismiss();
    }
    let snapshot = session.snapshot();
    // Every policy-facing derived value comes from the engine (E020-T05); this
    // adapter contributes only the sensor-connectivity fact the engine lacks.
    let input: PolicyInput = session.policy_input(Utc::now(), is_connected);
    drop(session);

    remote_display_state::broadcast(
        &app_state.ws_tx,
        &app_state.session,
        &app_state.comm_policy,
        &app_state.today_cache,
    );
    update_tooltip(app, &snapshot);

    let signals = app_state.comm_policy.lock().unwrap_or_else(|e| e.into_inner()).evaluate(&input);

    tray_signal_exec::execute_tray(&signals.tray, app, &snapshot);

    let overlay = app_state.overlay.clone();
    tray_signal_exec::execute_overlay(&signals.overlay, &overlay, &snapshot);
    tray_signal_exec::execute_popup(&signals.popup, app);

    if let Some(ref notify) = signals.notify {
        tray_signal_exec::execute_notify(notify, &app_state);
        push_to_webhook(&app_state, notify);
    }
}

/// Mirrors a toast to the user's webhook so a phone or watch sees it too.
///
/// Only [`NotifySignal::Toast`] is mirrored: `Popup` is the on-screen
/// escalation of a toast that already went out, and pushing both would send
/// the same alert twice. Sending is fire-and-forget — see
/// [`crate::notify_webhook`] — so a dead receiver cannot stall this tick.
fn push_to_webhook(app_state: &AppState, notify: &NotifySignal) {
    let NotifySignal::Toast(message) = notify else {
        return;
    };
    let config = app_state
        .config
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .unwrap_or_default();
    if let Some(notifier) = WebhookNotifier::from_env_or_config(
        config.notify_webhook_enabled,
        config.notify_webhook_url.as_deref(),
    ) {
        notifier.send(Notification::alert("MoveUp", message));
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

use crate::tray_helpers::{build_tooltip_label, refresh_today_cache};

// Tests moved to tray_controller_tests.rs
