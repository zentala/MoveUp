//! overlay_renderer.rs — System-level progress bar overlay using raw WinAPI.
//!
//! Creates a native Windows window (NOT Tauri WebviewWindow) at position (0,0).
//! Window is N px tall × full screen width, always-on-top, no decorations.
//! Height configurable via OVERLAY_HEIGHT env var (1-20, default 4).
//!
//! ⚠️ CRITICAL: CreateWindowExW() MUST be called in the SAME THREAD as the
//! message loop (PeekMessage/DispatchMessage). Never call it from another thread.
//! All WinAPI window operations stay inside run_event_loop().

use std::sync::{Arc, Mutex};
use log::info;

/// Data source for overlay progress bar.
///
/// ```text
/// Demo:  WM_TIMER → demo_progress(frame) → cycling animation
/// Live:  serial.rs → session.rs → tray_controller.rs → overlay.update()
/// Mock:  WM_TIMER → mock_progress(frame) → simulated sit/stand cycle
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataSource {
    /// Cycling demo animation (0%→25%→50%→75%→100%). Default in debug builds.
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
}

/// Parses `OVERLAY_DATA` env var into a [`DataSource`].
///
/// Default: `Demo` in debug builds, `Live` in release builds.
fn parse_data_source() -> DataSource {
    let default = if cfg!(debug_assertions) { DataSource::Demo } else { DataSource::Live };
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
    log::info!("[OVERLAY] DataSource={:?} (OVERLAY_DATA env={:?}, debug={})",
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
            color_rgb: (76, 175, 80), // green
            visible: false,
            needs_redraw: false,
            frame_count: 0,
            data_source,
            bar_height,
            overlay_variant,
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
}

/// Calculates demo progress and color from frame count.
/// Cycles: 0% → 25% → 50% → 75% → 100% every 5 seconds (300 frames @ 60fps).
fn demo_progress(frame_count: u32) -> (f32, (u8, u8, u8)) {
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
fn mock_progress(frame_count: u32) -> (f32, (u8, u8, u8), bool) {
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
        (0.0, (76, 175, 80), false)
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
    let (data_source, bar_height) = state.lock()
        .map(|s| (s.data_source, s.bar_height))
        .unwrap_or((DataSource::Demo, 4));

    info!("[OVERLAY] data_source={:?}, render_mode={}, height={}", data_source, mode, bar_height);

    match mode.as_str() {
        "layered" => run_event_loop_layered(state, bar_height),
        _ => run_event_loop_opaque(state, bar_height),
    }
}

/// Opaque overlay: Black background, standard GDI rendering (STABLE)
///
/// ⚠️ No transparency, but reliable rendering.
#[cfg(target_os = "windows")]
fn run_event_loop_opaque(state: Arc<Mutex<OverlayState>>, bar_height: i32) {
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

        let (screen_width, screen_height, screen_x, screen_y) = if GetMonitorInfoW(hmonitor, &mut monitor_info).as_bool() {
            (
                monitor_info.rcMonitor.right - monitor_info.rcMonitor.left,
                bar_height,
                monitor_info.rcMonitor.left,
                monitor_info.rcMonitor.top,
            )
        } else {
            (1920, bar_height, 0, 0)
        };

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

        // 3. Create window at monitor position with screen_width x 4px
        let hwnd = match CreateWindowExA(
            WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
            PCSTR(CLASS_NAME.as_ptr()),
            PCSTR(WINDOW_NAME.as_ptr()),
            WS_POPUP | WS_VISIBLE,
            screen_x,           // x
            screen_y,           // y
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
            "[OPAQUE] Overlay window created: {}x{} @ ({},{}), timer={}ms",
            screen_width, screen_height, screen_x, screen_y, TIMER_INTERVAL_MS
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
            // WM_TIMER: Increment animation frame and invalidate
            let state_ptr = GetWindowLongPtrW(hwnd, WINDOW_LONG_PTR_INDEX(0)) as *mut Arc<Mutex<OverlayState>>;
            if !state_ptr.is_null() {
                let state = &*state_ptr;
                if let Ok(mut s) = state.lock() {
                    s.frame_count = s.frame_count.wrapping_add(1);

                    match s.data_source {
                        DataSource::Demo => {
                            let (progress, color) = demo_progress(s.frame_count);
                            s.progress = progress;
                            s.color_rgb = color;
                            s.visible = true;
                        }
                        DataSource::Mock => {
                            let (progress, color, visible) = mock_progress(s.frame_count);
                            s.progress = progress;
                            s.color_rgb = color;
                            s.visible = visible;
                        }
                        DataSource::Live => {} // External updates via update()/show()/hide()
                    }

                    let _ = InvalidateRect(Some(hwnd), None, false.into());
                }
            }
            LRESULT(0)
        }

        WM_PAINT => {
            let state_ptr = GetWindowLongPtrW(hwnd, WINDOW_LONG_PTR_INDEX(0)) as *mut Arc<Mutex<OverlayState>>;
            if !state_ptr.is_null() {
                let state = &*state_ptr;
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(hwnd, &mut ps);

                if let Ok(s) = state.lock() {
                    let mut rect: RECT = std::mem::zeroed();
                    let _ = GetClientRect(hwnd, &mut rect);
                    let window_width = rect.right - rect.left;
                    let window_height = rect.bottom - rect.top;

                    // Fill background with black
                    let black_brush = CreateSolidBrush(COLORREF(0));
                    if !black_brush.is_invalid() {
                        let _ = FillRect(hdc, &rect, black_brush);
                        let _ = DeleteObject(black_brush.into());
                    }

                    // Draw progress bar with variant-aware rendering
                    if s.visible {
                        let bar_width = ((window_width as f32) * s.progress.clamp(0.0, 1.0)) as i32;
                        let bar_width = if s.data_source != DataSource::Live { bar_width.max(1) } else { bar_width };

                        match s.overlay_variant {
                            0 => {
                                // Solid fill (default behavior)
                                let color = COLORREF(
                                    (s.color_rgb.0 as u32)
                                        | ((s.color_rgb.1 as u32) << 8)
                                        | ((s.color_rgb.2 as u32) << 16),
                                );
                                let brush = CreateSolidBrush(color);
                                if bar_width > 0 && !brush.is_invalid() {
                                    let bar_rect = RECT { left: 0, top: 0, right: bar_width, bottom: window_height };
                                    let _ = FillRect(hdc, &bar_rect, brush);
                                }
                                if !brush.is_invalid() { let _ = DeleteObject(brush.into()); }
                            }
                            1 => {
                                // Gradient: dark to bright, left to right
                                for x in 0..bar_width {
                                    let factor = x as f32 / bar_width.max(1) as f32;
                                    let r = (s.color_rgb.0 as f32 * factor) as u8;
                                    let g = (s.color_rgb.1 as f32 * factor) as u8;
                                    let b = (s.color_rgb.2 as f32 * factor) as u8;
                                    let col_color = COLORREF((r as u32) | ((g as u32) << 8) | ((b as u32) << 16));
                                    let brush = CreateSolidBrush(col_color);
                                    if !brush.is_invalid() {
                                        let col_rect = RECT { left: x, top: 0, right: x + 1, bottom: window_height };
                                        let _ = FillRect(hdc, &col_rect, brush);
                                        let _ = DeleteObject(brush.into());
                                    }
                                }
                            }
                            2 => {
                                // Pulsing: modulate brightness using frame_count
                                let pulse = (s.frame_count as f32 * 0.1).sin() * 0.3 + 0.7; // 0.4 to 1.0
                                let r = (s.color_rgb.0 as f32 * pulse) as u8;
                                let g = (s.color_rgb.1 as f32 * pulse) as u8;
                                let b = (s.color_rgb.2 as f32 * pulse) as u8;
                                let pulsed_color = COLORREF((r as u32) | ((g as u32) << 8) | ((b as u32) << 16));
                                let brush = CreateSolidBrush(pulsed_color);
                                if bar_width > 0 && !brush.is_invalid() {
                                    let bar_rect = RECT { left: 0, top: 0, right: bar_width, bottom: window_height };
                                    let _ = FillRect(hdc, &bar_rect, brush);
                                }
                                if !brush.is_invalid() { let _ = DeleteObject(brush.into()); }
                            }
                            _ => {} // Unknown variant, render nothing extra
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

/// Layered buffer — offscreen 32-bit ARGB DIBSection for UpdateLayeredWindow
#[cfg(target_os = "windows")]
struct LayeredBufferState {
    hdc_screen: windows::Win32::Graphics::Gdi::HDC, // screen DC (cached from GetDC(None) at init)
    hdc_mem: windows::Win32::Graphics::Gdi::HDC,
    hbm: windows::Win32::Graphics::Gdi::HBITMAP,
    old_hbm: windows::Win32::Graphics::Gdi::HGDIOBJ,
    bits_ptr: *mut u32, // BGRA pixel data (32-bit per pixel)
    width: i32,
    height: i32,
}

/// Layered overlay: Transparent with UpdateLayeredWindow (EXPERIMENTAL)
///
/// ⚠️ Uses WS_EX_LAYERED + UpdateLayeredWindow for transparency.
/// Renders to offscreen 32-bit ARGB bitmap, composites with per-pixel alpha.
#[cfg(target_os = "windows")]
fn run_event_loop_layered(state: Arc<Mutex<OverlayState>>, bar_height: i32) {
    use windows::Win32::Foundation::*;
    use windows::Win32::Graphics::Gdi::*;
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::*;
    use windows::core::PCSTR;

    const CLASS_NAME: &[u8] = b"zntlOverlayBarLayered\0";
    const WINDOW_NAME: &[u8] = b"zntl Overlay (Layered)\0";
    const TIMER_ID: usize = 1;
    const TIMER_INTERVAL_MS: u32 = 16; // ~60fps
    const SLOT_STATE: WINDOW_LONG_PTR_INDEX = WINDOW_LONG_PTR_INDEX(0);
    const SLOT_BUF: WINDOW_LONG_PTR_INDEX = WINDOW_LONG_PTR_INDEX(8);

    unsafe {
        // 1. Get primary monitor dimensions
        let hmonitor = MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY);
        let mut monitor_info: MONITORINFO = std::mem::zeroed();
        monitor_info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;

        let (screen_width, screen_height, screen_x, screen_y) = if GetMonitorInfoW(hmonitor, &mut monitor_info).as_bool() {
            (
                monitor_info.rcMonitor.right - monitor_info.rcMonitor.left,
                bar_height,
                monitor_info.rcMonitor.left,
                monitor_info.rcMonitor.top,
            )
        } else {
            (1920, bar_height, 0, 0)
        };

        // 2. Register window class with layered proc
        let hmodule = GetModuleHandleW(None).unwrap_or(HMODULE::default());
        let hinstance: HINSTANCE = hmodule.into();

        let wnd_class = WNDCLASSA {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc_layered),
            cbClsExtra: 0,
            cbWndExtra: 2 * std::mem::size_of::<isize>() as i32, // 16 bytes total: 2 slots
            hInstance: hinstance,
            hIcon: HICON::default(),
            hCursor: HCURSOR::default(),
            hbrBackground: HBRUSH::default(), // null — no OS background
            lpszMenuName: PCSTR::null(),
            lpszClassName: PCSTR(CLASS_NAME.as_ptr()),
        };

        let class_atom = RegisterClassA(&wnd_class);
        if class_atom == 0 {
            info!("⚠️ [LAYERED] Failed to register window class");
            return;
        }

        // 3. Create layered window at monitor's actual position (not (0,0)!)
        let hwnd = match CreateWindowExA(
            WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
            PCSTR(CLASS_NAME.as_ptr()),
            PCSTR(WINDOW_NAME.as_ptr()),
            WS_POPUP, // No WS_VISIBLE yet — show after first draw
            screen_x,
            screen_y,
            screen_width,
            screen_height,
            None,
            None,
            Some(hinstance),
            None,
        ) {
            Ok(h) => h,
            Err(_) => {
                info!("⚠️ [LAYERED] Failed to create window");
                return;
            }
        };

        // 4. Get screen DC for CreateDIBSection
        let hdc_screen = GetDC(None);

        // 5. Create 32-bit ARGB DIBSection (top-down: biHeight negative)
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: screen_width,
                biHeight: -screen_height, // negative = top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [RGBQUAD::default()],
        };

        let mut bits_void_ptr = std::ptr::null_mut();
        let hbm = match CreateDIBSection(Some(hdc_screen), &bmi, DIB_RGB_COLORS, &mut bits_void_ptr, None, 0) {
            Ok(h) => h,
            Err(_) => {
                info!("⚠️ [LAYERED] Failed to create DIBSection");
                ReleaseDC(None, hdc_screen);
                let _ = DestroyWindow(hwnd);
                return;
            }
        };

        let bits_ptr = bits_void_ptr as *mut u32;

        // 6. Create memory DC and select bitmap
        let hdc_mem = CreateCompatibleDC(Some(hdc_screen));
        if hdc_mem.is_invalid() {
            info!("⚠️ [LAYERED] Failed to create memory DC");
            ReleaseDC(None, hdc_screen);
            let _ = DeleteObject(hbm.into());
            let _ = DestroyWindow(hwnd);
            return;
        }

        let old_hbm = SelectObject(hdc_mem, hbm.into());

        // 7. Create LayeredBufferState and store in window (with cached screen DC)
        let buf_state = LayeredBufferState {
            hdc_screen,
            hdc_mem,
            hbm,
            old_hbm,
            bits_ptr,
            width: screen_width,
            height: screen_height,
        };

        let state_ptr = Box::into_raw(Box::new(state.clone()));
        let buf_ptr = Box::into_raw(Box::new(buf_state));

        SetWindowLongPtrW(hwnd, SLOT_STATE, state_ptr as isize);
        SetWindowLongPtrW(hwnd, SLOT_BUF, buf_ptr as isize);

        // 8. Draw initial frame before showing
        info!("[LAYERED] Drawing initial frame...");
        draw_layered_frame(hwnd, &state, &*buf_ptr);
        info!("[LAYERED] Initial frame drawn");

        // 9. Show window after first draw
        info!("[LAYERED] Calling ShowWindow(SW_SHOW)...");
        let show_result = ShowWindow(hwnd, SW_SHOW);
        info!("[LAYERED] ShowWindow returned: {:?}", show_result);
        let update_result = UpdateWindow(hwnd);
        info!("[LAYERED] UpdateWindow returned: {:?}", update_result);

        // 10. Set timer (no InvalidateRect — layered windows handle redraw differently)
        if SetTimer(Some(hwnd), TIMER_ID, TIMER_INTERVAL_MS, None) == 0 {
            info!("⚠️ [LAYERED] Failed to set timer");
            let _ = DestroyWindow(hwnd);
            return;
        }

        info!(
            "🎨 [LAYERED] WinAPI overlay window created: {}x{} @ ({},{}), UpdateLayeredWindow mode",
            screen_width, screen_height, screen_x, screen_y
        );

        // 11. Message loop
        let mut msg: MSG = std::mem::zeroed();
        loop {
            let msg_result = GetMessageW(&mut msg, None, 0, 0);

            if msg_result.0 == 0 {
                break;
            } else if msg_result.0 < 0 {
                info!("⚠️ [LAYERED] GetMessageW error");
                break;
            }

            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Cleanup
        let _ = DestroyWindow(hwnd);
        let _ = UnregisterClassA(PCSTR(CLASS_NAME.as_ptr()), Some(hinstance));
        let _ = Box::from_raw(state_ptr);
        let _ = Box::from_raw(buf_ptr);
    }
}

/// Draw a frame to the layered buffer and composite to screen with UpdateLayeredWindow
#[cfg(target_os = "windows")]
unsafe fn draw_layered_frame(
    hwnd: windows::Win32::Foundation::HWND,
    state: &Arc<Mutex<OverlayState>>,
    buf: &LayeredBufferState,
) {
    use windows::Win32::Foundation::*;
    use windows::Win32::Graphics::Gdi::*;
    use windows::Win32::UI::WindowsAndMessaging::*;

    // 1. Clear entire buffer: all pixels transparent (alpha=0)
    let pixel_count = (buf.width * buf.height) as usize;
    std::ptr::write_bytes(buf.bits_ptr, 0, pixel_count);

    // 2. Lock state, snapshot progress/visible/color/data_source/variant/frame_count
    let (bar_progress, visible, color_rgb, data_source, overlay_variant, frame_count) = if let Ok(s) = state.lock() {
        (s.progress, s.visible, s.color_rgb, s.data_source, s.overlay_variant, s.frame_count)
    } else {
        log::error!("[DRAW] Failed to acquire state lock!");
        return;
    };

    // 3. Calculate bar width
    let bar_width = if visible {
        let w = ((buf.width as f32) * bar_progress.clamp(0.0, 1.0)) as i32;
        if data_source != DataSource::Live { w.max(1) } else { w }
    } else {
        0
    };

    // 4. Fill bar pixels with BGRA (pre-multiplied alpha), variant-aware
    let alpha = 200u32;

    match overlay_variant {
        0 => {
            // Solid fill (default)
            let r = ((color_rgb.0 as u32) * alpha / 255) as u32;
            let g = ((color_rgb.1 as u32) * alpha / 255) as u32;
            let b = ((color_rgb.2 as u32) * alpha / 255) as u32;
            let pixel = (alpha << 24) | (r << 16) | (g << 8) | b;
            for y in 0..buf.height {
                for x in 0..bar_width {
                    let idx = (y * buf.width + x) as usize;
                    *buf.bits_ptr.add(idx) = pixel;
                }
            }
        }
        1 => {
            // Gradient: dark to bright, left to right
            for y in 0..buf.height {
                for x in 0..bar_width {
                    let factor = x as f32 / bar_width.max(1) as f32;
                    let r = ((color_rgb.0 as f32 * factor) as u32 * alpha / 255) as u32;
                    let g = ((color_rgb.1 as f32 * factor) as u32 * alpha / 255) as u32;
                    let b = ((color_rgb.2 as f32 * factor) as u32 * alpha / 255) as u32;
                    let pixel = (alpha << 24) | (r << 16) | (g << 8) | b;
                    let idx = (y * buf.width + x) as usize;
                    *buf.bits_ptr.add(idx) = pixel;
                }
            }
        }
        2 => {
            // Pulsing: modulate brightness by frame_count
            let pulse = (frame_count as f32 * 0.1).sin() * 0.3 + 0.7; // 0.4 to 1.0
            let r = ((color_rgb.0 as f32 * pulse) as u32 * alpha / 255) as u32;
            let g = ((color_rgb.1 as f32 * pulse) as u32 * alpha / 255) as u32;
            let b = ((color_rgb.2 as f32 * pulse) as u32 * alpha / 255) as u32;
            let pixel = (alpha << 24) | (r << 16) | (g << 8) | b;
            for y in 0..buf.height {
                for x in 0..bar_width {
                    let idx = (y * buf.width + x) as usize;
                    *buf.bits_ptr.add(idx) = pixel;
                }
            }
        }
        _ => {} // Unknown variant, leave transparent
    }

    // 5. Use cached screen DC and call UpdateLayeredWindow
    let src_point = POINT { x: 0, y: 0 };
    let size = SIZE {
        cx: buf.width,
        cy: buf.height,
    };
    let blend = BLENDFUNCTION {
        BlendOp: 0,
        BlendFlags: 0,
        SourceConstantAlpha: 255,
        AlphaFormat: 1, // AC_SRC_ALPHA
    };

    // Window position on screen (must be explicit, not None)
    let dst_point = POINT { x: 0, y: 0 };

    match UpdateLayeredWindow(
        hwnd,
        Some(buf.hdc_screen),
        Some(&dst_point),  // ← Explicit position
        Some(&size),
        Some(buf.hdc_mem),
        Some(&src_point),
        COLORREF(0),
        Some(&blend),
        ULW_ALPHA,
    ) {
        Ok(_) => {}
        Err(e) => {
            log::warn!("[LAYERED] UpdateLayeredWindow failed: {:?}", e);
        }
    }
}

/// Window procedure for layered overlay
#[cfg(target_os = "windows")]
unsafe extern "system" fn wnd_proc_layered(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::Foundation::*;
    use windows::Win32::Graphics::Gdi::*;
    use windows::Win32::UI::WindowsAndMessaging::*;

    const SLOT_STATE: WINDOW_LONG_PTR_INDEX = WINDOW_LONG_PTR_INDEX(0);
    const SLOT_BUF: WINDOW_LONG_PTR_INDEX = WINDOW_LONG_PTR_INDEX(8);

    match msg {
        WM_TIMER => {
            // Increment frame counter and redraw
            // ⚠️ Unlike OPAQUE (which uses InvalidateRect), we must redraw every frame
            // because UpdateLayeredWindow doesn't use the paint pipeline.
            let state_ptr = GetWindowLongPtrW(hwnd, SLOT_STATE) as *mut Arc<Mutex<OverlayState>>;
            let buf_ptr = GetWindowLongPtrW(hwnd, SLOT_BUF) as *mut LayeredBufferState;

            if !state_ptr.is_null() && !buf_ptr.is_null() {
                let state = &*state_ptr;
                let buf = &*buf_ptr;

                if let Ok(mut s) = state.lock() {
                    s.frame_count = s.frame_count.wrapping_add(1);
                    match s.data_source {
                        DataSource::Demo => {
                            let (progress, color) = demo_progress(s.frame_count);
                            s.progress = progress;
                            s.color_rgb = color;
                            s.visible = true;
                        }
                        DataSource::Mock => {
                            let (progress, color, visible) = mock_progress(s.frame_count);
                            s.progress = progress;
                            s.color_rgb = color;
                            s.visible = visible;
                        }
                        DataSource::Live => {}
                    }
                    drop(s);
                    draw_layered_frame(hwnd, state, buf);
                }
            }
            LRESULT(0)
        }

        WM_DESTROY => {
            // Cleanup: restore old bitmap, free resources, quit
            let state_ptr = GetWindowLongPtrW(hwnd, SLOT_STATE) as *mut Arc<Mutex<OverlayState>>;
            let buf_ptr = GetWindowLongPtrW(hwnd, SLOT_BUF) as *mut LayeredBufferState;

            if !buf_ptr.is_null() {
                let buf = Box::from_raw(buf_ptr);
                let _ = SelectObject(buf.hdc_mem, buf.old_hbm);
                let _ = DeleteObject(buf.hbm.into());
                let _ = DeleteDC(buf.hdc_mem);
                let _ = ReleaseDC(None, buf.hdc_screen);
            }

            if !state_ptr.is_null() {
                let _ = Box::from_raw(state_ptr);
            }

            PostQuitMessage(0);
            LRESULT(0)
        }

        _ => DefWindowProcA(hwnd, msg, wparam, lparam),
    }
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

    /// Helper: create renderer with Live data source for production behavior tests
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
        // Simulate the bar width calculation used in WM_PAINT
        let screen_width = 1920i32;

        // 0% progress -> 0px (or 1px in demo/mock mode)
        let progress = 0.0f32;
        let bar_width = ((screen_width as f32) * progress.clamp(0.0, 1.0)) as i32;
        assert_eq!(bar_width, 0);
        assert_eq!(bar_width.max(1), 1); // demo/mock minimum

        // 25% -> 480px
        let progress = 0.25f32;
        let bar_width = ((screen_width as f32) * progress.clamp(0.0, 1.0)) as i32;
        assert_eq!(bar_width, 480);

        // 50% -> 960px
        let progress = 0.5f32;
        let bar_width = ((screen_width as f32) * progress.clamp(0.0, 1.0)) as i32;
        assert_eq!(bar_width, 960);

        // 100% -> 1920px
        let progress = 1.0f32;
        let bar_width = ((screen_width as f32) * progress.clamp(0.0, 1.0)) as i32;
        assert_eq!(bar_width, 1920);
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
        use crate::colors::color_for_progress;
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
}
