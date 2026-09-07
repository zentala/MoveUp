//! relay_commands_tests.rs — the desk's side of a remote command (E022-T07).
//!
//! Every test drives the real [`CommandCtx`]: a real `SessionManager`, a real
//! `CommunicationPolicy`, a real `EventLogger` writing into a `TempDir`, and
//! real profile JSON on disk. Nothing on the desk side of the seam is stubbed,
//! because the bugs this task can produce — a limit that never lands, a log
//! line that never appears — are exactly the ones a stub would hide.

#![cfg(test)]

use std::path::Path;
use std::sync::{Arc, Mutex};

use chrono::Local;
use serde_json::{json, Value};
use tempfile::TempDir;

use crate::alert_popup::AlertPopup;
use crate::communication_policy::CommunicationPolicy;
use crate::communication_profile::CommunicationProfile;
use crate::ergonomic_profile::ErgonomicProfile;
use crate::event_logger::EventLogger;
use crate::relay_commands::{execute, CommandCtx, CommandExecutor, MAX_COMMAND_AGE_MS};
use crate::remote_protocol::{Command, CommandResult, Envelope, MessageType, PROTOCOL_VERSION};
use crate::session::SessionManager;

const NOW: i64 = 1_757_000_000_000;

// ─── Fixture ────────────────────────────────────────────────────────────────

struct Fx {
    ctx: CommandCtx,
    _dirs: (TempDir, TempDir),
    logs: std::path::PathBuf,
}

impl Fx {
    fn new() -> Self {
        let data = TempDir::new().expect("data dir");
        let logs = TempDir::new().expect("log dir");
        let ctx = CommandCtx {
            session: Arc::new(Mutex::new(SessionManager::new())),
            comm_policy: Arc::new(Mutex::new(CommunicationPolicy::new(
                CommunicationProfile::default(),
                ErgonomicProfile::default(),
            ))),
            alert_popup: Arc::new(Mutex::new(AlertPopup::new())),
            app_data_dir: data.path().to_path_buf(),
            events: Some(Arc::new(EventLogger::new(logs.path().to_path_buf()))),
        };
        let logs_path = logs.path().to_path_buf();
        Self {
            ctx,
            _dirs: (data, logs),
            logs: logs_path,
        }
    }

    /// Runs a command whose `ts` is current.
    fn run(&self, name: &str, args: Value) -> CommandResult {
        self.run_at(name, args, NOW)
    }

    fn run_at(&self, name: &str, args: Value, ts: i64) -> CommandResult {
        execute(&envelope(name, args, ts), &self.ctx, NOW)
    }

    fn sit_limit_secs(&self) -> i64 {
        self.ctx.session.lock().unwrap().state.session_limit_secs
    }

    fn stand_limit_secs(&self) -> i64 {
        self.ctx.session.lock().unwrap().state.stand_limit_secs
    }

    /// Today's `events.log`, or an empty string if nothing was ever written.
    fn events(&self) -> String {
        let day = Local::now().format("%Y-%m-%d").to_string();
        std::fs::read_to_string(self.logs.join(day).join("events.log")).unwrap_or_default()
    }

    /// Writes an ergonomic profile whose sitting limit is `sitting_secs`.
    fn write_ergo_profile(&self, id: &str, sitting_secs: u32) {
        let mut p = ErgonomicProfile::default();
        p.limits.sitting_secs = sitting_secs;
        write_profile(&self.ctx.app_data_dir, "ergonomic", id, &p);
    }

    fn write_comm_profile(&self, id: &str, name: &str) {
        let mut p = CommunicationProfile::default();
        p.name = name.to_string();
        write_profile(&self.ctx.app_data_dir, "communication", id, &p);
    }
}

fn write_profile<T: serde::Serialize>(data_dir: &Path, kind: &str, id: &str, profile: &T) {
    let dir = data_dir.join("profiles").join(kind);
    std::fs::create_dir_all(&dir).expect("profile dir");
    std::fs::write(
        dir.join(format!("{id}.json")),
        serde_json::to_string(profile).expect("serialize profile"),
    )
    .expect("write profile");
}

fn envelope(name: &str, args: Value, ts: i64) -> Envelope<Command> {
    Envelope {
        v: PROTOCOL_VERSION,
        msg_type: MessageType::Command,
        id: "cmd-1".to_string(),
        ts,
        payload: Command {
            name: name.to_string(),
            args,
            viewer_id: Some("v-7".to_string()),
        },
    }
}

fn err_code(r: &CommandResult) -> String {
    r.error.as_ref().map(|e| e.code.clone()).unwrap_or_default()
}

// ─── set_limits ─────────────────────────────────────────────────────────────

#[test]
fn set_limits_applies_both_limits() {
    let fx = Fx::new();
    let r = fx.run("set_limits", json!({ "sit_min": 50, "stand_min": 15 }));

    assert!(r.ok, "expected ok, got {:?}", r.error);
    assert_eq!(r.command_id, "cmd-1");
    assert_eq!(fx.sit_limit_secs(), 50 * 60);
    assert_eq!(fx.stand_limit_secs(), 15 * 60);
}

