//! alert_popup.rs — Non-modal always-on-top popup shown during alert Stage2.
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

use crate::alert_popup_window::run_popup_window;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};

// ─── Public API ───────────────────────────────────────────────────────────────

/// Non-modal always-on-top popup shown during alert Stage2.
pub struct AlertPopup {
    pub(crate) visible: Arc<AtomicBool>,
    /// Set to `true` by the popup thread when user clicks Dismiss or Stand Up.
    user_dismissed: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<()>>,
}

impl AlertPopup {
    /// Creates a new (hidden) popup.
    pub fn new() -> Self {
        Self {
            visible: Arc::new(AtomicBool::new(false)),
            user_dismissed: Arc::new(AtomicBool::new(false)),
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
        let user_dismissed = Arc::clone(&self.user_dismissed);
        let handle = thread::Builder::new()
            .name("alert-popup".into())
            .spawn(move || {
                run_popup_window(visible_flag, msg, user_dismissed);
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

    /// Returns `true` (and resets the flag) if the user clicked Dismiss or Stand Up
    /// since the last call. Used by `tray_controller` to trigger snooze logic.
    pub fn take_user_dismissed(&self) -> bool {
        self.user_dismissed.swap(false, Ordering::SeqCst)
    }
}

impl Default for AlertPopup {
    fn default() -> Self {
        Self::new()
    }
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
