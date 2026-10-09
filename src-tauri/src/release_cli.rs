//! Maintenance dispatch must precede lifecycle, singleton, dotenv and database IO.
use std::{collections::BTreeMap, path::PathBuf};
use crate::{release_install::{BuildMetadata, StoreLock}, release_paths};
#[path = "release_activation.rs"]
mod activation;

pub fn execute(args: &[String], get: impl Fn(&str) -> Option<String>) -> Result<serde_json::Value, String> {
    let action = args.first().ok_or("missing release action")?;
    if action == "describe" && args.len() == 1 {
        return serde_json::to_value(BuildMetadata::current()).map_err(|e| e.to_string());
    }
    let mut options = BTreeMap::new();
    for pair in args[1..].chunks(2) {
        if pair.len() != 2 || !pair[0].starts_with("--") || options.insert(pair[0].as_str(), pair[1].as_str()).is_some() {
            return Err("expected unique --option value pairs".into());
        }
    }
    let allowed: &[&str] = match action.as_str() {
        "stage" => &["--root", "--source", "--version", "--run-directory"],
        "promote" => &["--root", "--run-directory"],
        "candidates" | "status" => &["--root"],
        "activate" => &[],
        _ => return Err("unknown release action".into()),
    };
    if options.keys().any(|k| !allowed.contains(k)) { return Err("unknown release option".into()); }
    if action == "activate" {
        return activation::execute(get);
    }
    let value = |key| options.get(key).copied().ok_or_else(|| format!("missing {key}"));
    let root = PathBuf::from(value("--root")?);
    let lock = StoreLock::acquire(&root).map_err(|e| e.to_string())?;
    match action.as_str() {
        "stage" => {
            // Malformed supervision must never be silently treated as unsupervised.
            crate::supervision::Configuration::from_env(&get)?;
            let running = crate::release_promotion::running(&PathBuf::from(value("--run-directory")?))
                .map_err(|e| e.to_string())?.map(|r| r.identity.version).into_iter().collect::<Vec<_>>();
            let version = value("--version")?;
            release_paths::version(version).map_err(|e| e.to_string())?;
            let removed = lock.stage(&PathBuf::from(value("--source")?), version, &running).map_err(|e| e.to_string())?;
            Ok(serde_json::json!({"status": "staged", "version": version, "artifact": root.join(version).join("desk.exe"), "removed": removed, "cutover": false}))
        }
        "promote" => crate::release_promotion::promote(&lock, &PathBuf::from(value("--run-directory")?), get),
        "candidates" => crate::release_projection::publish(&lock.store).map_err(|e| e.to_string()),
        _ => {
            let candidates = crate::candidate_list::CandidateList::in_store(&lock.store).read().map_err(|e| e.to_string())?;
            let marker = crate::last_known_good::LastKnownGood::in_store(&lock.store).read().map_err(|e| e.to_string())?;
            Ok(serde_json::json!({"candidates": candidates, "last_known_good": marker,
                "candidate_status": if candidates.is_none() { "missing" } else if candidates.as_ref().unwrap().is_empty() { "empty" } else { "available" },
                "marker_status": if marker.is_none() { "missing" } else { "available" }}))
        }
    }
}

pub fn dispatch() -> Option<i32> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().is_none_or(|a| a != "--release") { return None; }
    Some(match execute(&args[1..], |key| std::env::var(key).ok()) {
        Ok(report) => { println!("{report}"); 0 },
        Err(error) => { eprintln!("{}", serde_json::json!({"status": "failed", "error": error})); 1 },
    })
}
