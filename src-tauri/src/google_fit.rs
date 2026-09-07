//! Low-level Google Fit REST layer — credentials, endpoints, errors, and
//! the resolved-source cache. The requests themselves live in the sibling
//! [`crate::google_fit_client`]; caching of *readings* lives one layer up
//! in [`crate::google_fit_service`].
//!
//! **Token model.** A one-time consent yields a refresh token (in `.env`);
//! access tokens last ~1h, so the client refreshes on every call — Fit's
//! "aggregate today" is cheap and we poll infrequently, so token caching
//! would be premature.
//!
//! **Error model.** `FitError` separates *auth revoked* (user-actionable:
//! re-run OAuth) from *transient* (network blip, rate limit, 5xx). The
//! frontend reads `kind` to choose between a red dot and a reconnect CTA.
//!
//! **Secrets.** Credentials never enter logs or error strings; full
//! reqwest errors go to `log::warn!`, the user-facing string stays
//! sanitized.

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

/// Which Fit data streams this account uses, resolved once and cached for
/// the process lifetime.
///
/// The service caches *readings*; this caches *where readings come from*.
/// It carries its own mutex, so a discovery call never holds the service's
/// snapshot lock.
#[derive(Default)]
pub struct SourceCache {
    state: tokio::sync::Mutex<SourceCacheState>,
}

#[derive(Default)]
struct SourceCacheState {
    steps: Option<String>,
    steps_at_ms: i64,
    /// `true` when `steps` holds the fallback because discovery failed.
    steps_is_fallback: bool,
    /// `None` = never resolved. `Some(None)` = "this account has no
    /// heart-rate stream", which is an answer, not a failure.
    hr: Option<Option<String>>,
    hr_at_ms: i64,
}

/// How long a *failed* discovery is honored before retrying — without it
/// a flaky `dataSources` endpoint would be hit once per poll (12×/h).
const DISCOVERY_FAILURE_TTL_MS: i64 = 60 * 60 * 1000;

impl SourceCache {
    /// Resolve the steps data source, in precedence order: the
    /// `GOOGLE_FIT_STEPS_SOURCE` override, a cached discovery (kept until
    /// [`Self::invalidate`]), a fallback cached < 1h ago, then fresh
    /// discovery — cached on success, or the static fallback with a 1h TTL.
    pub async fn steps(&self, client: &GoogleFitClient) -> String {
        if let Some(over) = client.steps_source_override() {
            return over.to_string();
        }
        let now_ms = chrono::Utc::now().timestamp_millis();
        {
            let st = self.state.lock().await;
            if let Some(cached) = &st.steps {
                if !st.steps_is_fallback || now_ms - st.steps_at_ms < DISCOVERY_FAILURE_TTL_MS {
                    return cached.clone();
                }
            }
        }
        let (picked, is_fallback) = match client.list_step_sources().await {
            Ok(sources) => match GoogleFitClient::rank_sources(&sources) {
                Some(p) => {
                    log::info!("google_fit: discovered steps data source: {p}");
                    (p, false)
                }
                None => (DEFAULT_STEPS_DATA_SOURCE.to_string(), true),
            },
            Err(e) => {
                log::warn!(
                    "google_fit: data source discovery failed: {e}; caching default for {}min",
                    DISCOVERY_FAILURE_TTL_MS / 60_000,
                );
                (DEFAULT_STEPS_DATA_SOURCE.to_string(), true)
            }
        };
        let mut st = self.state.lock().await;
        st.steps = Some(picked.clone());
        st.steps_at_ms = now_ms;
        st.steps_is_fallback = is_fallback;
        picked
    }

    /// Resolve the heart-rate data source, or `None` when this account has
    /// none (a failed discovery is also `None`, retried after the TTL).
    /// Unlike steps there is no static fallback: guessing a heart-rate
    /// stream id would turn "you own no watch" into a 404 on every poll.
    pub async fn heart_rate(&self, client: &GoogleFitClient) -> Option<String> {
        let now_ms = chrono::Utc::now().timestamp_millis();
        {
            let st = self.state.lock().await;
            if let Some(cached) = &st.hr {
                if cached.is_some() || now_ms - st.hr_at_ms < DISCOVERY_FAILURE_TTL_MS {
                    return cached.clone();
                }
            }
        }
        let picked = match client.list_heart_rate_sources().await {
            Ok(sources) => GoogleFitClient::rank_sources(&sources),
            Err(e) => {
                log::warn!("google_fit: heart-rate discovery failed: {e}");
                None
            }
        };
        let mut st = self.state.lock().await;
        st.hr = Some(picked.clone());
        st.hr_at_ms = now_ms;
        picked
    }

    /// Drop everything. Called on `AuthRevoked`: new consent may grant
    /// new scopes, and therefore new sources.
    pub async fn invalidate(&self) {
        *self.state.lock().await = SourceCacheState::default();
    }
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
