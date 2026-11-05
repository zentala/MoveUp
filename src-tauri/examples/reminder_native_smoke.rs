//! Opt-in isolated host. Never invokes desk_lib::run or the diagnostics smoke scenario.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::{fs, io::Write, path::{Path, PathBuf}, time::Duration};
use desk_lib::reminder_native_fixture::{self as fixture, Role, APP_ID};
use serde_json::{json, Value};
use tauri::Manager;
use webview2_com::{Microsoft::Web::WebView2::Win32::ICoreWebView2Environment7, take_pwstr};
use windows::core::{Interface, PWSTR};

fn validated_root(path: &Path) -> Result<PathBuf, String> {
    use std::os::windows::fs::MetadataExt;
    let metadata = fs::symlink_metadata(path).map_err(|_| "private_root_missing")?;
    if !metadata.is_dir() || metadata.file_attributes() & 0x400 != 0 { return Err("private_root_reparse_or_not_directory".into()); }
    let root = path.canonicalize().map_err(|_| "private_root_canonical")?;
    let temp = std::env::temp_dir().canonicalize().map_err(|_| "temp_canonical")?;
    let name = root.file_name().and_then(|name| name.to_str()).ok_or("private_root_name")?;
    let suffix = name.strip_prefix("moveup-reminder-native-").ok_or("private_root_prefix")?;
    if root.parent() != Some(temp.as_path()) || uuid::Uuid::parse_str(suffix).is_err() { return Err("private_root_refused".into()); }
    Ok(root)
}
fn claim(root: &Path, role: Role) -> Result<PathBuf, String> {
    use std::os::windows::fs::MetadataExt;
    if role == Role::Input && fs::read_dir(root).map_err(|_| "root_read")?.next().is_some() { return Err("input_requires_fresh_empty_root".into()); }
    fs::OpenOptions::new().write(true).create_new(true).open(root.join(format!("claim-{}", role.name())))
        .map_err(|_| "role_already_claimed")?;
    let profiles = root.join("profiles");
    if !profiles.exists() { fs::create_dir(&profiles).map_err(|_| "profiles_create")?; }
    if fs::symlink_metadata(&profiles).map_err(|_| "profiles_metadata")?.file_attributes() & 0x400 != 0
        || profiles.canonicalize().map_err(|_| "profiles_canonical")?.parent() != Some(root) { return Err("profiles_escape_refused".into()); }
    let profile = profiles.join(role.name());
    fs::create_dir(&profile).map_err(|_| "profile_exists_or_unwritable")?;
    Ok(profile)
}
async fn ownership(app: &tauri::AppHandle, root: &Path, role: Role) -> Result<Value, String> {
    let window = app.get_webview_window(role.label()).ok_or("own_window_missing")?;
    let (hwnd, pid) = {
        let hwnd = window.hwnd().map_err(|_| "own_hwnd")?;
        let mut pid = 0;
        unsafe { windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(hwnd, Some(&mut pid)); }
        (hwnd.0 as usize as u64, pid)
    };
    if pid != std::process::id() { return Err("own_window_pid_mismatch".into()); }
    let (tx, rx) = tokio::sync::oneshot::channel();
    window.with_webview(move |platform| {
        let result: Result<String, String> = (|| {
            let environment = platform.environment().cast::<ICoreWebView2Environment7>().map_err(|_| "profile_api_unavailable")?;
            let mut directory = PWSTR::null();
            unsafe { environment.UserDataFolder(&mut directory) }.map_err(|_| "profile_query_failed")?;
            Ok(take_pwstr(directory))
        })();
        let _ = tx.send(result); // Native callback does not write disk or wait on business state.
    }).map_err(|_| "profile_dispatch")?;
    let actual = tokio::time::timeout(Duration::from_secs(8), rx).await.map_err(|_| "profile_timeout")?.map_err(|_| "profile_reply")??;
    if PathBuf::from(&actual).canonicalize().map_err(|_| "actual_profile")?
        != root.join("profiles").join(role.name()).canonicalize().map_err(|_| "expected_profile")? { return Err("private_profile_mismatch".into()); }
    Ok(json!({"app_id":APP_ID,"role":role.name(),"pid":pid,"hwnd":hwnd,
        "profile":actual,"executable":std::env::current_exe().map_err(|_| "own_executable")?}))
}
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 5 || args[1] != "--root" || args[3] != "--role" { return Err("usage: --root <fresh task Temp root> --role input|reminder".into()); }
    if std::env::current_exe().map_err(|_| "own_executable")?.file_stem().and_then(|value| value.to_str()) != Some("reminder_native_smoke") {
        return Err("production_or_renamed_executable_refused".into());
    }
    let role = Role::parse(&args[4])?;
    let root = validated_root(Path::new(&args[2]))?;
    let context = tauri::generate_context!("tauri.reminder-native-test.conf.json");
    let isolated_assets = matches!(&context.config().build.frontend_dist,
        Some(tauri::utils::config::FrontendDist::Directory(path)) if path.ends_with("node_modules/.cache/reminder-native-dist"));
    if context.config().identifier != APP_ID || !context.config().app.windows.is_empty() || context.config().build.dev_url.is_some() || !isolated_assets {
        return Err("nonisolated_build_configuration_refused".into());
    }
    let profile = claim(&root, role)?;
    fixture::builder(root.clone(), role).setup(move |app| {
        fixture::prepare(app.handle()).map_err(std::io::Error::other)?;
        let window = tauri::WebviewWindowBuilder::new(app, role.label(), tauri::WebviewUrl::App("reminder-native.html".into()))
            .title(format!("MoveUp isolated fixture — {}", role.name())).data_directory(profile)
            .initialization_script(format!("window.__REMINDER_NATIVE_ROLE__={};", serde_json::to_string(role.name()).unwrap()))
            .inner_size(if role == Role::Input { 800.0 } else { 420.0 }, if role == Role::Input { 500.0 } else { 240.0 })
            .position(if role == Role::Input { 80.0 } else { 550.0 }, if role == Role::Input { 80.0 } else { 300.0 })
            .decorations(role == Role::Input).always_on_top(role == Role::Reminder)
            .focused(false).visible(role == Role::Input).build()?;
        let handle = app.handle().clone();
        window.on_window_event(move |event| { if let tauri::WindowEvent::Focused(focused) = event { fixture::focus_event(&handle, *focused); } });
        let handle = app.handle().clone();
        tauri::async_runtime::spawn(async move {
            let result = match ownership(&handle, &root, role).await {
                Ok(identity) => fixture::native_ready(&handle, identity).await,
                Err(error) => Err(error),
            };
            let receipt = json!({"role":role.name(),"pid":std::process::id(),"succeeded":result.is_ok(),"error":result.as_ref().err(),
                "original_incident":"UNKNOWN","installed_production":"NOT_CHECKED","pointer_and_paint":"separate observed evidence required"});
            let written = fs::OpenOptions::new().write(true).create_new(true).open(root.join(format!("outcome-{}.json", role.name())))
                .and_then(|mut file| { file.write_all(&serde_json::to_vec_pretty(&receipt).unwrap())?; file.sync_all() });
            if role == Role::Reminder || result.is_err() || written.is_err() { handle.exit(if result.is_ok() && written.is_ok() { 0 } else { 1 }); }
        });
        Ok(())
    }).build(context).map_err(|_| "fixture_build_failed")?.run(|_, _| {});
    Ok(())
}
fn main() { if let Err(error) = run() { eprintln!("Isolated fixture refused: {error}"); std::process::exit(2); } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roots_are_fresh_named_direct_temp_children_and_roles_are_closed() {
        let valid = std::env::temp_dir().join(format!("moveup-reminder-native-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&valid).unwrap();
        let root = validated_root(&valid).unwrap();
        assert!(claim(&root, Role::Input).is_ok());
        assert!(claim(&root, Role::Input).is_err());
        assert!(claim(&root, Role::Reminder).is_ok());
        let nested = root.join(format!("moveup-reminder-native-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&nested).unwrap();
        assert!(validated_root(&nested).is_err());
        assert!(validated_root(&std::env::temp_dir()).is_err());
        assert!(Role::parse("production").is_err());
        assert!(Role::parse("input").is_ok() && Role::parse("reminder").is_ok());
        // Keep fixture evidence; never recursively remove a computed path.
    }
}
