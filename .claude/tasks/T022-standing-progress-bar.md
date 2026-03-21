# T022 — Standing Progress Bar (gold, 0→standing_target_mins)

**Status:** open
**Priority:** P2
**Depends on:** T027 (split session.rs + add standing_target_mins to config)
**Branch:** feat/T022-standing-progress-bar
**Vision:** `.agent/vision/2026-03-20-standing-gamification.md`

---

## Goal

When the user stands up, show a **gold progress bar** at the top of the screen that fills from
0 → `standing_target_mins` (default 15 min). When the bar reaches 100%, flash gold for 2s (lap
celebration), then start lap 2. Currently the bar is hidden while standing — standing is invisible
and unrewarding.

---

## Behaviour Spec

### Sitting state
Bar shows red/green as per sitting session progress (existing behaviour, unchanged).

### Standing state
```
0%                    50%                   100%
│▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓░░░░░░░░░░░░░░░░░░░░░░│  ← gold, filling
                                            ↓ at 100%:
                                         2s gold PULSE (flash)
                                            ↓ then lap 2:
│████│▓▓▓▓▓▓▓░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│
 lap1  new fill →
```
- Color: goldenrod `#DAA520`, brighter at 100% `#FFD700`
- Lap indicator: thin bright-gold segment on far left (8px per completed lap)
- Bar hidden when user sits down

### Lap completion flash
At `lap_progress >= 1.0`:
1. 2-second gold pulse (`set_variant(2)` with gold color already set)
2. After 2s: increment lap, reset progress, back to `set_variant(0)` solid gold fill

---

## Architecture Decisions (from eng review 2026-03-21)

### Lap flash state lives in `OverlayState` — NOT in AppState

**Why:** `OverlayState` already has a `Mutex` + background thread (`run_event_loop`) that checks
state every frame. Adding `lap_flash_until: Option<Instant>` there lets the thread self-manage
the timeout without external ticking. Zero changes to `AppState`.

```rust
// overlay_renderer.rs — add to OverlayState:
pub lap: u32,                           // completed laps today (for left-indicator rendering)
pub lap_flash_until: Option<Instant>,   // Some(t) = flash active until t
pub last_flashed_lap: u32,              // session lap that last triggered a flash (prevents re-flash on same lap)
pub standing_mode: bool,               // true when showing gold bar
```

The background thread (`run_event_loop` in `overlay_opaque.rs`/`overlay_layered.rs`) checks:
```rust
if let Some(until) = s.lap_flash_until {
    if Instant::now() > until {
        s.lap_flash_until = None;
        s.overlay_variant = 0; // back to solid
    }
}
```

### Testability: `tick_lap_flash(now: Instant)` method

Add to `OverlayRenderer`:
```rust
/// Advance the lap flash timer. Call run_event_loop's tick with a test Instant.
/// In production: run_event_loop calls this with Instant::now().
/// In tests: call with a synthetic future Instant to simulate expiry.
pub fn tick_lap_flash(&self, now: Instant) {
    if let Ok(mut s) = self.state.lock() {
        if let Some(until) = s.lap_flash_until {
            if now > until {
                s.lap_flash_until = None;
                s.overlay_variant = 0;
                s.needs_redraw = true;
            }
        }
    }
}
```

### Standing state transition

When user sits down, `on_state_changed()` must clear lap flash:
```rust
// In on_state_changed(), when state = Sitting:
overlay.clear_standing(); // new method: sets standing_mode=false, clears lap_flash_until, resets last_flashed_lap=0
```

---

## Config (after T027)

T027 adds these fields to `AppConfig` in `config.rs`:
```rust
pub standing_target_mins: u32,  // default 15, clamp 5–60
                                 // #[serde(alias = "stand_limit_mins")] for migration
pub stand_max_mins: u32,        // default 90, clamp 30–120 (warning threshold)
```

