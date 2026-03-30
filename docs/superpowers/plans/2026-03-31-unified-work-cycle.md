# Unified Work Cycle & Activity Status — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add screen-time awareness to the existing sit/stand cycle — a gentle "walk away" nudge when standing after 60+ min at the computer, plus live idle status in the UI.

**Architecture:** Extends the existing CommunicationPolicy with a one-shot screen break nudge (no new escalation axis). Adds `idle_secs` and `away_bout_secs` to SessionStateDto. Frontend gets activity status in StateIndicator and two new Settings toggles.

**Tech Stack:** Rust (Tauri 2), React + TypeScript, Vitest, `#[test]`

**Spec:** `docs/superpowers/specs/2026-03-31-unified-work-cycle-design.md`
**ADR:** `.arch/ADR/011-unified-sit-stand-walk-cycle.md`

---

## File Map

### Rust — Create
- `src-tauri/src/screen_break_nudge.rs` — nudge message pool + selection logic

### Rust — Modify
- `src-tauri/src/ergonomic_profile.rs` — add `max_continuous_computer_secs`, `computer_break_reset_secs` to Limits
- `src-tauri/src/session_types.rs` — add `idle_secs`, `max_continuous_computer_secs`, `computer_break_reset_secs` to SessionState; add `idle_secs`, `away_bout_secs` to SessionStateDto
- `src-tauri/src/session_reading.rs` — use configurable reset threshold; populate `idle_secs`
- `src-tauri/src/session_manager.rs` — wire new ergonomic profile fields into SessionState
- `src-tauri/src/communication_profile.rs` — add `ScreenBreakNudgeConfig` field
- `src-tauri/src/communication_profile_visuals.rs` — add `ScreenBreakNudgeConfig` struct
- `src-tauri/src/communication_policy.rs` — add `screen_break_nudge_fired` flag + nudge logic in evaluate()
- `src-tauri/src/communication_profile_defaults.rs` — add default nudge config
- `src-tauri/src/lib.rs` — register `screen_break_nudge` module
- `src-tauri/profiles/ergonomic/*.json` — add new limit fields
- `src-tauri/profiles/communication/*.json` — add screen_break_nudge section + update sitting toast message

### Frontend — Modify
- `src/types.ts` — add `idle_secs`, `away_bout_secs`, `continuous_computer_secs` to SessionStateDto
- `src/hooks/useDeskTypes.ts` — add `idleSecs`, `awayBoutSecs`, `continuousComputerSecs` to UseDeskResult
- `src/hooks/useDesk.ts` — extract and expose new fields
- `src/components/StateIndicator.tsx` — add Active/Idle display
- `src/components/settings/SettingsTypes.ts` — add toggles
- `src/components/SettingsPanel.tsx` — add toggles in More tab

---

## Task 1: Ergonomic Profile — New Limit Fields

**Files:**
- Modify: `src-tauri/src/ergonomic_profile.rs`
- Modify: `src-tauri/profiles/ergonomic/default.json`
- Modify: `src-tauri/profiles/ergonomic/strict.json`
- Modify: `src-tauri/profiles/ergonomic/relaxed.json`
- Modify: `src-tauri/profiles/ergonomic/demo.json`

- [ ] **Step 1: Add default helper functions**

In `src-tauri/src/ergonomic_profile.rs`, after line 18 (`fn default_posture_balance_min_sitting_secs`), add:

```rust
fn default_max_continuous_computer_secs() -> u32 { 3600 }
fn default_computer_break_reset_secs() -> u32 { 300 }
```

- [ ] **Step 2: Add fields to Limits struct**

In `src-tauri/src/ergonomic_profile.rs`, inside the `Limits` struct (after `posture_balance_min_sitting_secs` field at line 81), add:

```rust
    /// Maximum continuous computer time (Sitting+Standing) before screen break nudge (seconds).
    /// Default: 3600 (60 minutes). Set 0 to disable.
    #[serde(default = "default_max_continuous_computer_secs")]
    pub max_continuous_computer_secs: u32,

    /// Away duration to fully reset continuous computer timer (seconds).
    /// Replaces the hardcoded 300s in session_reading.rs.
    /// Default: 300 (5 minutes).
    #[serde(default = "default_computer_break_reset_secs")]
    pub computer_break_reset_secs: u32,
```

