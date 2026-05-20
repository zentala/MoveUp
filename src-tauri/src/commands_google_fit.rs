//! Tauri IPC commands for the Google Fit walking-steps integration.
//!
//! Both commands return the same `StepsView` shape so the frontend handles
//! a single contract. "Not configured" is a state, not an error — the
//! view's `configured = false` is the signal to render the setup hint.
//! "Auth revoked" and transient failures are encoded in `error_kind` so
//! the frontend never needs a try/catch around invoke().

use crate::google_fit_models::StepsView;
use crate::google_fit_service::GoogleFitState;
use tauri::State;

/// Read the current cached view (no network).
#[tauri::command]
pub async fn get_steps_today(state: State<'_, GoogleFitState>) -> Result<StepsView, String> {
    Ok(state.inner().clone().view().await)
}

/// Force a fresh API call and return the post-refresh view.
#[tauri::command]
pub async fn refresh_steps_now(state: State<'_, GoogleFitState>) -> Result<StepsView, String> {
    Ok(state.inner().clone().refresh().await)
}
