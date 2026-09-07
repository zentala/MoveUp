//! commands_relay.rs — relay IPC commands and the client supervisor (E022-T06).
//!
//! Six commands drive the whole feature from Settings: register once with a
//! license key, hand out a pairing code, list and revoke paired phones, turn
//! the relay off, and read the status the UI renders.
//!
//! The supervisor ([`sync_client`]) is the single place that decides whether
//! the outbound socket should be running. Every path that can change the
//! answer — registration, disable, a settings save, app startup — calls it and
//! nothing else touches [`crate::relay_client::spawn`]. Two conditions must
//! both hold: the feature is on AND a credential exists ([`plan_action`]).

use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Manager, State};

use crate::commands::{ensure_initialized, AppState};
use crate::config::AppConfig;
use crate::relay_auth::{
    clear_desk, effective_url, load_desk, save_desk, CredentialStore, DeskRecord, KeyringStore,
    PairingCode, RelayApi, Viewer,
};
use crate::relay_client::{self, ClientDeps, RelayConfig, SnapshotSource, TokioSleeper};
use crate::relay_status::{RelayState, RelayStatus};

type StoreState = tauri_plugin_store::Store<tauri::Wry>;

// ─── Supervisor ─────────────────────────────────────────────────────────────

/// What the supervisor should do with the client task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClientAction {
    /// Stop any running task and open a fresh connection.
    Start,
    /// Stop the task and report [`RelayState::Disabled`].
    Stop,
}

/// The two conditions, kept apart on purpose.
///
/// "Switched off" and "never registered" are different facts and neither
/// implies the other, but they produce the same action: no socket.
pub(crate) fn plan_action(enabled: bool, has_token: bool, has_desk: bool) -> ClientAction {
    if enabled && has_token && has_desk {
        ClientAction::Start
    } else {
        ClientAction::Stop
    }
}

/// The snapshot the relay sends right after `welcome`.
///
/// Built through [`crate::remote_display_state::build`] — the one composition
/// point — so the phone over the relay sees exactly what the LAN display sees.
pub(crate) struct AppSnapshot {
    pub(crate) session: Arc<Mutex<crate::session::SessionManager>>,
    pub(crate) comm_policy: Arc<Mutex<crate::communication_policy::CommunicationPolicy>>,
    pub(crate) today_cache: Arc<Mutex<crate::db::TodaySummary>>,
    pub(crate) health: crate::health_source::HealthState,
}

impl SnapshotSource for AppSnapshot {
    fn snapshot(&self) -> serde_json::Value {
        let state = crate::remote_display_state::build(
            &self.session,
            &self.comm_policy,
            &self.today_cache,
            self.health.last_view(),
        );
        serde_json::to_value(crate::ws_broadcaster::DisplayEvent::Snapshot(state))
            .unwrap_or(serde_json::Value::Null)
    }
}

/// Starts, restarts or stops the relay client so it matches the current
/// config and credentials. Safe to call as often as you like.
pub fn sync_client(app: &AppHandle) {
    let state: State<'_, AppState> = app.state();
    let (enabled, url) = {
        let cfg = state.config.lock().unwrap_or_else(|e| e.into_inner());
        let cfg = cfg.clone().unwrap_or_default();
        (cfg.relay_enabled, effective_url(&cfg.relay_url))
    };
    let desk = app.try_state::<StoreState>().and_then(|s| load_desk(s.inner()));
    let token = KeyringStore::new().ok().and_then(|s| s.read());

    // Stop first in every case: `Start` means "one fresh connection", and a
    // second task on the same room would be closed with 4409 anyway.
    if let Some(old) = state
        .relay_handle
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .take()
    {
        old.stop();
    }

    let action = plan_action(enabled, token.is_some(), desk.is_some());
    let (ClientAction::Start, Some(desk), Some(token)) = (action, desk, token) else {
        set_status(&state.relay, enabled, RelayState::Disabled, None);
        return;
    };
    set_status(&state.relay, enabled, RelayState::Connecting, None);

    let cfg = RelayConfig::new(url, desk.desk_id, token, crate::APP_VERSION);
    let deps = ClientDeps {
        ws_tx: state.ws_tx.clone(),
        status: state.relay.clone(),
        sleeper: Arc::new(TokioSleeper),
        snapshot: Arc::new(AppSnapshot {
            session: state.session.clone(),
            comm_policy: state.comm_policy.clone(),
            today_cache: state.today_cache.clone(),
            health: app.state::<crate::health_source::HealthState>().inner().clone(),
        }),
    };
    let slot = state.relay_handle.clone();
    // Spawned rather than called directly: `relay_client::spawn` uses
    // `tokio::spawn`, which panics outside a runtime, and this function is
    // called from synchronous setup code too.
    tauri::async_runtime::spawn(async move {
        let handle = relay_client::spawn(cfg, deps);
        *slot.lock().unwrap_or_else(|e| e.into_inner()) = Some(handle);
    });
}

