---
id: T045
status: open
created: 2026-03-23
priority: P1
review: CEO expansion + eng review (2026-03-23)
---
# T045: Popup Timer UX Redesign

## Problem

The popup (OneBarWidget) displays confusing, redundant information:
- Big numbers show **remaining** time (`7:00 / 10:00`) instead of **elapsed**
- "Standing for 3 min" text at top duplicates the timer
- Coach message also says "7 min left" — same number repeated in three places
- Timer doesn't visually reset when state changes (keeps showing previous session's value)
- Away state uses green color (same as standing) — should be gray (unknown state)
- Progress bar inside popup only appears during sitting
- Two separate bar components (ScreenProgressBar + inline bar) with duplicated color logic

## Solution

### Core changes (4)

#### 1. Big numbers: `elapsed / total` (not `remaining / total`)

| State    | Display         | Meaning                           |
|----------|-----------------|-----------------------------------|
| Sitting  | `23:00 / 40:00` | sat 23min, limit 40min            |
| Standing | `3:00 / 10:00`  | stood 3min, target 10min to reset |
| Away     | `5:00 / 10:00`  | away 5min, reset target 10min     |

User reads: "how much I did / how much I need". No mental arithmetic.

Data sources per state — computed ONCE in `useWidgetData`:
- **Sitting**: elapsed = `currentSessionSecs`, total = `limitSecs`
- **Standing/Walking/Away**: elapsed = `breakSecs`, total = `breakResetThreshold`

```
// useWidgetData.ts — new fields
//
//   state ──→ elapsed/total/colorScheme mapping
//   Sitting  → currentSessionSecs / limitSecs / "sitting"
//   Standing → breakSecs / breakResetThreshold / "standing"
//   Walking  → breakSecs / breakResetThreshold / "standing"
//   Away     → breakSecs / breakResetThreshold / "gray"
```

#### 2. Unified ProgressBar component (DRY)

Replace two separate bar implementations with one `ProgressBar` component:
- `ScreenProgressBar.tsx` (inline styles, sitting-only) — **DELETE**
- `one-bar__progress` (CSS classes, always visible) — **REPLACE**

```
// ProgressBar.tsx — unified progress bar
//
// Variants:
//   overlay: position:fixed, top:0, full-width, z-index:999999
//   inline:  position:relative, within parent flow
//
// Color schemes:
//   sitting:  green → amber → red  (ratio 0→1)
//   standing: green gradient       (ratio 0→1, pulse animation)
//   gray:     gray                 (ratio 0→1)
//
// Guards:
//   total=0 → ratio=0, no division
//   ratio capped at 1.0 for width calculation
//
// Debug:
//   data-progress="23:00/40:00 (57%)" attribute for DevTools
```

New `ProgressBar` API:
```tsx
<ProgressBar
  elapsed={number}
  total={number}
  variant="overlay" | "inline"
  colorScheme="sitting" | "standing" | "gray"
  shimmer={boolean}           // one-shot shimmer animation
/>
```

App.tsx reads barProps from WidgetProps (computed in useWidgetData). Bar is dumb:
```tsx
<ProgressBar
  elapsed={widgetProps.elapsed}
  total={widgetProps.total}
  variant="overlay"
  colorScheme={widgetProps.colorScheme}
/>
```

Bar always visible (remove `state === "Sitting"` gate in App.tsx).

CSS class names: `progress-bar`, `progress-bar__fill`, `progress-bar__fill--sitting`, `progress-bar__fill--standing`, `progress-bar__fill--gray`. Old `one-bar__progress*` classes removed from `one-bar.css`.

#### 3. Away state = gray color

- CSS: `.one-bar--away { filter: saturate(0.5); transition: filter 0.4s; }` (desaturation)
- Temperature system: already returns "away" — no change needed
- Progress bar: `colorScheme="gray"`
- Away means "we don't know where user is" — honest, neutral

#### 4. Remove redundant "State for Xm" text

Delete duration from state label: `● standing for 3m` → `● standing`
- Duration is already in big numbers
- Remove `<span className="one-bar__session-duration">` from OneBarTimer
- Remove `.one-bar__session-duration` from CSS (dead code cleanup)

### Coach messages (updated — no countdown duplication)

Big numbers now show elapsed/total, so coach provides MOTIVATION, not statistics:

**Sitting** (unchanged — countdown is useful here because big numbers show elapsed, not remaining):
- `limitRatio < 0.5`: `"On track."`
- `limitRatio >= 0.5`: `"Half limit used. X changes today."`
- `limitRatio >= 0.8`: `"Stand up within X min."`
- `limitRatio >= 1.0`: `"Limit exceeded by X min. Losing points."`

**Standing** (new — motivational):
- `breakResetProgress < 0.5`: `"Keep going — blood is flowing."`
- `breakResetProgress >= 0.5`: `"Almost there — more than halfway."`
- `breakResetProgress >= 1.0`: `"Full reset! Sit whenever you want."`

