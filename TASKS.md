# Desk App — Task Board

## Legend
- P0 = critical / blocking | P1 = high | P2 = medium | P3 = low | P4 = nice-to-have
- `[ ]` open | `[x]` done | `[-]` cancelled

---

## Sprint: Foundation (correctness before features)

> Critical gaps from CEO review: split truth, no calibration persistence, no daily reset.

- [ ] **T002** P0 — Config store: persist settings + calibration via `tauri-plugin-store` + **Rust owns today's totals** `.claude/tasks/0002-settings-store.md`
- [ ] **T008** P0 — (merged into T002) Calibration persistence — sitting/standing/thickness mm in AppConfig
- [ ] **T009** P1 — Daily reset: detect midnight rollover, reset in-memory counters `.claude/tasks/0009-daily-reset.md`
- [ ] **T006** P1 — Expose `standing_secs` from in-memory state (fix placeholder 0) `.claude/tasks/0006-standing-secs.md`

---

## Sprint: Settings Panel + Notifications

> Configurable thresholds and notification toggles.

- [ ] **T001** P1 — Settings panel UI: absorbs CalibrationWizard, sliders + toggles `.claude/tasks/0001-settings-panel.md`
- [ ] **T003** P2 — Notification preference toggles (3 types — position_changed removed) `.claude/tasks/0003-notification-prefs.md`
- [ ] **T004** P2 — Stand reminder: configurable "sit down after X min" limit `.claude/tasks/0004-stand-limit.md`

---

## Sprint: Session Stats

- [ ] **T005** P2 — Track `position_changes` counter in `SessionState` (Rust + UI) `.claude/tasks/0005-position-changes.md`

---

## Sprint: Polish + Delight

- [ ] **T010** P3 — Dynamic tray icon: green/amber/red by session state `.claude/tasks/0010-dynamic-tray-icon.md`
- [ ] **T011** P3 — Height-rail pulse animation on sit→stand transition `.claude/tasks/0011-rail-pulse.md`
- [ ] **T012** P3 — Yesterday delta arrow next to today's sitting time `.claude/tasks/0012-yesterday-delta.md`

---

## Deferred to BACKLOG

- T007 — `position_changes` in SQLite schema (deferred: in-memory counter sufficient until history view exists)

See `BACKLOG.md` for full list.
