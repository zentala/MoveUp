//! session_persistence.rs — One versioned snapshot of the engine.
//!
//! [`PersistedEngineState`] is the single thing written to
//! `tauri-plugin-store`: a `schema_version`, the whole [`SessionState`]
//! verbatim, and the manager-level values living outside it (notification
//! flags, the [`HourlyBreakTracker`]). Before E020-T04 this file hand-mirrored
//! a few `SessionState` fields, so a new engine field silently failed to
//! survive a restart.
//!
//! Schema `0` has neither `schema_version` nor `engine` (builds up to v0.6.0);
//! `1` adds both and still writes the v0 scalars, so a downgrade keeps
//! restoring credit and flags. [`PersistedEngineState::from_stored_value`]
//! reads either shape, migrating in memory rather than rewriting in place.

use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::{Manager, Runtime};
use tauri_plugin_store::Store;

use crate::hourly_break_tracker::HourlyBreakTracker;
use crate::session_manager::SessionManager;
use crate::session_types::SessionState;

/// Key used in tauri-plugin-store for persisted session state.
const STORE_KEY: &str = "persisted_session_state";

/// Schema version this build writes. See the module header for the history.
pub const ENGINE_SCHEMA_VERSION: u32 = 1;

/// Saves current session state to the store via AppHandle.
/// Preserves `daily_reset_after` from previously persisted state.
pub fn save_via_app(app: &tauri::AppHandle, session: &SessionManager) {
    if let Some(store) = app.try_state::<tauri_plugin_store::Store<tauri::Wry>>() {
        let reset = load_reset_after(store.inner());
        if let Err(e) = PersistedEngineState::from_session(session, reset).save(store.inner()) {
            log::error!("Failed to persist session state: {}", e);
        }
    }
}

/// Clears persisted session state on daily reset, but preserves the
/// reset timestamp so that DB seeding after restart filters correctly.
pub fn clear<R: Runtime>(store: &Store<R>) {
    let mut reset_state = PersistedEngineState::empty_for_today();
    reset_state.daily_reset_after = Some(chrono::Utc::now().to_rfc3339());
    match serde_json::to_value(&reset_state) {
        Ok(value) => {
            store.set(STORE_KEY, value);
            let _ = store.save();
        }
        Err(e) => {
            log::error!("Failed to serialize reset state, falling back to delete: {}", e);
            store.delete(STORE_KEY);
            let _ = store.save();
        }
    }
}

/// Loads only the daily_reset_after timestamp from persisted state.
/// Used by DB seeding to filter out pre-reset sessions.
pub fn load_reset_after<R: Runtime>(store: &Store<R>) -> Option<String> {
    let state = PersistedEngineState::from_stored_value(store.get(STORE_KEY)?)?;
    if state.date_local != today_local() {
        return None;
    }
    state.daily_reset_after
}

/// Today's local calendar date, the key every snapshot is filed under.
fn today_local() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// Everything that must survive an app restart within the same day.
/// See the module header for the schema-version history.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PersistedEngineState {
    /// Snapshot schema version. Absent in v0 files, which deserialize to `0`.
    pub schema_version: u32,
    /// Local date (YYYY-MM-DD) when saved. Discarded on load if not today.
    pub date_local: String,
    /// ISO timestamp of last daily reset. DB queries use this to filter
    /// out pre-reset sessions when seeding totals after restart.
    pub daily_reset_after: Option<String>,
    /// The whole engine state, verbatim. `None` for a v0 snapshot, which is
    /// why [`SessionManager::load_persisted_state`] keeps a scalar fallback.
    pub engine: Option<SessionState>,
    // ── v0 mirror of `engine` + the hourly break tracker. Still written so an
    //    older build can read our file; readers prefer `engine` when present.
    pub sitting_seconds: i64,
    pub daily_score: f32,
    pub hours_with_break: HashMap<u8, bool>,
    pub hours_active: HashMap<u8, bool>,
    pub current_away_secs: i64,
    // ── Notification flags (manager-level, not part of SessionState) ──
    pub alert_fired: bool,
    pub stand_alert_fired: bool,
    pub notify_inactivity_fired: bool,
    pub notify_posture_balance_fired: bool,
    pub praise_halfway_fired_today: bool,
    pub standing_target_reached_fired: bool,
}

/// Historical name, kept so existing call sites do not have to change.
pub type PersistedSessionState = PersistedEngineState;

impl PersistedEngineState {
    /// A blank snapshot dated today — the shape written on daily reset.
    pub fn empty_for_today() -> Self {
        Self {
            schema_version: ENGINE_SCHEMA_VERSION,
            date_local: today_local(),
            ..Default::default()
        }
    }

    /// Captures current state from SessionManager.
    /// `reset_after` is preserved from previous persisted state.
    pub fn from_session(session: &SessionManager, reset_after: Option<String>) -> Self {
        let tracker = &session.hourly_break_tracker;
        Self {
            schema_version: ENGINE_SCHEMA_VERSION,
            date_local: today_local(),
            daily_reset_after: reset_after,
            engine: Some(session.state.clone()),
            sitting_seconds: session.state.sitting_seconds,
            daily_score: session.state.daily_score,
            hours_with_break: tracker.hours_with_break.clone(),
            hours_active: tracker.hours_active.clone(),
            current_away_secs: tracker.current_away_secs,
            alert_fired: session.alert_fired,
            stand_alert_fired: session.stand_alert_fired,
            notify_inactivity_fired: session.notify_inactivity_fired,
            notify_posture_balance_fired: session.notify_posture_balance_fired,
            praise_halfway_fired_today: session.praise_halfway_fired_today,
            standing_target_reached_fired: session.standing_target_reached_fired,
        }
    }

