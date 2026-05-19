//! Low-level Google Fit REST client.
//!
//! Single responsibility: given OAuth2 client credentials + a long-lived
//! refresh token, hand back today's walking-step count. All HTTP concerns
//! live here so the service layer above can stay focused on caching and
//! scheduling.
//!
//! ## Token model
//! Google Fit's OAuth2 flow uses a one-time consent that yields a refresh
//! token (stored in `.env`). Access tokens last ~1h, so this module
//! refreshes on every call — Fit's "aggregate today" is cheap and we poll
//! infrequently, so token caching would be premature.
//!
//! ## Error model
//! `FitError` separates *auth revoked* (user-actionable: re-run OAuth) from
//! *transient* (network blip, rate limit, server 5xx). The frontend reads
//! `kind` to decide between a red dot and a "reconnect" CTA.
//!
//! ## Secrets
//! Credentials never enter logs or error strings; full reqwest errors go to
//! `log::warn!` so they show up in stderr/log files but the user-facing
//! string stays sanitized.

use crate::google_fit_models::{AggregateResponse, OAuthTokenResponse};
use serde::Serialize;
use serde_json::json;

/// Default production endpoints. Overridable in tests via `Endpoints::custom`.
const DEFAULT_TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
const DEFAULT_AGGREGATE_ENDPOINT: &str =
    "https://www.googleapis.com/fitness/v1/users/me/dataset:aggregate";
const DEFAULT_DATA_SOURCES_ENDPOINT: &str =
    "https://www.googleapis.com/fitness/v1/users/me/dataSources";
const STEPS_DATA_TYPE: &str = "com.google.step_count.delta";

/// Resolvable endpoints — production defaults or test overrides.
#[derive(Debug, Clone)]
pub struct Endpoints {
    pub token: String,
    pub aggregate: String,
    pub data_sources: String,
}

impl Default for Endpoints {
    fn default() -> Self {
        Self {
            token: DEFAULT_TOKEN_ENDPOINT.to_string(),
            aggregate: DEFAULT_AGGREGATE_ENDPOINT.to_string(),
            data_sources: DEFAULT_DATA_SOURCES_ENDPOINT.to_string(),
        }
    }
}

impl Endpoints {
    /// Build endpoints rooted at a single base URL — useful when pointing
    /// all calls at a wiremock server in tests.
    pub fn at_base(base: &str) -> Self {
        Self {
            token: format!("{base}/token"),
            aggregate: format!("{base}/aggregate"),
            data_sources: format!("{base}/dataSources"),
        }
    }
}
/// Fallback used when env override is unset and discovery returns nothing.
const DEFAULT_STEPS_DATA_SOURCE: &str =
    "derived:com.google.step_count.delta:com.google.android.gms:estimated_steps";
/// One full day in milliseconds — bucket size for the aggregate request.
const DAY_MS: i64 = 86_400_000;

/// Classification of API failures the frontend cares about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    /// Token revoked, expired beyond recovery, or scope missing. User must
    /// re-run the OAuth helper script and paste a fresh refresh token.
    AuthRevoked,
    /// Network failure, rate limit, server 5xx, parse error — retryable.
    Transient,
}

/// Sanitized client-error type with classification.
#[derive(Debug, Clone)]
pub struct FitError {
    pub kind: ErrorKind,
    pub message: String,
}

impl FitError {
    fn transient(msg: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Transient,
            message: msg.into(),
        }
    }

    fn auth_revoked(msg: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::AuthRevoked,
            message: msg.into(),
        }
    }
}

impl std::fmt::Display for FitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for FitError {}

/// Credentials and optional data-source override loaded from env.
#[derive(Debug, Clone)]
pub struct Credentials {
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: String,
    /// `GOOGLE_FIT_STEPS_SOURCE` — overrides discovery when set.
    pub steps_source_override: Option<String>,
}

