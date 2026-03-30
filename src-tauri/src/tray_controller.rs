//! Wires `desk:state-changed` + `desk:distance` events to tray tooltip,
//! overlay progress bar, and [`CommunicationPolicy`] signal engine.

use tauri::{AppHandle, Listener, Manager};

use crate::{
    colors::color_for_progress,
    commands::AppState,
    communication_policy::PolicyInput,
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
    app.listen("desk:state-changed", move |event| {
        if let Ok(payload) = serde_json::from_str::<StateChangedPayload>(event.payload()) {
            on_state_changed(&handle, &payload);
        }
    });

    let handle2 = app.clone();
    app.listen("desk:distance", move |_event| {
        update_from_policy(&handle2);
    });

    let handle3 = app.clone();
    app.listen("desk:device-lost", move |_event| {
        let _ = tray::update_tray(&handle3, "Desk \u{2014} sensor disconnected", DeskState::Away, 0.0);
    });

    let handle4 = app.clone();
    app.listen("desk:device-missing", move |_event| {
        let _ = tray::update_tray(&handle4, "Desk \u{2014} no sensor found", DeskState::Away, 0.0);
    });

    let handle5 = app.clone();
    app.listen("desk:device-connected", move |_event| {
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
        payload.sitting_seconds as f32 / snapshot.session_limit_secs as f32
    } else {
        0.0
    };

    let label = build_tooltip_label(
        payload.desk_height_cm,
        &payload.state,
        payload.sitting_seconds,
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

    let session = app_state.session.lock().unwrap_or_else(|e| e.into_inner());
    // Dismiss alert popup when user stands (popup is no longer relevant).
    if session.state.state == DeskState::Standing {
        app_state.alert_popup.lock().unwrap_or_else(|e| e.into_inner()).dismiss();
    }
    let snapshot = session.snapshot();
    let now = chrono::Utc::now();
    let mut raw_state = session.state.clone();
    raw_state.sitting_seconds_total = session.get_live_sitting_seconds_total(now);
    raw_state.standing_seconds = session.get_live_standing_seconds(now);
    drop(session);

    broadcast_remote_state(app, &app_state, &snapshot, &raw_state);
    update_tooltip(app, &snapshot);

    let is_connected = app_state.conn.connected_port.lock().unwrap_or_else(|e| e.into_inner()).is_some();
    let (standing_lap_progress, standing_lap, standing_lap_flash) =
        compute_standing_lap(&snapshot);

    let elapsed_secs = match snapshot.state {
        DeskState::Sitting => snapshot.sitting_seconds,
        DeskState::Standing => snapshot.break_seconds,
        _ => 0,
    };

    let input = PolicyInput {
        state: snapshot.state.clone(),
        elapsed_secs,
        sensor_connected: is_connected,
        standing_lap_progress,
        standing_lap,
        standing_lap_flash,
    };

    let signals = app_state.comm_policy.lock().unwrap_or_else(|e| e.into_inner()).evaluate(&input);

    tray_signal_exec::execute_tray(&signals.tray, app, &snapshot);

    let overlay = app_state.overlay.clone();
    tray_signal_exec::execute_overlay(&signals.overlay, &overlay, &snapshot);
    tray_signal_exec::execute_popup(&signals.popup, app);

    if let Some(ref notify) = signals.notify {
        tray_signal_exec::execute_notify(notify, &app_state);
    }
}

/// Computes standing lap progress, lap count, and flash trigger.
fn compute_standing_lap(snapshot: &crate::session::SessionStateDto) -> (f32, u32, bool) {
    if snapshot.state != DeskState::Standing || snapshot.stand_limit_secs <= 0 {
        return (0.0, 0, false);
    }
    let target = snapshot.stand_limit_secs;
    let session_secs = snapshot.break_seconds;
    let session_lap = (session_secs / target) as u32;
    let lap_progress = (session_secs % target) as f32 / target as f32;
    let total_laps = (snapshot.standing_seconds / target) as u32;
    (lap_progress, total_laps, session_lap > 0)
}

/// Broadcasts session state + metrics to remote display WebSocket clients.
fn broadcast_remote_state(
    _app: &AppHandle,
    app_state: &AppState,
    snapshot: &crate::session::SessionStateDto,
    raw_state: &crate::session_types::SessionState,
) {
    let ergo = app_state.comm_policy.lock().unwrap_or_else(|e| e.into_inner()).ergo_profile().clone();
    let metrics = crate::metrics::MetricEngine::with_defaults()
        .compute_all(raw_state, &ergo);
    let today = app_state.today_cache.lock().unwrap_or_else(|e| e.into_inner()).clone();
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
