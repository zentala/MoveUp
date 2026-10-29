//! Existing daily log directory, bounded serialization and exclusive publication.
use crate::ui_diagnostics_types::{IoFailure, Report, REPORT_LIMIT};
use serde::Serialize;
use std::{fs::{self, OpenOptions}, io::{self, Write}, path::{Path, PathBuf}};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReportError {
    pub stage: &'static str,
    pub kind: IoFailure,
    /// Publication succeeded but temporary-file cleanup failed.
    pub published_path: Option<PathBuf>,
}
fn failure(stage: &'static str, error: io::Error) -> ReportError {
    ReportError { stage, kind: error.kind().into(), published_path: None }
}

struct BoundedJson { bytes: Vec<u8>, exceeded: bool }
impl Write for BoundedJson {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > REPORT_LIMIT.saturating_sub(self.bytes.len()) {
            self.exceeded = true;
            return Err(io::ErrorKind::InvalidData.into());
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}
pub(crate) fn serialize(value: &impl Serialize) -> Result<Vec<u8>, ReportError> {
    let mut output = BoundedJson { bytes: Vec::new(), exceeded: false };
    serde_json::to_writer(&mut output, value).map_err(|_| ReportError {
        stage: if output.exceeded { "report_too_large" } else { "serialization" },
        kind: IoFailure::InvalidData, published_path: None,
    })?;
    Ok(output.bytes)
}

pub fn save_report(log_directory: &Path, report: &Report) -> Result<PathBuf, ReportError> {
    let bytes = serialize(report)?;
    let local = report.collected_utc.with_timezone(&chrono::Local);
    let day = log_directory.join(local.format("%Y-%m-%d").to_string());
    fs::create_dir_all(&day).map_err(|e| failure("day_directory", e))?;
    let filename = format!("ui-diagnostics-{}-{}.json", local.format("%H-%M-%S"), uuid::Uuid::new_v4());
    publish(&day, &filename, &bytes)
}

/// Exclusive hard-link publication is atomic and cannot replace a racing report.
/// No rename/copy fallback; remove only this invocation's private sibling temp.
pub(crate) fn publish(directory: &Path, filename: &str, bytes: &[u8]) -> Result<PathBuf, ReportError> {
    if Path::new(filename).components().count() != 1 || filename == "." || filename == ".."
        || filename.contains(['/', '\\', ':']) || bytes.len() > REPORT_LIMIT {
        return Err(failure("invalid_report", io::ErrorKind::InvalidInput.into()));
    }
    let path = directory.join(filename);
    let temporary = directory.join(format!(".ui-diagnostics-{}.tmp", uuid::Uuid::new_v4()));
    let mut file = OpenOptions::new().write(true).create_new(true).open(&temporary)
        .map_err(|e| failure("temporary_create", e))?;
    let result = (|| {
        file.write_all(bytes).map_err(|e| failure("temporary_write", e))?;
        file.sync_all().map_err(|e| failure("temporary_sync", e))?;
        drop(file);
        fs::hard_link(&temporary, &path).map_err(|e| failure("publish", e))?;
        Ok(path.clone())
    })();
    match fs::remove_file(&temporary) {
        Ok(()) => result,
        Err(error) => match result {
            Ok(path) => Err(ReportError { stage: "temporary_cleanup", kind: error.kind().into(),
                published_path: Some(path) }),
            Err(original) => Err(original),
        },
    }
}
