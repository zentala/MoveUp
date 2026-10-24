//! Consistent SQLite startup backups.
//!
//! Backups are copied with SQLite's online backup API so committed WAL pages
//! are included. A destination is published and retention runs only after the
//! copy passes integrity and schema checks.

use std::{
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

use log::{info, warn};
use rusqlite::{backup::StepResult, Connection, OpenFlags};

/// Maximum number of recognized MoveUp backups to retain.
const MAX_BACKUPS: usize = 30;
const BACKUP_TIMEOUT: Duration = Duration::from_secs(10);
const BUSY_RETRY_DELAY: Duration = Duration::from_millis(20);

#[derive(Debug, PartialEq, Eq)]
pub enum BackupOutcome {
    /// First launch, before the application has created its database.
    NoExistingDatabase,
    /// A complete, validated backup was published.
    Created(PathBuf),
}

/// Creates a collision-safe, SQLite-consistent backup of an existing database.
///
/// `Ok(NoExistingDatabase)` is the normal first-launch case. Any unreadable,
/// malformed, busy, incomplete, or unpublishable database returns an error;
/// failed attempts never run retention.
pub fn backup_database(app_data_dir: &Path) -> Result<BackupOutcome, String> {
    backup_database_with_presence(app_data_dir, Path::try_exists)
}

fn backup_database_with_presence(
    app_data_dir: &Path,
    database_exists: impl FnOnce(&Path) -> std::io::Result<bool>,
) -> Result<BackupOutcome, String> {
    let db_path = app_data_dir.join("desk.db");
    let database_exists = database_exists(&db_path)
        .map_err(|error| format!("Cannot inspect existing database path: {error}"))?;
    if !database_exists {
        info!("No existing desk.db to backup");
        return Ok(BackupOutcome::NoExistingDatabase);
    }

    let source = Connection::open_with_flags(
        &db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|error| format!("Cannot open existing database for backup: {error}"))?;
    source
        .busy_timeout(BACKUP_TIMEOUT)
        .map_err(|error| format!("Cannot set database backup timeout: {error}"))?;
    let source_sessions = validate_database(&source, "source")?;

    let backups_dir = app_data_dir.join("backups");
    fs::create_dir_all(&backups_dir)
        .map_err(|error| format!("Cannot create database backup directory: {error}"))?;

    let (temporary_path, temporary_file) = create_temporary_file(&backups_dir)?;
    drop(temporary_file);
    let _temporary_cleanup = TemporaryBackup(temporary_path.clone());
    let mut destination = Connection::open(&temporary_path)
        .map_err(|error| format!("Cannot open temporary database backup: {error}"))?;
    destination
        .busy_timeout(BACKUP_TIMEOUT)
        .map_err(|error| format!("Cannot set temporary backup timeout: {error}"))?;

    {
        let backup = rusqlite::backup::Backup::new(&source, &mut destination)
            .map_err(|error| format!("Cannot start SQLite online backup: {error}"))?;
        let started = Instant::now();
        loop {
            match backup.step(128).map_err(|error| format!("SQLite backup failed: {error}"))? {
                StepResult::Done => break,
                StepResult::More => {}
                StepResult::Busy | StepResult::Locked if started.elapsed() < BACKUP_TIMEOUT => {
                    thread::sleep(BUSY_RETRY_DELAY);
                }
                StepResult::Busy | StepResult::Locked => {
                    return Err(format!(
                        "SQLite backup remained busy or locked for {} seconds",
                        BACKUP_TIMEOUT.as_secs()
                    ));
                }
                _ => unreachable!("SQLite backup returned a future step result"),
            }
        }
    }

    let journal_mode: String = destination
        .query_row("PRAGMA journal_mode=DELETE", [], |row| row.get(0))
        .map_err(|error| format!("Cannot finalize temporary database backup: {error}"))?;
    if !journal_mode.eq_ignore_ascii_case("delete") {
        return Err(format!(
            "Temporary database backup retained unexpected journal mode '{journal_mode}'"
        ));
    }
    let copied_sessions = validate_database(&destination, "temporary backup")?;
    if source_sessions > 0 && copied_sessions == 0 {
        return Err("Temporary backup lost all sessions from a nonempty source database".into());
    }

    destination
        .close()
        .map_err(|(_, error)| format!("Cannot close temporary database backup: {error}"))?;
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(&temporary_path)
        .and_then(|file| file.sync_all())
        .map_err(|error| format!("Cannot flush temporary database backup: {error}"))?;

    let backup_path = publish_without_overwrite(&temporary_path, &backups_dir)?;
    info!("DB backup created and validated: {}", backup_path.display());
    cleanup_old_backups(&backups_dir, &backup_path);
    Ok(BackupOutcome::Created(backup_path))
}

fn validate_database(connection: &Connection, label: &str) -> Result<i64, String> {
    let checks = connection
        .prepare("PRAGMA integrity_check")
        .and_then(|mut statement| {
            statement
                .query_map([], |row| row.get::<_, String>(0))?
                .collect::<Result<Vec<_>, _>>()
        })
        .map_err(|error| format!("Cannot check {label} database integrity: {error}"))?;
    if checks.len() != 1 || checks[0] != "ok" {
        return Err(format!("{label} database failed SQLite integrity_check: {checks:?}"));
    }

    let has_sessions: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='sessions')",
            [],
            |row| row.get(0),
        )
        .map_err(|error| format!("Cannot inspect {label} database schema: {error}"))?;
    if !has_sessions {
        return Err(format!("{label} database is missing the required sessions table"));
    }

    let columns = connection
        .prepare("PRAGMA table_info(sessions)")
        .and_then(|mut statement| {
            statement
                .query_map([], |row| row.get::<_, String>(1))?
                .collect::<Result<Vec<_>, _>>()
        })
        .map_err(|error| format!("Cannot inspect {label} sessions schema: {error}"))?;
    for required in ["started_at", "state"] {
        if !columns.iter().any(|column| column == required) {
            return Err(format!(
                "{label} database sessions table is missing required column '{required}'"
            ));
        }
    }

    connection
        .query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))
        .map_err(|error| format!("Cannot count sessions in {label} database: {error}"))
}

