//! candidate_list.rs — The ordered rollback candidate list (E014-T04).
//!
//! PM3 owns the supervision loop; it does not own the question "which build do I
//! try next". That is app knowledge, so MoveUp writes the answer down as a file
//! PM3 reads. Per decision D6 the order is: the newest installed build, then the
//! `last-known-good` build, then the next older retained build.
//!
//! When the newest build IS the last-known-good one, the second entry collapses
//! into the first — a candidate list that names the same build twice would make
//! PM3 retry a build it has already failed on, and would waste one of the three
//! retained slots on a duplicate.
//!
//! Format is a compatibility surface, matching the marker in
//! [`crate::last_known_good`]: one version per line, newline-terminated, most
//! preferred first. The build directory is the sibling of this file named by the
//! version, per [ADR 019](../../.arch/ADR/019-release-store-layout.md).
//!
//! An **empty** file means the store holds no builds. A **missing** file means
//! the list has never been written. Those are different facts and are reported
//! differently: [`CandidateList::read`] answers `None` for the second, never an
//! empty list, so "nobody wrote this yet" can never be read as "no builds".

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::last_known_good::LastKnownGood;
use crate::release_store::{ReleaseStore, RETAIN_COUNT};

/// File name of the candidate list inside the release store root.
pub const CANDIDATES_FILE: &str = "candidates";

/// Temp name used while writing the list.
///
/// Stable, not unique per write, for the reason given in
/// [`crate::last_known_good`]: a unique name turns every failed Windows rename
/// into an orphan nothing reclaims.
const CANDIDATES_TMP_FILE: &str = ".candidates.tmp";

/// Orders rollback candidates per decision D6.
///
/// `newest_first` is the retained versions, most recent first;
/// `last_known_good` is the version the marker names, if any. The result is
/// deduplicated, contains only versions present in `newest_first`, and is capped
/// at [`RETAIN_COUNT`] entries — the store never keeps more than that, so a
/// longer list would name builds that are about to be pruned.
///
/// A `last_known_good` that is not in `newest_first` is dropped: a build that is
/// no longer installed is not a candidate, however good it once was.
pub fn order_candidates(newest_first: &[String], last_known_good: Option<&str>) -> Vec<String> {
    fn push(ordered: &mut Vec<String>, version: &str) {
        if ordered.len() < RETAIN_COUNT && !ordered.iter().any(|seen| seen == version) {
            ordered.push(version.to_string());
        }
    }

    let mut ordered: Vec<String> = Vec::new();
    if let Some(newest) = newest_first.first() {
        push(&mut ordered, newest);
    }
    if let Some(good) = last_known_good {
        if newest_first.iter().any(|version| version == good) {
            push(&mut ordered, good);
        }
    }
    for version in newest_first.iter().skip(1) {
        push(&mut ordered, version);
    }
    ordered
}

/// The candidate list file.
pub struct CandidateList {
    path: PathBuf,
}

impl CandidateList {
    /// Uses `path` as the list file. Does not touch the filesystem.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// The list belonging to a release store, i.e. `<store root>/candidates`.
    pub fn in_store(store: &ReleaseStore) -> Self {
        Self::new(store.root().join(CANDIDATES_FILE))
    }

    /// The list file itself, whether or not it exists.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The candidates as last written, or `None` when the list was never written.
    ///
    /// `Some(vec![])` means the store held no builds at the last write. Blank
    /// lines are skipped so a trailing newline does not read as a nameless build.
    pub fn read(&self) -> io::Result<Option<Vec<String>>> {
        let contents = match fs::read_to_string(&self.path) {
            Ok(contents) => contents,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(err) => return Err(err),
        };
        let versions = contents
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect();
        Ok(Some(versions))
    }

    /// Recomputes the list from the store and the marker, writes it, returns it.
    ///
    /// This is the call site every other module should use: it keeps ordering in
    /// one place instead of letting each caller assemble its own list.
    pub fn refresh(
        &self,
        store: &ReleaseStore,
        marker: &LastKnownGood,
    ) -> io::Result<Vec<String>> {
        let newest_first = store.versions_newest_first()?;
        let good = marker.read()?;
        let ordered = order_candidates(&newest_first, good.as_deref());
        self.write(&ordered)?;
        Ok(ordered)
    }

    /// Writes the list, replacing any previous one.
    ///
    /// Written to a sibling temp file and renamed, so PM3 reads either the whole
    /// old list or the whole new one and never a half-written line.
    pub fn write(&self, versions: &[String]) -> io::Result<()> {
        let parent = self.path.parent().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "candidate list path has no parent directory",
            )
        })?;
        fs::create_dir_all(parent)?;

        let mut body = String::new();
        for version in versions {
            body.push_str(version);
            body.push('\n');
        }

        let tmp = parent.join(CANDIDATES_TMP_FILE);
        if let Err(err) = fs::write(&tmp, body) {
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
