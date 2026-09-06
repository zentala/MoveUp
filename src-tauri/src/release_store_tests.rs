//! Tests for [`crate::release_store`] (E014-T01).
//!
//! The two assertions the plan names: a fourth install prunes the oldest of
//! three retained releases, and pruning never removes the build marked
//! `last-known-good`. The rest cover the shadow paths from E014's test
//! strategy — no store, empty store, and a store holding non-release files.

use std::fs;
use std::path::Path;

use tempfile::TempDir;

use crate::release_store::{ReleaseStore, RELEASES_DIR, RETAIN_COUNT};

/// Builds a store under a temp dir with the given versions already installed.
fn store_with(versions: &[&str]) -> (TempDir, ReleaseStore) {
    let temp = TempDir::new().expect("temp dir");
    let store = ReleaseStore::under_app_data(temp.path());
    for version in versions {
        store.create_version_dir(version).expect("create version dir");
    }
    (temp, store)
}

fn write_marker(root: &Path, name: &str) {
    fs::create_dir_all(root).expect("create root");
    fs::write(root.join(name), b"0.1.0").expect("write marker");
}

#[test]
fn fourth_install_prunes_the_oldest_release() {
    let (_temp, store) = store_with(&["0.1.0", "0.2.0", "0.3.0"]);

    store.create_version_dir("0.4.0").expect("install fourth");
    let removed = store.prune(None).expect("prune");

    assert_eq!(removed, vec!["0.1.0".to_string()]);
    assert_eq!(
        store.versions_oldest_first().expect("list"),
        vec!["0.2.0", "0.3.0", "0.4.0"]
    );
    assert!(!store.contains("0.1.0"));
}

#[test]
fn pruning_never_removes_the_last_known_good_release() {
    let (_temp, store) = store_with(&["0.1.0", "0.2.0", "0.3.0", "0.4.0"]);

    let removed = store.prune(Some("0.1.0")).expect("prune");

    assert_eq!(removed, vec!["0.2.0".to_string()]);
    assert!(store.contains("0.1.0"), "protected release must survive");
    assert_eq!(
        store.versions_oldest_first().expect("list"),
        vec!["0.1.0", "0.3.0", "0.4.0"]
    );
}

#[test]
fn protected_release_survives_even_when_far_over_retention() {
    let (_temp, store) = store_with(&["0.1.0", "0.2.0", "0.3.0", "0.4.0", "0.5.0", "0.6.0"]);

    let removed = store.prune(Some("0.1.0")).expect("prune");

    assert_eq!(removed, vec!["0.2.0", "0.3.0", "0.4.0"]);
    assert_eq!(
        store.versions_oldest_first().expect("list"),
        vec!["0.1.0", "0.5.0", "0.6.0"]
    );
}

#[test]
fn prune_is_a_no_op_at_or_below_the_retention_count() {
    let (_temp, store) = store_with(&["0.1.0", "0.2.0", "0.3.0"]);

    assert!(store.prune(None).expect("prune").is_empty());
    assert_eq!(
        store.versions_oldest_first().expect("list").len(),
        RETAIN_COUNT
    );
}

#[test]
fn missing_store_lists_no_releases_instead_of_failing() {
    let temp = TempDir::new().expect("temp dir");
    let store = ReleaseStore::under_app_data(temp.path());

    assert!(!store.root().exists());
    assert!(store.versions_oldest_first().expect("list").is_empty());
    assert_eq!(store.newest().expect("newest"), None);
    assert!(store.prune(None).expect("prune").is_empty());
}

#[test]
fn empty_store_lists_no_releases() {
    let temp = TempDir::new().expect("temp dir");
    let store = ReleaseStore::under_app_data(temp.path());
    fs::create_dir_all(store.root()).expect("create root");

    assert!(store.versions_oldest_first().expect("list").is_empty());
    assert_eq!(store.newest().expect("newest"), None);
}

#[test]
fn state_files_in_the_root_are_not_releases() {
    let (_temp, store) = store_with(&["0.1.0"]);
    write_marker(store.root(), "last-known-good");
    write_marker(store.root(), "candidates.json");

    assert_eq!(store.versions_oldest_first().expect("list"), vec!["0.1.0"]);
}

#[test]
fn versions_order_numerically_not_lexicographically() {
    let (_temp, store) = store_with(&["0.9.0", "0.10.0", "0.10.1"]);

    assert_eq!(
        store.versions_oldest_first().expect("list"),
        vec!["0.9.0", "0.10.0", "0.10.1"]
    );
    assert_eq!(store.newest().expect("newest"), Some("0.10.1".to_string()));
}

#[test]
fn versions_newest_first_reverses_the_age_order() {
    let (_temp, store) = store_with(&["0.1.0", "0.2.0", "0.3.0"]);

    assert_eq!(
        store.versions_newest_first().expect("list"),
        vec!["0.3.0", "0.2.0", "0.1.0"]
    );
}

#[test]
fn unparsable_version_names_sort_oldest_and_are_pruned_first() {
    let (_temp, store) = store_with(&["nightly", "0.1.0", "0.2.0", "0.3.0"]);

    let removed = store.prune(None).expect("prune");

    assert_eq!(removed, vec!["nightly".to_string()]);
    assert_eq!(
        store.versions_oldest_first().expect("list"),
        vec!["0.1.0", "0.2.0", "0.3.0"]
    );
}

#[test]
fn creating_an_existing_version_dir_keeps_its_contents() {
    let (_temp, store) = store_with(&["0.1.0"]);
    let payload = store.version_dir("0.1.0").join("desk.exe");
    fs::write(&payload, b"binary").expect("write payload");

    let dir = store.create_version_dir("0.1.0").expect("re-create");

    assert_eq!(dir, store.version_dir("0.1.0"));
    assert!(payload.exists(), "re-install must not wipe the version dir");
}

#[test]
fn store_roots_at_the_releases_directory_under_app_data() {
    let temp = TempDir::new().expect("temp dir");
    let store = ReleaseStore::under_app_data(temp.path());

    assert_eq!(store.root(), temp.path().join(RELEASES_DIR));
    assert_eq!(
        store.version_dir("0.6.0"),
        temp.path().join(RELEASES_DIR).join("0.6.0")
    );
}
