//! PM3 candidate-file v1 beside the retained newline compatibility format.
use std::{fs, io::{self, Write}};
use crate::{candidate_list::{CandidateList, order_candidates}, last_known_good::LastKnownGood,
    release_store::ReleaseStore, release_install::complete};

pub fn document(store: &ReleaseStore) -> io::Result<serde_json::Value> {
    let good = LastKnownGood::in_store(store).read()?;
    document_for(store, good.as_deref())
}

pub fn document_for(store: &ReleaseStore, good: Option<&str>) -> io::Result<serde_json::Value> {
    let versions = store.versions_newest_first()?.into_iter()
        .filter(|id| complete(&store.version_dir(id), id).is_ok()).collect::<Vec<_>>();
    let ordered = order_candidates(&versions, good);
    Ok(serde_json::json!({"version": 1, "candidates": ordered.iter().map(|id|
        serde_json::json!({"id": id, "argv": [store.version_dir(id).join("desk.exe"), "--minimized"],
            "health": {"expectJson": {"version": id, "build_mode": "release"}}})).collect::<Vec<_>>() }))
}

/// A serialized writer prepares all bytes before replacing either public file.
/// PM3 consumes only the atomic JSON commit; legacy readers retain their format.
pub fn publish(store: &ReleaseStore) -> io::Result<serde_json::Value> {
    let doc = document(store)?;
    publish_document(store, doc)
}

pub fn publish_document(store: &ReleaseStore, doc: serde_json::Value) -> io::Result<serde_json::Value> {
    publish_with_fault(store, doc, |_| Ok(()))
}

fn publish_with_fault(store: &ReleaseStore, doc: serde_json::Value,
    mut before: impl FnMut(&str) -> io::Result<()>) -> io::Result<serde_json::Value> {
    let legacy = CandidateList::in_store(store);
    let old = match fs::read(legacy.path()) { Ok(bytes) => Some(bytes),
        Err(e) if e.kind() == io::ErrorKind::NotFound => None, Err(e) => return Err(e) };
    let ids = doc["candidates"].as_array().unwrap().iter()
        .map(|c| c["id"].as_str().unwrap().to_string()).collect::<Vec<_>>();
    let tmp = store.root().join(".candidates-json.tmp");
    let mut recovery_bytes = false;
    let result = (|| {
        let mut file = fs::File::create(&tmp)?;
        serde_json::to_writer(&mut file, &doc)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        drop(file);
        legacy.write(&ids)?;
        if let Err(e) = fs::rename(&tmp, store.root().join("candidates.json")) {
            let restored = (|| {
                match old { Some(bytes) => {
                    before("restore-write")?;
                    fs::write(&tmp, bytes)?;
                    recovery_bytes = true;
                    before("restore-rename")?;
                    fs::rename(&tmp, legacy.path())?;
                    recovery_bytes = false;
                }, None => {
                    before("restore-remove")?;
                    fs::remove_file(legacy.path())?;
                } }
                Ok::<(), io::Error>(())
            })();
            return Err(io::Error::new(e.kind(), match restored {
                Ok(()) => format!("JSON publication failed: {e}; previous legacy candidates restored"),
                Err(restore) => format!("JSON publication failed: {e}; legacy restoration failed: {restore}; candidate state uncertain; saved prior bytes: {}",
                    if recovery_bytes { tmp.display().to_string() } else { "unavailable".into() }),
            }));
        }
        Ok(doc)
    })();
    if !recovery_bytes { let _ = fs::remove_file(tmp); }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publication_failure_reports_successful_restoration_and_preserves_marker() {
        let temp = tempfile::tempdir().unwrap();
        let store = ReleaseStore::new(temp.path().into());
        fs::write(temp.path().join("candidates"), b"0.6.1\n").unwrap();
        fs::write(temp.path().join("last-known-good"), b"0.6.1\n").unwrap();
        fs::create_dir(temp.path().join("candidates.json")).unwrap();
        let doc = serde_json::json!({"version":1,"candidates":[{"id":"0.6.2"}]});
        let error = publish_document(&store, doc).unwrap_err().to_string();
        assert!(error.contains("JSON publication failed") && error.contains("previous legacy candidates restored"), "{error}");
        assert_eq!(fs::read(temp.path().join("candidates")).unwrap(), b"0.6.1\n");
        assert_eq!(fs::read(temp.path().join("last-known-good")).unwrap(), b"0.6.1\n");
        assert!(temp.path().join("candidates.json").is_dir());
    }

    #[test]
    fn compound_restoration_failures_report_primary_and_uncertain_state() {
        for operation in ["restore-write", "restore-rename", "restore-remove"] {
            let temp = tempfile::tempdir().unwrap();
            let store = ReleaseStore::new(temp.path().into());
            if operation != "restore-remove" {
                fs::write(temp.path().join("candidates"), b"0.6.1\n").unwrap();
            }
            fs::write(temp.path().join("last-known-good"), b"0.6.1\n").unwrap();
            fs::create_dir(temp.path().join("candidates.json")).unwrap();
            let doc = serde_json::json!({"version":1,"candidates":[{"id":"0.6.2"}]});
            let error = publish_with_fault(&store, doc, |at| {
                if at == operation { Err(io::Error::other(format!("injected {at}"))) } else { Ok(()) }
            }).unwrap_err().to_string();
            assert!(error.contains("JSON publication failed") && error.contains("legacy restoration failed")
                && error.contains(operation) && error.contains("state uncertain"), "{error}");
            assert_eq!(fs::read(temp.path().join("candidates")).unwrap(), b"0.6.2\n");
            assert_eq!(fs::read(temp.path().join("last-known-good")).unwrap(), b"0.6.1\n");
            assert!(temp.path().join("candidates.json").is_dir());
            if operation == "restore-rename" {
                assert_eq!(fs::read(temp.path().join(".candidates-json.tmp")).unwrap(), b"0.6.1\n");
                assert!(error.contains(".candidates-json.tmp"));
            }
        }
    }
}
