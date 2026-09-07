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
            notify_webhook_enabled: true,
            notify_webhook_url: Some("https://ntfy.sh/desk".to_string()),
            voice_ai_model: Some("openai/gpt-4o-mini".to_string()),
            relay_enabled: false,
            relay_url: crate::relay_auth::RELAY_DEFAULT_URL.to_string(),
            remote_lan_enabled: true,
        };

        let json = serde_json::to_value(&original).unwrap();
        let restored: AppConfig = serde_json::from_value(json).unwrap();

        assert_eq!(original.sitting_mm, restored.sitting_mm);
        assert_eq!(original.standing_mm, restored.standing_mm);
        assert_eq!(original.active_widget, restored.active_widget);
        assert!(!restored.show_welcome_on_startup);
        assert!(restored.notify_webhook_enabled);
        assert_eq!(
            restored.notify_webhook_url.as_deref(),
            Some("https://ntfy.sh/desk")
        );
        assert_eq!(
            restored.voice_ai_model.as_deref(),
            Some("openai/gpt-4o-mini")
        );
    }

    /// A config saved before E021 has no `voice_ai_model`; loading it must
    /// leave the field unset so the cheap default applies, not fail the parse.
    #[test]
    fn test_voice_ai_model_defaults_to_none_when_absent() {
        let json = serde_json::json!({ "sitting_mm": 750 });
        let config: AppConfig = serde_json::from_value(json).unwrap();
        assert_eq!(config.voice_ai_model, None);
        assert_eq!(AppConfig::default().voice_ai_model, None);
    }

    /// A config saved before E021 has neither webhook field; loading it must
    /// leave the webhook off rather than fail the whole config parse.
    #[test]
    fn test_webhook_defaults_off_when_absent() {
        let json = serde_json::json!({ "sitting_mm": 750 });
        let config: AppConfig = serde_json::from_value(json).unwrap();
        assert!(!config.notify_webhook_enabled);
        assert_eq!(config.notify_webhook_url, None);
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

    /// A config written before E022 has none of the three relay fields.
    /// Loading it must leave the relay off, the URL at the shipped default,
    /// and — the one that would silently take a feature away — the LAN
    /// display still ON.
    #[test]
    fn test_e022_relay_fields_default_when_absent() {
        let json = serde_json::json!({ "sitting_mm": 750 });
        let config: AppConfig = serde_json::from_value(json).unwrap();
        assert!(!config.relay_enabled, "the relay is opt-in");
        assert_eq!(config.relay_url, crate::relay_auth::RELAY_DEFAULT_URL);
        assert!(
            config.remote_lan_enabled,
            "an older config must not read as 'LAN turned off'"
        );
    }

    /// An empty URL is a cleared field, not an instruction to connect to
    /// nothing — clamping restores the default rather than leaving the client
    /// pointed at "".
    #[test]
    fn test_e022_empty_relay_url_clamps_to_the_default() {
        let config = AppConfig {
            relay_url: "   ".to_string(),
            ..Default::default()
        }
        .clamped();
        assert_eq!(config.relay_url, crate::relay_auth::RELAY_DEFAULT_URL);

        let custom = AppConfig {
            relay_url: "https://staging.example/".to_string(),
            ..Default::default()
        }
        .clamped();
        assert_eq!(custom.relay_url, "https://staging.example");
    }

    #[test]
    fn test_e022_relay_fields_survive_a_save_load_round_trip() {
        let config = AppConfig {
            relay_enabled: true,
            relay_url: "https://staging.example".to_string(),
            remote_lan_enabled: false,
            ..Default::default()
        };
        let restored: AppConfig =
            serde_json::from_value(serde_json::to_value(&config).unwrap()).unwrap();
        assert!(restored.relay_enabled);
        assert_eq!(restored.relay_url, "https://staging.example");
        assert!(!restored.remote_lan_enabled);
    }

    /// `save_settings` fires on every settings edit. Only the two fields the
    /// relay client actually reads may restart it — reconnecting because the
    /// overlay height moved would drop every live viewer for nothing.
    #[test]
    fn test_e022_only_relay_fields_restart_the_client() {
        use crate::commands_config::relay_settings_changed;

        let base = AppConfig::default();

        assert!(
            relay_settings_changed(None, &base),
            "no previous config means sync, not skip"
        );
        assert!(!relay_settings_changed(Some(&base), &base));

        let unrelated = AppConfig { sitting_mm: 700, ..base.clone() };
        assert!(!relay_settings_changed(Some(&base), &unrelated));

        let toggled = AppConfig { relay_enabled: true, ..base.clone() };
        assert!(relay_settings_changed(Some(&base), &toggled));

        let moved = AppConfig { relay_url: "https://staging.example".into(), ..base.clone() };
        assert!(relay_settings_changed(Some(&base), &moved));
    }
}
