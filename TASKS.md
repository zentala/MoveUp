# Desk App — Task Board

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
  - **CEO review:** Ship together — bar pulse alone too subtle.
  - **Depends on:** T-OVR-010 (split overlay_renderer.rs)
  - **New files:**
    - `alert_manager.rs` (~150 lines) — `AlertStage` enum, `AlertAction` enum, `AlertManager` struct
    - `alert_popup.rs` (~200 lines) — WinAPI popup window, own thread
  - **Modified files:**
    - `overlay_renderer.rs` — add `set_variant(&self, variant: u8)` method
    - `tray_controller.rs` — wire AlertManager tick + execute AlertActions
    - `lib.rs` + `commands.rs` — add AlertManager + AlertPopup to AppState
  - **Eng review decisions (2026-03-20):**
    - Bar pulse: `set_variant(2)` method on OverlayRenderer (explicit, same pattern as update/show/hide)
    - Popup thread: own thread, spawn on show, `AtomicBool` dismiss flag, join on dismiss
    - Time source: `Instant` (monotonic), `stage_entered_at` field
    - Dismiss returns to Snoozed (T015 fills in full snooze logic; T013 implements stub: instant re-entry to Stage1)
    - Safety: if progress drops below 1.0, also dismiss popup (missed event mitigation)
  - **AlertManager API:**
    ```rust
    tick(progress: f32) → Vec<AlertAction>
    on_standing() → Vec<AlertAction>   // reset + StopPulse + DismissPopup
    dismiss()                          // → Snoozed (stub in T013, filled by T015)
    ```
  - **AlertPopup API:**
    ```rust
    show(msg: String)    // spawns WinAPI thread
    dismiss()            // sets AtomicBool, thread exits
    is_visible() → bool
    ```
  - **Stage 1 (at limit):** `overlay.set_variant(2)` — bar starts pulsing
  - **Stage 2 (+2min):** popup at right-bottom (notification area):
    ```
                                    ┌────────────────────────┐
                                    │  You've been sitting   │
                                    │  for 45 minutes.       │
                                    │  Take a 5-min break!   │
                                    │  [Dismiss]  [Stand up] │
                                    └────────────────────────┘
                                    ↑ right-bottom, near tray
    ```
    - ~400x200px, always-on-top, non-modal, arrow cursor
    - Auto-dismiss on `DeskState::Standing` OR `progress < 1.0`
  - **Tests (~15):**
    - AlertManager: idle→stage1→stage2 transitions, timing, on_standing reset
    - dismiss() returns to Idle, progress oscillation debounce, rapid sit/stand
    - set_variant() updates state, popup show/dismiss lifecycle

### Snooze & Escalation (P2)

- [x] **T015** P2 — Snooze with deescalating frequency + tone shift
  - **CEO review decisions (2026-03-20):**
  - **Deescalating cooldown** — more dismisses = longer intervals (app backs off):
    ```
    snooze_durations: [5, 15, 30, 60]  // minutes
    Dismiss #1 → 5 min   ("just checking")
    Dismiss #2 → 15 min  ("ok, later")
    Dismiss #3 → 30 min  ("I hear you")
    Dismiss #4+ → 60 min ("last reminder")
    Standing → reset snooze_index to 0
    ```
  - **Tone shift at dismiss #3** — if nagging doesn't work, stop nagging. Inspire instead:
    - Dismiss 1-2: neutral ("Time for a stretch!", "Your body needs a break")
    - Dismiss 3+: positive ("Even 2 min standing helps blood flow", "Quick stand = fresh mind")
    - Messages are configurable array, not hardcoded
  - **Bar during snooze:**
    ```
    UNDER LIMIT:     green bar, growing
    AT LIMIT:        red bar, PULSING
    SNOOZED:         red bar, SOLID (passive reminder — "I heard your dismiss")
    SNOOZE EXPIRED:  red bar, PULSING again
    STANDING:        bar hidden, everything resets
    ```
  - **Implementation:** extend `AlertManager` with `snooze_index: usize`, `snoozed_until: Option<Instant>`, `snooze_durations: Vec<Duration>`, message arrays
  - **Eng review decisions (2026-03-20):**
    - T013 must implement `dismiss()` → `Snoozed` from the start (even as stub); T015 fills in logic
    - Message selection at Stage2 ENTRY via `fn popup_message(&self) -> &str`; threshold: `snooze_index >= 3` → positive
    - `progress < 1.0` while Snoozed → cancel snooze → Idle (problem resolved)
    - Message arrays in `AlertConfig` with defaults (settings panel exposure deferred to BACKLOG)
  - **Tests (~8):**
    - Dismiss #1 → 5min snooze, neutral message on next popup
    - Dismiss #2 → 15min snooze, neutral message
    - Dismiss #3 → 30min snooze, **positive** message on next popup
    - Dismiss #4+ → 60min snooze (capped)
    - Standing while Snoozed → Idle, snooze_index=0
    - `progress < 1.0` while Snoozed → cancel snooze → Idle
    - Bar variant: solid red during Snoozed, pulsing after expiry (Stage1)
    - Snooze expiry → Stage1 (not Stage2 directly)

- [ ] **T020** P2 — Integration test: full alert flow (sit→alert→dismiss→snooze→re-alert) `.claude/tasks/T020-integration-test-alert-flow.md`
- [ ] **T021** P3 — Fix: dismiss snooze not triggered when sensor disconnected `.claude/tasks/T021-dismiss-without-sensor.md`

