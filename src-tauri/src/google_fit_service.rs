//! Service layer over `GoogleFitClient`.
//!
//! Owns the in-memory `StepsSnapshot` cache so that:
//!   - UI reads are instant (no per-render network call),
//!   - polling and on-demand refresh share one code path,
//!   - the client is loaded once from env at startup and reused.
//!
//! Thread-safety: state is wrapped in `tokio::sync::Mutex` because refresh
//! is `async`; lock is held only across the cache assignment, not across
//! the network call itself.

use crate::google_fit::{Credentials, GoogleFitClient};
use crate::google_fit_models::StepsSnapshot;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Returns (start_of_today_ms, end_of_today_ms) in the local timezone.
fn local_day_window_ms() -> (i64, i64) {
    use chrono::{Datelike, Local, TimeZone};
    let now = Local::now();
    let start = Local
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
        .single()
        .unwrap_or(now);
    let start_ms = start.timestamp_millis();
    (start_ms, start_ms + 86_400_000)
}

/// Cacheing wrapper around `GoogleFitClient`.
///
/// `client = None` means "Google Fit is not configured" — all reads return
/// `None` and refreshes are no-ops. This is the expected steady state for
/// users who have not gone through the OAuth setup.
pub struct GoogleFitService {
    client: Option<GoogleFitClient>,
    cache: Mutex<Option<StepsSnapshot>>,
}

impl GoogleFitService {
    /// Construct from environment. Always succeeds; absence of credentials
    /// just disables the service.
    pub fn from_env() -> Self {
        let client = Credentials::from_env().map(GoogleFitClient::new);
        Self {
            client,
            cache: Mutex::new(None),
        }
    }

    /// Whether OAuth credentials were present at startup.
    pub fn is_configured(&self) -> bool {
        self.client.is_some()
    }

    /// Return the cached snapshot without hitting the network.
    pub async fn snapshot(&self) -> Option<StepsSnapshot> {
        self.cache.lock().await.clone()
    }

    /// Force a fresh API call, update the cache, and return the new snapshot.
    /// Returns `Err("not configured")` when no credentials were found.
    pub async fn refresh(&self) -> Result<StepsSnapshot, String> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| "google fit: not configured".to_string())?;
        let (start, end) = local_day_window_ms();
        let steps = client.fetch_steps(start, end).await?;
        let snap = StepsSnapshot {
            steps_today: steps,
            fetched_at_ms: chrono::Utc::now().timestamp_millis(),
        };
        *self.cache.lock().await = Some(snap.clone());
        Ok(snap)
    }
}

/// Type alias used by Tauri state.
pub type GoogleFitState = Arc<GoogleFitService>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn unconfigured_service_returns_none_snapshot() {
        std::env::remove_var("GOOGLE_FIT_CLIENT_ID");
        std::env::remove_var("GOOGLE_FIT_CLIENT_SECRET");
        std::env::remove_var("GOOGLE_FIT_REFRESH_TOKEN");
        let svc = GoogleFitService::from_env();
        assert!(!svc.is_configured());
        assert!(svc.snapshot().await.is_none());
    }

    #[tokio::test]
    async fn unconfigured_refresh_returns_not_configured_error() {
        std::env::remove_var("GOOGLE_FIT_CLIENT_ID");
        std::env::remove_var("GOOGLE_FIT_CLIENT_SECRET");
        std::env::remove_var("GOOGLE_FIT_REFRESH_TOKEN");
        let svc = GoogleFitService::from_env();
        let err = svc.refresh().await.unwrap_err();
        assert!(err.contains("not configured"));
    }

    #[test]
    fn local_day_window_spans_24_hours() {
        let (start, end) = local_day_window_ms();
        assert_eq!(end - start, 86_400_000);
    }
}
