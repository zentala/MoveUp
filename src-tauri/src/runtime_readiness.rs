//! Flat, bounded readiness; no session aggregate or outbound health IO.
use serde::{Deserialize, Serialize};
use std::sync::{Arc, atomic::Ordering};
use crate::app_lifecycle::AppLifecycle;
pub const APP_ID: &str = "io.zntl.desk";
pub const PATH: &str = "/display/readiness";
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReadinessState { Initializing, Ready, Degraded, Failed }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessStatus {
    pub app_id: String,
    pub version: String,
    pub pid: u32,
    pub run_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supervisor_run_id: Option<String>,
    pub build_mode: String,
    pub ready: bool,
    pub state: ReadinessState,
    pub reason: Option<String>,
    pub tray_initialized: bool,
    pub persistence_initialized: bool,
    pub setup_complete: bool,
    pub remote_listening: bool,
    pub sensor_present: bool,
    pub session_store_available: bool,
    pub quit_requested: bool,
}
impl ReadinessStatus {
    pub fn snapshot(lifecycle: &AppLifecycle) -> Self {
        let id = &lifecycle.identity;
        Self {
            app_id: APP_ID.into(), version: id.version.clone(), pid: id.pid,
            run_id: id.run_id.clone(), supervisor_run_id: id.supervisor_run_id.clone(),
            build_mode: id.build_mode.clone(), ready: false,
            state: if lifecycle.failed.load(Ordering::Acquire) { ReadinessState::Failed }
                else { ReadinessState::Initializing }, reason: None,
            tray_initialized: lifecycle.tray_initialized.load(Ordering::Acquire),
            persistence_initialized: lifecycle.persistence_initialized.load(Ordering::Acquire),
            setup_complete: lifecycle.setup_complete.load(Ordering::Acquire),
            remote_listening: lifecycle.remote_listening.load(Ordering::Acquire),
            sensor_present: false, session_store_available: false,
            quit_requested: lifecycle.quit_requested.load(Ordering::Acquire),
        }
    }
    pub fn compute_state(&mut self) {
        self.ready = false;
        let reason = if self.state == ReadinessState::Failed { Some("observed_failure") }
            else if self.quit_requested { Some("quit_requested") }
            else if !self.tray_initialized { Some("tray_not_ready") }
            else if !self.persistence_initialized { Some("persistence_not_ready") }
            else if !self.setup_complete { Some("setup_incomplete") }
            else { None };
        if let Some(reason) = reason {
            if self.state != ReadinessState::Failed { self.state = ReadinessState::Initializing; }
            self.reason = Some(reason.into());
        } else {
            self.ready = true;
            self.state = if self.sensor_present { ReadinessState::Ready } else { ReadinessState::Degraded };
            self.reason = if self.sensor_present { None } else { Some("sensor_missing".into()) };
        }
    }
}
pub type ReadinessReader = Arc<dyn Fn() -> ReadinessStatus + Send + Sync>;
/// Separate router keeps existing /display/api and RemoteState callers intact.
pub fn routes(read: ReadinessReader) -> axum::Router {
    axum::Router::new().route(PATH, axum::routing::get(move || {
        let status = read();
        async move { axum::Json(status) }
    }))
}
