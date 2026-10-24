#!/usr/bin/env python3
"""Create a protected, verified SQLite copy for the additive history upgrade.

This tool only prepares a backup. The application remains the sole schema
migrator; this script never changes the source database.
"""

from __future__ import annotations

import argparse
import json
import os
import sqlite3
import sys
from datetime import datetime, timezone
from pathlib import Path


EXPECTED_SESSION_COLUMNS = (
    "id",
    "started_at",
    "ended_at",
    "state",
    "duration_seconds",
    "sitting_seconds",
    "standing_seconds",
    "position_changes",
    "session_limit_secs",
    "break_credit",
    "date_local",
)
DESTINATION_PREFIX = "moveup-history-preflight-"
PRODUCTION_IDENTITIES = (
    ("appdata", "roaming", "io.zntl.desk"),
    ("appdata", "local", "io.zntl.desk"),
    ("appdata", "roaming", "moveup"),
    ("appdata", "local", "moveup"),
    ("appdata", "moveup"),
)
DATABASE_NAME = "desk.db"
SETTINGS_NAME = "settings.json"
CURRENT_HISTORY_KIND_COLUMN = "history_kind"
CURRENT_SESSION_COLUMN_SHAPES = {
    "id": ("INTEGER", 0, None, 1),
    "started_at": ("TEXT", 1, None, 0),
    "ended_at": ("TEXT", 0, None, 0),
    "state": ("TEXT", 1, None, 0),
    "duration_seconds": ("INTEGER", 0, None, 0),
    "sitting_seconds": ("INTEGER", 0, "0", 0),
    "standing_seconds": ("INTEGER", 0, "0", 0),
    "position_changes": ("INTEGER", 0, "0", 0),
    "session_limit_secs": ("INTEGER", 0, "2700", 0),
    "date_local": ("TEXT", 0, None, 0),
    "break_credit": ("TEXT", 0, None, 0),
    "history_kind": ("TEXT", 0, None, 0),
}


class PreparationError(Exception):
    """A safe refusal to prepare a backup."""


class PartialPreparationError(PreparationError):
    """The settings half is verified but database backup failed afterward."""

    def __init__(self, message: str, result: dict[str, object]) -> None:
        super().__init__(message)
        self.result = result


def _inside(path: Path, parent: Path) -> bool:
    try:
        path.relative_to(parent)
        return True
    except ValueError:
        return False


def _session_column_rows(connection: sqlite3.Connection) -> tuple[tuple[object, ...], ...]:
    try:
        rows = connection.execute("PRAGMA table_info(sessions)").fetchall()
    except sqlite3.DatabaseError as exc:
        raise PreparationError(f"Cannot inspect sessions schema: {exc}") from exc
    if not rows:
        raise PreparationError("Required sessions table is missing")
    return tuple(tuple(row) for row in rows)


def _check_schema(
    connection: sqlite3.Connection, *, current_schema_backup_only: bool = False
) -> tuple[str, ...]:
    rows = _session_column_rows(connection)
    columns = tuple(str(row[1]) for row in rows)
    if CURRENT_HISTORY_KIND_COLUMN in columns:
        if not current_schema_backup_only:
            raise PreparationError(
                "Schema is not eligible: history_kind is already present; stop for review"
            )
        expected_current = set(EXPECTED_SESSION_COLUMNS) | {CURRENT_HISTORY_KIND_COLUMN}
        if len(columns) != len(expected_current) or set(columns) != expected_current:
            raise PreparationError(
                "Current backup-only schema is not exact: sessions columns were "
                f"{columns!r}; expected the reviewed fields "
                f"{tuple(EXPECTED_SESSION_COLUMNS) + (CURRENT_HISTORY_KIND_COLUMN,)!r}"
            )
        # PRAGMA table_info: cid, name, type, notnull, dflt_value, pk.
        # Compare by name because supported CREATE/ALTER histories yield
        # different valid column ordinals.
        for row in rows:
            name = str(row[1])
            actual_shape = (
                str(row[2]).strip().upper(),
                row[3],
                row[4],
                row[5],
            )
            expected_shape = CURRENT_SESSION_COLUMN_SHAPES[name]
            if actual_shape != expected_shape:
                raise PreparationError(
                    "Current backup-only schema has unsupported semantics for "
                    f"{name}: found {actual_shape!r}; expected {expected_shape!r}"
                )
        return columns

    if current_schema_backup_only:
        raise PreparationError(
            "Current backup-only mode requires the reviewed history_kind TEXT column"
        )
    if len(columns) != len(EXPECTED_SESSION_COLUMNS) or set(columns) != set(
        EXPECTED_SESSION_COLUMNS
    ):
        raise PreparationError(
            "Schema is not eligible for the reviewed additive-only upgrade "
            f"(sessions columns were {columns!r}; expected fields "
            f"{EXPECTED_SESSION_COLUMNS!r})"
        )
    return columns


