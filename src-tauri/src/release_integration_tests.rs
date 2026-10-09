use std::{fs, path::PathBuf, collections::HashMap, io::{Read, Write}, net::TcpListener};
use crate::{release_cli::execute, release_install::BuildMetadata, app_lifecycle::RunId,
    app_lifecycle_store::{RunStore, RunRecord}, runtime_readiness::{ReadinessStatus, ReadinessState}};
#[path = "../tests/support/node.rs"] mod node;

struct Fixture { _temp: tempfile::TempDir, root: PathBuf, source: PathBuf, run: PathBuf }
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("installed releases");
        let source = temp.path().join("complete build");
        let run = temp.path().join("runtime");
        fs::create_dir(&source).unwrap(); fs::create_dir(&run).unwrap();
        fs::create_dir_all(source.join("dist/assets")).unwrap();
        fs::write(source.join("dist/index.html"), "<html>fixture</html>").unwrap();
        fs::write(source.join("dist/welcome.html"), "<html>fixture</html>").unwrap();
        fs::write(source.join("dist/assets/app.js"), "fixture()").unwrap();
        fs::write(source.join("desk.exe"), "synthetic executable").unwrap();
        Self { _temp: temp, root, source, run }
    }
    fn metadata(&self, version: &str) {
        let metadata = BuildMetadata { schema_version: 1, version: version.into(),
            app_id: "io.zntl.desk".into(), readiness_contract: 1, build_mode: "release".into() };
        fs::write(self.source.join("release.json"), serde_json::to_vec(&metadata).unwrap()).unwrap();
    }
    fn stage(&self, version: &str) -> Result<serde_json::Value, String> {
        self.metadata(version);
        self.command(&["stage", "--source", self.source.to_str().unwrap(), "--version", version,
            "--run-directory", self.run.to_str().unwrap()], &HashMap::new())
    }
    fn command(&self, args: &[&str], env: &HashMap<String, String>) -> Result<serde_json::Value, String> {
        let mut args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        args.extend(["--root".into(), self.root.to_string_lossy().into()]);
        execute(&args, |key| env.get(key).cloned())
    }
    fn bytes(&self, file: &str) -> Vec<u8> { fs::read(self.root.join(file)).unwrap() }
}

#[test]
fn release_install_complete_immutable_and_launch_directory_independent() {
    let f = Fixture::new();
    assert_eq!(f.stage("0.6.1").unwrap()["cutover"], false);
    let original = f.bytes("candidates.json");
    assert!(f.stage("0.6.1").unwrap_err().contains("immutable"));
    assert_eq!(f.bytes("candidates.json"), original);
    let expected = crate::remote_server::default_dist_path(&f.root.join("0.6.1/desk.exe"));
    assert!(expected.join("assets/app.js").is_file());
    assert!(expected.is_absolute());
    fs::remove_file(f.source.join("dist/index.html")).unwrap();
    assert!(f.stage("0.6.2").is_err());
    assert!(!f.root.join("0.6.2").exists());
    assert_eq!(f.bytes("candidates.json"), original);
}

#[test]
fn release_missing_assets_legacy_metadata_and_escape_leave_state_unchanged() {
    let f = Fixture::new(); f.stage("0.6.1").unwrap();
    crate::last_known_good::LastKnownGood::new(f.root.join("last-known-good")).mark("0.6.1").unwrap();
    let original = f.bytes("candidates.json");
    for version in ["../outside", "C:/outside", "0.6.0", "00.6.2", "0.6.2;bad"] {
        assert!(f.stage(version).is_err(), "{version}");
    }
    fs::remove_dir_all(f.source.join("dist")).unwrap();
    assert!(f.stage("0.6.2").is_err());
    assert_eq!(f.bytes("candidates.json"), original);
    assert_eq!(f.bytes("last-known-good"), b"0.6.1\n");
    assert!(!f.root.join("0.6.2").exists());
}

#[test]
fn release_referenced_asset_and_welcome_page_are_required() {
    let f = Fixture::new();
    fs::write(f.source.join("dist/index.html"), "<script src=\"/assets/missing.js\"></script>").unwrap();
    assert!(f.stage("0.6.1").unwrap_err().contains("missing assets"));
    fs::write(f.source.join("dist/index.html"), "<script src=\"/assets/app.js\"></script>").unwrap();
    fs::remove_file(f.source.join("dist/welcome.html")).unwrap();
    assert!(f.stage("0.6.1").is_err());
}

