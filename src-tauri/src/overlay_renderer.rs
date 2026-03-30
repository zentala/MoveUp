//! overlay_renderer.rs — Public API, `DataSource`, `OverlayState`, event loop dispatcher.
//!
//! WinAPI window at (0,0), N px tall x full screen width, always-on-top.
//! Standing mode methods live in `overlay_standing.rs` (separate impl block).

use std::sync::{Arc, Mutex};
use std::time::Instant;
use log::info;

/// Data source for overlay progress bar (Demo, Live, or Mock).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataSource {
    /// Cycling demo animation (0%→25%→50%→75%→100%). Requires OVERLAY_DATA=demo.
    Demo,
    /// Real sensor data via tray_controller.rs. Default in release builds.
    Live,
    /// Simulated 40-min sit / 10-min stand cycle, compressed to ~3 min.
    Mock,
}

/// Shared state — written from Tauri thread, read from WinAPI thread.
pub struct OverlayState {
    pub progress: f32,           // 0.0 – 1.0
    pub color_rgb: (u8, u8, u8), // RGB
    pub visible: bool,
    pub needs_redraw: bool,      // dirty flag — redraw only when changed
    pub frame_count: u32,        // For animation (demo/mock cycling)
    pub data_source: DataSource, // OVERLAY_DATA=demo|live|mock
    pub bar_height: i32,         // Default 4, configurable via OVERLAY_HEIGHT
    pub overlay_variant: u8,     // 0=solid, 1=gradient, 2=pulsing (OVERLAY_VARIANT env)
    /// True when showing gold standing bar (false = sitting bar).
    pub standing_mode: bool,
    /// Completed laps today (for left-indicator rendering).
    pub lap: u32,
    /// Flash active until this instant (None = no flash).
    pub lap_flash_until: Option<Instant>,
    /// Session lap that last triggered a flash (prevents re-flash on same lap).
    pub last_flashed_lap: u32,
}

/// Parses `OVERLAY_DATA` env var into a [`DataSource`].
///
/// Default: `Live` in ALL builds. Demo/Mock require explicit env var.
fn parse_data_source() -> DataSource {
    let default = DataSource::Live;
    let result = match std::env::var("OVERLAY_DATA").as_deref() {
        Ok("demo") => DataSource::Demo,
        Ok("live") => DataSource::Live,
        Ok("mock") => DataSource::Mock,
        Ok(other) => {
            log::warn!("[OVERLAY] Unknown OVERLAY_DATA='{}', using default {:?}", other, default);
            default
        }
        Err(_) => default,
    };
    log::info!(
        "[OVERLAY] DataSource={:?} (OVERLAY_DATA env={:?}, debug={})",
        result,
        std::env::var("OVERLAY_DATA").ok(),
        cfg!(debug_assertions),
    );
    result
}

impl Default for OverlayState {
    fn default() -> Self {
        let data_source = parse_data_source();
        let bar_height = std::env::var("OVERLAY_HEIGHT")
            .ok()
            .and_then(|v| v.parse::<i32>().ok())
            .unwrap_or(4)
            .clamp(1, 20);
        let overlay_variant = std::env::var("OVERLAY_VARIANT")
            .ok()
            .and_then(|v| v.parse::<u8>().ok())
            .unwrap_or(0)
            .min(2); // 0=solid, 1=gradient, 2=pulsing
        Self {
            progress: 0.0,
            color_rgb: crate::colors::NEUTRAL, // neutral default
            visible: false,
            needs_redraw: false,
            frame_count: 0,
            data_source,
            bar_height,
            overlay_variant,
            standing_mode: false,
            lap: 0,
            lap_flash_until: None,
            last_flashed_lap: 0,
        }
    }
}

