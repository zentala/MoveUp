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

use crate::google_fit::{default_steps_source, Credentials, ErrorKind, FitError, GoogleFitClient};
use crate::google_fit_models::{StepsSnapshot, StepsView};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

/// Returns (start_of_today_ms, end_of_tomorrow_local_midnight_ms).
///
/// Both bounds are derived as **local midnight** instants. End-of-day is
/// *not* `start + 86_400_000`, because on DST transition days the local
/// day is 23h or 25h long and `+24h` lands an hour off — past or before
/// the actual next midnight. Computing the next local midnight via the
/// timezone gives the correct boundary in all cases.
///
/// Clock is injected so tests can pin the window to any historical date,
/// including DST transitions.
fn local_day_window_ms(now: chrono::DateTime<chrono::Local>) -> (i64, i64) {
    use chrono::{Datelike, Days, Local, TimeZone};
    let start = Local
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
        .single()
        .unwrap_or(now);
    let next = start.checked_add_days(Days::new(1)).unwrap_or(start);
    let end = Local
        .with_ymd_and_hms(next.year(), next.month(), next.day(), 0, 0, 0)
        .single()
        .unwrap_or(next);
    (start.timestamp_millis(), end.timestamp_millis())
}

/// Cached service state guarded by a single mutex.
#[derive(Default)]
struct ServiceState {
    snapshot: Option<StepsSnapshot>,
    last_error_kind: Option<ErrorKind>,
    last_error_message: Option<String>,
    /// Discovered data source — cached after the first successful discovery
    /// so subsequent refreshes skip the extra HTTP call.
    cached_source: Option<String>,
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

    /// Resolve the steps data source: env override → cached discovery →
    /// fresh discovery → static fallback.
    async fn resolve_source(&self, client: &GoogleFitClient) -> String {
        if let Some(over) = client.steps_source_override() {
            return over.to_string();
        }
        {
            let st = self.state.lock().await;
            if let Some(cached) = &st.cached_source {
                return cached.clone();
            }
        }
        // Try discovery — on any failure, fall back to the static default
        // so the call chain still produces *some* answer.
        match client.list_step_sources().await {
            Ok(sources) => {
                let picked = GoogleFitClient::rank_step_sources(&sources)
                    .unwrap_or_else(|| default_steps_source().to_string());
                log::info!("google_fit: discovered steps data source: {picked}");
                self.state.lock().await.cached_source = Some(picked.clone());
                picked
            }
            Err(e) => {
                log::warn!("google_fit: data source discovery failed: {e}; falling back to default");
                default_steps_source().to_string()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn clear_env() {
        std::env::remove_var("GOOGLE_CLIENT_ID");
        std::env::remove_var("GOOGLE_CLIENT_SECRET");
        std::env::remove_var("GOOGLE_REFRESH_TOKEN");
        std::env::remove_var("GOOGLE_FIT_STEPS_SOURCE");
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
        use chrono::{Local, TimeZone};
        let now = Local.with_ymd_and_hms(2026, 7, 15, 14, 30, 0).single().unwrap();
        let (start, end) = local_day_window_ms(now);
        assert_eq!(end - start, 86_400_000, "non-DST day must span exactly 24h");
    }

    #[test]
    fn local_day_window_handles_dst_spring_forward() {
        use chrono::{Local, TimeZone};
        // Poland 2026 spring-forward: 2026-03-29 02:00 → 03:00.
        let now = Local.with_ymd_and_hms(2026, 3, 29, 12, 0, 0).single().unwrap();
        let (start, end) = local_day_window_ms(now);
        let span_hours = (end - start) / 3_600_000;
        assert!(
            span_hours == 23 || span_hours == 24,
            "spring-forward span = {span_hours}h; expected 23 (DST) or 24 (no-DST host)"
        );
    }

    #[test]
    fn local_day_window_handles_dst_fall_back() {
        use chrono::{Local, TimeZone};
        let now = Local.with_ymd_and_hms(2026, 10, 25, 12, 0, 0).single().unwrap();
        let (start, end) = local_day_window_ms(now);
        let span_hours = (end - start) / 3_600_000;
        assert!(
            span_hours == 25 || span_hours == 24,
            "fall-back span = {span_hours}h; expected 25 (DST) or 24 (no-DST host)"
        );
    }
}
