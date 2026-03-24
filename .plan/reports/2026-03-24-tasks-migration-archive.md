# Desk App — Task Board (Migration Archive)

> **Archived 2026-03-24** — Original TASKS.md content preserved here during PM migration to `.plan/epics/`.

## Legend
- P0 = critical / blocking | P1 = high | P2 = medium | P3 = low | P4 = nice-to-have
- `[ ]` open | `[x]` done | `[-]` cancelled

---

## Sprint: Foundation (correctness before features)

> Critical gaps from CEO review: split truth, no calibration persistence, no daily reset.

- [x] **T002** P0 — Config store: persist settings + calibration via `tauri-plugin-store` + **Rust owns today's totals** `.claude/tasks/0002-settings-store.md`
- [x] **T008** P0 — (merged into T002) Calibration persistence — sitting/standing/thickness mm in AppConfig
- [x] **T009** P1 — Daily reset: detect midnight rollover, reset in-memory counters `.claude/tasks/0009-daily-reset.md`
- [x] **T006** P1 — Expose `standing_secs` from in-memory state (fix placeholder 0) `.claude/tasks/0006-standing-secs.md`

---

## Sprint: Settings Panel + Notifications

> Configurable thresholds and notification toggles.

- [x] **T001** P1 — Settings panel UI: absorbs CalibrationWizard, sliders + toggles `.claude/tasks/0001-settings-panel.md`
- [x] **T003** P2 — Notification preference toggles (3 types — position_changed removed) `.claude/tasks/0003-notification-prefs.md`
- [x] **T004** P2 — Stand reminder: configurable "sit down after X min" limit `.claude/tasks/0004-stand-limit.md`

---

## Sprint: Session Stats

- [x] **T005** P2 — Track `position_changes` counter in `SessionState` (Rust + UI) `.claude/tasks/0005-position-changes.md`

---

## Sprint: Overlay Progress Bar

> See `.claude/overlay/TASKS.md` for full overlay task list.

- [x] **T-OVR-010** P0 — Split `overlay_renderer.rs` (1060 lines, limit 250) → `overlay_opaque.rs`, `overlay_layered.rs`, `overlay_variants.rs`
- [x] **T-OVR-012** P1 — Precommit hook: fail build if any `.rs`/`.ts`/`.tsx` file > 250 lines
- [ ] **T-OVR-009** P2 — Choose production render mode (OPAQUE vs LAYERED)
- [ ] **T-OVR-011** P3 — Verify debug overlay info shows in popup

---

## Sprint: Foundation — Refactor (BEFORE alerts sprint)

> Sprint completed 2026-03-21. See `.claude/journals/2026-03-21-ux-communication-sprint.md` for the full orchestrator log.

- [x] **T027a** P0 — Split `session.rs` (1348 lines → 4 files ≤250) + add `standing_target_mins` / `stand_max_mins` to config `.claude/tasks/T027-split-session-rs.md`
  - **Blocks:** T027b, T022, T023, T024, T028
- [x] **T027b** P0 — Split db.rs, serial.rs, commands.rs, overlay_tests.rs (all >250L) + add `limit_used_secs` + `active_widget` `.claude/tasks/T027b-split-oversized-files.md`
  - **Depends on:** T027a
  - **Blocks:** T023, T024, T029 — pre-commit hook will reject commits on these files

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

- [x] **T013+T014** P1 — AlertManager + Stage 1 (bar pulse) + Stage 2 (popup) **BUNDLED**
- [x] **T015** P2 — Snooze with deescalating frequency + tone shift
- [ ] **T020** P2 — Integration test: full alert flow (sit→alert→dismiss→snooze→re-alert)
- [ ] **T021** P3 — Fix: dismiss snooze not triggered when sensor disconnected

- [x] **T022** P2 — Standing progress bar: gold bar fills 0→standing_target_mins, lap flash at 100%
- [x] **T028** P2 — Points system: +1/min standing, +5/session, −0.5/min sitting; score in tooltip + floating window
- [x] **T016** P2 — Tray icon redesign: white base icon (desk silhouette) + small colored dot (4-5px); dot = gold when standing
- [x] **T025** P2 — Welcome/onboarding popup on first launch

### Bugs (P1)

- [x] **T023** P1 — Fix tooltip while standing
- [x] **T024** P1 — Investigate + fix notifications
- [x] **T029** P1 — Floating window spec + tests
- [x] **T030** P1 — Floating window fixes

### Advanced Stages & Notifications (P3)

- [ ] **T017** P3 — Stages 3-5 implementation
- [ ] **T018** P3 — Notification A/B: both backends simultaneously with feature flag
- [ ] **T019** P4 — Success notifications + gamification

---

## Sprint: Widget System (floating window redesign)

- [x] **T031** P1 — Widget architecture
- [x] **T032** P2 — Widget "One Bar"
- [x] **T033** P3 — Widget "Timeline Zen"

---

## Sprint: Critical Bugs

- [ ] **T036** P0 — Investigate: standing not detected, sessions show continuous sitting
- [x] **T037** P1 — Height reading stabilization

---

## Sprint: UX Fixes

- [x] **T038** P1 — Popup closes on click outside
- [ ] **T039** P2 — Connection status UI
- [x] **T040** P2 — "No sessions yet" JSON field name mismatch fix

---

## Sprint: Documentation

- [x] **T041** P1 — Map full application UX flow

---

## Sprint: Session Timer — Tests & Persistence

- [x] **T042** P1 — Session timer unit tests
- [ ] **T043** P1 — Tooltip + UI integration tests
- [ ] **T044** P2 — Standing sessions persistence

---

## Sprint: Popup UX Redesign

- [x] **T045** P1 — Popup timer UX

---

## Sprint: E001 Verification (post-merge)

- [x] **T046** P0 — Run `cargo test` and fix Rust compilation/test failures
- [ ] **T047** P1 — Visual verification
- [ ] **T048** P1 — Fix any issues found during verification

---

## Sprint: Polish + Delight

- [-] **T010** ~~P3 — Dynamic tray icon~~ (replaced by T016)
- [ ] **T011** P3 — Height-rail pulse animation
- [ ] **T012** P3 — Yesterday delta arrow
- [ ] **T026** P3 — App icon design
- [x] **T034** P1 — Visual layout polish
- [x] **T035** P2 — Settings panel: tabbed layout

---

## Vision / Architecture (future)

- **T-ARCH-001** — Extended body positions

---

## Deferred to BACKLOG

- T007 — `position_changes` in SQLite schema
