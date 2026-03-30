//! commands_profiles.rs — IPC commands for listing and switching profiles.

use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::commands::{ensure_initialized, AppState};
use crate::profile_loader::{list_profiles, load_profile, validate_communication_profile};
use crate::communication_profile::CommunicationProfile;
use crate::ergonomic_profile::ErgonomicProfile;

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
    /// File stem of the active communication profile.
    pub communication_id: String,
    /// File stem of the active ergonomic profile.
    pub ergonomic_id: String,
}

// ── Store keys ───────────────────────────────────────────────────────────────

const KEY_ACTIVE_COMM: &str = "active_communication_profile";
const KEY_ACTIVE_ERGO: &str = "active_ergonomic_profile";
const DEFAULT_PROFILE_ID: &str = "default";

// ── helpers ──────────────────────────────────────────────────────────────────

/// Rejects profile IDs that could be used for path traversal.
/// Only alphanumeric characters, dashes, and underscores are allowed.
fn validate_profile_id(id: &str) -> Result<(), String> {
    if id.is_empty() || !id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(format!(
            "Invalid profile ID '{}': only alphanumeric, dash, underscore allowed",
            id
        ));
    }
    Ok(())
}

fn get_active_id(app: &AppHandle, key: &str) -> String {
    app.try_state::<tauri_plugin_store::Store<tauri::Wry>>()
        .and_then(|store| {
            store.get(key).and_then(|v| v.as_str().map(String::from))
        })
        .unwrap_or_else(|| DEFAULT_PROFILE_ID.to_string())
}

fn save_active_id(app: &AppHandle, key: &str, id: &str) -> Result<(), String> {
    use tauri::Manager;
    let store = app
        .try_state::<tauri_plugin_store::Store<tauri::Wry>>()
        .ok_or_else(|| "Failed to access store".to_string())?;
    store.set(key, serde_json::Value::String(id.to_string()));
    store.save().map_err(|e| format!("Failed to save store: {}", e))
}

fn profiles_dir(app: &AppHandle, subdir: &str) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|d| d.join("profiles").join(subdir))
        .map_err(|e| format!("Cannot get app data dir: {}", e))
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

/// Returns the currently active profile IDs from the store.
#[tauri::command]
pub fn get_active_profiles(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ActiveProfiles, String> {
    ensure_initialized(&app, &state)?;
    Ok(ActiveProfiles {
        communication_id: get_active_id(&app, KEY_ACTIVE_COMM),
        ergonomic_id: get_active_id(&app, KEY_ACTIVE_ERGO),
    })
}

/// Switches the active communication profile by ID.
#[tauri::command]
pub fn switch_communication_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    validate_profile_id(&id)?;
    ensure_initialized(&app, &state)?;
    let dir = profiles_dir(&app, "communication")?;
    let path = dir.join(format!("{}.json", id));
    if !path.exists() {
        return Err(format!("Profile '{}' not found", id));
    }
    let profile = load_profile::<CommunicationProfile>(&path);
    state.comm_policy.lock().unwrap().set_comm_profile(profile);
    save_active_id(&app, KEY_ACTIVE_COMM, &id)?;
    log::info!("Switched communication profile to '{}'", id);
    Ok(())
}

/// Switches the active ergonomic profile by ID.
#[tauri::command]
pub fn switch_ergonomic_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    validate_profile_id(&id)?;
    ensure_initialized(&app, &state)?;
    let dir = profiles_dir(&app, "ergonomic")?;
    let path = dir.join(format!("{}.json", id));
    if !path.exists() {
        return Err(format!("Profile '{}' not found", id));
    }
    let profile = load_profile::<ErgonomicProfile>(&path);
    // Update session limits from new profile
    {
        let mut session = state.session.lock().unwrap();
        session.state.session_limit_secs = profile.limits.sitting_secs as i64;
        session.state.stand_limit_secs = profile.limits.standing_target_secs as i64;
    }
    state.comm_policy.lock().unwrap().set_ergo_profile(profile);
    save_active_id(&app, KEY_ACTIVE_ERGO, &id)?;
    log::info!("Switched ergonomic profile to '{}'", id);
    Ok(())
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
    let new_id: String = new_name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

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
