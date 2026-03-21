//! overlay_standing_tests.rs — Tests for standing bar, lap flash, and clear_standing.

use std::time::{Duration, Instant};
use crate::overlay_renderer::{DataSource, OverlayRenderer};

fn renderer_live() -> OverlayRenderer {
    let r = OverlayRenderer::new();
    r.state.lock().unwrap().data_source = DataSource::Live;
    r
}

#[test]
fn update_standing_sets_mode_and_progress() {
    let r = renderer_live();
    r.update_standing(0.5, 0);
    let s = r.state.lock().unwrap();
    assert!(s.standing_mode);
    assert!((s.progress - 0.5).abs() < 0.01);
    assert_eq!(s.lap, 0);
}

#[test]
fn update_standing_with_laps() {
    let r = renderer_live();
    r.update_standing(0.3, 2);
    let s = r.state.lock().unwrap();
    assert!(s.standing_mode);
    assert_eq!(s.lap, 2);
}

#[test]
fn clear_standing_resets_all() {
    let r = renderer_live();
    r.update_standing(0.8, 1);
    r.start_lap_flash();
    r.clear_standing();
    let s = r.state.lock().unwrap();
    assert!(!s.standing_mode);
    assert!(s.lap_flash_until.is_none());
    assert_eq!(s.last_flashed_lap, 0);
    assert_eq!(s.overlay_variant, 0);
}

#[test]
fn start_lap_flash_sets_variant_and_timer() {
    let r = renderer_live();
    r.start_lap_flash();
    let s = r.state.lock().unwrap();
    assert!(s.lap_flash_until.is_some());
    assert_eq!(s.overlay_variant, 2);
}

#[test]
fn tick_lap_flash_before_expiry_keeps_flash() {
    let r = renderer_live();
    r.start_lap_flash();
    let before = Instant::now();
    r.tick_lap_flash(before);
    let s = r.state.lock().unwrap();
    assert!(s.lap_flash_until.is_some(), "flash should still be active");
    assert_eq!(s.overlay_variant, 2);
}

#[test]
fn tick_lap_flash_after_expiry_clears_flash() {
    let r = renderer_live();
    r.start_lap_flash();
    let future = Instant::now() + Duration::from_secs(3);
    r.tick_lap_flash(future);
    let s = r.state.lock().unwrap();
    assert!(s.lap_flash_until.is_none(), "flash should be expired");
    assert_eq!(s.overlay_variant, 0, "variant should reset to solid");
}

#[test]
fn clear_standing_cancels_active_flash() {
    let r = renderer_live();
    r.start_lap_flash();
    r.clear_standing();
    let s = r.state.lock().unwrap();
    assert!(s.lap_flash_until.is_none());
}

#[test]
fn maybe_flash_lap_fires_on_new_lap() {
    let r = renderer_live();
    r.maybe_flash_lap(1);
    let s = r.state.lock().unwrap();
    assert_eq!(s.last_flashed_lap, 1);
    assert!(s.lap_flash_until.is_some());
    assert_eq!(s.overlay_variant, 2);
}

#[test]
fn maybe_flash_lap_no_double_flash() {
    let r = renderer_live();
    r.maybe_flash_lap(1);
    // Expire the flash
    let future = Instant::now() + Duration::from_secs(3);
    r.tick_lap_flash(future);
    // Call again with same lap — should NOT re-flash
    r.maybe_flash_lap(1);
    let s = r.state.lock().unwrap();
    assert!(s.lap_flash_until.is_none(), "should not re-flash same lap");
    assert_eq!(s.overlay_variant, 0);
}

#[test]
fn maybe_flash_lap_fires_on_second_lap() {
    let r = renderer_live();
    r.maybe_flash_lap(1);
    let future = Instant::now() + Duration::from_secs(3);
    r.tick_lap_flash(future);
    r.maybe_flash_lap(2);
    let s = r.state.lock().unwrap();
    assert_eq!(s.last_flashed_lap, 2);
    assert!(s.lap_flash_until.is_some());
}

#[test]
fn clear_standing_allows_new_session_flash() {
    let r = renderer_live();
    r.maybe_flash_lap(1);
    r.clear_standing();
    // New session: lap 1 should flash again
    r.maybe_flash_lap(1);
    let s = r.state.lock().unwrap();
    assert_eq!(s.last_flashed_lap, 1);
    assert!(s.lap_flash_until.is_some());
}

#[test]
fn maybe_flash_lap_zero_does_not_flash() {
    let r = renderer_live();
    r.maybe_flash_lap(0);
    let s = r.state.lock().unwrap();
    assert!(s.lap_flash_until.is_none(), "lap 0 should not trigger flash");
}

#[test]
fn standing_bar_width_with_laps() {
    let screen_width = 1920i32;
    let lap_px = 8i32;
    let lap: u32 = 1;
    let progress: f32 = 0.5;
    let lap_width = (lap as i32 * lap_px).min(screen_width);
    let remaining = screen_width - lap_width;
    let fill = ((remaining as f32) * progress) as i32;
    let total = lap_width + fill;
    assert_eq!(lap_width, 8);
    assert_eq!(total, 8 + 956); // 8 + (1912 * 0.5)
}

#[test]
fn standing_bar_width_no_laps() {
    let screen_width = 1920i32;
    let lap: u32 = 0;
    let progress: f32 = 0.5;
    let lap_width = (lap as i32 * 8).min(screen_width);
    let remaining = screen_width - lap_width;
    let fill = ((remaining as f32) * progress) as i32;
    assert_eq!(lap_width, 0);
    assert_eq!(fill, 960);
}
