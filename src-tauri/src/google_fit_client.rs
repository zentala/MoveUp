//! HTTP calls against the Google Fit REST API.
//!
//! Split out of [`crate::google_fit`], which keeps the credential, endpoint
//! and error types this client is built from. Everything that actually
//! talks to Google lives here: token refresh, data-source discovery and the
//! aggregate step query.

use crate::google_fit::{classify_response, Credentials, Endpoints, ErrorKind, FitError};
use crate::google_fit_models::{AggregateResponse, OAuthTokenResponse};
use serde_json::json;

/// Fit data type identifier for step deltas.
const STEPS_DATA_TYPE: &str = "com.google.step_count.delta";
/// Fit data type identifier for heart rate.
pub const HEART_RATE_DATA_TYPE: &str = "com.google.heart_rate.bpm";
/// One full day in milliseconds — bucket size for the aggregate request.
const DAY_MS: i64 = 86_400_000;

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
            return Err(match classify_response(status, &body) {
                ErrorKind::AuthRevoked => {
                    log::warn!("google_fit: refresh token revoked (status {status}, body: {body})");
                    FitError::auth_revoked("google fit: refresh token revoked")
                }
                ErrorKind::Transient => {
                    log::warn!("google_fit: token http {status}, body: {body}");
                    FitError::transient(format!("google fit: token http {}", status.as_u16()))
                }
            });
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
        self.list_sources(STEPS_DATA_TYPE).await
    }

    /// List heart-rate data sources available to this user.
    ///
    /// An empty list is the answer to "does this account have heart rate at
    /// all" — the question T01's spike asked by hand. The service layer
    /// reads it to decide whether to issue a second aggregate call.
    pub async fn list_heart_rate_sources(&self) -> Result<Vec<String>, FitError> {
        self.list_sources(HEART_RATE_DATA_TYPE).await
    }

    /// List the data-stream ids the user has for one Fit data type.
    async fn list_sources(&self, data_type: &str) -> Result<Vec<String>, FitError> {
        let access_token = self.refresh_access_token().await?;
        let url = format!("{}?dataTypeName={data_type}", self.endpoints.data_sources);
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
            let body = resp.text().await.unwrap_or_default();
            return Err(match classify_response(status, &body) {
                ErrorKind::AuthRevoked => FitError::auth_revoked("google fit: auth revoked"),
                ErrorKind::Transient => FitError::transient(format!(
                    "google fit: dataSources http {}",
                    status.as_u16()
                )),
            });
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

    /// Pick the best data source from a list of Fit data streams.
    ///
    /// Preference: aggregated derived sources (`merge_*`, `estimated_*`)
    /// over device-specific raw streams, because they fold together
    /// everything a user has connected. The ranking is data-type agnostic,
    /// so steps and heart rate both go through it.
    pub fn rank_sources(sources: &[String]) -> Option<String> {
        let priority = |s: &str| -> i32 {
            if s.contains(":merge_") {
                0
            } else if s.contains(":estimated_") {
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
        let parsed = self
            .aggregate(STEPS_DATA_TYPE, source, start_of_day_ms, end_of_day_ms)
            .await?;
        Ok(parsed.total_steps())
    }

    /// Fetch today's average heart rate, in whole bpm.
    ///
    /// `Ok(None)` means the window holds no heart-rate points — a user
    /// whose devices never recorded any. That is not an error, and callers
    /// must not surface it as one.
    pub async fn fetch_heart_rate(
        &self,
        start_of_day_ms: i64,
        end_of_day_ms: i64,
        source: &str,
    ) -> Result<Option<u16>, FitError> {
        let parsed = self
            .aggregate(HEART_RATE_DATA_TYPE, source, start_of_day_ms, end_of_day_ms)
            .await?;
        Ok(parsed.average_bpm())
    }

    /// Issue one `dataset:aggregate` request and parse the response.
    async fn aggregate(
        &self,
        data_type: &str,
        source: &str,
        start_of_day_ms: i64,
        end_of_day_ms: i64,
    ) -> Result<AggregateResponse, FitError> {
        let access_token = self.refresh_access_token().await?;
        let body = json!({
            "aggregateBy": [{
                "dataTypeName": data_type,
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
            let body = resp.text().await.unwrap_or_default();
            log::warn!("google_fit: aggregate http {status}, body: {body}");
            return Err(match classify_response(status, &body) {
                ErrorKind::AuthRevoked => FitError::auth_revoked("google fit: auth revoked"),
                ErrorKind::Transient => FitError::transient(format!(
                    "google fit: aggregate http {}",
                    status.as_u16()
                )),
            });
        }
        resp.json::<AggregateResponse>().await.map_err(|e| {
            log::warn!("google_fit: aggregate parse: {e:?}");
            FitError::transient("google fit: aggregate parse failed")
        })
    }
}
