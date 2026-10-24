//! Shared owner for the plugin Store and its atomic settings-file writes.

use std::{
    ops::Deref,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
};

use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_store::{Store, StoreExt};

pub struct AppStore<R: Runtime> {
    store: Arc<Store<R>>,
    path: PathBuf,
    write_lock: Arc<Mutex<()>>,
}

impl<R: Runtime> Clone for AppStore<R> {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            path: self.path.clone(),
            write_lock: self.write_lock.clone(),
        }
    }
}

impl<R: Runtime> Deref for AppStore<R> {
    type Target = Store<R>;

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}

impl<R: Runtime> AppStore<R> {
    pub(crate) fn shares_store_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.store, &other.store)
    }

    /// Preserves the existing `State::inner()` reader call shape while
    /// retaining the application-owned persistence boundary.
    pub fn inner(&self) -> &Self {
        self
    }

    pub fn write(&self) -> AppStoreWriteGuard<'_, R> {
        let guard = self.write_lock.lock().unwrap_or_else(|error| error.into_inner());
        AppStoreWriteGuard { owner: self, _guard: guard }
    }

    /// Atomically writes the current complete Store object.
    pub fn save(&self) -> Result<(), String> {
        self.write().save()
    }

    fn save_locked(&self) -> Result<(), String> {
        let mut entries = serde_json::Map::new();
        for (key, value) in self.store.entries() {
            entries.insert(key.clone(), value.clone());
        }
        let bytes = serde_json::to_vec_pretty(&entries)
            .map_err(|error| format!("Failed to serialize settings store: {error}"))?;
        crate::store_persistence::atomic_write(&self.path, &bytes)
    }
}

pub struct AppStoreWriteGuard<'a, R: Runtime> {
    owner: &'a AppStore<R>,
    _guard: MutexGuard<'a, ()>,
}

impl<R: Runtime> Deref for AppStoreWriteGuard<'_, R> {
    type Target = Store<R>;

    fn deref(&self) -> &Self::Target {
        &self.owner.store
    }
}

impl<R: Runtime> AppStoreWriteGuard<'_, R> {
    /// Saves while the shared write lock remains held, including rollback.
    pub fn save(&self) -> Result<(), String> {
        self.owner.save_locked()
    }
}

pub fn initialize<R: Runtime>(app: &AppHandle<R>) -> Result<AppStore<R>, String> {
    let file = app.path().app_data_dir().map_err(|error| error.to_string())?.join("settings.json");
    initialize_at(app, &file)
}

pub(crate) fn initialize_at<R: Runtime>(app: &AppHandle<R>, file: &Path) -> Result<AppStore<R>, String> {
    if let Some(existing) = app.try_state::<AppStore<R>>() {
        return Ok(existing.inner().clone());
    }

    // This plugin version ignores load errors in build_inner. Surface existing
    // file IO/schema failures before exposing a usable shared Store.
    match std::fs::read(file) {
        Ok(bytes) => {
            let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
            if !value.is_object() {
                return Err("Settings store must contain a JSON object".into());
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.to_string()),
    }

    let plugin_store = app
        .store_builder(file)
        .disable_auto_save()
        .build()
        .map_err(|error| error.to_string())?;

    // tauri-plugin-store 2.4.4 saves every registered Store again at RunEvent::Exit,
    // even when auto-save is disabled. Remove only its resource registration;
    // the shared Arc remains available to the application's Rust readers.
    plugin_store.close_resource();

    let shared = AppStore {
        store: plugin_store,
        path: file.to_path_buf(),
        write_lock: Arc::new(Mutex::new(())),
    };
    if app.manage(shared.clone()) {
        return Ok(shared);
    }
    app.try_state::<AppStore<R>>()
        .map(|state| state.inner().clone())
        .ok_or_else(|| "Shared store registration failed".into())
}
