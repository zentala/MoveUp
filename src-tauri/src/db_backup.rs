//! db_backup.rs — Automatic database backup on startup.
//!
//! Creates timestamped copies of `desk.db` in a `backups/` subdirectory
//! inside the app data folder. Never overwrites previous backups.
//! Retains last 30 backups (oldest are cleaned up).

use std::path::{Path, PathBuf};

use log::{info, warn};

/// Maximum number of backup files to retain.
const MAX_BACKUPS: usize = 30;

/// Creates a timestamped backup of the database file.
///
/// Backup path: `{app_data_dir}/backups/desk-YYYYMMDD-HHMMSS.db`
/// Returns the backup path on success, or None if the source doesn't exist.
pub fn backup_database(app_data_dir: &Path) -> Option<PathBuf> {
    let db_path = app_data_dir.join("desk.db");
    if !db_path.exists() {
        info!("No existing desk.db to backup");
        return None;
    }

    let backups_dir = app_data_dir.join("backups");
    if let Err(e) = std::fs::create_dir_all(&backups_dir) {
        warn!("Failed to create backups dir: {}", e);
        return None;
    }

    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let backup_name = format!("desk-{}.db", timestamp);
    let backup_path = backups_dir.join(&backup_name);

    match std::fs::copy(&db_path, &backup_path) {
        Ok(bytes) => {
            info!(
                "DB backup created: {} ({} bytes)",
                backup_path.display(),
                bytes
            );
            cleanup_old_backups(&backups_dir);
            Some(backup_path)
        }
        Err(e) => {
            warn!("Failed to backup desk.db: {}", e);
            None
        }
    }
}

/// Lists all backup files sorted by name (oldest first).
pub fn list_backups(app_data_dir: &Path) -> Vec<PathBuf> {
    let backups_dir = app_data_dir.join("backups");
    let mut backups: Vec<PathBuf> = std::fs::read_dir(&backups_dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension().map_or(false, |ext| ext == "db")
                && p.file_name()
                    .map_or(false, |n| n.to_string_lossy().starts_with("desk-"))
        })
        .collect();
    backups.sort();
    backups
}

/// Restores a backup by copying it over the current `desk.db`.
///
/// Returns Ok(()) on success, Err with message on failure.
pub fn restore_backup(app_data_dir: &Path, backup_path: &Path) -> Result<(), String> {
    if !backup_path.exists() {
        return Err(format!("Backup not found: {}", backup_path.display()));
    }

    let db_path = app_data_dir.join("desk.db");

    // Safety: backup current DB before overwriting (in case restore file is corrupt)
    let safety_path = app_data_dir.join("desk.db.pre-restore");
    if db_path.exists() {
        std::fs::copy(&db_path, &safety_path)
            .map_err(|e| format!("Failed to create safety copy: {}", e))?;
    }

    std::fs::copy(backup_path, &db_path)
        .map_err(|e| format!("Failed to restore backup: {}", e))?;

    info!(
        "DB restored from: {} (safety copy at desk.db.pre-restore)",
        backup_path.display()
    );
    Ok(())
}

/// Removes oldest backups when count exceeds MAX_BACKUPS.
fn cleanup_old_backups(backups_dir: &Path) {
    let mut backups: Vec<PathBuf> = std::fs::read_dir(backups_dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension().map_or(false, |ext| ext == "db")
                && p.file_name()
                    .map_or(false, |n| n.to_string_lossy().starts_with("desk-"))
        })
        .collect();

    if backups.len() <= MAX_BACKUPS {
        return;
    }

    backups.sort();
    let to_remove = backups.len() - MAX_BACKUPS;
    for path in backups.iter().take(to_remove) {
        if let Err(e) = std::fs::remove_file(path) {
            warn!("Failed to remove old backup {}: {}", path.display(), e);
        } else {
            info!("Removed old backup: {}", path.display());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn backup_creates_file() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("desk.db");
        fs::write(&db_path, b"test data").unwrap();

        let result = backup_database(tmp.path());
        assert!(result.is_some());
        let backup = result.unwrap();
        assert!(backup.exists());
        assert_eq!(fs::read(&backup).unwrap(), b"test data");
    }

    #[test]
    fn backup_returns_none_when_no_db() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(backup_database(tmp.path()).is_none());
    }

    #[test]
    fn list_backups_finds_files() {
        let tmp = tempfile::tempdir().unwrap();
        let backups_dir = tmp.path().join("backups");
        fs::create_dir_all(&backups_dir).unwrap();
        fs::write(backups_dir.join("desk-20260325-120000.db"), b"a").unwrap();
        fs::write(backups_dir.join("desk-20260325-130000.db"), b"b").unwrap();
        fs::write(backups_dir.join("other.txt"), b"c").unwrap();

        let list = list_backups(tmp.path());
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn restore_copies_backup_over_db() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("desk.db");
        fs::write(&db_path, b"current").unwrap();

        let backups_dir = tmp.path().join("backups");
        fs::create_dir_all(&backups_dir).unwrap();
        let backup = backups_dir.join("desk-20260325-120000.db");
        fs::write(&backup, b"old data").unwrap();

        restore_backup(tmp.path(), &backup).unwrap();
        assert_eq!(fs::read(&db_path).unwrap(), b"old data");
        assert!(tmp.path().join("desk.db.pre-restore").exists());
    }

    #[test]
    fn cleanup_keeps_max_backups() {
        let tmp = tempfile::tempdir().unwrap();
        let backups_dir = tmp.path().join("backups");
        fs::create_dir_all(&backups_dir).unwrap();

        for i in 0..35 {
            let name = format!("desk-20260325-{:06}.db", i);
            fs::write(backups_dir.join(name), b"x").unwrap();
        }

        cleanup_old_backups(&backups_dir);
        let remaining: Vec<_> = fs::read_dir(&backups_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(remaining.len(), MAX_BACKUPS);
    }
}
