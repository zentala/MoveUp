//! health_probe.rs — What counts as a good build (E014-T03).
//!
//! Decision D5 of the E014 plan: a build has proved itself when it stays alive
//! for thirty seconds **and** answers `GET /display/api` with JSON inside the
//! health timeout. Neither half is enough on its own, and that is the whole
//! point of this module:
//!
//! - a process that starts, stays up and never serves is a broken frontend
//!   bundle or a panicking command handler, not a working build;
//! - a process that serves once and then exits is a crash loop that happened to
//!   answer one request before dying.
//!
//! So the probe watches for the whole window rather than sampling once. It
//! reads liveness and the HTTP surface through [`ProcessLiveness`], [`DisplayApi`]
//! and [`Clock`], which is what lets a test drive thirty seconds of observation
//! without waiting thirty seconds — the real implementations below are thin.
//!
//! The verdict converts to [`ProbeOutcome`] so it feeds straight into the
//! `last-known-good` marker (E014-T02). Wiring the probe to an actual install is
//! wave 3 of the epic and is blocked on PM3; nothing in this repo calls
//! [`probe`] yet.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::process::Child;
use std::time::{Duration, Instant};

use crate::last_known_good::ProbeOutcome;

/// How long a build must stay alive to count as good (D5).
pub const ALIVE_REQUIREMENT_SECS: u64 = 30;

/// How long one `/display/api` request may take before it counts as a miss.
pub const HTTP_TIMEOUT_SECS: u64 = 5;

/// Gap between observations during the window.
pub const POLL_INTERVAL_SECS: u64 = 1;

/// The endpoint that proves the app is serving, not merely running.
pub const DISPLAY_API_PATH: &str = "/display/api";

/// Timings of one probe run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProbeConfig {
    /// How long the process must stay alive.
    pub alive_requirement: Duration,
    /// Per-request timeout for `/display/api`.
    pub http_timeout: Duration,
    /// Gap between observations.
    pub poll_interval: Duration,
}

impl Default for ProbeConfig {
    fn default() -> Self {
        Self {
            alive_requirement: Duration::from_secs(ALIVE_REQUIREMENT_SECS),
            http_timeout: Duration::from_secs(HTTP_TIMEOUT_SECS),
            poll_interval: Duration::from_secs(POLL_INTERVAL_SECS),
        }
    }
}

/// What the probe concluded, and why.
///
/// The reason is kept because a supervisor log that says only "failed" cannot
/// tell a crash loop from a build that never served, and those need different
/// fixes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// Alive for the whole window and served JSON at least once.
    Good,
    /// The process was gone before the window closed.
    ExitedEarly {
        /// How long it lasted.
        alive_for: Duration,
        /// Whether it had managed to serve JSON before it died.
        served: bool,
    },
    /// Alive for the whole window, but `/display/api` never returned JSON.
    NeverServedJson,
}

impl Verdict {
    /// The marker-facing outcome. Only [`Verdict::Good`] promotes a build.
    pub fn outcome(&self) -> ProbeOutcome {
        match self {
            Verdict::Good => ProbeOutcome::Good,
            _ => ProbeOutcome::Failed,
        }
    }
}

/// Whether the build's process is still running.
pub trait ProcessLiveness {
    /// True while the process is running. A reaped or unknown process is dead.
    fn is_alive(&mut self) -> bool;
}

/// The build's `/display/api` surface.
pub trait DisplayApi {
    /// Fetches the endpoint, returning the response body.
    ///
    /// Every failure — refused connection, timeout, non-200 — is `Err`, because
    /// the probe treats them identically: not serving yet.
    fn fetch(&mut self, timeout: Duration) -> Result<String, String>;
}

/// Monotonic time, in milliseconds since an arbitrary origin.
pub trait Clock {
    /// Milliseconds elapsed since this clock's origin.
    fn now_millis(&self) -> u64;
    /// Waits for `duration`.
    fn sleep(&mut self, duration: Duration);
}

/// Runs the D5 probe over one observation window.
///
/// Returns as soon as the process dies; otherwise watches until the window
/// closes, remembering whether `/display/api` ever answered with JSON.
pub fn probe(
    process: &mut impl ProcessLiveness,
    api: &mut impl DisplayApi,
    clock: &mut impl Clock,
    config: &ProbeConfig,
) -> Verdict {
    let start = clock.now_millis();
    let mut served = false;

    loop {
        if !process.is_alive() {
            let alive_for = Duration::from_millis(clock.now_millis().saturating_sub(start));
            return Verdict::ExitedEarly { alive_for, served };
        }

        if !served {
            served = matches!(api.fetch(config.http_timeout), Ok(body) if is_json(&body));
        }

        let elapsed = Duration::from_millis(clock.now_millis().saturating_sub(start));
        if elapsed >= config.alive_requirement {
            break;
        }
        clock.sleep(config.poll_interval);
    }

    if served {
        Verdict::Good
    } else {
        Verdict::NeverServedJson
    }
}

/// Whether a response body is JSON. An empty body is not.
fn is_json(body: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(body).is_ok()
}

/// Wall-clock implementation of [`Clock`].
pub struct SystemClock {
    origin: Instant,
}

impl SystemClock {
    /// Starts a clock whose origin is now.
    pub fn new() -> Self {
        Self {
            origin: Instant::now(),
        }
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemClock {
    fn now_millis(&self) -> u64 {
        self.origin.elapsed().as_millis() as u64
    }

    fn sleep(&mut self, duration: Duration) {
        std::thread::sleep(duration);
    }
}

impl ProcessLiveness for Child {
    fn is_alive(&mut self) -> bool {
        matches!(self.try_wait(), Ok(None))
    }
}

/// Reads `/display/api` over plain HTTP/1.1 on the loopback interface.
///
/// Hand-rolled rather than run through `reqwest` because the probe runs on a
/// plain thread outside any tokio runtime, and building a runtime just to ask
/// one loopback question would panic when a caller already has one. The request
/// sends `Connection: close`, so the body is whatever arrives before EOF.
pub struct HttpDisplayApi {
    addr: SocketAddr,
}

impl HttpDisplayApi {
    /// Targets [`DISPLAY_API_PATH`] on `addr`.
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }
}

impl DisplayApi for HttpDisplayApi {
    fn fetch(&mut self, timeout: Duration) -> Result<String, String> {
        let mut stream =
            TcpStream::connect_timeout(&self.addr, timeout).map_err(|err| err.to_string())?;
        stream
            .set_read_timeout(Some(timeout))
            .map_err(|err| err.to_string())?;
        stream
            .set_write_timeout(Some(timeout))
            .map_err(|err| err.to_string())?;

        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nAccept: application/json\r\nConnection: close\r\n\r\n",
            DISPLAY_API_PATH, self.addr
        );
        stream
            .write_all(request.as_bytes())
            .map_err(|err| err.to_string())?;

        let mut raw = Vec::new();
        stream.read_to_end(&mut raw).map_err(|err| err.to_string())?;
        split_ok_body(&String::from_utf8_lossy(&raw))
    }
}

/// Body of a 200 response, or an error describing what came back instead.
fn split_ok_body(response: &str) -> Result<String, String> {
    let (head, body) = response
        .split_once("\r\n\r\n")
        .ok_or_else(|| "malformed response: no header terminator".to_string())?;
    let status = head.lines().next().unwrap_or_default();
    if !status.contains(" 200") {
        return Err(format!("unexpected status line: {status}"));
    }
    Ok(body.to_string())
}
