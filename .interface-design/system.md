# Desk App — Interface Design System

## Direction
**Precision instrument panel.** Oiled-steel surfaces, amber active indicators, warm
readout text. Feels like bespoke hardware software — a cockpit for your desk, not a
SaaS dashboard. Solo dev glancing at this between coding sessions: answer "am I ok?"
in under one second.

## Depth Strategy
**Borders-only.** No shadows. Technical, precise. Everything reads as a flat panel with
quiet structural lines. Mixed depth strategies are not allowed.

## Spacing
4px base grid. Panel padding: 11px 13px. Row gaps: 3px between panels, 7px within.

## Signature Element
**HeightRail** — a 3px vertical amber rail on the left edge of the main state card.
An amber dot slides to show current desk height within its physical range (60–130 cm).
A faint tick at 30% marks the sitting threshold. Makes the hardware tangible in the UI.

---

## Tokens

### Surfaces
| Token | Value | Use |
|-------|-------|-----|
| `--panel-base` | `#141210` | page / window background |
| `--panel-raised` | `#1c1a17` | default card surface |
| `--panel-elevated` | `#242118` | dropdowns, popovers |
| `--panel-overlay` | `#2c2920` | hover state backgrounds |

### Borders
| Token | Value | Use |
|-------|-------|-----|
| `--rail-subtle` | `rgba(255,215,140,0.05)` | card borders, track backgrounds |
| `--rail-default` | `rgba(255,215,140,0.10)` | button borders, dividers |
| `--rail-emphasis` | `rgba(255,215,140,0.22)` | hover, focus |
| `--rail-active` | `rgba(255,215,140,0.45)` | focused inputs, active rings |

### Text
| Token | Value | Use |
|-------|-------|-----|
| `--ink-primary` | `#f0ebe0` | default text |
| `--ink-secondary` | `#b0a898` | supporting / button labels |
| `--ink-tertiary` | `#706858` | metadata, labels, headings |
| `--ink-muted` | `#403830` | disabled / placeholder |

### Amber beam
| Token | Value | Use |
|-------|-------|-----|
| `--beam` | `#d97706` | primary accent, height readout, rail indicator |
| `--beam-hi` | `#f59e0b` | hover / active accent |
| `--beam-glow` | `rgba(217,119,6,0.20)` | glow for standing/walking dot |
| `--beam-dim` | `rgba(217,119,6,0.08)` | subtle tint backgrounds |

### Semantic signals
| Token | Value | Use |
|-------|-------|-----|
| `--signal-ok` | `#65a30d` | Sitting state dot (healthy lime) |
| `--signal-warn` | `#c2762d` | approaching limit |
| `--signal-alert` | `#b91c1c` | limit exceeded / error |
| `--signal-up` | `#d97706` | Standing / Walking state dot |
| `--signal-away` | `#4a7c9e` | Away state dot |

---

## Typography
Full monospace throughout — `ui-monospace, "Cascadia Code", "Fira Mono", "Consolas"`.
This is an instrument panel, not a content app. Monospace is the right call.

| Role | Size | Weight | Notes |
|------|------|--------|-------|
| Header label | 10px | 600 | uppercase, 0.14em tracking |
| State label | 17px | 700 | -0.02em tracking |
| Elapsed time | 22px | 700 | -0.03em tracking, `--ink-primary` |
| Height readout | 12px | 500 | `--beam`, 0.04em tracking |
| Today stat value | 14px | 600 | -0.01em tracking |
| Today stat label | 10px | 400 | uppercase, 0.08em tracking, `--ink-tertiary` |
| Metadata / remaining | 11px | 400 | `--ink-tertiary` |

---

## Component Patterns

### Panel row
`.panel-row` — `padding: 11px 13px`, `border: 1px solid --rail-subtle`,
`background: --panel-raised`, `border-radius: --r-md (5px)`.
Rows separated by 3px gap (not margin — gap on `.app`).

### HeightRail
`width: 3px`, amber `--panel-raised`-based background, indicator dot 7×7px with
`box-shadow: 0 0 7px --beam-glow`. Tick at `bottom: 30%`.
Transition: `bottom 0.9s cubic-bezier(0.25, 0.46, 0.45, 0.94)`.

### State dot
6×6px circle. Glowing variants for Sitting (`--signal-ok`) and Standing/Walking
(`--beam`). Away uses `--signal-away` without glow.

### Progress bar
3px height, `--rail-subtle` track. Fill classes: `--ok / --warn / --alert`.
No border-radius override — inherits 2px.

### Buttons
Transparent background, `--rail-default` border, `--ink-secondary` text.
Hover: `--panel-overlay` bg, `--rail-emphasis` border, `--ink-primary` text.
Font: 11px monospace, 0.04em tracking. Radius: `--r-sm (3px)`.

---

## Radii
| Token | Value | Use |
|-------|-------|-----|
| `--r-sm` | 3px | buttons, inputs |
| `--r-md` | 5px | panel rows, cards |
| `--r-lg` | 8px | modals, large overlays |
