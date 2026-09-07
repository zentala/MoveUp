//! remote_protocol.rs — the versioned message contract for both remote
//! transports (E022, relay + LAN).
//!
//! One envelope `{v, type, id, ts, payload}` wraps every WebSocket message in
//! both directions on both transports. The Rust structs here, the zod schemas
//! in `src/remote/protocol.ts`, and the JSON files in
//! `tests/fixtures/relay-protocol/` are three views of the same contract; the
//! fixtures are the arbiter, and both test suites round-trip all of them.
//!
//! Nothing here talks to a socket. Transport lives in `relay_client.rs`
//! (desk) and `remote_server.rs` (LAN).

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

/// Wire version this build speaks. A peer announcing anything else is closed
/// with [`CLOSE_PROTOCOL_ERROR`] rather than guessed at.
pub const PROTOCOL_VERSION: u32 = 1;

// ─── Close codes (WebSocket application range) ──────────────────────────────

/// Malformed message or unsupported `v`.
pub const CLOSE_PROTOCOL_ERROR: u16 = 4400;
/// `hello` missing, late or carrying an invalid token.
pub const CLOSE_UNAUTHENTICATED: u16 = 4401;
/// License expired or revoked. The desk must NOT reconnect on this.
pub const CLOSE_UNENTITLED: u16 = 4402;
/// This device's token was revoked. The desk must NOT reconnect on this.
pub const CLOSE_REVOKED: u16 = 4403;
/// A newer desk connection took the room. The old one must NOT reconnect.
pub const CLOSE_REPLACED: u16 = 4409;
/// Too many messages.
pub const CLOSE_RATE_LIMITED: u16 = 4429;
/// Relay shedding load; reconnect with backoff.
pub const CLOSE_SHEDDING: u16 = 4503;

// ─── Envelope ───────────────────────────────────────────────────────────────

/// The one wrapper every message travels in.
///
/// `ts` is the sender's unix-milliseconds and is used for latency measurement
/// and staleness rejection only — never for ordering or logic.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Envelope<T> {
    pub v: u32,
    #[serde(rename = "type")]
    pub msg_type: MessageType,
    pub id: String,
    pub ts: i64,
    pub payload: T,
}

/// Message discriminator.
///
/// [`MessageType::Unknown`] exists so a peer from a later minor version can
/// send a type we do not know without the whole envelope failing to parse —
/// forward compatibility is a property of the envelope, not of the payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageType {
    Hello,
    Welcome,
    Event,
    DeskStatus,
    Command,
    CommandResult,
    Ping,
    Pong,
    Error,
    Unknown(String),
}

impl MessageType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Hello => "hello",
            Self::Welcome => "welcome",
            Self::Event => "event",
            Self::DeskStatus => "desk_status",
            Self::Command => "command",
            Self::CommandResult => "command_result",
            Self::Ping => "ping",
            Self::Pong => "pong",
            Self::Error => "error",
            Self::Unknown(s) => s,
        }
    }

    fn from_wire(s: &str) -> Self {
        match s {
            "hello" => Self::Hello,
            "welcome" => Self::Welcome,
            "event" => Self::Event,
            "desk_status" => Self::DeskStatus,
            "command" => Self::Command,
            "command_result" => Self::CommandResult,
            "ping" => Self::Ping,
            "pong" => Self::Pong,
            "error" => Self::Error,
            other => Self::Unknown(other.to_string()),
        }
    }
}

impl Serialize for MessageType {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for MessageType {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(Self::from_wire(&String::deserialize(d)?))
    }
}

// ─── Payloads ───────────────────────────────────────────────────────────────

/// Which end of the room a socket is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export_to = "../../src/generated/"))]
pub enum Role {
    Desk,
    Viewer,
}

/// Self-description of the connecting client, for support and telemetry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export_to = "../../src/generated/"))]
pub struct ClientInfo {
    pub app: String,
    pub version: String,
}

/// First message on every socket. Must arrive within 5 s of the upgrade.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export_to = "../../src/generated/"))]
pub struct Hello {
    pub role: Role,
    pub desk_id: String,
    /// Opaque bearer token. Never logged, never put in a URL.
    pub token: String,
    pub client: ClientInfo,
}

/// The relay's answer to a valid `hello`.
///
/// `snapshot` is the last state the room saw, so a viewer that connects while
/// the desk is offline shows stale data marked stale instead of an endless
/// "Reconnecting…". `null` means the room has never seen a desk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export_to = "../../src/generated/"))]
pub struct Welcome {
    pub role: Role,
    pub desk_id: String,
    pub desk_online: bool,
    pub viewer_count: u32,
    #[cfg_attr(test, ts(type = "Record<string, unknown> | null"))]
    pub snapshot: Option<Value>,
    pub snapshot_ts: Option<i64>,
}

/// Desk presence, pushed to viewers on connect and disconnect.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export_to = "../../src/generated/"))]
pub struct DeskStatus {
    pub online: bool,
    pub since: i64,
}