#[test]
fn set_limits_applies_only_the_argument_it_was_given() {
    let fx = Fx::new();
    let before = fx.stand_limit_secs();
    assert!(fx.run("set_limits", json!({ "sit_min": 45 })).ok);

    assert_eq!(fx.sit_limit_secs(), 45 * 60);
    assert_eq!(fx.stand_limit_secs(), before, "stand limit must be untouched");
}

/// The bound lives in `COMMAND_ALLOWLIST`; this asserts the desk enforces it
/// itself rather than trusting that the relay did.
#[test]
fn set_limits_rejects_out_of_range_value() {
    let fx = Fx::new();
    let before = fx.sit_limit_secs();
    let r = fx.run("set_limits", json!({ "sit_min": 9000 }));

    assert!(!r.ok);
    assert_eq!(err_code(&r), "bad_args");
    assert_eq!(fx.sit_limit_secs(), before, "a rejected command must change nothing");
}

#[test]
fn set_limits_rejects_empty_args() {
    let fx = Fx::new();
    let r = fx.run("set_limits", json!({}));
    assert!(!r.ok);
    assert_eq!(err_code(&r), "bad_args");
}

#[test]
fn set_limits_rejects_an_unknown_argument() {
    let fx = Fx::new();
    let r = fx.run("set_limits", json!({ "sit_min": 45, "drop_table": 1 }));
    assert!(!r.ok);
    assert_eq!(err_code(&r), "bad_args");
}

// ─── ack_alert ──────────────────────────────────────────────────────────────

#[test]
fn ack_alert_snoozes_the_policy() {
    let fx = Fx::new();
    assert!(!fx.ctx.comm_policy.lock().unwrap().is_snoozed());

    let r = fx.run("ack_alert", json!({}));

    assert!(r.ok, "expected ok, got {:?}", r.error);
    assert!(
        fx.ctx.comm_policy.lock().unwrap().is_snoozed(),
        "a remote ack must start the snooze, exactly as a click on the popup does"
    );
}

/// The remote path calls `CommunicationPolicy::dismiss` itself, so it must not
/// also raise the popup's `user_dismissed` flag — `tray_controller` polls that
/// flag and would dismiss a second time, advancing `snooze_index` twice for
/// one acknowledgement.
#[test]
fn ack_alert_does_not_leave_a_pending_user_dismiss_for_the_tray() {
    let fx = Fx::new();
    assert!(fx.run("ack_alert", json!({})).ok);

    assert!(
        !fx.ctx.alert_popup.lock().unwrap().take_user_dismissed(),
        "the tray must not see a second dismiss for the same ack"
    );
}

// ─── switch_profile ─────────────────────────────────────────────────────────

#[test]
fn switch_profile_applies_an_ergonomic_profile_from_disk() {
    let fx = Fx::new();
    fx.write_ergo_profile("relaxed", 3600);

    let r = fx.run(
        "switch_profile",
        json!({ "kind": "ergonomic", "name": "relaxed" }),
    );

    assert!(r.ok, "expected ok, got {:?}", r.error);
    assert_eq!(
        fx.ctx.comm_policy.lock().unwrap().ergo_profile().limits.sitting_secs,
        3600
    );
    assert_eq!(fx.sit_limit_secs(), 3600, "the engine limit follows the profile");
}

#[test]
fn switch_profile_applies_a_communication_profile_from_disk() {
    let fx = Fx::new();
    fx.write_comm_profile("gentle", "Gentle");

    let r = fx.run(
        "switch_profile",
        json!({ "kind": "communication", "name": "gentle" }),
    );

    assert!(r.ok, "expected ok, got {:?}", r.error);
    assert_eq!(fx.ctx.comm_policy.lock().unwrap().comm_profile().name, "Gentle");
}

/// A profile that is not on disk must fail loudly. `load_profile` answers with
/// `Default` for a missing file, so without the existence check the desk would
/// report success while quietly resetting the user to the default profile.
#[test]
fn switch_profile_fails_for_a_profile_that_is_not_installed() {
    let fx = Fx::new();
    let before = fx.ctx.comm_policy.lock().unwrap().ergo_profile().limits.sitting_secs;

    let r = fx.run(
        "switch_profile",
        json!({ "kind": "ergonomic", "name": "nope" }),
    );

    assert!(!r.ok);
    assert_eq!(err_code(&r), "exec_failed");
    assert_eq!(
        fx.ctx.comm_policy.lock().unwrap().ergo_profile().limits.sitting_secs,
        before
    );
}

#[test]
fn switch_profile_rejects_a_path_traversal_name() {
    let fx = Fx::new();
    let r = fx.run(
        "switch_profile",
        json!({ "kind": "ergonomic", "name": "../../etc/passwd" }),
    );
    assert!(!r.ok);
    assert_eq!(err_code(&r), "bad_args", "the allowlist rejects it before the loader sees it");
}

#[test]
fn switch_profile_rejects_an_unknown_kind() {
    let fx = Fx::new();
    let r = fx.run("switch_profile", json!({ "kind": "overlay", "name": "x" }));
    assert!(!r.ok);
    assert_eq!(err_code(&r), "bad_args");
}