- [ ] **Step 3: Add fields to Limits::default()**

In the `impl Default for Limits` block (around line 84-97), add inside the `Self { ... }`:

```rust
            max_continuous_computer_secs: default_max_continuous_computer_secs(),
            computer_break_reset_secs: default_computer_break_reset_secs(),
```

- [ ] **Step 4: Update all ergonomic profile JSON files**

Add to `limits` section of each profile:

`default.json`:
```json
    "max_continuous_computer_secs": 3600,
    "computer_break_reset_secs": 300
```

`strict.json`:
```json
    "max_continuous_computer_secs": 2700,
    "computer_break_reset_secs": 300
```

`relaxed.json`:
```json
    "max_continuous_computer_secs": 5400,
    "computer_break_reset_secs": 300
```

`demo.json`:
```json
    "max_continuous_computer_secs": 60,
    "computer_break_reset_secs": 10
```

- [ ] **Step 5: Run Rust tests**

Run: `cd src-tauri && cargo test`
Expected: All existing tests PASS. No new tests needed yet — the serde defaults ensure backward compat.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/ergonomic_profile.rs src-tauri/profiles/ergonomic/
git commit -m "feat(desk): add max_continuous_computer_secs to ergonomic profile"
```

---

## Task 2: SessionState & DTO — New Fields

**Files:**
- Modify: `src-tauri/src/session_types.rs`
- Modify: `src-tauri/src/session_manager.rs`

- [ ] **Step 1: Add fields to SessionState**

In `src-tauri/src/session_types.rs`, inside `SessionState` struct, after `posture_balance_min_sitting_secs` (line 146), add:

```rust
    /// Maximum continuous computer time (from ergonomic profile).
    pub max_continuous_computer_secs: i64,
    /// Away duration to reset continuous computer timer (from ergonomic profile).
    pub computer_break_reset_secs: i64,
    /// Current system idle time in seconds (refreshed every tick from activity.rs).
    pub idle_secs: i64,
```

- [ ] **Step 2: Add fields to SessionStateDto**

In `src-tauri/src/session_types.rs`, inside `SessionStateDto` struct, after `sitting_seconds_total` (line 176), add:

```rust
    /// Current system idle time in seconds.
    pub idle_secs: i64,
    /// Current continuous Away duration (seconds).
    pub away_bout_secs: i64,
    /// Maximum continuous computer time limit (seconds, from ergonomic profile).
    pub max_continuous_computer_secs: i64,
```

- [ ] **Step 3: Wire new fields in SessionManager initialization**

Find where `SessionState` is constructed in `session_manager.rs` (the `new()` or `default()` method). Add:

```rust
            max_continuous_computer_secs: 3600,
            computer_break_reset_secs: 300,
            idle_secs: 0,
```

- [ ] **Step 4: Wire ergonomic profile fields into SessionState**

Find where ergonomic profile limits are applied to SessionState in `session_manager.rs` (look for `break_min_secs` assignment). Add alongside:

```rust
        self.state.max_continuous_computer_secs = profile.limits.max_continuous_computer_secs as i64;
        self.state.computer_break_reset_secs = profile.limits.computer_break_reset_secs as i64;
```

- [ ] **Step 5: Update snapshot() to include new DTO fields**

Find the `snapshot()` method in `session_manager.rs` that builds `SessionStateDto`. Add the new fields:

```rust
            idle_secs: self.state.idle_secs,
            away_bout_secs: self.state.away_bout_secs,
            max_continuous_computer_secs: self.state.max_continuous_computer_secs,
```

- [ ] **Step 6: Run Rust tests**

Run: `cd src-tauri && cargo test`
Expected: PASS. May need to update test fixtures that construct SessionState/SessionStateDto.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/session_types.rs src-tauri/src/session_manager.rs
git commit -m "feat(desk): add idle_secs and away_bout_secs to SessionStateDto"
```

---

## Task 3: Configurable Computer Break Reset

**Files:**
- Modify: `src-tauri/src/session_reading.rs`
- Test: existing `#[test]` modules in session reading tests

- [ ] **Step 1: Write failing test**

