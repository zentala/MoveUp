//! Explicitly isolated Windows host; never calls desk_lib::run/product setup.
//! Build with process-local TAURI_CONFIG set to tauri.ui-diagnostics-test.conf.json
//! contents and `cargo build --example ui_diagnostics_smoke`, then restore the env.
//! Tauri codegen reads the config's parent and merges TAURI_CONFIG, so the filename
//! argument alone does not select an alternate config. Runtime guards remain mandatory.
//! build.rs links the existing Windows resource for Common Controls v6 activation.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use desk_lib::{app_lifecycle::RunId, event_logger::EventLogger, ui_diagnostics::{self as diagnostics, CaptureResult}};
use serde_json::{json, Value};
use std::{path::{Path, PathBuf}, sync::{Arc, atomic::{AtomicBool, Ordering}}, time::Duration};
use tauri::{Manager, menu::{ContextMenu, Menu, MenuBuilder}, tray::TrayIconBuilder};
use webview2_com::{Microsoft::Web::WebView2::Win32::*, CallDevToolsProtocolMethodCompletedHandler, take_pwstr};
use windows::core::{Interface, PCWSTR, PWSTR};

const APP_ID: &str = "io.zntl.moveup.ui-diagnostics-test";
struct Fixture { root: PathBuf, menu: Menu<tauri::Wry>, preview: AtomicBool }
type Result<T> = std::result::Result<T, &'static str>;

