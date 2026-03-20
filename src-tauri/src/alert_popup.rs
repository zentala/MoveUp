//! alert_popup.rs — WinAPI popup window for sit-too-long alerts.
//!
//! Spawns a small always-on-top window near the system tray (right-bottom corner)
//! with [Dismiss] and [Stand up] buttons. The window lives on its own thread;
//! an [`AtomicBool`] signals the thread to exit cleanly.
//!
//! ## Layout
//! ```text
//! ┌────────────────────────────────┐
//! │  You've been sitting           │
//! │  for 45 minutes.               │
//! │  Take a 5-min break!           │
//! │                                │
//! │       [Dismiss]  [Stand up]    │
//! └────────────────────────────────┘
//!        ↑ right-bottom, near tray
//! ```
//!
//! ## Thread lifecycle
//! - [`show`](AlertPopup::show) spawns the WinAPI thread and sets `visible = true`.
//! - [`dismiss`](AlertPopup::dismiss) sets `visible = false`; the thread exits its
//!   message loop and the handle is joined.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};

// ─── Public API ───────────────────────────────────────────────────────────────

/// Non-modal always-on-top popup shown during alert Stage2.
pub struct AlertPopup {
    visible: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<()>>,
}

impl AlertPopup {
    /// Creates a new (hidden) popup.
    pub fn new() -> Self {
        Self {
            visible: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
        }
    }

    /// Spawns the WinAPI popup thread and makes the window visible.
    ///
    /// If a popup is already visible this is a no-op.
    pub fn show(&mut self, msg: String) {
        if self.visible.load(Ordering::SeqCst) {
            return; // Already showing
        }
        self.visible.store(true, Ordering::SeqCst);
        let visible_flag = Arc::clone(&self.visible);
        let handle = thread::Builder::new()
            .name("alert-popup".into())
            .spawn(move || {
                run_popup_window(visible_flag, msg);
            })
            .expect("Failed to spawn alert popup thread");
        self.thread_handle = Some(handle);
    }

    /// Signals the popup thread to exit and joins it.
    ///
    /// Safe to call when no popup is visible.
    pub fn dismiss(&mut self) {
        self.visible.store(false, Ordering::SeqCst);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }

    /// Returns `true` if the popup is currently visible.
    #[allow(dead_code)]
    pub fn is_visible(&self) -> bool {
        self.visible.load(Ordering::SeqCst)
    }
}

impl Default for AlertPopup {
    fn default() -> Self {
        Self::new()
    }
}

// ─── WinAPI implementation ────────────────────────────────────────────────────

/// ID for the Dismiss button.
const BTN_DISMISS: usize = 101;
/// ID for the Stand Up button.
const BTN_STAND_UP: usize = 102;

/// Popup window width in pixels.
const POPUP_WIDTH: i32 = 400;
/// Popup window height in pixels.
const POPUP_HEIGHT: i32 = 200;
/// Margin from screen right/bottom edges.
const MARGIN: i32 = 16;

/// Runs the WinAPI message loop for the popup on the calling thread.
///
/// Exits when `visible_flag` is set to `false` (polled via `PeekMessage` loop + sleep).
#[cfg(target_os = "windows")]
fn run_popup_window(visible_flag: Arc<AtomicBool>, msg: String) {
    use windows::Win32::Foundation::*;
    use windows::Win32::Graphics::Gdi::*;
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::*;
    use windows::core::PCWSTR;

    const CLASS_NAME: &str = "zntlAlertPopup\0";
    const WINDOW_TITLE: &str = "zntl Desk Alert\0";

    let msg_wide: Vec<u16> = msg.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        let hmodule = GetModuleHandleW(None).unwrap_or_default();
        let hinstance: HINSTANCE = hmodule.into();

        let class_wide: Vec<u16> = CLASS_NAME.encode_utf16().collect();
        let title_wide: Vec<u16> = WINDOW_TITLE.encode_utf16().collect();

        let arrow_cursor = LoadCursorW(None, IDC_ARROW).unwrap_or_default();

        // Register window class (ignore ALREADY_EXISTS error on re-use)
        let wnd_class = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(popup_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: std::mem::size_of::<isize>() as i32,
            hInstance: hinstance,
            hIcon: HICON::default(),
            hCursor: arrow_cursor,
            hbrBackground: HBRUSH((COLOR_WINDOW.0 + 1) as isize as *mut std::ffi::c_void),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: PCWSTR(class_wide.as_ptr()),
        };
        let _ = RegisterClassW(&wnd_class);

        // Determine right-bottom position (work area excludes taskbar)
        let hmonitor = MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY);
        let mut monitor_info: MONITORINFO = std::mem::zeroed();
        monitor_info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        let (scr_right, scr_bottom) = if GetMonitorInfoW(hmonitor, &mut monitor_info).as_bool() {
            (monitor_info.rcWork.right, monitor_info.rcWork.bottom)
        } else {
            (1920, 1080)
        };

        let x = scr_right - POPUP_WIDTH - MARGIN;
        let y = scr_bottom - POPUP_HEIGHT - MARGIN;

        let hwnd = match CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            PCWSTR(class_wide.as_ptr()),
            PCWSTR(title_wide.as_ptr()),
            WS_POPUP | WS_BORDER | WS_VISIBLE,
            x,
            y,
            POPUP_WIDTH,
            POPUP_HEIGHT,
            None,
            None,
            Some(hinstance),
            Some(msg_wide.as_ptr() as *const std::ffi::c_void),
        ) {
            Ok(h) => h,
            Err(e) => {
                log::error!("[AlertPopup] CreateWindowExW failed: {:?}", e);
                visible_flag.store(false, Ordering::SeqCst);
                return;
            }
        };

        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = UpdateWindow(hwnd);

        let mut msg_loop: MSG = std::mem::zeroed();
        loop {
            // Poll visible_flag — dismiss() sets it to false
            if !visible_flag.load(Ordering::SeqCst) {
                let _ = DestroyWindow(hwnd);
                break;
            }

            while PeekMessageW(&mut msg_loop, None, 0, 0, PM_REMOVE).as_bool() {
                if msg_loop.message == WM_QUIT {
                    let _ = DestroyWindow(hwnd);
                    return;
                }
                let _ = TranslateMessage(&msg_loop);
                DispatchMessageW(&msg_loop);
            }

            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    }
}