fn create_temporary_file(directory: &Path) -> Result<(PathBuf, File), String> {
    for _ in 0..8 {
        let path = directory.join(format!(".desk-backup-{}.tmp", uuid::Uuid::new_v4()));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(format!("Cannot create temporary database backup: {error}")),
        }
    }
    Err("Cannot allocate a unique temporary database backup name".into())
}

struct TemporaryBackup(PathBuf);

impl Drop for TemporaryBackup {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_file(&self.0) {
            if error.kind() != std::io::ErrorKind::NotFound {
                warn!("Failed to clean temporary database backup {}: {error}", self.0.display());
            }
        }
    }
}

fn publish_without_overwrite(temporary: &Path, backups_dir: &Path) -> Result<PathBuf, String> {
    for _ in 0..8 {
        let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let path = backups_dir.join(format!("desk-{stamp}-{}.db", uuid::Uuid::new_v4()));
        match fs::hard_link(temporary, &path) {
            Ok(()) => {
                if let Err(error) = fs::remove_file(temporary) {
                    warn!("Validated backup published but temporary link cleanup failed at {}: {error}", temporary.display());
                }
                return Ok(path);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!("Cannot publish validated database backup without overwriting: {error}"));
            }
        }
    }
    Err("Cannot allocate a unique database backup name after repeated collisions".into())
}

/// Lists recognized automatic backup files oldest first.
pub fn list_backups(app_data_dir: &Path) -> Vec<PathBuf> {
    let backups_dir = app_data_dir.join("backups");
    let mut backups: Vec<PathBuf> = match fs::read_dir(&backups_dir) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| is_managed_backup(path))
            .collect(),
        Err(_) => Vec::new(),
    };
    backups.sort();
    backups
}

fn is_managed_backup(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else { return false };
    let Some(stem) = name.strip_prefix("desk-").and_then(|name| name.strip_suffix(".db")) else {
        return false;
    };
    let Some((date, suffix)) = stem.split_once('-') else { return false };
    if date.len() != 8 || !date.bytes().all(|byte| byte.is_ascii_digit()) {
        return false;
    }
    let (time, id) = if let Some((time, id)) = suffix.split_once('-') {
        (time, Some(id))
    } else {
        (suffix, None)
    };
    time.len() == 6
        && time.bytes().all(|byte| byte.is_ascii_digit())
        && id.map_or(true, |id| uuid::Uuid::parse_str(id).is_ok())
}

/// Restores a backup by copying it over the current `desk.db`.
///
/// This legacy operation is not used by automatic backup creation and is not
/// made crash-safe by this module.
pub fn restore_backup(app_data_dir: &Path, backup_path: &Path) -> Result<(), String> {
    if !backup_path.exists() {
        return Err(format!("Backup not found: {}", backup_path.display()));
    }

    let db_path = app_data_dir.join("desk.db");
    let safety_path = app_data_dir.join("desk.db.pre-restore");
    if db_path.exists() {
        fs::copy(&db_path, &safety_path)
            .map_err(|error| format!("Failed to create safety copy: {error}"))?;
    }
    fs::copy(backup_path, &db_path)
        .map_err(|error| format!("Failed to restore backup: {error}"))?;
    info!("DB restored from: {} (safety copy at desk.db.pre-restore)", backup_path.display());
    Ok(())
}

