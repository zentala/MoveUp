# Communication Architecture & Profile System — Design Spec

## Problem Statement

SmartDesk has multiple UI channels (tray icon, overlay bar, popup window, notifications) that communicate with the user, but there is no unified communication policy. Colors mean different things in different places:

- Tray shows green when sitting (= "within limit"), but timeline shows sitting as red
- Standing is gold in tray, but gold ≈ orange ≈ warning yellow — visually confusing
- Sensor disconnected shows gray (same as Away state — indistinguishable)
- No blinking capability exists — tray icon is always static
- Popup header shows green "Sitting" while overlay bar shows red — contradictory signals

The user cannot glance at any single element and get a coherent answer to "do I need to do something?"

## Design Goals

1. **One color = one meaning, everywhere** — no context-dependent color semantics
2. **Silence when OK** — don't signal anything when no action is needed
3. **Profile-driven** — all communication policy (thresholds, channels, patterns) defined in swappable JSON profiles
4. **Experimentation-first** — easy to try different approaches without code changes
5. **Coordinated** — tray, overlay, popup, and notifications speak the same language simultaneously

## Color Dictionary (global, immutable — profiles do NOT change these)

| Color | Hex | Meaning | Rule |
|-------|-----|---------|------|
| **None** | — | No action needed | Default state for all elements |
| **Yellow** | `#ffc107` | Heads-up: change position soon | Warning, not urgent |
| **Red** | `#f44336` | Change position now | Action required |
| **Gray** | `#808080` | System issue (sensor disconnected, loading) | Infrastructure, not ergonomics |

**Green is removed from the system.** Sitting at a computer is never "green." Absence of signal = OK.

### What changed

| Color | Old meaning | New meaning |
|-------|------------|-------------|
| Green `#4caf50` | "Sitting within limit" / "Standing" (timeline) | **Removed** |
| Gold `#DAA520` | "Standing" | **Removed** (too close to yellow/warning) |
| Yellow `#ffc107` | "Approaching sitting limit" | "Approaching ANY limit" (sitting or standing) |
| Red `#f44336` | "Exceeded sitting limit" | "Exceeded ANY limit" |
| Gray `#808080` | "Away" or "Walking" | "Sensor disconnected / system issue" |

## Profile System

### What a profile controls

