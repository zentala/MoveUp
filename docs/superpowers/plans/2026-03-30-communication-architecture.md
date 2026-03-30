# Communication Architecture & Profile System — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace scattered color/signal decisions with a centralized CommunicationPolicy module driven by two swappable JSON profiles (ergonomic + communication), enabling fast experimentation without code changes.

**Architecture:** CommunicationPolicy reads two JSON profiles and evaluates `(state, elapsed_secs) → Signals`. TrayController becomes a pure executor. AlertManager is absorbed. Profile hot-reload enables edit-JSON-see-result-in-1s workflow.

**Tech Stack:** Rust (Tauri 2), serde_json, React/TypeScript, tauri-plugin-store

**Spec:** `docs/superpowers/specs/2026-03-30-communication-architecture-design.md`

---

## Wave 1 — Profile Engine (delivers experimentation capability)

### Task 1: Define signal types

**Files:**
- Create: `src-tauri/src/communication_types.rs`
- Modify: `src-tauri/src/lib.rs` (add module declaration)

- [ ] **Step 1: Create signal enums**

```rust
//! communication_types.rs — Signal enums for the communication architecture.
//!
//! CommunicationPolicy evaluates state and returns these signals.
//! TrayController executes them without making decisions.

use serde::{Deserialize, Serialize};

/// What the tray icon dot should show.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TraySignal {
    /// No dot visible — desk icon only.
    None,
    /// Solid yellow dot.
    Yellow,
    /// Solid red dot.
    Red,
    /// Blinking pattern by name (references communication profile blink_patterns).
    Blink(String),
}

/// What the overlay progress bar should show.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OverlaySignal {
    /// Bar not visible.
    Hidden,
    /// Dark/subtle bar showing time progress without color urgency.
    Neutral { progress: f32 },
    /// Standing progress bar with lap tracking.
    Progress { progress: f32, lap: u32, flash: bool },
    /// Yellow bar.
    Yellow { progress: f32 },
    /// Red bar.
    Red { progress: f32 },
    /// Red bar pulsing per pattern.
    PulseRed { progress: f32 },
}

/// What the popup header should show.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PopupSignal {
    /// State text without color emphasis.
    Neutral,
    /// State text + dot in yellow.
    Yellow,
    /// State text + dot in red.
    Red,
    /// State text in gray (disconnected).
    Gray,
}

/// What notification to fire (if any).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NotifySignal {
    /// System toast notification.
    Toast(String),
    /// In-app popup with dismiss/snooze.
    Popup(String),
}

/// Complete set of signals for all UI channels.
#[derive(Debug, Clone)]
pub struct Signals {
    pub tray: TraySignal,
    pub overlay: OverlaySignal,
    pub popup: PopupSignal,
    pub notify: Option<NotifySignal>,
}

impl Default for Signals {
    fn default() -> Self {
        Self {
            tray: TraySignal::None,
            overlay: OverlaySignal::Hidden,
            popup: PopupSignal::Neutral,
            notify: None,
        }
    }
}
```

- [ ] **Step 2: Add module to lib.rs**

Add `mod communication_types;` to module declarations in `src-tauri/src/lib.rs`.

- [ ] **Step 3: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: compiles with no errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/communication_types.rs src-tauri/src/lib.rs
git commit -m "feat(desk): add signal type enums for communication architecture"
```

---

### Task 2: Define profile structs and loader

**Files:**
- Create: `src-tauri/src/ergonomic_profile.rs`
- Create: `src-tauri/src/communication_profile.rs`
- Create: `src-tauri/src/profile_loader.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write ergonomic profile struct**

```rust
//! ergonomic_profile.rs — Ergonomic profile defining limits, scoring, and KPI thresholds.

use serde::{Deserialize, Serialize};

/// Ergonomic profile — HOW we evaluate the user's behavior.
/// Loaded from JSON, hot-reloadable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErgonomicProfile {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_limits")]
    pub limits: Limits,
    #[serde(default)]
    pub scoring: Scoring,
    #[serde(default)]
    pub kpi: KpiThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Limits {
    #[serde(default = "default_sitting_secs")]
    pub sitting_secs: u64,
    #[serde(default = "default_standing_secs")]
    pub standing_secs: u64,
    #[serde(default = "default_standing_target_secs")]
    pub standing_target_secs: u64,
    #[serde(default = "default_standing_max_secs")]
    pub standing_max_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scoring {
    #[serde(default = "default_pts_standing")]
    pub pts_standing_per_min: f32,
    #[serde(default = "default_pts_bonus")]
    pub pts_session_bonus: f32,
    #[serde(default = "default_pts_sitting")]
    pub pts_sitting_per_min: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiThresholds {
    #[serde(default = "default_standing_green")]
    pub standing_green_pct: f32,
    #[serde(default = "default_standing_yellow")]
    pub standing_yellow_pct: f32,
    #[serde(default = "default_changes_green")]
    pub changes_green: f32,
    #[serde(default = "default_changes_yellow")]
    pub changes_yellow: f32,
    #[serde(default = "default_break_yellow")]
    pub break_yellow_missed: u8,
    #[serde(default = "default_break_red")]
    pub break_red_missed: u8,
    #[serde(default = "default_session_green")]
    pub session_green_mins: u32,
    #[serde(default = "default_session_yellow")]
    pub session_yellow_mins: u32,
    #[serde(default = "default_early_data")]
    pub early_data_threshold_mins: u32,
}

// Default functions matching current AppConfig values
fn default_limits() -> Limits { Limits::default() }
fn default_sitting_secs() -> u64 { 2400 }
fn default_standing_secs() -> u64 { 1200 }
fn default_standing_target_secs() -> u64 { 900 }
fn default_standing_max_secs() -> u64 { 5400 }
fn default_pts_standing() -> f32 { 1.0 }
fn default_pts_bonus() -> f32 { 5.0 }
fn default_pts_sitting() -> f32 { -0.5 }
fn default_standing_green() -> f32 { 15.0 }
fn default_standing_yellow() -> f32 { 10.0 }
fn default_changes_green() -> f32 { 1.0 }
fn default_changes_yellow() -> f32 { 0.5 }
fn default_break_yellow() -> u8 { 2 }
fn default_break_red() -> u8 { 3 }
fn default_session_green() -> u32 { 45 }
fn default_session_yellow() -> u32 { 75 }
fn default_early_data() -> u32 { 30 }

impl Default for Limits {
    fn default() -> Self {
        Self {
            sitting_secs: default_sitting_secs(),
            standing_secs: default_standing_secs(),
            standing_target_secs: default_standing_target_secs(),
            standing_max_secs: default_standing_max_secs(),
        }
    }
}

impl Default for Scoring {
    fn default() -> Self {
        Self {
            pts_standing_per_min: default_pts_standing(),
            pts_session_bonus: default_pts_bonus(),
            pts_sitting_per_min: default_pts_sitting(),
        }
    }
}

impl Default for KpiThresholds {
    fn default() -> Self {
        Self {
            standing_green_pct: default_standing_green(),
            standing_yellow_pct: default_standing_yellow(),
            changes_green: default_changes_green(),
            changes_yellow: default_changes_yellow(),
            break_yellow_missed: default_break_yellow(),
            break_red_missed: default_break_red(),
            session_green_mins: default_session_green(),
            session_yellow_mins: default_session_yellow(),
            early_data_threshold_mins: default_early_data(),
        }
    }
}

impl Default for ErgonomicProfile {
    fn default() -> Self {
        Self {
            id: "standard".to_string(),
            name: "Standard Ergonomic".to_string(),
            description: "Balanced sit/stand limits with moderate scoring".to_string(),
            limits: Limits::default(),
            scoring: Scoring::default(),
            kpi: KpiThresholds::default(),
        }
    }
}
```

