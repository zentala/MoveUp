//! Google Fit as a [`HealthSource`].
//!
//! Owns:
//!   - the in-memory [`HealthSnapshot`] cache so UI reads are instant,
//!   - the last error (so a cached read can report classification without
//!     re-hitting the network),
//!   - a [`SourceCache`], which resolves *which* Fit data streams this
//!     account uses (one discovery per data type, per process).
//!
//! Thread-safety: tokio `Mutex` around all mutable state. An in-flight
//! refresh is shared across concurrent callers via a oneshot broadcast
//! so a flurry of clicks results in **one** outbound API call.

use crate::google_fit::{Credentials, ErrorKind, FitError, GoogleFitClient, SourceCache};
use crate::health_models::{HealthSnapshot, HealthView};
use crate::health_source::HealthSource;
use async_trait::async_trait;
use tokio::sync::{broadcast, Mutex};

/// `source_id` this service stamps on every snapshot it produces.
pub const GOOGLE_FIT_SOURCE_ID: &str = "google_fit";

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

/// Cached service state guarded by a single mutex.
#[derive(Default)]
struct ServiceState {
    snapshot: Option<HealthSnapshot>,
    last_error_kind: Option<ErrorKind>,
    last_error_message: Option<String>,
    /// In-flight refresh broadcast — concurrent callers subscribe instead
    /// of issuing parallel API calls.
    inflight: Option<broadcast::Sender<Result<HealthSnapshot, FitError>>>,
}

/// Caching wrapper around `GoogleFitClient`.
pub struct GoogleFitService {
    client: Option<GoogleFitClient>,
    sources: SourceCache,
    state: Mutex<ServiceState>,
}

impl GoogleFitService {
    /// Construct from environment. Always succeeds; absence of credentials
    /// just disables the service.
    pub fn from_env() -> Self {
        let client = Credentials::from_env().map(GoogleFitClient::new);
        Self {
            client,
            sources: SourceCache::default(),
            state: Mutex::new(ServiceState::default()),
        }
    }

    /// Test-only constructor: build a configured service pointed at a
    /// custom set of endpoints (wiremock).
    #[cfg(test)]
    pub fn with_client(client: GoogleFitClient) -> Self {
        Self {
            client: Some(client),
            sources: SourceCache::default(),
            state: Mutex::new(ServiceState::default()),
        }
    }

    /// Whether OAuth credentials were present at startup.
    pub fn is_configured(&self) -> bool {
        self.client.is_some()
    }

    /// Best-effort heart rate for the window. Never fails the refresh:
    /// Fit is first a step source, and an account without heart rate (or
    /// without the heart-rate scope) must still show its steps.
    async fn fetch_hr(&self, client: &GoogleFitClient, start: i64, end: i64) -> Option<u16> {
        let source = self.sources.heart_rate(client).await?;
        match client.fetch_heart_rate(start, end, &source).await {
            Ok(bpm) => bpm,
            Err(e) => {
                log::warn!("google_fit: heart rate unavailable: {e}");
                None
            }
        }
    }

    /// Run one refresh: steps, then best-effort heart rate.
    async fn fetch_snapshot(&self, client: &GoogleFitClient) -> Result<HealthSnapshot, FitError> {
        let source = self.sources.steps(client).await;
        let (start, end) = local_day_window_ms(chrono::Local::now());
        let steps = client.fetch_steps(start, end, &source).await?;
        Ok(HealthSnapshot {
            steps_today: steps,
            heart_rate_bpm: self.fetch_hr(client, start, end).await,
            hrv_rmssd_ms: None,
            source_id: GOOGLE_FIT_SOURCE_ID.to_string(),
            fetched_at_ms: chrono::Utc::now().timestamp_millis(),
        })
    }

    /// Persist a refresh result into the cache.
    async fn store(&self, result: &Result<HealthSnapshot, FitError>) {
        let mut revoked = false;
        let mut st = self.state.lock().await;
        st.inflight = None;
        match result {
            Ok(snap) => {
                st.snapshot = Some(snap.clone());
                st.last_error_kind = None;
                st.last_error_message = None;
            }
            Err(err) => {
                st.last_error_kind = Some(err.kind);
                st.last_error_message = Some(err.message.clone());
                revoked = err.kind == ErrorKind::AuthRevoked;
            }
        }
        drop(st);
        // Auth revoked invalidates the resolved sources — re-consent may
        // grant different scopes, and therefore different streams.
        if revoked {
            self.sources.invalidate().await;
        }
    }
}

#[async_trait]
impl HealthSource for GoogleFitService {
    fn id(&self) -> &str {
        GOOGLE_FIT_SOURCE_ID
    }

    /// Build a `HealthView` reflecting the current cached state without
    /// touching the network.
    async fn view(&self) -> HealthView {
        let st = self.state.lock().await;
        HealthView {
            configured: self.is_configured(),
            snapshot: st.snapshot.clone(),
            error_kind: st.last_error_kind,
            error_message: st.last_error_message.clone(),
        }
    }

    /// Force a fresh API call, update the cache, and return the new view.
    ///
    /// Concurrent callers piggyback on the first in-flight request; only
    /// one outbound HTTP call goes to Google per refresh window.
    async fn refresh(&self) -> HealthView {
        let Some(client) = self.client.as_ref() else {
            return HealthView::unconfigured();
        };

        // Subscribe to an existing in-flight refresh, or claim ownership.
        let (own_tx, mut rx) = {
            let mut st = self.state.lock().await;
            if let Some(tx) = &st.inflight {
                (None, tx.subscribe())
            } else {
                let (tx, rx) = broadcast::channel(1);
                st.inflight = Some(tx.clone());
                (Some(tx), rx)
            }
        };

        if let Some(tx) = own_tx {
            // We own the request — execute and broadcast the result.
            let result = self.fetch_snapshot(client).await;
            // Persist + clear inflight before broadcasting so late
            // subscribers always observe the post-result state.
            self.store(&result).await;
            // Ignore send errors — they just mean no piggyback subscriber.
            let _ = tx.send(result);
        } else {
            // Passenger — wait for the owner to broadcast.
            let _ = rx.recv().await;
        }

        self.view().await
    }
}
