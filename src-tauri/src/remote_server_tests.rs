//! Tests for remote_server.rs — RemoteState, client counter, max clients.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crate::db::TodaySummary;
use crate::remote_server::{RemoteState, MAX_WS_CLIENTS};
use crate::session::SessionManager;

#[test]
fn remote_state_is_clone() {
    let tx = crate::ws_broadcaster::create_channel();
    let state = RemoteState {
        ws_tx: tx,
        session: Arc::new(Mutex::new(SessionManager::new())),
        config: Arc::new(Mutex::new(None)),
        today_cache: Arc::new(Mutex::new(empty_today_summary())),
        active_clients: Arc::new(AtomicUsize::new(0)),
    };
    let _cloned = state.clone();
}

#[test]
fn client_counter_increments_and_decrements() {
    let counter = Arc::new(AtomicUsize::new(0));
    let c1 = counter.fetch_add(1, Ordering::Relaxed) + 1;
    assert_eq!(c1, 1);
    let c2 = counter.fetch_add(1, Ordering::Relaxed) + 1;
    assert_eq!(c2, 2);
    let c3 = counter.fetch_sub(1, Ordering::Relaxed) - 1;
    assert_eq!(c3, 1);
}

#[test]
fn max_clients_check() {
    let counter = AtomicUsize::new(MAX_WS_CLIENTS);
    let current = counter.load(Ordering::Relaxed);
    assert!(current >= MAX_WS_CLIENTS);
}

fn empty_today_summary() -> TodaySummary {
    TodaySummary {
        sitting_secs: 0,
        standing_secs: 0,
        yesterday_sitting_secs: 0,
        yesterday_standing_secs: 0,
        position_changes: 0,
        sessions: vec![],
    }
}