- [ ] **Step 2: Write communication profile struct**

```rust
//! communication_profile.rs — Communication profile defining escalation, patterns, and messages.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Communication profile — HOW we inform the user.
/// Loaded from JSON, hot-reloadable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationProfile {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_backend")]
    pub notification_backend: String,
    pub escalation: EscalationConfig,
    #[serde(default)]
    pub snooze: SnoozeConfig,
    #[serde(default)]
    pub disconnected: ChannelConfig,
    #[serde(default)]
    pub inactive: ChannelConfig,
    #[serde(default)]
    pub baseline: BaselineConfig,
    #[serde(default)]
    pub standing_overlay: StandingOverlayConfig,
    #[serde(default)]
    pub timeline: TimelineConfig,
    #[serde(default)]
    pub blink_patterns: HashMap<String, BlinkPattern>,
    #[serde(default)]
    pub overlay_patterns: HashMap<String, OverlayPattern>,
    #[serde(default)]
    pub messages: MessageConfig,
    #[serde(default)]
    pub periodic_notifications: PeriodicNotificationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationConfig {
    #[serde(default)]
    pub sitting: Vec<EscalationStep>,
    #[serde(default)]
    pub standing: Vec<EscalationStep>,
}

/// One step in the escalation sequence.
/// `at` is seconds relative to limit: -600 = 10 min before, 0 = at limit, +300 = 5 min after.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationStep {
    pub at: i64,
    #[serde(default = "default_none_str")]
    pub tray: String,
    #[serde(default = "default_none_str")]
    pub overlay: String,
    #[serde(default = "default_neutral_str")]
    pub popup_header: String,
    pub notify: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnoozeConfig {
    #[serde(default = "default_snooze_durations")]
    pub durations_mins: Vec<u32>,
    #[serde(default = "default_tone_shift")]
    pub tone_shift_after_dismisses: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChannelConfig {
    #[serde(default = "default_none_str")]
    pub tray: String,
    #[serde(default = "default_hidden_str")]
    pub overlay: String,
    #[serde(default = "default_neutral_str")]
    pub popup_header: String,
    pub notify: Option<String>,
    #[serde(default)]
    pub notify_once: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BaselineConfig {
    #[serde(default)]
    pub sitting: BaselineChannels,
    #[serde(default)]
    pub standing: BaselineChannels,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BaselineChannels {
    #[serde(default = "default_none_str")]
    pub tray: String,
    #[serde(default = "default_neutral_str")]
    pub overlay: String,
    #[serde(default = "default_neutral_str")]
    pub popup_header: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandingOverlayConfig {
    #[serde(default = "bool_true")]
    pub show_lap_counter: bool,
    #[serde(default = "bool_true")]
    pub flash_on_lap_complete: bool,
    #[serde(default = "default_bar_color")]
    pub bar_color: String,
    #[serde(default = "default_flash_color")]
    pub lap_flash_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimelineConfig {
    #[serde(default = "default_neutral_str")]
    pub sitting_within_limit: String,
    #[serde(default = "default_red_str")]
    pub sitting_over_limit: String,
    #[serde(default = "default_subtle_str")]
    pub standing_within_limit: String,
    #[serde(default = "default_red_str")]
    pub standing_over_limit: String,
    #[serde(default = "default_gray_str")]
    pub away: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlinkPattern {
    pub color: String,
    #[serde(default = "default_250")]
    pub on_ms: u64,
    #[serde(default = "default_250")]
    pub off_ms: u64,
    #[serde(default = "default_3")]
    pub count: u32,
    #[serde(default = "default_8500")]
    pub pause_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayPattern {
    pub color: String,
    #[serde(default = "default_04")]
    pub min_brightness: f32,
    #[serde(default = "default_10")]
    pub max_brightness: f32,
    #[serde(default = "default_1000")]
    pub cycle_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessageConfig {
    #[serde(default = "default_sitting_toast")]
    pub sitting_limit_toast: String,
    #[serde(default = "default_sitting_popup")]
    pub sitting_overdue_popup: String,
    #[serde(default = "default_standing_toast")]
    pub standing_limit_toast: String,
    #[serde(default = "default_standing_popup")]
    pub standing_overdue_popup: String,
    #[serde(default = "default_sensor_msg")]
    pub sensor_disconnected: String,
    #[serde(default)]
    pub neutral_popup_messages: Vec<String>,
    #[serde(default)]
    pub positive_popup_messages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodicNotificationConfig {
    #[serde(default = "bool_true")]
    pub inactivity_enabled: bool,
    #[serde(default = "default_60")]
    pub inactivity_after_mins: u32,
    #[serde(default = "bool_true")]
    pub posture_balance_enabled: bool,
    #[serde(default = "bool_true")]
    pub praise_halfway_enabled: bool,
}

// === Default value functions ===
fn default_backend() -> String { "toast".to_string() }
fn default_none_str() -> String { "none".to_string() }
fn default_hidden_str() -> String { "hidden".to_string() }
fn default_neutral_str() -> String { "neutral".to_string() }
fn default_red_str() -> String { "red".to_string() }
fn default_subtle_str() -> String { "subtle".to_string() }
fn default_gray_str() -> String { "gray".to_string() }
fn default_bar_color() -> String { "#b0a898".to_string() }
fn default_flash_color() -> String { "#ffc107".to_string() }
fn bool_true() -> bool { true }
fn default_snooze_durations() -> Vec<u32> { vec![5, 15, 30, 60] }
fn default_tone_shift() -> usize { 3 }
fn default_250() -> u64 { 250 }
fn default_3() -> u32 { 3 }
fn default_8500() -> u64 { 8500 }
fn default_04() -> f32 { 0.4 }
fn default_10() -> f32 { 1.0 }
fn default_1000() -> u64 { 1000 }
fn default_60() -> u32 { 60 }
fn default_sitting_toast() -> String { "Time to change position".to_string() }
fn default_sitting_popup() -> String { "You've been sitting for {minutes} minutes. Stand up!".to_string() }
fn default_standing_toast() -> String { "Maybe sit down for a bit".to_string() }
fn default_standing_popup() -> String { "You've been standing for {minutes} minutes.".to_string() }
fn default_sensor_msg() -> String { "Desk sensor not connected".to_string() }

impl Default for SnoozeConfig {
    fn default() -> Self {
        Self {
            durations_mins: default_snooze_durations(),
            tone_shift_after_dismisses: default_tone_shift(),
        }
    }
}

impl Default for StandingOverlayConfig {
    fn default() -> Self {
        Self {
            show_lap_counter: true,
            flash_on_lap_complete: true,
            bar_color: default_bar_color(),
            lap_flash_color: default_flash_color(),
        }
    }
}

impl Default for PeriodicNotificationConfig {
    fn default() -> Self {
        Self {
            inactivity_enabled: true,
            inactivity_after_mins: 60,
            posture_balance_enabled: true,
            praise_halfway_enabled: true,
        }
    }
}

impl Default for EscalationConfig {
    fn default() -> Self {
        Self {
            sitting: vec![
                EscalationStep { at: -600, tray: "yellow".into(), overlay: "yellow".into(), popup_header: "yellow".into(), notify: None },
                EscalationStep { at: 0, tray: "red".into(), overlay: "red".into(), popup_header: "red".into(), notify: Some("toast".into()) },
                EscalationStep { at: 300, tray: "blink_red".into(), overlay: "pulse_red".into(), popup_header: "red".into(), notify: Some("popup".into()) },
            ],
            standing: vec![
                EscalationStep { at: -300, tray: "yellow".into(), overlay: "yellow".into(), popup_header: "yellow".into(), notify: None },
                EscalationStep { at: 0, tray: "red".into(), overlay: "red".into(), popup_header: "red".into(), notify: Some("toast".into()) },
                EscalationStep { at: 300, tray: "blink_red".into(), overlay: "pulse_red".into(), popup_header: "red".into(), notify: Some("popup".into()) },
            ],
        }
    }
}

impl Default for CommunicationProfile {
    fn default() -> Self {
        let mut blink_patterns = HashMap::new();
        blink_patterns.insert("blink_red".to_string(), BlinkPattern {
            color: "red".to_string(), on_ms: 250, off_ms: 250, count: 3, pause_ms: 8500,
        });
        blink_patterns.insert("blink_gray".to_string(), BlinkPattern {
            color: "gray".to_string(), on_ms: 250, off_ms: 250, count: 3, pause_ms: 8500,
        });

        let mut overlay_patterns = HashMap::new();
        overlay_patterns.insert("pulse_red".to_string(), OverlayPattern {
            color: "red".to_string(), min_brightness: 0.4, max_brightness: 1.0, cycle_ms: 1000,
        });

        Self {
            id: "default".to_string(),
            name: "Default — Work".to_string(),
            description: "Standard ergonomic nudging for focused work sessions".to_string(),
            notification_backend: default_backend(),
            escalation: EscalationConfig::default(),
            snooze: SnoozeConfig::default(),
            disconnected: ChannelConfig {
                tray: "blink_gray".into(), overlay: "hidden".into(),
                popup_header: "gray".into(), notify: None, notify_once: Some("toast".into()),
            },
            inactive: ChannelConfig {
                tray: "none".into(), overlay: "hidden".into(),
                popup_header: "neutral".into(), notify: None, notify_once: None,
            },
            baseline: BaselineConfig {
                sitting: BaselineChannels { tray: "none".into(), overlay: "neutral".into(), popup_header: "neutral".into() },
                standing: BaselineChannels { tray: "none".into(), overlay: "progress".into(), popup_header: "neutral".into() },
            },
            standing_overlay: StandingOverlayConfig::default(),
            timeline: TimelineConfig::default(),
            blink_patterns,
            overlay_patterns,
            messages: MessageConfig::default(),
            periodic_notifications: PeriodicNotificationConfig::default(),
        }
    }
}
```

