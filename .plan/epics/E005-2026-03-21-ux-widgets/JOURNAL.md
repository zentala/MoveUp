# E005 Journal — UX Communication + Widget System

## Summary

All 13 tasks completed 2026-03-21 to 2026-03-22. See archive/ORCHESTRATOR-ORIGINAL.md for full sprint log.

### Waves completed
- **Wave 1** (T027a + T027b): Split session.rs (1348L -> 4 files), db.rs, serial.rs, commands.rs, overlay_tests.rs. Added limit_used_secs + active_widget config.
- **Wave 2** (T023 + T024 + T029): Fixed tooltip standing (now updates every second), added notification test command + lowered thresholds, wrote floating window spec with failing tests.
- **Wave 3** (T022 + T028 + T025): Gold standing bar with lap flash, points system (+1/min standing, -0.5/min sitting, +5 per lap), welcome popup.
- **Wave 4** (T030 + T016): Fixed session timer (split sitting_seconds vs current_session_secs), transition banner, tray icon white desk + colored dot.
- **Wave 5** (T031 + T032 + T033): Widget architecture (WidgetProps, registry, PlaceholderWidget), One Bar widget (temperature escalation, coach, timeline), Timeline Zen widget (minimalist, big timeline).
