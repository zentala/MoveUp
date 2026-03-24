# Domain-Driven Design — Desk App

## Ubiquitous Language

| Term | Definition |
|------|-----------|
| **DeskState** | Current user state: Sitting, Standing, Walking (future), Away |
| **Sitting** | Desk low, user at keyboard (active) |
| **Standing** | Desk high (counts as break from sitting) |
| **Walking** | Future: desk high, user away (smartwatch needed) |
| **Away** | No keyboard/mouse activity ≥60s, regardless of desk height |
| **Session** | Continuous period in a single DeskState, saved to DB on exit |
| **Break credit** | Reduction in sitting session after standing: <5min=none, 5-9min=partial (-20min), ≥10min=full reset |
| **Position change** | 5+ continuous minutes Standing or Away = 1 change |
| **Lap** | One completed standing bout reaching standing_target_mins |
| **Debounce** | 5 consecutive stable readings required before state transition |
| **MetricEngine** | Stateless trait computing KPIs from SessionState snapshot |
| **Widget** | Swappable UI component implementing WidgetProps interface |
| **Alert escalation** | Progressive nudge stages when sitting limit reached: bar pulse → popup → snooze |
| **Snooze** | Temporary alert suppression after dismiss, with deescalating durations |
