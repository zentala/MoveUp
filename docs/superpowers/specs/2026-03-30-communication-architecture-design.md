# Communication Architecture & Profile System — Design Spec

## Problem Statement

SmartDesk has multiple UI channels (tray icon, overlay bar, popup window, notifications) that communicate with the user, but there is no unified communication policy. Colors mean different things in different places:

- Tray shows green when sitting (= "within limit"), but timeline shows sitting as red
- Standing is gold in tray, but gold ≈ orange ≈ warning yellow — visually confusing
- Sensor disconnected shows gray (same as Away state — indistinguishable)
- No blinking capability exists — tray icon is always static
- Popup header shows green "Sitting" while overlay bar shows red — contradictory signals

Communication decisions are scattered across `tray_controller.rs`, `alert_manager.rs`, `notification_service.rs`, `colors.rs`, and `tray_icon.rs` — each making independent decisions with no shared policy.

The user cannot glance at any single element and get a coherent answer to "do I need to do something?"

## Design Goals

1. **One color = one meaning, everywhere** — no context-dependent color semantics
2. **Silence when OK** — don't signal anything when no action is needed
3. **Profile-driven** — all communication policy defined in swappable JSON profiles
4. **Experimentation-first** — edit JSON, see changes in 1 second, no recompilation
5. **Coordinated** — tray, overlay, popup, and notifications speak the same language simultaneously
6. **Single source of truth** — one module (CommunicationPolicy) makes ALL signal decisions

## Color Dictionary (global, immutable — profiles do NOT change these)

| Color | Hex | Meaning | Rule |
|-------|-----|---------|------|
| **None** | — | No action needed | Default state for all elements |
| **Yellow** | `#ffc107` | Heads-up: change position soon | Warning, not urgent |
| **Red** | `#f44336` | Change position now | Action required |
| **Gray** | `#808080` | System issue (sensor disconnected, loading) | Infrastructure, not ergonomics |

**Green is removed from the system.** Sitting at a computer is never "green." The app icon (desk silhouette) in the tray + the overlay bar are sufficient to confirm the app is running. Absence of colored signal = no action needed.

**Gold is removed.** Too close to yellow/warning — visually indistinguishable. Standing progress uses a neutral/subtle bar instead.

### What changed

| Color | Old meaning | New meaning |
|-------|------------|-------------|
| Green `#4caf50` | "Sitting within limit" / "Standing OK" (timeline) | **Removed** |
| Gold `#DAA520` | "Standing" | **Removed** (too close to yellow/warning) |
| Yellow `#ffc107` | "Approaching sitting limit" | "Approaching ANY limit" (sitting or standing) |
| Red `#f44336` | "Exceeded sitting limit" | "Exceeded ANY limit" |
| Gray `#808080` | "Away" or "Walking" | "Sensor disconnected / system issue" |

## Two Profile Types

The current `AppConfig` (119 fields in one struct) mixes concerns. We split it into two independent profile types, each swappable and editable as JSON.

### Why two profiles, not one

A user might want aggressive communication (early warnings, blinking) but relaxed scoring (low penalties). Or strict scoring with gentle communication (no popups, only overlay). These are independent axes:

- **Ergonomic Profile** = HOW we evaluate (limits, scoring rules, KPI thresholds)
- **Communication Profile** = HOW we inform (escalation timing, channels, patterns, messages)

### What stays outside profiles

| Setting | Where | Why not in a profile |
|---------|-------|---------------------|
| Sensor calibration (`sitting_mm`, `standing_mm`, `desk_thickness_mm`) | AppConfig (tauri-plugin-store) | Hardware-specific, depends on the desk |
| Active widget (`active_widget`) | AppConfig | UI layout preference |
| Telemetry (`telemetry_enabled`, `telemetry_device_id`) | AppConfig | Privacy/system setting |
| Welcome popup (`show_welcome_on_startup`) | AppConfig | One-time flag |
| Notification backend (`notification_backend`) | Communication Profile | Moved into profile — it's a communication preference |

