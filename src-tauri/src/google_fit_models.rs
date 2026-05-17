//! Data structures for the Google Fit walking-steps integration.
//!
//! Three categories of types live here:
//! 1. **OAuth wire types** — what the Google token endpoint returns.
//! 2. **Fit API wire types** — the minimal subset of the aggregate response.
//! 3. **Public snapshot type** — the cached value exposed to the UI.

use serde::{Deserialize, Serialize};

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

/// A single value entry; for step counts only `int_val` is populated.
#[derive(Debug, Deserialize)]
pub struct AggregateValue {
    #[serde(default, rename = "intVal")]
    pub int_val: Option<i64>,
}

/// Cached, UI-ready representation of "steps today".
///
/// `fetched_at_ms` is wall-clock unix-milliseconds at the moment the snapshot
/// was produced, allowing the frontend to render a relative "5 minutes ago".
#[derive(Debug, Clone, Serialize)]
pub struct StepsSnapshot {
    pub steps_today: i64,
    pub fetched_at_ms: i64,
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
}
