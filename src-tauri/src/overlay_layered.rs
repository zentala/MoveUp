//! overlay_layered.rs — LAYERED render backend (`WS_EX_LAYERED` + `UpdateLayeredWindow`).
//!
//! Uses a 32-bit ARGB DIBSection for per-pixel alpha compositing.
//! Experimental — use OPAQUE mode for production. `OVERLAY_MODE=layered` to enable.

use std::sync::{Arc, Mutex};
use log::info;

use crate::overlay_renderer::{DataSource, OverlayState};
use crate::overlay_variants::render_variant_pixels;

/// Offscreen 32-bit ARGB DIBSection for `UpdateLayeredWindow`.
#[cfg(target_os = "windows")]
pub(crate) struct LayeredBufferState {
    pub hdc_screen: windows::Win32::Graphics::Gdi::HDC, // screen DC cached at init
    pub hdc_mem: windows::Win32::Graphics::Gdi::HDC,
    pub hbm: windows::Win32::Graphics::Gdi::HBITMAP,
    pub old_hbm: windows::Win32::Graphics::Gdi::HGDIOBJ,
    pub bits_ptr: *mut u32, // BGRA pixel data (32-bit per pixel)
    pub width: i32,
    pub height: i32,
}

/// LAYERED render loop — transparent with `UpdateLayeredWindow` (EXPERIMENTAL).
///
/// ⚠️ Uses `WS_EX_LAYERED` + `UpdateLayeredWindow` for transparency.
/// Renders to offscreen 32-bit ARGB bitmap, composites with per-pixel alpha.
#[cfg(target_os = "windows")]
pub(crate) fn run_event_loop_layered(state: Arc<Mutex<OverlayState>>, bar_height: i32) {
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
        let hmonitor = MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY);
        let mut mi: MONITORINFO = std::mem::zeroed();
        mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;

        let (screen_width, screen_height, screen_x, screen_y) =
            if GetMonitorInfoW(hmonitor, &mut mi).as_bool() {
                (mi.rcMonitor.right - mi.rcMonitor.left, bar_height, mi.rcMonitor.left, mi.rcMonitor.top)
            } else {
                (1920, bar_height, 0, 0)
            };

        let hmodule = GetModuleHandleW(None).unwrap_or(HMODULE::default());
        let hinstance: HINSTANCE = hmodule.into();
        let arrow_cursor = LoadCursorW(None, IDC_ARROW).unwrap_or(HCURSOR::default());

        let wnd_class = WNDCLASSA {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc_layered),
            cbClsExtra: 0,
            cbWndExtra: 2 * std::mem::size_of::<isize>() as i32, // 2 slots: state + buf
            hInstance: hinstance,
            hIcon: HICON::default(),
            hCursor: arrow_cursor,
            hbrBackground: HBRUSH::default(), // null — no OS background
            lpszMenuName: PCSTR::null(),
            lpszClassName: PCSTR(CLASS_NAME.as_ptr()),
        };

        if RegisterClassA(&wnd_class) == 0 {
            info!("⚠️ [LAYERED] Failed to register window class");
            return;
        }

        let hwnd = match CreateWindowExA(
            WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
            PCSTR(CLASS_NAME.as_ptr()),
            PCSTR(WINDOW_NAME.as_ptr()),
            WS_POPUP, // No WS_VISIBLE yet — show after first draw
            screen_x, screen_y, screen_width, screen_height,
            None, None, Some(hinstance), None,
        ) {
            Ok(h) => h,
            Err(_) => { info!("⚠️ [LAYERED] Failed to create window"); return; }
        };

        let hdc_screen = GetDC(None);
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: screen_width, biHeight: -screen_height, // negative = top-down
                biPlanes: 1, biBitCount: 32, biCompression: BI_RGB.0,
                biSizeImage: 0, biXPelsPerMeter: 0, biYPelsPerMeter: 0,
                biClrUsed: 0, biClrImportant: 0,
            },
            bmiColors: [RGBQUAD::default()],
        };
        let mut bits_void_ptr = std::ptr::null_mut();
        let hbm = match CreateDIBSection(Some(hdc_screen), &bmi, DIB_RGB_COLORS, &mut bits_void_ptr, None, 0) {
            Ok(h) => h,
            Err(_) => { info!("⚠️ [LAYERED] Failed to create DIBSection"); ReleaseDC(None, hdc_screen); let _ = DestroyWindow(hwnd); return; }
        };
        let bits_ptr = bits_void_ptr as *mut u32;
        let hdc_mem = CreateCompatibleDC(Some(hdc_screen));
        if hdc_mem.is_invalid() {
            info!("⚠️ [LAYERED] Failed to create memory DC");
            ReleaseDC(None, hdc_screen); let _ = DeleteObject(hbm.into()); let _ = DestroyWindow(hwnd); return;
        }
        let old_hbm = SelectObject(hdc_mem, hbm.into());
        let buf_state = LayeredBufferState { hdc_screen, hdc_mem, hbm, old_hbm, bits_ptr, width: screen_width, height: screen_height };
        let state_ptr = Box::into_raw(Box::new(state.clone()));
        let buf_ptr = Box::into_raw(Box::new(buf_state));

        SetWindowLongPtrW(hwnd, SLOT_STATE, state_ptr as isize);
        SetWindowLongPtrW(hwnd, SLOT_BUF, buf_ptr as isize);

        draw_layered_frame(hwnd, &state, &*buf_ptr);
        info!("[LAYERED] Initial frame drawn, showing window");
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = UpdateWindow(hwnd);

        if SetTimer(Some(hwnd), TIMER_ID, TIMER_INTERVAL_MS, None) == 0 {
            info!("⚠️ [LAYERED] Failed to set timer");
            let _ = DestroyWindow(hwnd);
            return;
        }

        info!(
            "🎨 [LAYERED] WinAPI overlay window created: {}x{} @ ({},{}), UpdateLayeredWindow mode",
            screen_width, screen_height, screen_x, screen_y
        );

        let mut msg: MSG = std::mem::zeroed();
        loop {
            let msg_result = GetMessageW(&mut msg, None, 0, 0);
            if msg_result.0 == 0 { break; }
            if msg_result.0 < 0 { info!("⚠️ [LAYERED] GetMessageW error"); break; }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        let _ = DestroyWindow(hwnd);
        let _ = UnregisterClassA(PCSTR(CLASS_NAME.as_ptr()), Some(hinstance));
        let _ = Box::from_raw(state_ptr);
        let _ = Box::from_raw(buf_ptr);
    }
}