- [ ] **Step 3: Write profile loader with validation and hot-reload**

```rust
//! profile_loader.rs — Reads, validates, and watches JSON profile files.

use log::{info, warn};
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::communication_profile::CommunicationProfile;
use crate::ergonomic_profile::ErgonomicProfile;

/// Loads a profile from a JSON file, falling back to default on error.
pub fn load_profile<T: DeserializeOwned + Default>(path: &Path) -> T {
    match std::fs::read_to_string(path) {
        Ok(json) => match serde_json::from_str::<T>(&json) {
            Ok(profile) => profile,
            Err(e) => {
                warn!("Invalid profile {}: {} — using defaults", path.display(), e);
                T::default()
            }
        },
        Err(e) => {
            warn!("Cannot read profile {}: {} — using defaults", path.display(), e);
            T::default()
        }
    }
}

/// Validates a communication profile. Returns list of errors (empty = valid).
pub fn validate_communication_profile(p: &CommunicationProfile) -> Vec<String> {
    let mut errors = Vec::new();

    // Escalation "at" values must be strictly ascending
    for (name, steps) in [("sitting", &p.escalation.sitting), ("standing", &p.escalation.standing)] {
        for i in 1..steps.len() {
            if steps[i].at <= steps[i - 1].at {
                errors.push(format!(
                    "escalation.{}: 'at' values must be strictly ascending (step {} has at={}, step {} has at={})",
                    name, i - 1, steps[i - 1].at, i, steps[i].at
                ));
            }
        }
    }

    // Blink/overlay patterns referenced must exist
    for (name, steps) in [("sitting", &p.escalation.sitting), ("standing", &p.escalation.standing)] {
        for step in steps {
            if step.tray.starts_with("blink_") && !p.blink_patterns.contains_key(&step.tray) {
                errors.push(format!(
                    "escalation.{}: tray references undefined blink pattern '{}'", name, step.tray
                ));
            }
            if step.overlay.starts_with("pulse_") && !p.overlay_patterns.contains_key(&step.overlay) {
                errors.push(format!(
                    "escalation.{}: overlay references undefined pattern '{}'", name, step.overlay
                ));
            }
        }
    }

    errors
}

/// Watches a file for changes by polling last-modified time.
pub struct ProfileWatcher {
    path: PathBuf,
    last_modified: Option<SystemTime>,
}

impl ProfileWatcher {
    pub fn new(path: PathBuf) -> Self {
        let last_modified = std::fs::metadata(&path)
            .and_then(|m| m.modified())
            .ok();
        Self { path, last_modified }
    }

    /// Returns true if the file has changed since last check.
    pub fn has_changed(&mut self) -> bool {
        let current = std::fs::metadata(&self.path)
            .and_then(|m| m.modified())
            .ok();
        if current != self.last_modified {
            self.last_modified = current;
            true
        } else {
            false
        }
    }
}

/// Ensures profile directories exist and copies built-in profiles if missing.
pub fn ensure_profiles_dir(app_data_dir: &Path) {
    let comm_dir = app_data_dir.join("profiles").join("communication");
    let ergo_dir = app_data_dir.join("profiles").join("ergonomic");
    let _ = std::fs::create_dir_all(&comm_dir);
    let _ = std::fs::create_dir_all(&ergo_dir);

    // Write default profiles if they don't exist
    let default_comm = CommunicationProfile::default();
    let default_ergo = ErgonomicProfile::default();

    let comm_path = comm_dir.join("default.json");
    if !comm_path.exists() {
        if let Ok(json) = serde_json::to_string_pretty(&default_comm) {
            let _ = std::fs::write(&comm_path, json);
            info!("Created default communication profile at {}", comm_path.display());
        }
    }

    let ergo_path = ergo_dir.join("standard.json");
    if !ergo_path.exists() {
        if let Ok(json) = serde_json::to_string_pretty(&default_ergo) {
            let _ = std::fs::write(&ergo_path, json);
            info!("Created standard ergonomic profile at {}", ergo_path.display());
        }
    }
}

/// Lists all profile JSON files in a directory.
pub fn list_profiles(dir: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().map_or(false, |ext| ext == "json"))
                .collect()
        })
        .unwrap_or_default()
}
```

