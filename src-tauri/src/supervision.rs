//! Public PM3 intent contract. No PM3 implementation or Tauri dependency.
use std::{path::PathBuf, time::Duration};
use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};

pub const DEADLINE: Duration = Duration::from_secs(8);
pub const MAX_OUTPUT: usize = 65536;

/// One intent across clicks and retries; at most one request in flight.
pub struct QuitGate { running: AtomicBool }
impl QuitGate {
    pub const fn new() -> Self { Self { running: AtomicBool::new(false) } }
    pub fn begin(&self) -> bool { !self.running.swap(true, Ordering::AcqRel) }
    pub fn retry(&self) { self.running.store(false, Ordering::Release); }
}

#[derive(Clone, Debug)]
pub struct Configuration {
    pub node: PathBuf,
    pub home: PathBuf,
    pub service: String,
}

impl Configuration {
    pub fn from_env(get: impl Fn(&str) -> Option<String>) -> Result<Option<Self>, String> {
        let service = get("DESK_PM3_SERVICE").or_else(|| get("PM3_SERVICE"));
        let mode = get("DESK_SUPERVISION");
        if mode.as_deref() == Some("bridge") { return Ok(None); }
        let configured = service.is_some() || mode.is_some() || get("DESK_PM3_NODE").is_some()
            || get("DESK_PM3_HOME").is_some();
        if !configured { return Ok(None); }
        if mode.as_deref().is_some_and(|v| v != "pm3") {
            return Err("DESK_SUPERVISION must be pm3 or bridge".into());
        }
        let service = service.filter(|s| valid_service(s))
            .ok_or("Set DESK_PM3_SERVICE to project/service")?;
        let node = get("DESK_PM3_NODE").or_else(|| get("PM3_DAEMON_NODE"))
            .map(PathBuf::from).filter(|p| p.is_absolute())
            .ok_or("Set DESK_PM3_NODE to an absolute Node executable")?;
        let home = get("DESK_PM3_HOME").or_else(|| get("PM3_HOME")).map(PathBuf::from)
            .or_else(|| get("USERPROFILE").map(|p| PathBuf::from(p).join(".pm3")))
            .filter(|p| p.is_absolute()).ok_or("Set PM3_HOME to an absolute directory")?;
        Ok(Some(Self { node, home, service }))
    }

    pub fn arguments(&self, action: &str, _intent: &str) -> Vec<String> {
        let mut args = vec![self.home.join("releases/launch.mjs").to_string_lossy().into(),
            action.into(), self.service.clone()];
        if action == "describe" { args.push("--json".into()); }
        args
    }
}

fn valid_service(s: &str) -> bool {
    let parts: Vec<_> = s.split('/').collect();
    parts.len() == 2 && parts.iter().all(|p| !p.is_empty() && p.bytes()
        .all(|c| c.is_ascii_alphanumeric() || b"-_".contains(&c)))
}

pub struct Response { pub exit: i32, pub stdout: Vec<u8> }

/// Ordinary stop prints text; describe returns the installed flat state shape.
pub fn confirms(response: &Response, action: &str, service: &str, _intent: &str) -> bool {
    if response.exit != 0 || response.stdout.len() > MAX_OUTPUT { return false; }
    if action == "stop" {
        let Ok(text) = std::str::from_utf8(&response.stdout) else { return false; };
        return text.trim() == format!("{service}: stopped")
            || text.trim() == format!("{}: stopped", service.rsplit('/').next().unwrap_or(""));
    }
    if action != "describe" { return false; }
    let Ok(v) = serde_json::from_slice::<Value>(&response.stdout) else { return false; };
    v["name"].as_str() == Some(service) && v["state"] == "stopped"
}

#[async_trait::async_trait]
pub trait Transport: Sync {
    async fn invoke(&self, config: &Configuration, action: &str, intent: &str,
        remaining: Duration) -> Result<Response, String>;
}

/// One ordinary stop only. A failed/refused command is never retried or bypassed.
pub async fn stop(config: &Configuration, transport: &impl Transport,
    budget: Duration) -> Result<(), String> {
    let response = tokio::time::timeout(budget,
        transport.invoke(config, "stop", "", budget)).await
        .map_err(|_| "PM3 stop timed out; outcome unknown".to_string())??;
    if confirms(&response, "stop", &config.service, "") { return Ok(()); }
    Err("PM3 stop was not confirmed; inspect PM3 and retry Quit only after resolving the failure.".into())
}

/// Malformed supervision must not silently enable a competing direct login path.
pub(crate) fn pm3_owns_autostart(get: impl Fn(&str) -> Option<String>) -> bool {
    !matches!(Configuration::from_env(get), Ok(None))
}

/// Preserve a previously installed direct public-CLI Run activation even when
/// this particular release was opened manually without supervision variables.
pub(crate) fn is_pm3_login_activation(command: &str, launcher: &std::path::Path) -> bool {
    let command = command.trim().replace('\\', "/").to_ascii_lowercase();
    let launcher = launcher.to_string_lossy().replace('\\', "/").to_ascii_lowercase();
    command.ends_with(&format!("\"{launcher}\" start moveup/desk"))
        || command.ends_with(&format!(" {launcher} start moveup/desk"))
}
