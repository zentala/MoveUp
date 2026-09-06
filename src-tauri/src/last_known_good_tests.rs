//! Tests for [`crate::last_known_good`] (E014-T02).
//!
//! The two assertions the plan names: the marker is absent until a probe
//! succeeds, and a failed probe leaves the previous marker untouched. The rest
//! cover the shadow paths from E014's test strategy — no store, blank marker,
//! and a write that cannot happen — plus the seam with the real release store,
//! since the marker only earns its keep by protecting a build from pruning.

use std::fs;

use tempfile::TempDir;

use crate::last_known_good::{LastKnownGood, ProbeOutcome, MARKER_FILE};
use crate::release_store::ReleaseStore;

/// A release store in a temp dir with the given versions installed, and its marker.
fn store_with(versions: &[&str]) -> (TempDir, ReleaseStore, LastKnownGood) {
    let temp = TempDir::new().expect("temp dir");
    let store = ReleaseStore::under_app_data(temp.path());
    for version in versions {
        store.create_version_dir(version).expect("create version dir");
    }
    let marker = LastKnownGood::in_store(&store);
    (temp, store, marker)
}

#[test]
fn marker_is_absent_until_a_probe_succeeds() {
    let (_temp, _store, marker) = store_with(&["0.6.0"]);

    assert!(!marker.path().exists());
    assert_eq!(marker.read().expect("read"), None);

    let after = marker
        .record_probe("0.6.0", ProbeOutcome::Good)
        .expect("record probe");

    assert_eq!(after, Some("0.6.0".to_string()));
    assert!(marker.path().exists());
}

#[test]
fn failed_probe_leaves_the_previous_marker_untouched() {
    let (_temp, _store, marker) = store_with(&["0.6.0", "0.7.0"]);
    marker
        .record_probe("0.6.0", ProbeOutcome::Good)
        .expect("prove 0.6.0");

    let after = marker
        .record_probe("0.7.0", ProbeOutcome::Failed)
        .expect("record failed probe");

    assert_eq!(after, Some("0.6.0".to_string()));
    assert_eq!(marker.read().expect("read"), Some("0.6.0".to_string()));
}

#[test]
fn failed_probe_on_a_store_with_no_marker_writes_nothing() {
    let (_temp, _store, marker) = store_with(&["0.6.0"]);

    let after = marker
        .record_probe("0.6.0", ProbeOutcome::Failed)
        .expect("record failed probe");

    assert_eq!(after, None);
    assert!(!marker.path().exists(), "a failed probe must not create a marker");
}

#[test]
fn a_later_good_probe_promotes_over_the_previous_marker() {
    let (_temp, _store, marker) = store_with(&["0.6.0", "0.7.0"]);
    marker
        .record_probe("0.6.0", ProbeOutcome::Good)
        .expect("prove 0.6.0");

    let after = marker
        .record_probe("0.7.0", ProbeOutcome::Good)
        .expect("prove 0.7.0");

    assert_eq!(after, Some("0.7.0".to_string()));
}

#[test]
fn missing_store_reads_as_no_good_build() {
    let temp = TempDir::new().expect("temp dir");
    let store = ReleaseStore::under_app_data(temp.path());
    let marker = LastKnownGood::in_store(&store);

    assert!(!store.root().exists());
    assert_eq!(marker.read().expect("read"), None);
}

#[test]
fn blank_marker_reads_as_no_good_build() {
    let (_temp, _store, marker) = store_with(&["0.6.0"]);
    fs::create_dir_all(marker.path().parent().expect("parent")).expect("create root");
    fs::write(marker.path(), "   \n").expect("write blank marker");

    assert_eq!(marker.read().expect("read"), None);
}

#[test]
fn read_trims_the_trailing_newline_the_writer_adds() {
    let (_temp, _store, marker) = store_with(&["0.6.0"]);
    marker.mark("0.6.0").expect("mark");

    let raw = fs::read_to_string(marker.path()).expect("raw read");
    assert_eq!(raw, "0.6.0\n");
    assert_eq!(marker.read().expect("read"), Some("0.6.0".to_string()));
}

#[test]
fn marking_creates_the_store_root_when_it_does_not_exist_yet() {
    let temp = TempDir::new().expect("temp dir");
    let store = ReleaseStore::under_app_data(temp.path());
    let marker = LastKnownGood::in_store(&store);

    marker.mark("0.6.0").expect("mark into a store that has no root yet");

    assert!(store.root().is_dir());
    assert_eq!(marker.read().expect("read"), Some("0.6.0".to_string()));
}

#[test]
fn marking_reports_the_error_when_the_root_cannot_be_created() {
    let temp = TempDir::new().expect("temp dir");
    // A file where the releases directory should be: the write cannot happen.
    fs::write(temp.path().join("releases"), b"not a directory").expect("write blocker");
    let store = ReleaseStore::under_app_data(temp.path());
    let marker = LastKnownGood::in_store(&store);

    let err = marker.mark("0.6.0").expect_err("mark must fail, not panic");

    assert!(!err.to_string().is_empty());
    assert_eq!(marker.read().expect("read"), None);
}

#[test]
fn writing_leaves_no_temp_file_behind() {
    let (_temp, store, marker) = store_with(&["0.6.0"]);

    marker.mark("0.6.0").expect("mark");

    let leftovers: Vec<String> = fs::read_dir(store.root())
        .expect("read root")
        .map(|entry| entry.expect("entry").file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".tmp"))
        .collect();
    assert!(leftovers.is_empty(), "temp files left behind: {leftovers:?}");
}

#[test]
fn marker_sits_beside_the_version_directories_and_is_not_one() {
    let (_temp, store, marker) = store_with(&["0.6.0"]);

    marker.mark("0.6.0").expect("mark");

    assert_eq!(marker.path(), store.root().join(MARKER_FILE));
    assert_eq!(store.versions_oldest_first().expect("list"), vec!["0.6.0"]);
}

#[test]
fn the_marked_version_survives_pruning() {
    let (_temp, store, marker) = store_with(&["0.1.0", "0.2.0", "0.3.0", "0.4.0"]);
    marker
        .record_probe("0.1.0", ProbeOutcome::Good)
        .expect("prove the oldest build");

    let protected = marker.read().expect("read");
    let removed = store.prune(protected.as_deref()).expect("prune");

    assert_eq!(removed, vec!["0.2.0".to_string()]);
    assert!(store.contains("0.1.0"), "last-known-good must survive pruning");
}
