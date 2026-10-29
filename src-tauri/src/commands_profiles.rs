//! commands_profiles.rs — IPC commands for listing and switching profiles.

use std::path::Path;
use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Manager, Runtime, State};

use crate::commands::{ensure_initialized, AppState};
use crate::communication_policy::CommunicationPolicy;
use crate::profile_loader::{
    list_profiles, load_communication_profile, load_profile, validate_communication_profile,
};
use crate::communication_profile::CommunicationProfile;
use crate::ergonomic_profile::ErgonomicProfile;
use crate::session::SessionManager;

// ── DTOs ────────────────────────────────────────────────────────────────────

/// Lightweight profile descriptor returned by list commands.
#[derive(Debug, Serialize)]
pub struct ProfileInfo {
    /// File stem used as profile ID (e.g. `"default"`, `"gentle"`).
    pub id: String,
    /// Human-readable display name from the profile JSON.
    pub name: String,
    /// Short description of the profile's intent.
    pub description: String,
    /// Absolute path to the JSON file on disk.
    pub path: String,
}

/// Active profile IDs for both profile categories.
#[derive(Debug, Serialize)]
pub struct ActiveProfiles {
    /// ID of the communication profile currently loaded in the live policy.
    pub communication_id: String,
    /// ID of the ergonomic profile currently loaded in the live policy.
    pub ergonomic_id: String,
    /// Live ergonomic parameters used by the session engine.
    pub effective_limits: crate::ergonomic_profile::Limits,
    /// Runtime session-limit overrides, which may differ from profile values.
    pub session_limit_secs: i64,
    pub stand_limit_secs: i64,
    /// Actual communication settings loaded by the live policy.
    pub notifications_enabled: bool,
    pub dismiss_durations_mins: Vec<u32>,
    pub current_dismiss_minutes: u32,
    /// Value accepted by the actual Snooze command, or None if unsupported.
    pub snooze_minutes: Option<u16>,
    pub tone_shift_after_dismisses: u8,
    pub notify_cooldowns_secs: Vec<u64>,
}

pub(crate) fn active_profiles_snapshot(
    session: &SessionManager,
    policy: &CommunicationPolicy,
) -> ActiveProfiles {
    let comm = policy.comm_profile();
    let ergo = policy.ergo_profile();
    ActiveProfiles {
        communication_id: comm.id.clone(),
        ergonomic_id: ergo.id.clone(),
        effective_limits: session.limits.clone(),
        session_limit_secs: session.state.session_limit_secs,
        stand_limit_secs: session.state.stand_limit_secs,
        notifications_enabled: comm.notifications_enabled,
        dismiss_durations_mins: comm.snooze.durations_mins.clone(),
        current_dismiss_minutes: policy.current_dismiss_minutes(),
        snooze_minutes: policy.reminder_snooze_minutes().ok(),
        tone_shift_after_dismisses: comm.snooze.tone_shift_after_dismisses,
        notify_cooldowns_secs: comm.snooze.notify_cooldowns_secs.clone(),
    }
}

// ── Store keys ───────────────────────────────────────────────────────────────

pub(crate) const KEY_ACTIVE_COMM: &str = "active_communication_profile";
pub(crate) const KEY_ACTIVE_ERGO: &str = "active_ergonomic_profile";

// ── helpers ──────────────────────────────────────────────────────────────────

/// Rejects profile IDs that could be used for path traversal.
/// Only alphanumeric characters, dashes, and underscores are allowed.
pub(crate) fn validate_profile_id(id: &str) -> Result<(), String> {
    if id.is_empty() || !id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(format!(
            "Invalid profile ID '{}': only alphanumeric, dash, underscore allowed",
            id
        ));
    }
    Ok(())
}

/// Persists a profile selection while retaining the old Store cache on failure.
/// A failed save followed by a failed rollback save leaves disk state uncertain;
/// both errors are returned so callers cannot report a coherent success.
fn persist_active_id<R: Runtime>(store: &crate::app_store::AppStore<R>, key: &str, id: &str) -> Result<(), String> {
    let store = store.write();
    let previous = store.get(key);
    store.set(key, serde_json::Value::String(id.to_string()));
    if let Err(save_error) = store.save() {
        match previous {
            Some(value) => store.set(key, value),
            None => { store.delete(key); }
        }
        return match store.save() {
            Ok(()) => Err(format!("Failed to save store: {save_error}; in-memory selection restored and prior selection persisted")),
            Err(rollback_error) => Err(format!(
                "Failed to save store: {save_error}; rollback save also failed: {rollback_error}; in-memory selection restored but disk selection is uncertain"
            )),
        };
    }
    Ok(())
}

