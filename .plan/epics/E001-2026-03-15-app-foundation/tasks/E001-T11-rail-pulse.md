---
id: E001-T11
epic: E001
status: completed
completed: 2026-03-21
original_id: "0011"
title: T011 — Height Rail Pulse Animation on Sit->Stand Transition
---
# T011 — Height Rail Pulse Animation on Sit->Stand Transition

**Priority**: P3
**Status**: open
**Depends on**: T005 (position_changes in state payload to detect transition)

## Goal
When the user stands up after a sitting session, the amber indicator dot on the
HeightRail briefly pulses — a single soft glow animation. Acknowledges the transition.

## Scope

### `globals.css` — add keyframe
```css
@keyframes rail-pulse {
  0%   { box-shadow: 0 0 7px var(--beam-glow); }
  40%  { box-shadow: 0 0 16px var(--beam-glow), 0 0 28px var(--beam-dim); transform: translate(-50%, 50%) scale(1.4); }
  100% { box-shadow: 0 0 7px var(--beam-glow); transform: translate(-50%, 50%) scale(1); }
}

.height-rail__indicator--pulse {
  animation: rail-pulse 0.6s ease-out forwards;
}
```

### `HeightRail.tsx`
Add `prevState` prop or detect state change internally via `useEffect`:
```tsx
// When state changes FROM Sitting TO Standing/Walking, add pulse class
// Remove class after animation completes (600ms)
```

### `App.tsx`
Pass `state` prop to `HeightRail` so it can detect the Sitting->Standing transition.

## Tests
- Vitest: `height-rail__indicator--pulse` class added when state changes Sitting->Standing
- Vitest: pulse class removed after 600ms
- Vitest: pulse NOT added when state changes Standing->Walking (already standing)

## Acceptance criteria
- [ ] Pulse animates on Sitting->Standing transition
- [ ] No animation on Standing->Walking or other same-break transitions
- [ ] Animation is subtle (does not flash harshly)
