//! serial.rs — Auto-detecting background serial reader for VL53L1X sensor.
//!
//! Emitted Tauri events:
//! - `desk:distance`, `desk:device-connected`, `desk:device-lost`,
//!   `desk:device-missing`, `desk:sensor-error`, `desk:state-changed`,
//!   `desk:daily-reset`

use std::{
    io::{BufRead, BufReader},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use chrono::Utc;
use log::{error, info, warn};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::event_logger::EventLogger;
use crate::session::SessionManager;
use crate::serial_parser::{parse_distance, probe_port};
use crate::serial_periodic::{check_periodic, handle_reading};
use crate::snapshot_logger::SnapshotLogger;

// Re-export parser types for backwards compatibility.
pub use crate::serial_parser::{available_port_infos, PortInfo};

// ─── Constants ───────────────────────────────────────────────────────────────

pub(crate) const BAUD_RATE: u32 = 115_200;
pub(crate) const PING_CMD: &[u8] = b"PING\n";
pub(crate) const DEVICE_ID: &str = "DEVICE: zntl-desk-sensor v1";
pub(crate) const DISTANCE_PREFIX: &str = "distance: ";
pub(crate) const DISTANCE_SUFFIX: &str = " mm";
pub(crate) const PROBE_TIMEOUT_MS: u64 = 2_000;
const RESCAN_INTERVAL_SECS: u64 = 10;

// ─── Payloads ────────────────────────────────────────────────────────────────

/// Distance measurement forwarded to the frontend.
#[derive(Debug, Serialize, Clone)]
pub struct DistanceReading {
    pub mm: i32,
    pub cm: f32,
    pub timestamp: String,
}

/// Emitted once the sensor device is successfully identified.
#[derive(Debug, Serialize, Clone)]
pub struct DeviceConnected {
    pub port: String,
}

/// Emitted on serial / sensor errors.
#[derive(Debug, Serialize, Clone)]
pub struct SensorError {
    pub message: String,
    pub timestamp: String,
}

// ─── Background reader ───────────────────────────────────────────────────────

/// Runs the reader loop for a confirmed port.
fn reader_loop(
    app: &AppHandle,
    port_name: &str,
    stop: &Arc<AtomicBool>,
    session: &Arc<Mutex<SessionManager>>,
    db: &Arc<Mutex<Option<rusqlite::Connection>>>,
    config: &crate::config::AppConfig,
    snapshot_logger: &Arc<SnapshotLogger>,
    event_logger: &Arc<EventLogger>,
) {
    let port = match serialport::new(port_name, BAUD_RATE)
        .timeout(Duration::from_millis(2000))
        .open()
    {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to (re-)open {port_name}: {e}");
            emit_error(app, &format!("Cannot open {port_name}: {e}"));
            return;
        }
    };

    let mut reader = BufReader::new(port);
    let mut line = String::new();
    let mut last_periodic = std::time::Instant::now();

    while !stop.load(Ordering::Relaxed) {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => continue,
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => continue,
            Err(e) => {
                warn!("Serial read error on {port_name}: {e}");
                let _ = app.emit("desk:device-lost", ());
                return;
            }
        }

        let trimmed = line.trim();

        if let Some(mm) = parse_distance(trimmed) {
            let reading = DistanceReading {
                mm,
                cm: mm as f32 / 10.0,
                timestamp: Utc::now().to_rfc3339(),
            };
            let _ = app.emit("desk:distance", reading);

            handle_reading(app, mm, session, db, config, event_logger);

            if last_periodic.elapsed() >= Duration::from_secs(60) {
                check_periodic(app, session, config, snapshot_logger, event_logger, port_name);
                last_periodic = std::time::Instant::now();
            }
        } else if trimmed.to_ascii_uppercase().starts_with("ERROR") {
            emit_error(app, trimmed);
        }
    }

    info!("Serial reader stopped for {port_name}");
}

// ─── Public API ──────────────────────────────────────────────────────────────

/// Shared state tracking whether a connection is active.
#[derive(Default)]
pub struct ConnectionState {
    pub stop_flag: Mutex<Option<Arc<AtomicBool>>>,
    pub connected_port: Mutex<Option<String>>,
}

/// Scans all COM ports for the desk sensor, starts a background reader.
pub fn scan_and_connect(
    app: AppHandle,
    conn: Arc<ConnectionState>,
    session: Arc<Mutex<SessionManager>>,
    db: Arc<Mutex<Option<rusqlite::Connection>>>,
    config: Arc<Mutex<Option<crate::config::AppConfig>>>,
    snapshot_logger: Arc<SnapshotLogger>,
    event_logger: Arc<EventLogger>,
) {
    std::thread::spawn(move || {
        loop {
            {
                let flag_guard = conn.stop_flag.lock().unwrap();
                if flag_guard.is_some() {
                    drop(flag_guard);
                    std::thread::sleep(Duration::from_secs(RESCAN_INTERVAL_SECS));
                    continue;
                }
            }

            info!("Scanning serial ports for desk sensor...");
            let ports = serialport::available_ports().unwrap_or_default();

            let found = ports.iter().find(|p| {
                info!("Probing {}", p.port_name);
                probe_port(&p.port_name)
            });

            if let Some(port_info) = found {
                let port_name = port_info.port_name.clone();
                info!("Desk sensor found on {port_name}");

                let _ = app.emit(
                    "desk:device-connected",
                    DeviceConnected { port: port_name.clone() },
                );

                let stop = Arc::new(AtomicBool::new(false));
                {
                    *conn.stop_flag.lock().unwrap() = Some(stop.clone());
                    *conn.connected_port.lock().unwrap() = Some(port_name.clone());
                }

                event_logger.log(&format!("DEVICE connected {}", port_name));

                let cfg = config.lock().unwrap().clone().unwrap_or_default();
                reader_loop(&app, &port_name, &stop, &session, &db, &cfg, &snapshot_logger, &event_logger);

                event_logger.log("DEVICE lost");
                {
                    *conn.stop_flag.lock().unwrap() = None;
                    *conn.connected_port.lock().unwrap() = None;
                }
            } else {
                info!("No desk sensor found; retrying in {RESCAN_INTERVAL_SECS}s");
                let _ = app.emit("desk:device-missing", ());
            }

            std::thread::sleep(Duration::from_secs(RESCAN_INTERVAL_SECS));
        }
    });
}

/// Signals the active reader to stop.
pub fn stop_reading(conn: &ConnectionState) {
    let mut guard = conn.stop_flag.lock().unwrap();
    if let Some(flag) = guard.take() {
        flag.store(true, Ordering::Relaxed);
    }
}

fn emit_error(app: &AppHandle, message: &str) {
    let _ = app.emit(
        "desk:sensor-error",
        SensorError {
            message: message.to_string(),
            timestamp: Utc::now().to_rfc3339(),
        },
    );
}
