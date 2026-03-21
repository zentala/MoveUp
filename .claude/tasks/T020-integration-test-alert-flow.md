# T020 — Integration Test: Full Alert Flow (sit → alert → dismiss → snooze → re-alert)

**Status:** Open
**Priority:** P2
**Depends on:** T013+T014, T015 (both done)
**File:** `tests/integration/alert_flow.test.ts`

---

## What This Tests

The unit tests in `alert_manager_tests.rs` and `alert_snooze_tests.rs` cover the **state machine in isolation**. This task adds an integration test that drives the **full IPC chain**:

```
sensor reading (inject_reading)
  → serial.rs parses distance
  → session.rs updates sitting_secs + progress
  → tray_controller.rs calls alert_manager.tick()
  → AlertAction returned
  → overlay.set_variant() / alert_popup.show()
  → IPC state queryable via get_session_state / get_overlay_state
```

This is the only test layer that validates the whole pipeline end-to-end.

---

## Context You Need

### How inject_reading works
The app exposes a debug-only Tauri command:
```ts
await invoke('inject_reading', { distanceMm: 750 });  // SITTING_DISTANCE_MM
await invoke('inject_reading', { distanceMm: 1080 }); // STANDING_DISTANCE_MM
```
Each call simulates one sensor reading. Debounce requires 5 consecutive readings at the same distance before state changes (see `DEBOUNCE_COUNT = 5` in `tests/emulator/scenarios.ts`).

### How to query state
```ts
const state = await invoke('get_session_state');
// state.sitting_secs, state.standing_secs, state.desk_state
const overlay = await invoke('get_overlay_state'); // debug builds only
// overlay.data_source, overlay.progress, overlay.variant
```

### Constants (from `tests/emulator/scenarios.ts`)
```ts
const SITTING_DISTANCE_MM = 750;
const STANDING_DISTANCE_MM = 1080;
const DEBOUNCE_COUNT = 5;
```

### AlertManager thresholds (from `src-tauri/src/alert_config.rs`)
- `stage1_threshold`: 1.0 (100% of session limit)
- `stage2_delay_secs`: 120 (2 minutes after Stage1)
- Session limit default: 40 minutes (2400 seconds)

### How to simulate time passing
You can't inject time directly. Two options:
1. **Config override**: The test app should start with a test config where `session_limit_secs = 60` (instead of 2400) and `stage2_delay_secs = 2`. Check if there's a `--test-config` flag or env var; if not, add one (see `AlertConfig::default()` in `alert_config.rs`).
2. **inject_sitting_secs**: Add a new debug Tauri command `inject_sitting_secs(secs: u64)` that directly sets `session.sitting_secs` — simpler for testing. Preferred approach.

You'll likely need to add `inject_sitting_secs` as a Tauri command. Add it to `src-tauri/src/commands.rs` behind `#[cfg(debug_assertions)]`.

---

## Test Scenarios to Implement

### Test 1: Idle → Stage1 (bar pulses at session limit)
```
Given: sitting_secs = 0, stage = Idle
When: inject sitting_secs = session_limit (progress = 1.0)
Then: overlay.variant == 2 (pulsing)
```

### Test 2: Stage1 → Stage2 (popup appears after delay)
```
Given: stage = Stage1 (bar pulsing)
When: wait stage2_delay_secs (or inject time past threshold)
Then: alert_popup.is_visible() == true
      overlay.variant == 2 (still pulsing)
```

### Test 3: Standing resets everything
```
Given: stage = Stage2 (popup visible, bar pulsing)
When: inject 5× STANDING readings
Then: overlay.variant == 0 (solid)
      alert_popup.is_visible() == false
      alert_manager.stage == Idle
```

### Test 4: Dismiss → Snooze → Re-alert
```
Given: stage = Stage2 (popup visible)
When: simulate user clicking Dismiss (set user_dismissed flag)
      wait 1 tick (desk:distance fires)
Then: alert_manager.stage == Snoozed
      alert_popup.is_visible() == false
      overlay.variant == 0 (solid, not pulsing)
When: wait snooze duration (1st snooze = 5min, use short test config)
Then: alert_manager.stage == Stage1
      overlay.variant == 2 (pulsing again)
```

### Test 5: Progress drop cancels alert
```
Given: stage = Stage2 (popup visible)
When: inject 5× STANDING readings (progress drops below 1.0)
Then: stage == Idle, popup dismissed, bar solid
```

### Test 6: Snooze index increments correctly
```
Given: fresh session at limit
When: dismiss #1, wait snooze, popup re-appears
      dismiss #2, wait snooze, popup re-appears
Then: snooze_index == 2
      2nd popup shows neutral message
When: dismiss #3
Then: 3rd popup shows positive message (tone_shift_threshold = 3)
```

---

## How to Trigger Dismiss in Tests

`alert_popup_window.rs` runs in a thread. From the test (TypeScript), you can't click a button directly. Options:

1. **Add `trigger_popup_dismiss` debug command** in `commands.rs`:
   ```rust
   #[cfg(debug_assertions)]
   #[tauri::command]
   pub fn trigger_popup_dismiss(state: tauri::State<AppState>) {
       let mut popup = state.alert_popup.lock().unwrap();
       popup.dismiss();
       // Also set user_dismissed flag so snooze triggers
       // Need to expose a method: popup.simulate_user_dismiss()
   }
   ```
   `simulate_user_dismiss()` sets both `visible = false` AND `user_dismissed = true`.

2. **Add `simulate_user_dismiss()` to AlertPopup** in `alert_popup.rs`.

Approach 1 is recommended — keeps test API clean.

---

## Setup

The app must be running: `pnpm tauri:dev` in another terminal.
Run tests: `pnpm test:integration`

See `tests/integration/` for existing integration test examples.

---

## Acceptance Criteria

- [ ] All 6 scenarios implemented and passing
- [ ] `pnpm test:integration` passes with the app running
- [ ] `trigger_popup_dismiss` command added (debug only)
- [ ] `inject_sitting_secs` command added (debug only)
- [ ] Tests use short test config (session_limit=60s, stage2_delay=2s) — no 40-minute waits
