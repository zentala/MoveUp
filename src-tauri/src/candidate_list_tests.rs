//! Tests for [`crate::candidate_list`] (E014-T04).
//!
//! The two assertions the plan names: the order with three distinct candidates,
//! and the collapse when the newest build IS the last-known-good one. The rest
//! cover E014's shadow paths — nil (fresh install, one entry), empty (no builds,
//! empty list, no crash), and a write that cannot happen — plus the seam with
//! the real store and the real marker, since ordering is only correct if it
//! agrees with what those two actually report.

use std::fs;

use tempfile::TempDir;

use crate::candidate_list::{order_candidates, CandidateList, CANDIDATES_FILE};
use crate::last_known_good::{LastKnownGood, ProbeOutcome};
use crate::release_store::ReleaseStore;

/// A release store in a temp dir with the given versions installed, its marker,
/// and its candidate list.
fn store_with(versions: &[&str]) -> (TempDir, ReleaseStore, LastKnownGood, CandidateList) {
    let temp = TempDir::new().expect("temp dir");
    let store = ReleaseStore::under_app_data(temp.path());
    for version in versions {
        store.create_version_dir(version).expect("create version dir");
    }
    let marker = LastKnownGood::in_store(&store);
    let list = CandidateList::in_store(&store);
    (temp, store, marker, list)
}

fn owned(versions: &[&str]) -> Vec<String> {
    versions.iter().map(|v| v.to_string()).collect()
}

#[test]
fn three_distinct_candidates_order_newest_then_good_then_next_older() {
    let (_temp, store, marker, list) = store_with(&["0.4.0", "0.5.0", "0.6.0"]);
    marker
        .record_probe("0.4.0", ProbeOutcome::Good)
        .expect("prove the oldest build");

    let ordered = list.refresh(&store, &marker).expect("refresh");

    assert_eq!(ordered, owned(&["0.6.0", "0.4.0", "0.5.0"]));
    assert_eq!(list.read().expect("read"), Some(owned(&["0.6.0", "0.4.0", "0.5.0"])));
}

#[test]
fn newest_being_last_known_good_collapses_instead_of_duplicating() {
    let (_temp, store, marker, list) = store_with(&["0.4.0", "0.5.0", "0.6.0"]);
    marker
        .record_probe("0.6.0", ProbeOutcome::Good)
        .expect("prove the newest build");

    let ordered = list.refresh(&store, &marker).expect("refresh");

    assert_eq!(ordered, owned(&["0.6.0", "0.5.0", "0.4.0"]));
    assert_eq!(
        ordered.iter().filter(|v| *v == "0.6.0").count(),
        1,
        "the newest build must appear once, not twice"
    );
}

#[test]
fn fresh_install_lists_the_only_build_once() {
    let (_temp, store, marker, list) = store_with(&["0.6.0"]);

    let ordered = list.refresh(&store, &marker).expect("refresh");

    assert_eq!(ordered, owned(&["0.6.0"]));
    assert_eq!(marker.read().expect("read marker"), None);
}

#[test]
fn empty_store_writes_an_empty_list_and_does_not_crash() {
    let (_temp, store, marker, list) = store_with(&[]);

    let ordered = list.refresh(&store, &marker).expect("refresh");

    assert!(ordered.is_empty());
    assert!(list.path().exists(), "an empty store still writes the list");
    assert_eq!(
        list.read().expect("read"),
        Some(Vec::new()),
        "no builds must read as an empty list, not as an unwritten one"
    );
}

#[test]
fn an_unwritten_list_reads_as_none_never_as_an_empty_list() {
    let (_temp, _store, _marker, list) = store_with(&["0.6.0"]);

    assert!(!list.path().exists());
    assert_eq!(list.read().expect("read"), None);
}

#[test]
fn a_last_known_good_that_is_no_longer_installed_is_dropped() {
    let ordered = order_candidates(&owned(&["0.6.0", "0.5.0"]), Some("0.1.0"));

    assert_eq!(ordered, owned(&["0.6.0", "0.5.0"]));
}