Use `standing_target_mins` in T022. The field `stand_limit_secs` in `SessionStateDto` becomes
`standing_target_secs` after T027 (or keep old name and add new — check T027's decision).

---

## Key Code Context (read before implementing)

### `OverlayRenderer` public API — `src-tauri/src/overlay_renderer.rs`
```rust
impl OverlayRenderer {
    pub fn update(&self, progress: f32, color_rgb: (u8, u8, u8))
    pub fn show(&self)
    pub fn hide(&self)
    pub fn set_variant(&self, variant: u8)  // 0=solid, 1=gradient, 2=pulsing
    // ADD:
    pub fn update_standing(&self, lap_progress: f32, lap: u32)  // sets gold mode
    pub fn start_lap_flash(&self)                                // 2s gold pulse (internal use)
    pub fn maybe_flash_lap(&self, session_lap: u32)              // flash if session_lap > last_flashed_lap
    pub fn clear_standing(&self)                                 // revert to sitting, resets last_flashed_lap
    pub fn tick_lap_flash(&self, now: Instant)                   // for testing
}
```

### `tray_controller.rs` — where to wire
- `on_state_changed()`: called on `desk:state-changed` — transitions
- `update_overlay_progress()`: called on `desk:distance` ~every second

### `SessionStateDto` fields available
```rust
pub state: DeskState,
pub standing_seconds: i64,    // total standing today ← use for progress calc
pub stand_limit_secs: i64,    // standing target in secs (after T027: standing_target_secs)
pub desk_height_cm: f32,
```

### Progress calculation

**IMPORTANT:** Use `standing_session_secs` (current session, resets on sit) for bar progress — NOT `standing_seconds` (total today).
Mixing them causes flash to fire immediately when user stands up after completing a previous lap.

`standing_session_secs` is added to `SessionStateDto` by T028. If implementing T022 before T028 merges,
add `standing_session_secs: i64` to `SessionStateDto` yourself.

```rust
// In update_overlay_progress(), when state = Standing:
let target_secs = snapshot.standing_target_secs;  // after T027 rename (was stand_limit_secs)

// standing_session_secs = current continuous standing session (resets on sit)
// standing_seconds = cumulative standing today (never decreases)
let session_secs = snapshot.standing_session_secs;     // for progress + flash detection
let total_laps = (snapshot.standing_seconds / target_secs) as u32;  // for lap indicator only
let session_lap = (session_secs / target_secs) as u32;
let lap_progress = (session_secs % target_secs) as f32 / target_secs as f32;
// lap_progress: 0.0..1.0 within the current lap of this session
```

### `colors.rs` — add `color_for_standing`
```rust
pub fn color_for_standing(progress: f32) -> (u8, u8, u8) {
    // progress 0.0→1.0: goldenrod → bright gold
    let t = progress.clamp(0.0, 1.0);
    let r = (0xDA as f32 + (0xFF - 0xDA) as f32 * t) as u8;  // 218→255
    let g = (0xA5 as f32 + (0xD7 - 0xA5) as f32 * t) as u8;  // 165→215
    let b = 0x20u8;  // constant: 32
    (r, g, b)
}
// 0.0 → #DAA520 (goldenrod)
// 1.0 → #FFD720 (bright gold)
```

### `set_variant(2)` note
Variant 2 = pulsing. Used by BOTH alert Stage1 (red) AND lap flash (gold). The color is set
separately via `update()`. Call `overlay.update(lap_progress, gold_color)` BEFORE `set_variant(2)`
to ensure pulsing uses the gold color, not the alert red.

Update the docstring in `overlay_renderer.rs` line 150 to reflect both uses:
```
/// - `2` — pulsing animation (used by alert Stage1 and standing lap flash)
```

---

## Implementation Plan

### 1. `colors.rs` — add `color_for_standing()`

### 2. `overlay_renderer.rs` — extend `OverlayState` + add methods

Add to `OverlayState` struct:
```rust
pub standing_mode: bool,
pub lap: u32,
pub lap_flash_until: Option<Instant>,
```

Add methods to `OverlayRenderer`:
- `update_standing(lap_progress, lap)` — sets gold color + standing_mode=true + progress
- `start_lap_flash()` — sets lap_flash_until = Instant::now() + 2s, set_variant(2)
- `maybe_flash_lap(session_lap: u32)` — flashes only if session_lap > last_flashed_lap; updates last_flashed_lap
- `clear_standing()` — sets standing_mode=false, clears lap_flash_until, resets last_flashed_lap=0, set_variant(0)
- `tick_lap_flash(now: Instant)` — expires flash if now > lap_flash_until

### 3. `overlay_opaque.rs` — render gold bar + lap indicator

In the render function, check `s.standing_mode`:
```rust
if s.standing_mode {
    // gold bar: width = lap_progress * screen_width
    // lap indicator: thin bright segment on far left = lap * 8 px
    render_gold_bar(hdc, screen_width, bar_height, s.progress, s.lap);
} else {
    // existing sitting bar render (unchanged)
    render_sitting_bar(hdc, screen_width, bar_height, s.progress, s.color_rgb);
}
```

Also add `tick_lap_flash(Instant::now())` call inside the render loop.

### 4. `overlay_layered.rs` — same as step 3

### 5. `tray_controller.rs` — wire standing mode

In `update_overlay_progress()`, BEFORE the early return:
```rust
let snapshot = session.snapshot();

// Standing mode: show gold bar
if snapshot.state == DeskState::Standing {
    let target_secs = snapshot.stand_limit_secs;
    if target_secs > 0 {
        // Use standing_session_secs (current session) for progress — NOT standing_seconds (total today).
        // Bug otherwise: user who completed lap 1, sits, stands again → flash fires immediately
        // because total=900, target=900, lap=1, progress=0.0 at session start.
        let session_secs = snapshot.standing_session_secs;  // added to SessionStateDto by T028
        let session_lap = (session_secs / target_secs) as u32;
        let lap_progress = (session_secs % target_secs) as f32 / target_secs as f32;
        let total_laps = (snapshot.standing_seconds / target_secs) as u32; // for left indicator only
        let (r, g, b) = color_for_standing(lap_progress);

        overlay.update_standing(lap_progress, total_laps);
        overlay.update(lap_progress, (r, g, b));
        overlay.show();

        // Flash when session_lap increments. maybe_flash_lap() is idempotent:
        // compares session_lap against last_flashed_lap stored in OverlayState.
        // Resets to 0 in clear_standing() so each new session can earn its own flash.
        overlay.maybe_flash_lap(session_lap);
    }
    // Update tooltip (handled by update_tooltip() call at top)
    return;
}

// Sitting mode: existing logic unchanged
if snapshot.state != DeskState::Sitting {
    return;
}
```

In `on_state_changed()`, when transitioning to Sitting:
```rust
overlay.clear_standing(); // cancel any lap flash
```

---

## Files to Modify

| File | Change |
|------|--------|
| `src-tauri/src/colors.rs` | Add `color_for_standing(progress) → (u8,u8,u8)` |
| `src-tauri/src/overlay_renderer.rs` | Add `standing_mode`, `lap`, `lap_flash_until` to `OverlayState`; add 4 new methods |
| `src-tauri/src/overlay_opaque.rs` | Render gold bar branch + call `tick_lap_flash()` |
| `src-tauri/src/overlay_layered.rs` | Same |
| `src-tauri/src/tray_controller.rs` | Wire standing mode in `update_overlay_progress()` and `on_state_changed()` |
| `src-tauri/src/overlay_tests.rs` | Add tests for new overlay methods |

**Note:** `session.rs` / `session_types.rs` — no changes needed (standing_seconds already tracked).

---

## Tests (~20)

### Unit — `colors.rs`
- `color_for_standing(0.0)` → goldenrod `(218, 165, 32)`
- `color_for_standing(1.0)` → bright gold `(255, 215, 32)`
- `color_for_standing(0.5)` → mid gold (between the two)

### Unit — `overlay_renderer.rs` / `overlay_tests.rs`
- `update_standing(0.5, 0)` → standing_mode=true, progress=0.5, lap=0
- `clear_standing()` → standing_mode=false, lap_flash_until=None, last_flashed_lap=0
- `start_lap_flash()` → lap_flash_until=Some(Instant + 2s), variant=2
- `tick_lap_flash(now_before_expiry)` → lap_flash_until unchanged, variant=2
- `tick_lap_flash(now_after_expiry)` → lap_flash_until=None, variant=0 ← **key test**
- `start_lap_flash()` then `clear_standing()` → lap_flash_until=None (sit cancels flash)
- `maybe_flash_lap(1)` when last_flashed_lap=0 → flashes (lap_flash_until set), last_flashed_lap=1
- `maybe_flash_lap(1)` again when last_flashed_lap=1 → no flash ← **key test: no double-flash**
- `maybe_flash_lap(2)` when last_flashed_lap=1 → flashes (second lap bonus)
- `clear_standing()` → last_flashed_lap=0; `maybe_flash_lap(1)` → flashes again (new session)

### Unit — `tray_controller.rs` / progress calculation
Progress uses `standing_session_secs` (current session); lap indicator uses `standing_seconds` (total today):
- `session_secs=0, target=900` → session_lap=0, lap_progress=0.0, total_laps=0
- `session_secs=450, target=900` → session_lap=0, lap_progress=0.5, total_laps=0
- `session_secs=900, standing_seconds=900, target=900` → session_lap=1, lap_progress=0.0, total_laps=1 → maybe_flash_lap(1) fires
- `session_secs=1350, standing_seconds=1350, target=900` → session_lap=1, lap_progress=0.5
- Scenario: stand 15min (total=900), sit, stand again 5min (session_secs=300, total=1200):
  → session_lap=0, lap_progress=0.33, total_laps=1 (indicator shows 1 completed lap today)
  → no flash fires (session_lap=0, last_flashed_lap was reset to 0 on sit) ← **guards against old bug**

### Unit — `overlay_opaque.rs`
- Standing mode: bar width = lap_progress × screen_width
- Lap=0: no left indicator segment
- Lap=1: 8px left segment
- Lap=2: 16px left segment

---

## Acceptance Criteria

- [ ] When standing, top bar turns gold and fills over `standing_target_mins`
- [ ] At target reached: 2s gold pulse, then lap 2 starts from left
- [ ] Sitting → bar immediately returns to red sitting behavior (no lap flash residue)
- [ ] `color_for_standing()` in `colors.rs` (DRY, not hardcoded)
- [ ] `tick_lap_flash(now)` method enables unit testing without sleep
- [ ] Docstring on `set_variant()` updated (stale: only mentioned alert)
- [ ] All ~16 tests pass
- [ ] `cargo test` + `pnpm test:unit` pass (no regressions)
- [ ] All modified files ≤ 250 lines (pre-commit hook)
