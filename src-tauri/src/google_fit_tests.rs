//! Unit tests for the Google Fit modules.
//!
//! Pure, network-free assertions: credential parsing from the environment,
//! data-source ranking, HTTP error classification, the unconfigured-service
//! contract, and the DST-correct local day window. The wiremock-backed
//! request/response tests live in `google_fit_http_tests.rs`; the
//! service-level integration tests in `google_fit_service_tests.rs`.

#![cfg(test)]

use crate::google_fit::{classify_response, Credentials, ErrorKind, GoogleFitClient};
use crate::google_fit_service::{local_day_window_ms, GoogleFitService};
use crate::health_source::HealthSource;

/// Serialises tests that mutate the shared process environment.
///
/// `std::env` is global to the test binary, so these tests overwrite each
/// other's setup when the harness runs them on parallel threads.
static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Acquires the environment lock, ignoring poisoning left by an unrelated failure.
fn env_guard() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn clear_env() {
    std::env::remove_var("GOOGLE_CLIENT_ID");
    std::env::remove_var("GOOGLE_CLIENT_SECRET");
    std::env::remove_var("GOOGLE_REFRESH_TOKEN");
    std::env::remove_var("GOOGLE_FIT_STEPS_SOURCE");
}

#[test]
fn credentials_from_env_returns_none_when_missing() {
    let _env = env_guard();
    clear_env();
    assert!(Credentials::from_env().is_none());
}

#[test]
fn credentials_from_env_rejects_empty_values() {
    let _env = env_guard();
    clear_env();
    std::env::set_var("GOOGLE_CLIENT_ID", "");
    std::env::set_var("GOOGLE_CLIENT_SECRET", "x");
    std::env::set_var("GOOGLE_REFRESH_TOKEN", "y");
    assert!(Credentials::from_env().is_none());
    clear_env();
}

#[test]
fn credentials_picks_up_optional_source_override() {
    let _env = env_guard();
    clear_env();
    std::env::set_var("GOOGLE_CLIENT_ID", "id");
    std::env::set_var("GOOGLE_CLIENT_SECRET", "secret");
    std::env::set_var("GOOGLE_REFRESH_TOKEN", "tok");
    std::env::set_var("GOOGLE_FIT_STEPS_SOURCE", "derived:custom");
    let c = Credentials::from_env().expect("credentials should parse");
    assert_eq!(c.steps_source_override.as_deref(), Some("derived:custom"));
    clear_env();
}

#[test]
fn rank_sources_prefers_merge_step_deltas() {
    let v = vec![
        "raw:com.google.step_count.delta:samsung:SM-N9005:something".to_string(),
        "derived:com.google.step_count.delta:com.google.android.gms:estimated_steps".to_string(),
        "derived:com.google.step_count.delta:com.google.android.gms:merge_step_deltas".to_string(),
    ];
    assert_eq!(
        GoogleFitClient::rank_sources(&v).as_deref(),
        Some("derived:com.google.step_count.delta:com.google.android.gms:merge_step_deltas"),
    );
}

#[test]
fn rank_sources_falls_back_to_first_raw_when_no_derived() {
    let v = vec!["raw:com.google.step_count.delta:samsung:SM-N9005:x".to_string()];
    assert_eq!(GoogleFitClient::rank_sources(&v).as_deref(), Some(v[0].as_str()));
}

#[test]
fn rank_sources_none_when_empty() {
    assert!(GoogleFitClient::rank_sources(&[]).is_none());
}

#[test]
fn classify_response_treats_401_as_auth_revoked() {
    let status = reqwest::StatusCode::UNAUTHORIZED;
    assert_eq!(classify_response(status, ""), ErrorKind::AuthRevoked);
    // Even with empty / non-JSON body, 401 alone qualifies.
    assert_eq!(classify_response(status, "garbage"), ErrorKind::AuthRevoked);
}

