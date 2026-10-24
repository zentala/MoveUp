//! Atomic file replacement for the one shared settings Store.

use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use log::warn;

pub(crate) fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    atomic_write_with(path, bytes, || Ok(()), || {})
}

#[cfg(test)]
pub(crate) fn atomic_write_with(
    path: &Path,
    bytes: &[u8],
    before_replace: impl FnOnce() -> Result<(), String>,
    after_replace: impl FnOnce(),
) -> Result<(), String> {
    atomic_write_inner(path, bytes, before_replace, after_replace)
}

#[cfg(not(test))]
fn atomic_write_with(
    path: &Path,
    bytes: &[u8],
    before_replace: impl FnOnce() -> Result<(), String>,
    after_replace: impl FnOnce(),
) -> Result<(), String> {
    atomic_write_inner(path, bytes, before_replace, after_replace)
}

fn atomic_write_inner(
    path: &Path,
    bytes: &[u8],
    before_replace: impl FnOnce() -> Result<(), String>,
    after_replace: impl FnOnce(),
) -> Result<(), String> {
    let parent = path.parent().ok_or_else(|| format!("Settings path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("Cannot create settings directory {}: {error}", parent.display()))?;

    let (temporary, mut file) = create_temporary(parent)?;
    let mut cleanup = TemporaryFile(temporary.clone());
    file.write_all(bytes)
        .and_then(|_| file.flush())
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("Cannot flush temporary settings file {}: {error}", temporary.display()))?;
    drop(file);

    before_replace()?;
    fs::rename(&temporary, path).map_err(|error| {
        format!("Cannot atomically replace settings file {}: {error}", path.display())
    })?;
    cleanup.0 = PathBuf::new();
    after_replace();
    Ok(())
}

fn create_temporary(parent: &Path) -> Result<(PathBuf, File), String> {
    for _ in 0..8 {
        let path = parent.join(format!(".settings-{}.tmp", uuid::Uuid::new_v4()));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(format!("Cannot create temporary settings file: {error}")),
        }
    }
    Err("Cannot allocate a unique temporary settings file name".into())
}

struct TemporaryFile(PathBuf);

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        if self.0.as_os_str().is_empty() {
            return;
        }
        if let Err(error) = fs::remove_file(&self.0) {
            if error.kind() != std::io::ErrorKind::NotFound {
                warn!("Failed to remove temporary settings file {}: {error}", self.0.display());
            }
        }
    }
}
