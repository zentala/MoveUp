//! overlay_opaque.rs — OPAQUE render backend for the overlay progress bar.
//!
//! Uses standard GDI `BeginPaint`/`FillRect` with a black background.
//! Window style: `WS_POPUP | WS_VISIBLE` — no `WS_EX_LAYERED`.
//!
//! Entry point: `run_event_loop_opaque(state, bar_height)` called from
//! `overlay_renderer::run_event_loop()` when `OVERLAY_MODE` is not `"layered"`.

use std::sync::{Arc, Mutex};
use log::info;

use crate::overlay_renderer::{DataSource, OverlayState};
use crate::overlay_variants::render_variant_gdi;

/// OPAQUE render loop — black background, standard GDI rendering (STABLE).
///
/// ⚠️ No transparency, but reliable rendering on all Windows versions.
#[cfg(target_os = "windows")]
pub(crate) fn run_event_loop_opaque(state: Arc<Mutex<OverlayState>>, bar_height: i32) {
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

        let (screen_width, screen_height, screen_x, screen_y) =
            if GetMonitorInfoW(hmonitor, &mut monitor_info).as_bool() {
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

        let arrow_cursor = LoadCursorW(None, IDC_ARROW).unwrap_or(HCURSOR::default());

        let wnd_class = WNDCLASSA {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: std::mem::size_of::<isize>() as i32,
            hInstance: hinstance,
            hIcon: HICON::default(),
            hCursor: arrow_cursor,
            hbrBackground: HBRUSH(GetStockObject(BLACK_BRUSH).0),
            lpszMenuName: PCSTR::null(),
            lpszClassName: PCSTR(CLASS_NAME.as_ptr()),
        };

        let class_atom = RegisterClassA(&wnd_class);
        if class_atom == 0 {
            info!("⚠️ Failed to register window class");
            return;
        }

        // 3. Create window at monitor position with screen_width x bar_height px
        let hwnd = match CreateWindowExA(
            WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
            PCSTR(CLASS_NAME.as_ptr()),
            PCSTR(WINDOW_NAME.as_ptr()),
            WS_POPUP | WS_VISIBLE,
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

/// Window procedure — handles window messages for the OPAQUE overlay.
///
/// Accessed state pointer stored in cbWndExtra via `GetWindowLongPtrW(hwnd, 0)`.
#[cfg(target_os = "windows")]
pub(crate) unsafe extern "system" fn wnd_proc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use crate::overlay_renderer::{demo_progress, mock_progress};
    use windows::Win32::Foundation::*;
    use windows::Win32::Graphics::Gdi::*;
    use windows::Win32::UI::WindowsAndMessaging::*;

    match msg {
        WM_TIMER => {
            // WM_TIMER: Increment animation frame and invalidate
            let state_ptr =
                GetWindowLongPtrW(hwnd, WINDOW_LONG_PTR_INDEX(0)) as *mut Arc<Mutex<OverlayState>>;
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
            let state_ptr =
                GetWindowLongPtrW(hwnd, WINDOW_LONG_PTR_INDEX(0)) as *mut Arc<Mutex<OverlayState>>;
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
                        let bar_width = bar_width.max(1); // Always show at least 1px when visible

                        render_variant_gdi(
                            hdc,
                            bar_width,
                            window_height,
                            s.color_rgb,
                            s.frame_count,
                            s.overlay_variant,
                        );
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
