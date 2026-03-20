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

> Core UX: app must actively nudge user to stand. Without this the app is passive.

### Session limit popup (P1 — do first)

- [ ] **T013** P1 — Session limit popup: red overlay "Take a break!" when sitting limit reached
  - Custom-drawn window (like overlay bar — native WinAPI, not Tauri webview)
  - Shows: "You've been sitting 45 min. Take a break!"
  - Dismiss button → snooze 2h (configurable)
  - Auto-dismiss when `DeskState::Standing` detected (desk raised)
  - Must not block other apps (non-modal, always-on-top)

- [ ] **T014** P1 — Bar flashing at limit: overlay bar pulses red when session limit reached
  - Reuse variant=2 (pulsing) or add new "alert" variant
  - Triggered by `progress >= 1.0` in Live mode

- [ ] **T015** P2 — Snooze logic: dismiss → configurable cooldown before next alert
  - Default: 2h. Setting in AppConfig.
  - After snooze expires: popup + flash again
  - "Your body will thank you for a break."

### Tray icon (P2)

- [ ] **T016** P2 — Tray icon color dot: small colored circle on tray icon
  - NOT full icon recolor — small dot/badge overlay
  - Green → Yellow → Red synced with `color_for_progress()`
  - Replaces current T010 (dynamic tray icon) with better design

### Notification system (P3 — prototype first)

- [ ] **T017** P3 — Demo: native Windows toast notifications (via `tauri-plugin-notification`)
  - Show 4 variants: green/yellow/red/gray
  - Evaluate: can we customize colors? icons? actions? persistence?

- [ ] **T018** P3 — Demo: custom-drawn overlay notification popup
  - Like overlay bar but bigger — custom WinAPI window with text + progress
  - 4 color variants, animated progress bar at bottom
  - Auto-dismiss timer (configurable), or persist until action
  - Compare with native toast → choose approach

- [ ] **T019** P4 — Success notifications (gamification)
  - "You stood 40min today — great job!" (green)
  - Streak tracking: "3 days in a row!"
  - Milestone celebrations
  - Future: more gamification mechanics through experimentation

---

## Sprint: Polish + Delight

- [-] **T010** ~~P3 — Dynamic tray icon~~ (replaced by T016 — color dot approach)
- [ ] **T011** P3 — Height-rail pulse animation on sit→stand transition `.claude/tasks/0011-rail-pulse.md`
- [ ] **T012** P3 — Yesterday delta arrow next to today's sitting time `.claude/tasks/0012-yesterday-delta.md`

---

## Deferred to BACKLOG

- T007 — `position_changes` in SQLite schema (deferred: in-memory counter sufficient until history view exists)

See `BACKLOG.md` for full list.