### Ergonomic Profile schema

```jsonc
{
  "id": "standard",
  "name": "Standard Ergonomic",
  "description": "Balanced sit/stand limits with moderate scoring",

  // === Position limits ===
  "limits": {
    "sitting_secs": 2400,           // 40 minutes
    "standing_secs": 1200,          // 20 minutes
    "standing_target_secs": 900,    // 15 min = one "lap" (gold bar fills)
    "standing_max_secs": 5400       // 90 min ceiling before "sit down" nudge
  },

  // === Scoring ===
  "scoring": {
    "pts_standing_per_min": 1.0,    // points per minute of standing
    "pts_session_bonus": 5.0,       // bonus when completing a standing lap
    "pts_sitting_per_min": -0.5     // penalty per minute of sitting
  },

  // === KPI thresholds ===
  "kpi": {
    "standing_green_pct": 15.0,     // standing% >= 15% = green badge
    "standing_yellow_pct": 10.0,    // standing% 10-15% = yellow badge
    "changes_green": 1.0,           // position changes/h >= 1.0 = green
    "changes_yellow": 0.5,          // 0.5-1.0 = yellow
    "break_yellow_missed": 2,       // 2 missed hourly breaks = yellow
    "break_red_missed": 3,          // 3+ = red
    "session_green_mins": 45,       // longest session < 45 min = green
    "session_yellow_mins": 75,      // 45-75 = yellow, 75+ = red
    "early_data_threshold_mins": 30 // don't show KPIs until 30 min of data
  }
}
```

### Communication Profile schema

