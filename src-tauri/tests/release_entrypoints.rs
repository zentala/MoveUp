//! Actual maintenance binary failures remain confined to synthetic stores.
use std::{fs, path::PathBuf, process::{Command, Output, Stdio}, time::{Duration, Instant}};
use desk_lib::{app_lifecycle::RunId, app_lifecycle_store::{RunStore, RunRecord}};

struct Fixture { temp: tempfile::TempDir, root: PathBuf, source: PathBuf, run: PathBuf }
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("store"); let source = temp.path().join("build");
        let run = temp.path().join("runtime");
        fs::create_dir_all(source.join("dist/assets")).unwrap(); fs::create_dir(&run).unwrap();
        fs::write(source.join("desk.exe"), b"synthetic binary").unwrap();
        fs::write(source.join("dist/index.html"), "<script src=\"/assets/app.js\"></script>").unwrap();
        fs::write(source.join("dist/welcome.html"), "<html>fixture</html>").unwrap();
        fs::write(source.join("dist/assets/app.js"), "fixture()").unwrap();
        Self { temp, root, source, run }
    }
    fn invoke(&self, args: &[&str]) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_desk"));
        command.arg("--release").args(args).current_dir(self.temp.path())
            .env("LOCALAPPDATA", self.temp.path()).env("APPDATA", self.temp.path())
            .env("TEMP", self.temp.path()).env("TMP", self.temp.path())
            .stdout(Stdio::piped()).stderr(Stdio::piped());
        for key in ["DESK_SUPERVISION", "DESK_PM3_SERVICE", "PM3_SERVICE", "DESK_PM3_NODE", "DESK_PM3_HOME", "DESK_PM3_CONFIG"] {
            command.env_remove(key);
        }
        let mut child = command.spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while child.try_wait().unwrap().is_none() {
            if Instant::now() >= deadline { child.kill().unwrap(); panic!("maintenance timeout"); }
            std::thread::sleep(Duration::from_millis(10));
        }
        let output = child.wait_with_output().unwrap();
        assert!(!self.run.join("lifecycle.json").exists(), "maintenance bootstrapped lifecycle");
        assert!(!self.run.join("lifecycle.owner").exists(), "maintenance claimed ownership");
        for entry in walk(self.temp.path()) {
            assert!(!entry.file_name().unwrap().to_string_lossy().starts_with("moveup-startup-"), "startup diagnostic created");
            assert!(!matches!(entry.extension().and_then(|v| v.to_str()), Some("db" | "sqlite")), "database created: {entry:?}");
        }
        output
    }
    fn stage(&self, version: &str) -> Output {
        fs::write(self.source.join("release.json"), serde_json::to_vec(&serde_json::json!({
            "schema_version":1,"version":version,"app_id":"io.zntl.desk","readiness_contract":1,"build_mode":"release"
        })).unwrap()).unwrap();
        self.invoke(&["stage", "--root", self.root.to_str().unwrap(), "--source", self.source.to_str().unwrap(),
            "--version", version, "--run-directory", self.run.to_str().unwrap()])
    }
    fn ok(&self, version: &str) {
        let out = self.stage(version);
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    }
    fn bytes(&self, file: &str) -> Vec<u8> { fs::read(self.root.join(file)).unwrap() }
}

