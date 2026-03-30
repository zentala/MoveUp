//! session_persistence.rs — Persist notification flags and credit-adjusted state.
//!
//! Survives app restarts by saving to tauri-plugin-store. On startup,
//! restores flags and credit-reduced `sitting_seconds` so break credit
//! and notification suppression carry over across restarts.

use log::{info, warn};
use serde::{Deserialize, Serialize};
use tauri::{Manager, Runtime};
use tauri_plugin_store::Store;

use crate::session_manager::SessionManager;

/// Key used in tauri-plugin-store for persisted session state.
const STORE_KEY: &str = "persisted_session_state";

/// Saves current session state to the store via AppHandle.
/// Convenience wrapper used by serial_periodic and setup_helpers.
pub fn save_via_app(app: &tauri::AppHandle, session: &SessionManager) {
    let persisted = PersistedSessionState::from_session(session);
    if let Some(store) = app.try_state::<tauri_plugin_store::Store<tauri::Wry>>() {
        if let Err(e) = persisted.save(store.inner()) {
            log::error!("Failed to persist session state: {}", e);
        }
    }
}

/// Clears persisted session state from the store (e.g. on daily reset).
pub fn clear<R: Runtime>(store: &Store<R>) {
    store.delete(STORE_KEY);
    let _ = store.save();
}

/// State that must survive app restarts within the same day.
///
/// Includes notification flags (prevents duplicate alerts) and the
/// credit-reduced `sitting_seconds` (so break credit isn't lost).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedSessionState {
    /// Local date (YYYY-MM-DD) when this state was saved.
    /// On load, discard if date doesn't match today.
    pub date_local: String,
    /// Sitting seconds after break credit (may be less than DB total).
    pub sitting_seconds: i64,
    /// Daily score (gamification).
    pub daily_score: f32,
    // ── Notification flags ──
    pub alert_fired: bool,
    pub stand_alert_fired: bool,
    pub notify_inactivity_fired: bool,
    pub notify_posture_balance_fired: bool,
    pub praise_halfway_fired_today: bool,
    pub standing_target_reached_fired: bool,
}

impl PersistedSessionState {
    /// Captures current state from SessionManager.
    pub fn from_session(session: &SessionManager) -> Self {
        Self {
            date_local: chrono::Local::now().format("%Y-%m-%d").to_string(),
            sitting_seconds: session.state.sitting_seconds,
            daily_score: session.state.daily_score,
            alert_fired: session.alert_fired,
            stand_alert_fired: session.stand_alert_fired,
            notify_inactivity_fired: session.notify_inactivity_fired,
            notify_posture_balance_fired: session.notify_posture_balance_fired,
            praise_halfway_fired_today: session.praise_halfway_fired_today,
            standing_target_reached_fired: session.standing_target_reached_fired,
        }
    }

    /// Saves persisted state to the store.
    pub fn save<R: Runtime>(&self, store: &Store<R>) -> Result<(), String> {
        let value = serde_json::to_value(self)
            .map_err(|e| format!("Failed to serialize session state: {}", e))?;
        store.set(STORE_KEY, value);
        store
            .save()
            .map_err(|e| format!("Failed to save store: {}", e))?;
        Ok(())
    }

    /// Loads persisted state from store. Returns None if missing, corrupt,
    /// or from a different day (stale data after midnight reset).
    pub fn load<R: Runtime>(store: &Store<R>) -> Option<Self> {
        let value = store.get(STORE_KEY)?;
        let state: Self = serde_json::from_value(value).ok()?;
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        if state.date_local != today {
            info!(
                "Discarding persisted session state from {} (today is {})",
                state.date_local, today
            );
            return None;
        }
        Some(state)
    }
}

impl SessionManager {
    /// Restores notification flags and credit-reduced sitting_seconds
    /// from persisted state. Call AFTER `load_today_totals`.
    pub fn load_persisted_state(&mut self, persisted: &PersistedSessionState) {
        // Only override sitting_seconds if the persisted value is lower
        // (i.e., break credit was applied). The DB total is always >= credited.
        if persisted.sitting_seconds < self.state.sitting_seconds {
            info!(
                "Restoring credited sitting_seconds: {} (was {} from DB)",
                persisted.sitting_seconds, self.state.sitting_seconds
            );
            self.state.sitting_seconds = persisted.sitting_seconds;
        } else {
            warn!(
                "Persisted sitting_seconds ({}) >= DB total ({}), keeping DB value",
                persisted.sitting_seconds, self.state.sitting_seconds
            );
        }
        self.state.daily_score = persisted.daily_score;
        self.alert_fired = persisted.alert_fired;
        self.stand_alert_fired = persisted.stand_alert_fired;
        self.notify_inactivity_fired = persisted.notify_inactivity_fired;
        self.notify_posture_balance_fired = persisted.notify_posture_balance_fired;
        self.praise_halfway_fired_today = persisted.praise_halfway_fired_today;
        self.standing_target_reached_fired = persisted.standing_target_reached_fired;
        info!(
            "Restored persisted flags: alert={} stand_alert={} inactivity={} posture={} praise={} standing_target={}",
            self.alert_fired, self.stand_alert_fired,
            self.notify_inactivity_fired, self.notify_posture_balance_fired,
            self.praise_halfway_fired_today, self.standing_target_reached_fired,
        );
    }
}

// Tests in session_tests_persistence.rs
