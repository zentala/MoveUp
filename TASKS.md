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

## Sprint: Overlay Progress Bar

> See `.claude/overlay/TASKS.md` for full overlay task list.

- [ ] **T-OVR-010** P0 — Split `overlay_renderer.rs` (1060 lines, limit 250) → `overlay_opaque.rs`, `overlay_layered.rs`, `overlay_variants.rs`
- [ ] **T-OVR-012** P1 — Precommit hook: fail build if any `.rs`/`.ts`/`.tsx` file > 250 lines
- [ ] **T-OVR-009** P2 — Choose production render mode (OPAQUE vs LAYERED)
- [ ] **T-OVR-011** P3 — Verify debug overlay info shows in popup

---

## Sprint: Session Alerts & Notifications

> Core UX: app must actively nudge user to stand. Progressive escalation — gentle → firm.
> Architecture: `alert_manager.rs` owns escalation state machine, returns `Vec<AlertAction>`.
> `tray_controller.rs` executes actions (show popup, pulse bar, update tray).

### Architecture: Progressive Escalation

```
IDLE ──(progress≥1.0)──→ STAGE 1 (bar pulses red)
                              │ +2 min
                              ▼
                         STAGE 2 (popup: "Take a break!")
                              │ +5 min
                              ▼
                         STAGE 3 (popup more prominent)
                              │ +10 min
                              ▼
                         STAGE 4 (overlay expands)
                              │ +15 min
                              ▼
                         STAGE 5 (full-screen nudge)

ANY STAGE ──standing──→ IDLE (+ success animation)
ANY STAGE ──dismiss──→ SNOOZED ──(cooldown expires)──→ STAGE 1
```

### Foundation (P1 — do first)

- [ ] **T013** P1 — AlertManager module + Stage 1 (bar pulse at limit)
  - New `alert_manager.rs`: `AlertStage` enum, `AlertAction` enum, `AlertManager` struct
  - `tick(progress, sitting_secs)` → `Vec<AlertAction>` (pure logic, no WinAPI)
  - Stage 1: bar pulses red when `progress >= 1.0`
  - `on_standing()` → reset to IDLE
  - Wire into `tray_controller.rs` via `desk:distance` listener
  - Unit tests for state transitions, stage timing, reset on stand

- [ ] **T014** P1 — Stage 2: popup window ("Take a break!")
  - Native WinAPI window (like overlay bar — proven pattern)
  - Shows at `limit + 2min`: "You've been sitting 47 min. Take a 5-min break!"
  - Dismiss button → triggers `AlertManager::dismiss()`
  - Auto-dismiss when `DeskState::Standing` detected
  - Non-modal, always-on-top, arrow cursor

### Snooze & Escalation (P2)

- [ ] **T015** P2 — Snooze + re-escalation logic
  - Dismiss → `SNOOZED` state with configurable cooldown (default 2h, in AppConfig)
  - After cooldown expires → restart from Stage 1
  - "Your body will thank you for a break."
  - Tests: snooze timing, re-escalation, configurable duration

- [ ] **T016** P2 — Tray icon color dot
  - Small colored circle overlay on tray icon (not full recolor)
  - Green → Yellow → Red synced with `color_for_progress()`
  - Independent of alert system — always shows progress color

### Advanced Stages & Notifications (P3)

- [ ] **T017** P3 — Stages 3-5 implementation
  - Stage 3 (+5min): popup grows, can't dismiss for 5 seconds
  - Stage 4 (+10min): overlay bar expands (taller)
  - Stage 5 (+15min): full-screen overlay nudge
  - Each stage configurable (enable/disable in settings)

- [ ] **T018** P3 — Notification backend comparison
  - Demo A: native Windows toast (`tauri-plugin-notification`) — 4 color variants
  - Demo B: custom WinAPI popup — animated progress, auto-dismiss timer
  - Compare and choose approach for production

- [ ] **T019** P4 — Success notifications + gamification
  - Standing triggers success flash / brief green animation
  - "You stood 40min today — great job!" notifications
  - Streak tracking: "3 days in a row!"
  - Milestone celebrations
  - Future: adaptive nudge intensity based on user patterns

---

## Sprint: Polish + Delight

- [-] **T010** ~~P3 — Dynamic tray icon~~ (replaced by T016 — color dot approach)
- [ ] **T011** P3 — Height-rail pulse animation on sit→stand transition `.claude/tasks/0011-rail-pulse.md`
- [ ] **T012** P3 — Yesterday delta arrow next to today's sitting time `.claude/tasks/0012-yesterday-delta.md`

---

## Deferred to BACKLOG

- T007 — `position_changes` in SQLite schema (deferred: in-memory counter sufficient until history view exists)

See `BACKLOG.md` for full list.