#[test]
fn classify_response_treats_invalid_grant_json_as_auth_revoked() {
    // RFC 6749 §5.2 shape — Google returns this with HTTP 400 on revoked
    // refresh tokens.
    let status = reqwest::StatusCode::BAD_REQUEST;
    let body = r#"{"error":"invalid_grant","error_description":"Token has been expired or revoked."}"#;
    assert_eq!(classify_response(status, body), ErrorKind::AuthRevoked);
}

#[test]
fn classify_response_treats_other_5xx_as_transient() {
    let status = reqwest::StatusCode::SERVICE_UNAVAILABLE;
    assert_eq!(classify_response(status, "upstream timeout"), ErrorKind::Transient);
}

#[test]
fn classify_response_does_not_match_invalid_grant_substring_in_garbage() {
    // The old substring-based check would have false-positived on a
    // body that merely contained the word "invalid_grant" in prose.
    // The JSON-shape check rejects it.
    let status = reqwest::StatusCode::BAD_REQUEST;
    let body = "We saw invalid_grant somewhere in this prose, but it is not JSON.";
    assert_eq!(classify_response(status, body), ErrorKind::Transient);
}

#[test]
fn fit_error_serializes_kind_as_snake_case() {
    // Ensure the wire format the frontend reads is stable.
    assert_eq!(
        serde_json::to_string(&ErrorKind::AuthRevoked).unwrap(),
        "\"auth_revoked\"",
    );
    assert_eq!(
        serde_json::to_string(&ErrorKind::Transient).unwrap(),
        "\"transient\"",
    );
}

#[tokio::test]
async fn unconfigured_view_reports_not_configured() {
    clear_env();
    let svc = GoogleFitService::from_env();
    let v = svc.view().await;
    assert!(!v.configured);
    assert!(v.snapshot.is_none());
    assert!(v.error_kind.is_none());
}

#[tokio::test]
async fn unconfigured_refresh_does_not_error() {
    // The new contract: refresh always returns a view; unconfigured
    // is a state, not an error.
    clear_env();
    let svc = GoogleFitService::from_env();
    let v = svc.refresh().await;
    assert!(!v.configured);
    assert!(v.snapshot.is_none());
}

#[test]
fn local_day_window_spans_24h_on_normal_day() {
    use chrono::TimeZone;
    // Pin to Europe/Warsaw via chrono-tz so the assertion is host-TZ
    // independent (UTC CI would otherwise pass any test trivially).
    let warsaw = chrono_tz::Europe::Warsaw;
    let now = warsaw.with_ymd_and_hms(2026, 7, 15, 14, 30, 0).single().unwrap();
    let (start, end) = local_day_window_ms(now);
    assert_eq!(
        end - start,
        86_400_000,
        "non-DST CEST summer day must span exactly 24h"
    );
}

#[test]
fn local_day_window_is_exactly_23h_on_dst_spring_forward_in_warsaw() {
    use chrono::TimeZone;
    let warsaw = chrono_tz::Europe::Warsaw;
    // Poland 2026 spring-forward: 2026-03-29 02:00 CET → 03:00 CEST.
    let now = warsaw.with_ymd_and_hms(2026, 3, 29, 12, 0, 0).single().unwrap();
    let (start, end) = local_day_window_ms(now);
    assert_eq!(
        (end - start) / 3_600_000,
        23,
        "spring-forward day in Warsaw must be exactly 23h"
    );
}

#[test]
fn local_day_window_is_exactly_25h_on_dst_fall_back_in_warsaw() {
    use chrono::TimeZone;
    let warsaw = chrono_tz::Europe::Warsaw;
    // Poland 2026 fall-back: 2026-10-25 03:00 CEST → 02:00 CET.
    let now = warsaw.with_ymd_and_hms(2026, 10, 25, 12, 0, 0).single().unwrap();
    let (start, end) = local_day_window_ms(now);
    assert_eq!(
        (end - start) / 3_600_000,
        25,
        "fall-back day in Warsaw must be exactly 25h"
    );
}
