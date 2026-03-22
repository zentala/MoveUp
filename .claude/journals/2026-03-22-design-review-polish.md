# Session 2026-03-22 — Design Review + Polish

## Goal
Visual design audit of the Desk app, fix layout issues reported by user,
and polish tray icon behavior.

## Done

### Design Review (source-code CSS audit)
- Ran /design-review skill (browser-based audit blocked — Tauri app can't render in headless browser)
- Pivoted to source-code CSS audit against design checklist
- **7 findings**, all fixed:
  - FINDING-001: One Bar widget ignored design system tokens (hardcoded #ccc, Segoe UI) → unified with --ink-*, --font-mono
  - FINDING-003: .app used min-height instead of height → dead space at bottom
  - FINDING-005: Coach text could overflow → text-overflow: ellipsis
  - FINDING-007: Back button styled as danger → changed to link style

### impro? fixes (3 rounds)
- **Memory leak**: `Box::leak` in `generate_tray_icon` (4KB/s) → `Image::new_owned`
- **Tooltip positioning**: `position: fixed` → `position: absolute` (frameless window fix)
- **ZenTimeline height**: inline style → CSS class
- **prefers-reduced-motion**: added global `@media` rule
- **Hardcoded timeline colors**: `blockColor()` hex → CSS classes with design tokens
- **Duplicate `.app__header` selectors**: merged into one
- **Timeline Zen CSS**: aligned with design tokens (--signal-*, --ink-*, --panel-*)
- **Debug overlay**: inline styles → `.app__debug` CSS class

### Features
- **Window position**: bottom-right corner (dynamic via `current_monitor()`)
- **Tray behavior**: left-click = toggle window, right-click = menu (Settings, Quit)
- **View reset**: window always opens to widget view (not settings)
- **DESIGN.md**: created — documents instrument panel design system

### Tasks created
- **T035**: Settings tabbed layout (4 tabs instead of scrolling)
- **Backlog**: Alert popup redesign (WinAPI → Tauri WebviewWindow with HTML/CSS)

## Commits (17 total)
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

## Tests
- 131 TypeScript tests passing
- 183 Rust tests passing (cargo check clean, 2 pre-existing warnings)

## Next
- T035: Implement tabbed settings layout
- Visual verification: restart app, check bottom-right positioning, tray behavior, design cohesion
- Backlog: Alert popup redesign when starting notifications sprint
- Note: `.plan/` and `.arch/` directories not yet bootstrapped for this app
