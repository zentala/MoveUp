//! Public PM3 intent contract. No PM3 implementation or Tauri dependency.
use std::{path::PathBuf, time::Duration};
use serde_json::Value;
use std::sync::{OnceLock, atomic::{AtomicBool, Ordering}};

pub const DEADLINE: Duration = Duration::from_secs(8);
pub const MAX_OUTPUT: usize = 65536;

/// One intent across clicks and retries; at most one request in flight.
pub struct QuitGate { running: AtomicBool, intent: OnceLock<String> }
impl QuitGate {
    pub const fn new() -> Self { Self { running: AtomicBool::new(false), intent: OnceLock::new() } }
    pub fn begin(&self) -> bool { !self.running.swap(true, Ordering::AcqRel) }
    pub fn retry(&self) { self.running.store(false, Ordering::Release); }
    pub fn intent(&self) -> &str {
        self.intent.get_or_init(|| format!("quit-{}", uuid::Uuid::new_v4()))
    }
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

    pub fn arguments(&self, action: &str, intent: &str) -> Vec<String> {
        let mut args = vec![self.home.join("releases/launch.mjs").to_string_lossy().into(),
            action.into(), self.service.clone()];
        if action == "suspend" {
            args.extend(["--intent".into(), intent.into(), "--reason".into(), "MoveUp Quit".into()]);
        }
        args.push("--json".into());
        args
    }
}

fn valid_service(s: &str) -> bool {
    let parts: Vec<_> = s.split('/').collect();
    parts.len() == 2 && parts.iter().all(|p| !p.is_empty() && p.bytes()
        .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c)))
}

pub struct Response { pub exit: i32, pub stdout: Vec<u8> }

fn nonempty(v: &Value, key: &str) -> bool {
    v[key].as_str().is_some_and(|s| !s.is_empty())
}

/// Additional fields are compatible; missing or malformed evidence is unknown.
pub fn confirms(response: &Response, action: &str, service: &str, intent: &str) -> bool {
    if response.exit != 0 || response.stdout.len() > MAX_OUTPUT { return false; }
    let Ok(v) = serde_json::from_slice::<Value>(&response.stdout) else { return false; };
    if v["name"].as_str() != Some(service) { return false; }
    if action == "suspend" {
        return v["suspended"] == true && v["scope"] == "login"
            && v["intentId"].as_str() == Some(intent) && nonempty(&v, "loginId")
            && nonempty(&v, "acknowledgedAt") && v["holds"].as_u64().is_some_and(|n| n > 0);
    }
    if v["state"] != "known" { return false; }
    let sup = &v["supervision"];
    if !matches!(sup["mode"].as_str(), Some("owned" | "standing-down" | "awaiting-misses"
        | "suspended" | "no-candidate" | "degraded")) { return false; }
    if !complete_status(sup) { return false; }
    let suspension = &sup["suspension"];
    if !nonempty(suspension, "loginId") { return false; }
    let Some(holds) = suspension["holds"].as_array() else { return false; };
    holds.iter().all(|h| nonempty(h, "intentId") && nonempty(h, "acknowledgedAt")
        && h.get("reason").is_none_or(Value::is_string))
        && holds.iter().any(|h| h["intentId"].as_str() == Some(intent))
}

fn complete_status(v: &Value) -> bool {
    let nullable = |key: &str| v.get(key).is_some_and(|x| x.is_null() || x.is_string());
    let optional = |value: &Value, key: &str| value.get(key).is_none_or(Value::is_string);
    v["ownedChild"].is_boolean() && (v["observedLive"].is_boolean() || v["observedLive"] == "unknown")
        && nullable("activeCandidate") && nullable("promotedCandidate")
        && v["misses"].as_u64().is_some() && optional(v, "detail")
        && v["demoted"].as_array().is_some_and(|items| items.iter().all(|x|
            x["id"].is_string() && x["artifact"].is_string() && x["reason"].is_string()))
        && v.get("persistence").is_none_or(|p| p["ok"] == false && p["detail"].is_string())
        && v.get("lastCallback").is_none_or(|c| c["candidateId"].is_string()
            && c["ok"].is_boolean() && optional(c, "detail"))
}

#[async_trait::async_trait]
pub trait Transport: Sync {
    async fn invoke(&self, config: &Configuration, action: &str, intent: &str,
        remaining: Duration) -> Result<Response, String>;
}

/// Reconciliation uses only the side-effect-free dedicated supervision command.
pub async fn suspend(config: &Configuration, intent: &str, transport: &impl Transport,
    budget: Duration) -> Result<(), String> {
    let deadline = tokio::time::Instant::now() + budget;
    for action in ["suspend", "supervision", "suspend", "supervision"] {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() { break; }
        // Reserve time to read back even if a CLI invocation never answers.
        let attempt = remaining.min(Duration::from_secs(2));
        let result = tokio::time::timeout(attempt,
            transport.invoke(config, action, intent, attempt)).await;
        if let Ok(Ok(response)) = result {
            if confirms(&response, action, &config.service, intent) { return Ok(()); }
        }
    }
    Err("Could not confirm PM3 suspension. MoveUp is still running. Check PM3 supervision and retry Quit.".into())
}
