//! overlay.rs — Top-of-screen transparent progress bar overlay window.
//!
//! Creates a 4-pixel-tall transparent window pinned to the top of the primary
//! monitor. The frontend listens for `overlay:progress` events and renders a
//! coloured bar whose width reflects the sitting progress ratio.

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, WebviewWindowBuilder};

// ─── Event payload ────────────────────────────────────────────────────────────

/// Payload emitted to the overlay window on every progress update.
#[derive(Debug, Clone, Serialize)]
pub struct OverlayProgress {
    /// Fraction of the session limit consumed (0.0–1.0+).
    pub progress: f32,
    /// CSS colour string for the bar (e.g. `"#4caf50"`).
    pub color: String,
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Creates the overlay window, initially hidden.
///
/// The window is transparent, always-on-top, and skips the taskbar.
/// Its height is fixed at 4 px; width defaults to 1920 (overridden at runtime
/// by the frontend if needed). Position is (0, 0) — top-left of the screen.
pub fn setup_overlay(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // Determine primary monitor width for accurate sizing.
    let width = app
        .primary_monitor()
        .ok()
        .flatten()
        .map(|m| m.size().width)
        .unwrap_or(1920) as f64;

    WebviewWindowBuilder::new(app, "overlay", tauri::WebviewUrl::App("overlay.html".into()))
        .title("")
        .inner_size(width, 4.0)
        .position(0.0, 0.0)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .visible(false)
        .build()?;

    Ok(())
}

/// Emits a progress update to the overlay window.
///
/// `progress` — ratio 0.0–1.0 (values > 1.0 are clamped visually by CSS).
/// `color`    — CSS colour for the bar.
pub fn update_overlay(app: &AppHandle, progress: f32, color: &str) {
    if let Some(window) = app.get_webview_window("overlay") {
        let payload = OverlayProgress {
            progress,
            color: color.to_string(),
        };
        let _ = window.emit("overlay:progress", payload);
    }
}

/// Hides the overlay window.
pub fn hide_overlay(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("overlay") {
        let _ = window.hide();
    }
}

/// Shows the overlay window.
pub fn show_overlay(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("overlay") {
        let _ = window.show();
    }
}
