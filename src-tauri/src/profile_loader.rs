//! Profile loader — reads ergonomic and communication profiles from JSON files.
//!
//! Provides graceful fallback to defaults when a file is missing or malformed,
//! a file-mtime watcher for hot-reload, directory initialisation, and basic
//! validation of [`CommunicationProfile`] references.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::de::DeserializeOwned;

use crate::communication_profile::CommunicationProfile;

// ── load_profile ─────────────────────────────────────────────────────────────

/// Load a profile from a JSON file, falling back to `T::default()` on any error.
///
/// Errors (file not found, parse failure) are logged as warnings so the app
/// always starts regardless of the state of on-disk profile files.
pub fn load_profile<T: DeserializeOwned + Default>(path: &Path) -> T {
    match std::fs::read_to_string(path) {
        Ok(text) => match serde_json::from_str::<T>(&text) {
            Ok(profile) => profile,
            Err(e) => {
                log::warn!("profile_loader: parse error in {:?}: {}", path, e);
                T::default()
            }
        },
        Err(e) => {
            log::warn!("profile_loader: cannot read {:?}: {}", path, e);
            T::default()
        }
    }
}

/// Reads and validates a communication profile without replacing failures by defaults.
///
/// Activation and reload callers must keep the currently active profile when this fails.
pub fn load_communication_profile(path: &Path) -> Result<CommunicationProfile, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| format!("Cannot read communication profile {:?}: {}", path, error))?;
    let mut profile: CommunicationProfile = serde_json::from_str(&text)
        .map_err(|error| format!("Cannot parse communication profile {:?}: {}", path, error))?;
    if is_legacy_silent_profile(&text, &profile) {
        profile.notifications_enabled = false;
    }
    let errors = validate_communication_profile(&profile);
    if !errors.is_empty() {
        return Err(format!(
            "Invalid communication profile {:?}: {}",
            path,
            errors.join("; ")
        ));
    }
    Ok(profile)
}

/// Preserve the built-in Silent profile's intent when loading its pre-field JSON,
/// including a copy whose id and display name were changed by `duplicate_profile`.
fn is_legacy_silent_profile(text: &str, profile: &CommunicationProfile) -> bool {
    let Ok(candidate) = serde_json::from_str::<serde_json::Value>(text) else {
        return false;
    };
    let Some(candidate_fields) = candidate.as_object() else {
        return false;
    };
    if candidate_fields.contains_key("notifications_enabled") {
        return false;
    }

    profile.escalation.sitting.is_empty()
        && profile.escalation.standing.is_empty()
        && profile.disconnected.notify_once.is_none()
        && profile.inactive.notify_once.is_none()
        && !profile.screen_break_nudge.enabled
        && !profile.periodic_notifications.inactivity_enabled
        && !profile.periodic_notifications.posture_balance_enabled
        && !profile.periodic_notifications.praise_halfway_enabled
}

// ── validate_communication_profile ───────────────────────────────────────────

