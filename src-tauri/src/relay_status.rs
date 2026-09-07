//! relay_status.rs — what the relay connection is doing, and when to retry.
//!
//! Two things live here because they are one decision: a close code says what
//! happened, and what happened decides whether reconnecting is honest or is a
//! silent retry against a dead credential. `4402`/`4403`/`4409` are terminal —
//! the desk stops and says so in Settings rather than hammering the relay
//! forever (PLAN.md §Desktop outbound connection contract, step 5).
//!
//! Nothing here touches a socket; [`crate::relay_client`] drives it.

use std::time::Duration;

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::remote_protocol::{CLOSE_REPLACED, CLOSE_REVOKED, CLOSE_UNENTITLED};

/// First retry waits this long.
pub const BACKOFF_BASE: Duration = Duration::from_secs(1);
/// Retries never wait longer than this.
pub const BACKOFF_MAX: Duration = Duration::from_secs(60);
/// Each delay is spread ±20 % so a relay restart does not bring every desk
/// back in the same millisecond.
pub const BACKOFF_JITTER: f64 = 0.20;

/// The state Settings and the Debug tab render.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export_to = "../../src/generated/"))]
pub enum RelayState {
    /// Turned off, or no credential to connect with.
    Disabled,
    Connecting,
    Online,
    /// License expired or revoked — terminal.
    Unentitled,
    /// This desk's token was revoked — terminal.
    Revoked,
    /// Another instance took the room — terminal.
    Replaced,
    /// Transport failure; the client keeps retrying with backoff.
    Error,
}

impl RelayState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Connecting => "connecting",
            Self::Online => "online",
            Self::Unentitled => "unentitled",
            Self::Revoked => "revoked",
            Self::Replaced => "replaced",
            Self::Error => "error",
        }
    }
}

/// The payload `get_relay_status` returns (T06 exposes it over IPC).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export_to = "../../src/generated/"))]
pub struct RelayStatus {
    pub enabled: bool,
    pub state: RelayState,
    /// Unix milliseconds the current `state` was entered.
    pub since: i64,
    pub viewers_online: u32,
    /// Last transport or close reason. Never carries a token.
    pub last_error: Option<String>,
}

impl Default for RelayStatus {
    fn default() -> Self {
        Self {
            enabled: false,
            state: RelayState::Disabled,
            since: 0,
            viewers_online: 0,
            last_error: None,
        }
    }
}

impl RelayStatus {
    /// Moves to `state` at `now_ms`, replacing `last_error` with `error`.
    ///
    /// Leaving a non-`Online` state clears `viewers_online` — a stale viewer
    /// count next to "reconnecting" reads as if phones were still attached.
    pub fn enter(&mut self, state: RelayState, now_ms: i64, error: Option<String>) {
        if state != RelayState::Online {
            self.viewers_online = 0;
        }
        self.state = state;
        self.since = now_ms;
        self.last_error = error;
    }
}

/// Maps a WebSocket close code onto the state the user should see.
///
/// Anything not explicitly terminal is [`RelayState::Error`] — an unknown
/// code is a transport problem, never a reason to look connected.
pub fn state_for_close_code(code: u16) -> RelayState {
    match code {
        CLOSE_UNENTITLED => RelayState::Unentitled,
        CLOSE_REVOKED => RelayState::Revoked,
        CLOSE_REPLACED => RelayState::Replaced,
        _ => RelayState::Error,
    }
}

/// Whether the client may open another connection while in `state`.
pub fn should_reconnect(state: RelayState) -> bool {
    !matches!(
        state,
        RelayState::Disabled
            | RelayState::Unentitled
            | RelayState::Revoked
            | RelayState::Replaced
            | RelayState::Online
    )
}

/// Exponential retry schedule: 1 s, 2 s, 4 s … capped at 60 s, each ±20 %.
#[derive(Debug, Clone, Default)]
pub struct Backoff {
    attempt: u32,
}

impl Backoff {
    pub fn new() -> Self {
        Self::default()
    }

