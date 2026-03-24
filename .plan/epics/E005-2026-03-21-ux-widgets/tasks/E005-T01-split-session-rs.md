---
id: E005-T01
epic: E005
status: done
created: 2026-03-21
completed: 2026-03-21
original_id: T027
---
# T027 — Split session.rs (P0 sprint blocker)

**Status:** open
**Priority:** P0 — must complete BEFORE T022, T023, T024
**Branch:** feat/T027-split-session-rs

---

## Why This Exists

`session.rs` is **1348 lines** — 5.4x over the 250-line project limit.
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

Split into 4 files, all <= 250 lines:

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
/// Default 15 min. Gold bar fills 0->100% over this duration.
#[serde(default = "default_standing_target")]
pub standing_target_mins: u32,  // default: 15, clamp: 5-60

/// Maximum continuous standing before "consider sitting" nudge.
/// Default 90 min. Semantic: protective ceiling, not a goal.
#[serde(default = "default_stand_max")]
pub stand_max_mins: u32,  // default: 90, clamp: 30-120
```

**Migration: use `#[serde(alias)]` — zero extra code:**
```rust
#[serde(default = "default_standing_target", alias = "stand_limit_mins")]
pub standing_target_mins: u32,  // default 15, clamp 5-60
```

---

## Acceptance Criteria

- [ ] `session.rs` no longer exists OR is <= 50 lines (re-export only)
- [ ] All new files <= 250 lines
- [ ] `cargo test` passes (all existing session tests)
- [ ] `config.rs` has `standing_target_mins` (default 15) and `stand_max_mins` (default 90)
- [ ] Pre-commit hook passes on all modified files
- [ ] No behavior changes — pure refactor + config additions