/// Validate a [`CommunicationProfile`], returning a list of human-readable errors.
///
/// Checks:
/// - Escalation steps within each phase have strictly ascending `at` values.
/// - Blink pattern names referenced in escalation steps exist in `blink_patterns`.
/// - Overlay pattern names referenced in escalation steps exist in `overlay_patterns`.
pub fn validate_communication_profile(p: &CommunicationProfile) -> Vec<String> {
    let mut errors = Vec::new();

    if !matches!(p.notification_backend.as_str(), "toast" | "popup" | "both") {
        errors.push(format!(
            "notification_backend must be 'toast', 'popup', or 'both', got '{}'",
            p.notification_backend
        ));
    }

    if p.snooze.notify_cooldowns_secs.len() > crate::communication_profile::MAX_NOTIFY_REMINDERS {
        errors.push(format!(
            "snooze.notify_cooldowns_secs may configure at most four notifications, got {}",
            p.snooze.notify_cooldowns_secs.len()
        ));
    }

    for (phase, steps) in [
        ("sitting", p.escalation.sitting.as_slice()),
        ("standing", p.escalation.standing.as_slice()),
    ] {
        for step in steps {
            if let Some(notify) = step.notify.as_deref() {
                if !matches!(notify, "toast" | "popup") {
                    errors.push(format!(
                        "escalation.{phase} step at={} notify must be 'toast' or 'popup' when set, got '{}'",
                        step.at, notify
                    ));
                }
            }
        }
    }

    for (location, notify_once) in [
        ("disconnected", p.disconnected.notify_once.as_deref()),
        ("inactive", p.inactive.notify_once.as_deref()),
    ] {
        if let Some(notify_once) = notify_once {
            if !matches!(notify_once, "toast" | "popup") {
                errors.push(format!(
                    "{location}.notify_once must be 'toast' or 'popup' when set, got '{}'",
                    notify_once
                ));
            }
        }
    }

    // Check sitting escalation `at` values are strictly ascending.
    check_ascending(&p.escalation.sitting, "sitting", &mut errors);
    // Check standing escalation `at` values are strictly ascending.
    check_ascending(&p.escalation.standing, "standing", &mut errors);

    // Check all tray references for blink patterns.
    for phase in ["sitting", "standing"] {
        let steps = if phase == "sitting" {
            &p.escalation.sitting
        } else {
            &p.escalation.standing
        };
        for step in steps {
            if step.tray.starts_with("blink_") && !p.blink_patterns.contains_key(&step.tray) {
                errors.push(format!(
                    "{phase} step at={}: tray references unknown blink pattern '{}'",
                    step.at, step.tray
                ));
            }
            if step.overlay.starts_with("pulse_") && !p.overlay_patterns.contains_key(&step.overlay) {
                errors.push(format!(
                    "{phase} step at={}: overlay references unknown pattern '{}'",
                    step.at, step.overlay
                ));
            }
        }
    }

    errors
}

fn check_ascending(
    steps: &[crate::communication_profile::EscalationStep],
    phase: &str,
    errors: &mut Vec<String>,
) {
    let mut prev: Option<i64> = None;
    for step in steps {
        if let Some(p) = prev {
            if step.at <= p {
                errors.push(format!(
                    "{phase} escalation: step at={} must be > previous at={}",
                    step.at, p
                ));
            }
        }
        prev = Some(step.at);
    }
}

// ── ProfileWatcher ───────────────────────────────────────────────────────────

/// Polls a file's modification time to detect external changes for hot-reload.
///
/// Call [`ProfileWatcher::has_changed`] periodically; returns `true` once per
/// change (resets the baseline on each positive return).
pub struct ProfileWatcher {
    path: PathBuf,
    last_mtime: Option<SystemTime>,
}

impl ProfileWatcher {
    /// Create a new watcher for `path`. Records the current mtime immediately.
    pub fn new(path: PathBuf) -> Self {
        let last_mtime = mtime(&path);
        Self { path, last_mtime }
    }

    /// Returns `true` if the file has been modified since the last call.
    pub fn has_changed(&mut self) -> bool {
        let current = mtime(&self.path);
        if current != self.last_mtime {
            self.last_mtime = current;
            true
        } else {
            false
        }
    }
}

fn mtime(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).ok()?.modified().ok()
}

// ── Built-in profile JSON content ────────────────────────────────────────────

/// Built-in communication profiles embedded in the binary.
const COMM_PROFILE_DEFAULT: &str = include_str!("../profiles/communication/default.json");
const COMM_PROFILE_AGGRESSIVE: &str = include_str!("../profiles/communication/aggressive.json");
const COMM_PROFILE_GENTLE: &str = include_str!("../profiles/communication/gentle.json");
const COMM_PROFILE_SILENT: &str = include_str!("../profiles/communication/silent.json");

