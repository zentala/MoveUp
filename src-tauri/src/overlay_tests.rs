//! overlay_tests.rs — Core unit tests for the overlay renderer API.

use crate::colors::color_for_progress;
use crate::overlay_renderer::{DataSource, OverlayRenderer};

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
    assert_eq!(state.color_rgb, (0x5a, 0x55, 0x48));
}

#[test]
fn demo_source_ignores_external_updates() {
    let renderer = OverlayRenderer::new();
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
fn bar_width_calculation_matches_progress() {
    let screen_width = 1920i32;
    let calc = |p: f32| ((screen_width as f32) * p.clamp(0.0, 1.0)) as i32;
    assert_eq!(calc(0.0), 0);
    assert_eq!(calc(0.0).max(1), 1);
    assert_eq!(calc(0.25), 480);
    assert_eq!(calc(0.5), 960);
    assert_eq!(calc(1.0), 1920);
}

#[test]
fn colorref_format_is_bgr() {
    let (r, g, b) = (255u8, 128u8, 0u8);
    let colorref = (r as u32) | ((g as u32) << 8) | ((b as u32) << 16);
    assert_eq!(colorref, 0x000080FF);
}
