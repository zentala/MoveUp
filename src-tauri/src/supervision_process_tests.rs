//! Real hidden Node child, synthetic launcher only; never contacts PM3.
use crate::{supervision::{Configuration, Transport}, supervision_process::PublicLauncher};
use std::time::Duration;

#[path = "../tests/support/node.rs"] mod node;
use node::fixture_node;

fn fixture(service: &str) -> (tempfile::TempDir, Configuration) {
    let temp = tempfile::tempdir().unwrap();
    std::fs::create_dir(temp.path().join("releases")).unwrap();
    std::fs::write(temp.path().join("releases/launch.mjs"),
        include_str!("../../tests/fixtures/supervision-launcher.mjs")).unwrap();
    let node = fixture_node();
    let cfg = Configuration { node, home: temp.path().to_path_buf(), service: service.into() };
    (temp, cfg)
}

#[tokio::test]
async fn supervision_process_preserves_structured_argv_and_inherited_fixture_keys() {
    let (_temp, cfg) = fixture("fixture/argv");
    let out = PublicLauncher.invoke(&cfg, "stop", "", Duration::from_secs(2)).await.unwrap();
    assert_eq!(out.exit, 0);
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["action"], "stop");
    assert_eq!(v["args"], serde_json::json!([]));
    assert_eq!(v["home"].as_str(), cfg.home.to_str());
    for (field, key) in [("ipc", "PM3_TEST_IPC_PATH"), ("run", "PM3_TEST_RUN_DIR"), ("pidfile", "PM3_TEST_PIDFILE_PATH")] {
        assert_eq!(v[field].as_str(), std::env::var(key).ok().as_deref());
    }
}

#[tokio::test]
async fn supervision_process_enforces_both_output_caps() {
    for service in ["fixture/output", "fixture/stderr"] {
        let (_temp, cfg) = fixture(service);
        let err = PublicLauncher.invoke(&cfg, "stop", "", Duration::from_secs(2)).await.err().unwrap();
        assert!(err.contains("limit"), "{err}");
    }
}

#[tokio::test]
async fn supervision_process_kills_timed_out_helper() {
    let (_temp, cfg) = fixture("fixture/timeout");
    let start = std::time::Instant::now();
    let err = PublicLauncher.invoke(&cfg, "stop", "", Duration::from_millis(150)).await.err().unwrap();
    assert!(err.contains("timed out"));
    assert!(start.elapsed() < Duration::from_secs(2));
}

#[tokio::test]
async fn supervision_process_accepts_ordinary_plaintext_acknowledgement() {
    let (_temp, cfg) = fixture("fixture/desk");
    std::fs::write(cfg.home.join("releases/launch.mjs"), "process.stdout.write('desk: stopped\\n');").unwrap();
    let response = PublicLauncher.invoke(&cfg, "stop", "", Duration::from_secs(2)).await.unwrap();
    assert!(crate::supervision::confirms(&response, "stop", &cfg.service, ""));
}