/// WndProc for the alert popup window.
#[cfg(target_os = "windows")]
unsafe extern "system" fn popup_wnd_proc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::Foundation::*;
    use windows::Win32::Graphics::Gdi::*;
    use windows::Win32::UI::WindowsAndMessaging::*;
    use windows::core::PCWSTR;

    match msg {
        WM_CREATE => {
            // Retrieve message from lpCreateParams
            let create_struct = lparam.0 as *const CREATESTRUCTW;
            let msg_ptr = (*create_struct).lpCreateParams as *const u16;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, msg_ptr as isize);

            let hinstance = (*create_struct).hInstance;

            let btn_class: Vec<u16> = "BUTTON\0".encode_utf16().collect();

            // Create Dismiss button
            let dismiss_label: Vec<u16> = "Dismiss\0".encode_utf16().collect();
            let _ = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                PCWSTR(btn_class.as_ptr()),
                PCWSTR(dismiss_label.as_ptr()),
                WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_PUSHBUTTON as u32),
                220,
                150,
                80,
                30,
                Some(hwnd),
                Some(HMENU(BTN_DISMISS as *mut std::ffi::c_void)),
                Some(hinstance),
                None,
            );

            // Create Stand Up button
            let stand_label: Vec<u16> = "Stand up\0".encode_utf16().collect();
            let _ = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                PCWSTR(btn_class.as_ptr()),
                PCWSTR(stand_label.as_ptr()),
                WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_PUSHBUTTON as u32),
                310,
                150,
                80,
                30,
                Some(hwnd),
                Some(HMENU(BTN_STAND_UP as *mut std::ffi::c_void)),
                Some(hinstance),
                None,
            );

            LRESULT(0)
        }

        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);

            // Draw the message text
            let msg_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const u16;
            if !msg_ptr.is_null() {
                // Measure null-terminated wide string length
                let mut len = 0usize;
                while *msg_ptr.add(len) != 0 {
                    len += 1;
                }
                let msg_slice = std::slice::from_raw_parts_mut(msg_ptr as *mut u16, len);
                let mut rect = RECT { left: 16, top: 20, right: POPUP_WIDTH - 16, bottom: 140 };
                SetBkMode(hdc, TRANSPARENT);
                DrawTextW(hdc, msg_slice, &mut rect, DT_LEFT | DT_WORDBREAK);
            }

            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }

        WM_COMMAND => {
            let btn_id = (wparam.0 & 0xFFFF) as usize;
            if btn_id == BTN_DISMISS || btn_id == BTN_STAND_UP {
                let _ = PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
            }
            LRESULT(0)
        }

        WM_CLOSE => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }

        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// No-op stub for non-Windows platforms.
#[cfg(not(target_os = "windows"))]
fn run_popup_window(_visible_flag: Arc<AtomicBool>, _msg: String) {
    log::warn!("[AlertPopup] Popup not supported on this platform");
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Test 13: show() → is_visible() == true
    #[test]
    fn show_sets_visible() {
        let mut popup = AlertPopup::new();
        assert!(!popup.is_visible());
        // Set the AtomicBool directly to test the visible flag logic without
        // spawning an actual WinAPI thread (not feasible in unit test environment).
        popup.visible.store(true, Ordering::SeqCst);
        assert!(popup.is_visible());
    }

    // Test 14: dismiss() → is_visible() == false
    #[test]
    fn dismiss_clears_visible() {
        let mut popup = AlertPopup::new();
        popup.visible.store(true, Ordering::SeqCst);
        assert!(popup.is_visible());
        popup.dismiss();
        assert!(!popup.is_visible());
    }

    // Test 15: Double-dismiss is safe (no panic)
    #[test]
    fn double_dismiss_no_panic() {
        let mut popup = AlertPopup::new();
        popup.dismiss(); // no-op, not visible
        popup.dismiss(); // second no-op, must not panic
        assert!(!popup.is_visible());
    }
}
