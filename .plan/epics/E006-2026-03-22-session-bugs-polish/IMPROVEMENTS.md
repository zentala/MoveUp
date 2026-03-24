# E006 Improvements — Session Bugs & Polish

## 2026-03-22 — Design review session
- Memory leak: `Box::leak` in `generate_tray_icon` (4KB/s) -> `Image::new_owned`
- Tooltip positioning: `position: fixed` -> `position: absolute` (frameless window fix)
- ZenTimeline height: inline style -> CSS class
- prefers-reduced-motion: added global `@media` rule
- Hardcoded timeline colors: `blockColor()` hex -> CSS classes with design tokens
- Duplicate `.app__header` selectors: merged into one

## 2026-03-23 — Parallel tasks session
- T037 may fix T036 (standing detection) — needs real sensor verification
- T038 blur edge case — opening Settings from popup may trigger unwanted close
- T044 priority increases now that T040 shows sitting sessions but no standing ones
