# T027b — Split oversized Rust files + add limit_used_secs

**Status:** open
**Priority:** P0 — must complete BEFORE Wave 2 (T023, T024, T029 will be blocked)
**Branch:** feat/T027b-split-oversized-files
**Depends on:** T027a (session.rs split must be merged first)
**Wave:** 1b (sequential after T027a)

---

## Why This Exists

Pre-commit hook rejects `.rs` files > 250 lines. Four files besides session.rs exceed this:

| File | Lines | Who touches it? |
|------|-------|-----------------|
| `db.rs` | 467 | T024, T030 |
| `serial.rs` | 464 | T024 (notification loop) |
| `commands.rs` | 264 | T029 (new fields), T025 (welcome commands), T031 (widget) |
| `overlay_tests.rs` | 281 | Any overlay change |

Additionally, `limit_used_secs` and `active_widget` need to be added to DTOs and config (CEO + eng review decisions).

---

## Part 1: Split db.rs (467 → 3 files)

### Current structure
```
db.rs (467 lines)
├── ensure_schema() — CREATE TABLE statements (~50L)
├── save_session_state() — INSERT session row (~40L)
├── get_today_summary() — SELECT + aggregation (~80L)
├── get_yesterday_summary() — SELECT yesterday's data (~50L)
├── load_today_totals() — restore counters from DB (~60L)
├── save_daily_reset() — INSERT reset marker (~30L)
├── Helper functions — date parsing, row mapping (~60L)
└── #[cfg(test)] mod tests (~100L)
```

### Target structure

**`db.rs` (~60 lines) — connection + schema**
- `ensure_schema()` — CREATE TABLE
- `get_connection()` / connection helpers
- Re-exports from sub-modules

**`db_sessions.rs` (~120 lines) — session CRUD**
- `save_session_state()`
- `load_today_totals()`
- `save_daily_reset()`
- Helper functions (row mapping)

**`db_queries.rs` (~100 lines) — summary queries**
- `get_today_summary()`
- `get_yesterday_summary()`
- Date parsing helpers

**`db_tests.rs` (~100 lines) — all tests**
- Extracted from `#[cfg(test)]` in db.rs

---

## Part 2: Split serial.rs (464 → 3 files)

### Current structure
```
serial.rs (464 lines)
├── Constants — baud rate, timeouts (~20L)
├── auto_detect_port() — scan COM ports (~60L)
├── parse_distance() — parse "DIST: 750" (~30L)
├── parse_device_info() — parse "DEVICE: zntl-desk-sensor v1" (~20L)
├── start_reading() — main serial loop (~180L)
│   ├── read line, parse, feed session
│   ├── emit events
│   ├── notification check every 60s
│   └── daily reset check
├── start_auto_connect() — retry loop (~60L)
└── #[cfg(test)] mod tests (~80L)
```

### Target structure

**`serial.rs` (~150 lines) — main reader loop**
- Constants
- `start_reading()` — core loop (read, parse, feed session, emit events)
- `start_auto_connect()`

**`serial_parser.rs` (~80 lines) — parsing functions**
- `auto_detect_port()`
- `parse_distance()`
- `parse_device_info()`
- Tests for parsing

