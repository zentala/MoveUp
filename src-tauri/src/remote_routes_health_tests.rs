//! Tests for the LAN health push inlet (E021-T03).
//!
//! These drive the **real seam**: a request goes through the real router,
//! the real auth guard, the real [`PushHealthSource`], and the real
//! [`HealthAggregator`], and the assertion is made on what the aggregator
//! merges out the other end. Nothing between the route and the aggregator is
//! stubbed — a mismatch between what the route writes and what the aggregator
//! reads is exactly the failure a pair of separately-mocked tests cannot see.

use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use crate::communication_policy::CommunicationPolicy;
use crate::db::TodaySummary;
use crate::health_models::HealthView;
use crate::health_source::{HealthAggregator, HealthSource, HealthState};
use crate::remote_routes_health::{
    HealthPushBody, PushHealthSource, MAX_BODY_BYTES, PUSH_SOURCE_ID, STALE_AFTER_MS,
};
use crate::remote_server::{build_router, RemoteState};
use crate::session::SessionManager;

const TOKEN: &str = "test-token";

fn today_summary() -> TodaySummary {
    TodaySummary {
        sitting_secs: 0,
        standing_secs: 0,
        yesterday_sitting_secs: 0,
        yesterday_standing_secs: 0,
        position_changes: 0,
        sessions: vec![],
    }
}

/// Builds a [`RemoteState`] with the push inlet really registered.
///
/// Shared with `remote_display_state`'s parity test so both exercise the same
/// wiring the app ships.
pub async fn remote_state(
    session: Arc<Mutex<SessionManager>>,
    comm_policy: Arc<Mutex<CommunicationPolicy>>,
    today_cache: Arc<Mutex<TodaySummary>>,
) -> RemoteState {
    let health: HealthState = Arc::new(HealthAggregator::new(vec![]));
    let health_push = crate::remote_routes_health::register(&health).await;
    RemoteState {
        ws_tx: crate::ws_broadcaster::create_channel(),
        session,
        comm_policy,
        today_cache,
        active_clients: Arc::new(AtomicUsize::new(0)),
        health,
        health_push,
        remote_token: Some(TOKEN.to_string()),
        // The voice inlet's own wiring is exercised in
        // `remote_routes_voice_tests`; here it stays at its degraded default so
        // these tests keep asserting only the health path.
        voice: crate::remote_server::VoiceState::default(),
    }
}

async fn state_with_token(token: Option<&str>) -> RemoteState {
    let mut state = remote_state(
        Arc::new(Mutex::new(SessionManager::new())),
        Arc::new(Mutex::new(CommunicationPolicy::new(
            Default::default(),
            Default::default(),
        ))),
        Arc::new(Mutex::new(today_summary())),
    )
    .await;
    state.remote_token = token.map(str::to_string);
    state
}

fn push_request(token: Option<&str>, body: &str) -> Request<Body> {
    let mut builder = Request::builder()
        .method("POST")
        .uri("/display/health")
        .header("content-type", "application/json");
    if let Some(token) = token {
        builder = builder.header(crate::remote_auth::TOKEN_HEADER, token);
    }
    builder.body(Body::from(body.to_string())).expect("request")
}

async fn post(state: &RemoteState, token: Option<&str>, body: &str) -> StatusCode {
    build_router(state.clone())
        .oneshot(push_request(token, body))
        .await
        .expect("router responds")
        .status()
}

fn body_json(steps: u32, source: &str, measured_at_ms: i64) -> String {
    format!(
        r#"{{"steps_today":{steps},"heart_rate_bpm":61,"source_id":"{source}","measured_at_ms":{measured_at_ms}}}"#
    )
}

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

#[tokio::test]
async fn e021_t03_authenticated_push_reaches_the_aggregator() {
    let state = state_with_token(Some(TOKEN)).await;

    let status = post(&state, Some(TOKEN), &body_json(1234, "curl", now_ms())).await;
    assert_eq!(status, StatusCode::OK);

    // The assertion that matters: read back through the aggregator, not
    // through the push source the route just wrote to.
    let merged = state.health.view().await;
    assert!(merged.configured);
    let snapshot = merged.snapshot.expect("the pushed reading must be merged");
    assert_eq!(snapshot.steps_today, 1234);
    assert_eq!(snapshot.source_id, "curl");
    assert_eq!(snapshot.heart_rate_bpm, Some(61));
}