```jsonc
{
  "id": "default",
  "name": "Default — Work",
  "description": "Standard ergonomic nudging for focused work sessions",

  // === Notification backend ===
  "notification_backend": "toast",  // "toast", "popup", or "both"

  // === Escalation sequences ===
  // "at" is relative to limit: -600 = 10 min BEFORE limit, 0 = at limit, +300 = 5 min AFTER
  // Steps MUST be in ascending "at" order (validated on load, duplicates rejected)
  // Each step defines what each UI channel shows at that threshold
  "escalation": {
    "sitting": [
      {
        "at": -600,
        "tray": "yellow",
        "overlay": "yellow",
        "popup_header": "yellow",
        "notify": null
      },
      {
        "at": 0,
        "tray": "red",
        "overlay": "red",
        "popup_header": "red",
        "notify": "toast"
      },
      {
        "at": 300,
        "tray": "blink_red",
        "overlay": "pulse_red",
        "popup_header": "red",
        "notify": "popup"
      }
    ],
    "standing": [
      {
        "at": -300,
        "tray": "yellow",
        "overlay": "yellow",
        "popup_header": "yellow",
        "notify": null
      },
      {
        "at": 0,
        "tray": "red",
        "overlay": "red",
        "popup_header": "red",
        "notify": "toast"
      },
      {
        "at": 300,
        "tray": "blink_red",
        "overlay": "pulse_red",
        "popup_header": "red",
        "notify": "popup"
      }
    ]
  },

  // === Snooze/dismiss behavior (absorbed from AlertManager) ===
  "snooze": {
    "durations_mins": [5, 15, 30, 60],
    "tone_shift_after_dismisses": 3
  },

  // === Disconnected sensor behavior ===
  "disconnected": {
    "tray": "blink_gray",
    "overlay": "hidden",
    "popup_header": "gray",
    "notify_once": "toast"
  },

  // === Away / Walking (no action needed) ===
  "inactive": {
    "tray": "none",
    "overlay": "hidden",
    "popup_header": "neutral",
    "notify": null
  },

  // === Baseline (within limits, no urgency) ===
  "baseline": {
    "sitting": {
      "tray": "none",
      "overlay": "neutral",
      "popup_header": "neutral"
    },
    "standing": {
      "tray": "none",
      "overlay": "progress",
      "popup_header": "neutral"
    }
  },

  // === Standing overlay: lap tracking ===
  // When standing, the overlay shows progress toward standing_target_secs.
  // Each time you reach the target, that's one "lap" — the bar resets and
  // a counter shows completed laps. This encourages longer standing sessions.
  "standing_overlay": {
    "show_lap_counter": true,
    "flash_on_lap_complete": true,
    "bar_color": "#b0a898",         // subtle neutral (--ink-secondary)
    "lap_flash_color": "#ffc107"    // brief yellow flash on lap completion
  },

  // === Timeline block colors ===
  "timeline": {
    "sitting_within_limit": "neutral",
    "sitting_over_limit": "red",
    "standing_within_limit": "subtle",
    "standing_over_limit": "red",
    "away": "gray"
  },

  // === Blink patterns (referenced by name in escalation) ===
  "blink_patterns": {
    "blink_red": {
      "color": "red",
      "on_ms": 250,
      "off_ms": 250,
      "count": 3,
      "pause_ms": 8500
    },
    "blink_gray": {
      "color": "gray",
      "on_ms": 250,
      "off_ms": 250,
      "count": 3,
      "pause_ms": 8500
    }
  },

  // === Overlay pulse patterns (referenced by name in escalation) ===
  "overlay_patterns": {
    "pulse_red": {
      "color": "red",
      "min_brightness": 0.4,
      "max_brightness": 1.0,
      "cycle_ms": 1000
    }
  },

  // === Notification messages ===
  "messages": {
    "sitting_limit_toast": "Time to change position",
    "sitting_overdue_popup": "You've been sitting for {minutes} minutes. Stand up!",
    "standing_limit_toast": "Maybe sit down for a bit",
    "standing_overdue_popup": "You've been standing for {minutes} minutes.",
    "sensor_disconnected": "Desk sensor not connected",
    "neutral_popup_messages": [
      "Time for a stretch!",
      "Your body needs a break"
    ],
    "positive_popup_messages": [
      "Even 2 min standing helps blood flow",
      "Quick stand = fresh mind"
    ]
  },

  // === Periodic notifications (absorbed from current notification_service) ===
  "periodic_notifications": {
    "inactivity_enabled": true,
    "inactivity_after_mins": 60,
    "posture_balance_enabled": true,
    "praise_halfway_enabled": true
  }
}
```

### Signal value reference

#### Tray signals

| Value | Visual |
|-------|--------|
| `"none"` | No dot visible — desk icon only |
| `"yellow"` | Solid yellow dot |
| `"red"` | Solid red dot |
| `"blink_red"` | Red dot blinks per pattern from `blink_patterns` |
| `"blink_gray"` | Gray dot blinks per pattern from `blink_patterns` |

#### Overlay signals

| Value | Visual |
|-------|--------|
| `"hidden"` | Bar not visible |
| `"neutral"` | Dark/subtle bar (`#2c2920`) showing time progress — visible but not attention-grabbing |
| `"progress"` | Neutral bar showing standing lap progress + lap counter |
| `"yellow"` | Yellow bar |
| `"red"` | Red bar |
| `"pulse_red"` | Red bar pulsing per pattern from `overlay_patterns` |

#### Popup header signals

| Value | Visual |
|-------|--------|
| `"neutral"` | State text without color emphasis (e.g., "Sitting (12:34)") |
| `"yellow"` | State text + dot in yellow |
| `"red"` | State text + dot in red |
| `"gray"` | State text in gray (disconnected) |

#### Notify signals

| Value | Behavior |
|-------|----------|
| `null` | No notification |
| `"toast"` | System toast notification |
| `"popup"` | In-app popup window with dismiss/snooze |

### Built-in profiles

#### Communication profiles

