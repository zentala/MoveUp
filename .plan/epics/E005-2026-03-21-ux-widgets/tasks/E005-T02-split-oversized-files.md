---
id: E005-T02
epic: E005
status: done
created: 2026-03-21
completed: 2026-03-21
original_id: T027b
---
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

## Parts

1. Split db.rs (467 -> 3 files)
2. Split serial.rs (464 -> 3 files)
3. Split commands.rs (264 -> 2 files)
4. Split overlay_tests.rs (281 -> 2 files)
5. Add limit_used_secs to SessionStateDto
6. Add active_widget to AppConfig

---

## Acceptance Criteria

- [ ] `db.rs` <= 100 lines, `db_sessions.rs` + `db_queries.rs` + `db_tests.rs` all <= 250
- [ ] `serial.rs` <= 200 lines, `serial_parser.rs` + `serial_periodic.rs` all <= 250
- [ ] `commands.rs` <= 150 lines, `commands_config.rs` <= 250
- [ ] `overlay_tests.rs` <= 200 lines, `overlay_variant_tests.rs` <= 250
- [ ] `limit_used_secs` in SessionStateDto (Rust + TS)
- [ ] `active_widget` in AppConfig
- [ ] `useDesk` returns `limitUsedSecs`, `limitRemaining`, `limitRatio`
- [ ] `cargo test` passes
- [ ] `pnpm test:unit` passes
- [ ] Pre-commit hook passes on all files
- [ ] No behavior changes beyond new fields
