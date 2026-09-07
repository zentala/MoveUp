//! Tauri IPC commands for health data (E021).
//!
//! Replaces the Google-Fit-specific `commands_google_fit.rs`: the frontend
//! asks for *health*, and the [`HealthAggregator`](crate::health_source::HealthAggregator)
//! decides which sources answered.
//!
//! Both commands return the same `HealthView` shape so the frontend
//! handles a single contract. "Not configured" is a state, not an error —
//! `configured = false` is the signal to render the setup hint. Auth and
//! transient failures are encoded in `error_kind`, so the frontend never
//! needs a try/catch around `invoke`.

use crate::health_models::HealthView;
use crate::health_source::HealthState;
use tauri::State;

/// Read the current merged view across all sources (no network).
#[tauri::command]
pub async fn get_health_today(state: State<'_, HealthState>) -> Result<HealthView, String> {
    Ok(state.inner().clone().view().await)
}

/// Refresh every source and return the post-refresh merged view.
#[tauri::command]
pub async fn refresh_health_now(state: State<'_, HealthState>) -> Result<HealthView, String> {
    Ok(state.inner().clone().refresh().await)
}
