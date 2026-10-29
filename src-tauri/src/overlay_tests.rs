//! overlay_tests.rs — Core unit tests for the overlay renderer API.

use crate::colors::color_for_progress;
use crate::overlay_renderer::{DataSource, OverlayRenderer};

/// Helper: create renderer with Live data source for production behavior tests.
fn renderer_live() -> OverlayRenderer {
    renderer_for_source(DataSource::Live)
}

fn renderer_for_source(data_source: DataSource) -> OverlayRenderer {
    let mut state = crate::overlay_renderer::OverlayState::default();
    state.data_source = data_source;
    OverlayRenderer { state: std::sync::Arc::new(std::sync::Mutex::new(state)) }
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
    let renderer = renderer_for_source(DataSource::Demo);

    renderer.update(0.75, (255, 0, 0));
    renderer.show();
    let state = renderer.state.lock().unwrap();
    assert_eq!(state.progress, 0.0, "Demo source should ignore update()");
    assert!(!state.visible, "Demo source should ignore show()");
}

#[test]
fn mock_source_ignores_external_updates() {
    let renderer = renderer_for_source(DataSource::Mock);

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

#[cfg(target_os = "windows")]
mod native_exposure {
    use super::*;
    use std::sync::{Arc, Mutex};
    use windows::core::PCSTR;
    use windows::Win32::{Foundation::*, System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::*};

    // Only disposable off-screen windows; never start Tauri or locate installed HWNDs.
    struct Fixture {
        overlay: HWND,
        cover: HWND,
        renderer: OverlayRenderer,
        state_ptr: *mut Arc<Mutex<crate::overlay_renderer::OverlayState>>,
        class: Vec<u8>,
        instance: HINSTANCE,
        _serial: std::sync::MutexGuard<'static, ()>,
    }

    impl Fixture {
        unsafe fn new() -> Self {
            // The ordinary suite runs tests in parallel; these own windows share a
            // disposable off-screen rectangle and must not cover each other.
            static SERIAL: Mutex<()> = Mutex::new(());
            let serial = SERIAL.lock().unwrap_or_else(|error| error.into_inner());
            static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let class = format!("MoveUpExposureTest-{}-{id}\0", std::process::id()).into_bytes();
            let instance: HINSTANCE = GetModuleHandleW(None).unwrap().into();
            let wc = WNDCLASSA {
                lpfnWndProc: Some(crate::overlay_opaque::wnd_proc),
                cbWndExtra: (2 * std::mem::size_of::<isize>()) as i32,
                hInstance: instance, lpszClassName: PCSTR(class.as_ptr()),
                ..Default::default()
            };
            assert_ne!(RegisterClassA(&wc), 0);
            let create = |height| CreateWindowExA(
                WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW, PCSTR(class.as_ptr()), PCSTR::null(),
                WS_POPUP, -30000, -30000, 100, height, None, None, Some(instance), None,
            ).unwrap();
            let overlay = create(4);
            let cover = create(20);
            let mut state = crate::overlay_renderer::OverlayState::default();
            state.data_source = DataSource::Live;
            let renderer = OverlayRenderer { state: Arc::new(Mutex::new(state)) };
            let state_ptr = Box::into_raw(Box::new(renderer.state.clone()));
            SetWindowLongPtrW(overlay, WINDOW_LONG_PTR_INDEX(0), state_ptr as isize);
            let fixture = Self { overlay, cover, renderer, state_ptr, class, instance, _serial: serial };
            fixture.normal_cover();
            fixture
        }

        unsafe fn normal_cover(&self) {
            let flags = SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_SHOWWINDOW;
            SetWindowPos(self.overlay, Some(HWND_NOTOPMOST), 0, 0, 0, 0, flags).unwrap();
            SetWindowPos(self.overlay, Some(HWND_BOTTOM), 0, 0, 0, 0, flags).unwrap();
            SetWindowPos(self.cover, Some(HWND_TOP), 0, 0, 0, 0, flags).unwrap();
            assert!(self.cover_above());
            assert_eq!(GetWindowLongPtrW(self.overlay, GWL_EXSTYLE) & WS_EX_TOPMOST.0 as isize, 0);
        }

        unsafe fn cover_above(&self) -> bool {
            let mut current = GetWindow(self.overlay, GW_HWNDPREV).unwrap_or_default();
            for _ in 0..4096 {
                if current == self.cover { return true; }
                if current.is_invalid() { return false; }
                current = GetWindow(current, GW_HWNDPREV).unwrap_or_default();
            }
            panic!("unexpected window list length");
        }

        unsafe fn tick(&self) {
            SendMessageW(self.overlay, WM_TIMER, Some(WPARAM(1)), Some(LPARAM(0)));
        }

        unsafe fn allow_next_check(&self) {
            SetWindowLongPtrW(self.overlay,
                WINDOW_LONG_PTR_INDEX(std::mem::size_of::<isize>() as i32), 0);
        }

        unsafe fn painted_row(&self) -> Vec<u32> {
            use windows::Win32::Graphics::Gdi::*;
            #[link(name = "user32")]
            unsafe extern "system" {
                fn PrintWindow(hwnd: HWND, dc: HDC, flags: u32) -> i32;
            }
            let dc = GetDC(Some(self.overlay));
            let memory = CreateCompatibleDC(Some(dc));
            let bitmap = CreateCompatibleBitmap(dc, 100, 4);
            let previous = SelectObject(memory, bitmap.into());
            let printed = PrintWindow(self.overlay, memory, 0) != 0;
            let row = (0..100).map(|x| GetPixel(memory, x, 2).0).collect();
            let _ = SelectObject(memory, previous);
            let _ = DeleteObject(bitmap.into());
            let _ = DeleteDC(memory);
            let _ = ReleaseDC(Some(self.overlay), dc);
            assert!(printed, "capture only the disposable overlay's own rendering");
            row
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) { unsafe {
            SetWindowLongPtrW(self.overlay, WINDOW_LONG_PTR_INDEX(0), 0);
            let _ = DestroyWindow(self.cover);
            let _ = DestroyWindow(self.overlay);
            drop(Box::from_raw(self.state_ptr));
            let _ = UnregisterClassA(PCSTR(self.class.as_ptr()), Some(self.instance));
            let mut msg = MSG::default();
            let _ = PeekMessageW(&mut msg, None, WM_QUIT, WM_QUIT, PM_REMOVE);
        } }
    }

    #[test]
    fn continuously_visible_recovers_from_normal_cover_without_activation() { unsafe {
        // Approximation: ordinary native occlusion without TOPMOST. Windows did not
        // permit setting that bit independently of Z order. The installed one-raise
        // experiment supplies evidence for recovery with the misleading bit set.
        let foreground = GetForegroundWindow();
        let f = Fixture::new();
        f.renderer.update(0.6, (90, 85, 72));
        f.renderer.set_variant(0);
        f.renderer.show();
        assert!(f.renderer.state.lock().unwrap().visible);
        assert!(IsWindowVisible(f.overlay).as_bool());
        assert!(f.cover_above());
        let expected: Vec<_> = (0..100).map(|x| if x < 60 { 90 | (85 << 8) | (72 << 16) } else { 0 }).collect();
        assert_eq!(f.painted_row(), expected);
        f.tick();
        assert!(!f.cover_above(), "production timer must recover continuous-visible exposure");
        assert_ne!(GetWindowLongPtrW(f.overlay, GWL_EXSTYLE) & WS_EX_TOPMOST.0 as isize, 0);
        assert_eq!(GetForegroundWindow(), foreground);
        assert!(f.renderer.state.lock().unwrap().visible);
        assert_eq!(f.painted_row(), expected, "recovery preserves actual GDI output");
        // A second anomaly inside the same second must not cause a 60 Hz raise.
        f.normal_cover();
        f.renderer.show();
        f.tick();
        assert!(f.cover_above(), "exposure checks are limited to once per second");
        assert!(f.renderer.state.lock().unwrap().presentation_requested,
            "coalesce an actual show during cooldown instead of losing it");
        f.allow_next_check();
        f.tick();
        assert!(!f.cover_above());
        assert!(!f.renderer.state.lock().unwrap().presentation_requested);
        assert_eq!(GetForegroundWindow(), foreground);
    } }

    #[test]
    fn hidden_policy_and_intentional_topmost_cover_do_not_raise() { unsafe {
        let foreground = GetForegroundWindow();
        let f = Fixture::new();
        f.renderer.hide();
        f.tick();
        assert!(f.cover_above(), "Hidden must not raise the native bar");
        f.renderer.show();
        SetWindowPos(f.cover, Some(HWND_TOPMOST), 0, 0, 0, 0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER).unwrap();
        f.allow_next_check();
        f.tick();
        assert!(f.cover_above(), "intentional topmost cover takes precedence");
        assert!(!f.renderer.state.lock().unwrap().presentation_requested,
            "a blocked request is consumed, not retried by cached visible");
        f.normal_cover();
        f.allow_next_check();
        f.tick();
        assert!(f.cover_above(), "removing a blocker does not create a new presentation request");
        assert_eq!(GetForegroundWindow(), foreground);
    } }

    #[test]
    fn native_hidden_anomaly_recovers_without_activation_but_modal_cover_is_respected() { unsafe {
        let foreground = GetForegroundWindow();
        let f = Fixture::new();
        f.renderer.show();
        let _ = ShowWindow(f.overlay, SW_HIDE);
        f.tick();
        assert!(IsWindowVisible(f.overlay).as_bool());
        assert!(!f.cover_above());
        f.normal_cover();
        let style = GetWindowLongPtrW(f.cover, GWL_EXSTYLE);
        SetWindowLongPtrW(f.cover, GWL_EXSTYLE, style | WS_EX_DLGMODALFRAME.0 as isize);
        f.renderer.show();
        f.allow_next_check();
        f.tick();
        assert!(f.cover_above(), "modal surfaces take precedence");
        assert_eq!(GetForegroundWindow(), foreground);
    } }

    #[test]
    fn actual_policy_hidden_cases_reach_executor_and_never_raise() { unsafe {
        use crate::{communication_policy::{CommunicationPolicy, PolicyInput},
            communication_profile::CommunicationProfile, communication_types::OverlaySignal,
            ergonomic_profile::ErgonomicProfile, session::DeskState,
            session_manager::SessionManager, tray_signal_exec::execute_overlay};
        let foreground = GetForegroundWindow();
        let f = Fixture::new();
        let snapshot = SessionManager::new().snapshot();
        for (state, fresh, disabled) in [
            (DeskState::Away, true, false),
            (DeskState::Walking, true, false),
            (DeskState::Sitting, false, false),
            (DeskState::Sitting, true, true),
        ] {
            let mut profile = CommunicationProfile::default();
            if disabled {
                profile.baseline.sitting.overlay = "hidden".into();
                for step in &mut profile.escalation.sitting { step.overlay = "hidden".into(); }
            }
            let mut policy = CommunicationPolicy::new(profile, ErgonomicProfile::default());
            let signal = policy.evaluate(&PolicyInput {
                state, elapsed_secs: 2700, sensor_connected: true, observation_fresh: fresh,
                standing_lap_progress: 0.0, standing_lap: 0, standing_lap_flash: false,
                continuous_computer_secs: 0,
            }).overlay;
            assert!(matches!(signal, OverlaySignal::Hidden));
            f.renderer.show();
            execute_overlay(&signal, &f.renderer, &snapshot);
            f.allow_next_check();
            f.tick();
            assert!(!f.renderer.state.lock().unwrap().visible);
            assert!(f.cover_above());
        }
        assert_eq!(GetForegroundWindow(), foreground);
    } }

    #[test]
    fn sitting_baseline_numeric_reaches_executor_and_native_painted_width() { unsafe {
        use crate::{communication_policy::{CommunicationPolicy, PolicyInput},
            communication_profile::CommunicationProfile, communication_types::OverlaySignal,
            ergonomic_profile::ErgonomicProfile, session::DeskState,
            session_manager::SessionManager, tray_signal_exec::execute_overlay};
        let foreground = GetForegroundWindow();
        let f = Fixture::new();
        // Deliberately leave snapshot sitting load at zero: the executor must use policy output.
        let snapshot = SessionManager::new().snapshot();
        let now = std::time::Instant::now();
        for snoozed in [false, true] {
            for (elapsed, limit, expected_progress, expected_width) in [
                (319, 2400, 0.13291667, 13), (319, 1200, 0.26583334, 26),
                (0, 2400, 0.0, 1),
            ] {
                let mut policy = CommunicationPolicy::new(
                    CommunicationProfile::default(), ErgonomicProfile::default(),
                );
                if snoozed { policy.snooze_for_at(5, now); }
                let signals = policy.evaluate_with_stand_limit_fired_at(&PolicyInput {
                    state: DeskState::Sitting, elapsed_secs: elapsed,
                    sensor_connected: true, observation_fresh: true,
                    standing_lap_progress: 0.0, standing_lap: 0, standing_lap_flash: false,
                    continuous_computer_secs: 0,
                }, limit, 0, false, now);
                assert!(matches!(signals.overlay, OverlaySignal::Neutral { progress }
                    if (progress - expected_progress).abs() < 0.000001));
                assert!(signals.notify.is_none());
                execute_overlay(&signals.overlay, &f.renderer, &snapshot);
                let (r, g, b, _) = color_for_progress(expected_progress);
                let color = u32::from(r) | (u32::from(g) << 8) | (u32::from(b) << 16);
                let row = f.painted_row();
                assert!(row[..expected_width].iter().all(|pixel| *pixel == color));
                assert!(row[expected_width..].iter().all(|pixel| *pixel == 0),
                    "actual GDI fill must stop at {expected_width}px, snoozed={snoozed}");
            }
        }
        assert_eq!(GetForegroundWindow(), foreground);
    } }

    #[test]
    fn sitting_warning_limit_and_pulse_execute_policy_and_recover_native_exposure() { unsafe {
        use crate::{communication_policy::{CommunicationPolicy, PolicyInput},
            communication_profile::CommunicationProfile, communication_types::OverlaySignal,
            ergonomic_profile::ErgonomicProfile, session::DeskState,
            session_manager::SessionManager, tray_signal_exec::execute_overlay};
        let foreground = GetForegroundWindow();
        let f = Fixture::new();
        let snapshot = SessionManager::new().snapshot();
        let ergonomic = ErgonomicProfile::default();
        let limit = i64::from(ergonomic.limits.sitting_secs);
        let mut policy = CommunicationPolicy::new(CommunicationProfile::default(), ergonomic);
        for (offset, color, variant) in [
            (-600, (255, 193, 7), 0), (0, (244, 67, 54), 0), (300, (244, 67, 54), 2),
        ] {
            f.normal_cover();
            let signals = policy.evaluate(&PolicyInput {
                state: DeskState::Sitting, elapsed_secs: limit + offset,
                sensor_connected: true, observation_fresh: true,
                standing_lap_progress: 0.0, standing_lap: 0, standing_lap_flash: false,
                continuous_computer_secs: 0,
            });
            assert!(match offset {
                -600 => matches!(signals.overlay, OverlaySignal::Yellow { .. }),
                0 => matches!(signals.overlay, OverlaySignal::Red { .. }),
                _ => matches!(signals.overlay, OverlaySignal::PulseRed { .. }),
            });
            // Execute only the overlay channel. No notification side effect is invoked.
            execute_overlay(&signals.overlay, &f.renderer, &snapshot);
            {
                let state = f.renderer.state.lock().unwrap();
                assert!(state.visible);
                assert_eq!(state.color_rgb, color);
                assert_eq!(state.overlay_variant, variant);
                assert!((state.progress - ((limit + offset) as f32 / limit as f32).min(1.0)).abs() < 0.001);
            }
            f.allow_next_check();
            f.tick();
            assert!(!f.cover_above());
            assert_ne!(GetWindowLongPtrW(f.overlay, GWL_EXSTYLE) & WS_EX_TOPMOST.0 as isize, 0);
            assert_eq!(GetForegroundWindow(), foreground);
        }
    } }

    #[test]
    fn fullscreen_guard_uses_complete_monitor_bounds_including_negative_origins() {
        use crate::overlay_opaque::covers_monitor;
        let monitor = RECT { left: -1920, top: 0, right: 0, bottom: 1080 };
        assert!(covers_monitor(&monitor, &monitor));
        assert!(covers_monitor(&RECT { left: -1928, top: -8, right: 8, bottom: 1088 }, &monitor));
        assert!(!covers_monitor(&RECT { left: -1920, top: 0, right: 0, bottom: 1040 }, &monitor));
        assert!(!covers_monitor(&RECT { left: -1920, top: 0, right: 0, bottom: 4 }, &monitor));
    }

    #[test]
    fn silence_cached_visible_and_animation_dirty_cannot_repeat_recovery() { unsafe {
        let foreground = GetForegroundWindow();
        let f = Fixture::new();
        f.renderer.show();
        f.tick();
        assert!(!f.cover_above());
        f.normal_cover();
        f.allow_next_check();
        f.renderer.update(0.8, (90, 85, 72)); // dirty pixels are not a presentation request
        f.renderer.start_lap_flash();
        f.renderer.state.lock().unwrap().lap_flash_until =
            Some(std::time::Instant::now() - std::time::Duration::from_secs(1));
        f.tick();
        assert!(f.renderer.state.lock().unwrap().needs_redraw);
        assert!(f.cover_above(), "cached visible and animation must not revive a consumed request");
        let _ = ShowWindow(f.overlay, SW_HIDE);
        f.allow_next_check();
        f.tick();
        assert!(!IsWindowVisible(f.overlay).as_bool(), "silence must not show the native window");
        f.renderer.show();
        f.renderer.show(); // actual requests coalesce
        f.tick();
        assert!(IsWindowVisible(f.overlay).as_bool());
        assert!(!f.cover_above());
        assert_eq!(GetForegroundWindow(), foreground);
    } }

    #[test]
    fn hidden_cancels_show_queued_during_cooldown() { unsafe {
        let foreground = GetForegroundWindow();
        let f = Fixture::new();
        f.renderer.show();
        f.tick();
        assert!(!f.cover_above());
        f.normal_cover();
        f.renderer.show();
        f.tick();
        assert!(f.cover_above());
        assert!(f.renderer.state.lock().unwrap().presentation_requested);
        f.renderer.hide();
        f.allow_next_check();
        f.tick();
        assert!(f.cover_above());
        assert!(!f.renderer.state.lock().unwrap().presentation_requested);
        assert_eq!(GetForegroundWindow(), foreground);
    } }

    #[test]
    fn known_zero_global_alpha_is_not_an_ordinary_cover() { unsafe {
        let foreground = GetForegroundWindow();
        let f = Fixture::new();
        let style = GetWindowLongPtrW(f.cover, GWL_EXSTYLE);
        SetWindowLongPtrW(f.cover, GWL_EXSTYLE, style | WS_EX_LAYERED.0 as isize);
        SetLayeredWindowAttributes(f.cover, COLORREF(0), 0, LWA_ALPHA).unwrap();
        f.renderer.show();
        f.tick();
        assert!(f.cover_above(), "alpha-zero ordinary surface does not occlude pixels");
        SetLayeredWindowAttributes(f.cover, COLORREF(0), 255, LWA_ALPHA).unwrap();
        f.renderer.show();
        f.allow_next_check();
        f.tick();
        assert!(!f.cover_above(), "opaque ordinary surface does require recovery");
        assert_eq!(GetForegroundWindow(), foreground);
    } }

    #[test]
    fn known_cloaked_surface_is_neither_ordinary_cover_nor_topmost_blocker() { unsafe {
        use windows::Win32::Graphics::Dwm::*;
        let foreground = GetForegroundWindow();
        let f = Fixture::new();
        let cloak = 1u32;
        DwmSetWindowAttribute(f.cover, DWMWA_CLOAK, &cloak as *const _ as _, 4).unwrap();
        let mut observed = 0u32;
        DwmGetWindowAttribute(f.cover, DWMWA_CLOAKED, &mut observed as *mut _ as _, 4).unwrap();
        assert_ne!(observed, 0, "fixture must actually be cloaked");
        f.renderer.show();
        f.tick();
        assert!(f.cover_above(), "cloaked ordinary window is not pixel occlusion");
        SetWindowPos(f.cover, Some(HWND_TOPMOST), 0, 0, 0, 0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER).unwrap();
        let _ = ShowWindow(f.overlay, SW_HIDE);
        f.renderer.show();
        f.allow_next_check();
        f.tick();
        assert!(IsWindowVisible(f.overlay).as_bool(), "cloaked topmost must not block native-hidden recovery");
        assert_eq!(GetForegroundWindow(), foreground);
    } }

    #[test]
    fn zero_alpha_topmost_does_not_block_but_transparent_style_alone_still_blocks() { unsafe {
        let foreground = GetForegroundWindow();
        let f = Fixture::new();
        let style = GetWindowLongPtrW(f.cover, GWL_EXSTYLE);
        SetWindowLongPtrW(f.cover, GWL_EXSTYLE,
            style | (WS_EX_LAYERED.0 | WS_EX_TRANSPARENT.0) as isize);
        SetLayeredWindowAttributes(f.cover, COLORREF(0), 0, LWA_ALPHA).unwrap();
        SetWindowPos(f.cover, Some(HWND_TOPMOST), 0, 0, 0, 0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER).unwrap();
        let _ = ShowWindow(f.overlay, SW_HIDE);
        f.renderer.show();
        f.tick();
        assert!(IsWindowVisible(f.overlay).as_bool(), "known alpha-zero topmost must be ignored");
        SetLayeredWindowAttributes(f.cover, COLORREF(0), 255, LWA_ALPHA).unwrap();
        SetWindowPos(f.cover, Some(HWND_TOPMOST), 0, 0, 0, 0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER).unwrap();
        let _ = ShowWindow(f.overlay, SW_HIDE);
        f.renderer.show();
        f.allow_next_check();
        f.tick();
        assert!(!IsWindowVisible(f.overlay).as_bool(), "TRANSPARENT style does not imply pixel transparency");
        assert_eq!(GetForegroundWindow(), foreground);
    } }
}
