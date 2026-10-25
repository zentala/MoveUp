//! remote_display_state.rs — Single derivation of [`RemoteDisplayState`].
//!
//! Both the tray tick (`tray_controller`) and the remote HTTP/WS server
//! (`remote_server`) need the same payload: session snapshot + KPI metrics
//! computed from live totals + the cached today summary. They used to derive
//! it twice, from `AppState` and `RemoteState` respectively, so a change to
//! one path silently diverged from the other. Both now call [`build`].
//!
//! The function takes the three shared handles directly rather than either
//! state struct, so it works from both callers without either knowing the
//! other's type.

use std::sync::{Arc, Mutex};

use tokio::sync::broadcast;

use crate::communication_policy::CommunicationPolicy;
use crate::db::TodaySummary;
use crate::health_models::HealthView;
use crate::metrics::MetricEngine;
use crate::session::SessionManager;
use crate::ws_broadcaster::{self, DisplayEvent, RemoteDisplayState};

/// Builds the remote-display payload from the shared session, policy and
/// today-summary handles.
///
/// KPI metrics are computed against *live* totals (`sitting_seconds_total`,
/// `standing_seconds` advanced to now), not the last persisted values, so a
/// client connecting mid-session sees the same numbers the tray shows.
///
/// Every lock uses `unwrap_or_else(|e| e.into_inner())`: all three reads are
/// read-only, so a poisoned mutex must not take the remote display down.
///
/// `health` arrives as a value rather than as a handle because the two
/// callers obtain it differently and should say so: the HTTP/WS handlers are
/// async and `await` a fresh merge, while the ~1 s tray tick is synchronous
/// and reads the aggregator's cached
/// [`last_view`](crate::health_source::HealthAggregator::last_view).
pub fn build(
    session: &Arc<Mutex<SessionManager>>,
    comm_policy: &Arc<Mutex<CommunicationPolicy>>,
    today_cache: &Arc<Mutex<TodaySummary>>,
    health: HealthView,
) -> RemoteDisplayState {
    let session_mgr = session.lock().unwrap_or_else(|e| e.into_inner());
    let snapshot = session_mgr.snapshot();

    let now = chrono::Utc::now();
    let mut raw = session_mgr.state.clone();
    raw.sitting_seconds_total = session_mgr.get_live_sitting_seconds_total(now);
    raw.standing_seconds = session_mgr.get_live_standing_seconds(now);
    drop(session_mgr);

    let ergo = comm_policy
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .ergo_profile()
        .clone();

    let metrics = MetricEngine::with_defaults().compute_all(&raw, &ergo);
    let today = today_cache
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();

    RemoteDisplayState {
        session: snapshot,
        metrics,
        today,
        health,
    }
}

