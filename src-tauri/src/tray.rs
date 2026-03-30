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

    // Load initial tray icon (disconnected state — gray dot)
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
    // Desk silhouette geometry
    const DESK_TOP: u32 = 2;
    const DESK_BOTTOM: u32 = 7;
    const DESK_LEFT: u32 = 1;
    const DESK_RIGHT: u32 = 31;
    const LEG_TOP: u32 = 7;
    const LEG_BOTTOM: u32 = 18;
    const LEG_WIDTH: u32 = 3;
    const LEFT_LEG_X: u32 = 3;
    const RIGHT_LEG_X: u32 = 26;
    // Status dot geometry
    const DOT_SIZE: u32 = 12;
    const DOT_RAISE: u32 = 2;
    const DOT_X: u32 = SIZE - DOT_SIZE;
    const DOT_Y: u32 = SIZE - DOT_SIZE - DOT_RAISE;

    let pixels = (SIZE * SIZE) as usize;
    let mut rgba = vec![0u8; pixels * 4]; // transparent background

    // Draw white desk silhouette
    for y in DESK_TOP..DESK_BOTTOM {
        for x in DESK_LEFT..DESK_RIGHT {
            set_pixel(&mut rgba, SIZE, x, y, 255, 255, 255, 255);
        }
    }
    for y in LEG_TOP..LEG_BOTTOM {
        for x in LEFT_LEG_X..(LEFT_LEG_X + LEG_WIDTH) {
            set_pixel(&mut rgba, SIZE, x, y, 255, 255, 255, 255);
        }
        for x in RIGHT_LEG_X..(RIGHT_LEG_X + LEG_WIDTH) {
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

    // Status dot at bottom-right corner
    for y in DOT_Y..(DOT_Y + DOT_SIZE) {
        for x in DOT_X..SIZE {
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

/// Selects the appropriate PNG tray icon name based on sitting state and progress.
///
/// Returns the icon file name (without `.png` extension):
/// - `tray-ok` (green) — sitting, <60% of session limit
/// - `tray-warn` (amber) — sitting, 60-85% of limit
/// - `tray-alert` (red) — sitting, >85% of limit
/// - `tray-standing` (gold) — standing (positive feedback)
/// - `tray-idle` (gray) — walking or away
fn icon_for_state_and_progress(state: DeskState, ratio: f32) -> &'static str {
    const THRESHOLD_YELLOW: f32 = 0.60;
    const THRESHOLD_RED: f32 = 0.85;

    match state {
        DeskState::Sitting => {
            if ratio >= THRESHOLD_RED {
                "tray-alert"
            } else if ratio >= THRESHOLD_YELLOW {
                "tray-warn"
            } else {
                "tray-ok"
            }
        }
        DeskState::Standing => "tray-standing",
        _ => "tray-idle", // Walking, Away
    }
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

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sitting_below_60_percent_returns_ok() {
        assert_eq!(icon_for_state_and_progress(DeskState::Sitting, 0.3), "tray-ok");
    }

    #[test]
    fn sitting_60_to_85_percent_returns_warn() {
        assert_eq!(icon_for_state_and_progress(DeskState::Sitting, 0.7), "tray-warn");
    }

    #[test]
    fn sitting_above_85_percent_returns_alert() {
        assert_eq!(icon_for_state_and_progress(DeskState::Sitting, 0.9), "tray-alert");
    }

    #[test]
    fn sitting_exactly_60_percent_returns_warn() {
        assert_eq!(icon_for_state_and_progress(DeskState::Sitting, 0.60), "tray-warn");
    }

    #[test]
    fn sitting_exactly_85_percent_returns_alert() {
        assert_eq!(icon_for_state_and_progress(DeskState::Sitting, 0.85), "tray-alert");
    }

    #[test]
    fn standing_returns_standing() {
        assert_eq!(icon_for_state_and_progress(DeskState::Standing, 0.5), "tray-standing");
    }

    #[test]
    fn walking_returns_idle() {
        assert_eq!(icon_for_state_and_progress(DeskState::Walking, 0.5), "tray-idle");
    }

    #[test]
    fn away_returns_idle() {
        assert_eq!(icon_for_state_and_progress(DeskState::Away, 0.5), "tray-idle");
    }

    #[test]
    fn standing_returns_standing_regardless_of_progress() {
        assert_eq!(icon_for_state_and_progress(DeskState::Standing, 0.0), "tray-standing");
        assert_eq!(icon_for_state_and_progress(DeskState::Standing, 1.0), "tray-standing");
    }
}