Add to the test module in `session_reading.rs` (or the appropriate test file):

```rust
#[test]
fn computer_timer_resets_at_configurable_threshold() {
    let mut mgr = test_session_manager();
    mgr.state.computer_break_reset_secs = 120; // 2 min instead of default 5 min

    // Accumulate 50s of computer time
    for _ in 0..50 {
        mgr.on_reading(750, true); // sitting height, active
    }
    assert!(mgr.state.continuous_computer_secs >= 45);

    // Go Away for 120 seconds (the configured threshold)
    for i in 0..125 {
        mgr.on_reading(750, false); // inactive
    }
    assert_eq!(mgr.state.continuous_computer_secs, 0, "should reset at configured threshold");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test computer_timer_resets_at_configurable`
Expected: FAIL — currently hardcoded to 300.

- [ ] **Step 3: Replace hardcoded 300 with configurable value**

In `src-tauri/src/session_reading.rs`, line 190, change:

```rust
                if self.state.away_bout_secs == 300 {
```

to:

```rust
                if self.state.away_bout_secs == self.state.computer_break_reset_secs {
```

- [ ] **Step 4: Populate idle_secs in accumulate_ongoing**

In `src-tauri/src/session_reading.rs`, inside `accumulate_ongoing()`, after line 173 (`self.last_accumulate_ran = true;`), add:

```rust
        // Refresh system idle time from activity module.
        self.state.idle_secs = crate::activity::get_idle_seconds() as i64;
```

- [ ] **Step 5: Run tests**

