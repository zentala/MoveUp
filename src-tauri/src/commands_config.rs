//! commands_config.rs — Configuration, calibration, and debug IPC commands.

use tauri::State;

use crate::commands::{ensure_initialized, AppState};
use crate::config::AppConfig;

/// Returns the current application configuration.
#[tauri::command]
pub fn get_settings(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<AppConfig, String> {
    ensure_initialized(&app, &state)?;
    Ok(state.config.lock().unwrap().clone().unwrap_or_default())
}

/// Saves updated application configuration.
/// Validates via `config.clamped()`, writes to store, and updates SessionManager.
#[tauri::command]
pub fn save_settings(
    config: AppConfig,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    use tauri::Manager;

    ensure_initialized(&app, &state)?;

    let clamped = config.clamped();

    // Save to store
    let store_state = app
        .try_state::<tauri_plugin_store::Store<tauri::Wry>>()
        .ok_or_else(|| "Failed to access store".to_string())?;

    clamped.save(store_state.inner())?;

    // Update in-memory config
    *state.config.lock().unwrap() = Some(clamped.clone());

    // Update session manager with new calibration
    let mut session = state.session.lock().unwrap();
    session.sitting_height_cm = clamped.sitting_mm as f32 / 10.0;
    session.standing_height_cm = clamped.standing_mm as f32 / 10.0;
    session.desk_thickness_cm = clamped.desk_thickness_mm as f32 / 10.0;

    Ok(())
}

/// Updates height calibration values used by the session state machine.
///
/// All parameters are optional; only provided values are updated.
#[tauri::command]
pub fn calibrate(
    sitting_mm: Option<i32>,
    standing_mm: Option<i32>,
    desk_thickness_mm: Option<i32>,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    ensure_initialized(&app, &state)?;
    let mut session = state.session.lock().unwrap();
    if let Some(mm) = sitting_mm {
        session.sitting_height_cm = mm as f32 / 10.0;
    }
    if let Some(mm) = standing_mm {
        session.standing_height_cm = mm as f32 / 10.0;
    }
    if let Some(mm) = desk_thickness_mm {
        session.desk_thickness_cm = mm as f32 / 10.0;
    }
    Ok(())
}

/// Updates the sitting session limit.
#[tauri::command]
pub fn set_session_limit(
    minutes: u32,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    ensure_initialized(&app, &state)?;
    state.session.lock().unwrap().set_limit_minutes(minutes);
    Ok(())
}

/// Updates the standing session limit.
#[tauri::command]
pub fn set_stand_limit(
    minutes: u32,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    ensure_initialized(&app, &state)?;
    state
        .session
        .lock()
        .unwrap()
        .set_stand_limit_minutes(minutes);
    Ok(())
}

/// Returns current overlay state for debugging.
#[tauri::command]
#[cfg(debug_assertions)]
pub fn get_overlay_state(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let overlay_state = state
        .overlay
        .debug_state()
        .ok_or_else(|| "Failed to lock overlay state".to_string())?;
    Ok(overlay_state)
}
