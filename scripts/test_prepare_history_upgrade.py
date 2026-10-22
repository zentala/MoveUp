"""Disposable tests for the protected two-part history recovery set."""

from __future__ import annotations

import contextlib
import importlib.util
import io
import json
import sqlite3
import tempfile
import unittest
from datetime import datetime, timezone
from pathlib import Path
from unittest import mock


SCRIPT = Path(__file__).with_name("prepare-history-upgrade.py")
SPEC = importlib.util.spec_from_file_location("prepare_history_upgrade", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
prepare = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(prepare)


CREATE_SESSIONS = """
CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    started_at TEXT NOT NULL,
    ended_at TEXT,
    state TEXT NOT NULL,
    duration_seconds INTEGER,
    sitting_seconds INTEGER DEFAULT 0,
    standing_seconds INTEGER DEFAULT 0,
    position_changes INTEGER DEFAULT 0,
    session_limit_secs INTEGER DEFAULT 2700,
    date_local TEXT,
    break_credit TEXT
)
"""
SETTINGS_BYTES = b'{\n  "private-value": "DO_NOT_PRINT", "format": [1, 2]\n}\n'


class FixedDateTime(datetime):
    @classmethod
    def now(cls, tz: timezone | None = None) -> datetime:
        value = cls(2026, 10, 22, 12, 30, 45, 123456, tzinfo=timezone.utc)
        return value.astimezone(tz) if tz else value


class PrepareHistoryUpgradeTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        self.source_dir = self.root / "application-data"
        self.source_dir.mkdir()
        self.destination_dir = self.root / "moveup-history-preflight-test"
        self.destination_dir.mkdir()
        self.source = self.source_dir / "desk.db"
        self.settings_source = self.source_dir / "settings.json"
        self.settings_source.write_bytes(SETTINGS_BYTES)
        self.source_writer: sqlite3.Connection | None = None

    def tearDown(self) -> None:
        if self.source_writer is not None:
            self.source_writer.close()
        self.temporary.cleanup()

    def create_database(self, rows: int = 3, *, wal: bool = False) -> None:
        connection = sqlite3.connect(self.source)
        if wal:
            mode = connection.execute("PRAGMA journal_mode=WAL").fetchone()[0]
            self.assertEqual(mode.lower(), "wal")
        connection.execute(CREATE_SESSIONS)
        connection.executemany(
            "INSERT INTO sessions (started_at, state, date_local, break_credit) "
            "VALUES (?, ?, ?, ?)",
            [
                (f"2026-10-22T12:{index:02d}:00Z", "Sitting", "2026-10-22", None)
                for index in range(rows)
            ],
        )
        connection.commit()
        self.source_writer = connection if wal else None
        if not wal:
            connection.close()

    def prepare(self) -> dict[str, object]:
        return prepare.prepare_backup(
            self.source, self.settings_source, self.destination_dir
        )

    def test_complete_set_copies_wal_database_and_exact_settings_bytes(self) -> None:
        self.create_database(wal=True)
        self.assertEqual(
            [row[1] for row in self.source_writer.execute("PRAGMA table_info(sessions)")][-2:],
            ["date_local", "break_credit"],
        )
        before = self.source_writer.execute(
            "SELECT id, started_at, state, date_local, break_credit FROM sessions ORDER BY id"
        ).fetchall()

        result = self.prepare()

        recovery_set = Path(str(result["recovery_set"]))
        self.assertEqual(result["status"], "complete")
        self.assertEqual(result["sessions"], 3)
        self.assertEqual(result["integrity_check"], "ok")
        self.assertEqual(set(path.name for path in recovery_set.iterdir()), {"desk.db", "settings.json"})
        self.assertEqual((recovery_set / "settings.json").read_bytes(), SETTINGS_BYTES)
        self.assertEqual(self.settings_source.read_bytes(), SETTINGS_BYTES)
        self.assertNotIn("DO_NOT_PRINT", json.dumps(result))

        copied = sqlite3.connect(recovery_set / "desk.db")
        self.assertEqual(
            copied.execute(
                "SELECT id, started_at, state, date_local, break_credit "
                "FROM sessions ORDER BY id"
            ).fetchall(),
            before,
        )
        copied.close()
        after = self.source_writer.execute(
            "SELECT id, started_at, state, date_local, break_credit FROM sessions ORDER BY id"
        ).fetchall()
        self.assertEqual(after, before)
        self.source_writer.close()

    def test_repeated_preparation_creates_distinct_complete_sets(self) -> None:
        self.create_database()
        first = Path(str(self.prepare()["recovery_set"]))
        second = Path(str(self.prepare()["recovery_set"]))
        self.assertNotEqual(first, second)
        for recovery_set in (first, second):
            self.assertEqual(
                {path.name for path in recovery_set.iterdir()}, {"desk.db", "settings.json"}
            )
            self.assertEqual((recovery_set / "settings.json").read_bytes(), SETTINGS_BYTES)

    def test_existing_recovery_set_is_never_overwritten(self) -> None:
        self.create_database()
        existing = self.destination_dir / (
            f"{prepare.DESTINATION_PREFIX}20261022T123045.123456Z"
        )
        existing.mkdir()
        sentinel = existing / "last-good.txt"
        sentinel.write_bytes(b"preserve this prior set")
        with mock.patch.object(prepare, "datetime", FixedDateTime):
            with self.assertRaisesRegex(prepare.PreparationError, "overwrite existing recovery set"):
                self.prepare()
        self.assertEqual(sentinel.read_bytes(), b"preserve this prior set")
        self.assertEqual(list(existing.iterdir()), [sentinel])

    def test_empty_and_ineligible_database_schemas_are_refused_before_writes(self) -> None:
        self.create_database(rows=0)
        with self.assertRaisesRegex(prepare.PreparationError, "empty sessions"):
            self.prepare()
        self.assertEqual(list(self.destination_dir.iterdir()), [])

        for schema, expected_message in (
            (
                "CREATE TABLE sessions (id INTEGER PRIMARY KEY, started_at TEXT, state TEXT, "
                "break_credit TEXT, history_kind TEXT)",
                "history_kind is already present",
            ),
            (
                "CREATE TABLE sessions (id INTEGER PRIMARY KEY, started_at TEXT, state TEXT, "
                "break_credit TEXT)",
                "not eligible",
            ),
        ):
            self.source.unlink()
            connection = sqlite3.connect(self.source)
            connection.execute(schema)
            connection.execute(
                "INSERT INTO sessions (started_at, state) VALUES ('2026-10-22', 'Sitting')"
            )
            connection.commit()
            connection.close()
            with self.assertRaisesRegex(prepare.PreparationError, expected_message):
                self.prepare()
            self.assertEqual(list(self.destination_dir.iterdir()), [])

    def test_corrupt_database_and_missing_source_fail_before_writes(self) -> None:
        self.source.write_bytes(b"not a sqlite database")
        with self.assertRaises(prepare.PreparationError):
            self.prepare()
        self.assertEqual(list(self.destination_dir.iterdir()), [])

        self.source.unlink()
        with self.assertRaisesRegex(prepare.PreparationError, "Source database is unavailable"):
            self.prepare()
        self.assertEqual(list(self.destination_dir.iterdir()), [])

    def test_missing_corrupt_and_non_object_settings_fail_before_writes(self) -> None:
        self.create_database()
        self.settings_source.unlink()
        with self.assertRaisesRegex(prepare.PreparationError, "Settings source is unavailable"):
            self.prepare()
        self.assertEqual(list(self.destination_dir.iterdir()), [])

        for content in (b"{broken", b'["not", "an", "object"]'):
            self.settings_source.write_bytes(content)
            with self.assertRaises(prepare.PreparationError):
                self.prepare()
            self.assertEqual(list(self.destination_dir.iterdir()), [])

    def test_settings_read_io_failure_fails_closed(self) -> None:
        self.create_database()
        original_read_bytes = Path.read_bytes

        def fail_settings_read(path: Path) -> bytes:
            if path == self.settings_source:
                raise PermissionError("simulated settings read failure")
            return original_read_bytes(path)

        with mock.patch.object(Path, "read_bytes", fail_settings_read):
            with self.assertRaisesRegex(prepare.PreparationError, "Settings source could not be read"):
                self.prepare()
        self.assertEqual(list(self.destination_dir.iterdir()), [])

    def test_settings_destination_io_failure_creates_no_recovery_set(self) -> None:
        self.create_database()

        def leave_partial_settings(recovery_set: Path, content: bytes) -> Path:
            (recovery_set / "settings.json").write_bytes(content[:8])
            raise prepare.PreparationError("simulated settings destination write failure")

        with mock.patch.object(
            prepare, "_write_settings_copy", side_effect=leave_partial_settings
        ):
            with self.assertRaisesRegex(prepare.PreparationError, "simulated settings destination"):
                self.prepare()
        self.assertEqual(list(self.destination_dir.iterdir()), [])

    def test_later_database_failure_preserves_settings_and_last_good_set(self) -> None:
        self.create_database()
        last_good = self.destination_dir / "moveup-history-preflight-previous"
        last_good.mkdir()
        sentinel = last_good / "settings.json"
        sentinel.write_bytes(b"prior valid settings bytes")
        with mock.patch.object(
            prepare, "_write_database_copy", side_effect=prepare.PreparationError("simulated DB failure")
        ):
            with self.assertRaises(prepare.PartialPreparationError) as raised:
                self.prepare()

        result = raised.exception.result
        self.assertEqual(result["status"], "partial")
        self.assertEqual(result["available"], ["settings.json"])
        self.assertEqual(result["missing"], ["desk.db"])
        partial_set = Path(str(result["recovery_set"]))
        self.assertEqual({path.name for path in partial_set.iterdir()}, {"settings.json"})
        self.assertEqual((partial_set / "settings.json").read_bytes(), SETTINGS_BYTES)
        self.assertEqual(sentinel.read_bytes(), b"prior valid settings bytes")

    def test_partial_cli_report_is_truthful_and_never_prints_settings_values(self) -> None:
        self.create_database()
        stdout = io.StringIO()
        stderr = io.StringIO()
        with mock.patch.object(
            prepare, "_write_database_copy", side_effect=prepare.PreparationError("synthetic DB failure")
        ):
            with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
                exit_code = prepare.main(
                    [
                        "--source", str(self.source),
                        "--settings-source", str(self.settings_source),
                        "--destination-dir", str(self.destination_dir),
                    ]
                )
        report = stderr.getvalue()
        self.assertEqual(exit_code, 2)
        self.assertIn('"status": "partial"', report)
        self.assertIn('"available": [\n    "settings.json"', report)
        self.assertNotIn("DO_NOT_PRINT", stdout.getvalue() + report)

    def test_destination_overlap_and_automatic_pruning_paths_are_refused(self) -> None:
        self.create_database()
        unsafe = self.source_dir / "moveup-history-preflight-unsafe"
        unsafe.mkdir()
        with self.assertRaisesRegex(prepare.PreparationError, "overlap either source"):
            prepare.prepare_backup(self.source, self.settings_source, unsafe)

        pruned = self.root / "backups" / "moveup-history-preflight-unsafe"
        pruned.mkdir(parents=True)
        with self.assertRaisesRegex(prepare.PreparationError, "Automatic backup/pruning"):
            prepare.prepare_backup(self.source, self.settings_source, pruned)

    def test_production_identities_are_rejected_with_independent_sources(self) -> None:
        self.create_database()
        identities = (
            ("AppData", "Roaming", "io.zntl.desk"),
            ("AppData", "Local", "io.zntl.desk"),
            ("AppData", "Roaming", "MoveUp"),
            ("AppData", "Local", "MoveUp"),
        )
        for identity in identities:
            with self.subTest(identity=identity):
                production_destination = self.root.joinpath(
                    *identity, "moveup-history-preflight-synthetic"
                )
                production_destination.mkdir(parents=True)
                # Independent sibling sources ensure overlap alone cannot trigger the guard.
                self.assertNotEqual(self.source.parent, production_destination.parent)
                with self.assertRaisesRegex(prepare.PreparationError, "production data directory"):
                    prepare.prepare_backup(
                        self.source, self.settings_source, production_destination
                    )
                self.assertEqual(list(production_destination.iterdir()), [])

    def test_main_success_metadata_does_not_include_settings_plaintext(self) -> None:
        self.create_database()
        stdout = io.StringIO()
        stderr = io.StringIO()
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            exit_code = prepare.main(
                [
                    "--source", str(self.source),
                    "--settings-source", str(self.settings_source),
                    "--destination-dir", str(self.destination_dir),
                ]
            )
        self.assertEqual(exit_code, 0)
        self.assertEqual(stderr.getvalue(), "")
        self.assertIn('"status": "complete"', stdout.getvalue())
        self.assertNotIn("DO_NOT_PRINT", stdout.getvalue())


if __name__ == "__main__":
    unittest.main()
