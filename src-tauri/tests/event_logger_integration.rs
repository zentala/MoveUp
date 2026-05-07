//! Integration test: EventLogger writes to a real filesystem path.
//!
//! Verifies that the logger actually creates the directory, file, and content
//! on a real (temp) filesystem — not just the unit-test happy path.

use desk_lib::event_logger::EventLogger;

#[test]
fn writes_to_real_path() {
    let tmp = tempfile::tempdir().unwrap();
    let logger = EventLogger::new(tmp.path().join("logs"));
    logger.log("TEST_MESSAGE");
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let path = tmp.path().join("logs").join(&today).join("events.log");
    assert!(path.exists(), "events.log not created at {:?}", path);
    let body = std::fs::read_to_string(&path).unwrap();
    assert!(body.contains("TEST_MESSAGE"), "missing message in: {body}");
}

#[test]
fn creates_nested_base_dir() {
    let tmp = tempfile::tempdir().unwrap();
    // logs/ does not exist yet — new() must create it
    let logs_path = tmp.path().join("nested").join("logs");
    assert!(!logs_path.exists());
    let logger = EventLogger::new(logs_path.clone());
    logger.log("NESTED_DIR_TEST");
    assert!(logs_path.exists(), "base_dir should be created by EventLogger::new");
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let events = logs_path.join(&today).join("events.log");
    assert!(events.exists(), "events.log not found at {:?}", events);
}

#[test]
fn multiple_messages_all_present() {
    let tmp = tempfile::tempdir().unwrap();
    let logger = EventLogger::new(tmp.path().join("logs"));
    logger.log("STATE Sitting");
    logger.log("ALERT sit_limit");
    logger.log("RESET daily");
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let path = tmp.path().join("logs").join(&today).join("events.log");
    let body = std::fs::read_to_string(&path).unwrap();
    assert!(body.contains("STATE Sitting"), "missing STATE line");
    assert!(body.contains("ALERT sit_limit"), "missing ALERT line");
    assert!(body.contains("RESET daily"), "missing RESET line");
    assert_eq!(body.lines().count(), 3, "expected 3 lines, got:\n{body}");
}
