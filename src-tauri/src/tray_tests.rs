//! tray_tests.rs — Unit tests for tray icon selection logic.

use crate::session::DeskState;
use crate::tray::icon_for_state_and_progress;
use crate::tray_icon::{generate_tray_icon, generate_tray_icon_no_dot, IconUpdate, TrayAppearance};

#[test]
fn tray_contrast_changes_only_white_silhouette_and_round_trips_every_state() {
    let mut frames = vec![generate_tray_icon_no_dot().unwrap()];
    for state in [DeskState::Sitting, DeskState::Standing, DeskState::Walking, DeskState::Away] {
        for ratio in [0.0, 0.60, 0.85, 1.0] {
            frames.push(generate_tray_icon(state.clone(), ratio).unwrap());
        }
    }
    for source in frames {
        let original = source.rgba().to_vec();
        let mut appearance = TrayAppearance::new(source, true);
        let light_tray = appearance.render();
        assert_eq!((light_tray.width(), light_tray.height()), (32, 32));
        for (before, after) in original.chunks_exact(4).zip(light_tray.rgba().chunks_exact(4)) {
            if before[..3] == [255, 255, 255] && before[3] != 0 {
                assert_eq!(after, &[22, 22, 22, before[3]]);
            } else {
                assert_eq!(after, before, "status colors and transparent pixels must stay unchanged");
            }
        }
        assert_eq!(appearance.update(IconUpdate::Theme(Some(false))).unwrap().rgba(), original);
        assert!(appearance.update(IconUpdate::Theme(None)).is_none());
        assert!(appearance.update(IconUpdate::Frame(None)).is_none());
        assert_eq!(appearance.render().rgba(), original);
        assert_eq!(appearance.update(IconUpdate::Theme(Some(true))).unwrap().rgba(), light_tray.rgba());
        assert!(appearance.update(IconUpdate::Theme(Some(true))).is_none());
    }
}

#[test]
fn existing_png_source_pixels_are_unchanged_in_both_system_themes() {
    for bytes in [
        include_bytes!("../icons/tray/tray-ok.png").as_slice(),
        include_bytes!("../icons/tray/tray-warn.png").as_slice(),
        include_bytes!("../icons/tray/tray-alert.png").as_slice(),
        include_bytes!("../icons/tray/tray-idle.png").as_slice(),
    ] {
        let source = tauri::image::Image::from_bytes(bytes).unwrap();
        let original = source.rgba().to_vec();
        let mut appearance = TrayAppearance::new(source, true);
        assert_eq!(appearance.render().rgba(), original);
        assert_eq!(appearance.update(IconUpdate::Theme(Some(false))).unwrap().rgba(), original);
    }
}

#[test]
fn queued_theme_refresh_keeps_the_latest_blink_frame() {
    use std::{rc::Rc, sync::{Arc, Mutex, mpsc}};
    let frame = Arc::new(Mutex::new(TrayAppearance::new(
        generate_tray_icon(DeskState::Sitting, 0.9).unwrap(), false)));
    let (tx, rx) = mpsc::channel::<Box<dyn FnOnce() + Send>>();
    let output = Arc::new(Mutex::new(Vec::new()));
    let receiver_frame = frame.clone();
    let receiver_output = output.clone();
    crate::tray::dispatch_tray_update(|task| { tx.send(task).unwrap(); Ok(()) },
        || Some(Rc::new(())), move |_| {
            let image = receiver_frame.lock().unwrap().update(IconUpdate::Theme(Some(true))).unwrap();
            *receiver_output.lock().unwrap() = image.rgba().to_vec();
        }).unwrap();
    // A blink-off update wins before the queued theme notification executes.
    frame.lock().unwrap().update(IconUpdate::Frame(generate_tray_icon_no_dot()));
    assert!(output.lock().unwrap().is_empty());
    rx.recv().unwrap()();
    let expected = TrayAppearance::new(generate_tray_icon_no_dot().unwrap(), true).render();
    assert_eq!(*output.lock().unwrap(), expected.rgba());
    // The next on phase retains its current red dot under the new theme.
    let on = frame.lock().unwrap().update(IconUpdate::Frame(generate_tray_icon(DeskState::Sitting, 0.9))).unwrap();
    let expected = TrayAppearance::new(generate_tray_icon(DeskState::Sitting, 0.9).unwrap(), true).render();
    assert_eq!(on.rgba(), expected.rgba());
}

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