/// Built-in ergonomic profiles embedded in the binary.
const ERGO_PROFILE_DEFAULT: &str = include_str!("../profiles/ergonomic/default.json");
const ERGO_PROFILE_STRICT: &str = include_str!("../profiles/ergonomic/strict.json");
const ERGO_PROFILE_RELAXED: &str = include_str!("../profiles/ergonomic/relaxed.json");

// ── ensure_profiles_dir ──────────────────────────────────────────────────────

/// Create the `profiles/` directory tree under `app_data_dir` and write all
/// built-in profile JSON files if they don't already exist.
///
/// Copies embedded JSON profiles from the binary to the app data directory.
/// This ensures users have a full set of built-in profiles to choose from and
/// allows them to customize profiles by editing these files.
///
/// Layout created:
/// ```text
/// <app_data_dir>/profiles/ergonomic/default.json
/// <app_data_dir>/profiles/ergonomic/strict.json
/// <app_data_dir>/profiles/ergonomic/relaxed.json
/// <app_data_dir>/profiles/communication/default.json
/// <app_data_dir>/profiles/communication/aggressive.json
/// <app_data_dir>/profiles/communication/gentle.json
/// <app_data_dir>/profiles/communication/silent.json
/// ```
pub fn ensure_profiles_dir(app_data_dir: &Path) {
    let ergo_dir = app_data_dir.join("profiles").join("ergonomic");
    let comm_dir = app_data_dir.join("profiles").join("communication");

    for dir in [&ergo_dir, &comm_dir] {
        if let Err(e) = std::fs::create_dir_all(dir) {
            log::warn!("ensure_profiles_dir: cannot create {:?}: {}", dir, e);
        }
    }

    // Write ergonomic profiles
    write_if_missing(&ergo_dir.join("default.json"), ERGO_PROFILE_DEFAULT);
    write_if_missing(&ergo_dir.join("strict.json"), ERGO_PROFILE_STRICT);
    write_if_missing(&ergo_dir.join("relaxed.json"), ERGO_PROFILE_RELAXED);

    // Write communication profiles
    write_if_missing(&comm_dir.join("default.json"), COMM_PROFILE_DEFAULT);
    write_if_missing(&comm_dir.join("aggressive.json"), COMM_PROFILE_AGGRESSIVE);
    write_if_missing(&comm_dir.join("gentle.json"), COMM_PROFILE_GENTLE);
    write_if_missing(&comm_dir.join("silent.json"), COMM_PROFILE_SILENT);
}

/// Write embedded profile JSON content to disk if the file doesn't exist.
fn write_if_missing(path: &Path, content: &str) {
    if path.exists() {
        return;
    }
    if let Err(e) = std::fs::write(path, content) {
        log::warn!("ensure_profiles_dir: cannot write {:?}: {}", path, e);
    }
}

// ── list_profiles ─────────────────────────────────────────────────────────────

/// List all `.json` profile files in `dir`, sorted alphabetically by file name.
///
/// Returns an empty vec if the directory cannot be read.
pub fn list_profiles(dir: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = match std::fs::read_dir(dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
            .collect(),
        Err(e) => {
            log::warn!("list_profiles: cannot read {:?}: {}", dir, e);
            return vec![];
        }
    };
    paths.sort();
    paths
}

#[cfg(test)]
mod profile_validation_tests {
    use super::{load_communication_profile, validate_communication_profile};
    use crate::communication_profile::{CommunicationProfile, EscalationStep};

    fn step(at: i64, notify: Option<&str>) -> EscalationStep {
        EscalationStep {
            at,
            tray: "yellow".into(),
            overlay: "yellow".into(),
            popup_header: "yellow".into(),
            notify: notify.map(str::to_owned),
        }
    }

    fn set_escalation(profile: &mut CommunicationProfile, phase: &str, notify: Option<&str>) {
        let steps = vec![step(0, notify)];
        match phase {
            "sitting" => profile.escalation.sitting = steps,
            "standing" => profile.escalation.standing = steps,
            _ => unreachable!(),
        }
    }

