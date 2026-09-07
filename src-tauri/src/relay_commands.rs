//! relay_commands.rs — running a remote viewer's command on the desk (E022-T07).
//!
//! Three commands, no generic write path: `ack_alert`, `set_limits`,
//! `switch_profile`. The allowlist and every argument bound live as data in
//! [`crate::remote_protocol::COMMAND_ALLOWLIST`], and this module re-checks
//! them with [`validate_command`] even though the relay checked them first —
//! the desk never trusts that the relay validated anything.
//!
//! Nothing here talks to a socket and nothing here needs an `AppHandle`:
//! [`CommandCtx`] carries plain handles, so `relay_client.rs` can execute a
//! command from its tokio task and the tests can execute one with no Tauri
//! runtime at all.

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde_json::Value;

use crate::alert_popup::AlertPopup;
use crate::commands_profiles::{
    switch_communication_profile_by_name, switch_ergonomic_profile_by_name,
};
use crate::communication_policy::CommunicationPolicy;
use crate::event_logger::EventLogger;
use crate::remote_protocol::{check_version, validate_command, Command, CommandResult, Envelope, ErrorBody};
use crate::session::SessionManager;

/// How far a command's `ts` may be from this machine's clock, in either
/// direction, before it is refused.
///
/// Both directions matter. A command older than this is a replay of something
/// the user asked for minutes ago — acting on it is a surprise. One stamped in
/// the future would, if only the past were checked, never expire at all.
pub const MAX_COMMAND_AGE_MS: i64 = 30_000;

/// Everything a command may touch. An `AppState`-shaped subset, deliberately
/// not `AppState` itself, so this module stays constructible in a unit test.
pub struct CommandCtx {
    pub session: Arc<Mutex<SessionManager>>,
    pub comm_policy: Arc<Mutex<CommunicationPolicy>>,
    pub alert_popup: Arc<Mutex<AlertPopup>>,
    /// Root of the app data directory — `profiles/` hangs off it.
    pub app_data_dir: PathBuf,
    /// Where `REMOTE …` lines go. `None` means the app has no event log yet;
    /// the command still runs, it is just not written down.
    pub events: Option<Arc<EventLogger>>,
    /// Command ids admitted in the last [`MAX_COMMAND_AGE_MS`], each paired
    /// with the time it expires from this list. The relay is untrusted (ADR
    /// 023 — "the desk never trusts the relay") and could resend the same
    /// envelope inside its own staleness window; this stops that resend from
    /// executing twice (security review 2026-09-07, finding Medium #2).
    pub seen_commands: Mutex<VecDeque<(String, i64)>>,
}

/// What `relay_client.rs` calls. A trait so the client task keeps knowing
/// nothing about `SessionManager` or the profile files.
pub trait CommandExecutor: Send + Sync {
    fn execute(&self, env: &Envelope<Command>) -> CommandResult;
}

impl CommandExecutor for CommandCtx {
    fn execute(&self, env: &Envelope<Command>) -> CommandResult {
        execute(env, self, chrono::Utc::now().timestamp_millis())
    }
}

// ─── Entry point ────────────────────────────────────────────────────────────

/// Admits, runs and records one command. Never panics and never returns
/// `Err` — a refusal is a `CommandResult` with `ok: false`, because the viewer
/// is owed an answer either way.
///
/// `now_ms` is passed in rather than read here so the staleness rule is
/// testable without sleeping.
pub fn execute(env: &Envelope<Command>, ctx: &CommandCtx, now_ms: i64) -> CommandResult {
    let name = sanitize(&env.payload.name);
    let viewer = sanitize(env.payload.viewer_id.as_deref().unwrap_or("unknown"));

    if let Err(e) = admit(env, ctx, now_ms) {
        // A denial is logged under its own verb so `grep 'REMOTE DENIED'`
        // answers "was anything refused" without reading every ok line.
        log_event(ctx, &format!("REMOTE DENIED {}", name));
        log::warn!("relay_commands: denied '{}' from viewer={} ({})", name, viewer, e.code);
        return failed(&env.id, e);
    }

    match run(&env.payload, ctx) {
        Ok(()) => {
            log_event(ctx, &format!("REMOTE {} viewer={} ok", name, viewer));
            CommandResult {
                command_id: env.id.clone(),
                ok: true,
                error: None,
            }
        }
        Err(e) => {
            log_event(ctx, &format!("REMOTE {} viewer={} err={}", name, viewer, e.code));
            failed(&env.id, e)
        }
    }
}

fn failed(command_id: &str, error: ErrorBody) -> CommandResult {
    CommandResult {
        command_id: command_id.to_string(),
        ok: false,
        error: Some(error),
    }
}

