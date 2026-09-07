//! Tests for the health inlet: the [`HealthSource`] trait contract and the
//! [`HealthAggregator`] merge rules (E021-T02).
//!
//! Four shadow paths per `rules/testing.md`:
//!   - **happy** — one configured source with steps and heart rate;
//!   - **nil** — no sources registered at all;
//!   - **empty** — a configured source that has not fetched anything yet
//!     (`configured = true, snapshot = None`), which must not read as "no
//!     source configured";
//!   - **error** — a failing source, alone and alongside a healthy one.
//!
//! The aggregator is exercised through the real trait, never a mocked
//! aggregator: a fake source stands in for the network boundary only.

#![cfg(test)]

use crate::health_models::{HealthErrorKind, HealthSnapshot, HealthView};
use crate::health_source::{HealthAggregator, HealthSource};
use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// A source with a fixed answer — stands in for a network boundary.
struct FakeSource {
    id: String,
    view: HealthView,
    refreshes: AtomicUsize,
}

impl FakeSource {
    fn new(id: &str, view: HealthView) -> Arc<Self> {
        Arc::new(Self {
            id: id.to_string(),
            view,
            refreshes: AtomicUsize::new(0),
        })
    }
}

#[async_trait]
impl HealthSource for FakeSource {
    fn id(&self) -> &str {
        &self.id
    }
    async fn view(&self) -> HealthView {
        self.view.clone()
    }
    async fn refresh(&self) -> HealthView {
        self.refreshes.fetch_add(1, Ordering::SeqCst);
        self.view.clone()
    }
}

fn snapshot(source_id: &str, steps: i64, hr: Option<u16>, at_ms: i64) -> HealthSnapshot {
    HealthSnapshot {
        steps_today: steps,
        heart_rate_bpm: hr,
        hrv_rmssd_ms: None,
        source_id: source_id.to_string(),
        fetched_at_ms: at_ms,
    }
}

fn ok_view(source_id: &str, steps: i64, hr: Option<u16>, at_ms: i64) -> HealthView {
    HealthView {
        configured: true,
        snapshot: Some(snapshot(source_id, steps, hr, at_ms)),
        error_kind: None,
        error_message: None,
    }
}

fn err_view(kind: HealthErrorKind, msg: &str) -> HealthView {
    HealthView {
        configured: true,
        snapshot: None,
        error_kind: Some(kind),
        error_message: Some(msg.to_string()),
    }
}

// ---------------------------------------------------------------- happy

#[tokio::test]
async fn single_source_passes_steps_and_heart_rate_through() {
    let agg = HealthAggregator::new(vec![FakeSource::new(
        "google_fit",
        ok_view("google_fit", 4321, Some(72), 1_000),
    )]);
    let v = agg.view().await;
    assert!(v.configured);
    let snap = v.snapshot.expect("snapshot");
    assert_eq!(snap.steps_today, 4321);
    assert_eq!(snap.heart_rate_bpm, Some(72));
    assert_eq!(snap.source_id, "google_fit");
    assert!(v.error_kind.is_none());
}

#[tokio::test]
async fn freshest_snapshot_wins_regardless_of_registration_order() {
    let agg = HealthAggregator::new(vec![
        FakeSource::new("phone", ok_view("phone", 9000, Some(60), 5_000)),
        FakeSource::new("google_fit", ok_view("google_fit", 100, None, 1_000)),
    ]);
    let snap = agg.view().await.snapshot.expect("snapshot");
    assert_eq!(snap.source_id, "phone", "5_000 is fresher than 1_000");
    assert_eq!(snap.steps_today, 9000);
}

#[tokio::test]
async fn refresh_hits_every_registered_source() {
    let a = FakeSource::new("a", ok_view("a", 1, None, 10));
    let b = FakeSource::new("b", ok_view("b", 2, None, 20));
    let agg = HealthAggregator::new(vec![a.clone(), b.clone()]);
    let v = agg.refresh().await;
    assert_eq!(a.refreshes.load(Ordering::SeqCst), 1);
    assert_eq!(b.refreshes.load(Ordering::SeqCst), 1);
    assert_eq!(v.snapshot.expect("snapshot").source_id, "b");
}

