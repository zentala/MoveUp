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
    pub frame_count: u32,        // For test animation (cycles through colors)
    pub demo_mode: bool,         // If true, cycle progress for testing
}

impl Default for OverlayState {
    fn default() -> Self {
        let demo_mode = std::env::var("OVERLAY_DEMO_MODE")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false);
        Self {
            progress: 0.0,
            color_rgb: (76, 175, 80), // green
            visible: false,
            needs_redraw: false,
            frame_count: 0,
            demo_mode,
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
    let demo_mode = std::env::var("OVERLAY_DEMO_MODE")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);

    match mode.as_str() {
        "layered" => {
            if demo_mode {
                info!("🎨 [EXPERIMENTAL + DEMO] Starting overlay in LAYERED mode with demo cycling (0%, 25%, 50%, 75%, 100% every 5s)");
            } else {
                info!("🎨 [EXPERIMENTAL] Starting overlay in LAYERED mode (UpdateLayeredWindow)");
            }
            run_event_loop_layered(state);
        }
        _ => {
            if demo_mode {
                info!("🎨 [STABLE + DEMO] Starting overlay in OPAQUE mode with demo cycling (every 5s)");
            } else {
                info!("🎨 [STABLE] Starting overlay in OPAQUE mode (black background)");
            }
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
            // WM_TIMER: Increment animation frame and invalidate
            let state_ptr = GetWindowLongPtrW(hwnd, WINDOW_LONG_PTR_INDEX(0)) as *mut Arc<Mutex<OverlayState>>;
            if !state_ptr.is_null() {
                let state = &*state_ptr;
                if let Ok(mut s) = state.lock() {
                    s.frame_count = s.frame_count.wrapping_add(1);

                    // Demo mode: cycle progress through 0%, 25%, 50%, 75%, 100% every 5 seconds (300 frames @ 60fps)
                    if s.demo_mode {
                        let stage = ((s.frame_count / 300) % 5) as u32; // 300 frames @ 60fps = 5 seconds per stage
                        s.progress = stage as f32 / 4.0; // 0/4, 1/4, 2/4, 3/4, 4/4
                        s.visible = true; // Always show in demo mode
                    }

                    // For test: always redraw to cycle through colors
                    let _ = InvalidateRect(Some(hwnd), None, false.into());
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

                    // Fill background with black
                    let black_brush = CreateSolidBrush(COLORREF(0));
                    if !black_brush.is_invalid() {
                        let _ = FillRect(hdc, &rect, black_brush);
                        let _ = DeleteObject(black_brush.into());
                    }

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

                        // Draw progress bar ON TOP of gold
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
fn run_event_loop_layered(state: Arc<Mutex<OverlayState>>) {
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
                4i32,
                monitor_info.rcMonitor.left,
                monitor_info.rcMonitor.top,
            )
        } else {
            (1920, 4, 0, 0)
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

    // 2. Lock state, snapshot progress/visible/frame_count/demo_mode
    let (mut bar_progress, visible, frame_count, demo_mode) = if let Ok(s) = state.lock() {
        let v = s.visible;
        if v {
            log::warn!("🟢 [DRAW] visible=TRUE, will show bar");
        } else {
            log::warn!("🔴 [DRAW] visible=FALSE, bar will be 0px");
        }
        (s.progress, v, s.frame_count, s.demo_mode)
    } else {
        log::error!("[DRAW] Failed to acquire state lock!");
        return;
    };

    // Demo mode: override progress based on frame count (0%, 25%, 50%, 75%, 100% every 5 seconds)
    if demo_mode {
        let stage = ((frame_count / 300) % 5) as u32;
        bar_progress = stage as f32 / 4.0;
        log::warn!("🔵 [DEMO-OVERRIDE] frame_count={}, stage={}, bar_progress={:.2}% (before: calc from state)", frame_count, stage, bar_progress * 100.0);
    }

    // 3. Calculate bar width - always show at least 1px when visible
    let bar_width = if visible {
        let w = ((buf.width as f32) * bar_progress.max(0.0).min(1.0)) as i32;
        w.max(1)  // Minimum 1 pixel wide
    } else {
        0
    };

    log::warn!("📊 [DRAW] frame={} | visible={} | progress={:.2}% | bar_width={}/{} | demo={}",
        frame_count, visible, bar_progress * 100.0, bar_width, buf.width, demo_mode);

    // 4. Fill bar pixels with BGRA (background stays transparent)
    // Semi-transparent white bar (always visible when visible=true)
    // Alpha = 128 (50% opacity), RGB = white (255, 255, 255)
    let alpha = 128u32;
    let pixel = ((alpha as u32) << 24) | 0x00_FF_FF_FF;  // ARGB: 50% white

    for y in 0..buf.height {
        for x in 0..bar_width {
            let idx = (y * buf.width + x) as usize;
            *buf.bits_ptr.add(idx) = pixel;
        }
    }

    if bar_width > 0 {
        log::debug!("[LAYERED] Filled {} pixels", bar_width * buf.height);
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

    log::debug!("[LAYERED] Calling UpdateLayeredWindow: hdc_screen={:?}, hdc_mem={:?}, size={}x{}", buf.hdc_screen.0, buf.hdc_mem.0, size.cx, size.cy);
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
        Ok(_) => {
            log::debug!("[LAYERED] UpdateLayeredWindow SUCCESS");
        }
        Err(e) => {
            log::warn!("[LAYERED] UpdateLayeredWindow FAILED: {:?}", e);
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
                    let fc = s.frame_count;
                    let dm = s.demo_mode;
                    if dm {
                        s.visible = true; // Always show in demo mode
                    }
                    // Log every 60 frames (approx 1 second at 60fps)
                    if fc % 60 == 0 {
                        log::warn!("⏱️ [TIMER] frame_count={}, demo_mode={}, visible={}", fc, dm, s.visible);
                    }
                    drop(s); // release lock before draw
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
