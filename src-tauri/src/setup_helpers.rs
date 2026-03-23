//! setup_helpers.rs — Extracted setup logic from lib.rs.
//!
//! Contains window positioning and device notification listeners
//! to keep lib.rs under the 250-line limit.

use std::sync::{Arc, Mutex};
use std::time::Instant;

use tauri::{AppHandle, Listener, Manager};
use tauri_plugin_notification::NotificationExt;
use window_vibrancy::apply_acrylic;

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
