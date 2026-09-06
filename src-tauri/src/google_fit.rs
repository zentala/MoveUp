//! Low-level Google Fit REST client — credentials, endpoints, errors.
//!
//! Single responsibility: given OAuth2 client credentials + a long-lived
//! refresh token, hand back today's walking-step count. All HTTP concerns
//! live here (and in the sibling [`crate::google_fit_client`], which holds
//! the request-issuing [`GoogleFitClient`]) so the service layer above can
//! stay focused on caching and scheduling.
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

pub use crate::google_fit_client::GoogleFitClient;
pub use crate::google_fit_models::ErrorKind;

/// Default production endpoints. Overridable in tests via `Endpoints::at_base`.
const DEFAULT_TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
const DEFAULT_AGGREGATE_ENDPOINT: &str =
    "https://www.googleapis.com/fitness/v1/users/me/dataset:aggregate";
const DEFAULT_DATA_SOURCES_ENDPOINT: &str =
    "https://www.googleapis.com/fitness/v1/users/me/dataSources";

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
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn at_base(base: &str) -> Self {
        Self {
            token: format!("{base}/token"),
            aggregate: format!("{base}/aggregate"),
            data_sources: format!("{base}/dataSources"),
        }
    }
}
/// Fallback used when env override is unset and discovery returns nothing.
pub const DEFAULT_STEPS_DATA_SOURCE: &str =
    "derived:com.google.step_count.delta:com.google.android.gms:estimated_steps";

/// OAuth 2.0 error response body (RFC 6749 §5.2 shape).
#[derive(serde::Deserialize)]
struct OAuthErrorBody {
    #[serde(default)]
    error: String,
}

/// Decide whether a non-2xx HTTP response represents an "auth revoked"
/// condition the user must act on, vs a transient failure to retry.
///
/// Auth-revoked sources of truth:
///   1. HTTP 401 — bearer token rejected.
///   2. JSON body `{"error":"invalid_grant", ...}` per RFC 6749 §5.2.
///      Google returns this with HTTP 400 on revoked refresh tokens,
///      which is why we cannot rely on status alone.
///
/// Substring matching on the raw body is brittle (whitespace, casing,
/// alternative wrappers); JSON parse is the documented contract.
pub(crate) fn classify_response(status: reqwest::StatusCode, body: &str) -> ErrorKind {
    if status.as_u16() == 401 {
        return ErrorKind::AuthRevoked;
    }
    if let Ok(parsed) = serde_json::from_str::<OAuthErrorBody>(body) {
        if parsed.error == "invalid_grant" {
            return ErrorKind::AuthRevoked;
        }
    }
    ErrorKind::Transient
}

/// Sanitized client-error type with classification.
#[derive(Debug, Clone)]
pub struct FitError {
    pub kind: ErrorKind,
    pub message: String,
}

impl FitError {
    pub(crate) fn transient(msg: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Transient,
            message: msg.into(),
        }
    }

    pub(crate) fn auth_revoked(msg: impl Into<String>) -> Self {
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