/// A viewer asking the desk to do one of the allowlisted things.
///
/// `viewer_id` is `None` on the wire from the viewer; the relay fills it in
/// before forwarding, so the desk always knows who asked.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export_to = "../../src/generated/"))]
pub struct Command {
    pub name: String,
    #[cfg_attr(test, ts(type = "Record<string, unknown>"))]
    pub args: Value,
    pub viewer_id: Option<String>,
}

/// The desk's answer, routed back to the originating viewer only.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export_to = "../../src/generated/"))]
pub struct CommandResult {
    /// The `id` of the `command` envelope this answers.
    pub command_id: String,
    pub ok: bool,
    pub error: Option<ErrorBody>,
}

/// A non-fatal error, and the body of every failed REST response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export_to = "../../src/generated/"))]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
}

impl ErrorBody {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
        }
    }
}

// ─── Command allowlist, as data ─────────────────────────────────────────────

/// An integer argument and the range it must fall in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IntArg {
    pub name: &'static str,
    pub min: i64,
    pub max: i64,
}

/// A string argument. Empty `allowed` means "any profile-name-shaped slug".
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StringArg {
    pub name: &'static str,
    pub allowed: &'static [&'static str],
    pub max_len: usize,
}

/// One allowlisted command and the exact shape of its arguments.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CommandSpec {
    pub name: &'static str,
    pub ints: &'static [IntArg],
    pub strings: &'static [StringArg],
    /// `set_limits` is meaningless with no argument, so `{}` is rejected.
    pub require_one_int: bool,
}

/// Every command a remote viewer may ask for. Anything else is
/// `unknown_command` — there is no generic write path (ADR 023).
pub const COMMAND_ALLOWLIST: &[CommandSpec] = &[
    CommandSpec {
        name: "ack_alert",
        ints: &[],
        strings: &[],
        require_one_int: false,
    },
    CommandSpec {
        name: "set_limits",
        ints: &[
            IntArg {
                name: "sit_min",
                min: 5,
                max: 240,
            },
            IntArg {
                name: "stand_min",
                min: 1,
                max: 120,
            },
        ],
        strings: &[],
        require_one_int: true,
    },
    CommandSpec {
        name: "switch_profile",
        ints: &[],
        strings: &[
            StringArg {
                name: "kind",
                allowed: &["ergonomic", "communication"],
                max_len: 16,
            },
            StringArg {
                name: "name",
                allowed: &[],
                max_len: 32,
            },
        ],
        require_one_int: false,
    },
];

/// Longest a profile name may be when no explicit allowlist constrains it.
fn is_slug(s: &str, max_len: usize) -> bool {
    !s.is_empty()
        && s.len() <= max_len
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

pub fn spec_for(name: &str) -> Option<&'static CommandSpec> {
    COMMAND_ALLOWLIST.iter().find(|s| s.name == name)
}

/// Rejects a message whose `v` this build does not speak.
pub fn check_version(v: u32) -> Result<(), ErrorBody> {
    if v == PROTOCOL_VERSION {
        Ok(())
    } else {
        Err(ErrorBody::new(
            "unsupported_version",
            format!("expected protocol v{PROTOCOL_VERSION}, got v{v}"),
        ))
    }
}

/// Checks a command name and its arguments against [`COMMAND_ALLOWLIST`].
///
/// Runs on the relay *and* again on the desk — the desk never trusts that the
/// relay validated anything.
pub fn validate_command(name: &str, args: &Value) -> Result<(), ErrorBody> {
    let spec =
        spec_for(name).ok_or_else(|| ErrorBody::new("unknown_command", format!("no such command: {name}")))?;
    let map = args
        .as_object()
        .ok_or_else(|| ErrorBody::new("bad_args", "args must be an object"))?;

    for key in map.keys() {
        let known = spec.ints.iter().any(|a| a.name == key)
            || spec.strings.iter().any(|a| a.name == key);
        if !known {
            return Err(ErrorBody::new("bad_args", format!("unknown argument: {key}")));
        }
    }

    let mut ints_present = 0;
    for arg in spec.ints {
        let Some(v) = map.get(arg.name) else { continue };
        let n = v
            .as_i64()
            .ok_or_else(|| ErrorBody::new("bad_args", format!("{} must be an integer", arg.name)))?;
        if n < arg.min || n > arg.max {
            return Err(ErrorBody::new(
                "bad_args",
                format!("{} must be {}..={}", arg.name, arg.min, arg.max),
            ));
        }
        ints_present += 1;
    }
    if spec.require_one_int && ints_present == 0 {
        return Err(ErrorBody::new(
            "bad_args",
            format!("{name} needs at least one argument"),
        ));
    }

    for arg in spec.strings {
        let v = map
            .get(arg.name)
            .ok_or_else(|| ErrorBody::new("bad_args", format!("missing argument: {}", arg.name)))?;
        let s = v
            .as_str()
            .ok_or_else(|| ErrorBody::new("bad_args", format!("{} must be a string", arg.name)))?;
        let ok = if arg.allowed.is_empty() {
            is_slug(s, arg.max_len)
        } else {
            arg.allowed.contains(&s)
        };
        if !ok {
            return Err(ErrorBody::new(
                "bad_args",
                format!("{} has an unacceptable value", arg.name),
            ));
        }
    }

    Ok(())
}