def _read_only_uri(path: Path) -> str:
    return f"{path.as_uri()}?mode=ro"


def _contains_sequence(parts: tuple[str, ...], sequence: tuple[str, ...]) -> bool:
    return any(parts[index : index + len(sequence)] == sequence for index in range(len(parts)))


def _validate_destination(
    sources: tuple[Path, ...], destination_dir: Path
) -> Path:
    if not destination_dir.exists() or not destination_dir.is_dir():
        raise PreparationError("Destination directory must already exist")
    resolved_dir = destination_dir.resolve(strict=True)
    if not resolved_dir.name.lower().startswith(DESTINATION_PREFIX):
        raise PreparationError(
            f"Destination directory must be a dedicated {DESTINATION_PREFIX}… location"
        )

    for source in sources:
        source_parent = source.parent.resolve(strict=True)
        if _inside(resolved_dir, source_parent) or _inside(source_parent, resolved_dir):
            raise PreparationError(
                "Destination must not overlap either source directory or a pruning tree"
            )
    if any(part.lower() == "backups" for part in resolved_dir.parts):
        raise PreparationError("Automatic backup/pruning directories are not allowed")
    lowered_parts = tuple(part.lower() for part in resolved_dir.parts)
    if any(_contains_sequence(lowered_parts, identity) for identity in PRODUCTION_IDENTITIES):
        raise PreparationError(
            "The Tauri production data directory io.zntl.desk is not a backup destination"
        )

    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S.%fZ")
    recovery_set = resolved_dir / f"{DESTINATION_PREFIX}{stamp}"
    if recovery_set.exists():
        raise PreparationError(f"Refusing to overwrite existing recovery set: {recovery_set}")
    return recovery_set


def _resolve_file(path_arg: Path, label: str) -> Path:
    try:
        path = path_arg.resolve(strict=True)
    except OSError as exc:
        raise PreparationError(f"{label} is unavailable: {exc}") from exc
    if not path.is_file():
        raise PreparationError(f"{label} must be an existing regular file")
    return path


def _inspect_database(
    source: Path, *, current_schema_backup_only: bool = False
) -> tuple[tuple[str, ...], int]:
    connection: sqlite3.Connection | None = None
    try:
        connection = sqlite3.connect(_read_only_uri(source), uri=True, timeout=5.0)
        columns = _check_schema(
            connection, current_schema_backup_only=current_schema_backup_only
        )
        count = connection.execute("SELECT COUNT(*) FROM sessions").fetchone()[0]
        if count <= 0:
            raise PreparationError("Refusing an empty sessions table")
        return columns, count
    except sqlite3.DatabaseError as exc:
        raise PreparationError(f"SQLite source validation failed: {exc}") from exc
    finally:
        if connection is not None:
            connection.close()