impl Credentials {
    /// Read credentials from environment variables. Returns `None` if any
    /// of the three required variables is missing — callers should treat
    /// this as "Google Fit not configured" rather than an error.
    pub fn from_env() -> Option<Self> {
        let client_id = std::env::var("GOOGLE_CLIENT_ID").ok()?;
        let client_secret = std::env::var("GOOGLE_CLIENT_SECRET").ok()?;
        let refresh_token = std::env::var("GOOGLE_REFRESH_TOKEN").ok()?;
        if client_id.is_empty() || client_secret.is_empty() || refresh_token.is_empty() {
            return None;
        }
        let steps_source_override = std::env::var("GOOGLE_FIT_STEPS_SOURCE")
            .ok()
            .filter(|s| !s.is_empty());
        Some(Self {
            client_id,
            client_secret,
            refresh_token,
            steps_source_override,
        })
    }
}

/// HTTP client bound to a set of credentials.
pub struct GoogleFitClient {
    creds: Credentials,
    http: reqwest::Client,
    endpoints: Endpoints,
}

impl GoogleFitClient {
    pub fn new(creds: Credentials) -> Self {
        Self::with_endpoints(creds, Endpoints::default())
    }

    pub fn with_endpoints(creds: Credentials, endpoints: Endpoints) -> Self {
        Self {
            creds,
            http: reqwest::Client::new(),
            endpoints,
        }
    }

    /// Env override if present.
    pub fn steps_source_override(&self) -> Option<&str> {
        self.creds.steps_source_override.as_deref()
    }

    /// Exchange the refresh token for a short-lived access token.
    async fn refresh_access_token(&self) -> Result<String, FitError> {
        let form = [
            ("client_id", self.creds.client_id.as_str()),
            ("client_secret", self.creds.client_secret.as_str()),
            ("refresh_token", self.creds.refresh_token.as_str()),
            ("grant_type", "refresh_token"),
        ];
        let resp = self
            .http
            .post(&self.endpoints.token)
            .form(&form)
            .send()
            .await
            .map_err(|e| {
                log::warn!("google_fit: token request: {e:?}");
                FitError::transient("google fit: token request failed")
            })?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            // Google returns `{"error":"invalid_grant", ...}` on revoked /
            // expired refresh tokens. 401 anywhere in the token flow means
            // the same — user must re-consent.
            if body.contains("invalid_grant") || status.as_u16() == 401 {
                log::warn!("google_fit: refresh token revoked (status {status}, body: {body})");
                return Err(FitError::auth_revoked("google fit: refresh token revoked"));
            }
            log::warn!("google_fit: token http {status}, body: {body}");
            return Err(FitError::transient(format!(
                "google fit: token http {}",
                status.as_u16()
            )));
        }
        let body: OAuthTokenResponse = resp.json().await.map_err(|e| {
            log::warn!("google_fit: token parse: {e:?}");
            FitError::transient("google fit: token parse failed")
        })?;
        Ok(body.access_token)
    }

    /// List step-count data sources available to this user. Used by the
    /// service layer's auto-discovery on first refresh when no env override
    /// is set.
    pub async fn list_step_sources(&self) -> Result<Vec<String>, FitError> {
        let access_token = self.refresh_access_token().await?;
        let url = format!("{}?dataTypeName={STEPS_DATA_TYPE}", self.endpoints.data_sources);
        let resp = self
            .http
            .get(&url)
            .bearer_auth(&access_token)
            .send()
            .await
            .map_err(|e| {
                log::warn!("google_fit: dataSources request: {e:?}");
                FitError::transient("google fit: dataSources request failed")
            })?;
        let status = resp.status();
        if !status.is_success() {
            if status.as_u16() == 401 {
                return Err(FitError::auth_revoked("google fit: auth revoked"));
            }
            return Err(FitError::transient(format!(
                "google fit: dataSources http {}",
                status.as_u16()
            )));
        }
        #[derive(serde::Deserialize)]
        struct DataSourcesResponse {
            #[serde(default)]
            #[serde(rename = "dataSource")]
            data_source: Vec<DataSourceItem>,
        }
        #[derive(serde::Deserialize)]
        struct DataSourceItem {
            #[serde(default)]
            #[serde(rename = "dataStreamId")]
            data_stream_id: String,
        }
        let parsed: DataSourcesResponse = resp.json().await.map_err(|e| {
            log::warn!("google_fit: dataSources parse: {e:?}");
            FitError::transient("google fit: dataSources parse failed")
        })?;
        Ok(parsed
            .data_source
            .into_iter()
            .map(|d| d.data_stream_id)
            .filter(|s| !s.is_empty())
            .collect())
    }

    /// Pick the best data source from the list of step sources.
    ///
    /// Preference: aggregated derived sources (`merge_step_deltas`,
    /// `estimated_steps`) over device-specific raw streams, because they
    /// fold together everything a user has connected.
    pub fn rank_step_sources(sources: &[String]) -> Option<String> {
        let priority = |s: &str| -> i32 {
            if s.ends_with(":merge_step_deltas") {
                0
            } else if s.ends_with(":estimated_steps") {
                1
            } else if s.starts_with("derived:") {
                2
            } else if s.starts_with("raw:") {
                3
            } else {
                4
            }
        };
        sources.iter().min_by_key(|s| priority(s)).cloned()
    }

    /// Fetch step count for the given window from the given data source.
    pub async fn fetch_steps(
        &self,
        start_of_day_ms: i64,
        end_of_day_ms: i64,
        source: &str,
    ) -> Result<i64, FitError> {
        let access_token = self.refresh_access_token().await?;
        let body = json!({
            "aggregateBy": [{
                "dataTypeName": STEPS_DATA_TYPE,
                "dataSourceId": source,
            }],
            "bucketByTime": { "durationMillis": DAY_MS },
            "startTimeMillis": start_of_day_ms,
            "endTimeMillis": end_of_day_ms,
        });
        let resp = self
            .http
            .post(&self.endpoints.aggregate)
            .bearer_auth(&access_token)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                log::warn!("google_fit: aggregate request: {e:?}");
                FitError::transient("google fit: aggregate request failed")
            })?;
        let status = resp.status();
        if !status.is_success() {
            if status.as_u16() == 401 {
                return Err(FitError::auth_revoked("google fit: auth revoked"));
            }
            let body = resp.text().await.unwrap_or_default();
            log::warn!("google_fit: aggregate http {status}, body: {body}");
            return Err(FitError::transient(format!(
                "google fit: aggregate http {}",
                status.as_u16()
            )));
        }
        let parsed: AggregateResponse = resp.json().await.map_err(|e| {
            log::warn!("google_fit: aggregate parse: {e:?}");
            FitError::transient("google fit: aggregate parse failed")
        })?;
        Ok(parsed.total_steps())
    }
}