Run: `cd src-tauri && cargo test`
Expected: All PASS including new test.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/session_reading.rs
git commit -m "feat(desk): configurable computer break reset + idle_secs in state"
```

---

## Task 4: Communication Profile — Screen Break Nudge Config

**Files:**
- Create: `src-tauri/src/screen_break_nudge.rs`
- Modify: `src-tauri/src/communication_profile_visuals.rs`
- Modify: `src-tauri/src/communication_profile.rs`
- Modify: `src-tauri/src/communication_profile_defaults.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/profiles/communication/*.json`

- [ ] **Step 1: Create screen_break_nudge.rs**

Create `src-tauri/src/screen_break_nudge.rs`:

```rust
//! Screen break nudge — random encouraging message when standing too long at screen.

use rand::seq::SliceRandom;

/// Select a random message from the nudge pool.
/// Returns None if pool is empty.
pub fn pick_nudge_message(messages: &[String]) -> Option<String> {
    let mut rng = rand::rng();
    messages.choose(&mut rng).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_from_empty_pool_returns_none() {
        assert!(pick_nudge_message(&[]).is_none());
    }

    #[test]
    fn pick_from_pool_returns_some() {
        let pool = vec!["msg1".to_string(), "msg2".to_string()];
        let result = pick_nudge_message(&pool);
        assert!(result.is_some());
        assert!(pool.contains(&result.unwrap()));
    }
}
```

- [ ] **Step 2: Register module in lib.rs**

In `src-tauri/src/lib.rs`, add alongside other `mod` declarations:

```rust
mod screen_break_nudge;
```

- [ ] **Step 3: Add ScreenBreakNudgeConfig struct**

In `src-tauri/src/communication_profile_visuals.rs`, add after `PeriodicNotificationConfig`:

```rust
/// Configuration for the screen-break nudge toast.
///
/// Fires once when standing and continuous_computer_secs exceeds the ergonomic
/// profile limit. Resets when computer timer resets (5+ min Away).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenBreakNudgeConfig {
    /// Whether screen break nudges are enabled.
    #[serde(default = "super::defs::default_nudge_enabled")]
    pub enabled: bool,

    /// Pool of encouraging messages (one is chosen randomly).
    #[serde(default = "super::defs::default_nudge_messages")]
    pub messages: Vec<String>,
}

impl Default for ScreenBreakNudgeConfig {
    fn default() -> Self {
        Self {
            enabled: super::defs::default_nudge_enabled(),
            messages: super::defs::default_nudge_messages(),
        }
    }
}
```

- [ ] **Step 4: Add defaults in communication_profile_defaults.rs**

In `src-tauri/src/communication_profile_defaults.rs`, add:

```rust
pub fn default_nudge_enabled() -> bool { true }

pub fn default_nudge_messages() -> Vec<String> {
    vec![
        "You've been at the screen a while — perfect moment to grab water".to_string(),
        "Your eyes would love a 5-minute break".to_string(),
        "Quick stretch? 2 minutes is all it takes".to_string(),
        "Step away for a moment — your focus will be sharper when you return".to_string(),
        "Screen break time — look out the window for a minute".to_string(),
    ]
}
```

- [ ] **Step 5: Add field to CommunicationProfile**

In `src-tauri/src/communication_profile.rs`, inside `CommunicationProfile` struct (after `periodic_notifications`), add:

```rust
    /// Screen break nudge configuration (fires when standing + computer time exceeded).
    #[serde(default)]
    pub screen_break_nudge: ScreenBreakNudgeConfig,
```

Add the import at the top:
```rust
pub use crate::communication_profile_visuals::ScreenBreakNudgeConfig;
```

And in the `Default` impl, add:
```rust
            screen_break_nudge: ScreenBreakNudgeConfig::default(),
```

- [ ] **Step 6: Update all communication profile JSONs**

Add to each `communication/*.json` after `periodic_notifications`:

`default.json`:
```json
  "screen_break_nudge": {
    "enabled": true,
    "messages": [
      "You've been at the screen a while — perfect moment to grab water",
      "Your eyes would love a 5-minute break",
      "Quick stretch? 2 minutes is all it takes",
      "Step away for a moment — your focus will be sharper when you return",
      "Screen break time — look out the window for a minute"
    ]
  }
```

`aggressive.json`: same messages, `"enabled": true`
`gentle.json`: same messages, `"enabled": true`
`silent.json`: `"enabled": false`, empty messages `[]`
`demo.json`: `"enabled": true`, add `"Demo: screen break nudge!"` as only message

- [ ] **Step 7: Update sitting_limit_toast in all profiles**

In all `communication/*.json` files, change `messages.sitting_limit_toast`:

From: `"Time to change position"`
To: `"Time for a change — stand up or step away from the screen"`

- [ ] **Step 8: Check `rand` dependency**

Run: `cd src-tauri && grep -q 'rand' Cargo.toml && echo "exists" || echo "need to add"`

If missing, add to `[dependencies]` in `Cargo.toml`:
```toml
rand = "0.9"
```

- [ ] **Step 9: Run Rust tests**

Run: `cd src-tauri && cargo test`
Expected: All PASS including new screen_break_nudge tests.

- [ ] **Step 10: Commit**

```bash
git add src-tauri/src/screen_break_nudge.rs src-tauri/src/lib.rs \
  src-tauri/src/communication_profile_visuals.rs \
  src-tauri/src/communication_profile.rs \
  src-tauri/src/communication_profile_defaults.rs \
  src-tauri/profiles/communication/
git commit -m "feat(desk): add screen break nudge config to communication profiles"
```

---

## Task 5: CommunicationPolicy — Nudge Logic

**Files:**
- Modify: `src-tauri/src/communication_policy.rs`

- [ ] **Step 1: Write failing test**

Add to `communication_policy.rs` test module (or create one):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ergonomic_profile::ErgonomicProfile;
    use crate::communication_profile::CommunicationProfile;

    fn test_policy() -> CommunicationPolicy {
        let mut ergo = ErgonomicProfile::default();
        ergo.limits.max_continuous_computer_secs = 60; // 60s for testing
        let comm = CommunicationProfile::default();
        CommunicationPolicy::new(comm, ergo)
    }

    #[test]
    fn nudge_fires_when_standing_past_computer_limit() {
        let mut policy = test_policy();
        let input = PolicyInput {
            state: DeskState::Standing,
            elapsed_secs: 30, // 30s standing (within standing limit)
            sensor_connected: true,
            standing_lap_progress: 0.5,
            standing_lap: 0,
            standing_lap_flash: false,
            continuous_computer_secs: 65, // Over the 60s limit
        };
        let signals = policy.evaluate(&input);
        assert!(signals.notify.is_some(), "nudge toast should fire");
    }

    #[test]
    fn nudge_does_not_fire_when_sitting() {
        let mut policy = test_policy();
        let input = PolicyInput {
            state: DeskState::Sitting,
            elapsed_secs: 30,
            sensor_connected: true,
            standing_lap_progress: 0.0,
            standing_lap: 0,
            standing_lap_flash: false,
            continuous_computer_secs: 65,
        };
        let signals = policy.evaluate(&input);
        // Sitting has its own escalation — nudge should NOT override it
        // At 30s elapsed with 2400 limit, no escalation step is active, so no notify
        assert!(signals.notify.is_none());
    }

    #[test]
    fn nudge_fires_only_once() {
        let mut policy = test_policy();
        let input = PolicyInput {
            state: DeskState::Standing,
            elapsed_secs: 30,
            sensor_connected: true,
            standing_lap_progress: 0.5,
            standing_lap: 0,
            standing_lap_flash: false,
            continuous_computer_secs: 65,
        };
        let s1 = policy.evaluate(&input);
        assert!(s1.notify.is_some(), "first call fires nudge");
        let s2 = policy.evaluate(&input);
        assert!(s2.notify.is_none(), "second call should NOT fire nudge again");
    }

    #[test]
    fn nudge_resets_on_position_change() {
        let mut policy = test_policy();
        let input = PolicyInput {
            state: DeskState::Standing,
            elapsed_secs: 30,
            sensor_connected: true,
            standing_lap_progress: 0.5,
            standing_lap: 0,
            standing_lap_flash: false,
            continuous_computer_secs: 65,
        };
        policy.evaluate(&input); // fires nudge
        policy.on_position_changed(); // reset
        // After position change, if computer_secs is still high, can fire again
        let s2 = policy.evaluate(&input);
        assert!(s2.notify.is_some(), "nudge should fire again after reset");
    }
}
```

- [ ] **Step 2: Add `continuous_computer_secs` to PolicyInput**

In `src-tauri/src/communication_policy.rs`, add to `PolicyInput` struct (after `standing_lap_flash`):

```rust
    /// Continuous seconds at the computer (Sitting+Standing). Resets after 5+ min Away.
    pub continuous_computer_secs: i64,
```

- [ ] **Step 3: Add nudge flag to CommunicationPolicy**

In the `CommunicationPolicy` struct, add field:

```rust
    /// Whether the screen break nudge has been fired this computer session.
    screen_break_nudge_fired: bool,
```

Initialize to `false` in `new()`.

- [ ] **Step 4: Reset nudge flag in on_position_changed()**

In `on_position_changed()`, add:

```rust
        self.screen_break_nudge_fired = false;
```

- [ ] **Step 5: Add nudge check in evaluate()**

In `evaluate()`, after the existing escalation logic (after the `match active_step` block, before the closing `}`), add:

```rust
        // Screen break nudge: when standing within limits but computer time exceeded
        let mut signals = match active_step {
            Some((idx, step)) => self.step_to_signals(step, idx, input),
            None => self.make_baseline(input),
        };

        // Only nudge when standing (sitting has its own escalation) and no existing notification
        if input.state == DeskState::Standing
            && signals.notify.is_none()
            && !self.screen_break_nudge_fired
            && self.ergo_profile.limits.max_continuous_computer_secs > 0
            && input.continuous_computer_secs >= self.ergo_profile.limits.max_continuous_computer_secs as i64
            && self.comm_profile.screen_break_nudge.enabled
        {
            if let Some(msg) = crate::screen_break_nudge::pick_nudge_message(
                &self.comm_profile.screen_break_nudge.messages,
            ) {
                signals.notify = Some(NotifySignal::Toast(msg));
                self.screen_break_nudge_fired = true;
            }
        }

        signals
```

Note: this replaces the existing `match active_step` block — the result now goes into a mutable `signals` variable.

- [ ] **Step 6: Pass continuous_computer_secs from tray_controller**

In `src-tauri/src/tray_controller.rs`, in the `update_from_policy` function (around line 130), update the PolicyInput construction:

```rust
    let input = PolicyInput {
        state: snapshot.state.clone(),
        elapsed_secs,
        sensor_connected: is_connected,
        standing_lap_progress,
        standing_lap,
        standing_lap_flash,
        continuous_computer_secs: snapshot.continuous_computer_secs,
    };
```

- [ ] **Step 7: Run tests**

Run: `cd src-tauri && cargo test`
Expected: All PASS including 4 new nudge tests.

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/communication_policy.rs src-tauri/src/tray_controller.rs
git commit -m "feat(desk): screen break nudge when standing past computer time limit"
```

---

## Task 6: Frontend — DTO & Hook Updates

**Files:**
- Modify: `src/types.ts`
- Modify: `src/hooks/useDeskTypes.ts`
- Modify: `src/hooks/useDesk.ts`

- [ ] **Step 1: Add fields to SessionStateDto**

In `src/types.ts`, inside `SessionStateDto` interface (after `current_session_secs` at line 76), add:

```typescript
  /** Current system idle time in seconds. */
  idle_secs: number;
  /** Current continuous Away duration in seconds. */
  away_bout_secs: number;
  /** Continuous seconds at computer (Sitting+Standing). Resets after 5+ min Away. */
  continuous_computer_secs: number;
  /** Maximum continuous computer time limit from ergonomic profile. */
  max_continuous_computer_secs: number;