    /// Parses a stored JSON value of ANY known schema version, migrating it up
    /// to [`ENGINE_SCHEMA_VERSION`]. `None` when it does not parse at all.
    /// v0 → v1 rewrites no field: a v0 value parses with `schema_version == 0`
    /// and `engine == None`, so the loader falls back to the scalar path.
    pub fn from_stored_value(value: serde_json::Value) -> Option<Self> {
        let mut state: Self = serde_json::from_value(value)
            .map_err(|e| warn!("Discarding unreadable persisted session state: {}", e))
            .ok()?;
        if state.schema_version < ENGINE_SCHEMA_VERSION {
            info!(
                "Migrating persisted session state v{} -> v{}",
                state.schema_version, ENGINE_SCHEMA_VERSION
            );
            state.schema_version = ENGINE_SCHEMA_VERSION;
        }
        Some(state)
    }

    /// Saves persisted state to the store.
    pub fn save<R: Runtime>(&self, store: &Store<R>) -> Result<(), String> {
        let value = serde_json::to_value(self)
            .map_err(|e| format!("Failed to serialize session state: {}", e))?;
        store.set(STORE_KEY, value);
        store.save().map_err(|e| format!("Failed to save store: {}", e))
    }

    /// Loads persisted state from store. Returns None if missing, corrupt,
    /// or from a different day (stale data after midnight reset).
    pub fn load<R: Runtime>(store: &Store<R>) -> Option<Self> {
        let state = Self::from_stored_value(store.get(STORE_KEY)?)?;
        let today = today_local();
        if state.date_local != today {
            info!("Discarding persisted state from {} (today is {})", state.date_local, today);
            return None;
        }
        Some(state)
    }
}

impl SessionManager {
    /// Restores the engine from a persisted snapshot. Call AFTER
    /// `load_today_totals`, which seeds today's raw totals from SQLite. The DB
    /// stays authoritative for raw daily totals; the snapshot is authoritative
    /// for what the DB cannot know — credited sitting time, notification
    /// flags, the hourly break tracker. Live timer anchors are dropped: the
    /// app was not running, so that gap is not sitting time.
    pub fn load_persisted_state(&mut self, persisted: &PersistedEngineState) {
        let from_db = self.state.clone();
        if let Some(engine) = &persisted.engine {
            self.state = engine.clone();
            self.state.standing_seconds = from_db.standing_seconds.max(engine.standing_seconds);
            self.state.position_changes = from_db.position_changes.max(engine.position_changes);
            self.state.sitting_seconds_total = from_db
                .sitting_seconds_total
                .max(engine.sitting_seconds_total);
            self.clear_live_anchors();
        }
        self.restore_credited_sitting(persisted.sitting_seconds, from_db.sitting_seconds);
        self.state.daily_score = persisted.daily_score;
        self.hourly_break_tracker = HourlyBreakTracker::restore(
            persisted.hours_with_break.clone(),
            persisted.hours_active.clone(),
            persisted.current_away_secs,
        );
        self.state.hourly_breaks_covered = self.hourly_break_tracker.hours_with_break();
        self.state.hourly_breaks_active = self.hourly_break_tracker.hours_active();
        self.alert_fired = persisted.alert_fired;
        self.stand_alert_fired = persisted.stand_alert_fired;
        self.notify_inactivity_fired = persisted.notify_inactivity_fired;
        self.notify_posture_balance_fired = persisted.notify_posture_balance_fired;
        self.praise_halfway_fired_today = persisted.praise_halfway_fired_today;
        self.standing_target_reached_fired = persisted.standing_target_reached_fired;
        info!(
            "Restored snapshot v{} (engine={}): sitting={}s score={}",
            persisted.schema_version, persisted.engine.is_some(),
            self.state.sitting_seconds, self.state.daily_score,
        );
    }

    /// Takes the persisted credited value only when it is lower than the DB
    /// total — i.e. only when break credit had actually reduced it.
    fn restore_credited_sitting(&mut self, persisted: i64, from_db: i64) {
        if persisted < from_db {
            info!(
                "Restoring credited sitting_seconds: {} (was {} from DB)",
                persisted, from_db
            );
            self.state.sitting_seconds = persisted;
        } else {
            warn!(
                "Persisted sitting_seconds ({}) >= DB total ({}), keeping DB value",
                persisted, from_db
            );
            self.state.sitting_seconds = from_db;
        }
    }

    /// Drops every "started at" anchor a live counter measures from.
    fn clear_live_anchors(&mut self) {
        self.state.sitting_started = None;
        self.state.break_started = None;
        self.state.standing_session_started = None;
        self.state.standing_bout_started = None;
        self.state.last_tick_ts = None;
        self.state.last_accumulate_ts = None;
    }
}

// Tests in session_tests_persistence.rs
