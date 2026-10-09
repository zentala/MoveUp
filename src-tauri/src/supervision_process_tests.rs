//! Real hidden Node child, synthetic launcher only; never contacts PM3.
use crate::{supervision::{Configuration, Transport}, supervision_process::PublicLauncher};
use std::{path::PathBuf, time::Duration};

fn fixture(service: &str) -> (tempfile::TempDir, Configuration) {
    let temp = tempfile::tempdir().unwrap();
    std::fs::create_dir(temp.path().join("releases")).unwrap();
    std::fs::write(temp.path().join("releases/launch.mjs"),
        include_str!("../../tests/fixtures/supervision-launcher.mjs")).unwrap();
    let node = PathBuf::from(std::env::var("PM3_DAEMON_NODE")
        .or_else(|_| std::env::var("DESK_PM3_NODE"))
        .expect("Set an absolute PM3_DAEMON_NODE for synthetic Node transport tests"));
    assert!(node.is_absolute() && node.is_file());
    let cfg = Configuration { node, home: temp.path().to_path_buf(), service: service.into() };
    (temp, cfg)
}

#[tokio::test]
async fn supervision_process_preserves_structured_argv_and_inherited_fixture_keys() {
    let (_temp, cfg) = fixture("fixture/argv");
    let out = PublicLauncher.invoke(&cfg, "suspend", "intent with spaces & $", Duration::from_secs(2)).await.unwrap();
    assert_eq!(out.exit, 0);
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["action"], "suspend");
    assert_eq!(v["args"][1], "intent with spaces & $");
    assert_eq!(v["home"].as_str(), cfg.home.to_str());
    for (field, key) in [("ipc", "PM3_TEST_IPC_PATH"), ("run", "PM3_TEST_RUN_DIR"), ("pidfile", "PM3_TEST_PIDFILE_PATH")] {
        assert_eq!(v[field].as_str(), std::env::var(key).ok().as_deref());
    }
}

#[tokio::test]
async fn supervision_process_enforces_both_output_caps() {
    for service in ["fixture/output", "fixture/stderr"] {
        let (_temp, cfg) = fixture(service);
        let err = PublicLauncher.invoke(&cfg, "suspend", "quit", Duration::from_secs(2)).await.err().unwrap();
        assert!(err.contains("limit"), "{err}");
    }
}

#[tokio::test]
async fn supervision_process_kills_timed_out_helper() {
    let (_temp, cfg) = fixture("fixture/timeout");
    let start = std::time::Instant::now();
    let err = PublicLauncher.invoke(&cfg, "suspend", "quit", Duration::from_millis(150)).await.err().unwrap();
    assert!(err.contains("timed out"));
    assert!(start.elapsed() < Duration::from_secs(2));
}

#[tokio::test]
async fn supervision_process_accepts_frozen_acknowledgement() {
    let (_temp, cfg) = fixture("fixture/desk");
    let response = PublicLauncher.invoke(&cfg, "suspend", "quit", Duration::from_secs(2)).await.unwrap();
    assert!(crate::supervision::confirms(&response, "suspend", &cfg.service, "quit"));
}