**Away** (new — motivational):
- `breakResetProgress < 1.0`: `"Break counts toward reset."`
- `breakResetProgress >= 1.0`: `"Full reset! Come back fresh."`

### Delight animations (5)

#### D1. Smooth number transition (15 min)
200ms CSS opacity fade on big number when state changes.
- Managed by `useTimerAnimations` hook (separate file)
- Hook returns `{ isFading: boolean, isShimmering: boolean }`
- OneBarTimer applies `one-bar__big-number--fading` class when `isFading`
- CSS: `opacity: 0 → 1` transition, 200ms

#### D2. Standing bar pulse (10 min)
CSS keyframe animation on progress bar fill when standing:
```css
@keyframes bar-pulse {
  0%, 100% { opacity: 0.7; }
  50% { opacity: 1.0; }
}
.progress-bar__fill--standing {
  animation: bar-pulse 2s ease-in-out infinite;
}
```

#### D3. Reset celebration shimmer (20 min)
One-shot CSS animation when `breakResetProgress` crosses 1.0:
- Detection logic in `useTimerAnimations` hook (useRef tracks previous value)
- Hook returns `isShimmering: boolean` → passed to `<ProgressBar shimmer={true} />`
- CSS: bright green flash, 500ms, then settle to normal
- Must handle rapid state changes: if state changes during shimmer, cancel shimmer

```
// useTimerAnimations state machine:
//   IDLE ──(state changed)──→ FADING (200ms) ──→ IDLE
//   IDLE ──(resetProgress crosses 1.0)──→ SHIMMERING (500ms) ──→ IDLE
//   FADING + state changed ──→ restart FADING (no stacking)
```

#### D4. Away desaturation (5 min)
```css
.one-bar--away { filter: saturate(0.5); transition: filter 0.4s; }
```

#### D5. Big number tooltip (20 min)
Native `title` attribute — zero overhead:
```
title="Sitting 23m of 40m. 6 changes today. Score: +12"
```

## Files to modify

### NEW files
- `src/components/ProgressBar.tsx` (~60 lines) — unified progress bar
- `src/components/ProgressBar.css` (~50 lines) — bar styles, keyframes
- `src/components/ProgressBar.test.tsx` (~80 lines) — unit tests
- `src/hooks/useTimerAnimations.ts` (~40 lines) — fade + shimmer detection
- `src/hooks/useTimerAnimations.test.ts` (~50 lines) — animation hook tests

### MODIFIED files
- `src/types.ts` — add `elapsed`, `total`, `colorScheme` to WidgetProps
- `src/hooks/useWidgetData.ts` — compute elapsed/total/colorScheme from state
- `src/widgets/one-bar/OneBarTimer.tsx` — elapsed/total from props, remove duration label, apply animations, tooltip
- `src/widgets/one-bar/OneBarCoach.tsx` — motivational standing/away messages
- `src/widgets/OneBarWidget.tsx` — import ProgressBar instead of inline bar
- `src/App.tsx` — remove `state === "Sitting"` gate, use ProgressBar variant="overlay" with props from widgetProps
- `src/widgets/one-bar/one-bar.css` — away desaturation, remove `.one-bar__session-duration`, remove `one-bar__progress*` (moved to ProgressBar.css)

### DELETED files
- `src/components/ScreenProgressBar.tsx` — replaced by unified ProgressBar

### JSDoc updates (stale after T045)
- `OneBarTimer.tsx:1-5` — "Shows limitRemaining" → "Shows elapsed/total"
- `App.tsx:1-6` — mentions ScreenProgressBar → update to ProgressBar

### TESTS to UPDATE (~6 breaking)
- `src/widgets/OneBarWidget.test.tsx:96-109` — asserts limitRemaining in big number → change to elapsed
- `src/widgets/OneBarWidget.test.tsx:125-137` — asserts "for 3m" text → remove assertion
- `src/widgets/OneBarWidget.test.tsx:139-150` — asserts "for 4m" text → remove assertion
- `src/widgets/OneBarWidget.test.tsx:152-168` — asserts "for 7m" text → remove assertion
- `src/widgets/OneBarWidget.test.tsx:170-181` — asserts "for 5m" text → remove assertion
- `src/widgets/one-bar/OneBarCoach.test.tsx:67-76` — asserts "min left until reset" → change to motivational

### TEST FACTORIES to UPDATE (add defaults for new WidgetProps fields)
- `src/widgets/OneBarWidget.test.tsx:10-33` — add `elapsed: 600, total: 2400, colorScheme: "sitting"`
- `src/widgets/one-bar/OneBarCoach.test.tsx:8-31` — same
- `src/widgets/one-bar/temperature.test.ts:8-31` — same

### NEW tests (~15)

