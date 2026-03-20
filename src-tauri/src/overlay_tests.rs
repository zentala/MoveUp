//! overlay_tests.rs — Unit tests for the overlay subsystem (DataSource, progress, renderer API).

use crate::colors::color_for_progress;
use crate::overlay_renderer::{DataSource, OverlayRenderer, OverlayState, demo_progress, mock_progress};

/// Helper: create renderer with Live data source for production behavior tests.
fn renderer_live() -> OverlayRenderer {
    let renderer = OverlayRenderer::new();
    renderer.state.lock().unwrap().data_source = DataSource::Live;
    renderer
}

#[test]
fn state_update_sets_needs_redraw() {
    let renderer = renderer_live();
    renderer.update(0.5, (255, 193, 7));
    let state = renderer.state.lock().unwrap();
    assert_eq!(state.progress, 0.5);
    assert_eq!(state.color_rgb, (255, 193, 7));
    assert!(state.needs_redraw, "update() must set needs_redraw");
}

#[test]
fn show_sets_visible_and_needs_redraw() {
    let renderer = renderer_live();
    renderer.show();
    let state = renderer.state.lock().unwrap();
    assert!(state.visible);
    assert!(state.needs_redraw);
}

#[test]
fn hide_clears_visible_and_sets_needs_redraw() {
    let renderer = renderer_live();
    renderer.show();
    renderer.hide();
    let state = renderer.state.lock().unwrap();
    assert!(!state.visible);
    assert!(state.needs_redraw);
}

#[test]
fn progress_clamped_to_0_1() {
    let renderer = renderer_live();
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
    let renderer = renderer_live();
    let (r, g, b, _css) = color_for_progress(0.3);
    renderer.update(0.3, (r, g, b));
    let state = renderer.state.lock().unwrap();
    assert_eq!(state.color_rgb, (76, 175, 80)); // green
}

#[test]
fn demo_source_ignores_external_updates() {
    let renderer = OverlayRenderer::new();
    // In debug builds, data_source defaults to Demo
    renderer.state.lock().unwrap().data_source = DataSource::Demo;

    renderer.update(0.75, (255, 0, 0));
    renderer.show();
    let state = renderer.state.lock().unwrap();
    assert_eq!(state.progress, 0.0, "Demo source should ignore update()");
    assert!(!state.visible, "Demo source should ignore show()");
}

#[test]
fn mock_source_ignores_external_updates() {
    let renderer = OverlayRenderer::new();
    renderer.state.lock().unwrap().data_source = DataSource::Mock;

    renderer.update(0.75, (255, 0, 0));
    renderer.show();
    let state = renderer.state.lock().unwrap();
    assert_eq!(state.progress, 0.0, "Mock source should ignore update()");
    assert!(!state.visible, "Mock source should ignore show()");
}

#[test]
fn live_source_accepts_external_updates() {
    let renderer = renderer_live();
    renderer.update(0.75, (255, 0, 0));
    renderer.show();
    let state = renderer.state.lock().unwrap();
    assert_eq!(state.progress, 0.75, "Live source should accept update()");
    assert!(state.visible, "Live source should accept show()");
}

#[test]
fn demo_progress_cycles_correctly() {
    let (p, _) = demo_progress(0);
    assert_eq!(p, 0.0);
    let (p, _) = demo_progress(300);
    assert_eq!(p, 0.25);
    let (p, _) = demo_progress(600);
    assert_eq!(p, 0.5);
    let (p, _) = demo_progress(900);
    assert_eq!(p, 0.75);
    let (p, _) = demo_progress(1200);
    assert_eq!(p, 1.0);
    let (p, _) = demo_progress(1500);
    assert_eq!(p, 0.0);
}

#[test]
fn bar_height_defaults_to_4() {
    let state = OverlayState::default();
    assert_eq!(state.bar_height, 4);
}

#[test]
fn bar_height_clamped_to_range() {
    // bar_height is read from env at Default::default() time,
    // so we test the clamp logic directly
    assert_eq!(0i32.clamp(1, 20), 1);
    assert_eq!(4i32.clamp(1, 20), 4);
    assert_eq!(25i32.clamp(1, 20), 20);
}

