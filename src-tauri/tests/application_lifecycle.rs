//! Real child-process fixtures; only synthetic paths supplied by the parent.
use desk_lib::{app_lifecycle::AppLifecycle, app_lifecycle_store::{RunStore, PreviousRun},
    startup_diagnostics::{self, Diagnostic}};
use std::{fs, path::Path, process::{Command, Stdio}, sync::Arc, time::{Duration, Instant}};

fn child(directory: &Path, mode: &str) -> std::process::Child {
    Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "lifecycle_child_fixture", "--nocapture"])
        .env("MOVEUP_LIFECYCLE_FIXTURE_DIR", directory)
        .env("MOVEUP_LIFECYCLE_FIXTURE_MODE", mode)
        .env("PM3_RUN_ID", "fixture-supervisor")
        .stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap()
}

fn wait(mut child: std::process::Child) -> std::process::ExitStatus {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        if let Some(status) = child.try_wait().unwrap() { return status; }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("lifecycle child exceeded deadline");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn lifecycle_child_fixture() {
    let Some(path) = std::env::var_os("MOVEUP_LIFECYCLE_FIXTURE_DIR") else { return };
    let directory = std::path::PathBuf::from(path);
    let mode = std::env::var("MOVEUP_LIFECYCLE_FIXTURE_MODE").unwrap();
    let lifecycle = Arc::new(AppLifecycle::new("fixture-version", directory.join("early.log")));
    // Prove the preexisting hook still runs without exposing payloads in our sink.
    let hook_path = directory.join("old-hook");
    std::panic::set_hook(Box::new(move |_| { let _ = fs::write(&hook_path, "called"); }));
    startup_diagnostics::install(lifecycle.clone());
    lifecycle.diagnostics.record(&lifecycle.identity.run_id, Diagnostic::Started);
    if mode == "early_failure" {
        lifecycle.fail(Diagnostic::SetupFailed);
        std::process::exit(17); // Before singleton ownership and normal logging.
    }
    if mode == "early_panic" { panic!("private-fixture-payload-must-not-be-logged"); }
    if mode == "secondary" {
        let error = lifecycle.claim_primary(&directory).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::WouldBlock);
        lifecycle.finish_exit("not_attempted");
        return;
    }
    lifecycle.claim_primary(&directory).unwrap();
    if mode == "panic_after_claim" { panic!("private-fixture-payload-must-not-be-logged"); }
    if mode == "hold" {
        fs::write(directory.join("held"), "ready").unwrap();
        loop { std::thread::sleep(Duration::from_millis(100)); }
    }
    assert_eq!(mode, "clean");
    lifecycle.request_quit();
    lifecycle.finish_exit("store=skipped_store_missing database=not_needed");
}

#[test]
fn early_failure_and_panic_leave_minimal_evidence_without_shared_state() {
    for (mode, exit, evidence) in [("early_failure", 17, "setup_failed"), ("early_panic", 101, "panic")] {
        let dir = tempfile::tempdir().unwrap();
        let status = wait(child(dir.path(), mode));
        assert_eq!(status.code(), Some(exit));
        let log = fs::read_to_string(dir.path().join("early.log")).unwrap();
        assert!(log.contains(evidence), "{log}");
        assert!(!log.contains("private-fixture-payload"));
        assert!(!dir.path().join("lifecycle.json").exists());
        if mode == "early_panic" { assert!(dir.path().join("old-hook").exists()); }
    }
}

#[test]
fn post_claim_panic_is_not_clean_and_next_run_reports_unknown_interruption() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(wait(child(dir.path(), "panic_after_claim")).code(), Some(101));
    assert!(fs::read_to_string(dir.path().join("early.log")).unwrap().contains("event=panic"));
    assert_eq!(RunStore::claim(dir.path()).unwrap().1, PreviousRun::InterruptedUnknown);
}

#[test]
fn real_secondary_and_forced_exit_preserve_owner_evidence() {
    let dir = tempfile::tempdir().unwrap();
    let mut owner = child(dir.path(), "hold");
    let deadline = Instant::now() + Duration::from_secs(20);
    while !dir.path().join("held").exists() {
        if Instant::now() >= deadline || owner.try_wait().unwrap().is_some() {
            let _ = owner.kill(); let _ = owner.wait();
            panic!("owner never acquired fixture lock");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let before = fs::read(dir.path().join("lifecycle.json")).unwrap();
    let secondary = wait(child(dir.path(), "secondary"));
    let after = fs::read(dir.path().join("lifecycle.json")).unwrap();
    owner.kill().unwrap();
    owner.wait().unwrap();
    assert!(secondary.success());
    assert_eq!(before, after);
    assert_eq!(RunStore::claim(dir.path()).unwrap().1, PreviousRun::InterruptedUnknown);
    assert!(wait(child(dir.path(), "clean")).success());
    assert_eq!(RunStore::claim(dir.path()).unwrap().1, PreviousRun::Clean);
}
