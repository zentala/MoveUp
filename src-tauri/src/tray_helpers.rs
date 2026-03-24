//! tray_helpers.rs — Tooltip formatting and today-cache refresh.

use tauri::AppHandle;
use crate::commands::AppState;
use crate::session::DeskState;
use tauri::Manager;

/// Builds tooltip like `"↕ 72.3 cm — Sitting (12:34) +38"` with state-appropriate duration and score.
pub(crate) fn build_tooltip_label(
    desk_height_cm: f32,
    state: &DeskState,
    sitting_secs: i64,
    _standing_secs: i64,
    break_secs: i64,
    daily_score: f32,
) -> String {
    let (state_str, duration_secs) = match state {
        DeskState::Sitting => ("Sitting", sitting_secs),
        DeskState::Standing => ("Standing", break_secs),
        DeskState::Walking => ("Walking", break_secs),
        DeskState::Away => ("Away", 0),
    };
    let score_str = if daily_score >= 0.0 {
        format!(" +{:.0}", daily_score)
    } else {
        format!(" {:.0}", daily_score)
    };
    format!(
        "\u{2195} {:.0} cm \u{2014} {} ({}){}", // ↕ and —
        desk_height_cm, state_str, format_duration(duration_secs), score_str,
    )
}

/// Formats a duration in seconds as `"MM:SS"`.
pub(crate) fn format_duration(secs: i64) -> String {
    let secs = secs.max(0);
    let minutes = secs / 60;
    let seconds = secs % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

/// Refreshes the `today_cache` in AppState from the database.
/// Called on state transitions (not every tick) to keep cache fresh.
pub(crate) fn refresh_today_cache(app: &AppHandle) {
    let app_state = app.state::<AppState>();
    let db_guard = app_state.db.lock().unwrap();
    if let Some(conn) = db_guard.as_ref() {
        if let Ok(mut summary) = crate::db::get_today_summary(conn) {
            let session = app_state.session.lock().unwrap();
            summary.position_changes = session.snapshot().position_changes;
            drop(session);
            drop(db_guard);
            let mut cache = app_state.today_cache.lock().unwrap();
            *cache = summary;
        }
    }
}
