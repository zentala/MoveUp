//! config_tests.rs — Unit tests for AppConfig defaults, clamping, and serde.

#[cfg(test)]
mod tests {
    use crate::config::AppConfig;

    #[test]
    fn test_config_clamping() {
        let config = AppConfig {
            sitting_mm: 300,
            standing_mm: 1500,
            ..Default::default()
        }
        .clamped();

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

        assert_eq!(config.sitting_mm, 720);
        assert_eq!(config.standing_mm, 1050);
    }

    #[test]
    fn test_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.sitting_mm, 720);
        assert_eq!(config.standing_mm, 1050);
        assert_eq!(config.active_widget, "one-bar");
        assert!(config.show_welcome_on_startup);
    }

    #[test]
    fn test_config_serde_roundtrip() {
        let original = AppConfig {
            sitting_mm: 750,
            standing_mm: 1100,
            desk_thickness_mm: 35,
            active_widget: "two-bar".to_string(),
            timeline_skin: "amber".to_string(),
            show_welcome_on_startup: false,
            show_activity_status: true,
            telemetry_enabled: false,
            telemetry_device_id: "test-uuid".to_string(),
        };

        let json = serde_json::to_value(&original).unwrap();
        let restored: AppConfig = serde_json::from_value(json).unwrap();

        assert_eq!(original.sitting_mm, restored.sitting_mm);
        assert_eq!(original.standing_mm, restored.standing_mm);
        assert_eq!(original.active_widget, restored.active_widget);
        assert!(!restored.show_welcome_on_startup);
    }

    #[test]
    fn test_welcome_defaults_true() {
        let config = AppConfig::default();
        assert!(config.show_welcome_on_startup);
    }

    #[test]
    fn test_welcome_serde_missing_defaults_true() {
        let json = serde_json::json!({ "sitting_mm": 750 });
        let config: AppConfig = serde_json::from_value(json).unwrap();
        assert!(config.show_welcome_on_startup);
    }
}
