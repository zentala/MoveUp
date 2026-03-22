# Design System — Desk App

## Direction

**Precision instrument panel** — oiled-steel surfaces, amber active indicators,
warm readout text. The app should feel like bespoke hardware software built
for a specific desk, not a generic SaaS dashboard.

Think: cockpit instrument, mechanical watch face, oscilloscope display.

## Principles

1. **One font** — full monospace. Everything reads like a calibration readout.
2. **Borders only** — no shadows. Depth comes from surface color elevation.
3. **Warm palette** — near-black with amber warmth. Never cold gray (#ccc).
4. **Data density** — every pixel earns its place. No decorative elements.
5. **Temperature escalation** — visual urgency builds through background color shift, not flashy UI changes.

## Tokens

### Surfaces (4-level elevation)

| Token | Value | Usage |
|-------|-------|-------|
| `--panel-base` | `#141210` | Window background |
| `--panel-raised` | `#1c1a17` | Cards, sections |
| `--panel-elevated` | `#242118` | Active/pressed states |
| `--panel-overlay` | `#2c2920` | Tooltips, dropdowns |

### Borders (amber-tinted rails)

| Token | Opacity | Usage |
|-------|---------|-------|
| `--rail-subtle` | 5% | Default borders, dividers |
| `--rail-default` | 10% | Input borders |
| `--rail-emphasis` | 22% | Hover borders |
| `--rail-active` | 45% | Focus rings |

### Text (warm off-white hierarchy)

| Token | Value | Usage |
|-------|-------|-------|
| `--ink-primary` | `#f0ebe0` | Primary text, big numbers |
| `--ink-secondary` | `#b0a898` | Secondary labels |
| `--ink-tertiary` | `#706858` | Tertiary, muted labels |
| `--ink-muted` | `#403830` | Barely visible, disabled |

### Amber Beam (active indicator)

| Token | Value | Usage |
|-------|-------|-------|
| `--beam` | `#d97706` | Primary accent (desk height, standing) |
| `--beam-hi` | `#f59e0b` | Hover/active beam |
| `--beam-glow` | `rgba(217,119,6,0.20)` | Glow around beam elements |
| `--beam-dim` | `rgba(217,119,6,0.08)` | Subtle beam hint |

### Semantic Signals

| Token | Value | Meaning |
|-------|-------|---------|
| `--signal-ok` | `#65a30d` | Good state (sitting within limit) |
| `--signal-warn` | `#c2762d` | Warning (approaching limit) |
| `--signal-alert` | `#b91c1c` | Alert (limit exceeded, overtime) |
| `--signal-up` | `#d97706` | Standing/active (amber) |
| `--signal-away` | `#4a7c9e` | Away/idle (cool blue-gray) |

## Typography

- **Font:** `ui-monospace, "Cascadia Code", "Fira Mono", "Consolas", monospace`
- **Base size:** 13px
- **Line height:** 1.5
- **Big numbers:** 26px, weight 700, `font-variant-numeric: tabular-nums`
- **Labels:** 10-11px, uppercase, `letter-spacing: 0.08em`
- **Smoothing:** `-webkit-font-smoothing: antialiased`

## Spacing

- **Base unit:** 4px
- **Common gaps:** 4, 7, 8, 11, 13px
- **Section padding:** 11-13px
- **Window padding:** 12-16px

## Radii

| Token | Value | Usage |
|-------|-------|-------|
| `--r-sm` | 3px | Buttons, timeline blocks |
| `--r-md` | 5px | Cards, sections |
| `--r-lg` | 8px | Modals, large panels |

## Temperature Variants

The One Bar widget background shifts subtly based on sitting urgency:

| State | Background | Effect |
|-------|-----------|--------|
| calm (<50%) | `var(--panel-base)` | Neutral |
| warm (50-80%) | `#12100c` | Slight warm shift |
| hot (80-100%) | `#140e0c` | Warmer |
| burning (>100%) | `#16100c` | Noticeably warm |
| standing | `#0e100c` | Slight green shift |
| reset | `#0e110c` | Green-shifted + glow |
| away | `var(--panel-base)` | Neutral |

## Window

- **Size:** 420x240px
- **Position:** Bottom-right corner (16px from right edge, 56px from bottom for taskbar)
- **Background:** `rgba(20, 18, 16, 0.84)` with `backdrop-filter: blur(12px)`
- **Decorations:** None (frameless)
- **Draggable:** Header area only

## Anti-patterns

- No sans-serif fonts (everything monospace)
- No cold grays (#ccc, #888) — use warm `--ink-*` tokens
- No shadows for depth — borders only
- No decorative elements — every element shows data
- No hardcoded colors in widget CSS — use design tokens
