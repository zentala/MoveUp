//! remote_auth.rs — Shared-secret guard for the remote server's write
//! endpoints (E021-T03).
//!
//! Reads on `/display/ws` and `/display/api` stay open: they only expose what
//! the phone on the same LAN already sees on the desk. Anything that *writes*
//! into the app — the health inlet here, the voice inlet in T06 — carries a
//! shared secret in the `X-Desk-Token` header instead.
//!
//! Two properties this module exists to hold:
//!
//! - **No token configured means closed, not open.** A missing
//!   `DESK_REMOTE_TOKEN` answers `503`, never `200`. An unconfigured guard
//!   that silently accepts every request is the "silence looks like success"
//!   failure applied to auth.
//! - **The comparison is constant-time.** A byte-by-byte `==` returns as soon
//!   as it finds a mismatch, so response latency leaks how many leading bytes
//!   a guess got right. [`constant_time_eq`] folds every byte either way.
//!   Written by hand rather than pulling in `subtle`: it is nine lines and the
//!   crate is not already in the dependency graph.

use axum::http::{HeaderMap, StatusCode};

/// Header the client presents the shared secret in.
pub const TOKEN_HEADER: &str = "x-desk-token";

/// Environment variable holding the expected shared secret.
pub const TOKEN_ENV: &str = "DESK_REMOTE_TOKEN";

/// Reads the expected token from the environment.
///
/// A missing variable and a blank one are the same thing — "not configured" —
/// so both map to `None` and every guarded route answers `503`.
pub fn token_from_env() -> Option<String> {
    std::env::var(TOKEN_ENV)
        .ok()
        .map(|raw| raw.trim().to_string())
        .filter(|token| !token.is_empty())
}

/// Rejects a request that does not present `expected` in `X-Desk-Token`.
///
/// - `expected` is `None` or blank → `503 Service Unavailable`; the inlet is
///   not configured and must not accept anything.
/// - header absent, unreadable as ASCII, or wrong → `401 Unauthorized`.
/// - otherwise `Ok(())`.
pub fn require_token(headers: &HeaderMap, expected: Option<&str>) -> Result<(), StatusCode> {
    let Some(expected) = expected.filter(|token| !token.is_empty()) else {
        log::warn!(
            "remote write endpoint refused: {} is not set; set it to enable the inlet",
            TOKEN_ENV
        );
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    };

    let presented = headers
        .get(TOKEN_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");

    if constant_time_eq(presented.as_bytes(), expected.as_bytes()) {
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

/// Compares two byte strings without an early return.
///
/// Length difference is folded into the same accumulator as the byte
/// differences, so neither the number of matching leading bytes nor the
/// length of the presented value is observable through timing.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    let mut diff: usize = a.len() ^ b.len();
    for i in 0..a.len().max(b.len()) {
        let x = a.get(i).copied().unwrap_or(0);
        let y = b.get(i).copied().unwrap_or(0);
        diff |= usize::from(x ^ y);
    }
    diff == 0
}

// Tests in remote_auth_tests.rs