#[test]
fn the_list_never_grows_past_the_retention_count() {
    let ordered = order_candidates(&owned(&["0.9.0", "0.8.0", "0.7.0", "0.6.0"]), Some("0.6.0"));

    assert_eq!(ordered, owned(&["0.9.0", "0.6.0", "0.8.0"]));
}

#[test]
fn no_marker_orders_purely_by_age() {
    let ordered = order_candidates(&owned(&["0.6.0", "0.5.0", "0.4.0"]), None);

    assert_eq!(ordered, owned(&["0.6.0", "0.5.0", "0.4.0"]));
}

#[test]
fn ordering_uses_version_numbers_not_directory_creation_order() {
    let (_temp, store, marker, list) = store_with(&["0.10.0", "0.9.0"]);

    let ordered = list.refresh(&store, &marker).expect("refresh");

    assert_eq!(ordered, owned(&["0.10.0", "0.9.0"]));
}

#[test]
fn refreshing_again_replaces_the_previous_list() {
    let (_temp, store, marker, list) = store_with(&["0.5.0", "0.6.0"]);
    list.refresh(&store, &marker).expect("first refresh");

    store.create_version_dir("0.7.0").expect("install a newer build");
    let ordered = list.refresh(&store, &marker).expect("second refresh");

    assert_eq!(ordered, owned(&["0.7.0", "0.6.0", "0.5.0"]));
    assert_eq!(list.read().expect("read"), Some(ordered));
}

#[test]
fn the_file_is_one_version_per_line_newline_terminated() {
    let (_temp, store, marker, list) = store_with(&["0.5.0", "0.6.0"]);

    list.refresh(&store, &marker).expect("refresh");

    let raw = fs::read_to_string(list.path()).expect("raw read");
    assert_eq!(raw, "0.6.0\n0.5.0\n");
}

#[test]
fn the_list_sits_beside_the_version_directories_and_is_not_one() {
    let (_temp, store, marker, list) = store_with(&["0.6.0"]);

    list.refresh(&store, &marker).expect("refresh");

    assert_eq!(list.path(), store.root().join(CANDIDATES_FILE));
    assert_eq!(store.versions_oldest_first().expect("list"), vec!["0.6.0"]);
}

#[test]
fn writing_leaves_no_temp_file_behind() {
    let (_temp, store, marker, list) = store_with(&["0.6.0"]);

    list.refresh(&store, &marker).expect("refresh");

    let leftovers: Vec<String> = fs::read_dir(store.root())
        .expect("read root")
        .map(|entry| entry.expect("entry").file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".tmp"))
        .collect();
    assert!(leftovers.is_empty(), "temp files left behind: {leftovers:?}");
}

#[test]
fn writing_reports_the_error_when_the_root_cannot_be_created() {
    let temp = TempDir::new().expect("temp dir");
    // A file where the releases directory should be: the write cannot happen.
    fs::write(temp.path().join("releases"), b"not a directory").expect("write blocker");
    let store = ReleaseStore::under_app_data(temp.path());
    let list = CandidateList::in_store(&store);

    let err = list
        .write(&owned(&["0.6.0"]))
        .expect_err("write must fail, not panic");

    assert!(!err.to_string().is_empty());
    assert_eq!(list.read().expect("read"), None);
}

#[test]
fn a_pruned_store_and_its_list_agree_on_which_builds_exist() {
    let (_temp, store, marker, list) = store_with(&["0.1.0", "0.2.0", "0.3.0", "0.4.0"]);
    marker
        .record_probe("0.1.0", ProbeOutcome::Good)
        .expect("prove the oldest build");

    let protected = marker.read().expect("read marker");
    store.prune(protected.as_deref()).expect("prune");
    let ordered = list.refresh(&store, &marker).expect("refresh");

    assert_eq!(ordered, owned(&["0.4.0", "0.1.0", "0.3.0"]));
    for version in &ordered {
        assert!(store.contains(version), "listed a build that is not installed: {version}");
    }
}
