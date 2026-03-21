# T028 — Points system (in-memory, daily score)

**Status:** open
**Priority:** P2
**Depends on:** T022 (standing progress bar), T027 (split session.rs)
**Branch:** feat/T028-points-system

---

## Goal

Track a daily posture score. The score is the motivational "why" behind the gold bar.
In-memory only (resets at midnight — same as other daily counters).

---

## Scoring Rules

```
+1.0  per minute standing
+5.0  per complete standing session (standing_target_mins reached)
-0.5  per minute sitting
 0    walking / away (neutral)
```

### Balance check
Good behavior: Stand 15 min → +20 pts (15×1 + 5 bonus)
That compensates: 40 min sitting → −20 pts
So 40 min sitting + 15 min standing break → net 0. Fair.

### Configurable (but not UI-exposed yet)
Add to `AppConfig`:
```rust
#[serde(default = "default_pts_standing_per_min")]
pub pts_standing_per_min: f32,    // default: 1.0
#[serde(default = "default_pts_session_bonus")]
pub pts_session_bonus: f32,       // default: 5.0
#[serde(default = "default_pts_sitting_per_min")]
pub pts_sitting_per_min: f32,     // default: -0.5
```
Settings UI exposure deferred to BACKLOG.

---

## Implementation

### `session_types.rs` — add score to `SessionState`

```rust
pub struct SessionState {
    // ... existing fields ...
    /// Daily posture score (in-memory, resets at midnight).
    pub daily_score: f32,
    /// Whether the standing session bonus (+5) was awarded for current standing session.
    pub standing_bonus_awarded: bool,
}
```

### `session_manager.rs` — score accumulation

**Where to call:** Score accumulates in `accumulate_score_tick()` — a new method called from
`serial.rs` every second (right after `on_reading()`). This follows the existing pattern where
config-dependent methods receive config as a parameter:

```rust
// serial.rs (inside per-second sensor loop):
let result = sess.on_reading(mm, active);
sess.accumulate_score_tick(&config);  // ← new call, same pattern as check_notification_conditions
```

```rust
// session_manager.rs — new method:
pub fn accumulate_score_tick(&mut self, config: &AppConfig) {
    match self.state.state {
        DeskState::Sitting  => self.state.daily_score += config.pts_sitting_per_min / 60.0,
        DeskState::Standing => self.state.daily_score += config.pts_standing_per_min / 60.0,
        _ => {}  // Walking / Away: neutral
    }
}
```

**Lap bonus (per lap, NOT per session end):**

Award +5 when lap completes — i.e., when `standing_session_secs` crosses a multiple of `target_secs`.
Track with `lap_bonus_awarded_for_lap: u32` (the last lap that received a bonus):

```rust
// In accumulate_score_tick(), after score delta:
if self.state.state == DeskState::Standing {
    let target_secs = config.standing_target_mins as i64 * 60;
    let current_lap = self.state.standing_session_secs / target_secs;
    if current_lap > self.state.lap_bonus_awarded_for_lap as i64 {
        self.state.daily_score += config.pts_session_bonus;
        self.state.lap_bonus_awarded_for_lap = current_lap as u32;
    }
}
```

Reset `lap_bonus_awarded_for_lap = 0` when user sits (new standing session starts).

In daily reset: `self.state.daily_score = 0.0; self.state.lap_bonus_awarded_for_lap = 0;`

### `session_types.rs` — add score to `SessionStateDto`

Also add `standing_session_secs` to `SessionStateDto` — T022 (gold bar, same wave) needs it for
progress calculation. Without this field in the snapshot, T022 cannot compute lap_progress.

```rust
pub struct SessionStateDto {
    // ... existing fields ...
    pub daily_score: f32,
    pub standing_session_secs: i64,  // current continuous standing session (resets on sit) — needed by T022
}
```

### `tray_controller.rs` — show score in tooltip

