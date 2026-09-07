//! commands_catalog.rs — IPC command exposing the static data catalog.
//!
//! Returns a structured description of every data source the desk app
//! produces or consumes. Pure metadata: no live queries, no I/O. Feeds the
//! Catalog tab of the analyst dashboard (E012).

use serde::Serialize;

use crate::commands_catalog_sources::{
    events_log_source, profiles_communication_source, profiles_ergonomic_source, remote_ws_source,
    sensor_source, snapshots_source, sqlite_sessions_source, store_source, voice_notes_source,
};

/// Top-level catalog payload returned to the frontend.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DataCatalog {
    pub sources: Vec<DataSource>,
    pub generated_at: String,
}

/// Static description of a single data source.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DataSource {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub location: String,
    pub retention: String,
    pub fields: Vec<FieldMeta>,
    pub sample_row: Option<String>,
    pub description: String,
}

/// One field on a data source row.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FieldMeta {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub description: String,
}

/// Build the static data catalog. Pure function — no I/O.
pub fn build_catalog() -> DataCatalog {
    DataCatalog {
        sources: vec![
            sensor_source(),
            sqlite_sessions_source(),
            snapshots_source(),
            events_log_source(),
            profiles_ergonomic_source(),
            profiles_communication_source(),
            store_source(),
            remote_ws_source(),
            voice_notes_source(),
        ],
        generated_at: chrono::Utc::now().to_rfc3339(),
    }
}

/// IPC command: return the static data catalog describing all desk app data sources.
///
/// Pure metadata — does not query live data. Used by the Catalog tab of the
/// analyst dashboard (E012) to render "what data does this app produce".
#[tauri::command]
pub fn get_data_catalog() -> Result<DataCatalog, String> {
    Ok(build_catalog())
}