// ─── Admission ──────────────────────────────────────────────────────────────

#[test]
fn rejects_a_command_older_than_the_staleness_window() {
    let fx = Fx::new();
    let before = fx.sit_limit_secs();
    let r = fx.run_at("set_limits", json!({ "sit_min": 45 }), NOW - MAX_COMMAND_AGE_MS - 1);

    assert!(!r.ok);
    assert_eq!(err_code(&r), "stale_command");
    assert_eq!(fx.sit_limit_secs(), before, "a stale command must not be executed");
}

/// Symmetry matters: checking only the past would let a timestamp far in the
/// future stay valid forever.
#[test]
fn rejects_a_command_stamped_far_in_the_future() {
    let fx = Fx::new();
    let r = fx.run_at("set_limits", json!({ "sit_min": 45 }), NOW + MAX_COMMAND_AGE_MS + 1);
    assert!(!r.ok);
    assert_eq!(err_code(&r), "stale_command");
}

#[test]
fn accepts_a_command_at_the_edge_of_the_staleness_window() {
    let fx = Fx::new();
    let r = fx.run_at("set_limits", json!({ "sit_min": 45 }), NOW - MAX_COMMAND_AGE_MS);
    assert!(r.ok, "expected ok, got {:?}", r.error);
}

#[test]
fn rejects_an_unknown_command_name() {
    let fx = Fx::new();
    let r = fx.run("shutdown", json!({}));
    assert!(!r.ok);
    assert_eq!(err_code(&r), "unknown_command");
}

#[test]
fn rejects_a_command_on_an_unsupported_protocol_version() {
    let fx = Fx::new();
    let mut env = envelope("set_limits", json!({ "sit_min": 45 }), NOW);
    env.v = PROTOCOL_VERSION + 1;

    let r = execute(&env, &fx.ctx, NOW);

    assert!(!r.ok);
    assert_eq!(err_code(&r), "unsupported_version");
}

// ─── The event log ──────────────────────────────────────────────────────────

#[test]
fn logs_an_ok_line_naming_the_command_and_the_viewer() {
    let fx = Fx::new();
    assert!(fx.run("set_limits", json!({ "sit_min": 45 })).ok);

    assert!(
        fx.events().contains("REMOTE set_limits viewer=v-7 ok"),
        "events.log was: {:?}",
        fx.events()
    );
}

#[test]
fn logs_an_err_line_carrying_the_failure_code() {
    let fx = Fx::new();
    assert!(!fx
        .run("switch_profile", json!({ "kind": "ergonomic", "name": "nope" }))
        .ok);

    assert!(
        fx.events().contains("REMOTE switch_profile viewer=v-7 err=exec_failed"),
        "events.log was: {:?}",
        fx.events()
    );
}

#[test]
fn logs_a_denied_line_for_a_refused_command() {
    let fx = Fx::new();
    assert!(!fx.run("shutdown", json!({})).ok);

    let log = fx.events();
    assert!(log.contains("REMOTE DENIED shutdown"), "events.log was: {log:?}");
    assert!(!log.contains(" ok"), "a denial must not also read as a success");
}

/// A command name arrives over the network. Without sanitising it, an attacker
/// could inject whole lines into `events.log` and forge, say, a success record.
#[test]
fn a_hostile_command_name_cannot_forge_a_log_line() {
    let fx = Fx::new();
    let r = fx.run("evil\nREMOTE set_limits viewer=v-7 ok", json!({}));

    assert!(!r.ok);
    let log = fx.events();
    assert_eq!(log.lines().count(), 1, "one command, one line — was: {log:?}");
    assert!(log.contains("REMOTE DENIED evil."), "events.log was: {log:?}");
}

/// The log is a side effect, not the point. A desk with no event logger yet
/// must still execute and answer.
#[test]
fn runs_without_an_event_logger() {
    let mut fx = Fx::new();
    fx.ctx.events = None;

    let r = fx.run("set_limits", json!({ "sit_min": 45 }));

    assert!(r.ok, "expected ok, got {:?}", r.error);
    assert_eq!(fx.sit_limit_secs(), 45 * 60);
}

// ─── The trait the relay client calls ───────────────────────────────────────

/// `relay_client.rs` only ever sees `dyn CommandExecutor`; this asserts the
/// real context is reachable through that seam, so the client's wiring is not
/// tested against a shape only the tests have.
#[test]
fn command_ctx_is_usable_as_a_dyn_executor() {
    let fx = Fx::new();
    let exec: &dyn CommandExecutor = &fx.ctx;

    // No injected clock here on purpose: the trait reads the real one, so this
    // also proves a freshly stamped command is not accidentally stale.
    let now = chrono::Utc::now().timestamp_millis();
    let r = exec.execute(&envelope("set_limits", json!({ "sit_min": 45 }), now));

    assert!(r.ok, "expected ok, got {:?}", r.error);
    assert_eq!(fx.sit_limit_secs(), 45 * 60);
}