- [ ] **Step 4: Add modules to lib.rs**

Add to module declarations:
```rust
mod communication_profile;
mod ergonomic_profile;
mod profile_loader;
```

- [ ] **Step 5: Verify compilation**

Run: `cd src-tauri && cargo check`

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/ergonomic_profile.rs src-tauri/src/communication_profile.rs src-tauri/src/profile_loader.rs src-tauri/src/lib.rs
git commit -m "feat(desk): add ergonomic and communication profile structs with loader"
```

---

### Task 3: Implement CommunicationPolicy

**Files:**
- Create: `src-tauri/src/communication_policy.rs`
- Create: `src-tauri/src/communication_policy_tests.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing tests for evaluate()**

Create `src-tauri/src/communication_policy_tests.rs` with tests covering:
- Sitting baseline (within limit) → all signals None/Neutral
- Sitting warning (10 min before limit) → yellow tray + overlay
- Sitting at limit → red + toast
- Sitting overdue (+5 min) → blink_red + pulse_red + popup
- Standing baseline → overlay progress, tray none
- Standing warning → yellow
- Standing at limit → red + toast
- Disconnected → gray blink + toast once
- Away → all none/hidden
- Snooze: after dismiss, signals revert until snooze expires

Run: `cd src-tauri && cargo test communication_policy`
Expected: FAIL — module doesn't exist yet.

- [ ] **Step 2: Implement CommunicationPolicy**

