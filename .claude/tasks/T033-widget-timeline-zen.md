# T033 — Widget "Timeline Zen"

**Priority:** P3
**Depends on:** T031 (widget architecture)
**Wave:** 5 (parallel with T032)

## Goal

Minimalist widget. Timeline big, numbers small, zero text.
For users who don't want coaching — just a visual rhythm check.

## Design Concept

No coach sentence. No previous session card. No points.
Just: timeline + one number + one bar.

The timeline IS the interface. Everything you need is in the proportions
and colors of the blocks.

```
┌──────────────────────────────────────────────┐
│                                              │
│ ▐████████░░░░▐███░░░▐██████████▐██▐████░░░░│  ← timeline (48px tall — BIGGER)
│  09         10         11         12    now  │
│                                              │
│          ● 23:14 / 40:00           72.4 cm  │  ← one timer line + height
│ ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓░░░░░░░░░░░░░░░░░░░░░│  ← the bar
│                                              │
└──────────────────────────────────────────────┘
```

Entire widget: ~140px tall. Ultra compact.

## Key Differences from One Bar

| Aspect | One Bar | Timeline Zen |
|--------|---------|--------------|
| Timeline height | 28px | 48px (hero) |
| Coach sentence | yes | no |
| Previous session | shown | hidden (visible on timeline) |
| Temperature escalation | full (bg, border, glow) | subtle (only bar color) |
| Header | title + height + settings | height + settings only |
| Total height | ~200px | ~140px |

## Components

### `src/widgets/TimelineZenWidget.tsx` (~60 lines)

```tsx
const TimelineZenWidget: FC<WidgetProps> = (props) => {
  return (
    <div className="zen-widget">
      <ZenTimeline sessions={props.todaySessions} state={props.state} />
      <ZenStatus {...props} />
    </div>
  );
};
```

### Timeline — uses shared `<SessionTimeline>` from T031

Import `SessionTimeline` from `@/widgets/shared/SessionTimeline`.
Pass: `height={48}`, no legend, ghost opacity 15%.

Differences from One Bar usage:
- 48px tall (not 28px) — the hero element
- No legend (colors are self-explanatory after first use)
- Hover still shows tooltip
- Ghost rhythm lines slightly more visible (15% opacity)
- Time labels: just hours, no "teraz" label (the glowing edge IS "teraz")

### `src/widgets/timeline-zen/ZenStatus.tsx` (~40 lines)

Single centered line:
- State dot (colored) + timer (`limitRemaining` / `limitSecs`) + height
- Below: the bar (3px, thinner than One Bar)
- No text. Just the number and the bar.

```tsx
<div className="zen-status">
  <span className={`zen-dot zen-dot--${props.state?.toLowerCase()}`}>●</span>
  <span className="zen-timer">{formatTimer(limitRemaining)}</span>
  <span className="zen-separator">/</span>
  <span className="zen-limit">{formatTimer(limitSecs)}</span>
  <span className="zen-height">{deskHeightCm.toFixed(1)} cm</span>
</div>
<div className="zen-bar">
  <div className="zen-bar__fill" style={{ width: `${ratio}%` }} />
</div>
```

### `src/widgets/timeline-zen/timeline-zen.css` (~60 lines)

Minimal theming. Only the bar and dot change color:
- Sitting: red dot, red bar
- Standing: green dot, green bar
- Away: gray dot, gray bar
- After reset: bright green dot, empty bar

Background always dark. No glow, no border changes.

## Files to create

| File | Lines |
|------|-------|
| `src/widgets/TimelineZenWidget.tsx` | ~60 |
| ~~`src/widgets/timeline-zen/ZenTimeline.tsx`~~ | Uses shared `SessionTimeline` from T031 |
| `src/widgets/timeline-zen/ZenStatus.tsx` | ~40 |
| `src/widgets/timeline-zen/timeline-zen.css` | ~60 |

## Files to modify

| File | Change |
|------|--------|
| `src/widgets/registry.ts` | Register TimelineZenWidget |

## Tests

- `ZenTimeline`: renders correct blocks, 48px height
- `ZenStatus`: shows limitRemaining not sittingSeconds
- `TimelineZenWidget`: snapshot test, renders in all 3 states

## Acceptance criteria

- [ ] Widget renders in all 3 states
- [ ] Timeline is 48px tall (hero element)
- [ ] No coach text, no previous session card
- [ ] Timer shows limitRemaining
- [ ] Bar drains when standing/away (same logic as One Bar)
- [ ] Total widget height ≤ 140px
- [ ] No file > 250 lines
