use crate::{app_lifecycle::AppLifecycle, app_lifecycle_store::{PreviousRun, RunStore, RunRecord},
    event_logger::EventLogger, startup_diagnostics::{Diagnostic, StartupDiagnostics}};
use std::{fs, sync::{Arc, atomic::Ordering}};
use tempfile::tempdir;

#[test]
fn normal_exit_and_quit_are_correlated_in_existing_event_log() {
    let dir = tempdir().unwrap();
    let lifecycle = AppLifecycle::new("0.6.0", dir.path().join("early.log"));
    let previous = lifecycle.claim_primary(dir.path()).unwrap();
    assert_eq!(previous, PreviousRun::None);
    lifecycle.attach_logger(Arc::new(EventLogger::new(dir.path().join("logs"))), previous);
    lifecycle.tray_initialized.store(true, Ordering::Release);
    lifecycle.persistence_initialized.store(true, Ordering::Release);
    lifecycle.setup_finished();
    assert!(lifecycle.essential_ready());
    lifecycle.request_quit();
    assert!(!lifecycle.essential_ready());
    lifecycle.finish_exit("store=skipped_store_missing database=saved");
    let record: RunRecord = serde_json::from_slice(&fs::read(dir.path().join("lifecycle.json")).unwrap()).unwrap();
    assert_eq!(record.identity.run_id, lifecycle.identity.run_id);
    assert_eq!(record.outcome, "clean_exit");
    assert!(record.quit_requested);
    assert!(record.flush.contains("skipped_store_missing"));
    let day = chrono::Local::now().format("%Y-%m-%d").to_string();
    let log = fs::read_to_string(dir.path().join("logs").join(day).join("events.log")).unwrap();
    for stage in ["started", "ready", "quit_requested", "clean_exit"] {
        assert!(log.lines().any(|line| line.contains(&lifecycle.identity.run_id)
            && line.contains(&format!("stage={stage}"))), "{stage}: {log}");
    }
    drop(lifecycle);
    assert_eq!(RunStore::claim(dir.path()).unwrap().1, PreviousRun::Clean);
}

#[test]
fn missing_exit_and_corrupt_record_are_interrupted_unknown() {
    let dir = tempdir().unwrap();
    let lifecycle = AppLifecycle::new("0.6.0", dir.path().join("early.log"));
    lifecycle.claim_primary(dir.path()).unwrap();
    lifecycle.request_quit(); // Intent alone does not prove a clean exit.
    drop(lifecycle);
    assert_eq!(RunStore::claim(dir.path()).unwrap().1, PreviousRun::InterruptedUnknown);
    fs::write(dir.path().join("lifecycle.json"), "{broken").unwrap();
    assert_eq!(RunStore::claim(dir.path()).unwrap().1, PreviousRun::InterruptedUnknown);
}

#[test]
fn secondary_activation_and_repeated_claim_cannot_replace_owner() {
    let dir = tempdir().unwrap();
    let owner = AppLifecycle::new("0.6.0", dir.path().join("owner.log"));
    owner.claim_primary(dir.path()).unwrap();
    let path = dir.path().join("lifecycle.json");
    let before = fs::read(&path).unwrap();
    let secondary = AppLifecycle::new("0.6.0", dir.path().join("secondary.log"));
    assert_eq!(secondary.claim_primary(dir.path()).unwrap_err().kind(), std::io::ErrorKind::WouldBlock);
    secondary.finish_exit("not_attempted");
    assert!(owner.claim_primary(dir.path()).is_err());
    owner.event("secondary_activation");
    assert_eq!(fs::read(path).unwrap(), before);
}

#[test]
fn failed_write_keeps_old_atomic_record_and_reports_without_failing_app() {
    let dir = tempdir().unwrap();
    let lifecycle = AppLifecycle::new("0.6.0", dir.path().join("early.log"));
    lifecycle.claim_primary(dir.path()).unwrap();
    let before = fs::read(dir.path().join("lifecycle.json")).unwrap();
    fs::create_dir(dir.path().join("lifecycle.json.tmp")).unwrap(); // deterministic write failure
    lifecycle.request_quit();
    assert_eq!(fs::read(dir.path().join("lifecycle.json")).unwrap(), before);
    assert!(!lifecycle.failed.load(Ordering::Acquire));
    assert!(fs::read_to_string(dir.path().join("early.log")).unwrap().contains("state_write_failed"));
}

#[test]
fn failure_cannot_be_overwritten_by_ready_or_clean_exit() {
    let dir = tempdir().unwrap();
    let lifecycle = AppLifecycle::new("0.6.0", dir.path().join("early.log"));
    lifecycle.claim_primary(dir.path()).unwrap();
    lifecycle.fail(Diagnostic::SetupFailed);
    lifecycle.tray_initialized.store(true, Ordering::Release);
    lifecycle.persistence_initialized.store(true, Ordering::Release);
    lifecycle.setup_finished();
    assert!(!lifecycle.essential_ready());
    lifecycle.finish_exit("failed");
    let record: RunRecord = serde_json::from_slice(&fs::read(dir.path().join("lifecycle.json")).unwrap()).unwrap();
    assert_eq!(record.outcome, "failed");
}

#[test]
fn diagnostics_append_and_unwritable_sink_is_nonfatal() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("early.log");
    StartupDiagnostics::new(path.clone()).record("fixture", Diagnostic::Started);
    StartupDiagnostics::new(path.clone()).record("fixture", Diagnostic::Panic);
    let log = fs::read_to_string(path).unwrap();
    assert_eq!(log.lines().count(), 2);
    StartupDiagnostics::new(dir.path().to_path_buf()).record("fixture", Diagnostic::Panic);
}
