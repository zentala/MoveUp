//! config.rs — Application configuration and settings persistence.
//!
//! Manages hardware calibration and UI preferences via tauri-plugin-store.
//! Ergonomic limits, scoring, KPI thresholds, and notification settings
//! have moved to [`ErgonomicProfile`] and [`CommunicationProfile`].

use log::warn;
use serde::{Deserialize, Serialize};
use tauri::Runtime;
use tauri_plugin_store::Store;

fn bool_true() -> bool { true }
fn default_sitting_mm() -> i32 { 720 }
fn default_standing_mm() -> i32 { 1050 }
fn default_thickness_mm() -> i32 { 30 }
fn default_active_widget() -> String { "one-bar".to_string() }
fn bool_false() -> bool { false }
fn default_empty_string() -> String { String::new() }

// ─── AppConfig ───────────────────────────────────────────────────────────────

/// Hardware calibration, UI preferences, and privacy settings.
///
/// Ergonomic limits, scoring, KPI thresholds live in [`ErgonomicProfile`].
/// Notification flags and backend live in [`CommunicationProfile`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    // Calibration
    #[serde(default = "default_sitting_mm")]
    pub sitting_mm: i32,
    #[serde(default = "default_standing_mm")]
    pub standing_mm: i32,
    #[serde(default = "default_thickness_mm")]
    pub desk_thickness_mm: i32,
    /// Which widget layout is active (validated by TS widget registry).
    #[serde(default = "default_active_widget")]
    pub active_widget: String,
    /// Show welcome popup on startup. Set to false after first dismiss.
    #[serde(default = "bool_true")]
    pub show_welcome_on_startup: bool,
    /// Whether anonymous telemetry is enabled. Default: false (opt-in).
    #[serde(default = "bool_false")]
    pub telemetry_enabled: bool,
    /// Random device ID for telemetry (UUID v4). Generated once on first load.
    #[serde(default = "default_empty_string")]
    pub telemetry_device_id: String,
}

impl AppConfig {
    /// Loads config from the store, returning defaults if missing or corrupt.
    /// Generates a stable telemetry device ID on first load if not yet set.
    pub fn load<R: Runtime>(store: &Store<R>) -> Self {
        let mut config = match store.get("app_config") {
            Some(serde_json::Value::Object(map)) => {
                serde_json::from_value::<AppConfig>(serde_json::Value::Object(map))
                    .unwrap_or_default()
                    .clamped()
            }
            _ => Self::default().clamped(),
        };
        if config.telemetry_device_id.is_empty() {
            config.telemetry_device_id = uuid::Uuid::new_v4().to_string();
        }
        config
    }

    /// Saves this config to the store.
    pub fn save<R: Runtime>(&self, store: &Store<R>) -> Result<(), String> {
        let value =
            serde_json::to_value(self).map_err(|e| format!("Failed to serialize config: {}", e))?;
        store.set("app_config", value);
        store
            .save()
            .map_err(|e| format!("Failed to save store: {}", e))?;
        Ok(())
    }

    /// Clamps all values to valid ranges and validates calibration.
    /// Resets inverted calibration to defaults with a warning.
    pub fn clamped(mut self) -> Self {
        self.sitting_mm = self.sitting_mm.clamp(400, 900);
        self.standing_mm = self.standing_mm.clamp(900, 1400);

        // Validate non-inverted calibration.
        if self.sitting_mm >= self.standing_mm {
            warn!("calibration inverted — reset to defaults");
            self.sitting_mm = default_sitting_mm();
            self.standing_mm = default_standing_mm();
        }

        self
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            sitting_mm: default_sitting_mm(),
            standing_mm: default_standing_mm(),
            desk_thickness_mm: default_thickness_mm(),
            active_widget: default_active_widget(),
            show_welcome_on_startup: bool_true(),
            telemetry_enabled: bool_false(),
            telemetry_device_id: default_empty_string(),
        }
    }
}

// Tests moved to config_tests.rs to stay under 250-line limit.
