//! Signal execution — maps CommunicationPolicy signals to UI side-effects.
//!
//! Extracted from `tray_controller.rs` to keep it under 250 lines.
//!
//! Blink engine: [`TrayBlinker`] lives in a `Mutex<TrayBlinker>` inside [`BlinkState`],
//! which is managed as Tauri app state. A background thread ticks the blinker at ~50ms
//! and calls `tray::update_tray` / `tray::update_tray_no_dot` based on the result.

use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

use crate::{
    colors::{color_for_progress, color_for_standing},
    commands::AppState,
    communication_types::{NotifySignal, OverlaySignal, PopupSignal, TraySignal},
    session::DeskState,
    tray,
    tray_blink::{BlinkPattern, TrayBlinker, DEFAULT_BLINK_PATTERN},
};

// ─── BlinkState ───────────────────────────────────────────────────────────────

/// Shared state for the blink background thread.
pub struct BlinkState {
    pub blinker: Mutex<TrayBlinker>,
    /// Name of the currently active blink pattern (empty = none).
    pub active_pattern_name: Mutex<String>,
    /// Progress ratio forwarded from the last TraySignal::Blink cycle.
    pub progress: Mutex<f32>,
}

impl BlinkState {
    pub fn new() -> Self {
        Self {
            blinker: Mutex::new(TrayBlinker::new()),
            active_pattern_name: Mutex::new(String::new()),
            progress: Mutex::new(0.0),
        }
    }
}

