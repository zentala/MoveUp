//! serial.rs — background serial reader for VL53L1X sensor.
//!
//! Spawns a thread that reads lines from the XIAO ESP32-C3 and emits
//! Tauri events to the frontend:
//!   - `desk:distance`     — DistanceReading payload
//!   - `desk:sensor-error` — SensorError payload

use std::{
    io::{self, BufRead, BufReader},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use chrono::Utc;
use log::{error, info, warn};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

const BAUD_RATE: u32 = 115_200;
const DISTANCE_PREFIX: &str = "distance: ";
const DISTANCE_SUFFIX: &str = " mm";

#[derive(Debug, Serialize, Clone)]
pub struct DistanceReading {
    pub mm: i32,
    pub cm: f32,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct SensorError {
    pub message: String,
    pub timestamp: String,
}

/// Parses `"distance: 1234 mm"` → `Some(1234)`.
fn parse_distance(line: &str) -> Option<i32> {
    let inner = line
        .strip_prefix(DISTANCE_PREFIX)?
        .strip_suffix(DISTANCE_SUFFIX)?;
    inner.trim().parse().ok()
}

/// Spawns a background thread that reads from `port_name` until `stop` is set.
/// Returns immediately; the thread owns the serial handle.
pub fn spawn_reader(app: AppHandle, port_name: String, stop: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        info!("Opening serial port {port_name} @ {BAUD_RATE}");

        let port = match serialport::new(&port_name, BAUD_RATE)
            .timeout(Duration::from_millis(2000))
            .open()
        {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to open {port_name}: {e}");
                let _ = app.emit(
                    "desk:sensor-error",
                    SensorError {
                        message: format!("Cannot open {port_name}: {e}"),
                        timestamp: Utc::now().to_rfc3339(),
                    },
                );
                return;
            }
        };

        let mut reader = BufReader::new(port);
        let mut line = String::new();

        while !stop.load(Ordering::Relaxed) {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => continue, // timeout / no data
                Ok(_) => {}
                Err(e) if e.kind() == io::ErrorKind::TimedOut => continue,
                Err(e) => {
                    warn!("Serial read error: {e}");
                    let _ = app.emit(
                        "desk:sensor-error",
                        SensorError {
                            message: format!("Serial error: {e}"),
                            timestamp: Utc::now().to_rfc3339(),
                        },
                    );
                    break;
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
            } else if trimmed.to_ascii_uppercase().starts_with("ERROR") {
                let _ = app.emit(
                    "desk:sensor-error",
                    SensorError {
                        message: trimmed.to_string(),
                        timestamp: Utc::now().to_rfc3339(),
                    },
                );
            }
        }

        info!("Serial reader stopped for {port_name}");
    });
}

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
    fn rejects_garbage() {
        assert_eq!(parse_distance("rst:0x15 boot"), None);
        assert_eq!(parse_distance("ERROR: out of range"), None);
        assert_eq!(parse_distance(""), None);
    }
}
