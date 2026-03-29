//! commands_welcome.rs — Welcome popup window commands.

use tauri::{Manager, WebviewWindowBuilder, WebviewUrl};
use crate::config::AppConfig;

/// Creates and shows the welcome popup window.
pub fn show_welcome_window(app: &tauri::AppHandle) -> Result<(), String> {
    // Don't create a duplicate if already open
    if app.get_webview_window("welcome").is_some() {
        return Ok(());
    }
    WebviewWindowBuilder::new(app, "welcome", WebviewUrl::App("welcome.html".into()))
        .title("Smart Desk \u{2014} Witaj!")
        .inner_size(480.0, 420.0)
        .resizable(false)
        .always_on_top(true)
        .center()
        .decorations(true)
        .build()
        .map_err(|e| format!("Failed to create welcome window: {}", e))?;
    Ok(())
}

/// Dismisses the welcome popup. If `dont_show_again` is true, persists
/// the preference so the popup won't appear on future launches.
#[tauri::command]
pub fn dismiss_welcome(dont_show_again: bool, app: tauri::AppHandle) -> Result<(), String> {
    if dont_show_again {
        if let Some(store) = app.try_state::<tauri_plugin_store::Store<tauri::Wry>>() {
            let mut config = AppConfig::load(store.inner());
            config.show_welcome_on_startup = false;
            config.save(store.inner())?;
        }
    }
    if let Some(win) = app.get_webview_window("welcome") {
        let _ = win.close();
    }
    Ok(())
}

/// Re-opens the welcome popup (callable from settings).
#[tauri::command]
pub fn show_welcome(app: tauri::AppHandle) -> Result<(), String> {
    show_welcome_window(&app)
}
