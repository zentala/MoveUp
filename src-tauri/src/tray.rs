//! tray.rs — System tray icon and menu for the Desk application.
//!
//! Loads PNG tray icons based on sitting state and session progress.
//! The tooltip shows the current desk height and session state.
//! Left-click toggles the main window; the context menu has "Show Desk" and "Quit".

use std::path::PathBuf;
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

use crate::tray_icon::icon_for_state_and_progress;
use crate::session::DeskState;

// ─── Public API ───────────────────────────────────────────────────────────────

/// Creates the system tray icon, tooltip, and context menu.
///
/// Left-click toggles the main window. The menu exposes "Show Desk" and "Quit".
pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let show_item = MenuItem::with_id(app, "show", "Show Desk", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = MenuBuilder::new(app)
        .item(&show_item)
        .item(&separator)
        .item(&quit_item)
        .build()?;

    // Load initial tray icon (default: idle state, 0.0 progress)
    // Try to load PNG, but don't fail if it's missing (e.g., in dev mode before bundling)
    let mut builder = TrayIconBuilder::with_id("main-tray");
    if let Ok(icon_path) = icon_path_for_state(app, DeskState::Away, 0.0) {
        if let Ok(icon) = Image::from_path(&icon_path) {
            builder = builder.icon(icon);
        }
    }

    builder
        .tooltip("Desk — connecting…")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => toggle_main_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

/// Updates the tray tooltip text and icon based on session state and progress.
///
/// `label` — the tooltip string (e.g. "↕ 72.3 cm — Sitting (12:34)").
/// `state` — current desk state (Sitting, Standing, Walking, Away).
/// `progress_ratio` — sitting time / session limit (0.0–1.0+).
pub fn update_tray(
    app: &AppHandle,
    label: &str,
    state: DeskState,
    progress_ratio: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_tooltip(Some(label));

        // Try to load PNG icon, fall back to gray square if it fails
        if let Ok(icon_path) = icon_path_for_state(app, state, progress_ratio) {
            if let Ok(icon) = Image::from_path(&icon_path) {
                let _ = tray.set_icon(Some(icon));
            }
        }
    }

    Ok(())
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Resolves the PNG icon path for a given state and progress ratio.
///
/// Uses `icon_for_state_and_progress()` to select the icon name based on both
/// sitting state and progress, then constructs the full path under `src-tauri/icons/tray/`.
fn icon_path_for_state(
    app: &AppHandle,
    state: DeskState,
    progress_ratio: f32,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let icon_name = icon_for_state_and_progress(state, progress_ratio);
    let icon_path = app.path().resource_dir()?.join("icons/tray").join(format!("{}.png", icon_name));
    Ok(icon_path)
}

/// Toggles the main window between visible/hidden.
fn toggle_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}
