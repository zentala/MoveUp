//! Complete immutable staging, serialized with projection and promotion.
use std::{fs::{self, File, OpenOptions}, io, path::Path};
use crate::{release_paths::{self, invalid}, release_store::ReleaseStore};

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildMetadata {
    pub schema_version: u32,
    pub version: String,
    pub app_id: String,
    pub readiness_contract: u32,
    pub build_mode: String,
}
impl BuildMetadata {
    pub fn current() -> Self { Self { schema_version: 1, version: crate::APP_VERSION.into(),
        app_id: crate::runtime_readiness::APP_ID.into(), readiness_contract: 1,
        build_mode: if cfg!(debug_assertions) { "debug" } else { "release" }.into() } }
}

pub fn complete(directory: &Path, version: &str) -> io::Result<BuildMetadata> {
    release_paths::version(version)?;
    release_paths::tree(directory)?;
    let metadata: BuildMetadata = serde_json::from_slice(&fs::read(directory.join("release.json"))?)?;
    let parts: Vec<u64> = version.split('.').map(|p| p.parse().unwrap()).collect();
    if metadata.schema_version != 1 || metadata.version != version
        || metadata.app_id != crate::runtime_readiness::APP_ID || metadata.readiness_contract != 1
        || metadata.build_mode != "release" || parts.as_slice() <= [0, 6, 0].as_slice() {
        return Err(invalid("build metadata does not declare an installable typed-readiness release"));
    }
    if !directory.join("desk.exe").is_file()
        || !crate::remote_server::default_dist_path(&directory.join("desk.exe")).join("index.html").is_file()
        || !directory.join("dist/welcome.html").is_file()
        || !directory.join("dist/assets").is_dir()
        || fs::read_dir(directory.join("dist/assets"))?.next().is_none() {
        return Err(invalid("complete desk.exe, dist/index.html, dist/welcome.html and dist/assets are required"));
    }
    let references = regex::Regex::new(r#"(?:src|href)=["'](?:/|\./)?(assets/[^"']+)["']"#).unwrap();
    for html in ["index.html", "welcome.html"] {
        let body = fs::read_to_string(directory.join("dist").join(html))?;
        for asset in references.captures_iter(&body) {
            let path = directory.join("dist").join(&asset[1]);
            release_paths::no_links(&path)?;
            if !path.is_file() { return Err(invalid("frontend references missing assets")); }
        }
    }
    Ok(metadata)
}

pub struct StoreLock { pub store: ReleaseStore, _lock: File }
impl StoreLock {
    pub fn acquire(root: &Path) -> io::Result<Self> {
        release_paths::no_links(root)?;
        fs::create_dir_all(root)?;
        // Also reject malicious state-file redirects before opening any writer.
        for name in [".release.owner", "candidates", "candidates.json", "last-known-good",
            ".candidates.tmp", ".candidates-json.tmp", ".last-known-good.tmp"] {
            release_paths::no_links(&root.join(name))?;
        }
        let lock = OpenOptions::new().read(true).write(true).create(true).truncate(false)
            .open(root.join(".release.owner"))?;
        lock.try_lock().map_err(io::Error::from)?;
        Ok(Self { store: ReleaseStore::new(root.into()), _lock: lock })
    }

    pub fn stage(&self, source: &Path, version: &str, running: &[String]) -> io::Result<Vec<String>> {
        release_paths::no_links(source)?;
        if fs::canonicalize(self.store.root())?.starts_with(fs::canonicalize(source)?) {
            return Err(invalid("release root must not be inside the build source"));
        }
        complete(source, version)?;
        for id in running { release_paths::version(id)?; }
        // Preflight removals before publishing anything. Invocation temp directories
        // are never releases; leftover stages cannot enter ordering or retention.
        for id in self.store.versions_oldest_first()? {
            if release_paths::version(&id).is_ok() { release_paths::tree(&self.store.version_dir(&id))?; }
        }
        let destination = self.store.version_dir(version);
        if destination.exists() { return Err(io::Error::new(io::ErrorKind::AlreadyExists, "immutable release already exists")); }
        let temp = self.store.root().join(format!(".stage-{}", uuid::Uuid::new_v4()));
        let result = (|| {
            release_paths::copy_tree(source, &temp)?;
            complete(&temp, version)?;
            fs::rename(&temp, &destination)?;
            let mut protected = running.to_vec(); protected.push(version.into());
            match self.retain(&protected) {
                Ok(removed) => Ok(removed),
                Err(e) => { let _ = fs::remove_dir_all(&destination); Err(e) },
            }
        })();
        if temp.exists() { let _ = fs::remove_dir_all(temp); }
        result
    }

    /// Protect all explicitly running builds, this maintenance binary, and the marker.
    pub fn retain(&self, running: &[String]) -> io::Result<Vec<String>> {
        for id in running { release_paths::version(id)?; }
        let good = crate::last_known_good::LastKnownGood::in_store(&self.store).read()?;
        let current = std::env::current_exe().ok();
        let versions = self.store.versions_oldest_first()?.into_iter()
            .filter(|id| release_paths::version(id).is_ok()).collect::<Vec<_>>();
        let mut remaining = versions.len();
        let mut removed = Vec::new();
        for id in versions {
            if remaining <= crate::release_store::RETAIN_COUNT { break; }
            release_paths::version(&id)?;
            if good.as_deref() == Some(&id) || running.contains(&id)
                || complete(&self.store.version_dir(&id), &id).is_err()
                || current.as_ref().is_some_and(|p| {
                    p.parent().and_then(|dir| fs::canonicalize(dir).ok())
                        == fs::canonicalize(self.store.version_dir(&id)).ok()
                }) { continue; }
            removed.push(id);
            remaining -= 1;
        }
        if remaining > crate::release_store::RETAIN_COUNT {
            return Err(invalid("retention cannot remove protected running/known-good releases"));
        }
        let retired = self.store.root().join(format!(".retired-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&retired)?;
        let mut moved = Vec::new();
        let result = (|| {
            for id in &removed {
                release_paths::tree(&self.store.version_dir(id))?;
                fs::rename(self.store.version_dir(id), retired.join(id))?;
                moved.push(id.clone());
            }
            crate::release_projection::publish(&self.store)?;
            Ok(())
        })();
        if let Err(error) = result {
            for id in moved {
                fs::rename(retired.join(&id), self.store.version_dir(&id))
                    .map_err(|e| io::Error::other(format!("publication failed: {error}; retention rollback failed: {e}")))?;
            }
            let _ = fs::remove_dir(&retired);
            return Err(error);
        }
        // Only invocation-owned retired artifacts, already absent from the published
        // list. Cleanup failure cannot undo committed publication or certify failure.
        let _ = fs::remove_dir_all(retired);
        Ok(removed)
    }
}