    /// Called on every `welcome` — a working connection erases the history of
    /// failed ones.
    pub fn reset(&mut self) {
        self.attempt = 0;
    }

    pub fn attempt(&self) -> u32 {
        self.attempt
    }

    /// The un-jittered delay for the current attempt.
    pub fn base_delay(&self) -> Duration {
        let secs = BACKOFF_BASE
            .as_secs()
            .saturating_mul(1u64 << self.attempt.min(16));
        Duration::from_secs(secs.min(BACKOFF_MAX.as_secs()))
    }

    /// The delay to actually sleep, then advances the attempt counter.
    pub fn next_delay(&mut self) -> Duration {
        let base = self.base_delay();
        self.attempt = self.attempt.saturating_add(1);
        apply_jitter(base, rand::rng().random_range(-1.0..=1.0))
    }
}

/// Spreads `base` by `factor` (-1.0 ..= 1.0) of [`BACKOFF_JITTER`].
pub fn apply_jitter(base: Duration, factor: f64) -> Duration {
    let f = factor.clamp(-1.0, 1.0);
    Duration::from_secs_f64(base.as_secs_f64() * (1.0 + BACKOFF_JITTER * f))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::remote_protocol::{CLOSE_PROTOCOL_ERROR, CLOSE_RATE_LIMITED, CLOSE_SHEDDING};

    #[test]
    fn terminal_close_codes_map_and_stop_the_loop() {
        for (code, state) in [
            (CLOSE_UNENTITLED, RelayState::Unentitled),
            (CLOSE_REVOKED, RelayState::Revoked),
            (CLOSE_REPLACED, RelayState::Replaced),
        ] {
            assert_eq!(state_for_close_code(code), state);
            assert!(!should_reconnect(state), "{code} must not reconnect");
        }
    }

    #[test]
    fn retryable_close_codes_map_to_error_and_reconnect() {
        for code in [CLOSE_PROTOCOL_ERROR, CLOSE_RATE_LIMITED, CLOSE_SHEDDING, 1006, 1000] {
            assert_eq!(state_for_close_code(code), RelayState::Error);
            assert!(should_reconnect(RelayState::Error), "{code} should reconnect");
        }
    }

    #[test]
    fn backoff_doubles_then_caps_at_sixty_seconds() {
        let mut b = Backoff::new();
        let seen: Vec<u64> = (0..10)
            .map(|_| {
                let d = b.base_delay().as_secs();
                b.next_delay();
                d
            })
            .collect();
        assert_eq!(seen, vec![1, 2, 4, 8, 16, 32, 60, 60, 60, 60]);
    }

    #[test]
    fn backoff_reset_returns_to_one_second() {
        let mut b = Backoff::new();
        for _ in 0..5 {
            b.next_delay();
        }
        assert!(b.attempt() > 0);
        b.reset();
        assert_eq!(b.base_delay(), BACKOFF_BASE);
    }

    #[test]
    fn jitter_stays_within_twenty_percent() {
        let base = Duration::from_secs(10);
        assert_eq!(apply_jitter(base, 0.0), base);
        assert_eq!(apply_jitter(base, 1.0), Duration::from_secs(12));
        assert_eq!(apply_jitter(base, -1.0), Duration::from_secs(8));
        let mut b = Backoff::new();
        for _ in 0..200 {
            let expected = b.base_delay().as_secs_f64();
            let actual = b.next_delay().as_secs_f64();
            assert!(
                actual >= expected * 0.8 - 1e-9 && actual <= expected * 1.2 + 1e-9,
                "{actual} outside ±20 % of {expected}"
            );
        }
    }

    #[test]
    fn leaving_online_clears_the_viewer_count() {
        let mut s = RelayStatus::default();
        s.enter(RelayState::Online, 10, None);
        s.viewers_online = 2;
        s.enter(RelayState::Error, 20, Some("ws: reset".into()));
        assert_eq!(s.viewers_online, 0);
        assert_eq!(s.since, 20);
        assert_eq!(s.last_error.as_deref(), Some("ws: reset"));
    }
}
