//! commands_profiles_tests.rs — Tests for profile ID validation and helpers.

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::commands_profiles::{
        active_profiles_snapshot, duplicate_profile, switch_communication_profile_by_name, switch_ergonomic_profile_by_name,
        validate_profile_id,
    };
    use crate::communication_policy::CommunicationPolicy;
    use crate::communication_profile::CommunicationProfile;
    use crate::ergonomic_profile::ErgonomicProfile;
    use crate::profile_loader::list_profiles;
    use crate::session::SessionManager;

    // ── validate_profile_id ────────────────────────────────────────────────

    #[test]
    fn accepts_simple_id() {
        assert!(validate_profile_id("default").is_ok());
    }

    #[test]
    fn accepts_dashes() {
        assert!(validate_profile_id("my-profile").is_ok());
    }

    #[test]
    fn accepts_underscores() {
        assert!(validate_profile_id("test_123").is_ok());
    }

    #[test]
    fn accepts_single_char() {
        assert!(validate_profile_id("a").is_ok());
    }

    #[test]
    fn rejects_empty() {
        assert!(validate_profile_id("").is_err());
    }

    #[test]
    fn rejects_path_traversal() {
        assert!(validate_profile_id("../etc/passwd").is_err());
    }

    #[test]
    fn rejects_forward_slash() {
        assert!(validate_profile_id("foo/bar").is_err());
    }

    #[test]
    fn rejects_backslash() {
        assert!(validate_profile_id(r"foo\bar").is_err());
    }

    #[test]
    fn rejects_spaces() {
        assert!(validate_profile_id("has spaces").is_err());
    }

    #[test]
    fn rejects_dots() {
        assert!(validate_profile_id("file.json").is_err());
    }

    #[test]
    fn rejects_special_chars() {
        assert!(validate_profile_id("test!@#").is_err());
    }

    // ── list_profiles ──────────────────────────────────────────────────────

    #[test]
    fn list_profiles_empty_for_missing_dir() {
        let result = list_profiles(std::path::Path::new("/nonexistent/dir/abc123"));
        assert!(result.is_empty());
    }

    #[test]
    fn list_profiles_finds_json_only() {
        let dir = std::env::temp_dir().join("desk_test_profiles_list");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("a.json"), "{}").unwrap();
        std::fs::write(dir.join("b.json"), "{}").unwrap();
        std::fs::write(dir.join("readme.txt"), "hi").unwrap();

        let result = list_profiles(&dir);
        let names: Vec<_> = result.iter().filter_map(|p| {
            p.file_name().and_then(|n| n.to_str()).map(String::from)
        }).collect();

        assert!(names.contains(&"a.json".to_string()));
        assert!(names.contains(&"b.json".to_string()));
        assert!(!names.contains(&"readme.txt".to_string()));

        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── switch_*_by_name (the AppHandle-free half, E022-T07) ───────────────

    fn policy() -> Mutex<CommunicationPolicy> {
        Mutex::new(CommunicationPolicy::new(
            CommunicationProfile::default(),
            ErgonomicProfile::default(),
        ))
    }

    /// `load_profile` falls back to `Default` for a file it cannot read, so
    /// without the existence check a switch to a profile the desk does not
    /// have would report success and quietly reset the user to the defaults.
    #[test]
    fn switch_by_name_fails_for_a_missing_profile_file() {
        let tmp = tempfile::TempDir::new().unwrap();
        let err = switch_communication_profile_by_name(tmp.path(), &policy(), "gentle")
            .expect_err("a profile that is not installed must not switch");
        assert!(err.contains("not found"), "was: {err}");
    }

    fn write_communication_profile(
        app_data_dir: &std::path::Path,
        id: &str,
        contents: &str,
    ) {
        let dir = app_data_dir.join("profiles").join("communication");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{id}.json")), contents).unwrap();
    }

    fn selected_policy() -> Mutex<CommunicationPolicy> {
        let mut selected = CommunicationProfile::default();
        selected.id = "selected".into();
        selected.name = "Keep this policy".into();
        Mutex::new(CommunicationPolicy::new(selected, ErgonomicProfile::default()))
    }

    #[test]
    fn runtime_snapshot_reports_session_override_and_current_live_policy_step() {
        let tmp = tempfile::TempDir::new().unwrap();
        let mut session = SessionManager::new();
        session.limits.sitting_secs = 40 * 60;
        session.state.session_limit_secs = 55 * 60;
        session.state.stand_limit_secs = 12 * 60;

        let mut comm = CommunicationProfile::default();
        comm.id = "live-remote-choice".into();
        comm.snooze.durations_mins = vec![4, 11, 23];
        comm.snooze.tone_shift_after_dismisses = 2;
        let policy = selected_policy();
        write_communication_profile(
            tmp.path(),
            "live-remote-choice",
            &serde_json::to_string(&comm).unwrap(),
        );
        switch_communication_profile_by_name(tmp.path(), &policy, "live-remote-choice").unwrap();
        let mut policy = policy.lock().unwrap();
        policy.dismiss_at(std::time::Instant::now());

        let snapshot = active_profiles_snapshot(&session, &policy);
        assert_eq!(snapshot.communication_id, "live-remote-choice");
        assert_eq!(snapshot.session_limit_secs, 55 * 60);
        assert_eq!(snapshot.effective_limits.sitting_secs, 40 * 60);
        assert_eq!(snapshot.stand_limit_secs, 12 * 60);
        assert_eq!(snapshot.dismiss_durations_mins, vec![4, 11, 23]);
        assert_eq!(snapshot.current_dismiss_minutes, 11);
        assert_eq!(snapshot.snooze_minutes, Some(11));
        assert_eq!(snapshot.tone_shift_after_dismisses, 2);
        assert_eq!(active_profiles_snapshot(&session, &policy).current_dismiss_minutes, 11,
            "reading the runtime summary must not advance the policy cursor");
    }

    #[test]
    fn runtime_snapshot_matches_snooze_command_validation_without_changing_dismiss_value() {
        let session = SessionManager::new();
        for (configured, expected_dismiss, expected_snooze) in [
            (0, 0, None),
            (u16::MAX as u32 + 1, u16::MAX as u32 + 1, None),
            (17, 17, Some(17)),
        ] {
            let mut comm = CommunicationProfile::default();
            comm.snooze.durations_mins = vec![configured];
            let policy = CommunicationPolicy::new(comm, ErgonomicProfile::default());
            let snapshot = active_profiles_snapshot(&session, &policy);
            assert_eq!(snapshot.current_dismiss_minutes, expected_dismiss);
            assert_eq!(snapshot.snooze_minutes, expected_snooze);
        }
    }

    #[test]
    fn communication_switch_rejects_invalid_channel_and_preserves_active_policy() {
        let tmp = tempfile::TempDir::new().unwrap();
        let mut invalid = serde_json::to_value(CommunicationProfile::default()).unwrap();
        invalid["id"] = "invalid".into();
        invalid["notification_backend"] = "mystery".into();
        write_communication_profile(tmp.path(), "invalid", &invalid.to_string());
        let policy = selected_policy();

        let error = switch_communication_profile_by_name(tmp.path(), &policy, "invalid")
            .expect_err("an invalid channel backend must not activate");

        assert!(error.contains("notification_backend"), "was: {error}");
        let active = policy.lock().unwrap();
        assert_eq!(active.comm_profile().id, "selected");
        assert_eq!(active.comm_profile().name, "Keep this policy");
    }

    #[test]
    fn communication_switch_rejects_malformed_json_and_preserves_active_policy() {
        let tmp = tempfile::TempDir::new().unwrap();
        write_communication_profile(tmp.path(), "malformed", "{\"id\":");
        let policy = selected_policy();

        let error = switch_communication_profile_by_name(tmp.path(), &policy, "malformed")
            .expect_err("malformed profile JSON must not activate defaults");

        assert!(error.contains("parse"), "was: {error}");
        let active = policy.lock().unwrap();
        assert_eq!(active.comm_profile().id, "selected");
        assert_eq!(active.comm_profile().name, "Keep this policy");
    }

    #[test]
    fn communication_switch_activates_valid_profile() {
        let tmp = tempfile::TempDir::new().unwrap();
        let mut valid = CommunicationProfile::default();
        valid.id = "valid".into();
        valid.name = "Valid profile".into();
        write_communication_profile(
            tmp.path(),
            "valid",
            &serde_json::to_string(&valid).unwrap(),
        );
        let policy = selected_policy();

        switch_communication_profile_by_name(tmp.path(), &policy, "valid").unwrap();

        assert_eq!(policy.lock().unwrap().comm_profile().id, "valid");
        assert_eq!(policy.lock().unwrap().comm_profile().name, "Valid profile");
    }

    #[test]
    fn communication_switch_preserves_legacy_profile_channels_without_popup_fields() {
        let tmp = tempfile::TempDir::new().unwrap();
        let mut legacy = serde_json::to_value(CommunicationProfile::default()).unwrap();
        legacy["id"] = "legacy".into();
        legacy["name"] = "Legacy profile".into();
        legacy["baseline"]["sitting"]["tray"] = "yellow".into();
        legacy["baseline"]["sitting"]["overlay"] = "hidden".into();
        legacy["baseline"]["sitting"].as_object_mut().unwrap().remove("popup");
        legacy["baseline"]["standing"]["tray"] = "red".into();
        legacy["baseline"]["standing"]["overlay"] = "yellow".into();
        legacy["baseline"]["standing"].as_object_mut().unwrap().remove("popup");
        write_communication_profile(tmp.path(), "legacy", &legacy.to_string());
        let policy = selected_policy();

        switch_communication_profile_by_name(tmp.path(), &policy, "legacy").unwrap();

        let active = policy.lock().unwrap();
        assert_eq!(active.comm_profile().id, "legacy");
        assert_eq!(active.comm_profile().baseline.sitting.tray, "yellow");
        assert_eq!(active.comm_profile().baseline.sitting.overlay, "hidden");
        assert_eq!(active.comm_profile().baseline.sitting.popup, "neutral");
        assert_eq!(active.comm_profile().baseline.standing.tray, "red");
        assert_eq!(active.comm_profile().baseline.standing.overlay, "yellow");
        assert_eq!(active.comm_profile().baseline.standing.popup, "neutral");
    }

    #[test]
    fn duplicate_legacy_silent_stays_silent_through_resolver_and_dispatch() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path().join("profiles").join("communication");
        std::fs::create_dir_all(&dir).unwrap();
        let mut legacy_silent: serde_json::Value =
            serde_json::from_str(include_str!("../profiles/communication/silent.json")).unwrap();
        legacy_silent["id"] = "quiet-custom-id".into();
        legacy_silent["name"] = "Quiet before duplication".into();
        legacy_silent.as_object_mut().unwrap().remove("notifications_enabled");
        let source = dir.join("quiet-custom-id.json");
        std::fs::write(&source, serde_json::to_string(&legacy_silent).unwrap()).unwrap();

        let duplicate_id = duplicate_profile(source.to_string_lossy().into_owned(), "Silent copy".into())
            .expect("legacy Silent profile should duplicate");
        let policy = selected_policy();
        switch_communication_profile_by_name(tmp.path(), &policy, &duplicate_id).unwrap();
        let policy = policy.lock().unwrap();
        assert!(!policy.comm_profile().notifications_enabled);

        let now = chrono::Utc::now();
        let intents = policy.resolve_session_notifications(
            &crate::communication_policy::SessionNotificationFacts {
                state: crate::session_types::DeskState::Standing,
                now,
                observation_fresh: true,
                last_position_change_at: Some(now - chrono::Duration::hours(2)),
                sitting_seconds_total: 7200,
                standing_seconds: 7200,
                standing_target_secs: 3600,
                inactivity_fired: false,
                posture_balance_fired: false,
                standing_target_fired: false,
                praise_fired: false,
            },
            true,
        );
        let mut logged = Vec::new();
        let mut dispatched_channels = 0;
        crate::notification_service::NotificationService::dispatch_with(
            &intents,
            |line| logged.push(line.to_string()),
            |channels, _, _| dispatched_channels += channels.len(),
        );

        assert!(intents.is_empty(), "legacy Silent must resolve no intents: {intents:?}");
        assert!(logged.is_empty(), "legacy Silent must log nothing: {logged:?}");
        assert_eq!(dispatched_channels, 0);
    }

    #[test]
    fn legacy_silent_visual_and_description_edits_remain_silent_at_dispatch() {
        for (id, field) in [
            ("legacy-description", "description"),
            ("legacy-timeline", "timeline"),
            ("legacy-overlay", "overlay"),
        ] {
            let tmp = tempfile::TempDir::new().unwrap();
            let mut legacy: serde_json::Value =
                serde_json::from_str(include_str!("../profiles/communication/silent.json")).unwrap();
            legacy.as_object_mut().unwrap().remove("notifications_enabled");
            legacy["id"] = id.into();
            legacy["name"] = format!("Customized {field}").into();
            match field {
                "description" => legacy["description"] = "A custom description".into(),
                "timeline" => legacy["timeline"]["sitting_within_limit"] = "blue".into(),
                "overlay" => legacy["standing_overlay"]["bar_color"] = "#123456".into(),
                _ => unreachable!(),
            }
            write_communication_profile(tmp.path(), id, &legacy.to_string());
            let policy = selected_policy();

            switch_communication_profile_by_name(tmp.path(), &policy, id).unwrap();
            let policy = policy.lock().unwrap();
            assert!(
                !policy.comm_profile().notifications_enabled,
                "legacy Silent {field} edit must not reenable notifications",
            );
            let now = chrono::Utc::now();
            let intents = policy.resolve_session_notifications(
                &crate::communication_policy::SessionNotificationFacts {
                    state: crate::session_types::DeskState::Standing,
                    now,
                    observation_fresh: true,
                    last_position_change_at: None,
                    sitting_seconds_total: 0,
                    standing_seconds: 60,
                    standing_target_secs: 60,
                    inactivity_fired: false,
                    posture_balance_fired: false,
                    standing_target_fired: false,
                    praise_fired: false,
                },
                true,
            );
            let mut logged = Vec::new();
            let mut dispatched_channels = 0;
            crate::notification_service::NotificationService::dispatch_with(
                &intents,
                |line| logged.push(line.to_string()),
                |channels, _, _| dispatched_channels += channels.len(),
            );

            assert!(intents.is_empty(), "an eligible standing-target event must still be silenced");
            assert!(logged.is_empty());
            assert_eq!(dispatched_channels, 0);
        }
    }

    #[test]
    fn duplicate_profile_preserves_explicit_notifications_enabled_value() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path().join("profiles").join("communication");
        std::fs::create_dir_all(&dir).unwrap();
        let mut explicit: serde_json::Value =
            serde_json::from_str(include_str!("../profiles/communication/silent.json")).unwrap();
        explicit["notifications_enabled"] = true.into();
        let source = dir.join("explicit.json");
        std::fs::write(&source, serde_json::to_string(&explicit).unwrap()).unwrap();

        let duplicate_id = duplicate_profile(source.to_string_lossy().into_owned(), "Explicit copy".into())
            .expect("profile with explicit field should duplicate");
        let policy = selected_policy();
        switch_communication_profile_by_name(tmp.path(), &policy, &duplicate_id).unwrap();

        assert!(policy.lock().unwrap().comm_profile().notifications_enabled);
    }

    #[test]
    fn silent_id_and_name_do_not_disable_a_different_legacy_profile() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path().join("profiles").join("communication");
        std::fs::create_dir_all(&dir).unwrap();
        let mut custom: serde_json::Value =
            serde_json::to_value(CommunicationProfile::default()).unwrap();
        custom["id"] = "silent".into();
        custom["name"] = "Silent".into();
        custom.as_object_mut().unwrap().remove("notifications_enabled");
        let source = dir.join("silent.json");
        std::fs::write(&source, serde_json::to_string(&custom).unwrap()).unwrap();

        let duplicate_id = duplicate_profile(source.to_string_lossy().into_owned(), "Custom copy".into())
            .expect("custom profile with a Silent identity should duplicate");
        let policy = selected_policy();
        switch_communication_profile_by_name(tmp.path(), &policy, &duplicate_id).unwrap();

        assert!(policy.lock().unwrap().comm_profile().notifications_enabled);
    }

    #[test]
    fn switch_by_name_rejects_a_traversing_id_before_touching_the_disk() {
        let tmp = tempfile::TempDir::new().unwrap();
        assert!(switch_ergonomic_profile_by_name(
            tmp.path(),
            &policy(),
            &Mutex::new(SessionManager::new()),
            "../../secrets",
        )
        .is_err());
    }

    /// The ergonomic switch owns two writes — the policy and the engine's
    /// limits — and a caller that got only one would be silently half-applied.
    #[test]
    fn switch_ergonomic_by_name_updates_both_the_policy_and_the_session_limits() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path().join("profiles").join("ergonomic");
        std::fs::create_dir_all(&dir).unwrap();
        let mut p = ErgonomicProfile::default();
        p.limits.sitting_secs = 1800;
        p.limits.standing_target_secs = 900;
        std::fs::write(
            dir.join("brisk.json"),
            serde_json::to_string(&p).unwrap(),
        )
        .unwrap();

        let policy = policy();
        let session = Arc::new(Mutex::new(SessionManager::new()));
        switch_ergonomic_profile_by_name(tmp.path(), &policy, &session, "brisk").unwrap();

        assert_eq!(policy.lock().unwrap().ergo_profile().limits.sitting_secs, 1800);
        assert_eq!(session.lock().unwrap().state.session_limit_secs, 1800);
        assert_eq!(session.lock().unwrap().state.stand_limit_secs, 900);
    }
}
