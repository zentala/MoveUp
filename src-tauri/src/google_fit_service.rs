//! Service layer over `GoogleFitClient`.
//!
//! Owns:
//!   - the in-memory `StepsSnapshot` cache so UI reads are instant,
//!   - the last error (so `get_steps_today` can return classification
//!     without re-hitting the network),
//!   - the discovered/overridden steps data source (one discovery per
//!     process lifetime).
//!
//! Thread-safety: tokio `Mutex` around all mutable state. An in-flight
//! refresh is shared across concurrent callers via a oneshot broadcast
//! so a flurry of clicks results in **one** outbound API call.

use crate::google_fit::{
    Credentials, ErrorKind, FitError, GoogleFitClient, DEFAULT_STEPS_DATA_SOURCE,
};
use crate::google_fit_models::{StepsSnapshot, StepsView};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

/// Returns (start_of_today_ms, end_of_tomorrow_local_midnight_ms) in the
/// timezone of the supplied `now` instant.
///
/// Both bounds are derived as **local midnight** instants. End-of-day is
/// *not* `start + 86_400_000`, because on DST transition days the local
/// day is 23h or 25h long and `+24h` lands an hour off — past or before
/// the actual next midnight. Computing the next local midnight via the
/// timezone gives the correct boundary in all cases.
///
/// Generic over `TimeZone` so tests can pin a specific zone
/// (e.g. `Europe/Warsaw` via `chrono-tz`) independent of the host's TZ.
pub(crate) fn local_day_window_ms<Tz>(now: chrono::DateTime<Tz>) -> (i64, i64)
where
    Tz: chrono::TimeZone,
{
    use chrono::{Datelike, Days};
    let tz = now.timezone();
    let start = tz
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
        .single()
        .unwrap_or(now);
    let next = start.clone().checked_add_days(Days::new(1)).unwrap_or(start.clone());
    let end = tz
        .with_ymd_and_hms(next.year(), next.month(), next.day(), 0, 0, 0)
        .single()
        .unwrap_or(next);
    (start.timestamp_millis(), end.timestamp_millis())
}

/// How long a *failed* discovery is honored before retrying. Prevents a
/// retry-storm against `dataSources` when that endpoint is transiently
/// flaky (we'd otherwise hit it once per poll = 12×/h).
const DISCOVERY_FAILURE_TTL_MS: i64 = 60 * 60 * 1000;

/// Cached service state guarded by a single mutex.
#[derive(Default)]
struct ServiceState {
    snapshot: Option<StepsSnapshot>,
    last_error_kind: Option<ErrorKind>,
    last_error_message: Option<String>,
    /// Discovered (or fallback) data source — cached after the first
    /// resolution attempt so subsequent refreshes skip the discovery
    /// HTTP call. Cleared on `AuthRevoked` (new consent may grant new
    /// scopes / sources).
    cached_source: Option<String>,
    /// Wall-clock ms at which `cached_source` was set. Used to decide
    /// when a fallback (default) entry should expire and be retried.
    cached_source_at_ms: i64,
    /// `true` when `cached_source` holds the static fallback because
    /// discovery failed (not a real discovery result). Re-attempted after
    /// `DISCOVERY_FAILURE_TTL_MS`.
    cached_source_is_fallback: bool,
    /// In-flight refresh broadcast — concurrent callers subscribe instead
    /// of issuing parallel API calls.
    inflight: Option<broadcast::Sender<Result<StepsSnapshot, FitError>>>,
}

/// Caching wrapper around `GoogleFitClient`.
pub struct GoogleFitService {
    client: Option<GoogleFitClient>,
    state: Mutex<ServiceState>,
}

impl GoogleFitService {
    /// Construct from environment. Always succeeds; absence of credentials
    /// just disables the service.
    pub fn from_env() -> Self {
        let client = Credentials::from_env().map(GoogleFitClient::new);
        Self {
            client,
            state: Mutex::new(ServiceState::default()),
        }
    }

    /// Test-only constructor: build a configured service pointed at a
    /// custom set of endpoints (wiremock).
    #[cfg(test)]
    pub fn with_client(client: GoogleFitClient) -> Self {
        Self {
            client: Some(client),
            state: Mutex::new(ServiceState::default()),
        }
    }

    /// Whether OAuth credentials were present at startup.
    pub fn is_configured(&self) -> bool {
        self.client.is_some()
    }

    /// Build a `StepsView` reflecting the current cached state without
    /// touching the network.
    pub async fn view(&self) -> StepsView {
        let st = self.state.lock().await;
        StepsView {
            configured: self.is_configured(),
            snapshot: st.snapshot.clone(),
            error_kind: st.last_error_kind,
            error_message: st.last_error_message.clone(),
        }
    }

