# Investigation: Away Detection Not Working When Standing

**Date:** 2026-03-25
**Reporter:** zentala
**Status:** OPEN — root cause not found in code, diagnostic logging added

---

## Symptom 1: Standing + inactive ≠ Away

**What happened:** User was standing at desk, walked away (no keyboard/mouse activity),
app stayed in Standing state — did NOT transition to Away. When sitting, Away detection
works correctly.

**Yesterday:** Opposite behavior — didn't count when sitting, counted when standing.
This flip-flopping suggests something environmental or timing-related, not a static code bug.

## Symptom 2: 8 hours of "Sitting" counted overnight (computer asleep)

**What happened:** User put computer to sleep at night. Next morning, app shows ~8 hours
of sitting time accumulated. The user was NOT at the computer at all — no keyboard/mouse
activity was possible because the machine was suspended.

**Hypothesis:** When Windows goes to sleep, `GetLastInputInfo` stops updating (no tick count
advancement during sleep). On wake, `GetTickCount()` jumps forward but `dwTime` from
`GetLastInputInfo` stays at the pre-sleep value. The elapsed idle time would be HUGE
(hours), so `is_active()` should return false. BUT:
- If the serial reader thread was also suspended during sleep, no `on_reading()` calls
  happened during sleep → no state transition could occur.
- On wake, the first reading should see idle >> 60s → candidate = Away.
- BUT if the session counters use wall-clock timestamps (like `sitting_started`), the
  elapsed time calculated on wake would include the entire sleep duration, inflating
  `sitting_seconds` before the Away transition fires.

**Root cause candidate:** `sitting_started` is set as a timestamp. When `handle_state_exit`
runs on wake (Sitting→Away), it calculates `elapsed = now - sitting_started`. If the user
was sitting before sleep, this elapsed time includes the ENTIRE sleep duration (8 hours),
which gets added to `sitting_seconds`.

**Fix needed:** On wake from sleep, detect the gap and either:
1. Cap elapsed time to time-since-last-reading (not time-since-sitting-started)
2. Detect sleep/wake events and force a state transition to Away

---

## Investigation: State Machine Logic (Standing→Away)

### What was checked

1. **`session_reading.rs:38-44`** — Candidate logic:
   ```rust
   let candidate = if !active { DeskState::Away }
                   else if desk_height_cm <= mid_cm { DeskState::Sitting }
                   else { DeskState::Standing };
   ```
   `!active` → Away regardless of desk height. **CORRECT.**

2. **`session_breaks.rs:47-76`** — `handle_state_exit` for Standing→Away:
   - Standing bout committed, break_started preserved, break continues.
   **CORRECT.**

3. **`activity.rs`** — `is_active()` uses `GetLastInputInfo` with 60s threshold.
   **CORRECT** (but untestable in unit tests — Windows API).

4. **`serial_periodic.rs:84`** — `is_active()` called unconditionally before `on_reading()`.
   **CORRECT.**

5. **`overlay_renderer.rs` + `overlay_opaque.rs`** — Checked if overlay generates
   hardware input (SetCursorPos, SendInput, etc.) that could reset idle timer.
   **NONE FOUND.** Window uses WS_EX_NOACTIVATE, GDI rendering only.

6. **`tray_controller.rs:143-161`** — Standing mode overlay path: early return after
   showing gold bar. Does NOT affect state machine.

7. **Debounce logic** — DEBOUNCE_COUNT=5, reset on candidate change. **CORRECT.**

8. **Git history** — `git diff c305f08..HEAD` on state machine files: only
   HourlyBreakTracker addition and NotificationService refactor. Neither affects
   Away detection.

### Tests checked/added

- `high_desk_inactive_produces_away` — passes ✓
- `standing_away_standing_sitting_only_counts_standing` — passes ✓
- **NEW:** `standing_to_away_on_inactivity` — full end-to-end: Sitting→Standing→Away
  with state_change event verification, break_started preservation, and break credit.
  **PASSES** ✓
- Total: 340/340 tests pass

### What was NOT found

No code-level bug that would cause different behavior between Sitting+inactive and
Standing+inactive. The logic is identical: `!active → Away`.

### Diagnostic logging added

In `serial_periodic.rs::handle_reading()`:
- Every state transition now logs `idle=XXs` in event log
- When Standing and idle > 30s, logs `Standing idle diagnostic` at debug level

### How to reproduce (next occurrence)

Run with: `RUST_LOG=desk_lib=debug pnpm tauri:dev`

Check event log (`AppData/Roaming/io.zntl.desk/logs/YYYY-MM-DD/events.log`):
- If `STATE Standing→Away idle=XXs` is MISSING: `is_active()` never returned false
- If `Standing idle diagnostic: idle=XXs active=true` with idle>60: Windows API bug

---

## Open Questions

1. Does Windows sleep/wake corrupt `GetLastInputInfo` timing?
2. Does something on this machine reset the idle timer during Standing overlay?
3. Should we detect sleep/wake events via Windows API (`WM_POWERBROADCAST`) and
   force-transition to Away?
4. Should `accumulate_ongoing` cap elapsed time to prevent sleep-inflated counters?
