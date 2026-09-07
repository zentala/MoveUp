//! Unit tests for `commands_catalog`. Split out to respect the 250-line cap.
//! Pattern mirrors `commands_analyst_tests.rs`.

#![cfg(test)]

use crate::commands_catalog::{build_catalog, get_data_catalog};

/// Every source the catalog must describe. Grows with each new data source —
/// E021-T06 added `voice_notes`.
const EXPECTED_SOURCE_IDS: &[&str] = &[
    "sensor",
    "sqlite_sessions",
    "snapshots",
    "events_log",
    "profiles_ergonomic",
    "profiles_communication",
    "store",
    "remote_ws",
    "voice_notes",
];

#[test]
fn catalog_has_all_expected_sources() {
    let catalog = build_catalog();
    assert!(
        catalog.sources.len() >= EXPECTED_SOURCE_IDS.len(),
        "expected >= {} sources, got {}",
        EXPECTED_SOURCE_IDS.len(),
        catalog.sources.len()
    );
}

#[test]
fn every_source_has_id_and_fields() {
    let catalog = build_catalog();
    for src in &catalog.sources {
        assert!(!src.id.is_empty(), "source has empty id");
        assert!(!src.name.is_empty(), "source {} has empty name", src.id);
        assert!(!src.kind.is_empty(), "source {} has empty kind", src.id);
        assert!(!src.fields.is_empty(), "source {} has zero fields", src.id);
        for field in &src.fields {
            assert!(!field.name.is_empty(), "field in {} has empty name", src.id);
            assert!(
                !field.ty.is_empty(),
                "field {} in {} has empty type",
                field.name,
                src.id
            );
        }
    }
}

#[test]
fn source_ids_match_research_report() {
    let catalog = build_catalog();
    let ids: Vec<&str> = catalog.sources.iter().map(|s| s.id.as_str()).collect();
    for expected in EXPECTED_SOURCE_IDS {
        assert!(ids.contains(expected), "missing source id: {}", expected);
    }
}

#[test]
fn source_ids_are_unique() {
    let catalog = build_catalog();
    let mut ids: Vec<&str> = catalog.sources.iter().map(|s| s.id.as_str()).collect();
    ids.sort();
    let before = ids.len();
    ids.dedup();
    assert_eq!(before, ids.len(), "duplicate source ids detected");
}

#[test]
fn generated_at_is_rfc3339() {
    let catalog = build_catalog();
    let parsed = chrono::DateTime::parse_from_rfc3339(&catalog.generated_at);
    assert!(
        parsed.is_ok(),
        "generated_at is not RFC3339: {} ({:?})",
        catalog.generated_at,
        parsed.err()
    );
}

#[test]
fn command_returns_ok() {
    let result = get_data_catalog();
    assert!(result.is_ok());
    let catalog = result.expect("ok");
    assert_eq!(catalog.sources.len(), EXPECTED_SOURCE_IDS.len());
}

#[test]
fn kinds_are_from_allowed_set() {
    let allowed = [
        "sensor", "sqlite", "json", "log", "config", "in_memory", "websocket", "kv",
    ];
    let catalog = build_catalog();
    for src in &catalog.sources {
        assert!(
            allowed.contains(&src.kind.as_str()),
            "source {} has unexpected kind: {}",
            src.id,
            src.kind
        );
    }
}