**`serial_periodic.rs` (~80 lines) — periodic checks**
- Notification condition check (extracted from start_reading's 60s interval)
- Daily reset check
- Tests for periodic logic

---

## Part 3: Split commands.rs (264 → 2 files)

### Current structure
```
commands.rs (264 lines)
├── AppState type alias (~5L)
├── get_session_state() — snapshot command (~20L)
├── get_today_summary() — DB query command (~30L)
├── get_settings() / save_settings() — config commands (~40L)
├── calibrate() — calibration command (~30L)
├── set_session_limit() / set_stand_limit() (~20L)
├── start_auto_connect() / stop_reading() (~30L)
├── inject_reading() — test helper (~20L)
├── get_overlay_state() — debug command (~20L)
└── register_commands() — list of all commands (~20L)
```

### Target structure

**`commands.rs` (~130 lines) — session + connection commands**
- `AppState`
- `get_session_state()`, `get_today_summary()`
- `start_auto_connect()`, `stop_reading()`
- `inject_reading()` (test helper)
- `register_commands()`

**`commands_config.rs` (~100 lines) — config + calibration**
- `get_settings()`, `save_settings()`
- `calibrate()`
- `set_session_limit()`, `set_stand_limit()`
- `get_overlay_state()` (debug)

---

## Part 4: Split overlay_tests.rs (281 → 2 files)

**`overlay_tests.rs` (~140 lines) — core overlay tests**
**`overlay_variant_tests.rs` (~140 lines) — variant + demo/mock tests**

Split at a logical boundary: core state tests vs rendering variant tests.

---

## Part 5: Add limit_used_secs to SessionStateDto

**CEO review decision:** Rust computes `limit_used_secs` as the single source of truth.
**Eng review decision:** Widget displays ONLY this value — no useTimer for the big number.

### Rust changes

**`session_types.rs`** (created by T027a):
```rust
pub struct SessionStateDto {
    // ... existing fields ...
    /// Seconds of sitting limit consumed. Includes break credit deductions.
    /// Negative value = limit not reached yet (inverted for widget convenience).
    /// Positive value = limit consumed. > limit_secs = overtime.
    pub limit_used_secs: i64,
}
```

**`session_manager.rs`** (created by T027a):
```rust
pub fn snapshot(&self) -> SessionStateDto {
    // ... existing fields ...
    limit_used_secs: self.compute_limit_used(),
}

/// Compute how many seconds of sitting limit have been consumed.
/// Accounts for break credits (short/long break deductions).
/// Returns 0..limit_secs normally, >limit_secs when overtime.
fn compute_limit_used(&self) -> i64 {
    // sitting_seconds minus any break credit already applied
    // Break credit is already subtracted from sitting_seconds in on_reading()
    // So limit_used_secs = sitting_seconds (current session accumulator)
    self.state.sitting_seconds
}
```

**Note:** The current session.rs already deducts break credit from `sitting_seconds` in `handle_state_transition()`. So `limit_used_secs = sitting_seconds` is correct — the credit is already baked in. Verify this during implementation by reading the break credit logic in session_manager.rs.

### TypeScript changes

**`src/types.ts`:**
```typescript
export interface SessionStateDto {
  // ... existing fields ...
  limit_used_secs: number;
}
```

**`src/hooks/useDesk.ts`:**
```typescript
// Add to state:
const [limitUsedSecs, setLimitUsedSecs] = useState(0);

// Add to fetchState:
setLimitUsedSecs(dto.limit_used_secs);

// Add computed fields to return:
limitUsedSecs,
limitRemaining: sessionLimitSecs - limitUsedSecs,  // can be negative
limitRatio: sessionLimitSecs > 0 ? limitUsedSecs / sessionLimitSecs : 0,
```

---

## Part 6: Add active_widget to AppConfig

**`config.rs`:**
```rust
#[serde(default = "default_active_widget")]
pub active_widget: String,

fn default_active_widget() -> String {
    "one-bar".to_string()
}
```

Add to `clamped()`: no clamping needed (string validation happens in TS widget registry).

---

## Files to Create

| File | Lines | Content |
|------|-------|---------|
| `src-tauri/src/db_sessions.rs` | ~120 | Session CRUD |
| `src-tauri/src/db_queries.rs` | ~100 | Summary queries |
| `src-tauri/src/db_tests.rs` | ~100 | DB tests |
| `src-tauri/src/serial_parser.rs` | ~80 | Parsing functions |
| `src-tauri/src/serial_periodic.rs` | ~80 | Periodic checks |
| `src-tauri/src/commands_config.rs` | ~100 | Config/calibration commands |
| `src-tauri/src/overlay_variant_tests.rs` | ~140 | Variant overlay tests |

## Files to Modify

| File | Change |
|------|--------|
| `src-tauri/src/db.rs` | Slim to ~60L, re-export sub-modules |
| `src-tauri/src/serial.rs` | Slim to ~150L, import parser + periodic |
| `src-tauri/src/commands.rs` | Slim to ~130L, import commands_config |
| `src-tauri/src/overlay_tests.rs` | Slim to ~140L, move variants to new file |
| `src-tauri/src/lib.rs` | Add new module declarations |
| `src-tauri/src/config.rs` | Add `active_widget: String` |
| `src-tauri/src/session_types.rs` | Add `limit_used_secs` to SessionStateDto |
| `src-tauri/src/session_manager.rs` | Add `compute_limit_used()`, populate in `snapshot()` |
| `src/types.ts` | Add `limit_used_secs` to SessionStateDto |
| `src/hooks/useDesk.ts` | Add `limitUsedSecs`, `limitRemaining`, `limitRatio` |

## Tests

**Rust:**
- `cargo test` — all existing tests must pass after splits
- New test: `compute_limit_used()` returns correct value after break credit
- New test: `active_widget` config serialization roundtrip
- New test: `limit_used_secs` present in `snapshot()` output

**TypeScript:**
- `pnpm test:unit` — all existing tests must pass
- New test: `useDesk` returns `limitRemaining` = `sessionLimitSecs - limitUsedSecs`
- New test: `limitRatio` = 0 when `sessionLimitSecs` = 0 (edge guard)

## Acceptance Criteria

- [ ] `db.rs` ≤ 100 lines, `db_sessions.rs` + `db_queries.rs` + `db_tests.rs` all ≤ 250
- [ ] `serial.rs` ≤ 200 lines, `serial_parser.rs` + `serial_periodic.rs` all ≤ 250
- [ ] `commands.rs` ≤ 150 lines, `commands_config.rs` ≤ 250
- [ ] `overlay_tests.rs` ≤ 200 lines, `overlay_variant_tests.rs` ≤ 250
- [ ] `limit_used_secs` in SessionStateDto (Rust + TS)
- [ ] `active_widget` in AppConfig
- [ ] `useDesk` returns `limitUsedSecs`, `limitRemaining`, `limitRatio`
- [ ] `cargo test` passes
- [ ] `pnpm test:unit` passes
- [ ] Pre-commit hook passes on all files
- [ ] No behavior changes beyond new fields
