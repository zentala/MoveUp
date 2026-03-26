//! commands_share.rs — IPC command for "Share My Stats" feature.

use tauri::State;

use crate::commands::{ensure_initialized, AppState};
use crate::metrics::MetricEngine;

/// Returns formatted share text with today's desk stats.
///
/// The text is suitable for pasting into social media, chat, or clipboard.
/// Uses emoji for visual appeal and includes a watermark URL.
#[tauri::command]
pub fn get_share_text(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    ensure_initialized(&app, &state)?;

    let (snapshot, ss) = {
        let s = state.session.lock().unwrap();
        let now = chrono::Utc::now();
        let mut raw = s.state.clone();
        raw.sitting_seconds_total = s.get_live_sitting_seconds_total(now);
        raw.standing_seconds = s.get_live_standing_seconds(now);
        (s.snapshot(), raw)
    };

    let cfg = state.config.lock().unwrap();
    let cfg = cfg.as_ref().ok_or("Config not loaded")?;
    let metrics = MetricEngine::with_defaults().compute_all(&ss, cfg);

    let mut standing_pct = "—".to_string();
    let mut position_changes = "—".to_string();
    let mut longest_session = "—".to_string();

    for m in &metrics {
        match m.id.as_str() {
            "standing_pct" => standing_pct = m.result.display.clone(),
            "position_rate" => {
                position_changes =
                    format!("{}", snapshot.position_changes);
            }
            "longest_session" => longest_session = m.result.display.clone(),
            _ => {}
        }
    }

    let score = snapshot.daily_score;
    let score_sign = if score >= 0.0 { "+" } else { "" };

    let text = format!(
        "My desk stats today:\n\
         \u{1f9cd} Standing: {}\n\
         \u{1f504} Position changes: {}\n\
         \u{23f1}\u{fe0f} Longest session: {}\n\
         \u{1f3c6} Score: {}{:.0} points\n\
         \n\
         Tracked by zntlDesk \u{2014} desk.zentala.io",
        standing_pct, position_changes, longest_session,
        score_sign, score,
    );

    Ok(text)
}
