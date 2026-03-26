//! telemetry.rs — Anonymous usage telemetry (opt-in only).
//!
//! Collects daily aggregate data and sends to the telemetry endpoint.
//! No personally identifiable information is ever collected.
//! Telemetry is disabled by default and must be explicitly opted in.

use log::{debug, warn};
use serde::Serialize;

use crate::config::AppConfig;
use crate::session_manager::SessionManager;

/// Daily aggregate telemetry payload.
/// Contains only anonymized usage metrics — no PII, no raw sensor data.
#[derive(Debug, Clone, Serialize)]
pub struct TelemetryPayload {
    /// Random UUID generated once per device (not linked to user identity).
    pub device_id: String,
    /// App version string (e.g. "0.3.0").
    pub app_version: String,
    /// Operating system identifier (e.g. "windows").
    pub os: String,
    /// Date of the report (YYYY-MM-DD).
    pub date: String,
    /// Percentage of active time spent standing (0.0–100.0).
    pub standing_pct: f64,
    /// Number of sit/stand position changes today.
    pub position_changes: u32,
    /// Longest continuous sitting session in minutes.
    pub longest_session_mins: u32,
    /// Daily gamification score.
    pub daily_score: f64,
    /// Total minutes the user was active (sitting + standing).
    pub total_active_mins: u32,
}

/// Telemetry API endpoint (placeholder — not yet deployed).
const _TELEMETRY_URL: &str = "https://telemetry.desk.zentala.io/api/report";

/// Builds a telemetry payload from the current session state.
pub fn build_payload(config: &AppConfig, session: &SessionManager) -> TelemetryPayload {
    let state = &session.state;
    let total_secs = state.sitting_seconds_total + state.standing_seconds;
    let standing_pct = if total_secs > 0 {
        (state.standing_seconds as f64 / total_secs as f64) * 100.0
    } else {
        0.0
    };

    TelemetryPayload {
        device_id: config.telemetry_device_id.clone(),
        app_version: crate::APP_VERSION.to_string(),
        os: std::env::consts::OS.to_string(),
        date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        standing_pct,
        position_changes: state.position_changes,
        longest_session_mins: (state.longest_computer_session_secs / 60) as u32,
        daily_score: state.daily_score as f64,
        total_active_mins: (total_secs / 60) as u32,
    }
}

/// Sends telemetry if enabled in config. Errors are logged but never propagated.
///
/// Called from the daily reset logic. If telemetry is disabled, returns immediately.
/// The actual HTTP POST is a TODO — the payload is built and logged but not sent.
pub fn send_telemetry_if_enabled(config: &AppConfig, session: &SessionManager) {
    if !config.telemetry_enabled {
        return;
    }

    let payload = build_payload(config, session);
    debug!("telemetry payload: {:?}", payload);

    // TODO: Send HTTP POST to _TELEMETRY_URL with the JSON payload.
    // Use a non-blocking HTTP client (reqwest or similar). Do NOT add reqwest
    // as a dependency yet — wait until the telemetry backend is deployed.
    // Example (future):
    //   reqwest::Client::new()
    //       .post(_TELEMETRY_URL)
    //       .json(&payload)
    //       .send()
    //       .await
    //       .ok();
    //
    // For now, just log that we would send telemetry.
    match serde_json::to_string(&payload) {
        Ok(json) => debug!("telemetry: would send {}", json),
        Err(e) => warn!("telemetry: failed to serialize payload: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_respects_opt_out() {
        let config = AppConfig {
            telemetry_enabled: false,
            ..AppConfig::default()
        };
        let session = SessionManager::new();
        // Should return immediately without error.
        send_telemetry_if_enabled(&config, &session);
    }

    #[test]
    fn payload_builds_with_defaults() {
        let config = AppConfig {
            telemetry_enabled: true,
            telemetry_device_id: "test-uuid".to_string(),
            ..AppConfig::default()
        };
        let session = SessionManager::new();
        let payload = build_payload(&config, &session);
        assert_eq!(payload.device_id, "test-uuid");
        assert_eq!(payload.os, std::env::consts::OS);
        assert_eq!(payload.standing_pct, 0.0);
        assert_eq!(payload.position_changes, 0);
    }
}
