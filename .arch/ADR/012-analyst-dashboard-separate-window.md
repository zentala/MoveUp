# ADR 012 — Analyst Dashboard as a Separate Tauri Window

- **Status**: accepted
- **Date**: 2026-05-16
- **Epic**: E012

## Context

E012 introduced a "data analyst" surface for inspecting all data sources the desk app produces and exploring 7 days of snapshots/events/sessions. Three viable hosting options existed:

1. **Route swap on the existing popup window** — reuse the 420×240 `main` window, swap content based on hash route.
2. **Dedicated Tauri window** — separate `analyst` webview window registered in `tauri.conf.json`, opened from the tray.
3. **Embedded browser tab** — serve over the existing remote WS server on `:3390` and open in default browser.

Each had real trade-offs. The decision affected window registration, IPC capability scoping, tray-menu wiring, and the cost of future Analyst features.

## Decision

**Use a dedicated Tauri window** (`label: "analyst"`, 1280×800, decorated, resizable). Opened from the tray menu item "Open Analyst". Loads `/#/analyst` for live mode and `/#/mockup/analyst` during design iteration.

## Alternatives

### Route swap on `main`
- ✅ Cheapest to implement — no new window, no capability changes.
- ❌ `main` is a 420×240 frameless overlay-style popup; the analyst surface needs much more screen real estate (5 charts + a sortable catalog table). Resizing `main` would break its role as a fast glanceable status popup.
- ❌ Single window means switching to Analyst hides the live status popup. Friction every time the user wants both visible.
- ❌ Forces all popup-shell decisions (transparency, alwaysOnTop, decorations) on the Analyst surface too.

### Browser tab via remote server
- ✅ Zero Tauri window cost; "free" cross-device viewing (phone, tablet).
- ❌ Authentication: remote server has no auth gate; exposing analyst publicly leaks 7 days of behavior data.
- ❌ Loses Tauri `invoke` — would force a new HTTP/WS API mirroring `commands_analyst.rs`, doubling the surface area.
- ❌ User explicitly wanted "an extra fullpage view inside this app", not a browser tab.

## Consequences

- One additional window registered in `tauri.conf.json`. Cost: ~10 lines of config + capability allowlist entry.
- Single-instance plugin still focuses `main` on second launch — unchanged behavior.
- Tray menu grew by one entry ("Open Analyst") between Settings and Quit.
- Future Analyst features (export, filters, custom date ranges past 7 days) can extend the same window without touching `main`'s popup ergonomics.
- Migration cost if reversed: low — analyst is hooks-driven; could be hosted in any container.
- The `/#/mockup/analyst` route stays in production builds as a no-Tauri-runtime fallback for design iteration and Storybook-like flows.

## Related

- E012 PLAN.md
- T04 task: `apps/desk/.plan/epics/E012-2026-05-16-analyst-dashboard/tasks/E012-T04-window.md`
- UX-FLOW.md §11 — describes user-facing behavior of the window
