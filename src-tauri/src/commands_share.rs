//! commands_share.rs — IPC command for "Share My Stats" feature.

use tauri::State;

use crate::commands::{ensure_initialized, AppState};
use crate::metrics::MetricEngine;

const SHARE_SITE_URL: &str = "https://desk.labs.zentala.agency";

/// Requests opening one allow-listed social URL through the operating system.
/// Facebook accepts the public site URL; MoveUp copies the personal text first
/// and does not claim that a composer displayed, prefilled, or published anything.
#[tauri::command]
pub fn open_share_destination(destination: String, text: String) -> Result<(), String> {
    let url = share_destination_url(&destination, &text)?;
    dispatch_share_url(&url, open_url_with_shell)
}

fn share_destination_url(destination: &str, text: &str) -> Result<String, String> {
    let site = percent_encode(SHARE_SITE_URL);
    let encoded_text = percent_encode(text);
    match destination {
        "x" => Ok(format!("https://twitter.com/intent/tweet?text={encoded_text}")),
        "reddit" => Ok(format!(
            "https://www.reddit.com/submit?title=My%20desk%20stats%20with%20MoveUp&selftext=true&text={encoded_text}"
        )),
        "facebook" => Ok(format!("https://www.facebook.com/sharer/sharer.php?u={site}")),
        _ => Err("Unsupported share destination".to_string()),
    }
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(byte as char);
        } else {
            use std::fmt::Write as _;
            let _ = write!(encoded, "%{byte:02X}");
        }
    }
    encoded
}

fn dispatch_share_url(
    url: &str,
    open: impl FnOnce(&str) -> Result<(), String>,
) -> Result<(), String> {
    open(url)
}

#[cfg(windows)]
fn open_url_with_shell(url: &str) -> Result<(), String> {
    use windows::core::{w, PCWSTR};
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let target: Vec<u16> = url.encode_utf16().chain(std::iter::once(0)).collect();
    let result = unsafe {
        ShellExecuteW(
            Some(HWND::default()),
            w!("open"),
            PCWSTR(target.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    if result.0 as isize <= 32 {
        Err(format!("Windows could not open the share URL (ShellExecuteW={})", result.0 as isize))
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
fn open_url_with_shell(_url: &str) -> Result<(), String> {
    Err("Opening share destinations is supported on Windows only".to_string())
}

#[cfg(test)]
mod share_destination_tests {
    use super::{dispatch_share_url, format_share_text, share_destination_url};

    #[test]
    fn backend_copy_text_uses_accepted_personal_stats_and_current_site() {
        assert_eq!(
            format_share_text("42%", "7", "30m", 12.4),
            "With MoveUp, I am building a habit of changing position. Here are my desk stats for today:\n🧍 Standing: 42%\n🔄 Position changes: 7\n⏱️ Longest session: 30m\n🏆 Score: +12 points\n\nTracked by MoveUp — desk.labs.zentala.agency"
        );
    }

    #[test]
    fn supported_destinations_use_fixed_https_hosts_and_encode_text() {
        let text = "MoveUp café & stats";
        let x = share_destination_url("x", text).unwrap();
        let reddit = share_destination_url("reddit", text).unwrap();
        let facebook = share_destination_url("facebook", text).unwrap();

        assert!(x.starts_with("https://twitter.com/intent/tweet?text="));
        assert!(x.contains("caf%C3%A9%20%26%20stats"));
        assert!(reddit.starts_with("https://www.reddit.com/submit?"));
        assert!(reddit.contains("caf%C3%A9%20%26%20stats"));
        assert!(!reddit.contains("url="), "Reddit text posts must use the text-post fields only");
        assert_eq!(facebook, "https://www.facebook.com/sharer/sharer.php?u=https%3A%2F%2Fdesk.labs.zentala.agency");
        assert!(share_destination_url("example.com", text).is_err());
    }

    #[test]
    fn opener_dispatch_is_captured_and_propagates_failure_without_os_launch() {
        let url = share_destination_url("x", "synthetic only").unwrap();
        let mut captured = None;
        let result = dispatch_share_url(&url, |target| {
            captured = Some(target.to_string());
            Ok(())
        });
        assert!(result.is_ok());
        assert_eq!(captured.as_deref(), Some(url.as_str()));

        let failed = dispatch_share_url(&url, |_| Err("synthetic opener failure".to_string()));
        assert_eq!(failed.unwrap_err(), "synthetic opener failure");
    }
}

/// IPC command for share text generation.
///
/// Returns formatted share text with today's desk stats.
/// The text is suitable for pasting into social media, chat, or clipboard.
/// Uses emoji for visual appeal and includes a watermark URL.
#[tauri::command]
pub fn get_share_text(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    ensure_initialized(&app, &state)?;

    let (snapshot, ss) = {
        let s = state.session.lock().map_err(|e| format!("session lock: {}", e))?;
        let now = chrono::Utc::now();
        let mut raw = s.state.clone();
        raw.sitting_seconds_total = s.get_live_sitting_seconds_total(now);
        raw.standing_seconds = s.get_live_standing_seconds(now);
        (s.snapshot(), raw)
    };

    let ergo = state.comm_policy.lock()
        .map_err(|e| format!("policy lock: {}", e))?
        .ergo_profile().clone();
    let metrics = MetricEngine::with_defaults().compute_all(&ss, &ergo);

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

    Ok(format_share_text(
        &standing_pct,
        &position_changes,
        &longest_session,
        snapshot.daily_score,
    ))
}

fn format_share_text(standing: &str, changes: &str, longest: &str, score: f32) -> String {
    let score_sign = if score >= 0.0 { "+" } else { "" };
    format!(
        "With MoveUp, I am building a habit of changing position. Here are my desk stats for today:\n\
         \u{1f9cd} Standing: {standing}\n\
         \u{1f504} Position changes: {changes}\n\
         \u{23f1}\u{fe0f} Longest session: {longest}\n\
         \u{1f3c6} Score: {score_sign}{score:.0} points\n\
         \n\
         Tracked by MoveUp \u{2014} desk.labs.zentala.agency"
    )
}