A profile defines the **communication policy** — when each UI channel activates and with what signal. It does NOT define what colors mean (that's the global dictionary above).

### Profile schema

```jsonc
{
  // === Identity ===
  "id": "default",
  "name": "Default — Work",
  "description": "Standard ergonomic nudging for focused work sessions",

  // === Position limits (seconds) ===
  "limits": {
    "sitting_secs": 2400,           // 40 minutes
    "standing_secs": 1200           // 20 minutes
  },

  // === Escalation sequences ===
  // "at" is relative to limit: -600 = 10 min BEFORE limit, 0 = at limit, +300 = 5 min AFTER
  // Each step defines what each channel does at that moment
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
      "overlay": "gold_progress",
      "popup_header": "neutral"
    }
  },

  // === Timeline block colors ===
  "timeline": {
    "sitting_within_limit": "neutral",
    "sitting_over_limit": "red",
    "standing_within_limit": "gold",
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

  // === Overlay pulse (referenced by name in escalation) ===
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
    "sensor_disconnected": "Desk sensor not connected"
  }
}
```

### Tray signal values

| Value | Visual |
|-------|--------|
| `"none"` | No dot visible — icon only (desk silhouette) |
| `"yellow"` | Solid yellow dot |
| `"red"` | Solid red dot |
| `"blink_red"` | References blink pattern by name from `blink_patterns` |
| `"blink_gray"` | References blink pattern by name from `blink_patterns` |

### Overlay signal values

| Value | Visual |
|-------|--------|
| `"hidden"` | Bar not visible |
| `"neutral"` | Dark/subtle bar (e.g., `#2c2920` panel-overlay) showing time progress without color urgency — visible but not attention-grabbing |
| `"yellow"` | Yellow bar |
| `"red"` | Red bar |
| `"gold_progress"` | Gold bar showing standing progress (current behavior) |
| `"pulse_red"` | References overlay pattern by name from `overlay_patterns` |

### Popup header signal values

| Value | Visual |
|-------|--------|
| `"neutral"` | State text without color emphasis (e.g., "Sitting (12:34)") |
| `"yellow"` | State text + dot in yellow |
| `"red"` | State text + dot in red |
| `"gray"` | State text in gray (disconnected) |

### Notify values

| Value | Behavior |
|-------|----------|
| `null` | No notification |
| `"toast"` | System toast notification |
| `"popup"` | In-app popup window with dismiss/snooze |

### Built-in profiles

| Profile | Philosophy | Use case |
|---------|-----------|----------|
| `default` | As described above — yellow 10 min before, red at limit, blink 5 min after | Normal focused work |
| `aggressive` | Yellow 15 min before, blink immediately at limit | User wants strong nudging |
| `gentle` | Yellow 5 min before, no blink, only toast at limit | Soft reminders |
| `silent` | Overlay only, no tray signals, no notifications | Gaming, calls, presentations |
| `demo` | Short limits (2 min sit, 1 min stand) for testing/demo | Development, demos |

### Profile storage

```
{app_data_dir}/profiles/
├── default.json          # shipped with app, overwritable
├── aggressive.json       # shipped with app
├── gentle.json           # shipped with app
├── silent.json           # shipped with app
├── demo.json             # shipped with app
└── custom-*.json         # user-created profiles
```

On first launch, built-in profiles are copied to `{app_data_dir}/profiles/`. User can edit or create new ones.

### Profile switching

- **Settings UI**: dropdown listing all profiles from the profiles directory
- **Hot reload**: active profile is watched for changes — edit JSON, app picks it up within 1 second
- **Active profile stored in**: `tauri-plugin-store` (persists across restarts)
- **No restart required**: switching profiles takes effect immediately

### Profile validation

On load, validate against schema. Invalid profile → log warning, fall back to `default.json`. Missing fields → use defaults from built-in `default.json` (merge strategy).

## Signal Matrix (for default profile)

### Sitting

| Phase | Time | Tray | Overlay | Popup header | Notification |
|-------|------|------|---------|--------------|--------------|
| Baseline | 0–30 min | nothing | neutral bar | "Sitting (12:34)" | — |
| Warning | 30–40 min | yellow dot | yellow bar | "Sitting" yellow | — |
| Limit | 40–45 min | red dot | red bar | "Sitting" red | toast |
| Overdue | 45+ min | red blink (3×/10s) | red pulsing | "Sitting" red | popup |

### Standing

| Phase | Time | Tray | Overlay | Popup header | Notification |
|-------|------|------|---------|--------------|--------------|
| Baseline | 0–15 min | nothing | gold progress bar | "Standing (05:32)" | — |
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

## Architecture: CommunicationPolicy module

### Current flow (scattered decisions)

```
serial.rs → session_manager → emit("desk:state-changed")
                                      ↓
                            tray_controller.rs (decides tray color)
                            tray_controller.rs (decides overlay color)
                            alert_manager.rs (decides escalation)
                            notification_service.rs (decides notifications)
```

Each module independently computes what to show. Colors are hardcoded across `tray.rs`, `colors.rs`, `tray_icon.rs`, `tray_controller.rs`.

### New flow (centralized policy)

```
serial.rs → session_manager → emit("desk:state-changed")
                                      ↓
                            CommunicationPolicy (new module)
                              ├── loads active profile JSON
                              ├── evaluate(state, elapsed_secs, limits) → Signals
                              │     Signals {
                              │       tray: TraySignal,      // None | Yellow | Red | Blink(pattern)
                              │       overlay: OverlaySignal, // Hidden | Neutral | Yellow | Red | Gold | Pulse(pattern)
                              │       popup: PopupSignal,     // Neutral | Yellow | Red | Gray
                              │       notify: Option<NotifySignal>,  // None | Toast(msg) | Popup(msg)
                              │     }
                              ↓
                    tray_controller.rs (executes Signals — no decisions)
                       ├── tray.rs (set icon per TraySignal)
                       ├── overlay (set color/variant per OverlaySignal)
                       ├── popup (emit header signal per PopupSignal)
                       └── notification_service (fire per NotifySignal)
```

### Key principle

**CommunicationPolicy decides. TrayController executes.** No color logic in tray_controller, tray.rs, or colors.rs. All decisions flow from the profile.

### New files

| File | Responsibility |
|------|---------------|
| `communication_policy.rs` | Load profile, evaluate state → Signals |
| `communication_types.rs` | Signal enums (TraySignal, OverlaySignal, etc.) |
| `profile_loader.rs` | Read/validate/watch JSON profiles, hot reload |
| `tray_blink.rs` | Blink engine for tray icon (timer-based show/hide dot) |

### Modified files

| File | Change |
|------|--------|
| `tray_controller.rs` | Remove color decisions, call `CommunicationPolicy::evaluate()` instead |
| `tray.rs` | Add `set_dot_visible(bool)` for blink support, remove color logic |
| `overlay_renderer.rs` | Accept signal enum instead of raw color values |
| `alert_manager.rs` | Remove hardcoded thresholds — read from profile via CommunicationPolicy |
| `colors.rs` | Simplify to just hex constants for the 4 dictionary colors |
| `src/utils/colors.ts` | Mirror Rust changes — remove green, add neutral |
| `OneBarWidget.tsx` | Read popup signal from backend instead of computing color |
| `OneBarTimeline.tsx` | Use profile timeline config for block colors |
| `KpiStrip.tsx` | Keep existing logic (KPI colors are metric-based, not position-based) |
| `globals.css` | Remove `--signal-ok` green, add `--signal-neutral` |

### Tray blink engine

```rust
/// Manages tray icon dot blinking via a Windows timer.
///
/// Pattern: [on_ms, off_ms] × count, then pause_ms, repeat.
/// Example "3×/10s": 250ms on, 250ms off, ×3, then 8500ms pause.
pub struct TrayBlinker {
    active: bool,
    pattern: BlinkPattern,
    phase: BlinkPhase,       // On | Off | Pause
    step: usize,             // which blink in the burst (0..count)
    timer_handle: Option<()>,
}

impl TrayBlinker {
    pub fn start(&mut self, pattern: BlinkPattern);
    pub fn stop(&mut self);  // → dot hidden (none state)
    pub fn tick(&mut self) -> bool;  // returns: should dot be visible?
}
```

Runs on same timer as overlay (~16ms), checks phase transitions.

### Profile hot reload

```rust
/// Watches active profile file for changes.
/// On change: re-parse, validate, swap CommunicationPolicy.
/// Invalid file → keep previous profile, log warning.
pub struct ProfileWatcher {
    path: PathBuf,
    last_modified: SystemTime,
    poll_interval_ms: u64,    // 1000ms
}
```

Checked every 1s in the existing periodic loop (`serial_periodic.rs`).

## KPI badges — no change

KPI badges (standing%, changes/h, breaks, screen time) keep their own green/yellow/red coloring. These are **metric evaluations**, not position signals — they answer "how is your day going overall?" not "what should you do right now?" This is a separate concern from the communication policy.

If in the future we want profiles to control KPI thresholds too, we extend the profile schema. Not now.

## Settings UI changes

### New: Profile selector

```
┌─ Profile ──────────────────────────┐
│ [Default — Work          ▾]       │
│                                    │
│ Description: Standard ergonomic    │
│ nudging for focused work sessions  │
│                                    │
│ [Edit JSON]  [Duplicate]  [Reset]  │
└────────────────────────────────────┘
```

- **Dropdown**: lists all `.json` files from profiles directory
- **Edit JSON**: opens profile file in system default editor
- **Duplicate**: copies current profile as `custom-{name}.json`
- **Reset**: restores built-in profile to factory defaults

### Removed from Settings

Individual threshold sliders (sitting limit, standing limit, etc.) are removed — these are now controlled by the active profile. The profile JSON is the single source of truth.

### Kept in Settings

- Sensor settings (COM port, calibration)
- Remote display settings
- Debug tab
- Notification backend (toast/popup/both) — this stays separate from profile because it's a system capability, not an ergonomic preference

## Migration path

### On first launch after update

1. Read current settings (sitting_limit, standing_limit, notification preferences)
2. Generate `custom-migrated.json` profile with those values
3. Set as active profile
4. Built-in profiles are also copied to profiles directory
5. User sees notification: "Your settings have been migrated to a profile"

### Backward compatibility

None needed — this is a pre-release app with one user. Clean break.

## What is NOT in scope

- **Profile sharing/import/export UI** — just copy JSON files manually
- **Per-application profiles** (auto-switch when gaming) — future feature, needs activity detection
- **Scheduled profiles** (aggressive during work hours, gentle evenings) — future feature
- **KPI threshold customization via profiles** — KPI badges keep independent logic for now
- **Green color** — deliberately removed, may reconsider after testing
- **Overlay visual redesign** — only color source changes, not rendering
- **Popup layout changes** — only header color source changes