/// Default fallback exposed for the service layer.
pub fn default_steps_source() -> &'static str {
    DEFAULT_STEPS_DATA_SOURCE
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clear_env() {
        std::env::remove_var("GOOGLE_CLIENT_ID");
        std::env::remove_var("GOOGLE_CLIENT_SECRET");
        std::env::remove_var("GOOGLE_REFRESH_TOKEN");
        std::env::remove_var("GOOGLE_FIT_STEPS_SOURCE");
    }

    #[test]
    fn credentials_from_env_returns_none_when_missing() {
        clear_env();
        assert!(Credentials::from_env().is_none());
    }

    #[test]
    fn credentials_from_env_rejects_empty_values() {
        clear_env();
        std::env::set_var("GOOGLE_CLIENT_ID", "");
        std::env::set_var("GOOGLE_CLIENT_SECRET", "x");
        std::env::set_var("GOOGLE_REFRESH_TOKEN", "y");
        assert!(Credentials::from_env().is_none());
        clear_env();
    }

    #[test]
    fn credentials_picks_up_optional_source_override() {
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
    fn rank_step_sources_prefers_merge_step_deltas() {
        let v = vec![
            "raw:com.google.step_count.delta:samsung:SM-N9005:something".to_string(),
            "derived:com.google.step_count.delta:com.google.android.gms:estimated_steps".to_string(),
            "derived:com.google.step_count.delta:com.google.android.gms:merge_step_deltas".to_string(),
        ];
        assert_eq!(
            GoogleFitClient::rank_step_sources(&v).as_deref(),
            Some("derived:com.google.step_count.delta:com.google.android.gms:merge_step_deltas"),
        );
    }

    #[test]
    fn rank_step_sources_falls_back_to_first_raw_when_no_derived() {
        let v = vec!["raw:com.google.step_count.delta:samsung:SM-N9005:x".to_string()];
        assert_eq!(GoogleFitClient::rank_step_sources(&v).as_deref(), Some(v[0].as_str()));
    }

    #[test]
    fn rank_step_sources_none_when_empty() {
        assert!(GoogleFitClient::rank_step_sources(&[]).is_none());
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
}
