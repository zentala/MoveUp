---
id: E012-T04
epic: E012
status: planned
created: 2026-05-16
branch: feat/E012-T04-window
---

# E012-T04 — Analyst window registration + tray menu entry

## Goal
Register a new Tauri window labeled `"analyst"` (separate from the existing `"main"` popup window) and add a tray menu item "Open Analyst" that shows/focuses it.

## Why
The Analyst UI is a long-lived inspection tool. Hiding it inside the popup would crowd the overlay. A separate window lets zentala keep both visible — popup for current state, analyst for the data view.

## Tauri config
Edit `apps/desk/src-tauri/tauri.conf.json`:

```jsonc
"windows": [
  { /* existing main window */ },
  {
    "label": "analyst",
    "title": "Smart Desk — Analyst",
    "url": "index.html#/analyst",
    "width": 1280,
    "height": 800,
    "minWidth": 1024,
    "minHeight": 600,
    "visible": false,
    "decorations": true,
    "resizable": true,
    "center": true
  }
]
```

## Tray menu
Edit `apps/desk/src-tauri/src/tray.rs`:
- Add menu item `"Open Analyst"` between existing items (suggest: under "Show widget")
- On click: `app.get_webview_window("analyst")` → show + set_focus; if missing, log error

## Single-instance interplay
The existing single-instance plugin focuses `"main"`. No change needed — Analyst is a child window of the same process.

## Route registration
In `apps/desk/src/App.tsx`:
- Detect `window.location.hash === '#/analyst'` and render `<AnalystWindow>` from `T03/analyst/` directly (live mode — but this task DOES NOT wire real data; it just routes)
- T05 swaps fixture data for real `invoke()` calls

For T04 scope: the analyst window opens, renders the same mockup component tree from T03 with the **same fixtures** (so a temporary feature flag like `__USE_MOCKUP_FIXTURES__ = true` in this task). T05 removes the flag.

## Capabilities
Edit `apps/desk/src-tauri/capabilities/default.json`:
- Ensure the new `analyst` window can `invoke` the new commands from T01/T02 (extend `windows: ["main", "analyst"]` on relevant permissions)

## Tests
**Rust unit:**
- `tray::build_menu()` includes an item with id `"open-analyst"`

**Manual smoke (zentala runs in PR):**
- Right-click tray → click "Open Analyst" → window appears centered, 1280×800
- Close window → tray click reopens it (not a new window — re-shows the existing one)

## Files touched
- `apps/desk/src-tauri/tauri.conf.json`
- `apps/desk/src-tauri/src/tray.rs`
- `apps/desk/src-tauri/capabilities/default.json`
- `apps/desk/src/App.tsx`

## DoD
- [ ] Both windows coexist; popup overlay still works
- [ ] Closing Analyst window does NOT close the app
- [ ] Tray menu item works (manual)
- [ ] All existing tests pass
