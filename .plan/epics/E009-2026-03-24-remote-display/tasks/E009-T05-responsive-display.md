---
id: E009-T05
epic: E009
status: completed
created: 2026-03-24
branch: feat/E009-T05-responsive-display
depends_on: [E009-T04]
title: Responsive Layout for Phone Display
---
# E009-T05: Responsive Layout for Phone Display

## What
Make the existing popup UI responsive so it looks good fullscreen on a phone
in landscape orientation. No new components — adapt existing ones with CSS.

## Context
The current popup is designed for a small desktop window (~400×500px).
A phone in landscape is typically ~800×360px (or higher on modern phones).
The layout needs to fill the screen without scrolling.

The app uses CSS variables and a design system (see `.arch/UX-FLOW.md`).
Changes should use existing variables, not introduce new ones.

## Implementation

### 1. Viewport meta tag
In `index.html`, ensure:
```html
<meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
```

### 2. Landscape enforcement
In main CSS file, add:
```css
/* Force landscape orientation hint for kiosk browsers */
@media (orientation: portrait) {
  body::before {
    content: "Please rotate to landscape";
    /* Full-screen overlay prompting rotation */
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--panel-bg, #1a1a2e);
    color: var(--ink-secondary, #888);
    font-size: 1.5rem;
    z-index: 9999;
  }
}
```

### 3. Remote display mode styles
Detect remote mode via a CSS class on `<body>` or `<html>`:

In `App.tsx` (or entry point):
```typescript
if (!window.__TAURI_INTERNALS__) {
  document.documentElement.classList.add('remote-display');
}
```

Then in CSS:
```css
.remote-display {
  /* Fill entire screen */
  width: 100vw;
  height: 100vh;
  overflow: hidden;

  /* Larger base font for readability at arm's length */
  font-size: clamp(14px, 2.5vw, 20px);
}

/* Hide settings gear in remote mode */
.remote-display .settings-trigger {
  display: none;
}

/* Maximize widget area */
.remote-display .widget-container {
  height: 100vh;
  padding: 0;
}
```

### 4. Widget adaptations
Each existing widget may need minor tweaks:

**OneBarWidget**: Main widget with progress bar + KPIs + timeline.
- KPI badges: use `flex-wrap` so they reflow on narrow heights
- Timeline: may need reduced height or horizontal scroll
- Progress bar: `width: 100%` (already responsive)

**TimelineZenWidget**: Timeline-focused widget.
- Should fill available space naturally

### 5. Font scaling
Replace fixed `px` sizes with `clamp()` where needed:
```css
/* Example: KPI badge value */
.kpi-value {
  font-size: clamp(1rem, 3vw, 1.5rem);
}
```

### 6. Screen always-on hint
For kiosk browsers that support it, add:
```html
<meta name="mobile-web-app-capable" content="yes">
<meta name="apple-mobile-web-app-capable" content="yes">
```

And in the remote hook, request wake lock if available:
```typescript
if ('wakeLock' in navigator) {
  navigator.wakeLock.request('screen').catch(() => {});
}
```

## What NOT to change
- Don't create a new widget or layout specifically for phone
- Don't change the desktop popup appearance
- Don't add new CSS variables or design tokens
- Don't restructure component hierarchy

## Testing

### Manual testing (most effective for responsive):
1. Open `http://localhost:3390/display` in Chrome DevTools → toggle device toolbar
2. Select a phone preset (e.g. Pixel 5) → landscape
3. Verify: no scrolling needed, all content visible, text readable
4. Test portrait → should show "rotate to landscape" overlay
5. Test on actual phone if available

### Unit tests:
- Existing component tests should still pass (CSS changes don't break rendering)
- Add test: `remote-display` class is added when `__TAURI_INTERNALS__` is absent

## Definition of Done
- [ ] Phone landscape shows full UI without scrolling
- [ ] Portrait shows rotation prompt
- [ ] Settings gear hidden in remote mode
- [ ] Text readable at arm's length (phone on desk)
- [ ] Desktop popup unchanged
- [ ] Wake lock requested in remote mode
- [ ] Existing tests pass
