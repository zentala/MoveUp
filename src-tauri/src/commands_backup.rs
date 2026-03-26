//! commands_backup.rs — Database backup IPC commands.

use tauri::Manager;

/// Lists all available database backups (newest last).
#[tauri::command]
pub fn list_db_backups(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let backups = crate::db_backup::list_backups(&app_data_dir);
    Ok(backups
        .into_iter()
        .filter_map(|p| p.to_str().map(|s| s.to_string()))
        .collect())
}

/// Restores a database backup by path. Requires app restart to take effect.
#[tauri::command]
pub fn restore_db_backup(app: tauri::AppHandle, backup_path: String) -> Result<(), String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let path = std::path::Path::new(&backup_path);
    let backups_dir = app_data_dir.join("backups");
    if !path.starts_with(&backups_dir) {
        return Err("Backup must be inside the backups directory".to_string());
    }
    crate::db_backup::restore_backup(&app_data_dir, path)
}
