//! Fail-closed filesystem boundaries for installed release maintenance.
use std::{fs, io, path::{Component, Path}};

pub fn invalid(message: &str) -> io::Error { io::Error::new(io::ErrorKind::InvalidInput, message) }

pub fn version(value: &str) -> io::Result<()> {
    let parts: Vec<_> = value.split('.').collect();
    if parts.len() != 3 || parts.iter().any(|p| p.is_empty() || p.len() > 9
        || !p.bytes().all(|c| c.is_ascii_digit()) || (p.len() > 1 && p.starts_with('0'))) {
        return Err(invalid("version must be three canonical numeric components"));
    }
    Ok(())
}

/// Inspect every existing ancestor, including Windows junctions (reparse points).
pub fn no_links(path: &Path) -> io::Result<()> {
    if !path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(invalid("path must be absolute without parent traversal"));
    }
    let mut walked = std::path::PathBuf::new();
    for part in path.components() {
        walked.push(part);
        match fs::symlink_metadata(&walked) {
            Ok(meta) => {
                #[cfg(windows)]
                let reparse = { use std::os::windows::fs::MetadataExt; meta.file_attributes() & 0x400 != 0 };
                #[cfg(not(windows))]
                let reparse = false;
                if meta.file_type().is_symlink() || reparse { return Err(invalid("symlink/junction paths are forbidden")); }
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {},
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

pub fn tree(path: &Path) -> io::Result<()> {
    no_links(path)?;
    let meta = fs::metadata(path)?;
    if meta.is_dir() {
        for entry in fs::read_dir(path)? { tree(&entry?.path())?; }
    } else if !meta.is_file() || meta.len() == 0 { return Err(invalid("incomplete artifact: empty or special file")); }
    Ok(())
}

pub fn copy_tree(source: &Path, destination: &Path) -> io::Result<()> {
    tree(source)?;
    fs::create_dir(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() { copy_tree(&entry.path(), &target)?; }
        else { fs::copy(entry.path(), target)?; }
    }
    Ok(())
}
