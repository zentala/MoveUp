# E002 Journal — Overlay Progress Bar

Executed 2026-03-16 to 2026-03-20. 10 of 12 tasks completed, 2 deferred to backlog.

## Key Events

### 2026-03-16 — Initial implementation
- Created `overlay_renderer.rs` with WinAPI window
- Discovered that Tauri WebviewWindow cannot create a clean 4px overlay (5 failed approaches documented)
- Chose raw WinAPI approach using `windows` crate

### 2026-03-19 — Demo mode debugging (v1-opaque)
- Haiku 4.5 agent attempted 4+ iterations to implement demo mode, all failed
- Root cause found by Opus 4.6: agent tested LAYERED mode while user saw OPAQUE mode (different code paths)
- Progress bar invisible because `visible=false` without desk sensor
- See `archive/v1-opaque-debugging/` for full iteration history

### 2026-03-20 — File split + cleanup
- `overlay_renderer.rs` grew to 1132 lines (4.5x over 250-line limit)
- Split into 5 files: overlay_renderer, overlay_opaque, overlay_layered, overlay_variants, overlay_tests
- Added precommit hook to enforce 250-line limit on staged files

## Deferred Tasks
- E002-T09: Choose production render mode → BACKLOG
- E002-T11: Verify debug overlay info in popup → BACKLOG
