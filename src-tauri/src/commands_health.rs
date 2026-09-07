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

use crate::commands::AppState;
use crate::db_voice_notes::VoiceNoteRow;
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

/// Voice notes dictated on `day` (local `YYYY-MM-DD`), oldest first (E021-T06).
///
/// Lives here rather than in a module of its own because voice is one more
/// thing the phone reports about the body's day, alongside steps and heart
/// rate — the same reason the route sits next to the health inlet.
///
/// An uninitialised database is an error, not an empty list: "no notes today"
/// and "the database is not open" must not look alike to the caller.
#[tauri::command]
pub fn list_voice_notes(state: State<'_, AppState>, day: String) -> Result<Vec<VoiceNoteRow>, String> {
    let guard = state.db.lock().unwrap_or_else(|e| e.into_inner());
    let conn = guard
        .as_ref()
        .ok_or_else(|| "database not initialized".to_string())?;
    crate::db_voice_notes::list_voice_notes(conn, &day).map_err(|e| e.to_string())
}