#[test]
fn release_store_inside_source_cannot_recursively_stage_itself() {
    let f = Fixture::new(); f.metadata("0.6.1");
    let root = f.source.join("nested releases");
    let args = ["stage", "--root", root.to_str().unwrap(), "--source", f.source.to_str().unwrap(),
        "--version", "0.6.1", "--run-directory", f.run.to_str().unwrap()]
        .map(str::to_string);
    assert!(execute(&args, |_| None).unwrap_err().contains("inside the build source"));
    assert!(!root.join("0.6.1").exists());
}

#[test]
fn release_order_retention_protects_good_and_running() {
    let f = Fixture::new(); f.stage("0.6.1").unwrap();
    crate::last_known_good::LastKnownGood::new(f.root.join("last-known-good")).mark("0.6.1").unwrap();
    f.stage("0.6.2").unwrap(); f.stage("0.6.3").unwrap();
    let (owner, _) = RunStore::claim(&f.run).unwrap();
    let identity = RunId::new("0.6.2");
    owner.write(&RunRecord { identity, outcome: "started".into(), quit_requested: false, flush: "not_attempted".into() }).unwrap();
    assert_eq!(f.stage("0.6.4").unwrap()["removed"], serde_json::json!(["0.6.3"]));
    assert_eq!(f.bytes("candidates"), b"0.6.4\n0.6.1\n0.6.2\n");
    f.command(&["candidates"], &HashMap::new()).unwrap();
    assert_eq!(f.bytes("candidates"), b"0.6.4\n0.6.1\n0.6.2\n");
}

#[test]
fn release_protected_legacy_and_running_bound_refuses_stage_without_publication() {
    let f = Fixture::new(); f.stage("0.6.1").unwrap(); f.stage("0.6.2").unwrap();
    fs::create_dir(f.root.join("0.6.0")).unwrap();
    fs::write(f.root.join("0.6.0/desk.exe"), "legacy unknown running identity").unwrap();
    crate::last_known_good::LastKnownGood::new(f.root.join("last-known-good")).mark("0.6.1").unwrap();
    let (owner, _) = RunStore::claim(&f.run).unwrap();
    owner.write(&RunRecord { identity: RunId::new("0.6.2"), outcome: "started".into(), quit_requested: false, flush: "not_attempted".into() }).unwrap();
    let candidates = f.bytes("candidates.json");
    assert!(f.stage("0.6.3").unwrap_err().contains("protected"));
    assert!(!f.root.join("0.6.3").exists());
    assert!(f.root.join("0.6.0/desk.exe").is_file());
    assert_eq!(f.bytes("candidates.json"), candidates);
    assert_eq!(f.bytes("last-known-good"), b"0.6.1\n");
}

#[test]
fn release_competing_mutation_and_publication_failure_preserve_previous() {
    let f = Fixture::new(); f.stage("0.6.1").unwrap();
    let lock = crate::release_install::StoreLock::acquire(&f.root).unwrap();
    assert!(f.stage("0.6.2").is_err()); drop(lock);
    // A foreign directory at the JSON pathname cannot be overwritten.
    let previous = f.bytes("candidates");
    fs::rename(f.root.join("candidates.json"), f.root.join("saved.json")).unwrap();
    fs::create_dir(f.root.join("candidates.json")).unwrap();
    assert!(f.stage("0.6.2").is_err());
    assert!(!f.root.join("0.6.2").exists());
    assert_eq!(f.bytes("candidates"), previous);
    assert_eq!(fs::read_dir(&f.root).unwrap().filter_map(Result::ok)
        .filter(|e| e.file_name().to_string_lossy().starts_with(".stage-")).count(), 0);
}

#[test]
fn release_symlink_or_junction_escape_is_refused() {
    let f = Fixture::new();
    let outside = f._temp.path().join("outside"); fs::create_dir(&outside).unwrap();
    #[cfg(windows)] {
        let status = std::process::Command::new("cmd").args(["/c", "mklink", "/J"])
            .arg(f.source.join("escaped")).arg(&outside).output().unwrap();
        assert!(status.status.success(), "junction fixture failed");
    }
    #[cfg(not(windows))] std::os::unix::fs::symlink(&outside, f.source.join("escaped")).unwrap();
    assert!(f.stage("0.6.1").unwrap_err().contains("symlink/junction"));
}

fn promotion_env(f: &Fixture, pid: u32, port: u16) -> HashMap<String, String> {
    [("PM3_CANDIDATE_ID", "0.6.1".into()), ("PM3_CANDIDATE_PID", pid.to_string()),
        ("PM3_OBSERVED_PID", pid.to_string()), ("PM3_RUN_ID", "owned-run".into()),
        ("PM3_OBSERVED_RUN_ID", "owned-run".into()), ("PM3_SERVICE", "moveup/desk".into()),
        ("DESK_SUPERVISION", "pm3".into()), ("DESK_PM3_SERVICE", "moveup/desk".into()),
        ("DESK_PM3_NODE", node::fixture_node().to_string_lossy().into()),
        ("DESK_PM3_HOME", f.run.to_string_lossy().into()), ("DESK_READINESS_PORT", port.to_string()),
        ("PM3_PROBE_URL", format!("http://127.0.0.1:{port}/display/readiness"))]
        .into_iter().map(|(k,v)| (k.into(),v)).collect()
}