def _validate_settings_json(content: bytes) -> None:
    try:
        value = json.loads(content)
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise PreparationError("Settings source is not valid JSON") from exc
    if not isinstance(value, dict):
        raise PreparationError("Settings JSON root must be an object")


def _read_settings_source(settings_source: Path) -> bytes:
    try:
        content = settings_source.read_bytes()
    except OSError as exc:
        raise PreparationError(f"Settings source could not be read: {exc}") from exc
    _validate_settings_json(content)
    return content


def _write_settings_copy(recovery_set: Path, content: bytes) -> Path:
    destination = recovery_set / SETTINGS_NAME
    try:
        descriptor = os.open(
            destination, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600
        )
        with os.fdopen(descriptor, "wb") as output:
            written = output.write(content)
            output.flush()
            os.fsync(output.fileno())
        if written != len(content) or destination.read_bytes() != content:
            raise PreparationError("Settings byte-copy verification failed")
        _validate_settings_json(destination.read_bytes())
        return destination
    except OSError as exc:
        raise PreparationError(f"Settings destination could not be written: {exc}") from exc


def _write_database_copy(
    source: Path,
    destination: Path,
    expected_columns: tuple[str, ...],
    expected_count: int,
    *,
    current_schema_backup_only: bool = False,
) -> int:
    source_connection: sqlite3.Connection | None = None
    backup_connection: sqlite3.Connection | None = None
    reserved = False
    verified = False
    try:
        fd = os.open(destination, os.O_CREAT | os.O_EXCL | os.O_RDWR, 0o600)
        os.close(fd)
        reserved = True

        source_connection = sqlite3.connect(
            _read_only_uri(source), uri=True, timeout=5.0
        )
        columns = _check_schema(
            source_connection, current_schema_backup_only=current_schema_backup_only
        )
        source_count_before = source_connection.execute(
            "SELECT COUNT(*) FROM sessions"
        ).fetchone()[0]
        if columns != expected_columns or source_count_before != expected_count:
            raise PreparationError("Database source changed after preflight; no complete set created")

        backup_connection = sqlite3.connect(destination, timeout=5.0)
        source_connection.backup(backup_connection)
        backup_connection.commit()

        integrity = tuple(
            row[0] for row in backup_connection.execute("PRAGMA integrity_check")
        )
        if integrity != ("ok",):
            raise PreparationError(f"Database backup integrity check failed: {integrity!r}")
        copied_columns = _check_schema(
            backup_connection, current_schema_backup_only=current_schema_backup_only
        )
        copied_count = backup_connection.execute(
            "SELECT COUNT(*) FROM sessions"
        ).fetchone()[0]
        source_count_after = source_connection.execute(
            "SELECT COUNT(*) FROM sessions"
        ).fetchone()[0]
        if (
            copied_columns != columns
            or copied_count != source_count_before
            or source_count_after != source_count_before
        ):
            raise PreparationError("Database backup schema/count validation failed")
        verified = True
        return copied_count
    except sqlite3.DatabaseError as exc:
        raise PreparationError(f"SQLite backup validation failed: {exc}") from exc
    except OSError as exc:
        raise PreparationError(f"Database destination could not be written: {exc}") from exc
    finally:
        if backup_connection is not None:
            backup_connection.close()
        if source_connection is not None:
            source_connection.close()
        if reserved:
            # The caller retains the verified settings half on a later DB failure.
            # Remove only this attempt's unverified database file.
            try:
                if not verified:
                    destination.unlink(missing_ok=True)
            except OSError:
                pass


