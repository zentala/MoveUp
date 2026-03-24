//! overlay_standing.rs — Standing progress bar methods for OverlayRenderer.
//!
//! Gold bar rendering, lap flash state, and standing mode transitions.
//! Separated from overlay_renderer.rs to keep files under 250 lines.

use std::time::Instant;

use crate::overlay_renderer::{DataSource, OverlayRenderer};

impl OverlayRenderer {
    /// Sets standing mode with progress and lap count (gold bar).
    pub fn update_standing(&self, lap_progress: f32, lap: u32) {
        if let Ok(mut s) = self.state.lock() {
            if s.data_source != DataSource::Live { return; }
            s.standing_mode = true;
            s.progress = lap_progress.clamp(0.0, 1.0);
            s.lap = lap;
            s.needs_redraw = true;
        }
    }

    /// Starts a 2-second gold pulse (lap celebration flash).
    pub fn start_lap_flash(&self) {
        if let Ok(mut s) = self.state.lock() {
            s.lap_flash_until = Some(Instant::now() + std::time::Duration::from_secs(2));
            s.overlay_variant = 2;
            s.needs_redraw = true;
        }
    }

    /// Flashes if `session_lap` > `last_flashed_lap` (idempotent).
    ///
    /// Prevents double-flash on the same lap. Resets via `clear_standing()`.
    pub fn maybe_flash_lap(&self, session_lap: u32) {
        let should_flash = {
            let s = match self.state.lock() {
                Ok(s) => s,
                Err(_) => return,
            };
            session_lap > 0 && session_lap > s.last_flashed_lap
        };
        if should_flash {
            if let Ok(mut s) = self.state.lock() {
                s.last_flashed_lap = session_lap;
            }
            self.start_lap_flash();
        }
    }

    /// Clears standing mode — revert to sitting bar, cancel any lap flash.
    pub fn clear_standing(&self) {
        if let Ok(mut s) = self.state.lock() {
            s.standing_mode = false;
            s.lap_flash_until = None;
            s.last_flashed_lap = 0;
            s.overlay_variant = 0;
            s.needs_redraw = true;
        }
    }

    /// Advance the lap flash timer. Called from render loop with `Instant::now()`,
    /// or from tests with a synthetic future instant.
    #[allow(dead_code)] // used in tests; will be called from render loop
    pub fn tick_lap_flash(&self, now: Instant) {
        if let Ok(mut s) = self.state.lock() {
            if let Some(until) = s.lap_flash_until {
                if now > until {
                    s.lap_flash_until = None;
                    s.overlay_variant = 0;
                    s.needs_redraw = true;
                }
            }
        }
    }
}