```rust
//! communication_policy.rs — Central brain for all UI signal decisions.
//!
//! Reads active profiles, evaluates current state, returns Signals.
//! Absorbs AlertManager logic (escalation, snooze, dismiss).
//! TrayController calls evaluate() and executes the returned Signals.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use log::info;

use crate::communication_profile::CommunicationProfile;
use crate::communication_types::*;
use crate::ergonomic_profile::ErgonomicProfile;
use crate::session_types::DeskState;

/// Input to the policy evaluator.
pub struct PolicyInput {
    pub state: DeskState,
    pub elapsed_secs: i64,
    pub sensor_connected: bool,
    /// Standing lap progress (0.0–1.0 within current lap).
    pub standing_lap_progress: f32,
    /// Current standing lap number.
    pub standing_lap: u32,
    /// Whether lap just completed (flash trigger).
    pub standing_lap_flash: bool,
}

/// Manages communication policy state (snooze, dismiss tracking).
pub struct CommunicationPolicy {
    comm_profile: CommunicationProfile,
    ergo_profile: ErgonomicProfile,
    /// When the current snooze expires (if snoozed).
    snoozed_until: Option<Instant>,
    /// How many times the user has dismissed in current position stint.
    snooze_index: usize,
    /// Whether we already fired the "sensor disconnected" one-time notification.
    disconnect_notified: bool,
    /// Last escalation step index that fired a notification (to avoid re-firing).
    last_notify_step: Option<usize>,
}

impl CommunicationPolicy {
    pub fn new(comm: CommunicationProfile, ergo: ErgonomicProfile) -> Self {
        Self {
            comm_profile: comm,
            ergo_profile: ergo,
            snoozed_until: None,
            snooze_index: 0,
            disconnect_notified: false,
            last_notify_step: None,
        }
    }

    /// Replace profiles (hot-reload). Preserves snooze state.
    pub fn set_comm_profile(&mut self, profile: CommunicationProfile) {
        self.comm_profile = profile;
    }

    pub fn set_ergo_profile(&mut self, profile: ErgonomicProfile) {
        self.ergo_profile = profile;
    }

    pub fn comm_profile(&self) -> &CommunicationProfile {
        &self.comm_profile
    }

    pub fn ergo_profile(&self) -> &ErgonomicProfile {
        &self.ergo_profile
    }

    /// Core evaluation: given current state, return what every UI channel should show.
    pub fn evaluate(&mut self, input: &PolicyInput) -> Signals {
        // Sensor disconnected — highest priority
        if !input.sensor_connected {
            return self.evaluate_disconnected();
        }

        // Away / Walking — silence
        if matches!(input.state, DeskState::Walking | DeskState::Away) {
            return self.evaluate_inactive();
        }

        // Sitting or Standing — check escalation
        let limit_secs = match input.state {
            DeskState::Sitting => self.ergo_profile.limits.sitting_secs as i64,
            DeskState::Standing => self.ergo_profile.limits.standing_secs as i64,
            _ => return Signals::default(),
        };

        let steps = match input.state {
            DeskState::Sitting => &self.comm_profile.escalation.sitting,
            DeskState::Standing => &self.comm_profile.escalation.standing,
            _ => return Signals::default(),
        };

        // How far past the limit are we? (negative = before limit)
        let offset = input.elapsed_secs - limit_secs;

        // Check if snoozed
        if let Some(until) = self.snoozed_until {
            if Instant::now() < until {
                return self.evaluate_baseline(input);
            }
            // Snooze expired — clear and continue evaluation
            self.snoozed_until = None;
        }

        // Find the highest escalation step we've reached
        let active_step = steps.iter().enumerate().rev().find(|(_, step)| offset >= step.at);

        match active_step {
            Some((idx, step)) => self.step_to_signals(step, idx, input),
            None => self.evaluate_baseline(input),
        }
    }

    /// User dismissed the popup/notification.
    pub fn dismiss(&mut self) {
        let durations = &self.comm_profile.snooze.durations_mins;
        let idx = self.snooze_index.min(durations.len().saturating_sub(1));
        let mins = durations.get(idx).copied().unwrap_or(5);
        self.snoozed_until = Some(Instant::now() + Duration::from_secs(mins as u64 * 60));
        self.snooze_index += 1;
        self.last_notify_step = None;
        info!("Snoozed for {} minutes (dismiss #{})", mins, self.snooze_index);
    }

    /// User changed position — reset escalation state.
    pub fn on_position_changed(&mut self) {
        self.snoozed_until = None;
        self.snooze_index = 0;
        self.last_notify_step = None;
    }

    /// Sensor reconnected — clear disconnect notification flag.
    pub fn on_sensor_connected(&mut self) {
        self.disconnect_notified = false;
    }

    fn evaluate_disconnected(&mut self) -> Signals {
        let cfg = &self.comm_profile.disconnected;
        let notify = if !self.disconnect_notified {
            if let Some(ref notify_type) = cfg.notify_once {
                self.disconnect_notified = true;
                Some(self.make_notify_signal(
                    notify_type,
                    &self.comm_profile.messages.sensor_disconnected.clone(),
                ))
            } else {
                None
            }
        } else {
            None
        };

        Signals {
            tray: self.parse_tray_signal(&cfg.tray),
            overlay: OverlaySignal::Hidden,
            popup: self.parse_popup_signal(&cfg.popup_header),
            notify,
        }
    }

    fn evaluate_inactive(&self) -> Signals {
        let cfg = &self.comm_profile.inactive;
        Signals {
            tray: self.parse_tray_signal(&cfg.tray),
            overlay: OverlaySignal::Hidden,
            popup: self.parse_popup_signal(&cfg.popup_header),
            notify: None,
        }
    }

    fn evaluate_baseline(&self, input: &PolicyInput) -> Signals {
        let baseline = match input.state {
            DeskState::Sitting => &self.comm_profile.baseline.sitting,
            DeskState::Standing => &self.comm_profile.baseline.standing,
            _ => return Signals::default(),
        };

        let limit_secs = match input.state {
            DeskState::Sitting => self.ergo_profile.limits.sitting_secs as f32,
            DeskState::Standing => self.ergo_profile.limits.standing_secs as f32,
            _ => 1.0,
        };
        let progress = input.elapsed_secs as f32 / limit_secs;

        let overlay = match baseline.overlay.as_str() {
            "neutral" => OverlaySignal::Neutral { progress },
            "progress" => OverlaySignal::Progress {
                progress: input.standing_lap_progress,
                lap: input.standing_lap,
                flash: input.standing_lap_flash,
            },
            "hidden" => OverlaySignal::Hidden,
            _ => OverlaySignal::Neutral { progress },
        };

        Signals {
            tray: self.parse_tray_signal(&baseline.tray),
            overlay,
            popup: self.parse_popup_signal(&baseline.popup_header),
            notify: None,
        }
    }

    fn step_to_signals(&mut self, step: &crate::communication_profile::EscalationStep, idx: usize, input: &PolicyInput) -> Signals {
        let limit_secs = match input.state {
            DeskState::Sitting => self.ergo_profile.limits.sitting_secs as f32,
            DeskState::Standing => self.ergo_profile.limits.standing_secs as f32,
            _ => 1.0,
        };
        let progress = (input.elapsed_secs as f32 / limit_secs).min(1.5);

        let overlay = match step.overlay.as_str() {
            "yellow" => OverlaySignal::Yellow { progress },
            "red" => OverlaySignal::Red { progress },
            "pulse_red" | s if s.starts_with("pulse_") => OverlaySignal::PulseRed { progress },
            "hidden" => OverlaySignal::Hidden,
            "neutral" => OverlaySignal::Neutral { progress },
            _ => OverlaySignal::Red { progress },
        };

        // Only fire notification once per escalation step
        let notify = if self.last_notify_step != Some(idx) {
            if let Some(ref notify_type) = step.notify {
                self.last_notify_step = Some(idx);
                let msg = self.get_message_for_state(input.state, notify_type);
                Some(self.make_notify_signal(notify_type, &msg))
            } else {
                None
            }
        } else {
            None
        };

        Signals {
            tray: self.parse_tray_signal(&step.tray),
            overlay,
            popup: self.parse_popup_signal(&step.popup_header),
            notify,
        }
    }

    fn parse_tray_signal(&self, s: &str) -> TraySignal {
        match s {
            "none" => TraySignal::None,
            "yellow" => TraySignal::Yellow,
            "red" => TraySignal::Red,
            s if s.starts_with("blink_") => TraySignal::Blink(s.to_string()),
            _ => TraySignal::None,
        }
    }

    fn parse_popup_signal(&self, s: &str) -> PopupSignal {
        match s {
            "neutral" => PopupSignal::Neutral,
            "yellow" => PopupSignal::Yellow,
            "red" => PopupSignal::Red,
            "gray" => PopupSignal::Gray,
            _ => PopupSignal::Neutral,
        }
    }

    fn make_notify_signal(&self, notify_type: &str, message: &str) -> NotifySignal {
        match notify_type {
            "toast" => NotifySignal::Toast(message.to_string()),
            "popup" => NotifySignal::Popup(message.to_string()),
            _ => NotifySignal::Toast(message.to_string()),
        }
    }

    fn get_message_for_state(&self, state: DeskState, notify_type: &str) -> String {
        match (state, notify_type) {
            (DeskState::Sitting, "toast") => self.comm_profile.messages.sitting_limit_toast.clone(),
            (DeskState::Sitting, "popup") => {
                if self.snooze_index >= self.comm_profile.snooze.tone_shift_after_dismisses {
                    self.comm_profile.messages.positive_popup_messages
                        .get(self.snooze_index % self.comm_profile.messages.positive_popup_messages.len().max(1))
                        .cloned()
                        .unwrap_or_else(|| self.comm_profile.messages.sitting_overdue_popup.clone())
                } else {
                    self.comm_profile.messages.neutral_popup_messages
                        .get(self.snooze_index % self.comm_profile.messages.neutral_popup_messages.len().max(1))
                        .cloned()
                        .unwrap_or_else(|| self.comm_profile.messages.sitting_overdue_popup.clone())
                }
            },
            (DeskState::Standing, "toast") => self.comm_profile.messages.standing_limit_toast.clone(),
            (DeskState::Standing, _) => self.comm_profile.messages.standing_overdue_popup.clone(),
            _ => "Time to change position".to_string(),
        }
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cd src-tauri && cargo test communication_policy`
Expected: all tests pass.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/communication_policy.rs src-tauri/src/communication_policy_tests.rs src-tauri/src/lib.rs
git commit -m "feat(desk): implement CommunicationPolicy with escalation and snooze logic"
```

---

### Task 4: Create default JSON profile files

**Files:**
- Create: `src-tauri/profiles/communication/default.json`
- Create: `src-tauri/profiles/communication/demo.json`
- Create: `src-tauri/profiles/ergonomic/standard.json`
- Create: `src-tauri/profiles/ergonomic/demo.json`

- [ ] **Step 1: Create default communication profile**

Serialize `CommunicationProfile::default()` to `src-tauri/profiles/communication/default.json`.

- [ ] **Step 2: Create demo communication profile**

Same as default but designed for short limits (the limits come from ergonomic profile, but demo comm profile can have shorter snooze durations: `[1, 2, 3]`).

- [ ] **Step 3: Create standard ergonomic profile**

Serialize `ErgonomicProfile::default()` to `src-tauri/profiles/ergonomic/standard.json`.

- [ ] **Step 4: Create demo ergonomic profile**

Same as standard but with `sitting_secs: 120`, `standing_secs: 60`, `standing_target_secs: 30`.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/profiles/
git commit -m "feat(desk): add built-in JSON profiles for communication and ergonomics"
```