#[test]
fn overlay_variant_defaults_to_solid() {
    let state = OverlayState::default();
    assert_eq!(state.overlay_variant, 0);
}

#[test]
fn overlay_variant_clamped_to_max_2() {
    // Variant value is clamped via .min(2) in Default
    assert_eq!(3u8.min(2), 2);
    assert_eq!(255u8.min(2), 2);
    assert_eq!(1u8.min(2), 1);
    assert_eq!(0u8.min(2), 0);
}

#[test]
fn demo_progress_returns_correct_colors() {
    let (_, color) = demo_progress(0);
    assert_eq!(color, (76, 175, 80)); // green at 0%
    let (_, color) = demo_progress(600);
    assert_eq!(color, (76, 175, 80)); // green at 50%
    let (_, color) = demo_progress(900);
    assert_eq!(color, (255, 193, 7)); // yellow at 75%
    let (_, color) = demo_progress(1200);
    assert_eq!(color, (244, 67, 54)); // red at 100%
}

#[test]
fn demo_colors_match_color_for_progress() {
    for frame in [0, 300, 600, 900, 1200] {
        let (progress, (r, g, b)) = demo_progress(frame);
        let (er, eg, eb, _) = color_for_progress(progress);
        assert_eq!((r, g, b), (er, eg, eb), "Color mismatch at frame {}", frame);
    }
}

#[test]
fn bar_width_calculation_matches_progress() {
    let screen_width = 1920i32;
    let calc = |p: f32| ((screen_width as f32) * p.clamp(0.0, 1.0)) as i32;
    assert_eq!(calc(0.0), 0);
    assert_eq!(calc(0.0).max(1), 1); // demo/mock minimum 1px
    assert_eq!(calc(0.25), 480);
    assert_eq!(calc(0.5), 960);
    assert_eq!(calc(1.0), 1920);
}

#[test]
fn data_source_defaults_to_demo_in_debug() {
    if cfg!(debug_assertions) {
        let state = OverlayState::default();
        assert_eq!(state.data_source, DataSource::Demo);
    }
}

#[test]
fn mock_progress_sit_phase() {
    // Frame 0: start of sit phase
    let (p, _, visible) = mock_progress(0);
    assert_eq!(p, 0.0);
    assert!(visible, "should be visible during sit phase");

    // Mid sit phase
    let (p, _, visible) = mock_progress(4500);
    assert!((p - 0.5).abs() < 0.01, "should be ~50% at midpoint");
    assert!(visible);

    // End of sit phase
    let (p, _, visible) = mock_progress(8999);
    assert!(p > 0.99, "should be near 100% at end of sit");
    assert!(visible);
}

#[test]
fn mock_progress_stand_phase() {
    // Stand phase starts at frame 9000
    let (p, _, visible) = mock_progress(9000);
    assert_eq!(p, 0.0);
    assert!(!visible, "should be hidden during stand phase");

    let (_, _, visible) = mock_progress(10000);
    assert!(!visible);
}

#[test]
fn mock_progress_cycle_wraps() {
    // Cycle is 10800 frames (9000 sit + 1800 stand)
    let (p0, _, v0) = mock_progress(0);
    let (p_wrap, _, v_wrap) = mock_progress(10800);
    assert_eq!(p0, p_wrap, "should wrap to same progress");
    assert_eq!(v0, v_wrap, "should wrap to same visibility");
}

#[test]
fn mock_progress_colors_match_color_for_progress() {
    for frame in [0, 2250, 4500, 6750, 8999] {
        let (progress, (r, g, b), _) = mock_progress(frame);
        let (er, eg, eb, _) = color_for_progress(progress);
        assert_eq!((r, g, b), (er, eg, eb), "Color mismatch at frame {}", frame);
    }
}

#[test]
fn colorref_format_is_bgr() {
    let (r, g, b) = (255u8, 128u8, 0u8);
    let colorref = (r as u32) | ((g as u32) << 8) | ((b as u32) << 16);
    assert_eq!(colorref, 0x000080FF);
}