```

- [ ] **Step 2: Add fields to UseDeskResult**

In `src/hooks/useDeskTypes.ts`, inside `UseDeskResult` interface (after `transition`), add:

```typescript
  /** Current system idle time in seconds. */
  idleSecs: number;
  /** Current continuous Away bout duration in seconds. */
  awayBoutSecs: number;
  /** Continuous seconds at the computer. Resets after 5+ min Away. */
  continuousComputerSecs: number;
```

- [ ] **Step 3: Add state variables and expose in useDesk**

In `src/hooks/useDesk.ts`, add state variables alongside existing ones (e.g. after `dailyScore`):

```typescript
const [idleSecs, setIdleSecs] = useState(0);
const [awayBoutSecs, setAwayBoutSecs] = useState(0);
const [continuousComputerSecs, setContinuousComputerSecs] = useState(0);
```

In the `fetchState` function (where `dto` fields are read), add:

```typescript
    setIdleSecs(dto.idle_secs ?? 0);
    setAwayBoutSecs(dto.away_bout_secs ?? 0);
    setContinuousComputerSecs(dto.continuous_computer_secs ?? 0);
```

In the return object, add:

```typescript
    idleSecs, awayBoutSecs, continuousComputerSecs,
```

- [ ] **Step 4: Run frontend tests**

Run: `pnpm test:unit`
Expected: PASS. Existing mocked tests should still work because new fields default to 0 via `?? 0`.

- [ ] **Step 5: Commit**

```bash
git add src/types.ts src/hooks/useDeskTypes.ts src/hooks/useDesk.ts
git commit -m "feat(desk): expose idle_secs and away_bout_secs in frontend hook"
```

---

## Task 7: StateIndicator — Activity Status Display

**Files:**
- Modify: `src/components/StateIndicator.tsx`
- Test: `src/components/StateIndicator.test.tsx` (create or modify)

- [ ] **Step 1: Write failing tests**

Create/modify `src/components/StateIndicator.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import StateIndicator from "./StateIndicator";

