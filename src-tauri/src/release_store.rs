//! release_store.rs — Versioned store of installed builds (E014-T01).
//!
//! Layout, per [ADR 019](../../.arch/ADR/019-release-store-layout.md): every
//! installed build gets its own directory under
//! `%LOCALAPPDATA%\MoveUp\releases\<version>\`. State files that PM3 reads —
//! the `last-known-good` marker (E014-T02) and the candidate list (E014-T04) —
//! are siblings of those directories, so anything in the root that is not a
//! directory is not a release.
//!
//! Three builds are retained. Installing a fourth prunes the oldest, except the
//! build a caller marks as protected: the `last-known-good` build is never
//! pruned, because retention that can delete the fallback is not retention.
//!
//! Ordering comes from the version string, never from directory mtimes — a
//! copy, a restore, or a backup tool walking the tree rewrites mtimes and would
//! silently reorder the rollback chain.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// How many builds the store keeps. See ADR 019 (decision D6).
pub const RETAIN_COUNT: usize = 3;

/// Directory under `%LOCALAPPDATA%\<product>\` that holds the release store.
pub const RELEASES_DIR: &str = "releases";

/// Sort key for a version directory name.
///
/// Dot-separated numeric versions compare component by component, so `0.9.0`
/// sorts before `0.10.0`. A name that is not a numeric version has no defined
/// age, so it sorts as the oldest thing in the store and is pruned first; the
/// name itself breaks ties to keep the order total and stable.
fn version_key(name: &str) -> (Option<Vec<u64>>, String) {
    let parsed = name
        .split('.')
        .map(|part| part.parse::<u64>().ok())
        .collect::<Option<Vec<u64>>>()
        .filter(|parts| !parts.is_empty());
    (parsed, name.to_string())
}

/// A store of installed builds rooted at one `releases` directory.
pub struct ReleaseStore {
    root: PathBuf,
}

impl ReleaseStore {
    /// Opens the store rooted at `root`. Does not touch the filesystem.
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Opens the store under an app-data directory, i.e. `<app_data>/releases`.
    pub fn under_app_data(app_data: &Path) -> Self {
        Self::new(app_data.join(RELEASES_DIR))
    }

    /// The `releases` directory itself.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Directory a given version lives in, whether or not it exists yet.
    pub fn version_dir(&self, version: &str) -> PathBuf {
        self.root.join(version)
    }

    /// Whether a version has a directory in the store.
    pub fn contains(&self, version: &str) -> bool {
        self.version_dir(version).is_dir()
    }

    /// Every retained version, oldest first.
    ///
    /// A store that does not exist yet is an empty store, not an error — that
    /// is the state of a fresh install. Files in the root (the marker, the
    /// candidate list) are not releases and are skipped.
    pub fn versions_oldest_first(&self) -> io::Result<Vec<String>> {
        let entries = match fs::read_dir(&self.root) {
            Ok(entries) => entries,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(err) => return Err(err),
        };

        let mut versions = Vec::new();
        for entry in entries {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            if let Some(name) = entry.file_name().to_str() {
                versions.push(name.to_string());
            }
        }
        versions.sort_by(|a, b| version_key(a).cmp(&version_key(b)));
        Ok(versions)
    }

    /// Every retained version, newest first — the order a candidate list walks.
    pub fn versions_newest_first(&self) -> io::Result<Vec<String>> {
        let mut versions = self.versions_oldest_first()?;
        versions.reverse();
        Ok(versions)
    }

    /// The newest retained version, or `None` when the store is empty.
    pub fn newest(&self) -> io::Result<Option<String>> {
        Ok(self.versions_oldest_first()?.pop())
    }

    /// Creates the directory for `version` and returns it.
    ///
    /// Idempotent: re-installing a version that is already in the store keeps
    /// the existing directory. Does not prune — call [`Self::prune`] after the
    /// build has been written, so a failed install cannot evict a good build.
    pub fn create_version_dir(&self, version: &str) -> io::Result<PathBuf> {
        let dir = self.version_dir(version);
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    /// Prunes oldest-first until at most [`RETAIN_COUNT`] builds remain.
    ///
    /// `protected` names a version that is never removed, however old it is —
    /// in practice the `last-known-good` build. When the oldest build is the
    /// protected one, the next-oldest goes instead, so the store still shrinks
    /// to the retention count.
    ///
    /// Returns the versions actually removed, oldest first.
    pub fn prune(&self, protected: Option<&str>) -> io::Result<Vec<String>> {
        let versions = self.versions_oldest_first()?;
        let mut remaining = versions.len();
        let mut removed = Vec::new();

        for version in versions {
            if remaining <= RETAIN_COUNT {
                break;
            }
            if Some(version.as_str()) == protected {
                continue;
            }
            self.remove_version(&version)?;
            remaining -= 1;
            removed.push(version);
        }
        Ok(removed)
    }

    /// Removes one version's directory. A version that is already gone is not
    /// an error — pruning must be safe to retry after a partial failure.
    fn remove_version(&self, version: &str) -> io::Result<()> {
        match fs::remove_dir_all(self.version_dir(version)) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err),
        }
    }
}
