use std::{
    fs,
    process::{Command, Stdio},
};

use super::store_persistence::atomic_write_with;

#[test]
fn replacement_error_keeps_last_good_json_and_cleans_temporary_file() {
    let temp = tempfile::tempdir().unwrap();
    let target = temp.path().join("settings.json");
    let original = br#"{"generation":"old","items":[1,2,3]}"#;
    fs::write(&target, original).unwrap();

    let error = atomic_write_with(
        &target,
        br#"{"generation":"new","items":[7,8,9]}"#,
        || Err("injected replacement failure".into()),
        || panic!("replacement hook must not run after failure"),
    )
    .expect_err("injected replacement failure must reach the caller");

    assert!(error.contains("injected replacement failure"));
    assert_eq!(fs::read(&target).unwrap(), original);
    assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 1, "failed temp file is cleaned up");
}

#[test]
fn abrupt_process_termination_leaves_old_or_new_complete_json() {
    const MODE: &str = "MOVEUP_ATOMIC_STORE_CHILD_MODE";
    const TARGET: &str = "MOVEUP_ATOMIC_STORE_CHILD_TARGET";
    if let (Ok(mode), Ok(target)) = (std::env::var(MODE), std::env::var(TARGET)) {
        let target = std::path::PathBuf::from(target);
        let before = || {
            if mode == "before-replace" {
                std::process::exit(61);
            }
            Ok(())
        };
        let after = || {
            if mode == "after-replace" {
                std::process::exit(62);
            }
        };
        let _ = atomic_write_with(
            &target,
            br#"{"generation":"new","items":[7,8,9]}"#,
            before,
            after,
        );
        panic!("the child must terminate at an injected replacement boundary");
    }

    let executable = std::env::current_exe().unwrap();
    for (mode, exit_code, generation) in [
        ("before-replace", 61, "old"),
        ("after-replace", 62, "new"),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("settings.json");
        fs::write(&target, br#"{"generation":"old","items":[1,2,3]}"#).unwrap();
        let child = Command::new(&executable)
            .arg("--exact")
            .arg("store_persistence_tests::abrupt_process_termination_leaves_old_or_new_complete_json")
            .env(MODE, mode)
            .env(TARGET, &target)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap();
        assert_eq!(child.code(), Some(exit_code), "child reached the {mode} crash point");

        let bytes = fs::read(&target).unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).expect("target remains complete JSON");
        assert_eq!(json["generation"], generation);
        assert_eq!(json["items"].as_array().unwrap().len(), 3);
    }
}
