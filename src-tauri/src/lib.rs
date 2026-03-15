/// zntl Desk — VL53L1X ToF sensor reader Tauri backend.
mod commands;
mod serial;

use commands::ReaderState;
use std::sync::Mutex;

/// Application entry point called from main.rs.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .manage(ReaderState {
            stop_flag: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_ports,
            commands::start_reading,
            commands::stop_reading,
        ])
        .run(tauri::generate_context!())
        .expect("error running Desk");
}
