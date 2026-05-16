---
id: E012-T02
epic: E012
status: planned
created: 2026-05-16
branch: feat/E012-T02-catalog-meta
---

# E012-T02 — Backend: data catalog metadata command

## Goal
Expose `get_data_catalog()` returning a structured description of every data source the app produces — schema, sample row, count, on-disk path, retention. This feeds the Catalog tab.

## Why
The Catalog tab is "what data do I have?" — a developer-facing reference. Building it from the frontend requires reading the same paths and shapes from multiple files; centralizing it in Rust keeps the frontend dumb.

## Command

```rust
#[tauri::command]
async fn get_data_catalog(app: AppHandle, state: State<AppState>) -> Result<DataCatalog, String>;
```

## Types

```rust
#[derive(Serialize)]
struct DataCatalog {
    sources: Vec<DataSource>,
    generated_at: String,  // ISO 8601
}

#[derive(Serialize)]
struct DataSource {
    id: String,            // "sensor" | "sqlite_sessions" | "snapshots" | "events_log" | "profiles_ergonomic" | "profiles_communication" | "store" | "remote_ws"
    name: String,          // human label
    kind: String,          // "stream" | "table" | "log" | "config" | "kv"
    path: Option<String>,  // absolute on-disk path or "in-memory"
    fields: Vec<FieldSpec>,
    row_count: Option<u64>,
    bytes_on_disk: Option<u64>,
    retention_days: Option<u32>,
    sample: Option<String>, // a JSON-stringified example row (or null)
    description: String,    // 1-2 sentences
}

#[derive(Serialize)]
struct FieldSpec {
    name: String,
    type_: String,         // serde rename "type" — "string" | "i64" | "f32" | "bool" | "enum<Sitting|Standing|Walking|Away>" | ...
    description: String,
}
```

## Implementation notes

- Sources to include (8 total, per `reports/2026-05-16-data-sources.md`):
  1. `sensor` — COM port stream (in-memory, fields from `DistanceReading`)
  2. `sqlite_sessions` — `desk.db` (count via `SELECT COUNT(*) FROM sessions`, bytes via file stat)
  3. `snapshots` — minute JSONs (count = number of `.json` files under `logs/`, bytes = sum)
  4. `events_log` — text logs (line count via wc-equivalent, bytes via stat sum)
  5. `profiles_ergonomic` — files in `profiles/ergonomic/`
  6. `profiles_communication` — files in `profiles/communication/`
  7. `store` — tauri-plugin-store keys (list known keys + types — hardcoded since plugin doesn't enumerate)
  8. `remote_ws` — message types broadcast on `:3390` (in-memory; fields from `ws_broadcaster.rs` message variants)
- For `sample`: read the newest file (snapshots/events) or first row (sqlite); for streams/profiles use a hardcoded example matching the actual shape.
- For `bytes_on_disk`: traverse with `walkdir` already in tree; cap walk to `app_data_dir/logs` for speed.
- Whole command must return in < 200 ms on a 7-day log dir (target: lazy load — don't read every snapshot, just count files).

## Tests

**Unit (Rust):**
- Each source builder function returns the expected `DataSource` shape in isolation (fixture-based)
- File-count walker handles empty dir, dir with 0 days, dir with mixed files
- Bytes-on-disk sums match `fs::metadata` for fixtures

**Integration (Rust):**
- `get_data_catalog` against a tempdir fixture with synthetic snapshots+events+db → returns all 8 sources, counts match

## Files touched
- `apps/desk/src-tauri/src/commands_analyst.rs` — extend with `get_data_catalog`
- `apps/desk/src-tauri/src/lib.rs` — register command
- `apps/desk/src-tauri/capabilities/default.json` — allowlist

## DoD
- [ ] Command returns all 8 sources with non-empty `name`, `kind`, `fields`
- [ ] Counts and byte sizes verified against `fs` for fixture
- [ ] Tests pass; coverage ≥ 80%
- [ ] File ≤ 250 lines
