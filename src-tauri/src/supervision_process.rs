//! Bounded structured process transport for the frozen public PM3 launcher.
use std::{process::Stdio, time::Duration};
use tokio::{io::{AsyncRead, AsyncReadExt}, process::Command};
use crate::supervision::{Configuration, Response, Transport, MAX_OUTPUT};

pub struct PublicLauncher;

impl PublicLauncher {
    pub async fn activate(&self, config: &Configuration, installed: &std::path::Path,
        remaining: Duration) -> Result<Response, String> {
        invoke(config, "start", "", remaining, Some(installed)).await
    }
}

async fn bounded_read(reader: impl AsyncRead + Unpin) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    reader.take((MAX_OUTPUT + 1) as u64).read_to_end(&mut bytes).await
        .map_err(|e| e.to_string())?;
    if bytes.len() > MAX_OUTPUT { return Err("PM3 output exceeded limit".into()); }
    Ok(bytes)
}

#[async_trait::async_trait]
impl Transport for PublicLauncher {
    async fn invoke(&self, config: &Configuration, action: &str, intent: &str,
        remaining: Duration) -> Result<Response, String> {
        invoke(config, action, intent, remaining, None).await
    }
}

async fn invoke(config: &Configuration, action: &str, intent: &str, remaining: Duration,
    installed: Option<&std::path::Path>) -> Result<Response, String> {
        if !config.node.is_file() || !config.home.join("releases/launch.mjs").is_file() {
            return Err("PM3 Node executable or public launcher is missing".into());
        }
        let mut cmd = Command::new(&config.node);
        cmd.args(config.arguments(action, intent)).env("PM3_HOME", &config.home)
            .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped())
            .kill_on_drop(true);
        if let Some(path) = installed {
            cmd.env("PM3_CONFIG", path).current_dir(path.parent().ok_or("Missing configuration directory")?);
        }
        // All other environment keys (including every fixture isolation key) inherit.
        #[cfg(windows)]
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        let mut child = cmd.spawn().map_err(|e| e.to_string())?;
        let stdout = child.stdout.take().ok_or("Missing stdout pipe")?;
        let stderr = child.stderr.take().ok_or("Missing stderr pipe")?;
        let work = async {
            let (status, out, _) = tokio::try_join!(
                async { child.wait().await.map_err(|e| e.to_string()) },
                bounded_read(stdout), bounded_read(stderr))?;
            Ok(Response { exit: status.code().unwrap_or(-1), stdout: out })
        };
        match tokio::time::timeout(remaining, work).await {
            Ok(result) => result,
            Err(_) => Err("PM3 acknowledgement timed out (outcome unknown)".into()),
        }
        // Dropping the child kills a stuck helper on timeout/read errors.
}