fn build_window(app: &tauri::AppHandle, root: &Path, name: &str) -> tauri::Result<tauri::WebviewWindow> {
    let window = tauri::WebviewWindowBuilder::new(app, name, tauri::WebviewUrl::App("ui_diagnostics_smoke.html".into()))
        .title(format!("MoveUp UI diagnostics fixture — {name}"))
        .data_directory(root.join("profiles").join(name))
        .initialization_script(format!("window.__FIXTURE_ROLE__={};", serde_json::to_string(name).unwrap()))
        .inner_size(if name == "main" { 420.0 } else { 800.0 }, if name == "main" { 240.0 } else { 600.0 })
        .decorations(true).visible(true).build()?;
    let receiver = app.clone();
    window.on_window_event(move |event| {
        if matches!(event, tauri::WindowEvent::Focused(true)) {
            let state = receiver.state::<Fixture>();
            if state.preview.swap(false, Ordering::AcqRel) {
                if let Some(window) = receiver.get_webview_window("analyst") {
                    let _ = state.menu.popup_at(window.as_ref().window().clone(), tauri::PhysicalPosition::new(40, 80));
                }
            }
        }
    });
    Ok(window)
}
async fn ui<T: Send + 'static>(app: &tauri::AppHandle, action: impl FnOnce(&tauri::AppHandle) -> T + Send + 'static) -> Result<T> {
    let (tx, rx) = tokio::sync::oneshot::channel(); let receiver = app.clone();
    app.run_on_main_thread(move || { let _ = tx.send(action(&receiver)); }).map_err(|_| "ui_dispatch")?;
    tokio::time::timeout(Duration::from_secs(8), rx).await.map_err(|_| "ui_timeout")?.map_err(|_| "ui_reply")
}
async fn capture(app: &tauri::AppHandle, root: &Path, phase: &str) -> Result<Value> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    diagnostics::capture(app, move |result| { let _ = tx.send(result); });
    let result = tokio::time::timeout(Duration::from_secs(15), rx).await.map_err(|_| "capture_timeout")?.map_err(|_| "capture_reply")?;
    let incomplete = matches!(&result, CaptureResult::Incomplete(_));
    match result {
        CaptureResult::Saved(path) | CaptureResult::Incomplete(path) => {
            let report: Value = serde_json::from_slice(&std::fs::read(&path).map_err(|_| "report_read")?).map_err(|_| "report_json")?;
            let collection_failed = report["facts"]["windows"].as_array().ok_or("report_windows")?.iter()
                .any(|window| window["collection"]["state"] == "collection_failed");
            if collection_failed && !incomplete { return Err("collection_failure_confirmation_missing"); }
            std::fs::write(root.join(format!("{phase}.json")), serde_json::to_vec_pretty(&json!({"report_path":path,"collection_incomplete":incomplete,"report":report})).unwrap()).map_err(|_| "phase_evidence")?;
            Ok(report)
        }
        _ => Err("capture_not_saved"),
    }
}
async fn prove_ownership(app: &tauri::AppHandle, root: &Path) -> Result<()> {
    let mut windows = Vec::new();
    for name in ["main", "analyst"] {
        let window = app.get_webview_window(name).ok_or("owned_window_missing")?;
        let (hwnd_value, pid) = {
            let hwnd = window.hwnd().map_err(|_| "owned_hwnd")?;
            let mut pid = 0;
            unsafe { windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(hwnd, Some(&mut pid)); }
            (hwnd.0 as usize, pid)
        };
        if pid != std::process::id() { return Err("foreign_window_refused"); }
        let (tx, rx) = tokio::sync::oneshot::channel();
        window.with_webview(move |platform| {
            let result: Result<String> = (|| {
                let environment = platform.environment().cast::<ICoreWebView2Environment7>().map_err(|_| "profile_api_unsupported")?;
                let mut directory = PWSTR::null();
                unsafe { environment.UserDataFolder(&mut directory) }.map_err(|_| "profile_query")?;
                Ok(take_pwstr(directory))
            })();
            let _ = tx.send(result);
        }).map_err(|_| "profile_dispatch")?;
        let actual = tokio::time::timeout(Duration::from_secs(8), rx).await.map_err(|_| "profile_timeout")?.map_err(|_| "profile_reply")??;
        let expected = root.join("profiles").join(name).canonicalize().map_err(|_| "profile_expected")?;
        if PathBuf::from(&actual).canonicalize().map_err(|_| "profile_actual")? != expected { return Err("foreign_profile_refused"); }
        windows.push(json!({"label":name,"pid":pid,"hwnd":hwnd_value,"actual_data_directory":actual}));
    }
    std::fs::write(root.join("ownership.json"), serde_json::to_vec_pretty(&json!({
        "app_id":app.config().identifier,"pid":std::process::id(),"executable":std::env::current_exe().map_err(|_| "executable")?,
        "root":root,"windows":windows,"plugins":"none; no product run/setup"})).unwrap()).map_err(|_| "ownership_evidence")?;
    Ok(())
}
async fn scenario(app: &tauri::AppHandle, root: &Path) -> Result<()> {
    prove_ownership(app, root).await?; // No eval/crash/destroy before native identity/profile proof.
    let mut healthy = Value::Null;
    for _ in 0..4 {
        healthy = capture(app, root, "healthy").await?;
        if healthy["facts"]["windows"][0]["collection"]["observation"]["boot"] == "mounted" { break; }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    if healthy["facts"]["windows"][1]["collection"]["state"] != "received" { return Err("analyst_not_received"); }
    let before_navigation = healthy["facts"]["windows"][0]["generation"]["navigation"].as_u64().unwrap();
    ui(app, |app| { let window = app.get_webview_window("main").unwrap(); window.navigate(window.url()?) })
        .await?.map_err(|_| "fixture_navigation")?;
    let mut navigation_proved = false;
    for _ in 0..4 {
        let report = capture(app, root, "navigation").await?;
        let generation = report["facts"]["windows"][0]["generation"]["navigation"].as_u64().unwrap();
        let events = report["facts"]["recent"].as_array().unwrap();
        let main_events: Vec<_> = events.iter().filter(|e| e["first"]["window"] == "main" && e["first"]["generation"]["navigation"] == generation).collect();
        let completed = main_events.iter().find(|e| e["first"]["kind"]["event"] == "navigation_completed" && e["first"]["kind"]["succeeded"] == true);
        navigation_proved = generation > before_navigation && main_events.iter().any(|e| e["first"]["kind"]["event"] == "navigation_started")
            && completed.is_some_and(|done| main_events.iter().any(|e| e["first"]["kind"]["event"] == "probe_finished"
                && e["first"]["sequence"].as_u64() > done["first"]["sequence"].as_u64()))
            && report["facts"]["windows"][0]["collection"]["observation"]["boot"] == "mounted";
        if navigation_proved { break; }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    if !navigation_proved { return Err("navigation_evidence_incomplete"); }
    ui(app, |app| app.get_webview_window("main").unwrap().eval("window.fixtureFallback()")).await?.map_err(|_| "fallback_eval")?;
    let fallback = capture(app, root, "fallback").await?;
    if fallback["facts"]["windows"][0]["collection"]["observation"]["boot"] != "fallback" { return Err("fallback_not_observed"); }
    ui(app, |app| app.get_webview_window("analyst").unwrap().destroy()).await?.map_err(|_| "destroy_analyst")?;
    let absent = capture(app, root, "analyst-absent").await?;
    if absent["facts"]["windows"][1]["collection"]["state"] != "not_present" { return Err("absence_not_observed"); }
    let private = root.to_path_buf();
    ui(app, move |app| build_window(app, &private, "analyst").map(|_| ())).await?.map_err(|_| "recreate_analyst")?;
    prove_ownership(app, root).await?;
    let recreated = capture(app, root, "analyst-recreated").await?;
    if recreated["facts"]["windows"][1]["generation"]["window"].as_u64().unwrap_or(0) < 2 { return Err("recreation_generation"); }
    // Only this host's main COM controller, already proven above. No debug port or process-name kill.
    let main = app.get_webview_window("main").ok_or("main_missing")?;
    let (tx, rx) = tokio::sync::oneshot::channel();
    main.with_webview(move |platform| {
        let method: Vec<u16> = "Page.crash".encode_utf16().chain(Some(0)).collect();
        let params: Vec<u16> = "{}".encode_utf16().chain(Some(0)).collect();
        let callback = CallDevToolsProtocolMethodCompletedHandler::create(Box::new(|_, _| Ok(())));
        let result = unsafe { platform.controller().CoreWebView2().and_then(|webview| webview.CallDevToolsProtocolMethod(PCWSTR(method.as_ptr()), PCWSTR(params.as_ptr()), &callback)) };
        let _ = tx.send(result.is_ok());
    }).map_err(|_| "crash_dispatch")?;
    let _accepted = tokio::time::timeout(Duration::from_secs(8), rx).await.map_err(|_| "crash_submission_timeout")?.map_err(|_| "crash_reply")?;
    let mut crashed = Value::Null;
    for _ in 0..4 {
        crashed = capture(app, root, "main-native-failure").await?;
        if crashed["facts"]["recent"].as_array().unwrap().iter().any(|e| e["first"]["kind"]["event"] == "process_failed") { break; }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    if !crashed["facts"]["recent"].as_array().unwrap().iter().any(|e| e["first"]["kind"]["event"] == "process_failed") { return Err("native_process_failure_not_observed"); }
    if crashed["facts"]["windows"][1]["collection"]["state"] != "received" { return Err("analyst_affected_by_main_failure"); }
    Ok(())
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let root = PathBuf::from(args.get(1).expect("provide a precreated private Temp root"));
    let root = root.canonicalize().expect("private root must exist");
    assert!(root.starts_with(std::env::temp_dir().canonicalize().unwrap()));
    assert!(root.file_name().unwrap().to_string_lossy().starts_with("moveup-ui-diagnostics-fixture-"));
    assert_eq!(std::env::current_exe().unwrap().file_stem().unwrap(), "ui_diagnostics_smoke");
    let context = tauri::generate_context!("tauri.ui-diagnostics-test.conf.json");
    assert_eq!(context.config().identifier, APP_ID);
    assert!(context.config().app.windows.is_empty());
    assert!(context.config().build.dev_url.is_none());
    let keep_open = args.iter().any(|arg| arg == "--keep-open");
    tauri::Builder::default().setup(move |app| {
        let menu = MenuBuilder::new(app).text("settings", "Settings").text("open-analyst", "Open Analyst")
            .text("save-ui-diagnostics", "Save UI diagnostics").separator().text("quit", "Quit fixture").build()?;
        app.manage(Fixture { root: root.clone(), menu: menu.clone(), preview: AtomicBool::new(false) });
        let mut tray = TrayIconBuilder::with_id("ui-diagnostics-fixture").tooltip("MoveUp UI diagnostics fixture").menu(&menu);
        if let Some(icon) = app.default_window_icon() { tray = tray.icon(icon.clone()); }
        tray.build(app)?;
        for name in ["main", "analyst"] { std::fs::create_dir_all(root.join("profiles").join(name))?; build_window(app.handle(), &root, name)?; }
        let identity = RunId { run_id: uuid::Uuid::new_v4().to_string(), pid: std::process::id(), version: env!("CARGO_PKG_VERSION").into(),
            build_mode: "fixture".into(), supervisor_run_id: None };
        diagnostics::initialize(app.handle(), identity, Arc::new(EventLogger::new(root.join("logs"))), root.join("logs"))?;
        let receiver = app.handle().clone();
        tauri::async_runtime::spawn(async move {
            let outcome = scenario(&receiver, &root).await;
            std::fs::write(root.join("fixture-result.json"), serde_json::to_vec_pretty(&json!({
                "result":if outcome.is_ok(){"passed"}else{"failed"},"failure":outcome.err(),"app_id":APP_ID,
                "pid":std::process::id(),"real_sleep_resume":"NOT_CHECKED","production_installation":"NOT_CHECKED"})).unwrap()).unwrap();
            if !keep_open || outcome.is_err() { receiver.exit(if outcome.is_ok() { 0 } else { 1 }); }
            else { receiver.state::<Fixture>().preview.store(true, Ordering::Release); }
        });
        Ok(())
    }).on_menu_event(|app, event| match event.id().as_ref() {
        "quit" => app.exit(0),
        "open-analyst" | "settings" => { if let Some(window) = app.get_webview_window("analyst") { let _ = window.show(); let _ = window.set_focus(); } }
        "save-ui-diagnostics" => {
            let receiver = app.clone();
            diagnostics::capture(app, move |result| {
                let (title, body) = diagnostics::confirmation(&result);
                let native_title = format!("MoveUp UI diagnostics fixture — {title}");
                let root = receiver.state::<Fixture>().root.clone();
                let _ = std::fs::write(root.join("native-confirmation-request.json"), serde_json::to_vec_pretty(&json!({
                    "app_id":APP_ID,"pid":std::process::id(),"title":native_title,"body":body,
                    "seam":"actual native menu -> shared capture -> shared confirmation -> fixture-only MessageBox"})).unwrap());
                tauri::async_runtime::spawn_blocking(move || unsafe {
                    use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OK};
                    let title: Vec<u16> = native_title.encode_utf16().chain(Some(0)).collect();
                    let body: Vec<u16> = body.encode_utf16().chain(Some(0)).collect();
                    // This fixture's own dialog thread needs no WebView/STA or production notification identity.
                    let result = MessageBoxW(None, PCWSTR(body.as_ptr()), PCWSTR(title.as_ptr()), MB_OK);
                    let _ = std::fs::write(root.join("native-confirmation-result.json"),
                        serde_json::to_vec_pretty(&json!({"pid":std::process::id(),"native_result":result.0})).unwrap());
                });
            });
        }
        _ => {}
    }).build(context).expect("isolated host build").run(|app, event| {
        if matches!(event, tauri::RunEvent::Exit) { diagnostics::shutdown(app); }
    });
}
