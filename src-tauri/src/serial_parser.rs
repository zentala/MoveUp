//! serial_parser.rs — Serial port parsing and auto-detection.
//!
//! Contains distance parsing, port probing, and port listing functions.

use std::{
    io::{BufRead, BufReader, Write},
    time::Duration,
};

use log::info;
use serde::Serialize;

use crate::serial::{BAUD_RATE, DEVICE_ID, DISTANCE_PREFIX, DISTANCE_SUFFIX, PROBE_TIMEOUT_MS};

// ─── Port info ───────────────────────────────────────────────────────────────

/// Serialisable descriptor for an available serial port.
#[derive(Debug, Serialize, Clone)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export_to = "../../src/generated/"))]
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

/// Parses `"distance: 1234 mm"` into `Some(1234)`.
pub fn parse_distance(line: &str) -> Option<i32> {
    let inner = line
        .strip_prefix(DISTANCE_PREFIX)?
        .strip_suffix(DISTANCE_SUFFIX)?;
    inner.trim().parse().ok()
}

// ─── Probe ───────────────────────────────────────────────────────────────────

/// Opens `port_name`, sends `PING\n`, and returns `true` if the device
/// responds with the expected device-ID string within 2 seconds.
pub fn probe_port(port_name: &str) -> bool {
    let mut port = match serialport::new(port_name, BAUD_RATE)
        .timeout(Duration::from_millis(PROBE_TIMEOUT_MS))
        .open()
    {
        Ok(p) => p,
        Err(_) => return false,
    };

    let _ = port.write_all(crate::serial::PING_CMD);
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
                    info!("Probe success on {port_name}");
                    return true;
                }
            }
            Err(_) => return false,
        }
    }
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
        assert_eq!(parse_distance("distance: 100"), None);
    }
}
