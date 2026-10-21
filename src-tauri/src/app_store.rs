//! Shared plugin Store ownership. No default write or format migration on load.
use std::{path::Path, sync::Arc};
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_store::{Store, StoreExt};

pub type AppStore<R> = Arc<Store<R>>;

pub fn initialize<R: Runtime>(app: &AppHandle<R>) -> Result<AppStore<R>, String> {
    let file = app.path().app_data_dir().map_err(|e| e.to_string())?.join("settings.json");
    initialize_at(app, &file)
}

pub(crate) fn initialize_at<R: Runtime>(app: &AppHandle<R>, file: &Path) -> Result<AppStore<R>, String> {
    if let Some(existing) = app.try_state::<AppStore<R>>() { return Ok(existing.inner().clone()); }
    // This plugin version ignores load errors in build_inner. Surface existing
    // file IO/schema failures before exposing a usable shared Store.
    match std::fs::read(file) {
        Ok(bytes) => {
            let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            if !value.is_object() { return Err("Settings store must contain a JSON object".into()); }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.to_string()),
    }
    let shared = app.store_builder(file).disable_auto_save().build().map_err(|e| e.to_string())?;
    if app.manage(shared.clone()) { return Ok(shared); }
    app.try_state::<AppStore<R>>().map(|state| state.inner().clone())
        .ok_or_else(|| "Shared store registration failed".into())
}
