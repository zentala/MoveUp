---
updated: 2026-03-24T12:00:00Z
active_epic: none
active_epic_path: null
current_wave: null
---

## Status
- E000 (maintenance) — open (permanent)
- E001 (Timer UX + KPI Dashboard + MetricEngine) — **DONE** (v0.1.0)
- E002 (UX Integrity) — **DONE** (v0.2.0, 13/13 tasks, 508 tests)

## Version
- Current: `v0.2.0` (tagged)
- Rule: each new epic bumps `0.MAJOR.0`

## Test Totals
- Rust: 339 tests
- TypeScript: 169 tests
- Total: 508

## What E002 Shipped
- Away state: inactive → Away regardless of desk height
- Live current session in timeline
- KPI labels with Unicode icons + "Today" header
- Unified color palette (Rust + TS)
- 2-layer gold visual for standing laps
- Popup redesign: timer first, state in header
- NotificationService: centralized 6 toasts, wired `notification_backend` config
- Process guard restored (`tauri-dev.sh`)
- UX-FLOW.md fully synced

## Next Steps
1. Pick next epic from BACKLOG.md or define E003
2. Candidates: SQLite time-series, timeline full window, notification A/B testing
3. Or: run app live, dogfood E002 changes, collect feedback
