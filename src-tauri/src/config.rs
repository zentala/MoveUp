//! config.rs — Application configuration and settings persistence.
//!
//! Manages all user-configurable settings via tauri-plugin-store.
//! Provides serde serialization with sensible defaults for missing fields.

use log::warn;
use serde::{Deserialize, Serialize};
use tauri::Runtime;
use tauri_plugin_store::Store;

// ─── Default value helpers ───────────────────────────────────────────────────

fn default_sit_limit() -> u32 {
    45
}

fn default_stand_limit() -> u32 {
    15
}

fn default_standing_target() -> u32 {
    15
}

fn default_stand_max() -> u32 {
    90
}

fn bool_true() -> bool {
    true
}

fn default_sitting_mm() -> i32 {
    720
}

fn default_standing_mm() -> i32 {
    1050
}

fn default_thickness_mm() -> i32 {
    30
}

fn default_active_widget() -> String {
    "one-bar".to_string()
}

fn default_notification_backend() -> String {
    "toast".to_string()
}

fn default_pts_standing_per_min() -> f32 {
    1.0
}

fn default_pts_session_bonus() -> f32 {
    5.0
}

fn default_pts_sitting_per_min() -> f32 {
    -0.5
}

// ─── AppConfig ───────────────────────────────────────────────────────────────

/// All user-configurable settings.
/// Every field has #[serde(default)] so a corrupt or missing store entry
/// falls back to defaults cleanly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_sit_limit")]
    pub sit_limit_mins: u32,
    #[serde(default = "default_stand_limit")]
    pub stand_limit_mins: u32,
    /// Standing target: duration for a "complete" standing session (gold bar fills, +5 pts bonus).
    /// Default 15 min. Gold bar fills 0->100% over this duration.
    #[serde(default = "default_standing_target")]
    pub standing_target_mins: u32,
    /// Maximum continuous standing before "consider sitting" nudge.
    /// Default 90 min. Semantic: protective ceiling, not a goal.
    #[serde(default = "default_stand_max")]
    pub stand_max_mins: u32,
    #[serde(default = "bool_true")]
    pub notify_inactivity: bool,
    #[serde(default = "bool_true")]
    pub notify_daily_posture_balance: bool,
    #[serde(default = "bool_true")]
    pub notify_praise_halfway: bool,
    // Calibration (formerly only in SessionManager, now persisted):
    #[serde(default = "default_sitting_mm")]
    pub sitting_mm: i32,
    #[serde(default = "default_standing_mm")]
    pub standing_mm: i32,
    #[serde(default = "default_thickness_mm")]
    pub desk_thickness_mm: i32,
    /// Which widget layout is active (validated by TS widget registry).
    #[serde(default = "default_active_widget")]
    pub active_widget: String,
    /// Notification backend: "toast" (native), "popup" (WinAPI), or "both".
    #[serde(default = "default_notification_backend")]
    pub notification_backend: String,
    /// Points awarded per minute of standing. Default: 1.0.
    #[serde(default = "default_pts_standing_per_min")]
    pub pts_standing_per_min: f32,
    /// Bonus points awarded when standing_target_mins is reached (per lap). Default: 5.0.
    #[serde(default = "default_pts_session_bonus")]
    pub pts_session_bonus: f32,
    /// Points per minute of sitting (negative = penalty). Default: -0.5.
    #[serde(default = "default_pts_sitting_per_min")]
    pub pts_sitting_per_min: f32,
}

impl AppConfig {
    /// Loads config from the store, returning defaults if missing or corrupt.
    pub fn load<R: Runtime>(store: &Store<R>) -> Self {
        match store.get("app_config") {
            Some(serde_json::Value::Object(map)) => {
                serde_json::from_value::<AppConfig>(serde_json::Value::Object(map))
                    .unwrap_or_default()
                    .clamped()
            }
            _ => Self::default().clamped(),
        }
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
        self.sit_limit_mins = self.sit_limit_mins.clamp(10, 90);
        self.stand_limit_mins = self.stand_limit_mins.clamp(5, 60);
        self.standing_target_mins = self.standing_target_mins.clamp(5, 60);
        self.stand_max_mins = self.stand_max_mins.clamp(30, 120);
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
            sit_limit_mins: default_sit_limit(),
            stand_limit_mins: default_stand_limit(),
            standing_target_mins: default_standing_target(),
            stand_max_mins: default_stand_max(),
            notify_inactivity: bool_true(),
            notify_daily_posture_balance: bool_true(),
            notify_praise_halfway: bool_true(),
            sitting_mm: default_sitting_mm(),
            standing_mm: default_standing_mm(),
            desk_thickness_mm: default_thickness_mm(),
            active_widget: default_active_widget(),
            notification_backend: default_notification_backend(),
            pts_standing_per_min: default_pts_standing_per_min(),
            pts_session_bonus: default_pts_session_bonus(),
            pts_sitting_per_min: default_pts_sitting_per_min(),
        }
    }
}

// Tests moved to config_tests.rs to stay under 250-line limit.
