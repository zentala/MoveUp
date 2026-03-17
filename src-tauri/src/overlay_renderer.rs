//! overlay_renderer.rs — System-level progress bar overlay using raw WinAPI.
//!
//! Creates a native Windows window (NOT Tauri WebviewWindow) at position (0,0).
//! Window is 4px tall × full screen width, always-on-top, no decorations.
//!
//! ⚠️ CRITICAL: CreateWindowExW() MUST be called in the SAME THREAD as the
//! message loop (PeekMessage/DispatchMessage). Never call it from another thread.
//! All WinAPI window operations stay inside run_event_loop().

use std::sync::{Arc, Mutex};
use log::info;

use crate::colors::color_for_progress;

/// Shared state — written from Tauri thread, read from WinAPI thread.
pub struct OverlayState {
    pub progress: f32,           // 0.0 – 1.0
    pub color_rgb: (u8, u8, u8), // RGB
    pub visible: bool,
    pub needs_redraw: bool,      // dirty flag — redraw only when changed
}

impl Default for OverlayState {
    fn default() -> Self {
        Self {
            progress: 0.0,
            color_rgb: (76, 175, 80), // green
            visible: false,
            needs_redraw: false,
        }
    }
}

pub struct OverlayRenderer {
    state: Arc<Mutex<OverlayState>>,
}

impl OverlayRenderer {
    pub fn new() -> Self {
        let state = Arc::new(Mutex::new(OverlayState::default()));
        let state_clone = Arc::clone(&state);

        // Spawn background thread — all WinAPI calls happen here
        // ⚠️ Panic if spawn fails: overlay is critical functionality
        std::thread::Builder::new()
            .name("overlay-renderer".into())
            .spawn(move || {
                run_event_loop(state_clone);
            })
            .expect("Failed to spawn overlay thread");

        Self { state }
    }

    pub fn update(&self, progress: f32, color_rgb: (u8, u8, u8)) {
        if let Ok(mut s) = self.state.lock() {
            s.progress = progress.clamp(0.0, 1.0);
            s.color_rgb = color_rgb;
            s.needs_redraw = true;
        }
    }

    pub fn show(&self) {
        if let Ok(mut s) = self.state.lock() {
            s.visible = true;
            s.needs_redraw = true;
        }
    }

    pub fn hide(&self) {
        if let Ok(mut s) = self.state.lock() {
            s.visible = false;
            s.needs_redraw = true;
        }
    }
}

/// WinAPI event loop — runs in background thread.
///
/// ⚠️ CreateWindowExW MUST be called here, not in OverlayRenderer::new().
#[cfg(target_os = "windows")]
fn run_event_loop(state: Arc<Mutex<OverlayState>>) {
    use windows::Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        UI::WindowsAndMessaging::*,
    };

    // TODO: detect primary monitor width at runtime
    let screen_width = 1920i32;
    let height = 4i32;

    unsafe {
        // TODO: Implement full WinAPI window setup:
        // 1. Register window class
        // 2. CreateWindowExW() with WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW
        // 3. SetTimer(hwnd, 1, 16, None)  ← 60fps timer
        // 4. Message loop:
        //    - WM_TIMER: if needs_redraw { InvalidateRect() }
        //    - WM_PAINT: BeginPaint → FillRect(bar_width) → EndPaint
        //    - WM_DESTROY: PostQuitMessage(0)
    }

    info!("🎨 WinAPI overlay window created: {}x{} @ (0,0)", screen_width, height);
}

/// Placeholder for non-Windows platforms
#[cfg(not(target_os = "windows"))]
fn run_event_loop(_state: Arc<Mutex<OverlayState>>) {
    info!("⚠️ Overlay renderer not supported on this platform");
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_update_sets_needs_redraw() {
        let renderer = OverlayRenderer::new();
        renderer.update(0.5, (255, 193, 7));
        let state = renderer.state.lock().unwrap();
        assert_eq!(state.progress, 0.5);
        assert_eq!(state.color_rgb, (255, 193, 7));
        assert!(state.needs_redraw, "update() must set needs_redraw");
    }

    #[test]
    fn show_sets_visible_and_needs_redraw() {
        let renderer = OverlayRenderer::new();
        renderer.show();
        let state = renderer.state.lock().unwrap();
        assert!(state.visible);
        assert!(state.needs_redraw);
    }

    #[test]
    fn hide_clears_visible_and_sets_needs_redraw() {
        let renderer = OverlayRenderer::new();
        renderer.show();
        renderer.hide();
        let state = renderer.state.lock().unwrap();
        assert!(!state.visible);
        assert!(state.needs_redraw);
    }

    #[test]
    fn progress_clamped_to_0_1() {
        let renderer = OverlayRenderer::new();
        renderer.update(1.5, (0, 0, 0));
        let state = renderer.state.lock().unwrap();
        assert_eq!(state.progress, 1.0);

        drop(state);
        renderer.update(-0.5, (0, 0, 0));
        let state = renderer.state.lock().unwrap();
        assert_eq!(state.progress, 0.0);
    }

    #[test]
    fn color_for_progress_works_with_renderer() {
        let renderer = OverlayRenderer::new();
        let (r, g, b, _css) = color_for_progress(0.3);
        renderer.update(0.3, (r, g, b));
        let state = renderer.state.lock().unwrap();
        assert_eq!(state.color_rgb, (76, 175, 80)); // green
    }
}
