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
///
/// Two implementations available:
/// - OPAQUE mode: Simple black background, fallback (reliable)
/// - LAYERED mode: Transparent with UpdateLayeredWindow (experimental)
///
/// Set `OVERLAY_MODE=layered` env var to switch (default: opaque)
#[cfg(target_os = "windows")]
fn run_event_loop(state: Arc<Mutex<OverlayState>>) {
    let mode = std::env::var("OVERLAY_MODE").unwrap_or_else(|_| "opaque".to_string());
    match mode.as_str() {
        "layered" => {
            info!("🎨 [EXPERIMENTAL] Starting overlay in LAYERED mode (UpdateLayeredWindow)");
            run_event_loop_layered(state);
        }
        _ => {
            info!("🎨 [STABLE] Starting overlay in OPAQUE mode (black background)");
            run_event_loop_opaque(state);
        }
    }
}

/// Opaque overlay: Black background, standard GDI rendering (STABLE)
///
/// ⚠️ No transparency, but reliable rendering.
#[cfg(target_os = "windows")]
fn run_event_loop_opaque(state: Arc<Mutex<OverlayState>>) {
    use windows::Win32::Foundation::*;
    use windows::Win32::Graphics::Gdi::*;
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::*;
    use windows::core::PCSTR;

    const CLASS_NAME: &[u8] = b"zntlOverlayBar\0";
    const WINDOW_NAME: &[u8] = b"zntl Overlay\0";
    const TIMER_ID: usize = 1;
    const TIMER_INTERVAL_MS: u32 = 16; // ~60fps

    unsafe {
        // 1. Get primary monitor dimensions using GetMonitorInfo
        let hmonitor = MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY);
        let mut monitor_info: MONITORINFO = std::mem::zeroed();
        monitor_info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;

        let screen_width = if GetMonitorInfoW(hmonitor, &mut monitor_info).as_bool() {
            monitor_info.rcMonitor.right - monitor_info.rcMonitor.left
        } else {
            1920 // fallback
        };
        let screen_height = 4i32;

        // 2. Register window class
        let hmodule = GetModuleHandleW(None).unwrap_or(HMODULE::default());
        let hinstance: HINSTANCE = hmodule.into();

        let wnd_class = WNDCLASSA {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: std::mem::size_of::<isize>() as i32,
            hInstance: hinstance,
            hIcon: HICON::default(),
            hCursor: HCURSOR::default(),
            hbrBackground: HBRUSH(GetStockObject(BLACK_BRUSH).0),
            lpszMenuName: PCSTR::null(),
            lpszClassName: PCSTR(CLASS_NAME.as_ptr()),
        };

        let class_atom = RegisterClassA(&wnd_class);
        if class_atom == 0 {
            info!("⚠️ Failed to register window class");
            return;
        }

        // 3. Create window at (0,0) with screen_width × 4px
        // Note: Removed WS_EX_LAYERED as it requires UpdateLayeredWindow for rendering.
        // Using standard WS_POPUP allows normal GDI drawing via BeginPaint/FillRect.
        let hwnd = match CreateWindowExA(
            WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
            PCSTR(CLASS_NAME.as_ptr()),
            PCSTR(WINDOW_NAME.as_ptr()),
            WS_POPUP | WS_VISIBLE,
            0,                  // x
            0,                  // y
            screen_width,       // width
            screen_height,      // height
            None,               // parent
            None,               // menu
            Some(hinstance),
            None,
        ) {
            Ok(h) => h,
            Err(_) => {
                info!("⚠️ Failed to create overlay window");
                return;
            }
        };

        // 4. Store state Arc pointer in window userdata (cbWndExtra)
        let state_ptr = Box::into_raw(Box::new(state.clone()));
        SetWindowLongPtrW(hwnd, WINDOW_LONG_PTR_INDEX(0), state_ptr as isize);

        // 5. Make window visible and set timer
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = UpdateWindow(hwnd);

        if SetTimer(Some(hwnd), TIMER_ID, TIMER_INTERVAL_MS, None) == 0 {
            info!("⚠️ Failed to set timer");
            let _ = DestroyWindow(hwnd);
            return;
        }

        info!(
            "🎨 WinAPI overlay window created: {}x{} @ (0,0), timer={}ms",
            screen_width, screen_height, TIMER_INTERVAL_MS
        );

        // 6. Message loop
        let mut msg: MSG = std::mem::zeroed();
        loop {
            let msg_result = GetMessageW(&mut msg, None, 0, 0);

            if msg_result.0 == 0 {
                // WM_QUIT received, exit loop
                break;
            } else if msg_result.0 < 0 {
                info!("⚠️ GetMessageW error");
                break;
            }

            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Cleanup
        let _ = DestroyWindow(hwnd);
        let _ = UnregisterClassA(PCSTR(CLASS_NAME.as_ptr()), Some(hinstance));
        let _ = Box::from_raw(state_ptr); // reclaim Arc
    }
}