fn walk(path: &std::path::Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() { paths.extend(walk(&path)); } else { paths.push(path); }
    }
    paths
}
fn failed(output: Output, expected: &str) {
    assert_eq!(output.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(report["status"], "failed");
    assert!(report["error"].as_str().unwrap().contains(expected), "{report}");
}

#[test]
fn invalid_arguments_and_paths_never_bootstrap_application() {
    let f = Fixture::new();
    for args in [vec![], vec!["unknown"], vec!["status", "--root"],
        vec!["status", "--root", "relative"], vec!["status", "--root", "x", "--root", "y"],
        vec!["activate"]] {
        failed(f.invoke(&args), "");
    }
    for version in ["../escape", "00.6.2", "0.6.2;bad", "0.6.0"] {
        failed(f.stage(version), "");
    }
    assert!(!f.temp.path().join("escape").exists());
}

#[test]
fn immutable_missing_assets_and_metadata_failures_preserve_candidates() {
    let f = Fixture::new(); f.ok("0.6.1");
    let before = f.bytes("candidates.json");
    failed(f.stage("0.6.1"), "immutable");
    fs::remove_file(f.source.join("dist/assets/app.js")).unwrap();
    fs::write(f.source.join("dist/assets/other.js"), "other()").unwrap();
    failed(f.stage("0.6.2"), "missing assets");
    fs::write(f.source.join("dist/assets/app.js"), "fixture()").unwrap();
    fs::remove_file(f.source.join("dist/welcome.html")).unwrap();
    failed(f.stage("0.6.2"), "complete");
    fs::write(f.source.join("dist/welcome.html"), "fixture").unwrap();
    fs::write(f.source.join("release.json"), "{}").unwrap();
    failed(f.invoke(&["stage", "--root", f.root.to_str().unwrap(), "--source", f.source.to_str().unwrap(),
        "--version", "0.6.2", "--run-directory", f.run.to_str().unwrap()]), "missing field");
    assert_eq!(f.bytes("candidates.json"), before);
    assert!(!f.root.join("0.6.2").exists());
}

#[test]
fn candidate_order_deduplication_retention_and_publication_failure_use_binary() {
    let f = Fixture::new(); f.ok("0.6.1");
    fs::write(f.root.join("last-known-good"), b"0.6.1\n").unwrap();
    f.ok("0.6.2"); f.ok("0.6.3"); f.ok("0.6.4");
    assert_eq!(f.bytes("candidates"), b"0.6.4\n0.6.1\n0.6.3\n");
    fs::write(f.root.join("last-known-good"), b"0.6.4\n").unwrap();
    assert!(f.invoke(&["candidates", "--root", f.root.to_str().unwrap()]).status.success());
    let ids: serde_json::Value = serde_json::from_slice(&f.bytes("candidates.json")).unwrap();
    assert_eq!(ids["candidates"].as_array().unwrap().len(), 3);
    assert_eq!(ids["candidates"][0]["id"], "0.6.4");
    let before = f.bytes("candidates");
    fs::rename(f.root.join("candidates.json"), f.root.join("saved.json")).unwrap();
    fs::create_dir(f.root.join("candidates.json")).unwrap();
    failed(f.stage("0.6.5"), "previous legacy candidates restored");
    assert_eq!(f.bytes("candidates"), before);
    assert_eq!(f.bytes("last-known-good"), b"0.6.4\n");
    assert!(!f.root.join("0.6.5").exists());
}

#[test]
fn unsafe_retention_protects_legacy_and_known_good_through_binary() {
    let f = Fixture::new(); f.ok("0.6.1");
    fs::write(f.root.join("last-known-good"), b"0.6.1\n").unwrap();
    for id in ["0.6.0", "0.5.0"] {
        fs::create_dir(f.root.join(id)).unwrap(); fs::write(f.root.join(id).join("desk.exe"), "legacy").unwrap();
    }
    let before = f.bytes("candidates.json");
    failed(f.stage("0.6.2"), "protected");
    assert_eq!(f.bytes("candidates.json"), before);
    assert_eq!(f.bytes("last-known-good"), b"0.6.1\n");
}

#[test]
fn junction_and_source_overlap_are_refused_through_binary() {
    let f = Fixture::new();
    let outside = f.temp.path().join("outside"); fs::create_dir(&outside).unwrap();
    #[cfg(windows)] {
        let out = Command::new("cmd").args(["/c", "mklink", "/J"]).arg(f.source.join("escaped")).arg(&outside).output().unwrap();
        assert!(out.status.success());
    }
    #[cfg(not(windows))] std::os::unix::fs::symlink(&outside, f.source.join("escaped")).unwrap();
    failed(f.stage("0.6.1"), "symlink/junction");
    // Remove only the fixture link, never its target.
    #[cfg(windows)] fs::remove_dir(f.source.join("escaped")).unwrap();
    #[cfg(not(windows))] fs::remove_file(f.source.join("escaped")).unwrap();
    f.stage("0.6.1");
    let nested = f.source.join("nested");
    failed(f.invoke(&["stage", "--root", nested.to_str().unwrap(), "--source", f.source.to_str().unwrap(),
        "--version", "0.6.1", "--run-directory", f.run.to_str().unwrap()]), "inside the build source");
}

#[test]
fn running_and_known_good_retention_preserve_lifecycle_record() {
    let f = Fixture::new(); f.ok("0.6.1"); f.ok("0.6.2"); f.ok("0.6.3");
    fs::write(f.root.join("last-known-good"), b"0.6.1\n").unwrap();
    let (owner, _) = RunStore::claim(&f.run).unwrap();
    owner.write(&RunRecord { identity: RunId::new("0.6.2"), outcome: "started".into(), quit_requested: false, flush: "not_attempted".into() }).unwrap();
    let before = fs::read(f.run.join("lifecycle.json")).unwrap();
    // invoke's no-bootstrap assertions are for absent owners; this case has one.
    fs::write(f.source.join("release.json"), serde_json::to_vec(&serde_json::json!({
        "schema_version":1,"version":"0.6.4","app_id":"io.zntl.desk","readiness_contract":1,"build_mode":"release"
    })).unwrap()).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_desk")).args(["--release", "stage", "--root"]).arg(&f.root)
        .arg("--source").arg(&f.source).args(["--version", "0.6.4", "--run-directory"]).arg(&f.run)
        .env("DESK_SUPERVISION", "bridge").output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(f.bytes("candidates"), b"0.6.4\n0.6.1\n0.6.2\n");
    assert_eq!(fs::read(f.run.join("lifecycle.json")).unwrap(), before);
}
