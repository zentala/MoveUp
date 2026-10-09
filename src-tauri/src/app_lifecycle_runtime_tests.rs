use std::sync::{Arc, Mutex, atomic::Ordering};
use crate::{app_lifecycle::AppLifecycle, app_lifecycle_runtime::{local_readiness, flush_state},
    runtime_readiness::{self, ReadinessState}, session::SessionManager, session_types::DeskState};
use tower::ServiceExt;

fn initialized() -> Arc<AppLifecycle> {
    let lifecycle = Arc::new(AppLifecycle::new("test", std::path::PathBuf::new()));
    lifecycle.tray_initialized.store(true, Ordering::Release);
    lifecycle.persistence_initialized.store(true, Ordering::Release);
    lifecycle.setup_finished();
    lifecycle
}

#[tokio::test]
async fn live_readiness_is_bounded_under_db_and_sensor_contention() {
    let lifecycle = initialized();
    let db = Arc::new(Mutex::new(Some(rusqlite::Connection::open_in_memory().unwrap())));
    let port = Arc::new(Mutex::new(Some("fixture".to_owned())));
    let read = {
        let (lifecycle, db, port) = (lifecycle.clone(), db.clone(), port.clone());
        Arc::new(move || local_readiness(&lifecycle, &db, &port, false))
    };
    assert_eq!(read().state, ReadinessState::Ready);
    let db_held = db.lock().unwrap();
    let port_held = port.lock().unwrap();
    let response = tokio::time::timeout(std::time::Duration::from_millis(500),
        runtime_readiness::routes(read.clone()).oneshot(
            axum::http::Request::builder().uri(runtime_readiness::PATH)
                .body(axum::body::Body::empty()).unwrap())).await.unwrap().unwrap();
    let body = axum::body::to_bytes(response.into_body(), 4096).await.unwrap();
    let status: runtime_readiness::ReadinessStatus = serde_json::from_slice(&body).unwrap();
    assert!(!status.ready);
    assert!(!status.persistence_initialized);
    drop(db_held); drop(port_held);
    *port.lock().unwrap() = None;
    assert_eq!(read().state, ReadinessState::Degraded);
    assert!(read().ready);
    *db.lock().unwrap() = None;
    assert!(!read().ready);
}

fn sitting() -> Mutex<SessionManager> {
    let mut session = SessionManager::new();
    session.state.state = DeskState::Sitting;
    session.state.sitting_started = Some(chrono::Utc::now() - chrono::Duration::seconds(60));
    Mutex::new(session)
}

#[test]
fn shutdown_preserves_session_insert_and_reports_store_failure() {
    let session = sitting();
    let connection = rusqlite::Connection::open_in_memory().unwrap();
    crate::db::init_schema(&connection).unwrap();
    let db = Mutex::new(Some(connection));
    assert_eq!(flush_state(&session, &db, |_| "failed"), "store=failed database=saved");
    let guard = db.lock().unwrap();
    let count: i64 = guard.as_ref().unwrap().query_row("SELECT COUNT(*) FROM sessions",
        [], |row| row.get(0)).unwrap();
    assert_eq!(count, 1);
}

#[test]
fn shutdown_reports_skipped_and_failed_flush_without_invented_success() {
    let session = sitting();
    let db = Mutex::new(None);
    assert!(flush_state(&session, &db, |_| "skipped_store_missing").contains("database=skipped_db_missing"));
    let held = db.lock().unwrap();
    assert!(flush_state(&session, &db, |_| "saved").contains("database=skipped_db_locked"));
    drop(held);
    *db.lock().unwrap() = Some(rusqlite::Connection::open_in_memory().unwrap()); // missing schema
    assert!(flush_state(&session, &db, |_| "saved").contains("database=failed"));
    let held = session.lock().unwrap();
    assert!(flush_state(&session, &db, |_| panic!("must not save")).contains("skipped_session_locked"));
    drop(held);
    assert!(flush_state(&Mutex::new(SessionManager::new()), &db, |_| "saved")
        .contains("database=not_needed"));
}