| Profile | Philosophy | Use case |
|---------|-----------|----------|
| `default` | Yellow 10 min before, red at limit, blink 5 min after | Normal focused work |
| `aggressive` | Yellow 15 min before, blink immediately at limit | Strong nudging |
| `gentle` | Yellow 5 min before, no blink, only toast at limit | Soft reminders |
| `silent` | Overlay only, no tray signals, no notifications | Gaming, calls, presentations |
| `demo` | Same as default but designed for short limits | Development, demos |

#### Ergonomic profiles

| Profile | Philosophy | Use case |
|---------|-----------|----------|
| `standard` | 40 min sit / 20 min stand, moderate scoring | Balanced |
| `strict` | 25 min sit / 15 min stand, harsh penalties | Maximum movement |
| `relaxed` | 60 min sit / 30 min stand, low penalties | Long focus sessions |
| `demo` | 2 min sit / 1 min stand, fast scoring | Testing, demos |

### Profile storage

```
{app_data_dir}/profiles/
├── communication/
│   ├── default.json
│   ├── aggressive.json
│   ├── gentle.json
│   ├── silent.json
│   ├── demo.json
│   └── custom-*.json         # user-created
├── ergonomic/
│   ├── standard.json
│   ├── strict.json
│   ├── relaxed.json
│   ├── demo.json
│   └── custom-*.json         # user-created
```

On first launch, built-in profiles are copied to `{app_data_dir}/profiles/`. User can edit or create new ones.

### Profile switching

- **Settings UI**: two dropdowns (one for each profile type) listing JSON files from their directory
- **Hot reload**: active profiles are watched for file changes — edit JSON, app picks it up within 1 second. The profile is parsed once on load/change and stored as a Rust struct in memory. No JSON parsing happens in the hot path (every-second evaluation loop).
- **Active profiles stored in**: `tauri-plugin-store` (persists across restarts)
- **No restart required**: switching profiles takes effect immediately
- **Changing a profile replaces the entire in-memory struct at once** — no partial updates, no race conditions

### Profile validation

On load, validate:
- JSON syntax valid
- All required fields present (missing optional fields → merge with built-in defaults)
- Escalation `"at"` values strictly ascending (no duplicates)
- Blink/overlay pattern names referenced in escalation must exist in `blink_patterns`/`overlay_patterns`
- Numeric values in valid ranges

Invalid profile → log warning with specific error, keep the previously loaded profile. If no previous profile exists (first launch), use a hard-coded fallback compiled into the binary — never depend on the filesystem for the last-resort profile.

## Signal Matrix (for default profiles)

### Sitting (limit: 40 min from standard ergonomic profile)

| Phase | Time | Tray | Overlay | Popup header | Notification |
|-------|------|------|---------|--------------|--------------|
| Baseline | 0–30 min | nothing | neutral bar | "Sitting (12:34)" | — |
| Warning | 30–40 min | yellow dot | yellow bar | "Sitting" yellow | — |
| Limit | 40–45 min | red dot | red bar | "Sitting" red | toast |
| Overdue | 45+ min | red blink (3×/10s) | red pulsing | "Sitting" red | popup |

### Standing (limit: 20 min from standard ergonomic profile)

| Phase | Time | Tray | Overlay | Popup header | Notification |
|-------|------|------|---------|--------------|--------------|
| Baseline | 0–15 min | nothing | neutral progress bar + lap counter | "Standing (05:32)" | — |
| Warning | 15–20 min | yellow dot | yellow bar | "Standing" yellow | — |
| Limit | 20–25 min | red dot | red bar | "Standing" red | toast |
| Overdue | 25+ min | red blink (3×/10s) | red pulsing | "Standing" red | popup |

### Away / Walking

| Phase | Tray | Overlay | Popup header | Notification |
|-------|------|---------|--------------|--------------|
| Always | nothing | hidden | "Away" neutral | — |

### Sensor disconnected

| Phase | Tray | Overlay | Popup header | Notification |
|-------|------|---------|--------------|--------------|
| On disconnect | gray blink (3×/10s) | hidden | "Sensor disconnected" gray | toast (once) |