#[tokio::test]
async fn register_adds_a_source_after_construction() {
    let agg = HealthAggregator::default();
    assert!(agg.source_ids().await.is_empty());
    agg.register(FakeSource::new("push", ok_view("push", 7, None, 3)))
        .await;
    assert_eq!(agg.source_ids().await, vec!["push".to_string()]);
    assert_eq!(agg.view().await.snapshot.expect("snapshot").steps_today, 7);
}

// ------------------------------------------------------------------ nil

#[tokio::test]
async fn no_sources_reports_unconfigured_not_an_error() {
    let agg = HealthAggregator::new(vec![]);
    let v = agg.view().await;
    assert!(!v.configured);
    assert!(v.snapshot.is_none());
    assert!(v.error_kind.is_none(), "absence of a source is not a failure");
}

#[tokio::test]
async fn unconfigured_source_does_not_make_the_view_configured() {
    let agg = HealthAggregator::new(vec![FakeSource::new(
        "google_fit",
        HealthView::unconfigured(),
    )]);
    assert!(!agg.view().await.configured);
}

// ---------------------------------------------------------------- empty

#[tokio::test]
async fn configured_source_without_a_snapshot_stays_configured() {
    // "Loading" must be distinguishable from "not set up" — the two lead
    // the UI to opposite states (spinner vs setup hint).
    let agg = HealthAggregator::new(vec![FakeSource::new(
        "google_fit",
        HealthView {
            configured: true,
            snapshot: None,
            error_kind: None,
            error_message: None,
        },
    )]);
    let v = agg.view().await;
    assert!(v.configured);
    assert!(v.snapshot.is_none());
    assert!(v.error_kind.is_none());
}

#[tokio::test]
async fn zero_steps_is_a_reading_not_a_missing_snapshot() {
    let agg = HealthAggregator::new(vec![FakeSource::new(
        "google_fit",
        ok_view("google_fit", 0, None, 1),
    )]);
    let snap = agg.view().await.snapshot.expect("zero steps is still a snapshot");
    assert_eq!(snap.steps_today, 0);
    assert_eq!(snap.heart_rate_bpm, None, "no sensor must not read as 0 bpm");
}

// ---------------------------------------------------------------- error

#[tokio::test]
async fn single_failing_source_passes_its_error_through() {
    let agg = HealthAggregator::new(vec![FakeSource::new(
        "google_fit",
        err_view(HealthErrorKind::AuthRevoked, "google fit: refresh token revoked"),
    )]);
    let v = agg.view().await;
    assert!(v.configured);
    assert_eq!(v.error_kind, Some(HealthErrorKind::AuthRevoked));
    assert_eq!(
        v.error_message.as_deref(),
        Some("google fit: refresh token revoked")
    );
}

#[tokio::test]
async fn healthy_source_never_hides_another_source_error() {
    let agg = HealthAggregator::new(vec![
        FakeSource::new("phone", ok_view("phone", 8000, Some(65), 9_000)),
        FakeSource::new(
            "google_fit",
            err_view(HealthErrorKind::AuthRevoked, "google fit: refresh token revoked"),
        ),
    ]);
    let v = agg.view().await;
    let snap = v.snapshot.expect("the healthy source still shows");
    assert_eq!(snap.source_id, "phone");
    assert_eq!(
        v.error_kind,
        Some(HealthErrorKind::AuthRevoked),
        "a working source must not mask broken credentials elsewhere"
    );
}

#[tokio::test]
async fn auth_revoked_outranks_transient() {
    let agg = HealthAggregator::new(vec![
        FakeSource::new("a", err_view(HealthErrorKind::Transient, "network blip")),
        FakeSource::new("b", err_view(HealthErrorKind::AuthRevoked, "revoked")),
    ]);
    let v = agg.view().await;
    assert_eq!(v.error_kind, Some(HealthErrorKind::AuthRevoked));
    assert_eq!(v.error_message.as_deref(), Some("revoked"));
}

#[tokio::test]
async fn merge_of_no_views_is_the_unconfigured_view() {
    assert_eq!(HealthAggregator::merge(vec![]), HealthView::unconfigured());
}
