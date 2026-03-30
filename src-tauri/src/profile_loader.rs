//! Profile loader — reads ergonomic and communication profiles from JSON files.
//!
//! Provides graceful fallback to defaults when a file is missing or malformed,
//! a file-mtime watcher for hot-reload, directory initialisation, and basic
//! validation of [`CommunicationProfile`] references.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::de::DeserializeOwned;

use crate::communication_profile::CommunicationProfile;
use crate::ergonomic_profile::ErgonomicProfile;

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

// ── validate_communication_profile ───────────────────────────────────────────

/// Validate a [`CommunicationProfile`], returning a list of human-readable errors.
///
/// Checks:
/// - Escalation steps within each phase have strictly ascending `at` values.
/// - Blink pattern names referenced in escalation steps exist in `blink_patterns`.
/// - Overlay pattern names referenced in escalation steps exist in `overlay_patterns`.
pub fn validate_communication_profile(p: &CommunicationProfile) -> Vec<String> {
    let mut errors = Vec::new();

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

// ── ensure_profiles_dir ──────────────────────────────────────────────────────

/// Create the `profiles/` directory tree under `app_data_dir` and write default
/// JSON files for any profiles that don't already exist.
///
/// Layout written:
/// ```text
/// <app_data_dir>/profiles/ergonomic/default.json
/// <app_data_dir>/profiles/communication/default.json
/// ```
pub fn ensure_profiles_dir(app_data_dir: &Path) {
    let ergo_dir = app_data_dir.join("profiles").join("ergonomic");
    let comm_dir = app_data_dir.join("profiles").join("communication");

    for dir in [&ergo_dir, &comm_dir] {
        if let Err(e) = std::fs::create_dir_all(dir) {
            log::warn!("ensure_profiles_dir: cannot create {:?}: {}", dir, e);
        }
    }

    write_default_if_missing::<ErgonomicProfile>(&ergo_dir.join("default.json"));
    write_default_if_missing::<CommunicationProfile>(&comm_dir.join("default.json"));
}

fn write_default_if_missing<T: serde::Serialize + Default>(path: &Path) {
    if path.exists() {
        return;
    }
    match serde_json::to_string_pretty(&T::default()) {
        Ok(json) => {
            if let Err(e) = std::fs::write(path, json) {
                log::warn!("ensure_profiles_dir: cannot write {:?}: {}", path, e);
            }
        }
        Err(e) => log::warn!("ensure_profiles_dir: serialize failed for {:?}: {}", path, e),
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

// ── load_all_profiles ─────────────────────────────────────────────────────────

/// Load all communication profiles from `dir` into a map keyed by file stem.
pub fn load_all_communication_profiles(dir: &Path) -> HashMap<String, CommunicationProfile> {
    list_profiles(dir)
        .into_iter()
        .filter_map(|path| {
            let key = path.file_stem()?.to_str()?.to_string();
            Some((key, load_profile::<CommunicationProfile>(&path)))
        })
        .collect()
}

/// Load all ergonomic profiles from `dir` into a map keyed by file stem.
pub fn load_all_ergonomic_profiles(dir: &Path) -> HashMap<String, ErgonomicProfile> {
    list_profiles(dir)
        .into_iter()
        .filter_map(|path| {
            let key = path.file_stem()?.to_str()?.to_string();
            Some((key, load_profile::<ErgonomicProfile>(&path)))
        })
        .collect()
}