describe("StateIndicator", () => {
  it("shows Active when idle < 30s", () => {
    render(<StateIndicator state="Standing" deskHeightCm={105} idleSecs={10} showActivity={true} />);
    expect(screen.getByText("Active")).toBeTruthy();
  });

  it("shows Idle duration when idle >= 30s", () => {
    render(<StateIndicator state="Standing" deskHeightCm={105} idleSecs={150} showActivity={true} />);
    expect(screen.getByText("Idle 2m 30s")).toBeTruthy();
  });

  it("hides height when Away", () => {
    render(<StateIndicator state="Away" deskHeightCm={105} idleSecs={300} showActivity={true} />);
    expect(screen.queryByText("105 cm")).toBeNull();
    expect(screen.getByText("Idle 5m 0s")).toBeTruthy();
  });

  it("hides activity when showActivity is false", () => {
    render(<StateIndicator state="Standing" deskHeightCm={105} idleSecs={150} showActivity={false} />);
    expect(screen.queryByText("Active")).toBeNull();
    expect(screen.queryByText("Idle")).toBeNull();
  });

  it("renders without activity props (backward compat)", () => {
    render(<StateIndicator state="Sitting" deskHeightCm={72} />);
    expect(screen.getByText("Sitting")).toBeTruthy();
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `pnpm test:unit -- StateIndicator`
Expected: FAIL — props don't exist yet.

- [ ] **Step 3: Implement StateIndicator changes**

Replace `src/components/StateIndicator.tsx`:

```tsx
/**
 * StateIndicator.tsx — current ergonomic state with status dot, desk height, and activity.
 */
import type { FC } from "react";
import type { DeskState } from "@/types";

interface StateIndicatorProps {
  state: DeskState | null;
  deskHeightCm: number;
  /** Current system idle seconds. */
  idleSecs?: number;
  /** Whether to show activity status. */
  showActivity?: boolean;
}

const STATE_DOT_CLASS: Record<DeskState, string> = {
  Sitting:  "state-indicator__dot--sitting",
  Standing: "state-indicator__dot--standing",
  Walking:  "state-indicator__dot--walking",
  Away:     "state-indicator__dot--away",
};

/** Format seconds into "Xm Ys" string. */
function formatIdleTime(secs: number): string {
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  return `${m}m ${s}s`;
}

const StateIndicator: FC<StateIndicatorProps> = ({
  state,
  deskHeightCm,
  idleSecs = 0,
  showActivity = false,
}) => {
  const dotClass = state ? STATE_DOT_CLASS[state] : "";
  const isAway = state === "Away";
  const isIdle = idleSecs >= 30;

  return (
    <div className="state-indicator">
      <div className="state-indicator__left">
        <span className={`state-indicator__dot ${dotClass}`} />
        <span className="state-indicator__label">{state ?? "Unknown"}</span>
        {showActivity && (
          <span className={`state-indicator__activity ${isIdle ? "state-indicator__activity--idle" : ""}`}>
            {isIdle ? `Idle ${formatIdleTime(idleSecs)}` : "Active"}
          </span>
        )}
      </div>
      {!isAway && (
        <span className="state-indicator__height">
          {deskHeightCm > 0 ? `${deskHeightCm.toFixed(0)} cm` : "— cm"}
        </span>
      )}
    </div>
  );
};

export default StateIndicator;
```

- [ ] **Step 4: Add CSS for activity status**

Find the StateIndicator CSS (likely `globals.css` or a component CSS). Add:

```css
.state-indicator__activity {
  margin-left: 0.5em;
  font-size: 0.8em;
  opacity: 0.5;
}

.state-indicator__activity--idle {
  opacity: 0.8;
  color: var(--color-warning, #ffc107);
}
```

- [ ] **Step 5: Run tests**

Run: `pnpm test:unit -- StateIndicator`
Expected: All 5 tests PASS.

- [ ] **Step 6: Commit**

```bash
git add src/components/StateIndicator.tsx src/components/StateIndicator.test.tsx
git commit -m "feat(desk): show Active/Idle status in StateIndicator"
```

---

## Task 8: Settings Toggles

**Files:**
- Modify: `src/components/settings/SettingsTypes.ts`
- Modify: `src/components/SettingsPanel.tsx`

- [ ] **Step 1: Add toggle fields to DeskSettings**

In `src/components/settings/SettingsTypes.ts`, add to `DeskSettings` interface:

```typescript
  /** Enable computer time tracking and screen break nudges. */
  enable_computer_time_tracking: boolean;
  /** Show Active/Idle status in StateIndicator. */
  show_activity_status: boolean;
```

Add to `DEFAULT_SETTINGS`:

```typescript
  enable_computer_time_tracking: true,
  show_activity_status: true,
```

- [ ] **Step 2: Add toggles in SettingsPanel More tab**

In `src/components/SettingsPanel.tsx`, inside the `activeTab === 3` block (the More tab), add after `<TimelineSkinSection />`:

```tsx
    <div className="settings-panel__section">
      <h3 className="settings-panel__section-title">Experimental</h3>
      <label className="settings-panel__toggle">
        <input
          type="checkbox"
          checked={settings.enable_computer_time_tracking}
          onChange={(e) => setSettings({ ...settings, enable_computer_time_tracking: e.target.checked })}
        />
        Screen time tracking & break nudges
      </label>
      <label className="settings-panel__toggle">
        <input
          type="checkbox"
          checked={settings.show_activity_status}
          onChange={(e) => setSettings({ ...settings, show_activity_status: e.target.checked })}
        />
        Show activity status (Active/Idle)
      </label>
    </div>
```

- [ ] **Step 3: Wire showActivity prop to StateIndicator**

Find where `<StateIndicator>` is rendered in the popup (likely `App.tsx` or a widget component). Add props:

```tsx
<StateIndicator
  state={state}
  deskHeightCm={deskHeightCm}
  idleSecs={idleSecs}
  showActivity={settings.show_activity_status}
/>
```

- [ ] **Step 4: Run frontend tests**

Run: `pnpm test:unit`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/components/settings/SettingsTypes.ts src/components/SettingsPanel.tsx
git commit -m "feat(desk): add Settings toggles for computer time and activity status"
```

---

## Task 9: Integration Wiring & Manual Verification

**Files:**
- Verify all tasks are wired correctly

- [ ] **Step 1: Build check**

Run: `cd src-tauri && cargo check`
Expected: No errors. All new types/fields are connected.

- [ ] **Step 2: Run full Rust test suite**

Run: `cd src-tauri && cargo test`
Expected: All PASS (existing + ~10 new tests).

- [ ] **Step 3: Run full TypeScript test suite**

Run: `pnpm test:unit`
Expected: All PASS.

- [ ] **Step 4: Start dev mode and verify**

Run: `pnpm tauri:dev:demo`

Verify:
1. Settings → More → "Screen time tracking" toggle visible and defaults ON
2. Settings → More → "Show activity status" toggle visible and defaults ON
3. StateIndicator shows "Active" when moving mouse
4. StateIndicator shows "Idle Xm Ys" after 30s no input
5. In demo mode, computer_time increments (check Debug tab if available)

- [ ] **Step 5: Test with mock mode**

Run: `pnpm tauri:dev:mock`

Verify:
1. After simulated standing transition, if computer_time > 60s, toast fires with nudge message
2. Toast fires only once per computer session
3. Sitting limit toast now says "stand up or step away from the screen"

- [ ] **Step 6: Commit any fixes**

```bash
git add -A
git commit -m "fix(desk): integration wiring for unified work cycle"
```

---

## Task 10: Update Documentation

**Files:**
- Modify: `.arch/UX-FLOW.md` (if exists)
- Modify: `CLAUDE.md`
- Modify: `PROJECT.xml`

- [ ] **Step 1: Update CLAUDE.md Session Logic section**

Add to the "Session Logic" section:

```markdown
- **Computer time tracking** (ADR 011): `continuous_computer_secs` tracks total time at computer (Sitting + Standing). After `computer_break_reset_secs` (default 5 min) of Away, resets to 0. When standing and computer time exceeds `max_continuous_computer_secs` (default 60 min), one gentle toast nudge fires encouraging a screen break. Configurable in ergonomic profile. Toggle: Settings → More → "Screen time tracking".
```

Add to "UI Components" section:

```markdown
5. **Activity status** — "Active" / "Idle Xm Ys" in StateIndicator. Toggle: Settings → More → "Show activity status".
```

- [ ] **Step 2: Update UX-FLOW.md**

Add section for computer time flow:

```markdown
## Computer Time (Unified Cycle)

Standing phase: when `continuous_computer_secs >= max_continuous_computer_secs`:
- One toast: random nudge from `screen_break_nudge.messages` pool
- No tray/overlay change (reserved for sitting/standing limits)
- Fires once per computer session (resets on 5+ min Away)

Activity status in StateIndicator:
- idle < 30s → "Active" (subtle gray)
- idle >= 30s → "Idle Xm Ys" (yellow text)
- Away state → no desk height shown
```

- [ ] **Step 3: Update PROJECT.xml if needed**

Add new Rust module `screen_break_nudge.rs` to the project map. Add new IPC fields.

- [ ] **Step 4: Commit docs**

```bash
git add CLAUDE.md .arch/ PROJECT.xml
git commit -m "docs(desk): update docs for unified work cycle and activity status"
```
