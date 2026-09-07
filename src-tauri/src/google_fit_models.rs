//! Wire types for the Google Fit integration.
//!
//! Two categories live here:
//! 1. **OAuth wire types** — what the Google token endpoint returns.
//! 2. **Fit API wire types** — the minimal subset of the aggregate
//!    response, for steps (`intVal`) and heart rate (`fpVal`).
//!
//! The UI-facing snapshot and view types are source-agnostic and live in
//! [`crate::health_models`].

use serde::{Deserialize, Serialize};

/// Classification of API failures the frontend cares about.
///
/// Lives here (with the other wire types) rather than in `google_fit.rs`
/// because it is serialized as part of `HealthView` — keeping it next to
/// the type it embeds avoids the otherwise circular `google_fit.rs`
/// ↔ `google_fit_models.rs` import. Re-exported as
/// [`HealthErrorKind`](crate::health_models::HealthErrorKind).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export_to = "../../src/generated/"))]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    /// Token revoked, expired beyond recovery, or scope missing. User
    /// must re-run the OAuth helper script.
    AuthRevoked,
    /// Network failure, rate limit, server 5xx, parse error — retryable.
    Transient,
}

/// Response body of `POST https://oauth2.googleapis.com/token`.
///
/// The `refresh_token` grant only returns a new `access_token`; the original
/// long-lived refresh token continues to be valid.
#[derive(Debug, Deserialize)]
#[allow(dead_code)] // `expires_in` and `token_type` kept for diagnostics / future caching
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub expires_in: u64,
    #[serde(default)]
    pub token_type: String,
}

/// Top-level Fit aggregate response (`POST /fitness/v1/users/me/dataset:aggregate`).
#[derive(Debug, Deserialize)]
pub struct AggregateResponse {
    #[serde(default)]
    pub bucket: Vec<AggregateBucket>,
}

#[derive(Debug, Deserialize)]
pub struct AggregateBucket {
    #[serde(default)]
    pub dataset: Vec<AggregateDataset>,
}

#[derive(Debug, Deserialize)]
pub struct AggregateDataset {
    #[serde(default)]
    pub point: Vec<AggregatePoint>,
}

#[derive(Debug, Deserialize)]
pub struct AggregatePoint {
    #[serde(default)]
    pub value: Vec<AggregateValue>,
}

/// A single value entry. Step counts populate `int_val`; heart rate is a
/// floating-point average and populates `fp_val`.
#[derive(Debug, Deserialize)]
pub struct AggregateValue {
    #[serde(default, rename = "intVal")]
    pub int_val: Option<i64>,
    #[serde(default, rename = "fpVal")]
    pub fp_val: Option<f64>,
}

impl AggregateResponse {
    /// Collapse the nested aggregate response into a single step count.
    ///
    /// Returns 0 when the response contains no buckets, no datasets, or no
    /// points — this matches Google's behaviour for users who have not yet
    /// recorded any steps today and avoids spurious "no data" errors at the
    /// UI layer.
    pub fn total_steps(&self) -> i64 {
        self.bucket
            .iter()
            .flat_map(|b| &b.dataset)
            .flat_map(|d| &d.point)
            .flat_map(|p| &p.value)
            .filter_map(|v| v.int_val)
            .sum()
    }

    /// Average heart rate across every `fpVal` in the response, rounded to
    /// whole beats per minute.
    ///
    /// Returns `None` when the response carries no floating-point values —
    /// the shape Google returns for a user whose devices record no heart
    /// rate. "No sensor" must not read as "0 bpm", so this is an `Option`,
    /// never a zero.
    pub fn average_bpm(&self) -> Option<u16> {
        let values: Vec<f64> = self
            .bucket
            .iter()
            .flat_map(|b| &b.dataset)
            .flat_map(|d| &d.point)
            .flat_map(|p| &p.value)
            .filter_map(|v| v.fp_val)
            .filter(|v| v.is_finite() && *v > 0.0)
            .collect();
        if values.is_empty() {
            return None;
        }
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        Some(mean.round().clamp(0.0, u16::MAX as f64) as u16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_steps_sums_all_int_vals() {
        let json = r#"{
            "bucket": [{
                "dataset": [{
                    "point": [
                        { "value": [{ "intVal": 1234 }] },
                        { "value": [{ "intVal": 5678 }] }
                    ]
                }]
            }]
        }"#;
        let resp: AggregateResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.total_steps(), 1234 + 5678);
    }

    #[test]
    fn total_steps_returns_zero_for_empty_response() {
        let resp: AggregateResponse = serde_json::from_str(r#"{"bucket": []}"#).unwrap();
        assert_eq!(resp.total_steps(), 0);
    }

    #[test]
    fn total_steps_skips_missing_int_val() {
        let json = r#"{
            "bucket": [{
                "dataset": [{
                    "point": [
                        { "value": [{}] },
                        { "value": [{ "intVal": 42 }] }
                    ]
                }]
            }]
        }"#;
        let resp: AggregateResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.total_steps(), 42);
    }

    #[test]
    fn oauth_token_response_parses_minimal_payload() {
        let json = r#"{"access_token":"abc","expires_in":3600,"token_type":"Bearer"}"#;
        let tok: OAuthTokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(tok.access_token, "abc");
        assert_eq!(tok.expires_in, 3600);
    }

    #[test]
    fn average_bpm_averages_fp_vals() {
        let json = r#"{
            "bucket": [{
                "dataset": [{
                    "point": [
                        { "value": [{ "fpVal": 70.0 }] },
                        { "value": [{ "fpVal": 75.0 }] }
                    ]
                }]
            }]
        }"#;
        let resp: AggregateResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.average_bpm(), Some(73)); // 72.5 rounds to 73
    }

    #[test]
    fn average_bpm_is_none_when_no_fp_vals() {
        let json = r#"{"bucket":[{"dataset":[{"point":[{"value":[{"intVal":10}]}]}]}]}"#;
        let resp: AggregateResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.average_bpm(), None, "no sensor must not read as 0 bpm");
    }

    #[test]
    fn average_bpm_is_none_for_empty_response() {
        let resp: AggregateResponse = serde_json::from_str(r#"{"bucket": []}"#).unwrap();
        assert_eq!(resp.average_bpm(), None);
    }
}
