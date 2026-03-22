//! tray.rs — System tray icon and menu for the Desk application.
//!
//! Generates dynamic tray icons (white desk silhouette + colored status dot)
//! based on sitting state and session progress. Falls back to PNG loading if available.
//! The tooltip shows the current desk height and session state.
//! Left-click toggles the main window; right-click shows context menu (Settings, Quit).

use std::path::PathBuf;
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

use crate::tray_icon::icon_for_state_and_progress;
use crate::session::DeskState;

// ─── Public API ───────────────────────────────────────────────────────────────

/// Creates the system tray icon, tooltip, and context menu.
///
/// Left-click directly toggles the main window (no menu).
/// Right-click shows context menu: Settings, separator, Quit.
pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = MenuBuilder::new(app)
        .item(&settings_item)
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
            "settings" => show_main_window_settings(app),
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

        // Try to load PNG first, fall back to generated RGBA icon
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

/// Generates a 32x32 RGBA tray icon: white desk silhouette + colored status dot.
///
/// Transparent background with a white desk shape (horizontal bar + two legs)
/// and a 4x4 colored dot at the bottom-right corner indicating state:
/// - Green (#00C864) — sitting, <60% progress
/// - Amber (#C89600) — sitting, 60-85% progress
/// - Red (#C80000) — sitting, >85% progress
/// - Gold (#DAA520) — standing (positive feedback)
/// - Gray (#808080) — walking or away
fn generate_tray_icon(state: DeskState, progress_ratio: f32) -> Option<Image<'static>> {
    const SIZE: u32 = 32;
    let pixels = (SIZE * SIZE) as usize;
    let mut rgba = vec![0u8; pixels * 4]; // transparent background

    // Draw white desk silhouette
    // Desk surface: y=10..14, x=4..28
    for y in 10u32..14 {
        for x in 4u32..28 {
            set_pixel(&mut rgba, SIZE, x, y, 255, 255, 255, 255);
        }
    }
    // Left leg: x=6..8, y=14..22
    for y in 14u32..22 {
        for x in 6u32..8 {
            set_pixel(&mut rgba, SIZE, x, y, 255, 255, 255, 255);
        }
    }
    // Right leg: x=24..26, y=14..22
    for y in 14u32..22 {
        for x in 24u32..26 {
            set_pixel(&mut rgba, SIZE, x, y, 255, 255, 255, 255);
        }
    }

    // Dot color based on state
    let (dr, dg, db) = match state {
        DeskState::Sitting => {
            if progress_ratio >= 0.85 { (200, 0, 0) }       // red #C80000
            else if progress_ratio >= 0.60 { (200, 150, 0) } // amber #C89600
            else { (0, 200, 100) }                            // green #00C864
        }
        DeskState::Standing => (218, 165, 32), // gold #DAA520
        _ => (128, 128, 128),                  // gray #808080
    };

    // 4x4 dot at bottom-right (x=26..30, y=26..30)
    for y in 26u32..30 {
        for x in 26u32..30 {
            set_pixel(&mut rgba, SIZE, x, y, dr, dg, db, 255);
        }
    }

    Some(Image::new_owned(rgba, SIZE, SIZE))
}

/// Sets an RGBA pixel in the buffer at (x, y).
fn set_pixel(buf: &mut [u8], width: u32, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) {
    let idx = ((y * width + x) as usize) * 4;
    buf[idx] = r;
    buf[idx + 1] = g;
    buf[idx + 2] = b;
    buf[idx + 3] = a;
}

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
///
/// On show: emits `desk:show-widget` so the frontend resets to the widget view
/// (not settings). On hide: just hides.
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
