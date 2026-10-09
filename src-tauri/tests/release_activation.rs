//! Real consumer + synthetic public launcher; no daemon or registry.
use std::{fs, process::Command};
#[path = "support/node.rs"] mod node;

#[test]
fn activation_selects_installed_config_and_consumes_public_plaintext() {
    let temp = tempfile::tempdir().unwrap(); let root = temp.path();
    let home = root.join("pm3 home"); let installed = root.join("installed config");
    let unrelated = root.join("unrelated"); let store = root.join("store");
    fs::create_dir_all(home.join("releases")).unwrap();
    for dir in [&installed, &unrelated, &store] { fs::create_dir(dir).unwrap(); }
    fs::write(store.join("candidates.json"), r#"{"version":1,"candidates":[]}"#).unwrap();
    let config_file = installed.join("pm3.yaml"); let node = node::fixture_node();
    let record = root.join("launcher-record.json");
    fs::write(home.join("releases/launch.mjs"), r#"
import fs from 'node:fs';
const config = JSON.parse(fs.readFileSync(process.env.PM3_CONFIG, 'utf8'));
fs.writeFileSync(process.env.FIXTURE_RECORD, JSON.stringify({args:process.argv.slice(2),
  config:process.env.PM3_CONFIG,cwd:process.cwd(),project:config.name,home:process.env.PM3_HOME}));
console.log(process.env.FIXTURE_OUTPUT);
process.exitCode = Number(process.env.FIXTURE_EXIT);
"#).unwrap();
    let module = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/pm3-config.mjs");
    // Generate through the real owner rather than asserting an invented shape.
    let out = Command::new(&node).args(["--input-type=module", "-e",
        "import {pathToFileURL} from 'node:url'; import fs from 'node:fs'; const {createConfiguration}=await import(pathToFileURL(process.argv[1])); const [installRoot,candidatesFile,nodeExecutable,pm3Home,callbackFile,runDirectory,configFile]=process.argv.slice(2); fs.writeFileSync(configFile,JSON.stringify(createConfiguration({installRoot,candidatesFile,nodeExecutable,pm3Home,callbackFile,runDirectory,configFile}).config));"])
        .arg(module).arg(&store).arg(store.join("candidates.json")).arg(&node).arg(&home)
        .arg(installed.join("release-callback.mjs")).arg(&unrelated).arg(&config_file).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let call = |output: &str, exit: &str, config: &std::path::Path| {
        Command::new(env!("CARGO_BIN_EXE_desk")).args(["--release", "activate"])
            .current_dir(&unrelated).env("DESK_SUPERVISION", "pm3").env("DESK_PM3_SERVICE", "moveup/desk")
            .env("DESK_PM3_NODE", &node).env("DESK_PM3_HOME", &home).env("DESK_PM3_CONFIG", config)
            .env("PM3_CONFIG", unrelated.join("foreign.yaml"))
            .env("FIXTURE_RECORD", &record).env("FIXTURE_OUTPUT", output).env("FIXTURE_EXIT", exit)
            .env("LOCALAPPDATA", root).env("APPDATA", root).env("TEMP", root).env("TMP", root).output().unwrap()
    };
    for text in ["desk: running (pid 1234)", "desk: suspended", "desk: no-candidate"] {
        let out = call(text, "0", &config_file);
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(report["status"], "activation-command-completed");
        assert_eq!(report["output"], text); assert_eq!(report["readiness"], "not-checked");
        let evidence: serde_json::Value = serde_json::from_slice(&fs::read(&record).unwrap()).unwrap();
        assert_eq!(evidence["args"], serde_json::json!(["start", "moveup/desk"]));
        assert_eq!(evidence["config"].as_str(), config_file.to_str());
        assert_eq!(evidence["project"], "moveup");
        assert_eq!(std::path::PathBuf::from(evidence["cwd"].as_str().unwrap()), installed);
    }
    let out = call("desk: held", "7", &config_file);
    assert_eq!(out.status.code(), Some(1));
    let error = String::from_utf8_lossy(&out.stderr);
    assert!(error.contains("exit 7") && error.contains("unconfirmed") && error.contains("held"));
    fs::remove_file(&record).unwrap();
    for path in [root.join("missing.yaml"), std::path::PathBuf::from("relative.yaml")] {
        assert_eq!(call("must not run", "0", &path).status.code(), Some(1));
        assert!(!record.exists());
    }
    let mut config: serde_json::Value = serde_json::from_slice(&fs::read(&config_file).unwrap()).unwrap();
    config["name"] = "foreign".into(); fs::write(&config_file, serde_json::to_vec(&config).unwrap()).unwrap();
    assert_eq!(call("must not run", "0", &config_file).status.code(), Some(1));
    assert!(!record.exists());
    assert!(!unrelated.join("lifecycle.json").exists());
    assert!(!fs::read_dir(root).unwrap().any(|e| e.unwrap().file_name().to_string_lossy().starts_with("moveup-startup-")));
}