## Architecture

### Current flow (scattered decisions)

```
serial.rs → session_manager → emit("desk:state-changed")
                                      ↓
                 ┌────────────────────┼──────────────────────────┐
                 ↓                    ↓                          ↓
    tray_controller.rs        alert_manager.rs       notification_service.rs
    (decides tray color)      (decides escalation)   (decides notifications)
    (decides overlay color)   (decides popup/snooze)
    (decides overlay variant)
```

Each module independently computes what to show. Colors are hardcoded across `tray.rs`, `colors.rs`, `tray_icon.rs`, `tray_controller.rs`. Alert thresholds are hardcoded in `alert_config.rs`. No coordination — signals can contradict each other.

### New flow (centralized policy)

```
serial.rs → session_manager → emit("desk:state-changed") / emit("desk:distance")
                                      ↓
                            CommunicationPolicy (new module)
                              ├── reads active Communication Profile
                              ├── reads active Ergonomic Profile (for limits)
                              ├── owns escalation state (snooze, dismiss — absorbed from AlertManager)
                              ├── evaluate(state, elapsed_secs) → Signals
                              │     Signals {
                              │       tray: TraySignal,
                              │       overlay: OverlaySignal,
                              │       popup: PopupSignal,
                              │       notify: Option<NotifySignal>,
                              │     }
                              ↓
                    tray_controller.rs (executes Signals — NO decisions)
                       ├── tray.rs → set icon per TraySignal
                       │     └── tray_blink.rs → blinking engine
                       ├── overlay_renderer → set color/variant per OverlaySignal
                       │     └── standing lap tracking per standing_overlay config
                       ├── emit("desk:popup-signal") → React frontend
                       └── notification_service → fire per NotifySignal
```

### Key principles

1. **CommunicationPolicy decides. TrayController executes.** No color logic anywhere except CommunicationPolicy.
2. **AlertManager is absorbed into CommunicationPolicy.** Snooze timers, dismiss counters, stage transitions — all move into CommunicationPolicy. One source of truth for all communication decisions. The escalation array in the profile IS the state machine definition.
3. **Ergonomic Profile feeds SessionManager.** Limits, scoring, KPI thresholds come from the ergonomic profile instead of AppConfig.
4. **Profile is parsed once, evaluated from memory.** JSON is parsed on load and on file change. The every-second evaluation reads a pre-parsed Rust struct — zero parsing overhead.

### New files

| File | Responsibility |
|------|---------------|
| `communication_policy.rs` | Central brain: load profile, evaluate state → Signals, manage snooze/dismiss state |
| `communication_types.rs` | Signal enums (TraySignal, OverlaySignal, PopupSignal, NotifySignal) |
| `profile_loader.rs` | Read/validate/watch JSON profiles from disk, hot reload, fallback logic |
| `tray_blink.rs` | Tray icon dot blinking engine (timer-based pattern playback) |
| `ergonomic_profile.rs` | Ergonomic profile struct + loader (limits, scoring, KPI thresholds) |

### Deleted files

| File | Reason |
|------|--------|
| `alert_manager.rs` | Absorbed into `communication_policy.rs` |
| `alert_config.rs` | Absorbed into communication profile JSON |
| `alert_actions.rs` | Replaced by signal enums in `communication_types.rs` |
| `tray_icon.rs` | Icon selection now driven by TraySignal, logic moves to `tray.rs` |

### Modified files