pub struct OverlayRenderer {
    pub(crate) state: Arc<Mutex<OverlayState>>,
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
            if s.data_source != DataSource::Live { return; }
            s.progress = progress.clamp(0.0, 1.0);
            s.color_rgb = color_rgb;
            s.needs_redraw = true;
        }
    }

    pub fn show(&self) {
        if let Ok(mut s) = self.state.lock() {
            if s.data_source != DataSource::Live { return; }
            s.visible = true;
            s.needs_redraw = true;
        }
    }

    pub fn hide(&self) {
        if let Ok(mut s) = self.state.lock() {
            if s.data_source != DataSource::Live { return; }
            s.visible = false;
            s.needs_redraw = true;
        }
    }

    /// Sets the visual variant of the overlay bar.
    ///
    /// - `0` — solid fill (default)
    /// - `1` — gradient
    /// - `2` — pulsing animation (used by alert Stage1 and standing lap flash)
    ///
    /// Operates in Live mode only (same guard as [`update`](Self::update)).
    pub fn set_variant(&self, variant: u8) {
        if let Ok(mut s) = self.state.lock() {
            if s.data_source != DataSource::Live { return; }
            s.overlay_variant = variant.min(2);
            s.needs_redraw = true;
        }
    }

    // Standing mode methods: update_standing, start_lap_flash, maybe_flash_lap,
    // clear_standing, tick_lap_flash — see overlay_standing.rs

    /// Returns overlay state as JSON for debugging (debug builds only).
    #[cfg(debug_assertions)]
    pub fn debug_state(&self) -> Option<serde_json::Value> {
        let s = self.state.lock().ok()?;
        Some(serde_json::json!({
            "data_source": format!("{:?}", s.data_source),
            "progress": (s.progress * 100.0).round() / 100.0,
            "progress_pct": format!("{:.1}%", s.progress * 100.0),
            "color_rgb": [s.color_rgb.0, s.color_rgb.1, s.color_rgb.2],
            "visible": s.visible,
            "frame_count": s.frame_count,
            "bar_height": s.bar_height,
            "overlay_variant": s.overlay_variant,
        }))
    }
}

/// Calculates demo progress and color from frame count.
/// Cycles: 0% → 25% → 50% → 75% → 100% every 5 seconds (300 frames @ 60fps).
pub(crate) fn demo_progress(frame_count: u32) -> (f32, (u8, u8, u8)) {
    use crate::colors::color_for_progress;
    let stage = ((frame_count / 300) % 5) as u32;
    let progress = stage as f32 / 4.0;
    let (r, g, b, _) = color_for_progress(progress);
    (progress, (r, g, b))
}

/// Calculates mock progress simulating a realistic sit/stand cycle.
///
/// ```text
/// |<── sit phase (9000 frames = 2.5 min) ──>|<── stand (1800 = 30s) ──>|
/// |  progress: 0.0 ──────────────────> 1.0  |  bar hidden              |
/// |  visible: true                          |  visible: false           |
/// |  Total cycle: 10800 frames = 3 min (simulates 50 min real)         |
/// ```
pub(crate) fn mock_progress(frame_count: u32) -> (f32, (u8, u8, u8), bool) {
    use crate::colors::color_for_progress;
    const SIT_FRAMES: u32 = 9000;   // 2.5 min @ 60fps
    const STAND_FRAMES: u32 = 1800; // 30s @ 60fps
    const CYCLE: u32 = SIT_FRAMES + STAND_FRAMES;

    let pos = frame_count % CYCLE;
    if pos < SIT_FRAMES {
        let progress = pos as f32 / SIT_FRAMES as f32;
        let (r, g, b, _) = color_for_progress(progress);
        (progress, (r, g, b), true)
    } else {
        // Stand phase: bar hidden, progress reset
        (0.0, crate::colors::NEUTRAL, false)
    }
}

/// WinAPI event loop dispatcher — runs in background thread.
///
/// ⚠️ CreateWindowExW MUST be called here, not in OverlayRenderer::new().
///
/// Dispatches to OPAQUE or LAYERED backend based on `OVERLAY_MODE` env var.
#[cfg(target_os = "windows")]
fn run_event_loop(state: Arc<Mutex<OverlayState>>) {
    use crate::overlay_layered::run_event_loop_layered;
    use crate::overlay_opaque::run_event_loop_opaque;

    let mode = std::env::var("OVERLAY_MODE").unwrap_or_else(|_| "opaque".to_string());
    let (data_source, bar_height) = state.lock()
        .map(|s| (s.data_source, s.bar_height))
        .unwrap_or((DataSource::Live, 4));

    info!("[OVERLAY] data_source={:?}, render_mode={}, height={}", data_source, mode, bar_height);

    match mode.as_str() {
        "layered" => run_event_loop_layered(state, bar_height),
        _ => run_event_loop_opaque(state, bar_height),
    }
}

/// Placeholder for non-Windows platforms.
#[cfg(not(target_os = "windows"))]
fn run_event_loop(_state: Arc<Mutex<OverlayState>>) {
    info!("⚠️ Overlay renderer not supported on this platform");
}

