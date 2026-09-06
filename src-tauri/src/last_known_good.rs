//! last_known_good.rs — The `last-known-good` marker (E014-T02).
//!
//! Per [ADR 019](../../.arch/ADR/019-release-store-layout.md), the marker is the
//! app's durable statement of which installed build last proved itself. It lives
//! next to the version directories, in the release store root, and PM3 reads it
//! to know where to roll back to.
//!
//! Two semantics carry the whole task:
//!
//! - the marker is **absent** until a probe succeeds, so a store that has never
//!   proved a build says so instead of naming one by default;
//! - a **failed** probe leaves the previous marker untouched, so a bad new build
//!   cannot erase the record of the last good one.
//!
//! The probe itself is E014-T03. This module only takes its verdict, expressed
//! as [`ProbeOutcome`], and owns the file.
//!
//! Format is a compatibility surface: one line holding the version string, with
//! a trailing newline. Reading trims surrounding whitespace, and a blank file
//! reads as "no good build" rather than as a build named by the empty string.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::release_store::ReleaseStore;

/// File name of the marker inside the release store root.
pub const MARKER_FILE: &str = "last-known-good";

/// Temp name used while writing the marker.
///
/// Deliberately stable, not unique per write: on Windows a rename over a file
/// somebody momentarily holds open fails, and a unique temp name would leave an
/// orphan nothing ever reclaims. A fixed name is overwritten by the next write.
const MARKER_TMP_FILE: &str = ".last-known-good.tmp";

/// Verdict of the D5 health probe for one build.
///
/// `Failed` covers every way a build can fail to prove itself, including a probe
/// that could not run at all — a port not listening is a failed probe, not an
/// error the caller has to distinguish here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeOutcome {
    /// Process stayed alive 30s and `/display/api` returned JSON.
    Good,
    /// Either condition was not met.
    Failed,
}

/// The `last-known-good` marker file.
pub struct LastKnownGood {
    path: PathBuf,
}

impl LastKnownGood {
    /// Uses `path` as the marker file. Does not touch the filesystem.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// The marker belonging to a release store, i.e. `<store root>/last-known-good`.
    pub fn in_store(store: &ReleaseStore) -> Self {
        Self::new(store.root().join(MARKER_FILE))
    }

    /// The marker file itself, whether or not it exists.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The version last proved good, or `None` when no build has proved itself.
    ///
    /// A missing store and a missing marker are both `None` — that is the state
    /// of a fresh install, not an error. So is a blank marker: a truncated write
    /// leaves no good build, which is safer than naming one that is not there.
    pub fn read(&self) -> io::Result<Option<String>> {
        let contents = match fs::read_to_string(&self.path) {
            Ok(contents) => contents,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(err) => return Err(err),
        };
        let version = contents.trim();
        if version.is_empty() {
            return Ok(None);
        }
        Ok(Some(version.to_string()))
    }

    /// Records a probe verdict for `version` and returns the marker afterwards.
    ///
    /// [`ProbeOutcome::Good`] promotes `version`; [`ProbeOutcome::Failed`] writes
    /// nothing at all, so whatever the marker already said still stands.
    pub fn record_probe(
        &self,
        version: &str,
        outcome: ProbeOutcome,
    ) -> io::Result<Option<String>> {
        if outcome == ProbeOutcome::Good {
            self.mark(version)?;
        }
        self.read()
    }

    /// Writes the marker, replacing any previous one.
    ///
    /// Written to a sibling temp file and renamed, so a reader either sees the
    /// old version or the new one and never a half-written line. Prefer
    /// [`Self::record_probe`]; this is the escape hatch for a caller that
    /// already holds a verdict from somewhere other than the probe.
    pub fn mark(&self, version: &str) -> io::Result<()> {
        let parent = self.path.parent().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "marker path has no parent directory",
            )
        })?;
        fs::create_dir_all(parent)?;

        let tmp = parent.join(MARKER_TMP_FILE);
        if let Err(err) = fs::write(&tmp, format!("{version}\n")) {
            let _ = fs::remove_file(&tmp);
            return Err(err);
        }
        if let Err(err) = fs::rename(&tmp, &self.path) {
            let _ = fs::remove_file(&tmp);
            return Err(err);
        }
        Ok(())
    }
}
