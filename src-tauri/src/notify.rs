//! notify.rs — Windows notification helper with correct AUMID.
//!
//! The `tauri-plugin-notification` skips setting AppUserModelId in dev mode,
//! causing Windows to attribute notifications to "PowerShell" or the parent
//! process. This module wraps `notify-rust` directly with AUMID always set.

#[cfg(test)]
use std::{
    cell::RefCell,
    marker::PhantomData,
    rc::Rc,
    sync::{Arc, Mutex},
};

/// App identifier used as Windows AppUserModelId.
#[cfg(all(not(test), target_os = "windows"))]
const APP_ID: &str = "io.zntl.desk";

/// Sends a Windows toast notification with the correct app identity.
///
/// Always sets `app_id` so notifications show as "Smart Desk" regardless
/// of whether the app runs from an installer or `pnpm tauri:dev`.
#[cfg(not(test))]
pub fn show(title: &str, body: &str) {
    let title = title.to_string();
    let body = body.to_string();
    tauri::async_runtime::spawn(async move {
        let mut notification = notify_rust::Notification::new();
        notification
            .appname("Smart Desk")
            .summary(&title)
            .body(&body);
        #[cfg(target_os = "windows")]
        notification.app_id(APP_ID);
        if let Err(e) = notification.show() {
            log::warn!("Toast notification failed: {e}");
        }
    });
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NotificationIntent {
    pub(crate) title: String,
    pub(crate) body: String,
}

#[cfg(test)]
thread_local! {
    static ACTIVE_CAPTURE: RefCell<Option<Arc<Mutex<Vec<NotificationIntent>>>>> = RefCell::new(None);
}

/// A per-thread test transport scope; test builds never dispatch to Windows.
#[cfg(test)]
pub(crate) struct NotificationCapture {
    intents: Arc<Mutex<Vec<NotificationIntent>>>,
    previous: Option<Arc<Mutex<Vec<NotificationIntent>>>>,
    _not_send: PhantomData<Rc<()>>,
}

#[cfg(test)]
pub(crate) fn capture() -> NotificationCapture {
    let intents = Arc::new(Mutex::new(Vec::new()));
    let previous = ACTIVE_CAPTURE.with(|active| active.replace(Some(intents.clone())));
    NotificationCapture { intents, previous, _not_send: PhantomData }
}

#[cfg(test)]
impl NotificationCapture {
    pub(crate) fn intents(&self) -> Vec<NotificationIntent> {
        self.intents.lock().unwrap_or_else(|error| error.into_inner()).clone()
    }
}

#[cfg(test)]
impl Drop for NotificationCapture {
    fn drop(&mut self) {
        ACTIVE_CAPTURE.with(|active| { active.replace(self.previous.take()); });
    }
}

#[cfg(test)]
pub fn show(title: &str, body: &str) {
    ACTIVE_CAPTURE.with(|active| {
        let capture = active.borrow().as_ref().cloned();
        if let Some(capture) = capture {
            capture.lock().unwrap_or_else(|error| error.into_inner()).push(NotificationIntent {
                title: title.to_string(),
                body: body.to_string(),
            });
        }
    });
}

#[cfg(test)]
#[test]
fn notification_capture_is_scoped_to_its_test_thread() {
    let capture = capture();
    std::thread::spawn(|| show("other test thread", "not captured here"))
        .join().expect("test notification thread completes");
    show("current test thread", "captured intent");

    assert_eq!(capture.intents(), vec![NotificationIntent {
        title: "current test thread".into(),
        body: "captured intent".into(),
    }]);
}
