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
    let legacy = CandidateList::in_store(store);
    let old = match fs::read(legacy.path()) { Ok(bytes) => Some(bytes),
        Err(e) if e.kind() == io::ErrorKind::NotFound => None, Err(e) => return Err(e) };
    let ids = doc["candidates"].as_array().unwrap().iter()
        .map(|c| c["id"].as_str().unwrap().to_string()).collect::<Vec<_>>();
    let tmp = store.root().join(".candidates-json.tmp");
    let result = (|| {
        let mut file = fs::File::create(&tmp)?;
        serde_json::to_writer(&mut file, &doc)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        drop(file);
        legacy.write(&ids)?;
        if let Err(e) = fs::rename(&tmp, store.root().join("candidates.json")) {
            match old { Some(bytes) => { let _ = fs::write(&tmp, bytes);
                let _ = fs::rename(&tmp, legacy.path()); }, None => { let _ = fs::remove_file(legacy.path()); } }
            return Err(e);
        }
        Ok(doc)
    })();
    let _ = fs::remove_file(tmp);
    result
}
