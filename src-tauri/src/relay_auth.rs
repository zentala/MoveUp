//! relay_auth.rs — desk credentials and the relay's REST surface (E022-T06).
//!
//! Two things live here because they are the two halves of one fact — "this
//! desk is registered": the secret half (`desk_token`, in the OS credential
//! store, never in a file and never logged) and the public half (`desk_id`,
//! `desk_name`, in `tauri-plugin-store` next to the rest of the settings).
//!
//! The REST calls are the desk side of PLAN.md §Protocol — REST. Every one of
//! them returns [`ErrorBody`] rather than a bare string, so the relay's own
//! error code (`invalid_license`, `viewer_limit`, …) survives all the way to
//! the Settings UI instead of being flattened into prose.
//!
//! Nothing here opens a socket — that is [`crate::relay_client`] — and nothing
//! here reads Tauri state; [`crate::commands_relay`] wires the two together.

use serde::{Deserialize, Serialize};
use tauri::Runtime;
use tauri_plugin_store::Store;

use crate::remote_protocol::ErrorBody;

/// Relay this build talks to unless Settings overrides it (D5).
pub const RELAY_DEFAULT_URL: &str = "https://relay.desk.zentala.io";
/// Credential-store service name, as it appears in Windows Credential Manager.
pub const KEYRING_SERVICE: &str = "MoveUp Relay";
/// Credential-store account name.
pub const KEYRING_ACCOUNT: &str = "desk_token";
/// `tauri-plugin-store` key holding the non-secret half of the registration.
pub const DESK_KEY: &str = "relay_desk";

/// How long a REST call may take before it counts as unreachable.
const HTTP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

// ─── Credential store ───────────────────────────────────────────────────────

/// Where the desk token lives.
///
/// A trait, not a bare `keyring::Entry`, so the commands can be exercised
/// without a credential store at all — and so the production path never has a
/// branch that could accidentally fall back to a file.
pub trait CredentialStore: Send + Sync {
    fn read(&self) -> Option<String>;
    fn write(&self, secret: &str) -> Result<(), String>;
    fn delete(&self) -> Result<(), String>;
}

/// The OS credential store, held as ONE entry created once.
///
/// One entry rather than one per call is not just tidiness: `keyring`'s mock
/// store keeps its secret inside the entry, so a per-call entry could never
/// round-trip under `cfg(test)`.
pub struct KeyringStore {
    entry: keyring::Entry,
}

impl KeyringStore {
    /// Opens the desk-token entry.
    ///
    /// Under `cfg(test)` the mock credential builder is installed first, so a
    /// test can never write to the real Windows Credential Manager.
    pub fn new() -> Result<Self, String> {
        #[cfg(test)]
        {
            static ONCE: std::sync::Once = std::sync::Once::new();
            ONCE.call_once(|| {
                keyring::set_default_credential_builder(keyring::mock::default_credential_builder())
            });
        }
        let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)
            .map_err(|e| format!("credential store unavailable: {e}"))?;
        Ok(Self { entry })
    }
}

impl CredentialStore for KeyringStore {
    fn read(&self) -> Option<String> {
        match self.entry.get_password() {
            Ok(s) if !s.is_empty() => Some(s),
            _ => None,
        }
    }

    fn write(&self, secret: &str) -> Result<(), String> {
        self.entry
            .set_password(secret)
            .map_err(|e| format!("could not store the desk token: {e}"))
    }

    /// Deleting a credential that is already gone is success — "no token" is
    /// the state the caller asked for, not a failure to reach it.
    fn delete(&self) -> Result<(), String> {
        match self.entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(format!("could not delete the desk token: {e}")),
        }
    }
}

// ─── Non-secret registration record ─────────────────────────────────────────

/// The public half of a registration, kept in `tauri-plugin-store`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export_to = "../../src/generated/"))]
pub struct DeskRecord {
    pub desk_id: String,
    pub desk_name: String,
    pub plan: Option<String>,
    pub expires_at: Option<String>,
}

/// Reads the stored registration, or `None` when this desk is not registered.
pub fn load_desk<R: Runtime>(store: &Store<R>) -> Option<DeskRecord> {
    serde_json::from_value(store.get(DESK_KEY)?).ok()
}

/// Persists the registration's public half.
pub fn save_desk<R: Runtime>(store: &Store<R>, record: &DeskRecord) -> Result<(), String> {
    let value = serde_json::to_value(record).map_err(|e| e.to_string())?;
    store.set(DESK_KEY, value);
    store.save().map_err(|e| e.to_string())
}

/// Forgets the registration's public half.
pub fn clear_desk<R: Runtime>(store: &Store<R>) -> Result<(), String> {
    store.delete(DESK_KEY);
    store.save().map_err(|e| e.to_string())
}

// ─── REST DTOs ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct RegisterRequest<'a> {
    license_key: &'a str,
    desk_name: &'a str,
    app_version: &'a str,
}

/// `POST /v1/desks/register` → `201`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Registration {
    pub desk_id: String,
    pub desk_token: String,
    pub plan: Option<String>,
    pub expires_at: Option<String>,
}

/// `POST /v1/desks/{id}/pairings` → `201`, plus the QR string built locally.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export_to = "../../src/generated/"))]
pub struct PairingCode {
    pub code: String,
    pub expires_at: String,
    /// What the QR encodes: the viewer app deep-linked to this pairing.
    pub qr_payload: String,
    /// Shown next to the code so a phone can be paired by typing.
    pub desk_id: String,
}

