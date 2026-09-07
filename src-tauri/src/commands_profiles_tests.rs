//! commands_profiles_tests.rs — Tests for profile ID validation and helpers.

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::commands_profiles::{
        switch_communication_profile_by_name, switch_ergonomic_profile_by_name,
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