/// Window procedure — handles window messages for the overlay.
///
/// Accessed state pointer stored in cbWndExtra via GetWindowLongPtrW(hwnd, 0).
#[cfg(target_os = "windows")]
unsafe extern "system" fn wnd_proc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::Foundation::*;
    use windows::Win32::Graphics::Gdi::*;
    use windows::Win32::UI::WindowsAndMessaging::*;

    match msg {
        WM_TIMER => {
            // WM_TIMER: Check if state needs redraw, invalidate if dirty
            let state_ptr = GetWindowLongPtrW(hwnd, WINDOW_LONG_PTR_INDEX(0)) as *mut Arc<Mutex<OverlayState>>;
            if !state_ptr.is_null() {
                let state = &*state_ptr;
                if let Ok(mut s) = state.lock() {
                    if s.needs_redraw {
                        log::debug!("WM_TIMER: Invalidating rect (progress={}, visible={})", s.progress, s.visible);
                        let _ = InvalidateRect(Some(hwnd), None, false.into());
                        s.needs_redraw = false;
                    }
                }
            }
            LRESULT(0)
        }

        WM_PAINT => {
            log::info!("WM_PAINT: Painting overlay");
            // WM_PAINT: Draw the progress bar
            let state_ptr = GetWindowLongPtrW(hwnd, WINDOW_LONG_PTR_INDEX(0)) as *mut Arc<Mutex<OverlayState>>;
            if !state_ptr.is_null() {
                let state = &*state_ptr;
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(hwnd, &mut ps);

                if let Ok(s) = state.lock() {
                    // Get window dimensions
                    let mut rect: RECT = std::mem::zeroed();
                    let _ = GetClientRect(hwnd, &mut rect);
                    let window_width = rect.right - rect.left;
                    let window_height = rect.bottom - rect.top;
                    log::debug!("WM_PAINT: window={}x{}, progress={}, visible={}", window_width, window_height, s.progress, s.visible);

                    // Calculate bar width (progress × window_width)
                    let bar_width = ((window_width as f32) * s.progress.max(0.0).min(1.0)) as i32;
                    log::info!("WM_PAINT: bar_width={} (progress={}), visible={}", bar_width, s.progress, s.visible);

                    if s.visible {
                        // Create brush with RGB color
                        // RGB(r, g, b) in Windows = r | (g << 8) | (b << 16)
                        let color = COLORREF(
                            (s.color_rgb.0 as u32)
                                | ((s.color_rgb.1 as u32) << 8)
                                | ((s.color_rgb.2 as u32) << 16),
                        );
                        log::info!("WM_PAINT: Creating brush with RGB({}, {}, {})", s.color_rgb.0, s.color_rgb.1, s.color_rgb.2);
                        let brush = CreateSolidBrush(color);
                        log::info!("WM_PAINT: brush.is_invalid()={}", brush.is_invalid());

                        // Draw progress bar
                        if bar_width > 0 && !brush.is_invalid() {
                            log::info!("WM_PAINT: Drawing progress bar at {}px", bar_width);
                            let bar_rect = RECT {
                                left: 0,
                                top: 0,
                                right: bar_width,
                                bottom: window_height,
                            };
                            let _ = FillRect(hdc, &bar_rect, brush);
                        }

                        // Clean up brush
                        if !brush.is_invalid() {
                            let _ = DeleteObject(brush.into());
                        }

                        // Fill background (black)
                        if bar_width < window_width {
                            let bg_rect = RECT {
                                left: bar_width,
                                top: 0,
                                right: window_width,
                                bottom: window_height,
                            };
                            let black_brush = GetStockObject(BLACK_BRUSH);
                            if !black_brush.is_invalid() {
                                let _ = FillRect(hdc, &bg_rect, HBRUSH(black_brush.0));
                            }
                        }
                    } else {
                        // Window hidden: fill entire area with black
                        let black_brush = GetStockObject(BLACK_BRUSH);
                        if !black_brush.is_invalid() {
                            let _ = FillRect(hdc, &rect, HBRUSH(black_brush.0));
                        }
                    }
                }

                let _ = EndPaint(hwnd, &ps);
            }
            LRESULT(0)
        }

        WM_DESTROY => {
            // Window is being destroyed, quit the message loop
            PostQuitMessage(0);
            LRESULT(0)
        }

        _ => {
            // Default window procedure for other messages
            DefWindowProcA(hwnd, msg, wparam, lparam)
        }
    }
}

/// Layered overlay: Transparent with UpdateLayeredWindow (EXPERIMENTAL)
///
/// ⚠️ Uses WS_EX_LAYERED + UpdateLayeredWindow for transparency.
/// Requires 32-bit ARGB bitmap and more complex rendering pipeline.
///
/// TODO (V3): Implement proper layered window rendering with offscreen DC
/// and per-pixel alpha blending for smooth transparency.
#[cfg(target_os = "windows")]
fn run_event_loop_layered(state: Arc<Mutex<OverlayState>>) {
    info!("⚠️ [EXPERIMENTAL] Layered mode not yet implemented");
    info!("   Falling back to opaque mode for now");
    // TODO: Implement UpdateLayeredWindow approach
    // For now, fall back to opaque to avoid hanging
    run_event_loop_opaque(state);
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
    use crate::colors::color_for_progress;

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
