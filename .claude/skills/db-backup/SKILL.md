---
name: db-backup
description: List, create, and restore desk app database backups. Use when user reports lost data, DB reset, or wants to recover previous state.
user_invocable: true
---

# Database Backup & Restore

## How it works

The desk app automatically backs up `desk.db` on every startup:
- Location: `{AppData}/io.zntl.desk/backups/desk-YYYYMMDD-HHMMSS.db`
- Max 30 backups retained (oldest auto-deleted)
- Never overwrites previous backups

## Commands

### List backups

```bash
ls -la "$(echo $APPDATA)/io.zntl.desk/backups/" 2>/dev/null || echo "No backups found"
```

Show the user the list with dates and sizes. Most recent = best candidate for restore.

### Restore a backup

**IMPORTANT:** The app must be STOPPED before restoring. Otherwise the running app
will overwrite the restored DB.

1. Stop the app (`pnpm tauri:dev` process)
2. Copy the backup over the current DB:
   ```bash
   APP_DIR="$(echo $APPDATA)/io.zntl.desk"
   # Safety: backup current DB first
   cp "$APP_DIR/desk.db" "$APP_DIR/desk.db.pre-restore"
   # Restore chosen backup
   cp "$APP_DIR/backups/desk-YYYYMMDD-HHMMSS.db" "$APP_DIR/desk.db"
   ```
3. Restart the app

### Manual backup (outside of app startup)

```bash
APP_DIR="$(echo $APPDATA)/io.zntl.desk"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
mkdir -p "$APP_DIR/backups"
cp "$APP_DIR/desk.db" "$APP_DIR/backups/desk-$TIMESTAMP.db"
echo "Backup created: desk-$TIMESTAMP.db"
```

### Check DB location and size

```bash
APP_DIR="$(echo $APPDATA)/io.zntl.desk"
echo "DB path: $APP_DIR/desk.db"
ls -la "$APP_DIR/desk.db" 2>/dev/null || echo "No desk.db found!"
echo ""
echo "Backups:"
ls -la "$APP_DIR/backups/" 2>/dev/null || echo "No backups directory"
```

### Inspect DB contents (quick sanity check)

```bash
APP_DIR="$(echo $APPDATA)/io.zntl.desk"
sqlite3 "$APP_DIR/desk.db" "SELECT COUNT(*) as sessions, MIN(started_at) as first, MAX(started_at) as last FROM sessions;"
```

## Rust API (for IPC commands)

Two Tauri commands are registered:
- `list_db_backups` — returns `Vec<String>` of backup file paths
- `restore_db_backup(backup_path: String)` — restores backup (validates path is inside backups dir)

Source: `src-tauri/src/db_backup.rs`

## When to use this skill

- User says "baza się zresetowała" / "straciłem dane" / "DB is empty"
- User wants to check if backups exist
- User wants to restore data from before a known-good point
- After a failed migration or schema change
