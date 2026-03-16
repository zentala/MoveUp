//! tray.rs — System tray icon and menu for the Desk application.
//!
//! Provides a color-coded 16×16 tray icon (green/yellow/red) with a tooltip
//! showing the current desk height and session state. Left-click toggles the
//! main window; the context menu has "Show Desk" and "Quit".

use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

// ─── Icon generation ──────────────────────────────────────────────────────────

/// Generates a 16×16 RGBA image filled with the given colour.
///
/// Draws a simple rounded square by zeroing corner pixels at a 2-pixel radius.
fn tray_icon(r: u8, g: u8, b: u8) -> Image<'static> {
    const SIZE: usize = 16;
    let mut pixels = vec![0u8; SIZE * SIZE * 4];

    for y in 0..SIZE {
        for x in 0..SIZE {
            // Approximate rounded corners: skip 2-pixel corner blocks.
            let corner = (x < 2 || x >= SIZE - 2) && (y < 2 || y >= SIZE - 2);
            let idx = (y * SIZE + x) * 4;
            if corner {
                pixels[idx] = 0;
                pixels[idx + 1] = 0;
                pixels[idx + 2] = 0;
                pixels[idx + 3] = 0;
            } else {
                pixels[idx] = r;
                pixels[idx + 1] = g;
                pixels[idx + 2] = b;
                pixels[idx + 3] = 255;
            }
        }
    }

    Image::new_owned(pixels, SIZE as u32, SIZE as u32)
}

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

    let icon = tray_icon(76, 175, 80); // green default

    TrayIconBuilder::with_id("main-tray")
        .icon(icon)
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

/// Updates the tray tooltip text and icon colour.
///
/// `label` — the tooltip string (e.g. "↕ 72.3 cm — Sitting (12:34)").
/// `r`, `g`, `b` — RGB components for the icon fill colour.
pub fn update_tray(app: &AppHandle, label: &str, r: u8, g: u8, b: u8) {
    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_tooltip(Some(label));
        let _ = tray.set_icon(Some(tray_icon(r, g, b)));
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

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
