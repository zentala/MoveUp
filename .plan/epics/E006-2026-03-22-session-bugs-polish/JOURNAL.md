# E006 Journal — Session Bugs & Polish

## Session 2026-03-22 ~10:00-12:00

- **Goal**: Investigate T036 (standing not detected, "No sessions yet"), fix UX issues
- **Done**:
  - **T036 Bug A FIXED** — `insert_session()` was never called from production code; `save_session_state()` created DB rows without `ended_at`, invisible to all queries. Fixed in `serial_periodic.rs`, regression test added. (commit: 4869ca8)
  - **T036 Bug B INSTRUMENTED** — debug logging added for debounce resets (transitions only). From user's logs: sensor sends ~10 readings/s (not 1/s), readings jump +/-27mm. Standing detection works mechanically but needs T037 (stabilization) to be reliable.
  - **Sensor "disconnected" race condition fixed** — `desk:device-connected` event fired before React listener mounted. Added `setConnected(true)` on `desk:state-changed`. (commit: dfa8a53)
  - **"CONNECTING" label fixed** — was fallback for null state; changed default state to "Away", removed all null/dash fallbacks. (commits: c8635c4, 4f3a2d0, d805118)
  - **"4m" -> "for 4m"** — added "for" prefix for context. (commit: d747e61)
  - **Lowercase labels** — "SITTING" -> "sitting"; removed CSS `text-transform: uppercase` from state label. (commits: 4f3a2d0, 7ffed60)
  - **Standing timer fixed** — widget showed frozen sitting timer when standing; now shows `breakSecs` (standing duration) when not sitting. (commit: 4e8d011)
  - **Cleanup** — removed dead `save_session_state()`, dead null check in `useWidgetData`, added 2 regression tests for standing/sitting timer display. (commit: 68d91a5)
  - **UX Flow Map created** — `.arch/UX-FLOW.md` — comprehensive document mapping every state x every UI element x colors/text/visibility. (commit: 8f629fb)
- **New tasks created**: T036-T041
  - T036 P0 — standing detection investigation (Bug B still needs live verification)
  - T037 P1 — height reading stabilization (moving average, 1cm rounding, trend lock)
  - T038 P1 — popup closes on click outside
  - T039 P2 — connection status UI cleanup
  - T040 P2 — "No sessions yet" (explained by Bug A, should be fixed now)
  - T041 P1 — UX flow map (DONE)
- **Decisions**: sensor sends 10/s, debounce=5 readings=0.5s (not 5s). T037 should add throttling/smoothing before debounce.
- **Observations**: multiple UX labels were confusing (CONNECTING, 4m, uppercase). Design system says uppercase for 10px labels but user finds it unclear — lowercase works better for state labels.
- **Improvements logged**: 6 items reviewed via impro? — 4 fixed, 1 deferred (ensure_initialized), 1 kept (PlaceholderWidget as dev tool)
- **Next**: T037 (height stabilization), T038 (popup blur close), verify T036 Bug A fix

---

## Session 2026-03-22 — Design Review + Polish

### Goal
Visual design audit of the Desk app, fix layout issues reported by user,
and polish tray icon behavior.

### Done

#### Design Review (source-code CSS audit)
- Ran /design-review skill (browser-based audit blocked — Tauri app can't render in headless browser)
- Pivoted to source-code CSS audit against design checklist
- **7 findings**, all fixed:
  - FINDING-001: One Bar widget ignored design system tokens (hardcoded #ccc, Segoe UI) -> unified with --ink-*, --font-mono
  - FINDING-003: .app used min-height instead of height -> dead space at bottom
  - FINDING-005: Coach text could overflow -> text-overflow: ellipsis
  - FINDING-007: Back button styled as danger -> changed to link style

#### impro? fixes (3 rounds)
- **Memory leak**: `Box::leak` in `generate_tray_icon` (4KB/s) -> `Image::new_owned`
- **Tooltip positioning**: `position: fixed` -> `position: absolute` (frameless window fix)
- **ZenTimeline height**: inline style -> CSS class
- **prefers-reduced-motion**: added global `@media` rule
- **Hardcoded timeline colors**: `blockColor()` hex -> CSS classes with design tokens
- **Duplicate `.app__header` selectors**: merged into one
- **Timeline Zen CSS**: aligned with design tokens (--signal-*, --ink-*, --panel-*)
- **Debug overlay**: inline styles -> `.app__debug` CSS class

#### Features
- **Window position**: bottom-right corner (dynamic via `current_monitor()`)
- **Tray behavior**: left-click = toggle window, right-click = menu (Settings, Quit)
- **View reset**: window always opens to widget view (not settings)
- **DESIGN.md**: created — documents instrument panel design system

#### Tasks created
- **T035**: Settings tabbed layout (4 tabs instead of scrolling)
- **Backlog**: Alert popup redesign (WinAPI -> Tauri WebviewWindow with HTML/CSS)

### Commits (17 total)
- 291f44d chore: add T034 task spec
- 389bb8b style: unify One Bar with design tokens
- 94793aa style: fix app shell height
- f48a054 style: Back button link style
- 79ae6e6 chore: update TASKS.md
- 983ac5b feat: window bottom-right positioning
- 994ef24 style: Timeline Zen design tokens
- 13349e6 style: debug overlay CSS class
- 4bc9288 docs: create DESIGN.md
- 7c47d92 feat: tray left-click toggle, right-click menu
- 52fece5 chore: T035 + backlog popup redesign
- 0716fa6 fix: memory leak in tray icon
- b882df9 fix: tooltip absolute positioning
- f68b029 style: ZenTimeline CSS height
- b6b5947 style: prefers-reduced-motion
- b60b010 style: timeline block CSS classes
- 46addff style: merge duplicate .app__header

#### Post-session fix
- 2f07fab fix: `show_menu_on_left_click(false)` — left-click was sometimes triggering menu instead of toggle

### Tests
- 131 TypeScript tests passing
- 183 Rust tests passing (cargo check clean, 2 pre-existing warnings)

### Next
- T035: Implement tabbed settings layout
- Visual verification: restart app, check bottom-right positioning, tray behavior, design cohesion

---

## Session 2026-03-23 ~afternoon

- **Goal**: Execute T035, T037, T038 in parallel subagents + investigate/fix T040
- **Done**:
  - T035 — Settings tabbed layout (4 tabs: Time, Calibr., Notif., More) — commit `7065181`
  - T037 — HeightStabilizer module (moving avg, cm rounding, trend lock) — commit `ce9b27d`
  - T038 — Popup closes on blur (`WindowEvent::Focused(false)`) — commit `86489d7`
  - T040 — "No sessions yet" fix: `SessionRow` JSON field names mismatched TS `SessionEntry` — commit `97fa989`
  - T044 — Cancelled (needs architecture planning first)
- **Decisions**: T040 root cause was `serde` field name mismatch, not missing sessions in DB. Fix was 4 `#[serde(rename)]` attributes.
- **Findings this session**: 1
  - T040 investigation revealed sessions ARE persisted correctly; the bug was purely serialization layer
- **Improvements logged**: 3 (informal via impro?)
  1. T037 may fix T036 (standing detection) — needs real sensor verification
  2. T038 blur edge case — opening Settings from popup may trigger unwanted close
  3. T044 priority increases now that T040 shows sitting sessions but no standing ones
- **Next**:
  - T044 architecture planning (standing session persistence)
  - T036 live sensor verification (may be fixed by T037)
  - T043 tooltip + UI integration tests