impl Default for BlinkState {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Blink thread ────────────────────────────────────────────────────────────

/// Spawns the blink background thread (call once from `lib.rs` setup).
///
/// The thread wakes every 50ms when the blinker is active, calls `tick()`,
/// and updates the tray icon accordingly.
pub fn start_blink_thread(app: AppHandle) {
    thread::spawn(move || {
        const TICK_MS: u64 = 50;
        let mut last = Instant::now();

        loop {
            thread::sleep(Duration::from_millis(TICK_MS));
            let elapsed = last.elapsed().as_millis() as u64;
            last = Instant::now();

            let blink_state = app.state::<BlinkState>();
            let mut blinker = blink_state.blinker
                .lock()
                .unwrap_or_else(|e| e.into_inner());

            if !blinker.is_active() {
                continue;
            }

            let dot_visible = blinker.tick(elapsed);
            let progress = *blink_state.progress
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            drop(blinker);

            if dot_visible {
                let _ = tray::update_tray(&app, "", DeskState::Sitting, progress.max(0.86));
            } else {
                let _ = tray::update_tray_no_dot(&app, "");
            }
        }
    });
}

// ─── Signal executors ─────────────────────────────────────────────────────────

/// Maps a [`TraySignal`] to a tray icon update.
///
/// For `Blink`, looks up the pattern in the communication profile and starts
/// (or keeps) the blinker. For any non-Blink signal, stops the blinker.
pub(crate) fn execute_tray(
    signal: &TraySignal,
    app: &AppHandle,
    snapshot: &crate::session::SessionStateDto,
) {
    let progress = if snapshot.session_limit_secs > 0 {
        snapshot.sitting_seconds as f32 / snapshot.session_limit_secs as f32
    } else {
        0.0
    };

    // Stop any active blinker for non-Blink signals
    if !matches!(signal, TraySignal::Blink(_)) {
        if let Some(blink_state) = app.try_state::<BlinkState>() {
            let mut blinker = blink_state.blinker.lock().unwrap_or_else(|e| e.into_inner());
            if blinker.is_active() {
                blinker.stop();
            }
            drop(blinker);
            *blink_state.active_pattern_name.lock().unwrap_or_else(|e| e.into_inner()) = String::new();
        }
    }

    match signal {
        TraySignal::None => {
            let _ = tray::update_tray_no_dot(app, "");
        }
        TraySignal::Yellow => {
            let _ = tray::update_tray(app, "", DeskState::Sitting, 0.7);
        }
        TraySignal::Red => {
            let _ = tray::update_tray(app, "", DeskState::Sitting, progress.max(0.86));
        }
        TraySignal::Blink(pattern_name) => {
            execute_tray_blink(app, pattern_name, progress);
        }
    }
}

/// Starts or continues a blink pattern by name from the active communication profile.
fn execute_tray_blink(app: &AppHandle, pattern_name: &str, progress: f32) {
    let blink_state = match app.try_state::<BlinkState>() {
        Some(s) => s,
        None => {
            // BlinkState not registered — fall back to red dot
            let _ = tray::update_tray(app, "", DeskState::Sitting, progress.max(0.86));
            return;
        }
    };

    // Update stored progress for the blink thread to use
    *blink_state.progress.lock().unwrap_or_else(|e| e.into_inner()) = progress;

    let mut current_name = blink_state.active_pattern_name.lock().unwrap_or_else(|e| e.into_inner());
    if *current_name == pattern_name {
        // Already running this pattern — nothing to do
        return;
    }

    // Look up pattern in the communication profile
    let app_state = app.state::<AppState>();
    let profile = app_state.comm_policy.lock().unwrap_or_else(|e| e.into_inner()).comm_profile().clone();
    let bp = profile.blink_patterns.get(pattern_name);

    let blink_pattern = if let Some(bp) = bp {
        BlinkPattern {
            on_ms: bp.on_ms as u64,
            off_ms: bp.off_ms as u64,
            count: bp.count,
            pause_ms: bp.pause_ms as u64,
        }
    } else {
        // Unknown pattern name — use a sensible default (3× blink / 10s)
        DEFAULT_BLINK_PATTERN.clone()
    };

    *current_name = pattern_name.to_string();
    drop(current_name);

    blink_state.blinker.lock().unwrap_or_else(|e| e.into_inner()).start(blink_pattern);
}

/// Maps an [`OverlaySignal`] to overlay renderer calls.
pub(crate) fn execute_overlay(
    signal: &OverlaySignal,
    overlay: &crate::overlay_renderer::OverlayRenderer,
    snapshot: &crate::session::SessionStateDto,
) {
    match signal {
        OverlaySignal::Hidden => {
            overlay.hide();
        }
        OverlaySignal::Neutral { progress } => {
            let (r, g, b, _) = color_for_progress(*progress);
            overlay.set_variant(0);
            overlay.update(*progress, (r, g, b));
            overlay.show();
        }
        OverlaySignal::Yellow { progress } => {
            overlay.set_variant(0);
            overlay.update(*progress, (255, 193, 7));
            overlay.show();
        }
        OverlaySignal::Red { progress } => {
            overlay.set_variant(0);
            overlay.update(*progress, (244, 67, 54));
            overlay.show();
        }
        OverlaySignal::PulseRed { progress } => {
            overlay.set_variant(2);
            overlay.update(*progress, (244, 67, 54));
            overlay.show();
        }
        OverlaySignal::Progress { progress, lap, flash } => {
            let (r, g, b) = color_for_standing(*progress);
            overlay.update(*progress, (r, g, b));
            overlay.update_standing(*progress, *lap);
            overlay.show();
            if *flash {
                let target = snapshot.stand_limit_secs;
                if target > 0 {
                    let session_lap = (snapshot.break_seconds / target) as u32;
                    overlay.maybe_flash_lap(session_lap);
                }
            }
        }
    }
}

/// Emits a `desk:popup-theme` event so the frontend can update the popup colour scheme.
pub(crate) fn execute_popup(signal: &PopupSignal, app: &AppHandle) {
    use tauri::Emitter;
    let theme = match signal {
        PopupSignal::Neutral => "neutral",
        PopupSignal::Yellow => "yellow",
        PopupSignal::Red => "red",
        PopupSignal::Gray => "gray",
    };
    let _ = app.emit(crate::desk_events::DESK_POPUP_THEME, theme);
}

/// Dispatches a [`NotifySignal`] to the appropriate notification backend.
pub(crate) fn execute_notify(signal: &NotifySignal, app_state: &AppState) {
    match signal {
        NotifySignal::Toast(msg) => {
            crate::notify::show("Smart Desk", msg);
        }
        NotifySignal::Popup(msg) => {
            app_state.alert_popup.lock().unwrap_or_else(|e| e.into_inner()).show(msg.clone());
        }
    }
}