/// One paired phone, as `GET /v1/desks/{id}/viewers` returns it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export_to = "../../src/generated/"))]
pub struct Viewer {
    pub viewer_id: String,
    pub device_name: String,
    pub paired_at: String,
    pub last_seen: Option<String>,
    #[serde(default)]
    pub online: bool,
}

#[derive(Debug, Deserialize)]
struct PairingResponse {
    code: String,
    expires_at: String,
}

#[derive(Debug, Deserialize)]
struct ErrorEnvelope {
    error: ErrorBody,
}

/// The deep link a phone follows after scanning the pairing QR.
pub fn qr_payload(base_url: &str, desk_id: &str, code: &str) -> String {
    format!(
        "{}/app/#/pair?d={desk_id}&c={code}",
        base_url.trim_end_matches('/')
    )
}

/// The relay URL to use: whatever Settings holds, or [`RELAY_DEFAULT_URL`].
pub fn effective_url(configured: &str) -> String {
    let trimmed = configured.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        RELAY_DEFAULT_URL.to_string()
    } else {
        trimmed.to_string()
    }
}

// ─── REST client ────────────────────────────────────────────────────────────

/// The desk's client for the relay's REST routes.
pub struct RelayApi {
    base: String,
    http: reqwest::Client,
}

impl RelayApi {
    pub fn new(base_url: &str) -> Self {
        Self {
            base: effective_url(base_url),
            http: reqwest::Client::builder()
                .timeout(HTTP_TIMEOUT)
                .build()
                .unwrap_or_default(),
        }
    }

    pub fn base(&self) -> &str {
        &self.base
    }

    fn url(&self, tail: &str) -> String {
        format!("{}{tail}", self.base)
    }

    /// Trades a license key for this desk's identity and token.
    pub async fn register(
        &self,
        license_key: &str,
        desk_name: &str,
        app_version: &str,
    ) -> Result<Registration, ErrorBody> {
        let body = RegisterRequest {
            license_key,
            desk_name,
            app_version,
        };
        let sent = self
            .http
            .post(self.url("/v1/desks/register"))
            .json(&body)
            .send()
            .await;
        json_or_error(sent).await
    }

    /// Asks for a fresh pairing code; any previous one stops working.
    pub async fn start_pairing(&self, desk_id: &str, token: &str) -> Result<PairingCode, ErrorBody> {
        let sent = self
            .http
            .post(self.url(&format!("/v1/desks/{desk_id}/pairings")))
            .bearer_auth(token)
            .json(&serde_json::json!({}))
            .send()
            .await;
        let got: PairingResponse = json_or_error(sent).await?;
        Ok(PairingCode {
            qr_payload: qr_payload(&self.base, desk_id, &got.code),
            desk_id: desk_id.to_string(),
            code: got.code,
            expires_at: got.expires_at,
        })
    }

    /// Every phone paired with this desk. An empty list is a valid answer.
    pub async fn list_viewers(&self, desk_id: &str, token: &str) -> Result<Vec<Viewer>, ErrorBody> {
        let sent = self
            .http
            .get(self.url(&format!("/v1/desks/{desk_id}/viewers")))
            .bearer_auth(token)
            .send()
            .await;
        json_or_error(sent).await
    }

    /// Revokes one phone; the relay closes its socket with `4403`.
    pub async fn revoke_viewer(
        &self,
        desk_id: &str,
        viewer_id: &str,
        token: &str,
    ) -> Result<(), ErrorBody> {
        self.delete(&format!("/v1/desks/{desk_id}/viewers/{viewer_id}"), token)
            .await
    }

    /// Revokes this desk and every viewer paired with it.
    pub async fn delete_desk(&self, desk_id: &str, token: &str) -> Result<(), ErrorBody> {
        self.delete(&format!("/v1/desks/{desk_id}"), token).await
    }

    async fn delete(&self, tail: &str, token: &str) -> Result<(), ErrorBody> {
        let sent = self
            .http
            .delete(self.url(tail))
            .bearer_auth(token)
            .send()
            .await;
        empty_or_error(sent).await
    }
}

/// A transport failure, phrased the same way as a relay-side error.
fn unreachable(e: reqwest::Error) -> ErrorBody {
    ErrorBody::new("unreachable", format!("relay unreachable: {e}"))
}

/// Parses the relay's own `{error: {code, message}}`, or synthesizes one from
/// the status. An unparseable body must never read as a success.
async fn to_error(resp: reqwest::Response) -> ErrorBody {
    let status = resp.status();
    match resp.json::<ErrorEnvelope>().await {
        Ok(env) => env.error,
        Err(_) => ErrorBody::new("http_error", format!("relay answered {status}")),
    }
}

async fn json_or_error<T: serde::de::DeserializeOwned>(
    sent: Result<reqwest::Response, reqwest::Error>,
) -> Result<T, ErrorBody> {
    let resp = sent.map_err(unreachable)?;
    if !resp.status().is_success() {
        return Err(to_error(resp).await);
    }
    resp.json::<T>()
        .await
        .map_err(|e| ErrorBody::new("bad_response", format!("unreadable relay response: {e}")))
}

async fn empty_or_error(
    sent: Result<reqwest::Response, reqwest::Error>,
) -> Result<(), ErrorBody> {
    let resp = sent.map_err(unreachable)?;
    if resp.status().is_success() {
        Ok(())
    } else {
        Err(to_error(resp).await)
    }
}