    fn builtin_profile(name: &str, json: &str) -> CommunicationProfile {
        serde_json::from_str(json).unwrap_or_else(|error| panic!("{name}: {error}"))
    }

    #[test]
    fn notification_backend_accepts_only_supported_values() {
        for value in ["toast", "popup", "both"] {
            let mut profile = CommunicationProfile::default();
            profile.notification_backend = value.into();
            assert!(validate_communication_profile(&profile).is_empty(), "{value}");
        }
        for value in ["unknown", "", " "] {
            let mut profile = CommunicationProfile::default();
            profile.notification_backend = value.into();
            let errors = validate_communication_profile(&profile);
            assert!(errors.iter().any(|error| error.contains("notification_backend")), "{value}: {errors:?}");
        }
    }

    #[test]
    fn reminder_schedule_rejects_more_than_four_notifications() {
        let mut profile = CommunicationProfile::default();
        profile.snooze.notify_cooldowns_secs = vec![0; 5];

        let errors = validate_communication_profile(&profile);

        assert!(errors.iter().any(|error| error.contains("four")), "{errors:?}");
    }

    #[test]
    fn fallible_loader_accepts_each_repository_builtin_profile() {
        let profiles_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("profiles")
            .join("communication");

        for id in ["default", "aggressive", "gentle", "silent", "demo"] {
            let profile = load_communication_profile(&profiles_dir.join(format!("{id}.json")))
                .unwrap_or_else(|error| panic!("{id}: {error}"));
            assert_eq!(profile.id, id);
            assert_eq!(profile.notifications_enabled, id != "silent", "{id}");
        }
    }

    #[test]
    fn fallible_loader_reports_invalid_and_malformed_profiles() {
        let temp = tempfile::TempDir::new().unwrap();
        let invalid_path = temp.path().join("invalid.json");
        let malformed_path = temp.path().join("malformed.json");
        let mut invalid = CommunicationProfile::default();
        invalid.notification_backend = "unexpected".into();
        std::fs::write(&invalid_path, serde_json::to_string(&invalid).unwrap()).unwrap();
        std::fs::write(&malformed_path, "{\"id\":").unwrap();

        let invalid_error = load_communication_profile(&invalid_path).unwrap_err();
        let malformed_error = load_communication_profile(&malformed_path).unwrap_err();

        assert!(invalid_error.contains("notification_backend"), "{invalid_error}");
        assert!(malformed_error.contains("Cannot parse"), "{malformed_error}");
    }

    #[test]
    fn escalation_notify_accepts_toast_popup_and_none_in_both_phases() {
        for phase in ["sitting", "standing"] {
            for value in [None, Some("toast"), Some("popup")] {
                let mut profile = CommunicationProfile::default();
                set_escalation(&mut profile, phase, value);
                assert!(validate_communication_profile(&profile).is_empty(), "{phase} {value:?}");
            }
        }
    }

    #[test]
    fn escalation_notify_rejects_explicit_unsupported_values_with_step_context() {
        for phase in ["sitting", "standing"] {
            for value in ["both", "unknown", "", " "] {
                let mut profile = CommunicationProfile::default();
                set_escalation(&mut profile, phase, Some(value));
                let errors = validate_communication_profile(&profile);
                assert!(errors.iter().any(|error| error.contains(phase) && error.contains("notify")), "{phase} {value:?}: {errors:?}");
            }
        }
    }

    #[test]
    fn notify_once_accepts_toast_popup_and_none_for_disconnected_and_inactive() {
        for location in ["disconnected", "inactive"] {
            for value in [None, Some("toast"), Some("popup")] {
                let mut profile = CommunicationProfile::default();
                let config = match location {
                    "disconnected" => &mut profile.disconnected,
                    _ => &mut profile.inactive,
                };
                config.notify_once = value.map(str::to_owned);
                assert!(validate_communication_profile(&profile).is_empty(), "{location} {value:?}");
            }
        }
    }

