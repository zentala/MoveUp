//! commands_profiles_tests.rs — Tests for profile ID validation and helpers.

#[cfg(test)]
mod tests {
    use crate::commands_profiles::validate_profile_id;
    use crate::profile_loader::list_profiles;

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
}