---

### Task 5: Wire CommunicationPolicy into AppState and TrayController

This is the core integration task — replace scattered decisions with centralized policy.

**Files:**
- Modify: `src-tauri/src/commands.rs` — add CommunicationPolicy to AppState, remove AlertManager
- Modify: `src-tauri/src/tray_controller.rs` — heavy rewrite: call evaluate(), execute Signals
- Modify: `src-tauri/src/lib.rs` — initialize CommunicationPolicy in setup, remove AlertManager init
- Modify: `src-tauri/src/serial_periodic.rs` — add profile watcher poll, remove direct alert checks

- [ ] **Step 1: Add CommunicationPolicy to AppState**

In `commands.rs`, replace `alert_manager: Arc<Mutex<AlertManager>>` with `comm_policy: Arc<Mutex<CommunicationPolicy>>`.

- [ ] **Step 2: Initialize CommunicationPolicy in lib.rs setup**

In the `Builder::default().setup()` closure:
1. Call `profile_loader::ensure_profiles_dir(app_data_dir)`
2. Load communication and ergonomic profiles from app_data_dir
3. Create `CommunicationPolicy::new(comm, ergo)`
4. Store in AppState
5. Remove AlertManager creation

- [ ] **Step 3: Rewrite tray_controller.rs — on_state_changed**

Replace the existing `on_state_changed` logic:
- Lock `comm_policy`
- Call `comm_policy.on_position_changed()` when state transitions
- Call `comm_policy.evaluate(input)` to get Signals
- Execute each signal (tray, overlay, popup, notify)
- Remove all direct color calculations

- [ ] **Step 4: Rewrite tray_controller.rs — update_overlay_progress (hot path)**

Replace the existing alert_manager ticking:
- Build `PolicyInput` from session snapshot
- Call `comm_policy.evaluate(input)`
- Execute returned Signals
- Handle popup dismiss: call `comm_policy.dismiss()`
- Remove `execute_alert_actions()` function entirely

- [ ] **Step 5: Add profile watcher to serial_periodic.rs**

In `check_periodic()`, add:
1. Check `ProfileWatcher::has_changed()` for both profiles
2. If changed, reload and call `comm_policy.set_comm_profile()` / `set_ergo_profile()`
3. Log the reload

- [ ] **Step 6: Update all AppState references**

Find and fix all `state.alert_manager` references across the codebase — they should now use `state.comm_policy`.

- [ ] **Step 7: Verify compilation and run existing tests**

Run: `cd src-tauri && cargo check && cargo test`
Some alert_manager tests will fail — that's expected (they'll be deleted next).

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/tray_controller.rs src-tauri/src/lib.rs src-tauri/src/serial_periodic.rs
git commit -m "feat(desk): wire CommunicationPolicy into AppState and TrayController"
```

---

### Task 6: Remove old AlertManager and update tests

**Files:**
- Delete: `src-tauri/src/alert_manager.rs`
- Delete: `src-tauri/src/alert_config.rs`
- Delete: `src-tauri/src/alert_actions.rs`
- Delete: `src-tauri/src/alert_manager_tests.rs`
- Delete: `src-tauri/src/tray_icon.rs`
- Modify: `src-tauri/src/lib.rs` — remove module declarations
- Modify: `src-tauri/src/tray_controller_tests.rs` — rewrite for new flow
- Delete: `src-tauri/src/alert_snooze_tests.rs` (if exists)

- [ ] **Step 1: Remove module declarations from lib.rs**

Remove:
```rust
mod alert_actions;
mod alert_config;
mod alert_manager;
mod tray_icon;
```

And their test module declarations.

- [ ] **Step 2: Delete the files**

```bash
rm src-tauri/src/alert_manager.rs src-tauri/src/alert_config.rs src-tauri/src/alert_actions.rs src-tauri/src/tray_icon.rs
rm src-tauri/src/alert_manager_tests.rs
```

- [ ] **Step 3: Fix any remaining references**

Grep for `alert_manager`, `AlertManager`, `AlertAction`, `AlertStage`, `AlertConfig`, `tray_icon::`, `icon_for_state` across the codebase. Fix or remove each reference.

- [ ] **Step 4: Rewrite tray_controller_tests.rs**

Replace AlertManager-based tests with CommunicationPolicy-based tests. Test the integration: mock session state → call evaluate → verify returned Signals.

- [ ] **Step 5: Verify all tests pass**

Run: `cd src-tauri && cargo test`
Expected: all tests pass (some count reduced from deleted alert tests).

- [ ] **Step 6: Commit**

```bash
git add -A src-tauri/src/
git commit -m "refactor(desk): remove AlertManager, absorbed into CommunicationPolicy"
```

---

### Task 7: Slim AppConfig — move fields to profiles

**Files:**
- Modify: `src-tauri/src/config.rs` — remove fields migrated to profiles
- Modify: `src-tauri/src/session_manager.rs` — read limits from ergonomic profile
- Modify: all files that read migrated AppConfig fields

- [ ] **Step 1: Remove migrated fields from AppConfig**

Remove from `config.rs`:
- `sit_limit_mins`, `stand_limit_mins`, `standing_target_mins`, `stand_max_mins` → ergonomic profile
- `pts_standing_per_min`, `pts_session_bonus`, `pts_sitting_per_min` → ergonomic profile
- `kpi_standing_green_pct`, `kpi_standing_yellow_pct`, `kpi_changes_green`, `kpi_changes_yellow`, `kpi_break_yellow_missed`, `kpi_break_red_missed`, `kpi_session_green_mins`, `kpi_session_yellow_mins`, `kpi_early_data_threshold_mins` → ergonomic profile
- `notify_inactivity`, `notify_daily_posture_balance`, `notify_praise_halfway` → communication profile
- `notification_backend` → communication profile

Keep: `sitting_mm`, `standing_mm`, `desk_thickness_mm`, `active_widget`, `show_welcome_on_startup`, `telemetry_enabled`, `telemetry_device_id`.

- [ ] **Step 2: Update session_manager.rs**

`SessionManager::new_from_config()` currently reads limits from AppConfig. Change to accept ergonomic profile limits instead:
- Add `pub fn set_limits_from_profile(&mut self, limits: &Limits)` method
- Or modify `new_from_config()` to accept `ErgonomicProfile` parameter

- [ ] **Step 3: Update all callers of removed AppConfig fields**

Grep for each removed field name. Update callers to read from the appropriate profile via CommunicationPolicy or ergonomic profile.

Key callers:
- `serial_periodic.rs` — reads `config.notify_inactivity` etc. → read from comm profile
- `session_breaks.rs` — reads `config.standing_target_mins` → read from ergo profile
- `session_daily.rs` — reads `config.sit_limit_mins` → read from ergo profile
- `commands.rs` IPC handlers — may expose config to frontend → update to expose profile data

- [ ] **Step 4: Verify compilation and tests**

Run: `cd src-tauri && cargo check && cargo test`

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/
git commit -m "refactor(desk): move limits, scoring, KPI thresholds to profiles; slim AppConfig"
```