fn cleanup_old_backups(backups_dir: &Path, keep: &Path) {
    let mut backups: Vec<PathBuf> = match fs::read_dir(backups_dir) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| is_managed_backup(path))
            .collect(),
        Err(error) => {
            warn!("Cannot list database backups for retention: {error}");
            return;
        }
    };
    if backups.len() <= MAX_BACKUPS {
        return;
    }

    backups.sort();
    let remove_count = backups.len() - MAX_BACKUPS;
    for path in backups.iter().filter(|path| path.as_path() != keep).take(remove_count) {
        if let Err(error) = fs::remove_file(path) {
            warn!("Failed to remove old backup {}: {error}", path.display());
        } else {
            info!("Removed old backup: {}", path.display());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn make_database(directory: &Path, rows: i64) -> (PathBuf, Connection) {
        let path = directory.join("desk.db");
        let connection = Connection::open(&path).unwrap();
        crate::db::init_schema(&connection).unwrap();
        for index in 0..rows {
            connection
                .execute(
                    "INSERT INTO sessions (started_at, state, duration_seconds) VALUES (?1, 'Sitting', ?2)",
                    rusqlite::params![format!("2026-10-24T08:{index:02}:00Z"), index],
                )
                .unwrap();
        }
        (path, connection)
    }

    fn created(result: Result<BackupOutcome, String>) -> PathBuf {
        match result.unwrap() {
            BackupOutcome::Created(path) => path,
            BackupOutcome::NoExistingDatabase => panic!("expected a backup"),
        }
    }

    #[test]
    fn backup_includes_committed_wal_data_without_changing_the_source() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("desk.db");
        let source = Connection::open(&db_path).unwrap();
        let mode: String = source.query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0)).unwrap();
        assert_eq!(mode.to_ascii_lowercase(), "wal");
        crate::db::init_schema(&source).unwrap();
        source.execute(
            "INSERT INTO sessions (started_at, state, duration_seconds) VALUES ('2026-10-24T08:00:00Z', 'Sitting', 42)",
            [],
        ).unwrap();
        let source_count: i64 = source.query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0)).unwrap();
        assert_eq!(source_count, 1);
        let main_before = fs::read(&db_path).unwrap();
        let wal_path = tmp.path().join("desk.db-wal");
        assert!(wal_path.exists(), "fixture must retain committed pages in WAL");
        let wal_before = fs::read(&wal_path).unwrap();

        let backup_path = created(backup_database(tmp.path()));
        let backup = Connection::open(&backup_path).unwrap();
        let backup_count: i64 = backup.query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0)).unwrap();
        assert_eq!(backup_count, source_count, "backup must include committed WAL rows");
        let integrity: String = backup.query_row("PRAGMA integrity_check", [], |row| row.get(0)).unwrap();
        assert_eq!(integrity, "ok");
        assert_eq!(fs::read(&db_path).unwrap(), main_before, "backup must leave the main source file unchanged");
        assert_eq!(fs::read(&wal_path).unwrap(), wal_before, "backup must leave the WAL source file unchanged");
    }

    #[test]
    fn absent_database_is_reported_as_first_launch_not_failure() {
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(backup_database(tmp.path()).unwrap(), BackupOutcome::NoExistingDatabase);
        assert!(!tmp.path().join("backups").exists());
    }

    #[test]
    fn database_metadata_error_is_not_absence_and_does_not_publish_or_prune() {
        let absent = tempfile::tempdir().unwrap();
        let absent_result = backup_database_with_presence(absent.path(), |_| Ok(false));
        assert_eq!(absent_result.unwrap(), BackupOutcome::NoExistingDatabase);
        assert!(!absent.path().join("backups").exists());

        let unreadable = tempfile::tempdir().unwrap();
        let backups_dir = unreadable.path().join("backups");
        fs::create_dir_all(&backups_dir).unwrap();
        for index in 0..31 {
            fs::write(
                backups_dir.join(format!("desk-20260325-{index:06}.db")),
                b"last-good",
            )
            .unwrap();
        }
        let mut before: Vec<_> = fs::read_dir(&backups_dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        before.sort();
        let result = backup_database_with_presence(unreadable.path(), |_| {
            Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "injected metadata denial"))
        });
        let mut after: Vec<_> = fs::read_dir(&backups_dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        after.sort();

        assert!(result.is_err(), "metadata errors must remain backup errors: {result:?}");
        assert!(result.unwrap_err().contains("injected metadata denial"));
        assert_eq!(after, before, "metadata errors must not publish or prune backups");
    }

    #[test]
    fn empty_and_corrupt_sources_fail_without_pruning_existing_backups() {
        for bytes in [Vec::new(), b"not a SQLite database".to_vec()] {
            let tmp = tempfile::tempdir().unwrap();
            let db = tmp.path().join("desk.db");
            fs::write(&db, &bytes).unwrap();
            let backups_dir = tmp.path().join("backups");
            fs::create_dir_all(&backups_dir).unwrap();
            let protected = backups_dir.join("desk-protected.db");
            fs::write(&protected, b"keep").unwrap();
            let result = backup_database(tmp.path());
            assert!(result.is_err(), "invalid source must not be reported as a backup: {result:?}");
            assert_eq!(fs::read(&db).unwrap(), bytes, "source bytes remain unchanged");
            assert_eq!(fs::read(&protected).unwrap(), b"keep");
            assert!(list_backups(tmp.path()).is_empty());
        }
    }

    #[test]
    fn backup_directory_io_failure_is_truthful_and_keeps_source() {
        let tmp = tempfile::tempdir().unwrap();
        let (db, source) = make_database(tmp.path(), 1);
        let source_count: i64 = source.query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0)).unwrap();
        drop(source);
        fs::write(tmp.path().join("backups"), b"not a directory").unwrap();
        assert!(backup_database(tmp.path()).is_err());
        let reopened = Connection::open(db).unwrap();
        let after: i64 = reopened.query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0)).unwrap();
        assert_eq!(after, source_count);
    }

    #[test]
    fn repeated_backups_use_distinct_names_and_keep_unrelated_files() {
        let tmp = tempfile::tempdir().unwrap();
        let (_db, source) = make_database(tmp.path(), 1);
        let first = created(backup_database(tmp.path()));
        let second = created(backup_database(tmp.path()));
        assert_ne!(first, second);
        let protected = tmp.path().join("backups/desk-protected.db");
        fs::write(&protected, b"keep").unwrap();
        assert_eq!(list_backups(tmp.path()).len(), 2);
        assert_eq!(fs::read(&protected).unwrap(), b"keep");
        drop(source);
    }

    #[test]
    fn retention_runs_only_after_a_verified_copy_and_preserves_unrelated_files() {
        let tmp = tempfile::tempdir().unwrap();
        let (_db, source) = make_database(tmp.path(), 1);
        let backups_dir = tmp.path().join("backups");
        fs::create_dir_all(&backups_dir).unwrap();
        for i in 0..35 {
            fs::write(backups_dir.join(format!("desk-20260325-{i:06}.db")), b"old").unwrap();
        }
        let protected = backups_dir.join("desk-protected.db");
        fs::write(&protected, b"keep").unwrap();
        assert!(created(backup_database(tmp.path())).exists());
        assert_eq!(list_backups(tmp.path()).len(), MAX_BACKUPS);
        assert_eq!(fs::read(protected).unwrap(), b"keep");
        drop(source);
    }

    #[test]
    fn retention_never_deletes_the_backup_just_published() {
        let tmp = tempfile::tempdir().unwrap();
        let backups_dir = tmp.path().join("backups");
        fs::create_dir_all(&backups_dir).unwrap();
        let keep = backups_dir.join("desk-20260325-000000-00000000-0000-4000-8000-000000000000.db");
        fs::write(&keep, b"new verified copy").unwrap();
        for i in 1..=30 {
            fs::write(backups_dir.join(format!("desk-20260325-{i:06}.db")), b"older copy").unwrap();
        }

        cleanup_old_backups(&backups_dir, &keep);

        assert!(keep.exists(), "retention must not remove the just-published backup even if it sorts first");
        assert_eq!(list_backups(tmp.path()).len(), MAX_BACKUPS);
    }

    #[test]
    fn cleanup_keeps_max_backups() {
        let tmp = tempfile::tempdir().unwrap();
        let backups_dir = tmp.path().join("backups");
        fs::create_dir_all(&backups_dir).unwrap();

        for i in 0..35 {
            let name = format!("desk-20260325-{i:06}.db");
            fs::write(backups_dir.join(name), b"x").unwrap();
        }

        cleanup_old_backups(&backups_dir, Path::new(""));
        let remaining: Vec<_> = fs::read_dir(&backups_dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .collect();
        assert_eq!(remaining.len(), MAX_BACKUPS);
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
}