fn set_status(
    status: &Arc<Mutex<RelayStatus>>,
    enabled: bool,
    state: RelayState,
    error: Option<String>,
) {
    let mut s = status.lock().unwrap_or_else(|e| e.into_inner());
    s.enabled = enabled;
    s.enter(state, chrono::Utc::now().timestamp_millis(), error);
}

// ─── Small helpers the commands share ───────────────────────────────────────

/// This machine's name, as it will appear in the phone's device list.
pub(crate) fn desk_name() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .ok()
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| "MoveUp desk".to_string())
}

/// The registration this desk is holding, or a message saying it holds none.
fn credentials(app: &AppHandle) -> Result<(DeskRecord, String), String> {
    let store = app
        .try_state::<StoreState>()
        .ok_or_else(|| "Failed to access store".to_string())?;
    let desk = load_desk(store.inner()).ok_or_else(|| "This desk is not registered".to_string())?;
    let token = KeyringStore::new()?
        .read()
        .ok_or_else(|| "No desk token in the credential store".to_string())?;
    Ok((desk, token))
}

fn config_of(state: &State<'_, AppState>) -> AppConfig {
    state
        .config
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .unwrap_or_default()
}

/// Persists `relay_enabled` without disturbing the rest of the config.
fn set_enabled(app: &AppHandle, state: &State<'_, AppState>, enabled: bool) -> Result<(), String> {
    let mut cfg = config_of(state);
    cfg.relay_enabled = enabled;
    let store = app
        .try_state::<StoreState>()
        .ok_or_else(|| "Failed to access store".to_string())?;
    cfg.save(store.inner())?;
    *state.config.lock().unwrap_or_else(|e| e.into_inner()) = Some(cfg);
    Ok(())
}

// ─── Commands ───────────────────────────────────────────────────────────────

/// Trades a license key for this desk's identity, stores both halves, and
/// turns the relay on. Nothing is stored when the relay rejects the key.
#[tauri::command]
pub async fn relay_register(
    license_key: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<RelayStatus, String> {
    ensure_initialized(&app, &state)?;
    let cfg = config_of(&state);
    let name = desk_name();
    let registration = RelayApi::new(&cfg.relay_url)
        .register(license_key.trim(), &name, crate::APP_VERSION)
        .await
        .map_err(|e| e.message)?;

    KeyringStore::new()?.write(&registration.desk_token)?;
    let store = app
        .try_state::<StoreState>()
        .ok_or_else(|| "Failed to access store".to_string())?;
    save_desk(
        store.inner(),
        &DeskRecord {
            desk_id: registration.desk_id,
            desk_name: name,
            plan: registration.plan,
            expires_at: registration.expires_at,
        },
    )?;
    set_enabled(&app, &state, true)?;
    sync_client(&app);
    Ok(state.relay.lock().unwrap_or_else(|e| e.into_inner()).clone())
}

/// Issues a pairing code, valid for a few minutes, plus the QR deep link.
#[tauri::command]
pub async fn relay_start_pairing(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<PairingCode, String> {
    let url = config_of(&state).relay_url;
    let (desk, token) = credentials(&app)?;
    RelayApi::new(&url)
        .start_pairing(&desk.desk_id, &token)
        .await
        .map_err(|e| e.message)
}

/// Every phone paired with this desk. An empty list means none are paired —
/// the UI must say so rather than render a blank panel.
#[tauri::command]
pub async fn relay_list_viewers(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<Viewer>, String> {
    let url = config_of(&state).relay_url;
    let (desk, token) = credentials(&app)?;
    RelayApi::new(&url)
        .list_viewers(&desk.desk_id, &token)
        .await
        .map_err(|e| e.message)
}

/// Revokes one phone. The relay closes that viewer's socket with `4403`.
#[tauri::command]
pub async fn relay_revoke_viewer(
    viewer_id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let url = config_of(&state).relay_url;
    let (desk, token) = credentials(&app)?;
    RelayApi::new(&url)
        .revoke_viewer(&desk.desk_id, &viewer_id, &token)
        .await
        .map_err(|e| e.message)
}

/// Turns the relay off and forgets this desk's credentials.
///
/// The local half is cleared even when the relay cannot be reached: a desk
/// must be able to switch itself off without the cloud's permission. The
/// remote failure is logged, not surfaced as a refusal.
#[tauri::command]
pub async fn relay_disable(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let url = config_of(&state).relay_url;
    if let Ok((desk, token)) = credentials(&app) {
        if let Err(e) = RelayApi::new(&url).delete_desk(&desk.desk_id, &token).await {
            log::warn!("Relay: remote revoke failed ({}), clearing locally anyway", e.code);
        }
    }
    KeyringStore::new()?.delete()?;
    if let Some(store) = app.try_state::<StoreState>() {
        clear_desk(store.inner())?;
    }
    set_enabled(&app, &state, false)?;
    sync_client(&app);
    Ok(())
}

/// The status Settings and the Debug tab render.
#[tauri::command]
pub fn get_relay_status(state: State<'_, AppState>) -> RelayStatus {
    state.relay.lock().unwrap_or_else(|e| e.into_inner()).clone()
}
