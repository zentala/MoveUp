//! Installed activation uses the public text CLI; success is not readiness proof.
use std::{fs, path::PathBuf};
use crate::{release_paths, supervision::{Configuration, DEADLINE}, supervision_process::PublicLauncher};

pub fn execute(get: impl Fn(&str) -> Option<String>) -> Result<serde_json::Value, String> {
    let config = Configuration::from_env(&get)?.ok_or("activation requires PM3 configuration")?;
    let installed = PathBuf::from(get("DESK_PM3_CONFIG").ok_or("Set DESK_PM3_CONFIG to the generated installed pm3.yaml")?);
    release_paths::no_links(&installed).map_err(|e| e.to_string())?;
    // prepare-pm3 publishes JSON syntax (valid YAML). Reject arbitrary inherited config.
    let document: serde_json::Value = serde_json::from_slice(&fs::read(&installed).map_err(|e| e.to_string())?)
        .map_err(|e| format!("Installed configuration must be generated JSON/YAML: {e}"))?;
    let services = document["services"].as_array().ok_or("Installed services are missing")?;
    let service = services.iter().find(|s| format!("{}/{}", document["name"].as_str().unwrap_or(""),
        s["name"].as_str().unwrap_or("")) == config.service).ok_or("Installed service identity mismatch")?;
    let env = &service["env"];
    if document["version"] != 1 || document["nodeAutoInstall"] != false || service["boot"] != false
        || env["DESK_SUPERVISION"] != "pm3" || env["DESK_PM3_SERVICE"] != config.service
        || env["DESK_PM3_NODE"].as_str() != config.node.to_str()
        || env["DESK_PM3_HOME"].as_str() != config.home.to_str()
        || env["DESK_PM3_CONFIG"].as_str() != installed.to_str() {
        return Err("Installed supervision configuration mismatch".into());
    }
    for path in [service["cwd"].as_str(), service["supervised"]["candidatesFile"].as_str()] {
        release_paths::no_links(&PathBuf::from(path.ok_or("Installed supervision path missing")?))
            .map_err(|e| e.to_string())?;
    }
    if !PathBuf::from(service["cwd"].as_str().unwrap()).is_dir()
        || !PathBuf::from(service["supervised"]["candidatesFile"].as_str().unwrap()).is_file() {
        return Err("Installed directory or candidates file missing".into());
    }
    tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|e| e.to_string())?
        .block_on(async {
            let response = PublicLauncher.activate(&config, &installed, DEADLINE).await?;
            let output = String::from_utf8(response.stdout).map_err(|_| "PM3 activation output invalid; outcome unknown")?;
            if response.exit != 0 {
                return Err(format!("PM3 activation failed (exit {}); outcome unconfirmed; stdout: {}", response.exit, output.trim()));
            }
            Ok(serde_json::json!({"status": "activation-command-completed", "service": config.service,
                "exit": response.exit, "output": output.trim(), "readiness": "not-checked", "configuration": installed}))
        })
}
