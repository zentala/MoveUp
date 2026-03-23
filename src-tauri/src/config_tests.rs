//! config_tests.rs — Unit tests for AppConfig defaults, clamping, and serde.

#[cfg(test)]
mod tests {
    use crate::config::AppConfig;

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

        assert_eq!(config.sitting_mm, 720);
        assert_eq!(config.standing_mm, 1050);
    }

    #[test]
    fn test_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.sit_limit_mins, 45);
        assert_eq!(config.stand_limit_mins, 15);
        assert_eq!(config.standing_target_mins, 15);
        assert_eq!(config.stand_max_mins, 90);
        assert!(config.notify_inactivity);
        assert_eq!(config.sitting_mm, 720);
        assert_eq!(config.standing_mm, 1050);
    }

    #[test]
    fn test_config_new_fields_clamping() {
        let config = AppConfig {
            standing_target_mins: 1,
            stand_max_mins: 200,
            ..Default::default()
        }
        .clamped();
        assert_eq!(config.standing_target_mins, 5);
        assert_eq!(config.stand_max_mins, 120);
    }

    #[test]
    fn test_config_serde_roundtrip() {
        let original = AppConfig {
            sit_limit_mins: 50,
            stand_limit_mins: 20,
            standing_target_mins: 25,
            stand_max_mins: 60,
            notify_inactivity: false,
            notify_daily_posture_balance: true,
            notify_praise_halfway: false,
            sitting_mm: 750,
            standing_mm: 1100,
            desk_thickness_mm: 35,
            active_widget: "two-bar".to_string(),
            notification_backend: "popup".to_string(),
            pts_standing_per_min: 2.0,
            pts_session_bonus: 10.0,
            pts_sitting_per_min: -1.0,
            show_welcome_on_startup: false,
            kpi_standing_green_pct: 15.0,
            kpi_standing_yellow_pct: 10.0,
            kpi_changes_green: 1.0,
            kpi_changes_yellow: 0.5,
            kpi_break_yellow_missed: 2,
            kpi_break_red_missed: 3,
            kpi_session_green_mins: 45,
            kpi_session_yellow_mins: 75,
            kpi_early_data_threshold_mins: 30,
        };

        let json = serde_json::to_value(&original).unwrap();
        let restored: AppConfig = serde_json::from_value(json).unwrap();

        assert_eq!(original.sit_limit_mins, restored.sit_limit_mins);
        assert_eq!(original.standing_mm, restored.standing_mm);
        assert_eq!(original.notify_inactivity, restored.notify_inactivity);
        assert_eq!(original.active_widget, restored.active_widget);
        assert_eq!(restored.notification_backend, "popup");
        assert!(!restored.show_welcome_on_startup);
    }

    #[test]
    fn test_notification_backend_default() {
        let config = AppConfig::default();
        assert_eq!(config.active_widget, "one-bar");
        assert_eq!(config.notification_backend, "toast");
    }

    #[test]
    fn test_score_config_defaults() {
        let config = AppConfig::default();
        assert!((config.pts_standing_per_min - 1.0).abs() < f32::EPSILON);
        assert!((config.pts_session_bonus - 5.0).abs() < f32::EPSILON);
        assert!((config.pts_sitting_per_min - (-0.5)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_welcome_defaults_true() {
        let config = AppConfig::default();
        assert!(config.show_welcome_on_startup);
    }

    #[test]
    fn test_welcome_serde_missing_defaults_true() {
        let json = serde_json::json!({ "sit_limit_mins": 30 });
        let config: AppConfig = serde_json::from_value(json).unwrap();
        assert!(config.show_welcome_on_startup);
    }
}
