//! Unit tests for commands_profiles helper functions.
//!
//! Tests cover `validate_profile_id`, `slugify_profile_name`, and the
//! file-system-facing `list_profiles` helper (via tempdir).

use crate::commands_profiles::{slugify_profile_name, validate_profile_id};
use crate::profile_loader::list_profiles;

// ── validate_profile_id ───────────────────────────────────────────────────────

#[test]
fn validate_accepts_plain_word() {
    assert!(validate_profile_id("default").is_ok());
}

#[test]
fn validate_accepts_dash_separated() {
    assert!(validate_profile_id("my-profile").is_ok());
}

#[test]
fn validate_accepts_underscore() {
    assert!(validate_profile_id("test_123").is_ok());
}

#[test]
fn validate_accepts_mixed_alphanumeric() {
    assert!(validate_profile_id("abc123").is_ok());
}

#[test]
fn validate_accepts_single_char() {
    assert!(validate_profile_id("a").is_ok());
}

#[test]
fn validate_rejects_empty_string() {
    assert!(validate_profile_id("").is_err());
}

#[test]
fn validate_rejects_path_traversal_dotdot() {
    assert!(validate_profile_id("../etc/passwd").is_err());
}

#[test]
fn validate_rejects_path_traversal_slash() {
    assert!(validate_profile_id("has/slash").is_err());
}

#[test]
fn validate_rejects_backslash() {
    assert!(validate_profile_id("has\\backslash").is_err());
}

#[test]
fn validate_rejects_spaces() {
    assert!(validate_profile_id("has spaces").is_err());
}

#[test]
fn validate_rejects_dot() {
    assert!(validate_profile_id("no.dots").is_err());
}

#[test]
fn validate_rejects_special_chars() {
    assert!(validate_profile_id("special!chars").is_err());
    assert!(validate_profile_id("at@sign").is_err());
    assert!(validate_profile_id("hash#tag").is_err());
}

// ── slugify_profile_name ──────────────────────────────────────────────────────

#[test]
fn slug_lowercases_and_replaces_spaces() {
    assert_eq!(slugify_profile_name("My New Profile"), Some("my-new-profile".to_string()));
}

#[test]
fn slug_handles_trailing_space() {
    assert_eq!(slugify_profile_name("test "), Some("test".to_string()));
}

#[test]
fn slug_collapses_consecutive_separators() {
    assert_eq!(slugify_profile_name("test  123"), Some("test-123".to_string()));
}

#[test]
fn slug_replaces_non_alphanumeric_with_dash() {
    assert_eq!(slugify_profile_name("hello!world"), Some("hello-world".to_string()));
}

#[test]
fn slug_preserves_already_valid_id() {
    assert_eq!(slugify_profile_name("my-profile"), Some("my-profile".to_string()));
}

#[test]
fn slug_returns_none_for_all_specials() {
    assert_eq!(slugify_profile_name("!!!!"), None);
}

#[test]
fn slug_returns_none_for_empty_string() {
    assert_eq!(slugify_profile_name(""), None);
}

#[test]
fn slug_output_passes_validate() {
    let names = ["My New Profile", "test 123", "gentle mode", "Aggressive-Plus"];
    for name in &names {
        let slug = slugify_profile_name(name).expect("slug should be Some");
        assert!(
            validate_profile_id(&slug).is_ok(),
            "slug '{}' from '{}' failed validation",
            slug,
            name
        );
    }
}

// ── list_profiles (filesystem) ────────────────────────────────────────────────

#[test]
fn list_profiles_returns_only_json_files() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let dir = tmp.path();

    std::fs::write(dir.join("alpha.json"), b"{}").unwrap();
    std::fs::write(dir.join("beta.json"), b"{}").unwrap();
    std::fs::write(dir.join("readme.txt"), b"ignore me").unwrap();
    std::fs::write(dir.join("noext"), b"also ignore").unwrap();

    let paths = list_profiles(dir);
    let names: Vec<_> = paths.iter()
        .filter_map(|p| p.file_name()?.to_str())
        .collect();

    assert_eq!(names.len(), 2);
    assert!(names.contains(&"alpha.json"));
    assert!(names.contains(&"beta.json"));
}

#[test]
fn list_profiles_returns_sorted_order() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let dir = tmp.path();

    std::fs::write(dir.join("zebra.json"), b"{}").unwrap();
    std::fs::write(dir.join("alpha.json"), b"{}").unwrap();
    std::fs::write(dir.join("middle.json"), b"{}").unwrap();

    let paths = list_profiles(dir);
    let names: Vec<_> = paths.iter()
        .filter_map(|p| p.file_stem()?.to_str())
        .collect();

    assert_eq!(names, vec!["alpha", "middle", "zebra"]);
}

#[test]
fn list_profiles_returns_empty_for_missing_dir() {
    let dir = std::path::Path::new("/nonexistent/path/that/does/not/exist");
    let paths = list_profiles(dir);
    assert!(paths.is_empty());
}

#[test]
fn list_profiles_returns_empty_for_empty_dir() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let paths = list_profiles(tmp.path());
    assert!(paths.is_empty());
}
