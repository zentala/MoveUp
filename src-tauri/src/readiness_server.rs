//! Local supervision transport, independent of the optional LAN display.
use std::{io, net::Ipv4Addr, sync::{Arc, atomic::Ordering}};
use crate::{app_lifecycle::AppLifecycle, runtime_readiness::ReadinessReader};

pub const DEFAULT_PORT: u16 = 3391;
pub const PORT_ENV: &str = "DESK_READINESS_PORT";

pub(crate) fn parse_port(value: Option<&str>) -> Result<u16, String> {
    match value {
        None => Ok(DEFAULT_PORT),
        Some(value) if !value.is_empty() && value.bytes().all(|c| c.is_ascii_digit()) =>
            value.parse::<u16>().ok().filter(|p| *p > 0)
                .ok_or_else(|| format!("{PORT_ENV} must be an integer from 1 to 65535")),
        Some(_) => Err(format!("{PORT_ENV} must be an integer from 1 to 65535")),
    }
}

fn failed(lifecycle: &AppLifecycle, stage: &str) {
    lifecycle.failed.store(true, Ordering::Release);
    lifecycle.event(stage);
}

/// Binding failure is observable and never falls back to another port.
/// Port zero is only used by isolated tests; production config rejects it.
pub(crate) async fn bind(port: u16, lifecycle: &AppLifecycle)
    -> io::Result<tokio::net::TcpListener> {
    match tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, port)).await {
        Ok(listener) => Ok(listener),
        Err(error) => {
            failed(lifecycle, "readiness_bind_failed");
            log::error!("Local readiness could not bind port {port}: {error}. \
                Free the port or set {PORT_ENV} and the PM3 probe to the same free port.");
            Err(error)
        }
    }
}

pub fn setup(app: &tauri::AppHandle) {
    use tauri::Manager;
    let lifecycle = app.state::<Arc<AppLifecycle>>().inner().clone();
    let port = match std::env::var(PORT_ENV) {
        Ok(value) => parse_port(Some(&value)),
        Err(std::env::VarError::NotPresent) => parse_port(None),
        Err(_) => Err(format!("{PORT_ENV} must contain a valid integer")),
    };
    let port = match port {
        Ok(port) => port,
        Err(error) => {
            failed(&lifecycle, "readiness_config_failed");
            log::error!("{error}; local readiness is unavailable. Correct the setting and restart.");
            return;
        }
    };
    let reader = crate::app_lifecycle_runtime::readiness_reader(app);
    tauri::async_runtime::spawn(start(port, reader, lifecycle));
}

async fn start(port: u16, reader: ReadinessReader, lifecycle: Arc<AppLifecycle>) {
    let Ok(listener) = bind(port, &lifecycle).await else { return };
    lifecycle.event("readiness_listening");
    if let Err(error) = axum::serve(listener, crate::runtime_readiness::routes(reader)).await {
        log::error!("Local readiness server stopped: {error}");
    }
    failed(&lifecycle, "readiness_stopped");
}