    #[test]
    fn notify_once_rejects_explicit_unsupported_values_with_field_context() {
        for location in ["disconnected", "inactive"] {
            for value in ["both", "unknown", "", " "] {
                let mut profile = CommunicationProfile::default();
                let config = match location {
                    "disconnected" => &mut profile.disconnected,
                    _ => &mut profile.inactive,
                };
                config.notify_once = Some(value.into());
                let errors = validate_communication_profile(&profile);
                assert!(errors.iter().any(|error| error.contains(location) && error.contains("notify_once")), "{location} {value:?}: {errors:?}");
            }
        }
    }

    #[test]
    fn channel_errors_accumulate_with_existing_ordering_and_pattern_errors() {
        let mut profile = CommunicationProfile::default();
        profile.notification_backend = "invalid".into();
        profile.escalation.sitting = vec![step(0, None), step(0, None)];
        profile.escalation.sitting[0].tray = "blink_missing".into();
        profile.escalation.sitting[0].overlay = "pulse_missing".into();
        let errors = validate_communication_profile(&profile);
        assert_eq!(errors.len(), 4, "{errors:?}");
        assert!(errors.iter().any(|error| error.contains("notification_backend")));
        assert!(errors.iter().any(|error| error.contains("must be > previous")));
        assert!(errors.iter().any(|error| error.contains("unknown blink pattern")));
        assert!(errors.iter().any(|error| error.contains("unknown pattern")));
    }

    #[test]
    fn empty_escalation_lists_and_all_builtin_channels_remain_valid() {
        let mut default_profile = CommunicationProfile::default();
        default_profile.escalation.sitting.clear();
        default_profile.escalation.standing.clear();
        assert!(validate_communication_profile(&default_profile).is_empty());

        for (name, json) in [
            ("default", include_str!("../profiles/communication/default.json")),
            ("aggressive", include_str!("../profiles/communication/aggressive.json")),
            ("gentle", include_str!("../profiles/communication/gentle.json")),
            ("silent", include_str!("../profiles/communication/silent.json")),
            ("demo", include_str!("../profiles/communication/demo.json")),
        ] {
            let profile = builtin_profile(name, json);
            assert!(validate_communication_profile(&profile).is_empty(), "{name}: {:?}", validate_communication_profile(&profile));
        }
    }

    #[test]
    fn legacy_partial_baseline_preserves_custom_channels_and_defaults_only_popup() {
        let profile: CommunicationProfile = serde_json::from_value(serde_json::json!({
            "baseline": {
                "sitting": { "tray": "legacy-sitting-tray", "overlay": "legacy-sitting-overlay" },
                "standing": { "tray": "legacy-standing-tray", "overlay": "legacy-standing-overlay" }
            }
        }))
        .expect("legacy baseline without popup should deserialize");

        assert_eq!(profile.baseline.sitting.tray, "legacy-sitting-tray");
        assert_eq!(profile.baseline.sitting.overlay, "legacy-sitting-overlay");
        assert_eq!(profile.baseline.sitting.popup, "neutral");
        assert_eq!(profile.baseline.standing.tray, "legacy-standing-tray");
        assert_eq!(profile.baseline.standing.overlay, "legacy-standing-overlay");
        assert_eq!(profile.baseline.standing.popup, "neutral");
    }

    #[test]
    fn supplied_baseline_popup_is_preserved_and_wrong_type_is_rejected() {
        let profile: CommunicationProfile = serde_json::from_value(serde_json::json!({
            "baseline": {
                "sitting": {
                    "tray": "custom-tray",
                    "overlay": "custom-overlay",
                    "popup": "warning"
                }
            }
        }))
        .expect("explicit popup should deserialize");
        assert_eq!(profile.baseline.sitting.popup, "warning");

        let malformed = serde_json::from_value::<CommunicationProfile>(serde_json::json!({
            "baseline": {
                "sitting": {
                    "tray": "custom-tray",
                    "overlay": "custom-overlay",
                    "popup": 7
                }
            }
        }));
        assert!(malformed.is_err(), "wrong-type popup must not silently default");
    }
}

