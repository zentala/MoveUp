//! Explicit tray Quit flushes before the ordinary supervisor stop can kill us.
use crate::supervision::{self, Configuration, QuitGate};

static GATE: QuitGate = QuitGate::new();

pub fn request(app: &tauri::AppHandle) {
    if !GATE.begin() { return; }
    let config = Configuration::from_env(|key| std::env::var(key).ok());
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let result = match config {
            Ok(None) => crate::app_lifecycle_runtime::prepare_shutdown(&app, true).map(|_| ()),
            Ok(Some(config)) => {
                match crate::app_lifecycle_runtime::prepare_shutdown(&app, true) {
                    Ok(_) => supervision::stop(&config,
                        &crate::supervision_process::PublicLauncher, supervision::DEADLINE).await,
                    Err(error) => Err(format!("Quit blocked: shutdown persistence incomplete ({error})")),
                }
            }
            Err(error) => Err(format!("Quit blocked: {error}. Correct supervision configuration and retry.")),
        };
        match result {
            Ok(()) => {
                app.exit(0); // Exit shares the already completed flush.
            }
            Err(error) => {
                if let Err(cancel) = crate::app_lifecycle_runtime::cancel_shutdown(&app) {
                    log::error!("Shutdown cancellation incomplete: {cancel}");
                }
                log::error!("{error}");
                crate::notify::show("MoveUp could not quit", &error);
                GATE.retry();
            }
        }
    });
}
