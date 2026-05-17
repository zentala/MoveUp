//! tray.rs — System tray icon, menu, and update API for the Desk application.
//!
//! Builds the tray via Tauri 2's `TrayIconBuilder`. Delegates icon generation
//! to [`tray_icon`] (RGBA pixel art). Tooltip text is built by [`tray_helpers`].
//! Left-click toggles the main window; right-click shows Settings + Quit menu.

use std::path::PathBuf;
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

use crate::session::DeskState;
use crate::tray_icon::{generate_tray_icon, generate_tray_icon_no_dot};

// ─── Public API ───────────────────────────────────────────────────────────────

/// Creates the system tray icon, tooltip, and context menu.
///
/// Left-click directly toggles the main window (no menu).
/// Right-click shows context menu: Settings, separator, Quit.
pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let analyst_item = MenuItem::with_id(app, "open-analyst", "Open Analyst", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = MenuBuilder::new(app)
        .item(&settings_item)
        .item(&analyst_item)
        .item(&separator)
        .item(&quit_item)
        .build()?;

    let mut builder = TrayIconBuilder::with_id("main-tray");
    let initial_icon = icon_path_for_state(app, DeskState::Away, 0.0)
        .ok()
        .and_then(|p| Image::from_path(&p).ok())
        .or_else(|| generate_tray_icon(DeskState::Away, 0.0));
    if let Some(icon) = initial_icon {
        builder = builder.icon(icon);
    }

    builder
        .tooltip("Desk — connecting…")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "settings" => show_main_window_settings(app),
            "open-analyst" => show_analyst_window(app),
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

/// Updates the tray tooltip and icon based on session state and progress.
///
/// `label` — tooltip string (e.g. "↕ 72.3 cm — Sitting (12:34)").
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
        let icon = if let Ok(icon_path) = icon_path_for_state(app, state.clone(), progress_ratio) {
            Image::from_path(&icon_path).ok()
        } else {
            None
        };
        let icon = icon.or_else(|| generate_tray_icon(state, progress_ratio));
        if let Some(icon) = icon {
            let _ = tray.set_icon(Some(icon));
        }
    }
    Ok(())
}

/// Updates the tray icon to show the desk silhouette with no colored dot.
///
/// Used when [`TrayBlinker`] says the dot is hidden (off phase / pause phase),
/// and also for the [`TraySignal::None`] baseline state.
pub fn update_tray_no_dot(
    app: &AppHandle,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(tray) = app.tray_by_id("main-tray") {
        if !label.is_empty() {
            let _ = tray.set_tooltip(Some(label));
        }
        if let Some(icon) = generate_tray_icon_no_dot() {
            let _ = tray.set_icon(Some(icon));
        }
    }
    Ok(())
}

/// Updates only the tray tooltip text (no icon change).
///
/// Lightweight alternative to [`update_tray`] for frequent per-second updates
/// where the icon does not need to change (e.g. while standing).
pub fn update_tray_tooltip(
    app: &AppHandle,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_tooltip(Some(label));
    }
    Ok(())
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Selects the PNG icon name based on sitting state and progress.
///
/// - `tray-ok` (green) — sitting, <60% of session limit
/// - `tray-warn` (amber) — sitting, 60-85% of limit
/// - `tray-alert` (red) — sitting, >85% of limit
/// - `tray-standing` (gold) — standing
/// - `tray-idle` (gray) — walking or away
pub(crate) fn icon_for_state_and_progress(state: DeskState, ratio: f32) -> &'static str {
    const THRESHOLD_YELLOW: f32 = 0.60;
    const THRESHOLD_RED: f32 = 0.85;

    match state {
        DeskState::Sitting => {
            if ratio >= THRESHOLD_RED { "tray-alert" }
            else if ratio >= THRESHOLD_YELLOW { "tray-warn" }
            else { "tray-ok" }
        }
        DeskState::Standing => "tray-standing",
        _ => "tray-idle",
    }
}

/// Resolves the PNG icon path for a given state and progress ratio.
fn icon_path_for_state(
    app: &AppHandle,
    state: DeskState,
    progress_ratio: f32,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let icon_name = icon_for_state_and_progress(state, progress_ratio);
    let icon_path = app
        .path()
        .resource_dir()?
        .join("icons/tray")
        .join(format!("{}.png", icon_name));
    Ok(icon_path)
}

/// Toggles the main window between visible/hidden.
fn toggle_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = app.emit("desk:show-widget", ());
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

/// Opens the main window directly to the Settings view.
fn show_main_window_settings(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = app.emit("desk:show-settings", ());
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Shows and focuses the Analyst window. Rebuilds it on-demand if the user
/// closed it earlier — Tauri destroys webview windows on close by default.
fn show_analyst_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("analyst") {
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }
    match tauri::WebviewWindowBuilder::new(
        app,
        "analyst",
        tauri::WebviewUrl::App("index.html#/analyst".into()),
    )
    .title("Smart Desk \u{2014} Analyst")
    .inner_size(1280.0, 800.0)
    .min_inner_size(960.0, 600.0)
    .resizable(true)
    .center()
    .decorations(true)
    .build()
    {
        Ok(window) => {
            let _ = window.show();
            let _ = window.set_focus();
        }
        Err(e) => log::error!("tray: failed to (re)build analyst window: {}", e),
    }
}

/// Ordered list of menu item ids on the tray menu (test introspection).
pub fn tray_menu_item_ids() -> &'static [&'static str] {
    &["settings", "open-analyst", "quit"]
}