```rust
// In update_tooltip():
let score_str = if snapshot.daily_score >= 0.0 {
    format!(" 🏆 +{:.0}", snapshot.daily_score)
} else {
    format!(" 📉 {:.0}", snapshot.daily_score)
};

let label = format!(
    "↕ {:.1} cm — {} ({}){}",
    snapshot.desk_height_cm,
    state_str,
    format_duration(duration_secs),
    score_str,
);
// e.g.: "↕ 114.0 cm — Standing 8:32  🏆 +38"
// e.g.: "↕ 72.0 cm — Sitting 15:00  📉 -8"
```

### `src/components/` — show score in floating window

In the floating window React component, display:
```tsx
<div className="daily-score">
  {score >= 0 ? `🏆 +${Math.round(score)}` : `📉 ${Math.round(score)}`} pts today
</div>
```

---

## Key Data Structures

After T027 split, the relevant files will be:
- `session_types.rs` — `SessionState`, `SessionStateDto`
- `session_manager.rs` — `SessionManager` (score logic here)
- `config.rs` — scoring config fields

`AppState` (in `commands.rs`) exposes session via `Arc<Mutex<SessionManager>>`.
`get_session_state` IPC command returns `SessionStateDto` — add `daily_score` to it.

---

## Coordination with T022 (same Wave 3)

T022 (gold bar) and T028 (points) are parallel agents in Wave 3. They use `standing_session_secs`
for different purposes and must agree on semantics:

| | T022 | T028 |
|---|---|---|
| Uses `standing_session_secs` for | gold bar progress within current session | `accumulate_score_tick()` increments it; lap bonus detection |
| Uses `standing_seconds` for | left lap indicator (total laps today) | not used directly |
| Lap = | `session_secs / target` (per-session, resets on sit) | same |

**Key difference:** T022 lap count for the bar resets to 0 each new standing session. T028 bonus
also resets per session (via `lap_bonus_awarded_for_lap = 0` on sit). These are consistent.

**Merge note:** If both agents touch `session_types.rs` to add `standing_session_secs`, the merge
coordinator should take whichever adds it first and verify the other agent's changes still apply.

---

## Note on `standing_session_secs` and `lap_bonus_awarded_for_lap`

`session.standing_seconds` = total standing today (cumulative, never resets mid-day).
`standing_session_secs` = duration of current continuous standing session (resets when user sits).

Both fields are needed. Add to `SessionState` in `session_types.rs` (after T027 split):
```rust
/// Standing seconds in current continuous standing session (resets on sit/away).
pub standing_session_secs: i64,
/// The last lap number for which the +5 bonus was awarded (resets per session).
pub lap_bonus_awarded_for_lap: u32,
```

`standing_session_secs` is incremented in `accumulate_score_tick()` when `state == Standing`.
Reset to 0 in `handle_state_transition()` when transitioning away from Standing.

---

## Tests (~12)

- Score starts at 0.0 on fresh day
- Sitting 60 ticks (1 min equivalent) → score delta = −0.5
- Standing 60 ticks (1 min equivalent) → score delta = +1.0
- Walking/Away ticks → score delta = 0.0
- Standing 15 min (`standing_session_secs=900`, `target=900`) → lap=1, bonus awarded, `lap_bonus_awarded_for_lap=1`
- Standing 14 min (`standing_session_secs=840`) → lap=0, NO bonus
- Lap 1 bonus awarded → tick again (still lap 1) → no second bonus ← **key test**
- Standing 30 min (2 laps) → 2 bonuses, `lap_bonus_awarded_for_lap=2`, score = +30+10 = +40
- Standing → sitting → standing again: `lap_bonus_awarded_for_lap=0`, can earn lap 1 bonus again
- Daily reset: score=0.0, lap_bonus_awarded_for_lap=0
- `SessionStateDto.daily_score` reflects accumulated score
- Score config defaults: pts_standing=1.0, pts_session_bonus=5.0, pts_sitting=-0.5

---

## Acceptance Criteria

- [ ] Score accumulates per second during sitting/standing
- [ ] +5 bonus awarded when standing_target_mins reached per session
- [ ] No double-award for same session
- [ ] Score resets at midnight (daily reset)
- [ ] Tray tooltip: `↕ 114.0 cm — Standing 8:32  🏆 +38`
- [ ] Floating window shows score
- [ ] `cargo test` + `pnpm test:unit` pass
