//! commands.rs — Tauri IPC commands exposed to the frontend.

use std::sync::{atomic::AtomicBool, Arc, Mutex};

use serde::Serialize;
use tauri::State;

use crate::serial;

/// Shared reader state managed by Tauri.
pub struct ReaderState {
    pub stop_flag: Mutex<Option<Arc<AtomicBool>>>,
}

#[derive(Debug, Serialize)]
pub struct PortInfo {
    name: String,
    description: Option<String>,
}

/// Lists available serial ports on the system.
#[tauri::command]
pub fn list_ports() -> Vec<PortInfo> {
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

/// Starts the background serial reader on `port`.
/// Stops any previously running reader first.
#[tauri::command]
pub fn start_reading(
    port: String,
    app: tauri::AppHandle,
    state: State<'_, ReaderState>,
) -> Result<(), String> {
    let mut guard = state.stop_flag.lock().map_err(|e| e.to_string())?;

    // Stop previous reader if running.
    if let Some(flag) = guard.take() {
        flag.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    let stop = Arc::new(AtomicBool::new(false));
    *guard = Some(stop.clone());

    serial::spawn_reader(app, port, stop);
    Ok(())
}

/// Signals the background reader thread to stop.
#[tauri::command]
pub fn stop_reading(state: State<'_, ReaderState>) -> Result<(), String> {
    let mut guard = state.stop_flag.lock().map_err(|e| e.to_string())?;
    if let Some(flag) = guard.take() {
        flag.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    Ok(())
}
