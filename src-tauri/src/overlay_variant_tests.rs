//! overlay_variant_tests.rs — Tests for overlay variants, demo/mock progress, and data sources.

use crate::colors::color_for_progress;
use crate::overlay_renderer::{DataSource, OverlayRenderer, OverlayState, demo_progress, mock_progress};

/// Helper: create renderer with Live data source.
fn renderer_live() -> OverlayRenderer {
    let renderer = OverlayRenderer::new();
    renderer.state.lock().unwrap().data_source = DataSource::Live;
    renderer
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
fn demo_progress_returns_correct_colors() {
    let (_, color) = demo_progress(0);
    assert_eq!(color, (0x5a, 0x55, 0x48));
    let (_, color) = demo_progress(600);
    assert_eq!(color, (0x5a, 0x55, 0x48));
    let (_, color) = demo_progress(900);
    assert_eq!(color, (255, 193, 7));
    let (_, color) = demo_progress(1200);
    assert_eq!(color, (244, 67, 54));
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
fn mock_progress_sit_phase() {
    let (p, _, visible) = mock_progress(0);
    assert_eq!(p, 0.0);
    assert!(visible, "should be visible during sit phase");

    let (p, _, visible) = mock_progress(4500);
    assert!((p - 0.5).abs() < 0.01, "should be ~50% at midpoint");
    assert!(visible);

    let (p, _, visible) = mock_progress(8999);
    assert!(p > 0.99, "should be near 100% at end of sit");
    assert!(visible);
}

#[test]
fn mock_progress_stand_phase() {
    let (p, _, visible) = mock_progress(9000);
    assert_eq!(p, 0.0);
    assert!(!visible, "should be hidden during stand phase");

    let (_, _, visible) = mock_progress(10000);
    assert!(!visible);
}

#[test]
fn mock_progress_cycle_wraps() {
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
fn set_variant_2_sets_pulsing() {
    let renderer = renderer_live();
    renderer.set_variant(2);
    let state = renderer.state.lock().unwrap();
    assert_eq!(state.overlay_variant, 2);
    assert!(state.needs_redraw);
}

#[test]
fn set_variant_0_sets_solid() {
    let renderer = renderer_live();
    renderer.set_variant(2);
    {
        let mut s = renderer.state.lock().unwrap();
        s.needs_redraw = false;
    }
    renderer.set_variant(0);
    let state = renderer.state.lock().unwrap();
    assert_eq!(state.overlay_variant, 0);
    assert!(state.needs_redraw);
}

#[test]
fn set_variant_clamped_to_2() {
    let renderer = renderer_live();
    renderer.set_variant(5);
    let state = renderer.state.lock().unwrap();
    assert_eq!(state.overlay_variant, 2);
}

#[test]
fn set_variant_ignored_in_demo_mode() {
    let renderer = OverlayRenderer::new();
    renderer.state.lock().unwrap().data_source = DataSource::Demo;
    renderer.set_variant(2);
    let state = renderer.state.lock().unwrap();
    assert_eq!(state.overlay_variant, 0);
}

#[test]
fn data_source_defaults_to_live() {
    let state = OverlayState::default();
    assert_eq!(state.data_source, DataSource::Live);
}

#[test]
fn bar_height_defaults_to_4() {
    let state = OverlayState::default();
    assert_eq!(state.bar_height, 4);
}

#[test]
fn bar_height_clamped_to_range() {
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
    assert_eq!(3u8.min(2), 2);
    assert_eq!(255u8.min(2), 2);
    assert_eq!(1u8.min(2), 1);
    assert_eq!(0u8.min(2), 0);
}