---

### Task 8: Settings UI — profile dropdowns

**Files:**
- Modify: `src/components/SettingsPanel.tsx` (or equivalent settings component)
- Create: IPC commands for profile listing/switching in `src-tauri/src/commands.rs`

- [ ] **Step 1: Add IPC commands for profile management**

In `commands.rs`, add:
- `list_communication_profiles()` → returns `Vec<{id, name, description, path}>`
- `list_ergonomic_profiles()` → returns `Vec<{id, name, description, path}>`
- `get_active_profiles()` → returns `{comm_id, ergo_id}`
- `switch_communication_profile(id: String)` → loads and activates
- `switch_ergonomic_profile(id: String)` → loads and activates
- `open_profile_in_editor(path: String)` → opens in system default editor
- `duplicate_profile(path: String, new_name: String)` → copies JSON file

- [ ] **Step 2: Add profile selector UI**

In the settings panel, add two dropdown sections:
- Ergonomic Profile selector with description and Edit/Duplicate/Reset buttons
- Communication Profile selector with description and Edit/Duplicate/Reset buttons

Remove individual sliders for sitting limit, standing limit, etc. — these are now in profiles.

- [ ] **Step 3: Test manually**

1. Start app with `pnpm tauri:dev`
2. Open Settings
3. Verify profile dropdowns show available profiles
4. Switch profiles — verify behavior changes
5. Click "Edit JSON" — verify file opens in editor
6. Edit the JSON file, save — verify hot-reload works within 1s

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands.rs src/components/
git commit -m "feat(desk): add profile selector dropdowns to Settings UI"
```

---

### Task 9: Wave 1 integration test and cleanup

**Files:**
- All modified files — final verification
- `src-tauri/Cargo.toml` — verify no unused dependencies

- [ ] **Step 1: Run full test suite**

```bash
cd src-tauri && cargo test
pnpm test:unit
```

Fix any failures.

- [ ] **Step 2: Manual smoke test**

1. `pnpm tauri:dev` — app starts, loads default profiles
2. Verify tray icon shows correctly (no dot when sitting in limit)
3. Verify overlay bar shows progress
4. Wait for sitting limit — verify yellow → red escalation
5. Switch to `demo` ergonomic profile — verify short limits work
6. Edit `default.json` communication profile — verify hot-reload
7. Disconnect sensor — verify gray tray behavior

- [ ] **Step 3: Commit any fixes**

```bash
git add -A
git commit -m "fix(desk): wave 1 integration fixes"
```

---

## Wave 2 — Enhanced Signals (blinking, color cleanup, extra profiles)

### Task 10: Implement tray blink engine

**Files:**
- Create: `src-tauri/src/tray_blink.rs`
- Create: `src-tauri/src/tray_blink_tests.rs`
- Modify: `src-tauri/src/tray.rs` — add `set_dot_visible(bool)`
- Modify: `src-tauri/src/tray_controller.rs` — integrate blinker
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing tests for TrayBlinker**

Test: start blink → tick sequence → verify on/off/pause phases.
Test: stop mid-blink → immediately invisible.
Test: start new pattern while another active → replaces.

Use `MockClock` to control time:
```rust
use std::cell::Cell;
use std::time::Instant;

struct MockClock { now: Cell<Instant> }
impl MockClock {
    fn advance(&self, ms: u64) {
        self.now.set(self.now.get() + std::time::Duration::from_millis(ms));
    }
}
```

- [ ] **Step 2: Implement TrayBlinker**

```rust
//! tray_blink.rs — Tray icon dot blinking engine.

pub struct TrayBlinker {
    active: bool,
    pattern: Option<BlinkPattern>,
    phase: BlinkPhase,
    step: usize,
    phase_started_ms: u64,
    elapsed_ms: u64,
}

enum BlinkPhase { On, Off, Pause }

pub struct BlinkPattern {
    pub on_ms: u64,
    pub off_ms: u64,
    pub count: u32,
    pub pause_ms: u64,
}

