//! tray_tests.rs — Unit tests for tray icon selection logic.

use crate::session::DeskState;
use crate::tray::icon_for_state_and_progress;

#[test]
fn queued_tray_lookup_use_and_drop_all_execute_on_receiver_thread() {
    use std::{rc::Rc, sync::{Arc, Mutex, mpsc}, thread};
    type Events = Arc<Mutex<Vec<(&'static str, thread::ThreadId)>>>;
    struct LocalHandle { _not_send: Rc<()>, events: Events }
    impl Drop for LocalHandle {
        fn drop(&mut self) { self.events.lock().unwrap().push(("drop", thread::current().id())); }
    }
    let events: Events = Arc::new(Mutex::new(vec![]));
    let (tx, rx) = mpsc::channel::<Box<dyn FnOnce() + Send>>();
    let lookup_events = events.clone();
    let sender = thread::spawn(move || {
        let sender_id = thread::current().id();
        crate::tray::dispatch_tray_update(|task| { tx.send(task).unwrap(); Ok(()) },
            move || {
                lookup_events.lock().unwrap().push(("lookup", thread::current().id()));
                Some(LocalHandle { _not_send: Rc::new(()), events: lookup_events })
            }, |handle| handle.events.lock().unwrap().push(("use", thread::current().id())))
            .unwrap();
        sender_id
    });
    let sender_id = sender.join().unwrap();
    assert!(events.lock().unwrap().is_empty(), "lookup must not run while scheduling");
    let receiver_id = thread::current().id();
    assert_ne!(sender_id, receiver_id);
    rx.recv().unwrap()();
    assert_eq!(*events.lock().unwrap(), vec![("lookup", receiver_id), ("use", receiver_id), ("drop", receiver_id)]);
}

#[test]
fn queued_missing_tray_is_a_noop_and_schedule_failure_does_not_lookup() {
    use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
    let looked_up = Arc::new(AtomicBool::new(false));
    let lookup_flag = looked_up.clone();
    let result = crate::tray::dispatch_tray_update::<()>(|_| Err(tauri::Error::FailedToReceiveMessage),
        move || { lookup_flag.store(true, Ordering::SeqCst); None }, |_| panic!("no tray"));
    assert!(result.is_err());
    assert!(!looked_up.load(Ordering::SeqCst));
    crate::tray::dispatch_tray_update::<()>(|task| { task(); Ok(()) }, || None, |_| panic!("no tray")).unwrap();
}

#[test]
fn sitting_below_60_percent_returns_ok() {
    assert_eq!(icon_for_state_and_progress(DeskState::Sitting, 0.3), "tray-ok");
}

#[test]
fn sitting_60_to_85_percent_returns_warn() {
    assert_eq!(icon_for_state_and_progress(DeskState::Sitting, 0.7), "tray-warn");
}

#[test]
fn sitting_above_85_percent_returns_alert() {
    assert_eq!(icon_for_state_and_progress(DeskState::Sitting, 0.9), "tray-alert");
}

#[test]
fn sitting_exactly_60_percent_returns_warn() {
    assert_eq!(icon_for_state_and_progress(DeskState::Sitting, 0.60), "tray-warn");
}

#[test]
fn sitting_exactly_85_percent_returns_alert() {
    assert_eq!(icon_for_state_and_progress(DeskState::Sitting, 0.85), "tray-alert");
}

#[test]
fn standing_returns_standing() {
    assert_eq!(icon_for_state_and_progress(DeskState::Standing, 0.5), "tray-standing");
}

#[test]
fn walking_returns_idle() {
    assert_eq!(icon_for_state_and_progress(DeskState::Walking, 0.5), "tray-idle");
}

#[test]
fn away_returns_idle() {
    assert_eq!(icon_for_state_and_progress(DeskState::Away, 0.5), "tray-idle");
}

#[test]
fn standing_returns_standing_regardless_of_progress() {
    assert_eq!(icon_for_state_and_progress(DeskState::Standing, 0.0), "tray-standing");
    assert_eq!(icon_for_state_and_progress(DeskState::Standing, 1.0), "tray-standing");
}
