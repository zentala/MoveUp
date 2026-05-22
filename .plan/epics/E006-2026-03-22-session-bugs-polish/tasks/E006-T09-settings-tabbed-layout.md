---
id: E006-T09
epic: E006
status: completed
created: 2026-03-22
completed: 2026-03-23
original_id: T035
title: T035 — Settings panel: tabbed layout instead of scrolling
---
# T035 — Settings panel: tabbed layout instead of scrolling

**Status:** open
**Priority:** P2
**Depends on:** none

---

## Problem

The settings panel is wider than tall (420x240 window). Currently all sections
stack vertically with `overflow-y: auto`, requiring scrolling. This feels
clunky in a compact instrument panel.

## Solution

Replace the scrollable layout with horizontal tabs. Each tab shows one section:

### Tabs

| # | Tab      | Content                           |
|---|----------|-----------------------------------|
| 1 | Time     | Sitting/standing limit sliders    |
| 2 | Calibr.  | Sitting/standing height, thickness |
| 3 | Notif.   | Notification toggles              |
| 4 | More     | Widget picker, "Show intro", etc. |

### Implementation

1. Add `activeTab` state (0-3) to SettingsPanel
2. Create `SettingsTabBar` component with tab buttons
3. Render only the active section's content (no scroll needed)
4. Tab bar uses `--rail-*` borders, active tab uses `--beam` accent
5. Back/Save footer stays pinned at bottom across all tabs

### Files to modify

- `src/components/SettingsPanel.tsx` — add tab state and tab bar
- `src/styles/globals.css` — add tab styles
- New: `src/components/settings/SettingsTabBar.tsx` (small component)