impl TrayBlinker {
    pub fn new() -> Self { /* inactive state */ }
    pub fn start(&mut self, pattern: BlinkPattern) { /* reset and activate */ }
    pub fn stop(&mut self) { /* deactivate */ }
    pub fn is_active(&self) -> bool { self.active }
    /// Advance by delta_ms, return whether dot should be visible.
    pub fn tick(&mut self, delta_ms: u64) -> bool { /* phase state machine */ }
}
```

- [ ] **Step 3: Add set_dot_visible to tray.rs**

Modify `generate_tray_icon()` to accept a `dot_visible: bool` parameter. When false, render the desk silhouette without any dot.

- [ ] **Step 4: Integrate blinker into tray_controller.rs**

When CommunicationPolicy returns `TraySignal::Blink(name)`:
1. Look up the blink pattern from the communication profile
2. Start the TrayBlinker with that pattern
3. On each tick (~60ms), call `blinker.tick(delta)` and update tray icon accordingly

When signal changes to non-blink: call `blinker.stop()`.

- [ ] **Step 5: Run tests**

Run: `cd src-tauri && cargo test tray_blink`

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/tray_blink.rs src-tauri/src/tray_blink_tests.rs src-tauri/src/tray.rs src-tauri/src/tray_controller.rs src-tauri/src/lib.rs
git commit -m "feat(desk): add tray icon blinking engine with pattern support"
```

---

### Task 11: Remove green from color system

**Files:**
- Modify: `src-tauri/src/colors.rs` — remove green constants, add neutral
- Modify: `src/utils/colors.ts` — mirror changes
- Modify: `src/styles/globals.css` — remove `--signal-ok`, add `--signal-neutral`
- Modify: `src/widgets/one-bar/one-bar.css` — update timeline block colors
- Modify: `src/widgets/one-bar/OneBarTimeline.tsx` — update block color logic

- [ ] **Step 1: Update Rust colors.rs**

Remove green/gold constants. Keep yellow, red. Add neutral:
```rust
pub const NEUTRAL: (u8, u8, u8) = (0x2c, 0x29, 0x20);
pub const YELLOW: (u8, u8, u8) = (0xff, 0xc1, 0x07);
pub const RED: (u8, u8, u8) = (0xf4, 0x43, 0x36);
pub const GRAY: (u8, u8, u8) = (0x80, 0x80, 0x80);
```

- [ ] **Step 2: Update TypeScript colors.ts**

Mirror the Rust changes.

- [ ] **Step 3: Update CSS**

In `globals.css`:
- Remove `--signal-ok: #4caf50;`
- Add `--signal-neutral: #2c2920;`
- Keep `--signal-warn`, `--signal-alert`

In `one-bar.css`:
- Update timeline sitting color to use `--signal-neutral` (or `--panel-overlay`)
- Update timeline standing color to use a subtle neutral instead of green

- [ ] **Step 4: Update OneBarTimeline.tsx**

Change `blockModifier()`:
- Sitting (within limit) → `"neutral"` CSS class
- Sitting (over limit) → `"alert"` CSS class
- Standing (within limit) → `"subtle"` CSS class (new)
- Standing (over limit) → `"alert"` CSS class
- Away → `"muted"` CSS class

- [ ] **Step 5: Run tests**

```bash
cd src-tauri && cargo test
pnpm test:unit
```

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/colors.rs src/utils/colors.ts src/styles/globals.css src/widgets/
git commit -m "refactor(desk): remove green/gold from color system, add neutral"
```

---

### Task 12: Create additional built-in profiles

**Files:**
- Create: `src-tauri/profiles/communication/aggressive.json`
- Create: `src-tauri/profiles/communication/gentle.json`
- Create: `src-tauri/profiles/communication/silent.json`
- Create: `src-tauri/profiles/ergonomic/strict.json`
- Create: `src-tauri/profiles/ergonomic/relaxed.json`

- [ ] **Step 1: Create aggressive communication profile**

Yellow 15 min before limit, blink immediately at limit, popup 2 min after.

- [ ] **Step 2: Create gentle communication profile**

Yellow 5 min before, no blink, only toast at limit, no popup.

- [ ] **Step 3: Create silent communication profile**

Overlay only, tray always "none", no notifications.

- [ ] **Step 4: Create strict ergonomic profile**

`sitting_secs: 1500` (25 min), `standing_secs: 900` (15 min), harsh scoring.

- [ ] **Step 5: Create relaxed ergonomic profile**

`sitting_secs: 3600` (60 min), `standing_secs: 1800` (30 min), low penalties.

- [ ] **Step 6: Update profile_loader to copy all built-in profiles**

Modify `ensure_profiles_dir()` to write all built-in profiles, not just defaults.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/profiles/ src-tauri/src/profile_loader.rs
git commit -m "feat(desk): add aggressive, gentle, silent, strict, relaxed profiles"
```

---

### Task 13: Wave 2 integration test and final cleanup

**Files:**
- All modified files — final verification

- [ ] **Step 1: Run full test suite**

```bash
cd src-tauri && cargo test
pnpm test:unit
```

- [ ] **Step 2: Manual smoke test**

1. `pnpm tauri:dev` — verify app starts
2. Sit for demo limit (2 min with demo profile) — verify yellow → red → blink escalation
3. Dismiss popup — verify snooze works
4. Stand up — verify blink stops immediately
5. Disconnect sensor — verify gray blink pattern (3×/10s)
6. Switch between all profiles — verify each behaves differently
7. Edit a profile JSON — verify hot-reload in ~1s
8. Check timeline — verify neutral/subtle colors instead of green/gold
9. Check KPI badges — still show green/yellow/red (independent from comm policy)

- [ ] **Step 3: Update PROJECT.xml**

Add new files to the project map if any new features/IPC events were added.

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "feat(desk): complete communication architecture wave 2 — blinking, colors, profiles"
```

---

## File Inventory Summary

### New files (8 Rust + profile JSONs)
| File | Lines (est.) |
|------|-------------|
| `communication_types.rs` | ~80 |
| `communication_profile.rs` | ~240 |
| `ergonomic_profile.rs` | ~160 |
| `profile_loader.rs` | ~120 |
| `communication_policy.rs` | ~250 |
| `communication_policy_tests.rs` | ~200 |
| `tray_blink.rs` | ~100 |
| `tray_blink_tests.rs` | ~100 |
| Profile JSONs (×7) | ~420 total |

### Deleted files (5)
| File | Lines |
|------|-------|
| `alert_manager.rs` | 185 |
| `alert_config.rs` | 57 |
| `alert_actions.rs` | 40 |
| `alert_manager_tests.rs` | 182 |
| `tray_icon.rs` | 88 |

### Modified files (~15)
`tray_controller.rs`, `tray.rs`, `colors.rs`, `config.rs`, `commands.rs`, `lib.rs`, `serial_periodic.rs`, `notification_service.rs`, `session_manager.rs`, `colors.ts`, `globals.css`, `one-bar.css`, `OneBarTimeline.tsx`, `OneBarWidget.tsx`, Settings components

### Net change
- ~1250 new lines of Rust
- ~420 lines of JSON profiles
- ~550 lines deleted
- ~30-50 tests rewritten