| File | Change |
|------|--------|
| `tray_controller.rs` | Remove all color/escalation decisions; call `CommunicationPolicy::evaluate()`, execute returned Signals |
| `tray.rs` | Add `set_dot_visible(bool)` for blink support; remove color selection logic; accept TraySignal |
| `overlay_renderer.rs` | Accept OverlaySignal enum instead of raw (r,g,b) values |
| `colors.rs` | Simplify to 4 hex constants (yellow, red, gray, neutral) |
| `config.rs` | Remove fields migrated to profiles (limits, scoring, KPI thresholds, notification prefs); keep hardware/UI/telemetry |
| `session_manager.rs` | Read limits from ergonomic profile instead of AppConfig |
| `serial_periodic.rs` | Add profile watcher poll; remove direct notification condition checks (moved to CommunicationPolicy) |
| `notification_service.rs` | Simplify to pure executor — receives NotifySignal, fires notification. No decision logic. |
| `lib.rs` | Register new modules, remove AlertManager initialization |
| `src/utils/colors.ts` | Remove green/gold, add neutral |
| `OneBarWidget.tsx` | Read PopupSignal from backend event instead of computing color locally |
| `OneBarTimeline.tsx` | Read timeline colors from profile config (via IPC) |
| `KpiStrip.tsx` | Read KPI thresholds from ergonomic profile (via IPC) instead of AppConfig |
| `globals.css` | Remove `--signal-ok` green, add `--signal-neutral` |
| Settings UI components | Replace individual sliders with two profile dropdowns |

### Tray blink engine

```rust
/// Manages tray icon dot blinking.
///
/// Pattern: [on_ms, off_ms] × count, then pause_ms, repeat.
/// Example "3×/10s": 250ms on, 250ms off, ×3, then 8500ms pause.
///
/// Uses its own timer (separate from overlay render loop) to avoid
/// coupling blink timing to overlay render performance.
///
/// State transitions (e.g., sitting→standing) immediately call stop(),
/// cancelling any active blink pattern.
pub struct TrayBlinker {
    active: bool,
    pattern: BlinkPattern,
    phase: BlinkPhase,       // On | Off | Pause
    step: usize,             // which blink in the burst (0..count)
}

impl TrayBlinker {
    pub fn start(&mut self, pattern: BlinkPattern);
    pub fn stop(&mut self);  // immediately stops, dot becomes hidden
    pub fn tick(&mut self, elapsed_ms: u64) -> bool;  // returns: should dot be visible?
}
```

Uses an injectable clock trait for testability:
```rust
trait Clock {
    fn now(&self) -> Instant;
}
```

Tests use `MockClock` to advance time without `thread::sleep`.

### Standing overlay lap tracking

Standing overlay keeps its existing visual behavior (progress bar + lap counter + flash on completion) but is now **configured by the communication profile** instead of hardcoded:

- `standing_overlay.show_lap_counter` — whether to show "Lap 2/3" text
- `standing_overlay.flash_on_lap_complete` — whether to flash the bar on lap completion
- `standing_overlay.bar_color` — color for the progress bar (default: neutral/subtle)
- `standing_overlay.lap_flash_color` — color for the completion flash

The lap duration itself (`standing_target_secs`) comes from the **ergonomic profile**, not the communication profile — it's a limit, not a visual preference.

## KPI badges

KPI badge coloring (green/yellow/red per metric) continues to work as today, but thresholds move from `AppConfig` to the **ergonomic profile**. The communication profile does NOT control KPI colors — they are metric evaluations ("how is your day overall?"), not position signals ("what should you do right now?").

## Settings UI changes

### New: Two profile selectors

```
┌─ Ergonomic Profile ────────────────┐
│ [Standard Ergonomic      ▾]       │
│ Balanced sit/stand limits          │
│ [Edit JSON]  [Duplicate]  [Reset]  │
└────────────────────────────────────┘

┌─ Communication Profile ────────────┐
│ [Default — Work          ▾]       │
│ Standard nudging for focused work  │
│ [Edit JSON]  [Duplicate]  [Reset]  │
└────────────────────────────────────┘
```

- **Dropdown**: lists all `.json` files from the profile type's directory
- **Edit JSON**: opens profile file in system default editor
- **Duplicate**: copies current profile as `custom-{name}.json`
- **Reset**: restores built-in profile to factory defaults

### Removed from Settings

