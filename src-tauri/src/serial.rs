//! serial.rs — Auto-detecting background serial reader for VL53L1X sensor.
//!
//! Emitted Tauri events:
//! - `desk:distance`          — [`DistanceReading`] payload
//! - `desk:device-connected`  — [`DeviceConnected`] payload
//! - `desk:device-lost`       — no payload
//! - `desk:sensor-error`      — [`SensorError`] payload
//! - `desk:state-changed`     — [`StateChangedPayload`] payload (via session)

use std::{
    io::{BufRead, BufReader, Write},
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
use tauri_plugin_notification::NotificationExt;

use crate::{activity::is_active, session::SessionManager};

// ─── Constants ───────────────────────────────────────────────────────────────

const BAUD_RATE: u32 = 115_200;
const PING_CMD: &[u8] = b"PING\n";
const DEVICE_ID: &str = "DEVICE: zntl-desk-sensor v1";
const DISTANCE_PREFIX: &str = "distance: ";
const DISTANCE_SUFFIX: &str = " mm";
/// Timeout used when probing a port for the PING response.
const PROBE_TIMEOUT_MS: u64 = 2_000;
/// How long to wait between re-scan attempts when not connected.
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

// ─── Port info (used by list_ports command) ───────────────────────────────────

/// Serialisable descriptor for an available serial port.
#[derive(Debug, Serialize, Clone)]
pub struct PortInfo {
    pub name: String,
    pub description: Option<String>,
}

/// Returns all available serial ports on the system.
pub fn available_port_infos() -> Vec<PortInfo> {
    serialport::available_ports()
        .unwrap_or_default()
        .into_iter()
        .map(|p| PortInfo {
            name: p.port_name,
            description: match p.port_type {
                serialport::SerialPortType::UsbPort(info) => info.product,
                _ => None,
            },
        })
        .collect()
}

// ─── Parsing ─────────────────────────────────────────────────────────────────

/// Parses `"distance: 1234 mm"` → `Some(1234)`.
fn parse_distance(line: &str) -> Option<i32> {
    let inner = line
        .strip_prefix(DISTANCE_PREFIX)?
        .strip_suffix(DISTANCE_SUFFIX)?;
    inner.trim().parse().ok()
}

// ─── Probe ───────────────────────────────────────────────────────────────────

/// Opens `port_name`, sends `PING\n`, and returns `true` if the device
/// responds with the expected device-ID string within 2 seconds.
fn probe_port(port_name: &str) -> bool {
    let mut port = match serialport::new(port_name, BAUD_RATE)
        .timeout(Duration::from_millis(PROBE_TIMEOUT_MS))
        .open()
    {
        Ok(p) => p,
        Err(_) => return false,
    };

    // Flush stale data, then send PING.
    let _ = port.write_all(PING_CMD);
    let _ = port.flush();

    let mut reader = BufReader::new(port);
    let deadline = std::time::Instant::now() + Duration::from_millis(PROBE_TIMEOUT_MS);

    loop {
        if std::time::Instant::now() >= deadline {
            return false;
        }
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => continue,
            Ok(_) => {
                if line.trim() == DEVICE_ID {
                    return true;
                }
            }
            Err(_) => return false,
        }
    }
}

// ─── Background reader ───────────────────────────────────────────────────────