/// Builds the payload with [`build`] and fans it out to remote display
/// clients as a `snapshot` event.
pub fn broadcast(
    ws_tx: &broadcast::Sender<String>,
    session: &Arc<Mutex<SessionManager>>,
    comm_policy: &Arc<Mutex<CommunicationPolicy>>,
    today_cache: &Arc<Mutex<TodaySummary>>,
    health: HealthView,
) {
    let state = build(session, comm_policy, today_cache, health);
    ws_broadcaster::broadcast_event(ws_tx, &DisplayEvent::Snapshot(state));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn today_summary(position_changes: u32) -> TodaySummary {
        TodaySummary {
            sitting_secs: 120,
            standing_secs: 60,
            yesterday_sitting_secs: 0,
            yesterday_standing_secs: 0,
            position_changes,
            sessions: vec![],
        }
    }

    fn handles() -> (
        Arc<Mutex<SessionManager>>,
        Arc<Mutex<CommunicationPolicy>>,
        Arc<Mutex<TodaySummary>>,
    ) {
        (
            Arc::new(Mutex::new(SessionManager::new())),
            Arc::new(Mutex::new(CommunicationPolicy::new(
                Default::default(),
                Default::default(),
            ))),
            Arc::new(Mutex::new(today_summary(7))),
        )
    }

    #[test]
    fn e019_t07_remote_display_state_carries_session_metrics_and_today() {
        let (session, policy, today) = handles();
        let built = build(&session, &policy, &today, HealthView::unconfigured());

        let expected = session
            .lock()
            .unwrap()
            .snapshot();
        assert_eq!(built.session.state, expected.state);
        assert_eq!(built.today.position_changes, 7);
        assert_eq!(built.today.sitting_secs, 120);
        assert!(
            !built.metrics.is_empty(),
            "metrics must be computed, not left empty"
        );
    }

    #[test]
    fn e019_t07_remote_display_state_survives_poisoned_locks() {
        let (session, policy, today) = handles();

        for lock in [
            &session as &dyn PoisonTarget,
            &policy as &dyn PoisonTarget,
            &today as &dyn PoisonTarget,
        ] {
            lock.poison();
        }

        let built = build(&session, &policy, &today, HealthView::unconfigured());
        assert_eq!(built.today.position_changes, 7);
        assert!(!built.metrics.is_empty());
    }

    #[tokio::test]
    async fn e019_t07_remote_display_state_matches_the_remote_server_path() {
        let (session, policy, today) = handles();
        let state =
            crate::remote_routes_health_tests::remote_state(session.clone(), policy.clone(), today.clone())
                .await;

        let server_call_started_at = chrono::Utc::now();
        let via_server = crate::remote_server::build_remote_display_state(&state).await;
        let server_call_finished_at = chrono::Utc::now();
        let direct_call_started_at = chrono::Utc::now();
        let direct = build(&session, &policy, &today, state.health.view().await);
        let direct_call_finished_at = chrono::Utc::now();

        assert!(
            server_call_finished_at <= direct_call_started_at,
            "server and direct calls must be captured in order"
        );

        let via_server = serde_json::to_value(&via_server).unwrap();
        let mut direct = serde_json::to_value(&direct).unwrap();
        let server_snapshot_at = via_server["session"]["live_observation"]["snapshot_at"]
            .as_str()
            .filter(|timestamp| !timestamp.is_empty())
            .and_then(|timestamp| chrono::DateTime::parse_from_rfc3339(timestamp).ok())
            .expect("remote server payload must include a valid observation snapshot_at");
        let direct_snapshot_at = direct["session"]["live_observation"]["snapshot_at"]
            .as_str()
            .filter(|timestamp| !timestamp.is_empty())
            .and_then(|timestamp| chrono::DateTime::parse_from_rfc3339(timestamp).ok())
            .expect("direct payload must include a valid observation snapshot_at");

        assert!(
            server_snapshot_at >= server_call_started_at
                && server_snapshot_at <= server_call_finished_at,
            "remote server snapshot_at must fall within its captured call interval"
        );
        assert!(
            direct_snapshot_at >= direct_call_started_at
                && direct_snapshot_at <= direct_call_finished_at,
            "direct snapshot_at must fall within its captured call interval"
        );

        direct["session"]["live_observation"]["snapshot_at"] =
            via_server["session"]["live_observation"]["snapshot_at"].clone();

        assert_eq!(
            via_server,
            direct,
            "remote_server must derive the payload through remote_display_state::build"
        );
    }

    #[test]
    fn e021_t03_remote_display_state_carries_the_health_view_it_was_given() {
        let (session, policy, today) = handles();
        let health = HealthView {
            configured: true,
            snapshot: Some(crate::health_models::HealthSnapshot {
                steps_today: 1234,
                heart_rate_bpm: Some(61),
                hrv_rmssd_ms: None,
                source_id: "curl".into(),
                fetched_at_ms: 1_700_000_000_000,
            }),
            error_kind: None,
            error_message: None,
        };

        let built = build(&session, &policy, &today, health.clone());
        assert_eq!(built.health, health);
    }

    #[test]
    fn e019_t07_remote_display_state_broadcasts_a_snapshot_event() {
        let (session, policy, today) = handles();
        let tx = ws_broadcaster::create_channel();
        let mut rx = tx.subscribe();

        broadcast(&tx, &session, &policy, &today, HealthView::unconfigured());

        let msg = rx.try_recv().expect("a snapshot must be published");
        let parsed: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(parsed["event"], "snapshot");
        assert_eq!(parsed["payload"]["today"]["position_changes"], 7);
    }

    /// Poisons a mutex by panicking while its guard is held.
    trait PoisonTarget {
        fn poison(&self);
    }

    impl<T: Send + 'static> PoisonTarget for Arc<Mutex<T>> {
        fn poison(&self) {
            let clone = self.clone();
            let _ = std::thread::spawn(move || {
                let _guard = clone.lock().unwrap();
                panic!("poison");
            })
            .join();
            assert!(self.is_poisoned(), "mutex should be poisoned");
        }
    }
}
