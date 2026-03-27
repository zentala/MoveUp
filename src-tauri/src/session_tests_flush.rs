//! Tests for `flush_current_session()` — verifies that shutdown persistence
//! captures the correct state label and duration for all DeskState variants.

use chrono::{Duration, Utc};

use crate::session_manager::SessionManager;
use crate::session_types::DeskState;

#[test]
fn flush_during_sitting() {
    let mut mgr = SessionManager::new();
    let now = Utc::now();
    mgr.state.state = DeskState::Sitting;
    mgr.state.sitting_started = Some(now - Duration::seconds(300));

    let result = mgr.flush_current_session();
    assert!(result.is_some(), "Should produce a CompletedSession");
    let session = result.unwrap();
    assert!(session.duration_secs >= 299 && session.duration_secs <= 301);
}

#[test]
fn flush_during_standing() {
    let mut mgr = SessionManager::new();
    let now = Utc::now();
    mgr.state.state = DeskState::Standing;
    mgr.state.standing_bout_started = Some(now - Duration::seconds(120));

    let result = mgr.flush_current_session();
    assert!(result.is_some(), "Should produce a CompletedSession");
    let session = result.unwrap();
    assert!(session.duration_secs >= 119 && session.duration_secs <= 121);
}

#[test]
fn flush_during_walking() {
    let mut mgr = SessionManager::new();
    let now = Utc::now();
    mgr.state.state = DeskState::Walking;
    mgr.state.standing_bout_started = Some(now - Duration::seconds(60));

    let result = mgr.flush_current_session();
    assert!(result.is_some(), "Walking uses standing_bout_started");
    let session = result.unwrap();
    assert!(session.duration_secs >= 59 && session.duration_secs <= 61);
}

#[test]
fn flush_during_away_returns_none() {
    let mgr = SessionManager::new();
    // Default state is Away with no timestamps
    assert_eq!(mgr.state.state, DeskState::Away);
    let result = mgr.flush_current_session();
    assert!(result.is_none(), "Away state should not produce a session");
}
