//! UI glue: do not mark a clean Quit until suspension is confirmed.
use crate::supervision::{self, Configuration, QuitGate};

static GATE: QuitGate = QuitGate::new();

pub fn request(app: &tauri::AppHandle) {
    if !GATE.begin() { return; }
    let config = Configuration::from_env(|key| std::env::var(key).ok());
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let result = match config {
            Ok(None) => Ok(()),
            Ok(Some(config)) => {
                supervision::suspend(&config, GATE.intent(),
                    &crate::supervision_process::PublicLauncher, supervision::DEADLINE).await
            }
            Err(error) => Err(format!("Quit blocked: {error}. Correct supervision configuration and retry.")),
        };
        match result {
            Ok(()) => {
                crate::app_lifecycle_runtime::request_quit(&app);
                app.exit(0); // Existing RunEvent::Exit owns the persistence flush.
            }
            Err(error) => {
                log::error!("{error}");
                crate::notify::show("MoveUp could not quit", &error);
                GATE.retry();
            }
        }
    });
}
