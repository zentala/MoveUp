//! notify.rs — Windows notification helper with correct AUMID.
//!
//! The `tauri-plugin-notification` skips setting AppUserModelId in dev mode,
//! causing Windows to attribute notifications to "PowerShell" or the parent
//! process. This module wraps `notify-rust` directly with AUMID always set.

/// App identifier used as Windows AppUserModelId.
const APP_ID: &str = "io.zntl.desk";

/// Sends a Windows toast notification with the correct app identity.
///
/// Always sets `app_id` so notifications show as "Smart Desk" regardless
/// of whether the app runs from an installer or `pnpm tauri:dev`.
pub fn show(title: &str, body: &str) {
    let title = title.to_string();
    let body = body.to_string();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = notify_rust::Notification::new()
            .appname("Smart Desk")
            .summary(&title)
            .body(&body)
            .app_id(APP_ID)
            .show()
        {
            log::warn!("Toast notification failed: {e}");
        }
    });
}
