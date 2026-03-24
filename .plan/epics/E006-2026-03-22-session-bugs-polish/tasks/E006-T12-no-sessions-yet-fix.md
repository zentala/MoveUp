---
id: E006-T12
epic: E006
status: done
created: 2026-03-23
completed: 2026-03-23
original_id: T040
---
# T040 — "No sessions yet" despite working: field name mismatch fix

**Status:** open
**Priority:** P2
**Depends on:** none

---

## Problem

User sees "No sessions yet" in the OneBarTimeline widget despite actively using the desk.
Sessions ARE saved to SQLite correctly — the bug is a **JSON field name mismatch** between
Rust backend and TypeScript frontend.

## Root Cause

`SessionRow` (Rust) serializes with snake_case DB column names, but `SessionEntry` (TypeScript)
expects different field names:

| Rust `SessionRow` field | JSON output       | TS `SessionEntry` expects | Match? |
|-------------------------|-------------------|---------------------------|--------|
| `started_at`            | `started_at`      | `start`                   | NO     |
| `ended_at`              | `ended_at`        | `end`                     | NO     |
| `duration_seconds`      | `duration_seconds` | `duration_secs`           | NO     |
| `id`                    | `id`              | (not in TS type)          | N/A    |

## Solution

Add `#[serde(rename = "...")]` to `SessionRow` in `db.rs` to match the TypeScript contract.

## Risk

- Low risk: only changes JSON serialization names, not DB columns or internal Rust field names