#[tokio::test]
async fn e021_t03_push_shows_up_in_the_display_api_payload() {
    let state = state_with_token(Some(TOKEN)).await;
    post(&state, Some(TOKEN), &body_json(777, "phone", now_ms())).await;

    let response = build_router(state.clone())
        .oneshot(
            Request::builder()
                .uri("/display/api")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("router responds");
    assert_eq!(response.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(response.into_body(), 1 << 20)
        .await
        .expect("body");
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(json["health"]["snapshot"]["steps_today"], 777);
    assert_eq!(json["health"]["snapshot"]["source_id"], "phone");
}

#[tokio::test]
async fn e021_t03_push_updates_the_cached_view_the_tray_tick_reads() {
    let state = state_with_token(Some(TOKEN)).await;
    assert_eq!(state.health.last_view(), HealthView::unconfigured());

    post(&state, Some(TOKEN), &body_json(99, "phone", now_ms())).await;

    let cached = state.health.last_view();
    assert_eq!(
        cached.snapshot.expect("cache must hold the push").steps_today,
        99,
        "the ~1s broadcast reads last_view(); a push that does not land there never reaches the phone"
    );
}

#[tokio::test]
async fn e021_t03_push_without_a_token_is_rejected_and_writes_nothing() {
    let state = state_with_token(Some(TOKEN)).await;

    let status = post(&state, None, &body_json(1234, "curl", now_ms())).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(state.health.view().await, HealthView::unconfigured());

    let status = post(&state, Some("wrong"), &body_json(1234, "curl", now_ms())).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(state.health.view().await, HealthView::unconfigured());
}

#[tokio::test]
async fn e021_t03_inlet_is_closed_when_no_token_is_configured() {
    let state = state_with_token(None).await;
    let status = post(&state, Some(TOKEN), &body_json(1234, "curl", now_ms())).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(state.health.view().await, HealthView::unconfigured());
}

#[tokio::test]
async fn e021_t03_oversized_body_is_refused() {
    let state = state_with_token(Some(TOKEN)).await;
    let padded = format!(
        r#"{{"steps_today":1,"source_id":"{}","measured_at_ms":1}}"#,
        "x".repeat(MAX_BODY_BYTES)
    );
    assert!(padded.len() > MAX_BODY_BYTES);

    let status = post(&state, Some(TOKEN), &padded).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(state.health.view().await, HealthView::unconfigured());
}

#[tokio::test]
async fn e021_t03_malformed_and_invalid_bodies_are_distinguished() {
    let state = state_with_token(Some(TOKEN)).await;

    // Not JSON at all, and JSON missing a required field: both malformed.
    assert_eq!(
        post(&state, Some(TOKEN), "not json").await,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        post(&state, Some(TOKEN), r#"{"steps_today":5}"#).await,
        StatusCode::BAD_REQUEST
    );

    // Well-formed JSON, unusable values: unprocessable.
    assert_eq!(
        post(&state, Some(TOKEN), &body_json(5, "", now_ms())).await,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        post(&state, Some(TOKEN), &body_json(5, "phone", 0)).await,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(state.health.view().await, HealthView::unconfigured());
}

#[tokio::test]
async fn e021_t03_source_id_longer_than_64_chars_is_rejected() {
    let state = state_with_token(Some(TOKEN)).await;
    let long = "s".repeat(65);
    assert_eq!(
        post(&state, Some(TOKEN), &body_json(5, &long, now_ms())).await,
        StatusCode::UNPROCESSABLE_ENTITY
    );

    let ok = "s".repeat(64);
    assert_eq!(
        post(&state, Some(TOKEN), &body_json(5, &ok, now_ms())).await,
        StatusCode::OK
    );
}

#[tokio::test]
async fn e021_t03_reading_goes_stale_after_an_hour() {
    let source = PushHealthSource::new();
    let pushed_at = 1_700_000_000_000;
    source
        .push(
            HealthPushBody {
                steps_today: 4321,
                heart_rate_bpm: None,
                hrv_rmssd_ms: None,
                source_id: "phone".into(),
                measured_at_ms: pushed_at,
            }
            .into_snapshot()
            .expect("valid body"),
        )
        .await;

    let fresh = source.view_at(pushed_at + STALE_AFTER_MS).await;
    assert_eq!(
        fresh.snapshot.expect("still fresh at the boundary").steps_today,
        4321
    );

    let stale = source.view_at(pushed_at + STALE_AFTER_MS + 1).await;
    assert!(
        stale.configured,
        "a stale source is still configured — it just has nothing current to say"
    );
    assert!(
        stale.snapshot.is_none(),
        "an hour-old reading must not keep being reported as today's"
    );
}

#[tokio::test]
async fn e021_t03_empty_inlet_reports_unconfigured_not_zero_steps() {
    let source = PushHealthSource::new();
    assert_eq!(source.view_at(now_ms()).await, HealthView::unconfigured());
    assert_eq!(source.id(), PUSH_SOURCE_ID);
}

#[tokio::test]
async fn e021_t03_registered_inlet_is_visible_to_the_aggregator() {
    let health: HealthState = Arc::new(HealthAggregator::new(vec![]));
    crate::remote_routes_health::register(&health).await;
    assert_eq!(health.source_ids().await, vec![PUSH_SOURCE_ID.to_string()]);
}

#[test]
fn e021_t03_non_finite_or_negative_hrv_is_rejected() {
    let body = |hrv: f32| HealthPushBody {
        steps_today: 1,
        heart_rate_bpm: None,
        hrv_rmssd_ms: Some(hrv),
        source_id: "phone".into(),
        measured_at_ms: 1,
    };
    assert!(body(f32::NAN).into_snapshot().is_none());
    assert!(body(-1.0).into_snapshot().is_none());
    assert!(body(42.5).into_snapshot().is_some());
}
