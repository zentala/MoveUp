# T032 — Widget "One Bar"

**Priority:** P2
**Depends on:** T031 (widget architecture)
**Wave:** 5 (parallel with T033)

## Goal

Implement the "One Bar" widget: horizontal popup with timeline, unified progress bar,
temperature escalation, and one-line coach.

## Design Reference

Mockups: `apps/desk/.superpowers/brainstorm/14480-1774063099/one-bar.html`
Vision: `apps/desk/.agent/vision/2026-03-21-product-vision-coach-and-business.md`

## Core Concept

**One bar, one meaning:** the progress bar represents limit usage.
- Sitting: bar fills up (0 → limit). Full = must stand.
- Standing/Away: bar drains back down. Empty = full reset.
- Same visual element, always the same meaning: less = better.

**Temperature escalation:** the entire widget changes color temperature
based on how close to the limit.
- Early sitting (0-50%): dark, calm, muted red
- Mid sitting (50-80%): warming, visible red
- Late sitting (80%+): hot, bright red, glowing border
- Standing: green, calm, triumphant
- Away: gray, neutral
- After reset: bright green, "✓ zresetowany"

## Layout (440×200px, horizontal)

```
┌──────────────────────────────────────────────┐
│ ↕ desk  72.4 cm                           ⚙ │  ← header: title + height + settings
│                                              │
│ ▐████████░░░░▐███░░░▐██████████▐██▐████░░░░│  ← timeline (28px tall, hover → details)
│  09:00    10:00    11:00    12:00    teraz   │
│                                              │
│ ● SIEDZĘ · 23 min        poprzednio:        │  ← state label + previous session
│ 23:14 / 40:00             stałem 16 min ✓   │  ← BIG timer + previous
│ ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓░░░░░░░░░░░░░░░░░░░░░│  ← THE bar (5px)
│                                              │
│ ▌ Zostało 17 min. Dobry moment na zmianę.   │  ← coach sentence
└──────────────────────────────────────────────┘
```

## Components

### `src/widgets/OneBarWidget.tsx` (~120 lines)

Main widget component. Receives `WidgetProps`.

```tsx
const OneBarWidget: FC<WidgetProps> = (props) => {
  const temperature = computeTemperature(props);

  return (
    <div className={`one-bar one-bar--${temperature}`}>
      <OneBarHeader {...props} />
      <OneBarTimeline sessions={props.todaySessions} state={props.state} />
      <OneBarTimer {...props} />
      <OneBarCoach {...props} />
    </div>
  );
};
```

### `src/widgets/one-bar/OneBarTimeline.tsx` (~80 lines)

Timeline component:
- Proportional blocks for each session today
- Colors: sitting=#6a1a1a..#c43030, standing=#1a5528..#1a6630, away=#333
- Current session has glowing right edge (box-shadow)
- Ghost rhythm lines (vertical, every `sit_limit_mins`, 10% opacity yellow)
- Hover on block → tooltip: "09:12 — siedziałem 42 min"
- Time labels at top: hours from first session to now

### `src/widgets/one-bar/OneBarTimer.tsx` (~60 lines)

Timer area:
- Left: state label (● SIEDZĘ/STOJĘ/AWAY) + big timer (`limitRemaining` formatted as mm:ss) + "/ 40:00"
- Right: previous session info ("poprzednio: stałem 16 min ✓")
- Below: THE progress bar (5px, one direction)
  - Sitting: fills with red gradient, width = `limitRatio * 100%`
  - Standing/Away: fills with green, width = `limitRatio * 100%` (draining)
  - After reset: width = 0%, green

### `src/widgets/one-bar/OneBarCoach.tsx` (~50 lines)

One sentence, context-dependent:

| State | Condition | Message |
|-------|-----------|---------|
| Sitting | ratio < 0.5 | "W normie." |
| Sitting | ratio 0.5–0.8 | "Połowa limitu. {changes} zmian dziś." |
| Sitting | ratio > 0.8 | "Wstań w ciągu {remaining} min." |
| Sitting | ratio >= 1.0 | "Limit przekroczony. Wstań teraz." |
| Standing | resetProgress < 1.0 | "Jeszcze {toReset} min do resetu." |
| Standing | resetProgress >= 1.0 | "Reset! Możesz usiąść — masz pełne {limit} min." |
| Away | resetProgress < 1.0 | "Przerwa się liczy. Do resetu {toReset} min." |
| Away | resetProgress >= 1.0 | "Reset! Wróć i masz pełne {limit} min." |

### `src/widgets/one-bar/temperature.ts` (~30 lines)

Computes CSS class name based on state + ratio:

```typescript
type Temperature = "calm" | "warm" | "hot" | "burning" | "standing" | "away" | "reset";

function computeTemperature(props: WidgetProps): Temperature {
  if (props.state === "Away") return "away";
  if (props.state === "Standing" || props.state === "Walking") {
    return props.breakResetProgress >= 1.0 ? "reset" : "standing";
  }
  if (props.limitRatio >= 1.0) return "burning";
  if (props.limitRatio >= 0.8) return "hot";
  if (props.limitRatio >= 0.5) return "warm";
  return "calm";
}
```

### `src/widgets/one-bar/one-bar.css` (~100 lines)

Temperature-based theming:
```css
.one-bar--calm    { --bg: #0c0c0c; --border: #1e1e1e; --glow: none; }
.one-bar--warm    { --bg: #0e0c0c; --border: #2a1a1a; --glow: rgba(140,40,40,0.04); }
.one-bar--hot     { --bg: #110c0c; --border: #3a1a1a; --glow: rgba(180,40,40,0.06); }
.one-bar--burning { --bg: #140c0c; --border: #4a1a1a; --glow: rgba(200,40,40,0.1); }
.one-bar--standing { --bg: #0c0d0c; --border: #1a2a1a; --glow: rgba(40,100,60,0.05); }
.one-bar--away    { --bg: #0c0c0d; --border: #222; --glow: none; }
.one-bar--reset   { --bg: #0c0e0c; --border: #1a3a1a; --glow: rgba(40,120,60,0.07); }
```

## Files to create

| File | Lines |
|------|-------|
| `src/widgets/OneBarWidget.tsx` | ~120 |
| `src/widgets/one-bar/OneBarTimeline.tsx` | ~80 |
| `src/widgets/one-bar/OneBarTimer.tsx` | ~60 |
| `src/widgets/one-bar/OneBarCoach.tsx` | ~50 |
| `src/widgets/one-bar/temperature.ts` | ~30 |
| `src/widgets/one-bar/one-bar.css` | ~100 |

## Files to modify

| File | Change |
|------|--------|
| `src/widgets/registry.ts` | Register OneBarWidget |

## Tests

- `temperature.ts`: all 7 temperature values for edge cases
- `OneBarCoach`: correct message for each state+condition combo
- `OneBarTimeline`: renders correct number of blocks from sessions[]
- `OneBarTimer`: shows limitRemaining (not sittingSeconds) as big number
- `OneBarWidget`: renders without crash, snapshot test per temperature

## Acceptance criteria

- [ ] Widget renders in all 3 states (sitting/standing/away)
- [ ] Temperature escalation visible (background, border, glow change)
- [ ] Progress bar has ONE meaning: fills when sitting, drains when standing/away
- [ ] Big number shows limitRemaining, not session duration
- [ ] Timeline hover shows session details (tooltip)
- [ ] Coach sentence is context-appropriate (max 1 sentence)
- [ ] Ghost rhythm lines visible on timeline
- [ ] Desk height shown in header
- [ ] No file > 250 lines
- [ ] All tests pass
