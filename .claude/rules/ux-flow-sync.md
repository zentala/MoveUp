# UX-FLOW.md — Single Source of Truth

`.arch/UX-FLOW.md` is the canonical reference for what the user sees in every state.

## Mandatory updates

Update `.arch/UX-FLOW.md` when ANY of the following change:
- State machine transitions (new states, changed conditions)
- What any UI element shows per state (popup, overlay, tray, timeline)
- Alert escalation behavior
- Points/scoring rules
- KPI definitions, thresholds, or display format
- Color semantics
- Configuration knobs

## When to update

- **Planning**: when a plan adds/changes UX behavior, update UX-FLOW first (before code)
- **Implementation**: after code lands, verify UX-FLOW matches reality
- **`done.` flow**: check if UX-FLOW needs updating as part of session close

## Goal: self-descriptive UI

Every visible element in the popup must be understandable without documentation:
- KPI badges: tooltip on hover explaining what the metric means
- State labels: clear, unambiguous names
- Progress bars: obvious what they measure
- Colors: consistent semantic meaning (green=ok, yellow=warn, red=alert, gold=standing)

If the developer (zentala) can't understand a UI element at a glance, it's a bug.