/// Draw a frame to the layered buffer and composite to screen with `UpdateLayeredWindow`.
#[cfg(target_os = "windows")]
pub(crate) unsafe fn draw_layered_frame(
    hwnd: windows::Win32::Foundation::HWND,
    state: &Arc<Mutex<OverlayState>>,
    buf: &LayeredBufferState,
) {
    use windows::Win32::Foundation::*;
    use windows::Win32::Graphics::Gdi::*;
    use windows::Win32::UI::WindowsAndMessaging::*;

    // Clear entire buffer: all pixels transparent (alpha=0)
    std::ptr::write_bytes(buf.bits_ptr, 0, (buf.width * buf.height) as usize);

    let (bar_progress, visible, color_rgb, _data_source, overlay_variant, frame_count) =
        if let Ok(s) = state.lock() {
            (s.progress, s.visible, s.color_rgb, s.data_source, s.overlay_variant, s.frame_count)
        } else {
            log::error!("[DRAW] Failed to acquire state lock!");
            return;
        };

    let bar_width = if visible {
        (((buf.width as f32) * bar_progress.clamp(0.0, 1.0)) as i32).max(1)
    } else {
        0
    };

    render_variant_pixels(buf.bits_ptr, buf.width, buf.height, bar_width, color_rgb, frame_count, overlay_variant);

    let src_point = POINT { x: 0, y: 0 };
    let dst_point = POINT { x: 0, y: 0 };
    let size = SIZE { cx: buf.width, cy: buf.height };
    let blend = BLENDFUNCTION { BlendOp: 0, BlendFlags: 0, SourceConstantAlpha: 255, AlphaFormat: 1 }; // AC_SRC_ALPHA

    match UpdateLayeredWindow(hwnd, Some(buf.hdc_screen), Some(&dst_point), Some(&size), Some(buf.hdc_mem), Some(&src_point), COLORREF(0), Some(&blend), ULW_ALPHA) {
        Ok(_) => {}
        Err(e) => { log::warn!("[LAYERED] UpdateLayeredWindow failed: {:?}", e); }
    }
}

/// Window procedure for the LAYERED overlay.
#[cfg(target_os = "windows")]
pub(crate) unsafe extern "system" fn wnd_proc_layered(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use crate::overlay_renderer::{demo_progress, mock_progress};
    use windows::Win32::Foundation::*;
    use windows::Win32::Graphics::Gdi::*;
    use windows::Win32::UI::WindowsAndMessaging::*;

    const SLOT_STATE: WINDOW_LONG_PTR_INDEX = WINDOW_LONG_PTR_INDEX(0);
    const SLOT_BUF: WINDOW_LONG_PTR_INDEX = WINDOW_LONG_PTR_INDEX(8);

    match msg {
        WM_TIMER => {
            // ⚠️ Unlike OPAQUE (InvalidateRect), we must redraw every frame —
            // UpdateLayeredWindow doesn't use the paint pipeline.
            let state_ptr = GetWindowLongPtrW(hwnd, SLOT_STATE) as *mut Arc<Mutex<OverlayState>>;
            let buf_ptr = GetWindowLongPtrW(hwnd, SLOT_BUF) as *mut LayeredBufferState;

            if !state_ptr.is_null() && !buf_ptr.is_null() {
                let state = &*state_ptr;
                if let Ok(mut s) = state.lock() {
                    s.frame_count = s.frame_count.wrapping_add(1);
                    match s.data_source {
                        DataSource::Demo => { let (p, c) = demo_progress(s.frame_count); s.progress = p; s.color_rgb = c; s.visible = true; }
                        DataSource::Mock => { let (p, c, v) = mock_progress(s.frame_count); s.progress = p; s.color_rgb = c; s.visible = v; }
                        DataSource::Live => {}
                    }
                    drop(s);
                    draw_layered_frame(hwnd, state, &*buf_ptr);
                }
            }
            LRESULT(0)
        }

        WM_DESTROY => {
            let state_ptr = GetWindowLongPtrW(hwnd, SLOT_STATE) as *mut Arc<Mutex<OverlayState>>;
            let buf_ptr = GetWindowLongPtrW(hwnd, SLOT_BUF) as *mut LayeredBufferState;

            if !buf_ptr.is_null() {
                let buf = Box::from_raw(buf_ptr);
                let _ = SelectObject(buf.hdc_mem, buf.old_hbm);
                let _ = DeleteObject(buf.hbm.into());
                let _ = DeleteDC(buf.hdc_mem);
                let _ = ReleaseDC(None, buf.hdc_screen);
            }
            if !state_ptr.is_null() { let _ = Box::from_raw(state_ptr); }

            PostQuitMessage(0);
            LRESULT(0)
        }

        _ => DefWindowProcA(hwnd, msg, wparam, lparam),
    }
}