- [x] **T022** P2 — Standing progress bar: gold bar fills 0→standing_target_mins, lap flash at 100% `.claude/tasks/T022-standing-progress-bar.md`
- [x] **T028** P2 — Points system: +1/min standing, +5/session, −0.5/min sitting; score in tooltip + floating window `.claude/tasks/T028-points-system.md`
- [x] **T016** P2 — Tray icon redesign: white base icon (desk silhouette) + small colored dot (4-5px); dot = gold when standing `.claude/tasks/T016-tray-icon-redesign.md`
- [x] **T025** P2 — Welcome/onboarding popup on first launch: friendly intro, draggable, "don't show again" `.claude/tasks/T025-welcome-popup.md`

### Bugs (P1)

- [x] **T023** P1 — Fix tooltip while standing: frozen timer + shows sitting_secs instead of standing_secs `.claude/tasks/T023-fix-tooltip-standing.md`
- [x] **T024** P1 — Investigate + fix notifications: user never sees them; debug trigger, lower thresholds, feature flag for backend `.claude/tasks/T024-notifications-debug-and-fix.md`
- [x] **T029** P1 — Floating window spec + tests: document expected behavior, write failing tests, extend StateChangedPayload `.claude/tasks/T029-floating-window-spec-and-tests.md`
  - **Confirmed bugs:** sitting timer shows wrong value (shows 50 when sat 5 min ago), no standing duration shown after transition
  - **Scope:** SPEC + TESTS ONLY — fixes go in T030
  - **Depends on:** T027 (session.rs split — tests go into session_tests.rs)
- [x] **T030** P1 — Floating window fixes: split sitting_seconds/current_session_secs, TransitionBanner, standing timer `.claude/tasks/T030-floating-window-fix.md`
  - **Fixes:** sitting timer wrong value, no "Stood for X min" after transition, frozen standing timer
  - **Depends on:** T029 (StateChangedPayload extended + root causes confirmed)

### Advanced Stages & Notifications (P3)

- [ ] **T017** P3 — Stages 3-5 implementation
  - Stage 3 (+5min): popup grows, can't dismiss for 5 seconds
  - Stage 4 (+10min): overlay bar expands (taller)
  - Stage 5 (+15min): full-screen overlay nudge
  - Each stage configurable (enable/disable in settings)

- [ ] **T018** P3 — Notification A/B: both backends simultaneously with feature flag
  - `NOTIFICATION_BACKEND=toast|popup|both`
  - Toast: native Windows (`tauri-plugin-notification`)
  - Popup: custom WinAPI (same as alert popup)
  - Feature flag for dev comparison; long-term: custom popup
  - Depends on T024 (notifications working at all)

- [ ] **T019** P4 — Success notifications + gamification
  - Standing triggers success flash / brief green animation
  - "You stood 40min today — great job!" notifications
  - Streak tracking: "3 days in a row!"
  - Milestone celebrations
  - Future: adaptive nudge intensity based on user patterns

---

## Sprint: Widget System (floating window redesign)

> **Philosophy:** Core app provides data; widgets handle presentation.
> User can switch between widget styles. Architecture supports experimentation.
> See `.agent/vision/2026-03-21-product-vision-coach-and-business.md` for product context.

- [x] **T031** P1 — Widget architecture: WidgetProps interface + registry + `active_widget` in AppConfig `.claude/tasks/T031-widget-architecture.md`
  - **Blocks:** T032, T033 — widgets need the architecture first
  - **Depends on:** T027 (session.rs split), T030 (floating window fixes)
- [x] **T032** P2 — Widget "One Bar": horizontal, unified progress bar, temperature escalation, coach sentence `.claude/tasks/T032-widget-one-bar.md`
  - **Depends on:** T031
- [x] **T033** P3 — Widget "Timeline Zen": minimalist, big timeline, zero text, ultra compact `.claude/tasks/T033-widget-timeline-zen.md`
  - **Depends on:** T031

---

## Sprint: Polish + Delight

- [-] **T010** ~~P3 — Dynamic tray icon~~ (replaced by T016 — color dot approach)
- [ ] **T011** P3 — Height-rail pulse animation on sit→stand transition `.claude/tasks/0011-rail-pulse.md`
- [ ] **T012** P3 — Yesterday delta arrow next to today's sitting time `.claude/tasks/0012-yesterday-delta.md`
- [ ] **T026** P3 — App icon design: desk silhouette SVG → PNG/ICO assets `.claude/tasks/T026-app-icon-design.md`
- [x] **T034** P1 — Visual layout polish: design system alignment + layout fill `.claude/tasks/T034-visual-layout-polish.md`
  - Fixed by /design-review on main, 2026-03-22. 7 findings, all fixed.
  - Remaining: visual verification checklist items (deferred — need manual testing)

---

## Vision / Architecture (future)

- **T-ARCH-001** — Extended body positions: kneeling / leaning / sitting / standing
  - Same sensor, richer height-to-position mapping
  - `DeskPosition` enum extensible, per-position session stats + points
  - Max standing session: 90 min (configurable, option: 60 min) — symmetric to sit limit
  - See `.agent/vision/2026-03-20-ux-communication-vision.md`

---

## Deferred to BACKLOG

- T007 — `position_changes` in SQLite schema (deferred: in-memory counter sufficient until history view exists)

See `BACKLOG.md` for full list.