def prepare_backup(
    source_arg: Path,
    settings_source_arg: Path,
    destination_dir_arg: Path,
    *,
    current_schema_backup_only: bool = False,
) -> dict[str, object]:
    source = _resolve_file(source_arg, "Source database")
    settings_source = _resolve_file(settings_source_arg, "Settings source")
    if os.path.normcase(str(source)) == os.path.normcase(str(settings_source)):
        raise PreparationError("Database and settings sources must be different files")

    recovery_set = _validate_destination(
        (source, settings_source), destination_dir_arg
    )
    expected_columns, expected_count = _inspect_database(
        source, current_schema_backup_only=current_schema_backup_only
    )
    settings_bytes = _read_settings_source(settings_source)

    try:
        # mkdir is exclusive: no existing recovery set or last-good copy is touched.
        recovery_set.mkdir(mode=0o700)
    except FileExistsError as exc:
        raise PreparationError(f"Refusing to overwrite existing recovery set: {recovery_set}") from exc
    except OSError as exc:
        raise PreparationError(f"Recovery-set directory could not be created: {exc}") from exc

    settings_copy = recovery_set / SETTINGS_NAME
    try:
        _write_settings_copy(recovery_set, settings_bytes)
    except PreparationError as exc:
        # The settings half did not complete verification; remove only this new set.
        cleanup_failed = False
        try:
            if settings_copy.exists():
                settings_copy.unlink()
            recovery_set.rmdir()
        except OSError:
            cleanup_failed = True
        if cleanup_failed:
            partial_result: dict[str, object] = {
                "status": "partial",
                "recovery_set": str(recovery_set),
                "available": [],
                "missing": [DATABASE_NAME, SETTINGS_NAME],
                "error": str(exc),
            }
            if settings_copy.exists():
                partial_result["unverified_partial"] = [SETTINGS_NAME]
            raise PartialPreparationError(str(exc), partial_result) from exc
        raise

    database_copy = recovery_set / DATABASE_NAME
    try:
        copied_count = _write_database_copy(
            source,
            database_copy,
            expected_columns,
            expected_count,
            current_schema_backup_only=current_schema_backup_only,
        )
    except PreparationError as exc:
        result: dict[str, object] = {
            "status": "partial",
            "recovery_set": str(recovery_set),
            "available": [SETTINGS_NAME],
            "missing": [DATABASE_NAME],
            "error": str(exc),
        }
        if database_copy.exists():
            result["unverified_partial"] = [DATABASE_NAME]
            result["missing"] = []
        raise PartialPreparationError(str(exc), result) from exc

    result: dict[str, object] = {
        "status": "complete",
        "recovery_set": str(recovery_set),
        "database": DATABASE_NAME,
        "settings": SETTINGS_NAME,
        "created_at_utc": datetime.now(timezone.utc).isoformat(),
        "sessions": copied_count,
        "mode": (
            "backup-only-current-schema"
            if current_schema_backup_only
            else "legacy-upgrade-preparation"
        ),
        "migration": "not-performed",
        "integrity_check": "ok",
    }
    if current_schema_backup_only:
        result["source_schema"] = "reviewed current sessions columns; history_kind TEXT NULL"
        result["backup_only"] = True
    else:
        result["eligible_schema"] = "date_local+break_credit; history_kind absent"
    return result


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", required=True, type=Path, help="SQLite source file")
    parser.add_argument(
        "--settings-source", required=True, type=Path, help="JSON settings source file"
    )
    parser.add_argument(
        "--destination-dir",
        required=True,
        type=Path,
        help=(
            "existing protected directory outside the source tree, named "
            f"{DESTINATION_PREFIX}…"
        ),
    )
    parser.add_argument(
        "--current-schema-backup-only",
        action="store_true",
        help=(
            "prepare a protected copy of the exact current sessions schema; "
            "this mode performs no migration and is not legacy-upgrade eligibility"
        ),
    )
    args = parser.parse_args(argv)
    try:
        result = prepare_backup(
            args.source,
            args.settings_source,
            args.destination_dir,
            current_schema_backup_only=args.current_schema_backup_only,
        )
    except PartialPreparationError as exc:
        print(json.dumps(exc.result, indent=2), file=sys.stderr)
        return 2
    except PreparationError as exc:
        print(f"Preparation refused: {exc}", file=sys.stderr)
        return 2
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
