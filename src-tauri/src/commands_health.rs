//! Tauri IPC commands for the Google Fit health integration.
//!
//! The frontend `StepsWidget` calls these to read cached step counts and
//! trigger on-demand refreshes. Both commands tolerate the "not configured"
//! case by returning structured info rather than errors so the UI can render
//! a calm "connect Google Fit" empty state.

use crate::google_fit_models::StepsSnapshot;
use crate::google_fit_service::GoogleFitState;
use serde::Serialize;
use tauri::State;

/// Wire format the frontend consumes.
///
/// `configured = false` → UI shows the setup prompt.
/// `configured = true, snapshot = None` → UI shows "loading…" until the
/// first successful refresh lands.
#[derive(Debug, Serialize)]
pub struct StepsView {
    pub configured: bool,
    pub snapshot: Option<StepsSnapshot>,
}

/// Read the cached steps snapshot (no network).
#[tauri::command]
pub async fn get_steps_today(state: State<'_, GoogleFitState>) -> Result<StepsView, String> {
    let svc = state.inner().clone();
    let snapshot = svc.snapshot().await;
    Ok(StepsView {
        configured: svc.is_configured(),
        snapshot,
    })
}

/// Force a fresh API call and return the new snapshot.
#[tauri::command]
pub async fn refresh_steps_now(state: State<'_, GoogleFitState>) -> Result<StepsSnapshot, String> {
    state.inner().clone().refresh().await
}
