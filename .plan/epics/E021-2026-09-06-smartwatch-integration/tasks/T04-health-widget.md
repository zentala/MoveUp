---
id: E021-T04
status: done
updated: 2026-09-08
evidence: 11211bd
---
# E021-T04: HealthWidget (rename StepsWidget) + useHealth hook
## Acceptance
Rename StepsWidget.tsx -> HealthWidget.tsx (+ test), src/hooks/useHealth.ts (Tauri: invoke get_health_today/refresh_health_now with existing useExponentialPoll ladder; remote: read health from reducer snapshot, no polling), HR badge when heart_rate_bpm present, source label from source_id, "Google Fit ends late 2026" tooltip when source_id == google_fit is the only source; deskReducer snapshot action carries health. Tests: HealthWidget.test.tsx, useHealth.test.ts, useRemoteDesk.test.ts extended. Verify: npx vitest run --config vite.config.ts src/components/HealthWidget.test.tsx src/hooks/useHealth.test.ts src/hooks/useRemoteDesk.test.ts.