**ProgressBar component (5):**
- overlay variant renders with fixed positioning
- inline variant renders relative positioning
- total=0 returns 0% (no div by zero)
- gray colorScheme applies correct class
- data-progress attribute shows debug info

**useTimerAnimations hook (4):**
- state change triggers fade (isFading=true for 200ms)
- breakResetProgress crossing 1.0 triggers shimmer
- breakResetProgress already at 1.0 does NOT trigger shimmer
- rapid state change within 200ms resets fade (no stacking)

**useWidgetData new fields (1):**
- sitting returns elapsed=currentSessionSecs, total=limitSecs, colorScheme="sitting"
- standing returns elapsed=breakSecs, total=breakResetThreshold, colorScheme="standing"
- away returns elapsed=breakSecs, total=breakResetThreshold, colorScheme="gray"

**OneBarTimer display (3):**
- sitting shows elapsed/total (currentSessionSecs / limitSecs)
- standing shows elapsed/total (breakSecs / breakResetThreshold)
- away shows elapsed/total in gray

**OneBarCoach motivational messages (2):**
- standing breakResetProgress < 0.5 returns "Keep going"
- standing breakResetProgress >= 0.5 returns "Almost there"

**CSS class presence (1):**
- away state applies desaturation class on widget

## No Rust changes needed

Session data already includes all needed fields:
- `current_session_secs` (sitting elapsed)
- `break_seconds` (standing/away elapsed)
- `session_limit_secs` (sitting total)
- `breakResetThreshold` (standing/away total)
- `breakResetProgress` (for coach + shimmer detection)

## Acceptance criteria

### Core
- [ ] Sitting: big numbers show `23:00 / 40:00` (elapsed / limit)
- [ ] Standing: big numbers show `3:00 / 10:00` (elapsed / target)
- [ ] Away: big numbers show `5:00 / 10:00` (elapsed / target, gray)
- [ ] State label shows `● sitting` without duration text
- [ ] Unified ProgressBar replaces both ScreenProgressBar and inline bar
- [ ] Progress bar visible in all states (sitting=gradient, standing=green, away=gray)
- [ ] Away = gray + desaturated everywhere (popup glow, bar, state dot)
- [ ] Coach standing/away: motivational messages, no countdown duplication
- [ ] Timer resets visually on state change (no stale values)
- [ ] ProgressBar guards total=0 (no division by zero)

### Delight
- [ ] D1: 200ms fade transition on big numbers at state change
- [ ] D2: Pulse animation on standing bar (2s heartbeat cycle)
- [ ] D3: Shimmer celebration on full reset (one-shot 500ms)
- [ ] D4: Away desaturation (filter: saturate(0.5))
- [ ] D5: Tooltip on big numbers (hover shows full breakdown)

### Tests
- [ ] ~6 breaking tests updated
- [ ] ~3 test factories updated with new WidgetProps defaults
- [ ] ~15 new tests added
- [ ] All existing tests pass
- [ ] ScreenProgressBar.tsx deleted, no dead imports
- [ ] Stale JSDoc updated (OneBarTimer, App.tsx)

## Review decisions log

### CEO Review (2026-03-23, SCOPE EXPANSION)

| Decision | Choice | Why |
|----------|--------|-----|
| Review mode | SCOPE EXPANSION | Bug-prone area but app is experimentation platform — delight matters |
| ProgressBar API | Generic props (elapsed, total, colorScheme) | App.tsx computes, bar is dumb. Minimal diff. |
| Bar height in popup | 4px (same as overlay) | Consistency over prominence |
| DRY bars | Unify into one ProgressBar | Eliminates duplication, prepares for KPI dashboard |
| Away display | With target (5:00 / 10:00) | Away also counts toward reset — show progress |
| Coach messages | Motivational (not countdown) | Big numbers inform, coach motivates — clear separation |
| All 5 delights | Include in T045 | Total ~70 min extra, high impact for experimentation platform |

### Eng Review (2026-03-23, BIG CHANGE)

| # | Decision | Choice | Why |
|---|----------|--------|-----|
| 1 | Elapsed/total mapping location | useWidgetData (add to WidgetProps) | DRY — single source of truth, computed once, consumed by App.tsx + OneBarTimer |
| 2 | Shimmer detection location | OneBarTimer (useTimerAnimations hook) | ProgressBar stays pure render. Explicit > clever. |
| 3 | OneBarTimer complexity | Extract hooks to useTimerAnimations.ts | Keep OneBarTimer as pure render. Hook testable independently. |
| — | WidgetProps expansion | Add elapsed, total, colorScheme + update 3 test factories | Mechanical change, no design decision needed |
| — | CSS class naming | New namespace (progress-bar__*) for standalone component | Obvious: new component = new BEM block |
| — | Stale JSDoc | Update OneBarTimer + App.tsx comments | Part of the change |
