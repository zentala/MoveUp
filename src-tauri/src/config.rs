//! config.rs — Application configuration and settings persistence.
//!
//! Manages all user-configurable settings via tauri-plugin-store.
//! Provides serde serialization with sensible defaults for missing fields.

use log::warn;
use serde::{Deserialize, Serialize};
use tauri_plugin_store::Store;
use tauri::Runtime;

// ─── Default value helpers ───────────────────────────────────────────────────

fn default_sit_limit() -> u32 {
    45
}

fn default_stand_limit() -> u32 {
    15
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
        let value = serde_json::to_value(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        store.set("app_config", value);
        store.save().map_err(|e| format!("Failed to save store: {}", e))?;
        Ok(())
    }

    /// Clamps all values to valid ranges and validates calibration.
    /// Resets inverted calibration to defaults with a warning.
    pub fn clamped(mut self) -> Self {
        self.sit_limit_mins = self.sit_limit_mins.clamp(10, 90);
        self.stand_limit_mins = self.stand_limit_mins.clamp(5, 60);
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
            notify_inactivity: bool_true(),
            notify_daily_posture_balance: bool_true(),
            notify_praise_halfway: bool_true(),
            sitting_mm: default_sitting_mm(),
            standing_mm: default_standing_mm(),
            desk_thickness_mm: default_thickness_mm(),
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_clamping() {
        let config = AppConfig {
            sit_limit_mins: 999,
            stand_limit_mins: 1,
            sitting_mm: 300,
            standing_mm: 1500,
            ..Default::default()
        }
        .clamped();

        assert_eq!(config.sit_limit_mins, 90);
        assert_eq!(config.stand_limit_mins, 5);
        assert_eq!(config.sitting_mm, 400);
        assert_eq!(config.standing_mm, 1400);
    }

    #[test]
    fn test_config_inverted_calibration_reset() {
        let config = AppConfig {
            sitting_mm: 1100,
            standing_mm: 700,
            ..Default::default()
        }
        .clamped();

        assert_eq!(config.sitting_mm, default_sitting_mm());
        assert_eq!(config.standing_mm, default_standing_mm());
    }

    #[test]
    fn test_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.sit_limit_mins, 45);
        assert_eq!(config.stand_limit_mins, 15);
        assert!(config.notify_inactivity);
        assert_eq!(config.sitting_mm, 720);
        assert_eq!(config.standing_mm, 1050);
    }

    #[test]
    fn test_config_serde_roundtrip() {
        let original = AppConfig {
            sit_limit_mins: 50,
            stand_limit_mins: 20,
            notify_inactivity: false,
            notify_daily_posture_balance: true,
            notify_praise_halfway: false,
            sitting_mm: 750,
            standing_mm: 1100,
            desk_thickness_mm: 35,
        };

        let json = serde_json::to_value(&original).unwrap();
        let restored: AppConfig = serde_json::from_value(json).unwrap();

        assert_eq!(original.sit_limit_mins, restored.sit_limit_mins);
        assert_eq!(original.standing_mm, restored.standing_mm);
        assert_eq!(original.notify_inactivity, restored.notify_inactivity);
    }
}