/// Shared persistent communication switch used by the IPC wrapper and disposable Store tests.
pub(crate) fn switch_communication_profile_persisted<R: Runtime>(
    app_data_dir: &Path,
    store: &crate::app_store::AppStore<R>,
    policy: &Mutex<CommunicationPolicy>,
    id: &str,
) -> Result<(), String> {
    let path = profile_path(app_data_dir, "communication", id)?;
    let profile = load_communication_profile(&path)?;
    // Checkpoints take policy before the Store writer. Keep that same order
    // while changing the persisted selection and the live policy together.
    let mut policy = policy.lock().unwrap_or_else(|e| e.into_inner());
    persist_active_id(store, KEY_ACTIVE_COMM, id)?;
    policy.set_comm_profile(profile);
    log::info!("Switched communication profile to '{}'", id);
    Ok(())
}

/// Shared persistent ergonomic switch used by the IPC wrapper and disposable Store tests.
pub(crate) fn switch_ergonomic_profile_persisted<R: Runtime>(
    app_data_dir: &Path,
    store: &crate::app_store::AppStore<R>,
    policy: &Mutex<CommunicationPolicy>,
    session: &Mutex<SessionManager>,
    id: &str,
) -> Result<(), String> {
    let path = profile_path(app_data_dir, "ergonomic", id)?;
    let profile = load_profile::<ErgonomicProfile>(&path);
    // Match checkpoint lock order: session, policy, then shared Store writer.
    let mut session = session.lock().unwrap_or_else(|e| e.into_inner());
    let mut policy = policy.lock().unwrap_or_else(|e| e.into_inner());
    persist_active_id(store, KEY_ACTIVE_ERGO, id)?;
    session.state.session_limit_secs = profile.limits.sitting_secs as i64;
    session.state.stand_limit_secs = profile.limits.standing_target_secs as i64;
    policy.set_ergo_profile(profile);
    log::info!("Switched ergonomic profile to '{}'", id);
    Ok(())
}

/// Converts a display name into a valid profile ID slug.
///
/// Lowercases the string, maps non-alphanumeric characters to dashes, collapses
/// consecutive dashes, and strips leading/trailing dashes. Returns `None` if the
/// result would be empty (e.g. input was `"!!!"`).
pub(crate) fn slugify_profile_name(name: &str) -> Option<String> {
    let slug: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if slug.is_empty() { None } else { Some(slug) }
}

fn app_data_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|e| format!("Cannot get app data dir: {}", e))
}

fn profiles_dir(app: &AppHandle, subdir: &str) -> Result<std::path::PathBuf, String> {
    Ok(app_data_dir(app)?.join("profiles").join(subdir))
}

fn build_profile_info<T>(path: &std::path::Path, id: &str, name: String, desc: String) -> ProfileInfo
where T: serde::de::DeserializeOwned + Default
{
    ProfileInfo {
        id: id.to_string(),
        name,
        description: desc,
        path: path.to_string_lossy().into_owned(),
    }
}

// ── Switching, without an AppHandle (E022-T07) ───────────────────────────────
//
// The relay command path runs on a tokio task with no `AppHandle`, so the half
// of a switch that touches the engine takes plain handles and lives here. The
// `#[tauri::command]` wrappers add the half these cannot do: persisting the
// active-profile ID. A remote switch is therefore live but not sticky.

/// Locates a profile file and refuses an unsafe or uninstalled ID.
///
/// The existence check is the point: `load_profile` answers with `Default` for
/// a file it cannot read, so without it a switch to a profile the desk does
/// not have would report success and quietly reset the user to the defaults.
pub(crate) fn profile_path(app_data_dir: &Path, subdir: &str, id: &str) -> Result<std::path::PathBuf, String> {
    validate_profile_id(id)?;
    let path = app_data_dir
        .join("profiles")
        .join(subdir)
        .join(format!("{}.json", id));
    if path.exists() {
        Ok(path)
    } else {
        Err(format!("Profile '{}' not found", id))
    }
}

/// Applies a communication profile by ID.
pub(crate) fn switch_communication_profile_by_name(
    app_data_dir: &Path,
    policy: &Mutex<CommunicationPolicy>,
    id: &str,
) -> Result<(), String> {
    let path = profile_path(app_data_dir, "communication", id)?;
    let profile = load_communication_profile(&path)?;
    policy.lock().unwrap_or_else(|e| e.into_inner()).set_comm_profile(profile);
    log::info!("Switched communication profile to '{}'", id);
    Ok(())
}

/// Applies an ergonomic profile by ID, including the two session limits the
/// profile owns. Takes `session` as well as `policy` because those limits live
/// on the engine; lock order is session first, then policy.
pub(crate) fn switch_ergonomic_profile_by_name(
    app_data_dir: &Path,
    policy: &Mutex<CommunicationPolicy>,
    session: &Mutex<SessionManager>,
    id: &str,
) -> Result<(), String> {
    let path = profile_path(app_data_dir, "ergonomic", id)?;
    let profile = load_profile::<ErgonomicProfile>(&path);
    {
        let mut session = session.lock().unwrap_or_else(|e| e.into_inner());
        session.state.session_limit_secs = profile.limits.sitting_secs as i64;
        session.state.stand_limit_secs = profile.limits.standing_target_secs as i64;
    }
    policy.lock().unwrap_or_else(|e| e.into_inner()).set_ergo_profile(profile);
    log::info!("Switched ergonomic profile to '{}'", id);
    Ok(())
}

