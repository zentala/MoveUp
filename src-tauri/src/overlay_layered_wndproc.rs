//! overlay_layered_wndproc.rs — Window procedure for the LAYERED overlay.

use std::sync::{Arc, Mutex};

use crate::overlay_layered::{draw_layered_frame, LayeredBufferState};
use crate::overlay_renderer::{demo_progress, mock_progress, DataSource, OverlayState};

/// Window procedure for the LAYERED overlay.
#[cfg(target_os = "windows")]
pub(crate) unsafe extern "system" fn wnd_proc_layered(
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
            // Unlike OPAQUE (InvalidateRect), we must redraw every frame —
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
                        DataSource::Live => {
                            if let Some(until) = s.lap_flash_until {
                                if std::time::Instant::now() > until {
                                    s.lap_flash_until = None;
                                    s.overlay_variant = 0;
                                    s.needs_redraw = true;
                                }
                            }
                        }
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
