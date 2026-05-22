---
id: E012-T11
epic: E012
status: pending
created: 2026-05-21
branch: feat/E012-T11-selected-day-pulse
title: Pulse highlight on KPI panels when selectedDay changes
---

# E012-T11: Pulse highlight on KPI panels when selectedDay changes

## Context

User wants the KPI cards on the Analyst dashboard to **briefly pulse
(highlight)** when `selectedDay` changes — a subtle "I just refreshed
to show you a different day" signal that ties the bottom day-navigator
visually to the donuts above.

Depends on T10 (KpiDonut component must exist) and T09 (selectedDay nav
on top + bottom). This task only adds the animation layer.

## Out of scope

- New chart types.
- New layouts.
- Pulsing anything other than KPI donut cards.

## Decisions

- **CSS-only animation, no library.** A `@keyframes` rule with two
  brief opacity + box-shadow pulses (~350ms total). Avoid adding
  `framer-motion` (≈30 KB) for a one-off effect.
- **Triggered via a `data-pulse-key={selectedDay}` attribute** on the
  donut card root. Using React's key remount would discard arc
  animation state (Recharts animates from 0); pulsing via CSS
  attribute change keeps the donut state intact while replaying the
  pulse animation.
- **Animation respects `prefers-reduced-motion`** — disable the
  keyframes for users who opt out.

## Scope (concrete)

- Update `apps/desk/src/analyst/charts/KpiDonut.tsx`:
  - Add a `pulseKey` prop. When it changes, the wrapper re-applies the
    `kpi-donut-pulse` CSS class for one cycle (use `useEffect` to
    add/remove on key change, or use `key={pulseKey}` on a transparent
    overlay div).
- New file `apps/desk/src/analyst/charts/kpi-donut.css`:
  - `@keyframes kpi-donut-pulse` — 0% scale(1) outline transparent →
    50% scale(1.02) outline `chartColors.primary` 40% → 100% back.
  - `.kpi-donut-pulse { animation: kpi-donut-pulse 350ms ease-out; }`
  - `@media (prefers-reduced-motion: reduce)` overrides
    `.kpi-donut-pulse { animation: none; }`.
- `KpiDonutPanel` passes `pulseKey={selectedDay}` to each child.

## Acceptance criteria

- [ ] When `selectedDay` changes, every KPI donut pulses once.
- [ ] No layout shift during the pulse (use `outline` not `border`).
- [ ] Pulse skipped under `prefers-reduced-motion`.
- [ ] No new runtime deps.
- [ ] Tests: `KpiDonut` updates `data-pulse-key` when prop changes;
  CSS class added then removed after animation duration.
- [ ] All files ≤ 250 LoC.

## Cross-references

- Depends on T10: `tasks/E012-T10-recharts-donut-kpis.md`
- Depends on T09: `tasks/E012-T09-analyst-layout-flip.md`
