---
id: E006-T06
epic: E006
status: completed
created: 2026-03-22
completed: 2026-03-22
original_id: T034
title: T034 — Visual layout polish (manual review required)
---
# T034 — Visual layout polish (manual review required)

**Status:** open
**Priority:** P1
**Depends on:** User visual review (screenshots)

---

## Context

After the UX Communication Sprint, the floating window has a new widget system.
User reported layout issues on first test:
1. Panel didn't fill the window box — gaps on sides and large empty space at bottom
2. Settings opened as default view instead of widget
3. Timer showed "...8m" (truncated/unclear)

Initial fixes applied (commit 10c9178):
- Window resized to 420x240
- flex:1 layout for widget fill
- "CONNECTING" label instead of "..."
- Removed auto-settings on default calibration

## Remaining work (needs visual testing)

### Layout verification
- [ ] One Bar widget fills entire 420x240 window with no dead space
- [ ] Timeline bar height is proportional (grows with window)
- [ ] No horizontal overflow or misaligned elements
- [ ] Settings panel scrollable when window is small

### Responsiveness
- [ ] Window resizable — widget adapts gracefully
- [ ] Minimum useful size (don't let it get too small to read)
- [ ] Maximum size doesn't stretch elements absurdly

### Visual polish
- [ ] Temperature colors visible and distinctive
- [ ] Coach text readable (not cut off)
- [ ] Big number (limitRemaining) is prominent and clear
- [ ] Timeline blocks have visible colors for each state
- [ ] Progress bar (5px) visible and correctly positioned

### Settings panel
- [ ] Opens via gear icon (not by default)
- [ ] Back button returns to widget
- [ ] Widget picker dropdown works
- [ ] All sliders and inputs fit within panel

## How to test

```bash
pnpm tauri:dev
```

Look at the floating window, interact with all states.
Screenshot any issues for follow-up fixes.
