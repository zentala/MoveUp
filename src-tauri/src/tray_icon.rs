//! tray_icon.rs — RGBA tray icon generation helpers.
//!
//! Generates 32×32 tray icons: white desk silhouette with a colored status dot,
//! or with no dot (for blink-off / neutral state).

use tauri::image::Image;
use crate::session::DeskState;

// ── Desk silhouette geometry ──────────────────────────────────────────────────

const SIZE: u32 = 32;
const DESK_TOP: u32 = 2;
const DESK_BOTTOM: u32 = 7;
const DESK_LEFT: u32 = 1;
const DESK_RIGHT: u32 = 31;
const LEG_TOP: u32 = 7;
const LEG_BOTTOM: u32 = 18;
const LEG_WIDTH: u32 = 3;
const LEFT_LEG_X: u32 = 3;
const RIGHT_LEG_X: u32 = 26;

// ── Status dot geometry ───────────────────────────────────────────────────────

const DOT_SIZE: u32 = 12;
const DOT_RAISE: u32 = 2;
const DOT_X: u32 = SIZE - DOT_SIZE;
const DOT_Y: u32 = SIZE - DOT_SIZE - DOT_RAISE;

// ── Public API ────────────────────────────────────────────────────────────────

/// Generates a 32×32 RGBA tray icon: white desk silhouette + colored status dot.
///
/// Dot color:
/// - Green (#00C864) — sitting, <60% progress
/// - Amber (#C89600) — sitting, 60-85% progress
/// - Red (#C80000) — sitting, >85% progress
/// - Gold (#DAA520) — standing (positive feedback)
/// - Gray (#808080) — walking or away
pub(crate) fn generate_tray_icon(state: DeskState, progress_ratio: f32) -> Option<Image<'static>> {
    let mut rgba = blank_silhouette();

    let (dr, dg, db) = match state {
        DeskState::Sitting => {
            if progress_ratio >= 0.85 { (200, 0, 0) }       // red #C80000
            else if progress_ratio >= 0.60 { (200, 150, 0) } // amber #C89600
            else { (0, 200, 100) }                            // green #00C864
        }
        DeskState::Standing => (218, 165, 32), // gold #DAA520
        _ => (128, 128, 128),                  // gray #808080
    };

    for y in DOT_Y..(DOT_Y + DOT_SIZE) {
        for x in DOT_X..SIZE {
            set_pixel(&mut rgba, SIZE, x, y, dr, dg, db, 255);
        }
    }

    Some(Image::new_owned(rgba, SIZE, SIZE))
}

/// Generates a 32×32 RGBA tray icon with only the white desk silhouette — no dot.
///
/// Used during blink-off and pause phases so the dot disappears cleanly,
/// and also for the [`TraySignal::None`] baseline state.
pub(crate) fn generate_tray_icon_no_dot() -> Option<Image<'static>> {
    let rgba = blank_silhouette();
    Some(Image::new_owned(rgba, SIZE, SIZE))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Allocates a pixel buffer and draws the white desk silhouette (no dot).
fn blank_silhouette() -> Vec<u8> {
    let pixels = (SIZE * SIZE) as usize;
    let mut rgba = vec![0u8; pixels * 4];

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

    rgba
}

/// Sets an RGBA pixel in the buffer at (x, y).
fn set_pixel(buf: &mut [u8], width: u32, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) {
    let idx = ((y * width + x) as usize) * 4;
    buf[idx] = r;
    buf[idx + 1] = g;
    buf[idx + 2] = b;
    buf[idx + 3] = a;
}
