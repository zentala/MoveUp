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

The frontend receives `{ started_at, ended_at, duration_seconds }` but destructures
`{ start, end, duration_secs }` → all fields are `undefined` → sessions array appears empty
or broken → "No sessions yet".

## Data Flow

```
DB (sessions table)
  → db_queries.rs: SELECT → SessionRow { started_at, ended_at, ... }
    → commands.rs: get_today_summary() → TodaySummary { sessions: Vec<SessionRow> }
      → JSON: { sessions: [{ started_at, ended_at, duration_seconds }] }
        → useDesk.ts: invoke("get_today_summary") → TodaySummaryDto
          → SessionEntry[] ← MISMATCH HERE
            → OneBarTimeline.tsx: sessions.length === 0 → "No sessions yet"
```

## Solution

Add `#[serde(rename = "...")]` to `SessionRow` in `db.rs` to match the TypeScript contract:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRow {
    #[serde(skip_serializing)]
    pub id: i64,
    #[serde(rename = "start")]
    pub started_at: String,
    #[serde(rename = "end")]
    pub ended_at: Option<String>,
    pub state: String,
    #[serde(rename = "duration_secs")]
    pub duration_seconds: Option<i64>,
}
```

Using `skip_serializing` for `id` (not `skip`) so deserialization from DB rows still works.

## Files to modify

1. **`src-tauri/src/db.rs`** (lines 77-84) — add serde rename attributes to `SessionRow`

## Testing

1. `cargo test` — ensure existing DB tests still pass (they test insert/query, not serialization)
2. Add a serialization test in `db_tests.rs`:
   - Create a `SessionRow`, serialize to JSON, verify field names are `start`, `end`, `duration_secs`
   - Verify `id` is NOT in the JSON output
3. Manual: run app, use desk, check timeline shows sessions

## Risk

- Low risk: only changes JSON serialization names, not DB columns or internal Rust field names
- `serde(rename)` only affects JSON — `rusqlite` reads by column index, not name
- If any Rust code deserializes `SessionRow` FROM JSON (unlikely), `rename` applies bidirectionally
