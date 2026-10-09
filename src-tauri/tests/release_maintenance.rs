//! Real pre-GUI CLI/callback dispatch, with temporary stores and synthetic health.
use std::{fs, io::{Read, Write}, net::TcpListener, path::Path, process::Command};
use desk_lib::{app_lifecycle::RunId, app_lifecycle_store::{RunStore, RunRecord},
    runtime_readiness::{ReadinessStatus, ReadinessState}};
#[path = "support/node.rs"] mod node;

fn stage(source: &Path, root: &Path, run: &Path, version: &str) {
    fs::write(source.join("release.json"), serde_json::to_vec(&serde_json::json!({
        "schema_version": 1, "version": version, "app_id": "io.zntl.desk",
        "readiness_contract": 1, "build_mode": "release" })).unwrap()).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_desk"))
        .args(["--release", "stage", "--root"]).arg(root).arg("--source").arg(source)
        .args(["--version", version, "--run-directory"]).arg(run)
        .env_remove("DESK_SUPERVISION").env_remove("DESK_PM3_SERVICE").env_remove("PM3_SERVICE")
        .env_remove("DESK_PM3_NODE").env_remove("DESK_PM3_HOME").output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["status"], "staged");
}

fn check_promotion(use_callback: bool) {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("complete source"); let root = temp.path().join("release store");
    let run = temp.path().join("run evidence");
    fs::create_dir_all(source.join("dist/assets")).unwrap(); fs::create_dir(&run).unwrap();
    fs::write(source.join("dist/index.html"), "<html>fixture</html>").unwrap();
    fs::write(source.join("dist/welcome.html"), "<html>fixture</html>").unwrap();
    fs::write(source.join("dist/assets/app.js"), "fixture()").unwrap();
    fs::copy(env!("CARGO_BIN_EXE_desk"), source.join("desk.exe")).unwrap();
    stage(&source, &root, &run, "0.6.1");
    fs::write(source.join("desk.exe"), b"broken newest executable").unwrap();
    stage(&source, &root, &run, "0.6.2");
    assert!(!run.join("lifecycle.json").exists(), "maintenance cannot bootstrap lifecycle");
    let (owner, _) = RunStore::claim(&run).unwrap();
    let mut identity = RunId::new("0.6.1"); identity.build_mode = "release".into();
    identity.supervisor_run_id = Some("owned-run".into());
    owner.write(&RunRecord { identity: identity.clone(), outcome: "started".into(), quit_requested: false, flush: "not_attempted".into() }).unwrap();
    let before = fs::read(run.join("lifecycle.json")).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap(); let port = listener.local_addr().unwrap().port();
    let mut body = ReadinessStatus { app_id: "io.zntl.desk".into(), version: identity.version.clone(),
        pid: identity.pid, run_id: identity.run_id.clone(), supervisor_run_id: identity.supervisor_run_id.clone(),
        build_mode: "release".into(), ready: false, state: ReadinessState::Initializing, reason: None,
        tray_initialized: true, persistence_initialized: true, setup_complete: true, remote_listening: true,
        sensor_present: false, session_store_available: true, quit_requested: false };
    body.compute_state();
    let server = std::thread::spawn(move || {
        listener.set_nonblocking(true).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(12);
        let mut requests = 0;
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    stream.set_read_timeout(Some(std::time::Duration::from_secs(2))).unwrap();
                    let mut request = [0; 2048]; stream.read(&mut request).unwrap();
                    let mut response = serde_json::to_value(&body).unwrap();
                    if requests == 0 { response["run_id"] = uuid::Uuid::new_v4().to_string().into(); }
                    if requests == 1 { response["build_mode"] = "debug".into(); }
                    let text = serde_json::to_string(&response).unwrap();
                    write!(stream, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", text.len(), text).unwrap();
                    requests += 1;
                    if requests == 3 { return true; }
                },
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if std::time::Instant::now() >= deadline { return false; }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }, Err(e) => panic!("{e}"),
            }
        }
    });
    let node = node::fixture_node();
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/release-callback.mjs");
    let callback = |id: &str, wrong: Option<(&str, &str)>| {
        let mut command = if use_callback {
            let mut command = Command::new(&node);
            command.arg(&script).arg(&root).arg(&run).arg(&run).arg(port.to_string()); command
        } else {
            let mut command = Command::new(root.join("0.6.1/desk.exe"));
            command.args(["--release", "promote", "--root"]).arg(&root)
                .arg("--run-directory").arg(&run); command
        };
        command
            .env("PM3_CANDIDATE_ID", id).env("PM3_CANDIDATE_PID", identity.pid.to_string())
            .env("PM3_OBSERVED_PID", identity.pid.to_string()).env("PM3_RUN_ID", "owned-run")
            .env("PM3_OBSERVED_RUN_ID", "owned-run").env("PM3_SERVICE", "moveup/desk")
            .env("DESK_SUPERVISION", "pm3").env("DESK_PM3_SERVICE", "moveup/desk")
            .env("DESK_PM3_NODE", &node).env("DESK_PM3_HOME", &run)
            .env("DESK_READINESS_PORT", port.to_string())
            .env("PM3_PROBE_URL", format!("http://127.0.0.1:{port}/display/readiness"));
        if let Some((key, value)) = wrong { command.env(key, value); }
        command.output().unwrap()
    };
    assert!(!callback("0.6.9", None).status.success());
    assert!(!root.join("last-known-good").exists());
    for (key, value) in [("PM3_OBSERVED_PID", "1"), ("PM3_OBSERVED_RUN_ID", "foreign")] {
        assert!(!callback("0.6.1", Some((key, value))).status.success());
        assert!(!root.join("last-known-good").exists());
    }
    assert!(!callback("0.6.1", None).status.success(), "foreign application run must fail");
    assert!(!root.join("last-known-good").exists());
    // All fresh identities now match; debug readiness must not certify a release.
    fs::write(root.join("last-known-good"), b"0.6.2\n").unwrap();
    let marker_before = fs::read(root.join("last-known-good")).unwrap();
    let legacy_before = fs::read(root.join("candidates")).unwrap();
    let json_before = fs::read(root.join("candidates.json")).unwrap();
    let debug = callback("0.6.1", None);
    assert_eq!(debug.status.code(), Some(1), "debug readiness cannot promote a release");
    let error: serde_json::Value = serde_json::from_slice(&debug.stderr).unwrap();
    if use_callback { assert_eq!(error["error"], "Maintenance callback failed: 1"); }
    else { assert!(error["error"].as_str().unwrap().contains("build mode")); }
    assert_eq!(fs::read(root.join("last-known-good")).unwrap(), marker_before);
    assert_eq!(fs::read(root.join("candidates")).unwrap(), legacy_before);
    assert_eq!(fs::read(root.join("candidates.json")).unwrap(), json_before);
    assert_eq!(fs::read(run.join("lifecycle.json")).unwrap(), before);
    let success = callback("0.6.1", None); let reached = server.join().unwrap();
    assert!(success.status.success(), "stdout={} stderr={}", String::from_utf8_lossy(&success.stdout), String::from_utf8_lossy(&success.stderr));
    assert!(reached, "callback never reached readiness");
    assert_eq!(fs::read(root.join("last-known-good")).unwrap(), b"0.6.1\n");
    let failure = callback("0.6.2", None); assert!(!failure.status.success());
    assert_eq!(fs::read(root.join("last-known-good")).unwrap(), b"0.6.1\n");
    assert_eq!(fs::read(run.join("lifecycle.json")).unwrap(), before);
}

#[test]
fn promotion_entrypoint_runs_before_gui_and_preserves_active_run_record() {
    check_promotion(false);
}

#[test]
fn callback_dispatches_observed_retained_binary_when_newest_cannot_execute() {
    check_promotion(true);
}
