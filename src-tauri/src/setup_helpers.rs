//! setup_helpers.rs — Extracted setup logic from lib.rs.
//!
//! Contains window positioning, device notification listeners, and
//! broadcast event wiring for the remote display WebSocket server.

use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use tauri::{AppHandle, Listener, Manager};
use tauri_plugin_notification::NotificationExt;
use window_vibrancy::apply_acrylic;

use crate::commands::AppState;
use crate::ws_broadcaster::{self, DisplayEvent};

/// Minimum interval between device-missing/lost notifications (5 minutes).
const DEVICE_NOTIFICATION_COOLDOWN_SECS: u64 = 300;

/// Applies Acrylic blur and positions the main window in the bottom-right corner.
pub fn position_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = apply_acrylic(&window, Some((18, 18, 18, 200)));

        if let Ok(Some(monitor)) = window.current_monitor() {
            let mon = monitor.size();
            let win = window.outer_size().unwrap_or_default();
            let x = mon.width as i32 - win.width as i32 - 16;
            let y = mon.height as i32 - win.height as i32 - 56;
            let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
        }
    }
}

/// Sets up throttled notifications for missing/lost sensor events.
pub fn setup_device_notifications(app: &AppHandle) {
    let last_notif = Arc::new(Mutex::new(
        Instant::now() - std::time::Duration::from_secs(DEVICE_NOTIFICATION_COOLDOWN_SECS),
    ));

    {
        let handle = app.clone();
        let last = Arc::clone(&last_notif);
        app.listen("desk:device-missing", move |_| {
            let mut guard = last.lock().unwrap();
            if guard.elapsed().as_secs() >= DEVICE_NOTIFICATION_COOLDOWN_SECS {
                let _ = handle
                    .notification()
                    .builder()
                    .title("zntlDesk")
                    .body("Sensor not connected. Plug in desk sensor.")
                    .show();
                *guard = Instant::now();
            }
        });
    }

    {
        let handle = app.clone();
        let last = Arc::clone(&last_notif);
        app.listen("desk:device-lost", move |_| {
            let mut guard = last.lock().unwrap();
            if guard.elapsed().as_secs() >= DEVICE_NOTIFICATION_COOLDOWN_SECS {
                let _ = handle
                    .notification()
                    .builder()
                    .title("zntlDesk")
                    .body("Sensor disconnected. Check USB cable.")
                    .show();
                *guard = Instant::now();
            }
        });
    }
}

/// Spawns the remote display HTTP+WS server and wires broadcast listeners.
pub fn setup_remote_display(app: &AppHandle) {
    let state: tauri::State<'_, AppState> = app.state();
    let remote_state = crate::remote_server::RemoteState {
        ws_tx: state.ws_tx.clone(),
        session: state.session.clone(),
        config: state.config.clone(),
        today_cache: state.today_cache.clone(),
        active_clients: Arc::new(AtomicUsize::new(0)),
    };
    let port = std::env::var("DESK_REMOTE_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(crate::remote_server::DEFAULT_PORT);
    tauri::async_runtime::spawn(async move {
        crate::remote_server::start(remote_state, port).await;
    });
    setup_broadcast_listeners(app);
}

/// Sets up broadcast listeners for device and daily-reset events.
/// Forwards these events to the WebSocket broadcast channel for remote clients.
pub fn setup_broadcast_listeners(app: &AppHandle) {
    // desk:device-connected → broadcast to remote clients
    let handle1 = app.clone();
    app.listen("desk:device-connected", move |event| {
        let ws_tx = handle1.state::<AppState>().ws_tx.clone();
        let port = serde_json::from_str::<serde_json::Value>(event.payload())
            .ok()
            .and_then(|v| v.get("port").and_then(|p| p.as_str()).map(String::from))
            .unwrap_or_default();
        ws_broadcaster::broadcast_event(&ws_tx, &DisplayEvent::DeviceConnected { port });
    });

    // desk:device-lost → broadcast to remote clients
    let handle2 = app.clone();
    app.listen("desk:device-lost", move |_event| {
        let ws_tx = handle2.state::<AppState>().ws_tx.clone();
        ws_broadcaster::broadcast_event(&ws_tx, &DisplayEvent::DeviceLost);
    });

    // desk:daily-reset → broadcast + clear today_cache
    let handle3 = app.clone();
    app.listen("desk:daily-reset", move |_event| {
        let state = handle3.state::<AppState>();
        ws_broadcaster::broadcast_event(&state.ws_tx, &DisplayEvent::DailyReset);
        // Clear cached today summary
        let mut cache = state.today_cache.lock().unwrap_or_else(|e| e.into_inner());
        *cache = crate::db::TodaySummary::default();
    });
}
