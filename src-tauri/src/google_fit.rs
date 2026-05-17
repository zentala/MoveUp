//! Low-level Google Fit REST client.
//!
//! Single responsibility: given OAuth2 client credentials + a long-lived
//! refresh token, hand back today's walking-step count. All HTTP concerns
//! live here so the service layer above can stay focused on caching and
//! scheduling.
//!
//! ## Token model
//! Google Fit's OAuth2 flow uses a one-time consent that yields a
//! refresh token (stored in `.env.local`). Access tokens last ~1h, so this
//! module refreshes on every call — Fit's "aggregate today" is cheap and
//! we poll infrequently, so token caching would be premature.
//!
//! ## Secrets
//! Credentials never enter logs or error strings. API errors are surfaced
//! as opaque `"google fit: <kind>"` messages.

use crate::google_fit_models::{AggregateResponse, OAuthTokenResponse};
use serde_json::json;

const TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
const AGGREGATE_ENDPOINT: &str =
    "https://www.googleapis.com/fitness/v1/users/me/dataset:aggregate";
const STEPS_DATA_TYPE: &str = "com.google.step_count.delta";
const STEPS_DATA_SOURCE: &str =
    "derived:com.google.step_count.delta:com.google.android.gms:estimated_steps";
/// One full day in milliseconds — bucket size for the aggregate request.
const DAY_MS: i64 = 86_400_000;

/// Credentials and tokens needed for the Fit API.
///
/// Loaded from environment (see `Credentials::from_env`) so the only place
/// secrets live at rest is `.env.local`, which is gitignored.
#[derive(Debug, Clone)]
pub struct Credentials {
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: String,
}

impl Credentials {
    /// Read credentials from environment variables. Returns `None` if any
    /// of the three required variables is missing — callers should treat
    /// this as "Google Fit not configured" rather than an error.
    pub fn from_env() -> Option<Self> {
        let client_id = std::env::var("GOOGLE_FIT_CLIENT_ID").ok()?;
        let client_secret = std::env::var("GOOGLE_FIT_CLIENT_SECRET").ok()?;
        let refresh_token = std::env::var("GOOGLE_FIT_REFRESH_TOKEN").ok()?;
        if client_id.is_empty() || client_secret.is_empty() || refresh_token.is_empty() {
            return None;
        }
        Some(Self {
            client_id,
            client_secret,
            refresh_token,
        })
    }
}

/// HTTP client bound to a set of credentials.
pub struct GoogleFitClient {
    creds: Credentials,
    http: reqwest::Client,
}

impl GoogleFitClient {
    pub fn new(creds: Credentials) -> Self {
        Self {
            creds,
            http: reqwest::Client::new(),
        }
    }

    /// Exchange the refresh token for a short-lived access token.
    async fn refresh_access_token(&self) -> Result<String, String> {
        let form = [
            ("client_id", self.creds.client_id.as_str()),
            ("client_secret", self.creds.client_secret.as_str()),
            ("refresh_token", self.creds.refresh_token.as_str()),
            ("grant_type", "refresh_token"),
        ];
        let resp = self
            .http
            .post(TOKEN_ENDPOINT)
            .form(&form)
            .send()
            .await
            .map_err(|_| "google fit: token request failed".to_string())?;
        if !resp.status().is_success() {
            return Err(format!("google fit: token http {}", resp.status().as_u16()));
        }
        let body: OAuthTokenResponse = resp
            .json()
            .await
            .map_err(|_| "google fit: token parse failed".to_string())?;
        Ok(body.access_token)
    }

    /// Fetch today's step count (local-day window).
    ///
    /// `start_of_day_ms` / `end_of_day_ms` must be unix milliseconds for the
    /// caller's local midnight boundaries — pushed in as parameters so this
    /// module stays free of timezone concerns and is trivially testable.
    pub async fn fetch_steps(
        &self,
        start_of_day_ms: i64,
        end_of_day_ms: i64,
    ) -> Result<i64, String> {
        let access_token = self.refresh_access_token().await?;
        let body = json!({
            "aggregateBy": [{
                "dataTypeName": STEPS_DATA_TYPE,
                "dataSourceId": STEPS_DATA_SOURCE
            }],
            "bucketByTime": { "durationMillis": DAY_MS },
            "startTimeMillis": start_of_day_ms,
            "endTimeMillis": end_of_day_ms
        });
        let resp = self
            .http
            .post(AGGREGATE_ENDPOINT)
            .bearer_auth(&access_token)
            .json(&body)
            .send()
            .await
            .map_err(|_| "google fit: aggregate request failed".to_string())?;
        if !resp.status().is_success() {
            return Err(format!(
                "google fit: aggregate http {}",
                resp.status().as_u16()
            ));
        }
        let parsed: AggregateResponse = resp
            .json()
            .await
            .map_err(|_| "google fit: aggregate parse failed".to_string())?;
        Ok(parsed.total_steps())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credentials_from_env_returns_none_when_missing() {
        // SAFETY: tests run sequentially within a single process — this still
        // races against parallel tests; if flakiness appears, gate behind
        // a serial_test mutex.
        std::env::remove_var("GOOGLE_FIT_CLIENT_ID");
        std::env::remove_var("GOOGLE_FIT_CLIENT_SECRET");
        std::env::remove_var("GOOGLE_FIT_REFRESH_TOKEN");
        assert!(Credentials::from_env().is_none());
    }

    #[test]
    fn credentials_from_env_rejects_empty_values() {
        std::env::set_var("GOOGLE_FIT_CLIENT_ID", "");
        std::env::set_var("GOOGLE_FIT_CLIENT_SECRET", "x");
        std::env::set_var("GOOGLE_FIT_REFRESH_TOKEN", "y");
        assert!(Credentials::from_env().is_none());
        std::env::remove_var("GOOGLE_FIT_CLIENT_ID");
        std::env::remove_var("GOOGLE_FIT_CLIENT_SECRET");
        std::env::remove_var("GOOGLE_FIT_REFRESH_TOKEN");
    }
}
