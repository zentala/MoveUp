---
id: E005-T08
epic: E005
status: done
created: 2026-03-21
completed: 2026-03-22
original_id: T025
---
# T025 — Welcome / onboarding popup on first launch

**Status:** open
**Priority:** P2
**Branch:** feat/T025-welcome-popup

---

## Goal

Show a friendly welcome popup when the app runs for the first time.

Two purposes:
1. Introduce the app warmly to the user
2. Validate that the Tauri WebviewWindow popup mechanism works

## Key Decisions

- Uses `WebviewWindowBuilder` (NOT WinAPI) — full CSS/React, draggable
- Button "Przetestuj powiadomienia" calls `trigger_test_notification` (T024 command)
- `show_welcome_on_startup: bool` with `#[serde(default = "bool_true")]` — default true
- Window: 480x420px, always-on-top, centered

---

## Acceptance Criteria

- [ ] Welcome popup appears on first run
- [ ] "Gotowy! Zaczynamy!" button closes the popup
- [ ] "Nie pokazuj" checkbox -> no popup on next launch
- [ ] "Pokaz intro ponownie" button in Settings reopens popup
- [ ] Popup is draggable
- [ ] `cargo test` passes, `pnpm test:unit` passes