    /// Resolve the steps data source.
    ///
    /// Order of precedence:
    ///   1. Env var override (`GOOGLE_FIT_STEPS_SOURCE`) — always wins.
    ///   2. Valid cached discovery — reused forever (until auth revoked).
    ///   3. Fallback cached < 1h ago — reused to avoid retry-storm against
    ///      a flaky `dataSources` endpoint.
    ///   4. Fresh discovery — cached on success, or fallback cached with
    ///      a 1h TTL on failure.
    async fn resolve_source(&self, client: &GoogleFitClient) -> String {
        if let Some(over) = client.steps_source_override() {
            return over.to_string();
        }
        let now_ms = chrono::Utc::now().timestamp_millis();
        {
            let st = self.state.lock().await;
            if let Some(cached) = &st.cached_source {
                // Real discovery result: cache indefinitely.
                if !st.cached_source_is_fallback {
                    return cached.clone();
                }
                // Fallback entry: honor for the TTL window before retrying.
                if now_ms - st.cached_source_at_ms < DISCOVERY_FAILURE_TTL_MS {
                    return cached.clone();
                }
            }
        }
        match client.list_step_sources().await {
            Ok(sources) => {
                let picked = GoogleFitClient::rank_step_sources(&sources)
                    .unwrap_or_else(|| DEFAULT_STEPS_DATA_SOURCE.to_string());
                let is_fallback_pick = picked == DEFAULT_STEPS_DATA_SOURCE && sources.is_empty();
                log::info!("google_fit: discovered steps data source: {picked}");
                let mut st = self.state.lock().await;
                st.cached_source = Some(picked.clone());
                st.cached_source_at_ms = now_ms;
                st.cached_source_is_fallback = is_fallback_pick;
                picked
            }
            Err(e) => {
                log::warn!(
                    "google_fit: data source discovery failed: {e}; caching default for {}min",
                    DISCOVERY_FAILURE_TTL_MS / 60_000,
                );
                let mut st = self.state.lock().await;
                st.cached_source = Some(DEFAULT_STEPS_DATA_SOURCE.to_string());
                st.cached_source_at_ms = now_ms;
                st.cached_source_is_fallback = true;
                DEFAULT_STEPS_DATA_SOURCE.to_string()
            }
        }
    }

    /// Force a fresh API call, update the cache, and return the new view.
    ///
    /// Concurrent callers piggyback on the first in-flight request; only
    /// one outbound HTTP call goes to Google per refresh window.
    pub async fn refresh(&self) -> StepsView {
        let Some(client) = self.client.as_ref() else {
            return StepsView {
                configured: false,
                snapshot: None,
                error_kind: None,
                error_message: None,
            };
        };

        // Subscribe to an existing in-flight refresh, or claim ownership.
        let (own_tx, mut rx) = {
            let mut st = self.state.lock().await;
            if let Some(tx) = &st.inflight {
                // Piggyback on the existing flight.
                let rx = tx.subscribe();
                (None, rx)
            } else {
                let (tx, rx) = broadcast::channel(1);
                st.inflight = Some(tx.clone());
                (Some(tx), rx)
            }
        };

        if let Some(tx) = own_tx {
            // We own the request — execute and broadcast result.
            let source = self.resolve_source(client).await;
            let (start, end) = local_day_window_ms(chrono::Local::now());
            let result = match client.fetch_steps(start, end, &source).await {
                Ok(steps) => Ok(StepsSnapshot {
                    steps_today: steps,
                    fetched_at_ms: chrono::Utc::now().timestamp_millis(),
                }),
                Err(e) => Err(e),
            };
            // Persist + clear inflight before broadcasting so late
            // subscribers always observe the post-result state.
            {
                let mut st = self.state.lock().await;
                st.inflight = None;
                match &result {
                    Ok(snap) => {
                        st.snapshot = Some(snap.clone());
                        st.last_error_kind = None;
                        st.last_error_message = None;
                    }
                    Err(err) => {
                        st.last_error_kind = Some(err.kind);
                        st.last_error_message = Some(err.message.clone());
                        // Auth revoked invalidates any cached source — next
                        // attempt will discover fresh after re-consent.
                        if err.kind == ErrorKind::AuthRevoked {
                            st.cached_source = None;
                            st.cached_source_at_ms = 0;
                            st.cached_source_is_fallback = false;
                        }
                    }
                }
            }
            // Ignore send errors — they just mean no piggyback subscriber.
            let _ = tx.send(result);
        } else {
            // Passenger — wait for the owner to broadcast.
            let _ = rx.recv().await;
        }

        self.view().await
    }
}

/// Type alias used by Tauri state.
pub type GoogleFitState = Arc<GoogleFitService>;