/// Everything checked before a single handle is locked: wire version,
/// freshness, replay, then the name and the arguments.
fn admit(env: &Envelope<Command>, ctx: &CommandCtx, now_ms: i64) -> Result<(), ErrorBody> {
    check_version(env.v)?;
    if (now_ms - env.ts).abs() > MAX_COMMAND_AGE_MS {
        return Err(ErrorBody::new(
            "stale_command",
            format!("command timestamp is more than {}s from now", MAX_COMMAND_AGE_MS / 1000),
        ));
    }
    if already_seen(ctx, &env.id, now_ms) {
        return Err(ErrorBody::new(
            "replayed_command",
            "this command id was already admitted".to_string(),
        ));
    }
    validate_command(&env.payload.name, &env.payload.args)?;
    remember(ctx, &env.id, now_ms);
    Ok(())
}

/// True when `id` was admitted within the current staleness window. Also
/// drops every entry that has aged out, so the list never grows unbounded.
fn already_seen(ctx: &CommandCtx, id: &str, now_ms: i64) -> bool {
    let mut seen = ctx.seen_commands.lock().unwrap_or_else(|e| e.into_inner());
    while matches!(seen.front(), Some((_, expires_at)) if *expires_at <= now_ms) {
        seen.pop_front();
    }
    seen.iter().any(|(seen_id, _)| seen_id == id)
}

/// Records `id` as admitted, to expire from the list after `MAX_COMMAND_AGE_MS`.
fn remember(ctx: &CommandCtx, id: &str, now_ms: i64) {
    let mut seen = ctx.seen_commands.lock().unwrap_or_else(|e| e.into_inner());
    seen.push_back((id.to_string(), now_ms + MAX_COMMAND_AGE_MS));
}

/// Dispatches an already-admitted command.
fn run(cmd: &Command, ctx: &CommandCtx) -> Result<(), ErrorBody> {
    match cmd.name.as_str() {
        "ack_alert" => {
            ack_alert(ctx);
            Ok(())
        }
        "set_limits" => set_limits(&cmd.args, ctx),
        "switch_profile" => switch_profile(&cmd.args, ctx),
        // `admit` already rejected anything outside the allowlist; this arm
        // exists so adding a spec without a handler fails loudly.
        other => Err(ErrorBody::new(
            "unknown_command",
            format!("no handler for allowlisted command: {}", sanitize(other)),
        )),
    }
}

// ─── The three commands ─────────────────────────────────────────────────────

/// Acknowledges the sit-limit alert the way a click on the popup does:
/// the policy takes its snooze step, then the window goes away.
///
/// The popup's `user_dismissed` flag is deliberately *not* set. That flag is
/// how the popup thread tells `tray_controller` to call
/// `CommunicationPolicy::dismiss`, and we have just called it — setting the
/// flag as well would advance `snooze_index` twice for one acknowledgement.
fn ack_alert(ctx: &CommandCtx) {
    ctx.comm_policy
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .dismiss();
    ctx.alert_popup
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .dismiss();
}

/// Sets either limit, or both. `admit` has already established that at least
/// one is present and that both are inside their allowlisted range.
fn set_limits(args: &Value, ctx: &CommandCtx) -> Result<(), ErrorBody> {
    let mut session = ctx.session.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(v) = args.get("sit_min").and_then(Value::as_i64) {
        session.set_limit_minutes(v as u32);
    }
    if let Some(v) = args.get("stand_min").and_then(Value::as_i64) {
        session.set_stand_limit_minutes(v as u32);
    }
    Ok(())
}

/// Applies a profile of either kind. A missing profile file is an
/// `exec_failed`, not a `bad_args`: the name was well-formed, the desk simply
/// does not have it.
fn switch_profile(args: &Value, ctx: &CommandCtx) -> Result<(), ErrorBody> {
    let kind = args.get("kind").and_then(Value::as_str).unwrap_or_default();
    let name = args.get("name").and_then(Value::as_str).unwrap_or_default();
    let outcome = match kind {
        "ergonomic" => switch_ergonomic_profile_by_name(
            &ctx.app_data_dir,
            &ctx.comm_policy,
            &ctx.session,
            name,
        ),
        _ => switch_communication_profile_by_name(&ctx.app_data_dir, &ctx.comm_policy, name),
    };
    outcome.map_err(|e| ErrorBody::new("exec_failed", e))
}

// ─── Logging ────────────────────────────────────────────────────────────────

/// Strips anything that could forge a log line or a second field out of a
/// value that arrived over the network, and caps its length.
///
/// Command and viewer names are attacker-controlled until `admit` has run —
/// and denial lines are written for names that never passed `admit` at all.
fn sanitize(s: &str) -> String {
    let cleaned: String = s
        .chars()
        .take(32)
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '.'
            }
        })
        .collect();
    if cleaned.is_empty() {
        "unknown".to_string()
    } else {
        cleaned
    }
}

fn log_event(ctx: &CommandCtx, line: &str) {
    if let Some(events) = &ctx.events {
        events.log(line);
    }
}