Individual sliders for sitting limit, standing limit, scoring weights, KPI thresholds — all moved to ergonomic profile JSON. Notification on/off toggles — moved to communication profile JSON.

### Kept in Settings (AppConfig)

- Sensor calibration (sitting_mm, standing_mm, desk_thickness_mm)
- Active widget layout
- Welcome popup toggle
- Telemetry settings
- Debug tab

## Execution: Two Waves

### Wave 1 — Profile Engine (delivers experimentation capability)

1. Define profile structs + signal enums (`communication_types.rs`, `ergonomic_profile.rs`)
2. Implement `profile_loader.rs` (read, validate, watch, hot reload, hard-coded fallback)
3. Implement `CommunicationPolicy::evaluate()` — absorb AlertManager logic (escalation, snooze, dismiss)
4. Wire `tray_controller.rs` to call CommunicationPolicy instead of making its own decisions
5. Wire `session_manager.rs` to read limits from ergonomic profile
6. Create `default.json` profiles (both types) — externalize current behavior
7. Settings UI: two profile dropdowns
8. Migrate `AppConfig`: remove fields that moved to profiles
9. Delete `alert_manager.rs`, `alert_config.rs`, `alert_actions.rs`
10. Update/rewrite affected tests

**After Wave 1:** you can switch between profiles, edit JSON and see changes in 1s, experiment with thresholds and escalation without recompiling. All existing behavior preserved, just externalized.

### Wave 2 — Enhanced Signals (adds blinking + color cleanup)

1. Implement `tray_blink.rs` (blinking engine with its own timer)
2. Remove green from color system — tray baseline becomes "no dot"
3. Remove gold — standing overlay becomes neutral/subtle
4. Add neutral overlay bar style
5. Create additional built-in profiles (aggressive, gentle, silent, strict, relaxed, demo)
6. Wire timeline colors from communication profile
7. Wire KPI thresholds from ergonomic profile
8. Frontend color system cleanup (globals.css, colors.ts)

**After Wave 2:** full new communication architecture with blinking, unified colors, and multiple profile options.

## Migration path

### On first launch after update

1. Read current `AppConfig` values
2. Generate `custom-migrated.json` ergonomic profile (limits, scoring, KPI thresholds)
3. Generate `custom-migrated.json` communication profile (notification prefs, current alert behavior)
4. Set both as active profiles
5. Copy built-in profiles to profiles directory
6. Slim down `AppConfig` to hardware/UI/telemetry only

### Backward compatibility

None needed — pre-release app with one user. Clean break.

## What is NOT in scope

- **Profile editor UI** (visual form) — just edit JSON and hot-reload
- **Per-application profiles** (auto-switch when gaming) — future, needs activity detection
- **Scheduled profiles** (work hours vs evening) — future
- **Profile sharing/import/export UI** — just copy JSON files
- **Overlay visual redesign** — only color source changes, not rendering engine
- **Popup layout changes** — only header color source changes
- **Green color** — deliberately removed, may reconsider after testing

## Review Feedback Incorporated

### From CEO Review
- Two-wave execution (Wave 1 = engine, Wave 2 = polish) — accepted
- DeskConfig continues to exist for non-communication settings — documented explicitly
- Green removal in Wave 1 (not deferred) — the desk icon + overlay confirm app is running

### From Eng Review
- AlertManager absorbed into CommunicationPolicy (Option A) — user explicitly requested single source of truth
- Standing overlay lap tracking included in CommunicationPolicy (configured by profile, not excluded)
- Hard-coded fallback profile compiled into binary — added to validation section
- Escalation `"at"` values must be strictly ascending — added to validation
- Blink timer separate from overlay render loop — documented in TrayBlinker section
- State transitions immediately stop active blinks — documented
- Profile parsed once, stored as Rust struct — no JSON in hot path — documented
- Injectable Clock trait for testable blink timing — documented
- Test rewrite scope: ~30-50 tests need rewriting out of 363 — tracked in wave planning
