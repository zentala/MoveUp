# T027 — Split session.rs (P0 sprint blocker)

**Status:** open
**Priority:** P0 — must complete BEFORE T022, T023, T024
**Branch:** feat/T027-split-session-rs

---

## Why This Exists

`session.rs` is **1348 lines** — 5.4× over the 250-line project limit.
The pre-commit hook (added in T-OVR-012) rejects commits on any `.rs` file > 250 lines.
**Every sprint task touches session.rs. Agents will be blocked at commit time.**

This split must happen first.

---

## Current Structure of `session.rs`

```
session.rs (1348 lines)
├── DeskState enum (28-36)
├── SessionState struct (37-57)
├── SessionStateDto struct (61-72)
├── StateChangedPayload struct (76-84)
├── CompletedSession struct (88-92)
├── NotificationEvent enum (96-101)
├── ReadingResult struct (105-108)
├── SessionManager struct + impl (113-~500)
│   ├── new(), load_today_totals(), snapshot(), on_tick()
│   ├── set_limit_minutes(), set_stand_limit_minutes()
│   ├── current_state(), is_initialized()
│   ├── check_notification_conditions()
│   ├── should_send_praise_halfway()
│   ├── on_reading() — main state machine (~200 lines)
│   └── handle_state_transition()
└── #[cfg(test)] mod tests (~800 lines!)
```

Tests are the biggest chunk by far.

---

## Target Structure

Split into 4 files, all ≤ 250 lines:

### `session_types.rs` (~80 lines)
All public structs, enums, DTOs:
- `DeskState`
- `SessionState`
- `SessionStateDto`
- `StateChangedPayload`
- `CompletedSession`
- `NotificationEvent` — **also add new variants here** (see T024 + config decisions):
  - `StandingTargetReached` (user completed standing_target_mins)
- `ReadingResult`

### `session_manager.rs` (~200 lines)
`SessionManager` struct + all `impl` methods:
- `new()`, `snapshot()`, `load_today_totals()`
- `set_limit_minutes()`, `set_stand_limit_minutes()`
- `on_tick()`, `on_reading()` — core state machine
- `handle_state_transition()`
- `check_notification_conditions()`
- `should_send_praise_halfway()`

### `session_config.rs` (~30 lines) — OR fold into `config.rs`
Config-related session defaults. May not need a separate file — evaluate during split.

### `session_tests.rs` (~800 lines)
All `#[cfg(test)]` content extracted from `session.rs`.
Pattern already established: `alert_manager_tests.rs`, `alert_snooze_tests.rs`, `overlay_tests.rs`.

---

## Config Changes (do in this task)

Add two new fields to `AppConfig` in `config.rs`:

```rust
/// Standing target: duration for a "complete" standing session (gold bar fills, +5 pts bonus).
/// Default 15 min. Gold bar fills 0→100% over this duration.
#[serde(default = "default_standing_target")]
pub standing_target_mins: u32,  // default: 15, clamp: 5–60

/// Maximum continuous standing before "consider sitting" nudge.
/// Default 90 min. Semantic: protective ceiling, not a goal.
#[serde(default = "default_stand_max")]
pub stand_max_mins: u32,  // default: 90, clamp: 30–120
```

Add helper functions:
```rust
fn default_standing_target() -> u32 { 15 }
fn default_stand_max() -> u32 { 90 }
```

Update `clamped()`:
```rust
self.standing_target_mins = self.standing_target_mins.clamp(5, 60);
self.stand_max_mins = self.stand_max_mins.clamp(30, 120);
```

**IMPORTANT semantic distinction:**
- `standing_target_mins` (was `stand_limit_mins`) = celebration threshold. Gold bar fills over this. +5 pts bonus per lap when reached.
- `stand_max_mins` = warning threshold. "You've been standing a long time, take a seat."
- Old field `stand_limit_mins` was doing BOTH jobs with conflicting semantics.

**Migration: use `#[serde(alias)]` — zero extra code:**
```rust
#[serde(default = "default_standing_target", alias = "stand_limit_mins")]
pub standing_target_mins: u32,  // default 15, clamp 5–60
```
Serde automatically reads old `stand_limit_mins` from JSON store. When user saves new settings,
key updates automatically. No manual migration code needed.

Update `SessionManager::set_stand_limit_minutes()` to use `standing_target_mins`.
Update `SessionState.stand_limit_secs` → rename to `standing_target_secs` (or add alongside — check
all callers first). Add `stand_max_secs: i64` for the new warning threshold.

---

## Files to Create/Modify

| Action | File |
|--------|------|
| Create | `src-tauri/src/session_types.rs` |
| Create | `src-tauri/src/session_manager.rs` |
| Create | `src-tauri/src/session_tests.rs` |
| Delete (replace) | `src-tauri/src/session.rs` → becomes thin re-export module |
| Modify | `src-tauri/src/config.rs` — add `standing_target_mins`, `stand_max_mins` |
| Modify | `src-tauri/src/lib.rs` — update imports |
| Modify | `src-tauri/src/serial.rs` — update imports |
| Modify | `src-tauri/src/commands.rs` — update imports |
| Modify | `src-tauri/src/tray_controller.rs` — update imports |

**session.rs becomes a thin re-export file:**
```rust
// session.rs — re-exports for backwards compatibility
pub use crate::session_types::*;
pub use crate::session_manager::*;
```
Or remove entirely and update all import sites.

---

## Tests

All existing tests must pass after split:
```bash
cargo test
```
No new tests needed — this is pure refactor. If tests fail, the split broke something.

---

## Acceptance Criteria

- [ ] `session.rs` no longer exists OR is ≤ 50 lines (re-export only)
- [ ] All new files ≤ 250 lines
- [ ] `cargo test` passes (all existing session tests)
- [ ] `config.rs` has `standing_target_mins` (default 15) and `stand_max_mins` (default 90)
- [ ] Pre-commit hook passes on all modified files
- [ ] No behavior changes — pure refactor + config additions
