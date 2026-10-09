//! Revalidate PM3's observation once, inside its callback deadline.
use std::{fs::{self, OpenOptions}, io::{self, Read}, path::Path, time::Duration};
use crate::{release_paths::{self, invalid}, release_install::{StoreLock, complete},
    health_probe::{validate_readiness, ExpectedReadiness}, app_lifecycle_store::RunRecord};

pub fn running(directory: &Path) -> io::Result<Option<RunRecord>> {
    release_paths::no_links(directory)?;
    let owner = directory.join("lifecycle.owner");
    release_paths::no_links(&owner)?;
    release_paths::no_links(&directory.join("lifecycle.json"))?;
    let file = match OpenOptions::new().read(true).write(true).open(owner) {
        Ok(file) => file, Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    match file.try_lock() {
        Ok(()) => return Ok(None),
        Err(std::fs::TryLockError::WouldBlock) => {},
        Err(e) => return Err(io::Error::from(e)),
    }
    let mut bytes = Vec::new();
    fs::File::open(directory.join("lifecycle.json"))?.take(8193).read_to_end(&mut bytes)?;
    if bytes.len() > 8192 { return Err(invalid("oversized run evidence")); }
    let record: RunRecord = serde_json::from_slice(&bytes)?;
    if record.identity.pid == 0 || record.quit_requested
        || !matches!(record.outcome.as_str(), "started" | "ready") {
        return Err(invalid("running identity is unknown or quitting"));
    }
    Ok(Some(record))
}

pub fn promote(lock: &StoreLock, directory: &Path, get: impl Fn(&str) -> Option<String>)
    -> Result<serde_json::Value, String> {
    let required = |key: &str| get(key).filter(|v| !v.is_empty()).ok_or_else(|| format!("missing {key}"));
    let id = required("PM3_CANDIDATE_ID")?;
    let pid: u32 = required("PM3_CANDIDATE_PID")?.parse().map_err(|_| "invalid owned PID")?;
    let run = required("PM3_RUN_ID")?;
    if pid == 0 || required("PM3_OBSERVED_PID")? != pid.to_string()
        || required("PM3_OBSERVED_RUN_ID")? != run {
        return Err("PM3 observed identity does not match its owned child".into());
    }
    let config = crate::supervision::Configuration::from_env(&get)?
        .ok_or("promotion requires configured PM3 supervision")?;
    if required("PM3_SERVICE")? != config.service { return Err("wrong PM3 service".into()); }
    complete(&lock.store.version_dir(&id), &id).map_err(|e| e.to_string())?;
    let mut published = Vec::new();
    fs::File::open(lock.store.root().join("candidates.json")).map_err(|e| e.to_string())?
        .take(1048577).read_to_end(&mut published).map_err(|e| e.to_string())?;
    if published.len() > 1048576 { return Err("oversized candidates".into()); }
    let document: serde_json::Value = serde_json::from_slice(&published).map_err(|e| e.to_string())?;
    if document["version"] != 1 || !document["candidates"].as_array().is_some_and(|items|
        items.iter().any(|c| c["id"] == id && c["argv"][0] == lock.store.version_dir(&id).join("desk.exe").to_string_lossy().as_ref())) {
        return Err("observed candidate is no longer retained/listed".into());
    }
    let record = running(directory).map_err(|e| e.to_string())?.ok_or("no owned active run record")?;
    if record.identity.version != id || record.identity.pid != pid
        || record.identity.supervisor_run_id.as_deref() != Some(&run)
        || record.identity.build_mode != "release" { return Err("wrong active process/run evidence".into()); }
    let port: u16 = get("DESK_READINESS_PORT").unwrap_or_else(|| "3391".into())
        .parse().map_err(|_| "invalid readiness port")?;
    if port == 0 || required("PM3_PROBE_URL")? != format!("http://127.0.0.1:{port}/display/readiness") {
        return Err("callback probe URL does not match configured local readiness".into());
    }
    let body = tokio::runtime::Builder::new_current_thread().enable_all().build()
        .map_err(|e| e.to_string())?.block_on(async {
            let client = reqwest::Client::builder().no_proxy().redirect(reqwest::redirect::Policy::none())
                .timeout(Duration::from_millis(800)).build().map_err(|e| e.to_string())?;
            let mut response = client.get(format!("http://127.0.0.1:{port}/display/readiness"))
                .send().await.map_err(|e| e.to_string())?.error_for_status().map_err(|e| e.to_string())?;
            let mut bytes = Vec::new();
            while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
                if bytes.len() + chunk.len() > 4096 { return Err("oversized readiness".into()); }
                bytes.extend_from_slice(&chunk);
            }
            String::from_utf8(bytes).map_err(|e| e.to_string())
        })?;
    let readiness = validate_readiness(&body, &ExpectedReadiness { version: &id, pid: Some(pid),
        run_id: Some(&record.identity.run_id), supervisor_run_id: Some(&run) })
        .map_err(|e| format!("readiness refused: {e:?}"))?;
    if readiness.build_mode != "release" || readiness.build_mode != record.identity.build_mode {
        return Err("fresh readiness build mode does not match the release artifact and active run".into());
    }
    // Re-read process ownership after IO, before any marker mutation.
    let after = running(directory).map_err(|e| e.to_string())?.ok_or("owner exited during callback")?;
    if after.identity.run_id != record.identity.run_id || after.identity.pid != pid
        || after.identity.version != id || after.identity.build_mode != "release"
        || after.identity.supervisor_run_id.as_deref() != Some(&run) {
        return Err("owner changed during callback".into());
    }
    let next = crate::release_projection::document_for(&lock.store, Some(&id)).map_err(|e| e.to_string())?;
    let old_legacy = fs::read(lock.store.root().join("candidates")).map_err(|e| e.to_string())?;
    crate::release_projection::publish_document(&lock.store, next).map_err(|e| e.to_string())?;
    if let Err(e) = crate::last_known_good::LastKnownGood::in_store(&lock.store).mark(&id) {
        for (name, bytes) in [("candidates", old_legacy), ("candidates.json", published)] {
            let temporary = lock.store.root().join(".candidates-json.tmp");
            fs::write(&temporary, bytes).map_err(|rollback| format!("marker failed: {e}; rollback failed: {rollback}"))?;
            fs::rename(temporary, lock.store.root().join(name)).map_err(|rollback| format!("marker failed: {e}; rollback failed: {rollback}"))?;
        }
        return Err(e.to_string());
    }
    // Installation owns retention; promotion never prunes an artifact during callback IO.
    Ok(serde_json::json!({"status": "promoted", "version": id, "pid": pid, "removed": []}))
}
