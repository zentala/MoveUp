# Overlay Progress Bar — Tasks

**Status:** In Development
**Reference:** [DEVELOPER-GUIDE.md](./DEVELOPER-GUIDE.md) | [CLAUDE.md](../../CLAUDE.md)

---

## Done

- [x] **T-OVR-001**: Dev mode — always-visible progress bar (3 commits, `demo_progress()` cycling)
- [x] **T-OVR-002**: Device disconnected notification (native Windows toast, throttled 5min)
- [x] **T-OVR-003**: Compare OPAQUE vs LAYERED ([MODE-COMPARISON.md](./MODE-COMPARISON.md))
- [x] **T-OVR-004**: Bar variants — solid/gradient/pulsing (`OVERLAY_VARIANT=0|1|2`)
- [x] **T-OVR-005**: Adjustable bar height (`OVERLAY_HEIGHT=1-20`, default 4)
- [x] **T-OVR-006**: Fix auto-test.sh — Windows-compatible, tests OPAQUE by default
- [x] **T-OVR-007**: Rust unit tests for overlay rendering logic (replaced broken Playwright test)
- [x] **T-OVR-008**: DataSource refactor — `dev_mode` → `DataSource { Demo, Live, Mock }` + `OVERLAY_DATA` env var

---

## In Progress / TODO

### [ ] T-OVR-009: Choose production render mode
**Priority:** P2
**Why:** Decide default for release builds.

OPAQUE (recommended by MODE-COMPARISON.md) vs LAYERED. Test both visually, pick one, hardcode as release default.

---

### [ ] T-OVR-010: Split overlay_renderer.rs (~1060 lines, limit 250)
**Priority:** P0
**Why:** File is 4x over the 250-line limit. Must split before adding more features.

**Proposed split:**
- `overlay_renderer.rs` — `DataSource` enum, `OverlayState`, `OverlayRenderer` API, `parse_data_source()`, `demo_progress()`, `mock_progress()`, `run_event_loop()` dispatcher
- `overlay_opaque.rs` — `run_event_loop_opaque()`, `wnd_proc()` (OPAQUE rendering)
- `overlay_layered.rs` — `run_event_loop_layered()`, `wnd_proc_layered()`, `draw_layered_frame()`, `LayeredBufferState`
- `overlay_variants.rs` — variant rendering logic (solid/gradient/pulsing) used by both modes

Tests stay in `overlay_renderer.rs` (or `overlay_tests.rs` if needed).

**Acceptance criteria:**
- [ ] No file > 250 lines
- [ ] `cargo test --lib` passes (101 tests)
- [ ] No behavior changes

---

### [ ] T-OVR-011: Verify debug overlay info in popup
**Priority:** P3
**Why:** Added `get_overlay_state` IPC command + popup display but not confirmed working.

Check that popup shows `overlay: Live | X.X% | visible=true | h=4px` line. If not, debug — check browser console for errors from `invoke("get_overlay_state")`.

---

### [ ] T-OVR-012: Precommit hook for file line count
**Priority:** P1
**Why:** Prevent files from exceeding 250-line limit again.

Add to precommit hook: check all staged `.rs`, `.ts`, `.tsx` files. Fail if any exceed 250 lines. Exclude test files and generated code.

---

## Archived

- [ORCHESTRATOR.md](./ORCHESTRATOR.md) — Wave-based parallel execution plan (completed 2026-03-19)
- [v1-opaque-debugging/](./v1-opaque-debugging/) — First debugging session (2026-03-19)