/// Runs the reader loop for a confirmed port until `stop` is set or an error
/// occurs. Emits `desk:distance`, `desk:sensor-error`, `desk:device-lost`.
/// Also feeds each valid reading into the [`SessionManager`] and fires a
/// native notification when `should_alert()` returns true.
fn reader_loop(
    app: &AppHandle,
    port_name: &str,
    stop: &Arc<AtomicBool>,
    session: &Arc<Mutex<SessionManager>>,
    config: &crate::config::AppConfig,
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
    let mut last_notification_check = std::time::Instant::now();

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
            // Emit raw distance to the frontend.
            let reading = DistanceReading {
                mm,
                cm: mm as f32 / 10.0,
                timestamp: Utc::now().to_rfc3339(),
            };
            let _ = app.emit("desk:distance", reading);

            // Feed into session state machine.
            let active = is_active();
            let (state_before, result) = {
                let mut sess = session.lock().unwrap();
                let state_before = sess.current_state();
                let result = sess.on_reading(mm, active);
                (state_before, result)
            };

            if let Some(payload) = result.state_change {
                let _ = app.emit("desk:state-changed", &payload);

                // Check if transitioning to Standing for praise-halfway notification
                use crate::session::DeskState;
                if state_before == DeskState::Sitting && payload.state == DeskState::Standing {
                    let praise = {
                        let mut sess = session.lock().unwrap();
                        sess.should_send_praise_halfway(config)
                    };

                    if praise {
                        let _ = app
                            .notification()
                            .builder()
                            .title("Halfway through your standing goal!")
                            .body("Keep it up.")
                            .show();
                    }
                }
            }

            // Persist completed sessions to the database
            if let Some(completed) = result.completed_session {
                // TODO: persist to database when db is available in serial context
                info!("Completed session: {:?}", completed);
            }

            // Check if an alert should fire (once per sitting stint).
            let alert = {
                let mut sess = session.lock().unwrap();
                sess.should_alert()
            };

            if alert {
                let _ = app
                    .notification()
                    .builder()
                    .title("Time to stand up!")
                    .body("You've been sitting for 40 minutes. Take a break.")
                    .show();
            }

            // Check for standing time limit alert.
            let stand_alert = {
                let mut sess = session.lock().unwrap();
                sess.should_stand_alert()
            };

            if stand_alert {
                let _ = app
                    .notification()
                    .builder()
                    .title("You've been standing a while")
                    .body("Ready to sit down for a bit?")
                    .show();
            }

            // Check notification conditions (inactivity, posture balance, praise) every 60s.
            if last_notification_check.elapsed() >= Duration::from_secs(60) {
                let notification_events = {
                    let mut sess = session.lock().unwrap();
                    sess.check_notification_conditions(config)
                };

                for event in notification_events {
                    use crate::session::NotificationEvent;
                    match event {
                        NotificationEvent::Inactivity => {
                            let _ = app
                                .notification()
                                .builder()
                                .title("No position change in 90 minutes")
                                .body("Time to move.")
                                .show();
                        }
                        NotificationEvent::PostureBalance => {
                            let _ = app
                                .notification()
                                .builder()
                                .title("You've been sitting most of today")
                                .body("Consider standing for a while.")
                                .show();
                        }
                        NotificationEvent::Praise => {
                            let _ = app
                                .notification()
                                .builder()
                                .title("Halfway through your standing goal!")
                                .body("Keep it up.")
                                .show();
                        }
                        NotificationEvent::StandLimitReached => {
                            // Handled separately via should_stand_alert
                        }
                    }
                }

                last_notification_check = std::time::Instant::now();
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

/// Scans all available COM ports for the desk sensor, opens a background
/// reader on the first match, and re-scans every 10 s if disconnected.
///
/// A new OS thread is spawned immediately; this function returns once the
/// thread is started.
pub fn scan_and_connect(
    app: AppHandle,
    conn: Arc<ConnectionState>,
    session: Arc<Mutex<SessionManager>>,
    config: Arc<std::sync::Mutex<Option<crate::config::AppConfig>>>,
) {
    std::thread::spawn(move || {
        loop {
            // If already connected, sleep and check again.
            {
                let flag_guard = conn.stop_flag.lock().unwrap();
                if flag_guard.is_some() {
                    drop(flag_guard);
                    std::thread::sleep(Duration::from_secs(RESCAN_INTERVAL_SECS));
                    continue;
                }
            }

            info!("Scanning serial ports for desk sensor…");
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
                    DeviceConnected {
                        port: port_name.clone(),
                    },
                );

                let stop = Arc::new(AtomicBool::new(false));
                {
                    let mut flag_guard = conn.stop_flag.lock().unwrap();
                    *flag_guard = Some(stop.clone());
                    let mut port_guard = conn.connected_port.lock().unwrap();
                    *port_guard = Some(port_name.clone());
                }

                // Get config for notification checks
                let cfg = config.lock().unwrap().clone().unwrap_or_default();

                // Block this loop thread while reading.
                reader_loop(&app, &port_name, &stop, &session, &cfg);

                // Reader ended (device lost or stop requested).
                {
                    let mut flag_guard = conn.stop_flag.lock().unwrap();
                    *flag_guard = None;
                    let mut port_guard = conn.connected_port.lock().unwrap();
                    *port_guard = None;
                }
            } else {
                info!("No desk sensor found; retrying in {RESCAN_INTERVAL_SECS}s");
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

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn emit_error(app: &AppHandle, message: &str) {
    let _ = app.emit(
        "desk:sensor-error",
        SensorError {
            message: message.to_string(),
            timestamp: Utc::now().to_rfc3339(),
        },
    );
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_line() {
        assert_eq!(parse_distance("distance: 342 mm"), Some(342));
    }

    #[test]
    fn parses_zero() {
        assert_eq!(parse_distance("distance: 0 mm"), Some(0));
    }

    #[test]
    fn parses_large_value() {
        assert_eq!(parse_distance("distance: 2000 mm"), Some(2000));
    }

    #[test]
    fn rejects_garbage() {
        assert_eq!(parse_distance("rst:0x15 boot"), None);
        assert_eq!(parse_distance("ERROR: out of range"), None);
        assert_eq!(parse_distance(""), None);
    }

    #[test]
    fn rejects_partial_prefix() {
        assert_eq!(parse_distance("distance: 100"), None); // missing " mm"
    }
}