fn readiness(identity: &RunId) -> ReadinessStatus {
    let mut body = ReadinessStatus { app_id: "io.zntl.desk".into(), version: identity.version.clone(),
        pid: identity.pid, run_id: identity.run_id.clone(), supervisor_run_id: identity.supervisor_run_id.clone(),
        build_mode: "release".into(), ready: false, state: ReadinessState::Initializing, reason: None,
        tray_initialized: true, persistence_initialized: true, setup_complete: true, remote_listening: true,
        sensor_present: false, session_store_available: true, quit_requested: false };
    body.compute_state(); body
}

fn serve(listener: TcpListener, body: ReadinessStatus) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream.set_read_timeout(Some(std::time::Duration::from_secs(3))).unwrap();
        let mut request = [0; 2048]; stream.read(&mut request).unwrap();
        let body = serde_json::to_string(&body).unwrap();
        write!(stream, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body).unwrap();
    })
}

#[test]
fn release_promotion_requires_owned_pid_run_and_fresh_typed_readiness() {
    let f = Fixture::new(); f.stage("0.6.1").unwrap();
    let (owner, _) = RunStore::claim(&f.run).unwrap();
    let mut identity = RunId::new("0.6.1"); identity.build_mode = "release".into();
    identity.supervisor_run_id = Some("owned-run".into());
    owner.write(&RunRecord { identity: identity.clone(), outcome: "started".into(), quit_requested: false, flush: "not_attempted".into() }).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap(); let port = listener.local_addr().unwrap().port();
    let env = promotion_env(&f, identity.pid, port);
    for (key, value) in [("PM3_OBSERVED_PID", "1"), ("PM3_OBSERVED_RUN_ID", "foreign"), ("PM3_CANDIDATE_ID", "0.6.9")] {
        let mut wrong = env.clone(); wrong.insert(key.into(), value.into());
        assert!(f.command(&["promote", "--run-directory", f.run.to_str().unwrap()], &wrong).is_err());
        assert!(!f.root.join("last-known-good").exists());
    }
    let mut foreign = readiness(&identity); foreign.run_id = uuid::Uuid::new_v4().to_string();
    let server = serve(listener, foreign);
    assert!(f.command(&["promote", "--run-directory", f.run.to_str().unwrap()], &env).is_err()); server.join().unwrap();
    assert!(!f.root.join("last-known-good").exists());
    let listener = TcpListener::bind("127.0.0.1:0").unwrap(); let port = listener.local_addr().unwrap().port();
    let server = serve(listener, readiness(&identity));
    let env = promotion_env(&f, identity.pid, port);
    assert_eq!(f.command(&["promote", "--run-directory", f.run.to_str().unwrap()], &env).unwrap()["status"], "promoted");
    server.join().unwrap(); assert_eq!(f.bytes("last-known-good"), b"0.6.1\n");
    let candidates_before = f.bytes("candidates.json");
    let legacy_before = f.bytes("candidates");
    fs::create_dir(f.root.join(".last-known-good.tmp")).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap(); let port = listener.local_addr().unwrap().port();
    let server = serve(listener, readiness(&identity));
    let env = promotion_env(&f, identity.pid, port);
    assert!(f.command(&["promote", "--run-directory", f.run.to_str().unwrap()], &env).is_err());
    server.join().unwrap();
    assert_eq!(f.bytes("last-known-good"), b"0.6.1\n");
    assert_eq!(f.bytes("candidates.json"), candidates_before);
    assert_eq!(f.bytes("candidates"), legacy_before);
    drop(owner);
    assert!(f.command(&["promote", "--run-directory", f.run.to_str().unwrap()], &env).is_err());
}

#[test]
fn release_status_and_invalid_maintenance_are_explicit() {
    let f = Fixture::new();
    let status = f.command(&["status"], &HashMap::new()).unwrap();
    assert_eq!(status["candidate_status"], "missing"); assert_eq!(status["marker_status"], "missing");
    assert!(execute(&["activate".into()], |_| Some("malformed".into())).is_err());
    assert!(execute(&["unknown".into()], |_| None).is_err());
    assert!(execute(&["status".into(), "--root".into(), "relative".into()], |_| None).is_err());
}