// ── Commands ─────────────────────────────────────────────────────────────────

/// Lists all available communication profiles sorted by name.
#[tauri::command]
pub fn list_communication_profiles(app: AppHandle) -> Result<Vec<ProfileInfo>, String> {
    let dir = profiles_dir(&app, "communication")?;
    let paths = list_profiles(&dir);
    let mut infos: Vec<ProfileInfo> = paths.iter().filter_map(|path| {
        let id = path.file_stem()?.to_str()?.to_string();
        let p = load_profile::<CommunicationProfile>(path);
        let errs = validate_communication_profile(&p);
        if !errs.is_empty() {
            log::warn!("Profile '{}' has validation errors: {:?}", id, errs);
        }
        Some(build_profile_info::<CommunicationProfile>(
            path, &id, p.name.clone(), p.description.clone(),
        ))
    }).collect();
    infos.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(infos)
}

/// Lists all available ergonomic profiles sorted by name.
#[tauri::command]
pub fn list_ergonomic_profiles(app: AppHandle) -> Result<Vec<ProfileInfo>, String> {
    let dir = profiles_dir(&app, "ergonomic")?;
    let paths = list_profiles(&dir);
    let mut infos: Vec<ProfileInfo> = paths.iter().filter_map(|path| {
        let id = path.file_stem()?.to_str()?.to_string();
        let p = load_profile::<ErgonomicProfile>(path);
        Some(build_profile_info::<ErgonomicProfile>(
            path, &id, p.name.clone(), p.description.clone(),
        ))
    }).collect();
    infos.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(infos)
}

/// Returns the currently loaded profiles and effective runtime parameters.
#[tauri::command]
pub fn get_active_profiles(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ActiveProfiles, String> {
    ensure_initialized(&app, &state)?;
    // Match checkpoint and profile-switch lock order: session before policy.
    let session = state.session.lock().unwrap_or_else(|e| e.into_inner());
    let policy = state.comm_policy.lock().unwrap_or_else(|e| e.into_inner());
    Ok(active_profiles_snapshot(&session, &policy))
}

/// Switches the active communication profile by ID.
#[tauri::command]
pub fn switch_communication_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    ensure_initialized(&app, &state)?;
    let dir = app_data_dir(&app)?;
    let store = app.try_state::<crate::app_store::AppStore<tauri::Wry>>()
        .ok_or_else(|| "Failed to access store".to_string())?;
    switch_communication_profile_persisted(&dir, store.inner(), &state.comm_policy, &id)
}

/// Switches the active ergonomic profile by ID.
#[tauri::command]
pub fn switch_ergonomic_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    ensure_initialized(&app, &state)?;
    let dir = app_data_dir(&app)?;
    let store = app.try_state::<crate::app_store::AppStore<tauri::Wry>>()
        .ok_or_else(|| "Failed to access store".to_string())?;
    switch_ergonomic_profile_persisted(&dir, store.inner(), &state.comm_policy, &state.session, &id)
}

/// Opens a profile JSON file in the system default editor.
#[tauri::command]
pub fn open_profile_in_editor(path: String) -> Result<(), String> {
    std::process::Command::new("cmd")
        .args(["/c", "start", "", &path])
        .spawn()
        .map_err(|e| format!("Failed to open editor: {}", e))?;
    Ok(())
}

/// Duplicates a profile file under a new name. Returns the new profile's ID.
#[tauri::command]
pub fn duplicate_profile(source_path: String, new_name: String) -> Result<String, String> {
    let src = std::path::Path::new(&source_path);
    let dir = src.parent().ok_or("Invalid source path")?;

    // Slugify new_name to use as file stem
    let new_id = slugify_profile_name(&new_name)
        .ok_or_else(|| format!("Cannot derive a valid ID from name '{}'", new_name))?;

    validate_profile_id(&new_id)?;
    let dest = dir.join(format!("{}.json", new_id));
    if dest.exists() {
        return Err(format!("Profile '{}' already exists", new_id));
    }

    // Load, patch name/id, re-serialize
    let text = std::fs::read_to_string(src)
        .map_err(|e| format!("Cannot read source profile: {}", e))?;
    let mut value: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("Cannot parse source profile: {}", e))?;

    if let serde_json::Value::Object(ref mut map) = value {
        map.insert("id".to_string(), serde_json::Value::String(new_id.clone()));
        map.insert("name".to_string(), serde_json::Value::String(new_name));
    }

    let out = serde_json::to_string_pretty(&value)
        .map_err(|e| format!("Cannot serialize profile: {}", e))?;
    std::fs::write(&dest, out)
        .map_err(|e| format!("Cannot write profile: {}", e))?;

    Ok(new_id)
}
