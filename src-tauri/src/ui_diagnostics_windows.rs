//! WebView2 objects and registrations stay on their owning Tauri UI/STA thread.
use crate::{ui_diagnostics::Diagnostics, ui_diagnostics_probe::{self as probe, Probe, Request}, ui_diagnostics_types::*};
use std::{cell::{Cell, RefCell}, collections::HashMap, rc::Rc, time::Instant};
use tauri::{AppHandle, WebviewWindow, Manager};
use webview2_com::{Microsoft::Web::WebView2::Win32::*, *};
use windows::core::{Interface, IUnknown, PWSTR};

thread_local! {
    static GENERATIONS: Cell<[u64; 2]> = Cell::new([0, 0]);
    static WINDOWS: RefCell<HashMap<usize, Rc<Entry>>> = RefCell::new(HashMap::new());
    static ENVIRONMENTS: RefCell<HashMap<usize, Environment>> = RefCell::new(HashMap::new());
}
struct Entry {
    label: WindowLabel, diagnostics: Diagnostics, webview: ICoreWebView2,
    probe: RefCell<Probe>, clock: Instant, closed: Cell<bool>, navigation_id: Cell<Option<u64>>,
    tokens: RefCell<Vec<(u8, i64)>>, environment: Cell<Option<usize>>, version: RefCell<Option<String>>,
}
impl Entry {
    fn emit(&self, kind: EventKind) { self.emit_at(kind, chrono::Utc::now()); }
    fn emit_at(&self, kind: EventKind, utc: chrono::DateTime<chrono::Utc>) {
        if !self.closed.get() {
            self.diagnostics.submit(Event { sequence: 0, utc, window: self.label,
                generation: self.probe.borrow().generation, kind });
        }
    }
    fn finish(&self) { self.finish_with(false); }
    fn finish_with(&self, failed: bool) {
        self.probe.borrow_mut().settle(self.version.borrow().clone(), failed);
    }
    fn failure(&self, operation: AdapterOperation, code: Option<i32>) { self.emit(EventKind::AdapterFailure { operation, code }); }
}
pub(crate) fn registration_failure(code: i32) -> Capability {
    match code { -2147467262 | -2147467263 => Capability::Unsupported,
        _ => Capability::RegistrationFailed { code: Some(code) } }
}
#[derive(Default)]
pub(crate) struct EnvironmentMembers([Option<u64>; 2]);
impl EnvironmentMembers {
    pub(crate) fn attach(&mut self, label: WindowLabel, generation: u64) { self.0[label.index()] = Some(generation); }
    pub(crate) fn detach(&mut self, label: WindowLabel, generation: u64) -> bool {
        if self.0[label.index()] == Some(generation) { self.0[label.index()] = None; }
        self.0.iter().all(Option::is_none)
    }
    pub(crate) fn owners(&self) -> Vec<(WindowLabel, u64)> {
        [WindowLabel::Main, WindowLabel::Analyst].into_iter().filter_map(|label| self.0[label.index()].map(|generation| (label, generation))).collect()
    }
}
struct Environment { interface: ICoreWebView2Environment5, token: i64, members: EnvironmentMembers }
fn key(label: WindowLabel) -> usize { label.index() }
fn entry(label: WindowLabel) -> Option<Rc<Entry>> { WINDOWS.with(|w| w.borrow().get(&key(label)).cloned()) }

/// Does not show/focus/create a window. No callbacks activate until explicitly called.
pub fn install(window: WebviewWindow, diagnostics: Diagnostics, label: WindowLabel, generation: u64) -> tauri::Result<()> {
    install_ready(window, diagnostics, label, generation, None)
}
fn install_ready(window: WebviewWindow, diagnostics: Diagnostics, label: WindowLabel, generation: u64,
    ready: Option<Box<dyn FnOnce() + Send>>) -> tauri::Result<()> {
    let observer = window.clone();
    let events_app = window.app_handle().clone();
    window.with_webview(move |platform| {
        let acquired = unsafe { platform.controller().CoreWebView2() };
        if let (Some(e), Ok(webview)) = (entry(label), acquired.as_ref()) {
            if (generation == 0 && e.webview.as_raw() == webview.as_raw()) || e.probe.borrow().generation.window == generation {
                if let Some(ready) = ready { ready(); } return;
            }
        }
        let generation = GENERATIONS.with(|counter| {
            let mut values = counter.get();
            let next = if generation == 0 { values[label.index()].saturating_add(1) } else { generation };
            values[label.index()] = values[label.index()].max(next); counter.set(values); next
        });
        remove(label, None);
        let webview = match acquired {
            Ok(webview) => webview,
            Err(error) => { acquisition_failed(&diagnostics, label, generation, error.code().0); if let Some(ready) = ready { ready(); } return; }
        };
        observer.on_window_event(move |event| {
            if matches!(event, tauri::WindowEvent::Destroyed) { let _ = unregister(&events_app, label, generation); }
        });
        let e = Rc::new(Entry { label, diagnostics, webview, probe: RefCell::new(Probe::new(Generation { window: generation, navigation: 0 })),
            clock: Instant::now(), closed: Cell::new(false), navigation_id: Cell::new(None),
            tokens: RefCell::new(Vec::new()), environment: Cell::new(None), version: RefCell::new(None) });
        WINDOWS.with(|w| w.borrow_mut().insert(key(label), e.clone()));
        e.emit(EventKind::Created); register_window(&e); register_environment(&e, platform.environment());
        if let Some(ready) = ready { ready(); }
    })
}
pub(crate) fn acquisition_failed(diagnostics: &Diagnostics, window: WindowLabel, generation: u64, code: i32) {
    // The caller has an actual native window; only its COM acquisition failed.
    for kind in [EventKind::Created, EventKind::AdapterFailure { operation: AdapterOperation::Acquisition, code: Some(code) },
        EventKind::ProcessCapability { capability: Capability::RegistrationFailed { code: Some(code) } },
        EventKind::BrowserCapability { capability: Capability::RegistrationFailed { code: Some(code) } }] {
        diagnostics.submit(Event { sequence: 0, utc: chrono::Utc::now(), window, generation: Generation { window: generation, navigation: 0 }, kind });
    }
}
fn register_window(e: &Rc<Entry>) {
    let weak = Rc::downgrade(e);
    let handler = ProcessFailedEventHandler::create(Box::new(move |_, args| {
        if let Some(e) = weak.upgrade() {
            let mut kind = COREWEBVIEW2_PROCESS_FAILED_KIND(0);
            let kind = args.as_ref().and_then(|a| unsafe { a.ProcessFailedKind(&mut kind).ok().map(|_| kind.0) });
            let extended = args.and_then(|a| a.cast::<ICoreWebView2ProcessFailedEventArgs2>().ok());
            let (mut reason, mut code) = (COREWEBVIEW2_PROCESS_FAILED_REASON(0), 0);
            let reason = extended.as_ref().and_then(|a| unsafe { a.Reason(&mut reason).ok().map(|_| reason.0) });
            let exit_code = extended.as_ref().and_then(|a| unsafe { a.ExitCode(&mut code).ok().map(|_| code) });
            e.emit(EventKind::ProcessFailed { kind, reason, exit_code });
        }
        Ok(())
    }));
    let mut token = 0;
    let capability = match unsafe { e.webview.add_ProcessFailed(&handler, &mut token) } {
        Ok(()) => { e.tokens.borrow_mut().push((0, token)); Capability::Supported }
        Err(error) => registration_failure(error.code().0),
    };
    e.emit(EventKind::ProcessCapability { capability });
    let weak = Rc::downgrade(e);
    let starting = NavigationStartingEventHandler::create(Box::new(move |_, args| {
        if let Some(e) = weak.upgrade() {
            let mut id = 0;
            let id = args.and_then(|a| unsafe { a.NavigationId(&mut id).ok().map(|_| id) });
            if id.is_some() && e.navigation_id.get() == id { return Ok(()); }
            e.navigation_id.set(id);
            let mut generation = e.probe.borrow().generation;
            generation.navigation = generation.navigation.saturating_add(1);
            e.probe.borrow_mut().invalidate_observed(generation, |generation| {
                e.diagnostics.submit(Event { sequence: 0, utc: chrono::Utc::now(), window: e.label,
                    generation, kind: EventKind::NavigationStarted });
            });
        }
        Ok(())
    }));
    register_navigation(e, 1, unsafe { e.webview.add_NavigationStarting(&starting, &mut token) }, token);
    let weak = Rc::downgrade(e);
    let completed = NavigationCompletedEventHandler::create(Box::new(move |_, args| {
        if let (Some(e), Some(args)) = (weak.upgrade(), args) {
            let (mut id, mut success) = (0, windows::core::BOOL(0));
            if unsafe { args.NavigationId(&mut id) }.is_ok() && e.navigation_id.get() == Some(id) {
                let succeeded = unsafe { args.IsSuccess(&mut success) }.is_ok() && success.as_bool();
                e.emit(EventKind::NavigationCompleted { succeeded });
            }
        }
        Ok(())
    }));
    register_navigation(e, 2, unsafe { e.webview.add_NavigationCompleted(&completed, &mut token) }, token);
}
fn register_navigation(e: &Entry, kind: u8, result: windows::core::Result<()>, token: i64) {
    match result { Ok(()) => e.tokens.borrow_mut().push((kind, token)),
        Err(error) => e.failure(AdapterOperation::NavigationRegistration, Some(error.code().0)) }
}
fn register_environment(e: &Rc<Entry>, environment: ICoreWebView2Environment) {
    // Basic version metadata does not depend on the optional environment5 events.
    let mut version = PWSTR::null();
    let version = match unsafe { environment.BrowserVersionString(&mut version) } {
        Ok(()) => {
            let text = take_pwstr(version);
            if text.len() <= 128 && text.chars().all(|c| c.is_ascii_digit() || c == '.') { Some(text) }
            else { e.failure(AdapterOperation::RuntimeVersion, None); None }
        }
        Err(error) => { e.failure(AdapterOperation::RuntimeVersion, Some(error.code().0)); None }
    };
    *e.version.borrow_mut() = version;
    let identity = environment.cast::<IUnknown>().map(|i| i.as_raw() as usize);
    let Ok(identity) = identity else { e.emit(EventKind::BrowserCapability { capability: Capability::Unsupported }); return };
    let existing = ENVIRONMENTS.with(|environments| {
        let mut environments = environments.borrow_mut();
        if let Some(environment) = environments.get_mut(&identity) { environment.members.attach(e.label, e.probe.borrow().generation.window); true } else { false }
    });
    if existing { e.environment.set(Some(identity)); e.emit(EventKind::BrowserCapability { capability: Capability::Supported }); return; }
    let interface = match environment.cast::<ICoreWebView2Environment5>() {
        Ok(interface) => interface,
        Err(error) => { e.emit(EventKind::BrowserCapability { capability: registration_failure(error.code().0) }); return; }
    };
    let handler = BrowserProcessExitedEventHandler::create(Box::new(move |_, args| {
        // One callback per COM environment, not one per window.
        let owners = ENVIRONMENTS.with(|all| all.borrow().get(&identity).map(|e| e.members.owners()).unwrap_or_default());
        let owner = owners.into_iter().find_map(|(label, generation)| entry(label).filter(|e| !e.closed.get() && e.probe.borrow().generation.window == generation));
        if let Some(e) = owner {
            let (mut kind, mut pid) = (COREWEBVIEW2_BROWSER_PROCESS_EXIT_KIND(0), 0);
            let kind = args.as_ref().and_then(|a| unsafe { a.BrowserProcessExitKind(&mut kind).ok().map(|_| kind.0) });
            let process_id = args.as_ref().and_then(|a| unsafe { a.BrowserProcessId(&mut pid).ok().map(|_| pid) });
            e.emit(EventKind::BrowserExited { kind, process_id });
        }
        Ok(())
    }));
    let mut token = 0;
    match unsafe { interface.add_BrowserProcessExited(&handler, &mut token) } {
        Ok(()) => {
            let mut members = EnvironmentMembers::default(); members.attach(e.label, e.probe.borrow().generation.window);
            ENVIRONMENTS.with(|all| all.borrow_mut().insert(identity, Environment { interface, token, members }));
            e.environment.set(Some(identity));
            e.emit(EventKind::BrowserCapability { capability: Capability::Supported });
        }
        Err(error) => e.emit(EventKind::BrowserCapability { capability: registration_failure(error.code().0) }),
    }
}
pub fn request(app: &AppHandle, label: WindowLabel, manual: bool) -> tauri::Result<()> {
    let receiver = app.clone();
    app.run_on_main_thread(move || request_on_sta(receiver, label, manual, None, None))
}
fn request_on_sta(timer_app: AppHandle, label: WindowLabel, manual: bool,
    context: Option<std::sync::Arc<std::sync::OnceLock<(Generation, Option<u64>)>>>,
    waiter: Option<tokio::sync::oneshot::Sender<(Option<String>, bool)>>) {
    let Some(e) = entry(label) else { if let Some(waiter) = waiter { let _ = waiter.send((None, true)); } return };
    if let Some(waiter) = waiter { e.probe.borrow_mut().watch(waiter); }
    let decision = e.probe.borrow_mut().request(e.clock.elapsed(), manual);
    if let Some(context) = context {
        let probe = match decision { Request::Start(ticket) | Request::Reuse(ticket) => Some(ticket.id), _ => None };
        let _ = context.set((e.probe.borrow().generation, probe));
    }
    match decision {
        Request::Reuse(_) if !e.probe.borrow().pending() => { e.finish(); return; }
        Request::Throttled | Request::Suspended => { e.finish(); return; }
        Request::Start(ticket) => {
            e.emit(EventKind::ProbeStarted { id: ticket.id });
            let callback_entry = Rc::downgrade(&e);
            let result = probe::execute(&e.webview, move |result| {
                if let Some(e) = callback_entry.upgrade() {
                    let result = e.probe.borrow_mut().complete(ticket, result);
                    if let Some(result) = result {
                        let failed = matches!(result, Collection::CollectionFailed { .. });
                        e.emit(EventKind::ProbeFinished { id: ticket.id, result }); e.finish_with(failed);
                    }
                }
            });
            if let Err(error) = result {
                let result = e.probe.borrow_mut().complete(ticket, Collection::CollectionFailed { code: Some(error.code().0) });
                if let Some(result) = result { e.emit(EventKind::ProbeFinished { id: ticket.id, result }); }
                e.finish_with(true); return;
            }
            let deadline_diagnostics = e.diagnostics.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(probe::TIMEOUT).await;
                let result = timer_app.run_on_main_thread(move || {
                    if let Some(e) = entry(label).filter(|e| e.probe.borrow().generation.window == ticket.generation.window) {
                        let timed_out = e.probe.borrow_mut().timeout(ticket, e.clock.elapsed());
                        if timed_out { e.emit(EventKind::ProbeFinished { id: ticket.id, result: Collection::Timeout }); e.finish(); }
                    }
                });
                finish_deadline_dispatch(&deadline_diagnostics, label, ticket, result);
            });
        }
        _ => {}
    }
}
/// Actual presence lookup only; collection never shows/focuses/creates a window.
pub fn collect_existing(app: &AppHandle, diagnostics: Diagnostics, label: WindowLabel,
    context: std::sync::Arc<std::sync::OnceLock<(Generation, Option<u64>)>>,
    waiter: tokio::sync::oneshot::Sender<(Option<String>, bool)>) -> tauri::Result<()> {
    let receiver = app.clone();
    app.run_on_main_thread(move || {
        let name = if label == WindowLabel::Main { "main" } else { "analyst" };
        if let Some(window) = receiver.get_webview_window(name) {
            let ready_app = receiver.clone(); let failed = diagnostics.clone(); let unavailable = diagnostics.clone();
            let visibility = window.clone();
            if install_ready(window, diagnostics, label, 0, Some(Box::new(move || {
                if entry(label).is_none() { unavailable.collection_failed(label); }
                match visibility.is_visible() {
                    Ok(visible) => window_evidence(label, EventKind::Visibility { visible }),
                    Err(error) => { let (category, code) = crate::ui_diagnostics::native_failure(&error);
                        window_evidence(label, EventKind::WindowOperationFailed { operation: WindowOperation::Visibility, category, code }); }
                }
                request_on_sta(ready_app, label, true, Some(context), Some(waiter));
            }))).is_err() { failed.collection_failed(label); }
        } else {
            remove(label, None); diagnostics.observed_absence(label); let _ = waiter.send((None, false));
        }
    })
}
pub fn window_evidence(label: WindowLabel, kind: EventKind) {
    if let Some(e) = entry(label) { e.emit(kind); }
}
pub fn record_operation(window: WebviewWindow, diagnostics: Diagnostics, label: WindowLabel,
    operation: WindowOperation, succeeded: bool, failure: Option<(WindowFailureCategory, Option<i32>)>) -> tauri::Result<()> {
    publish_operation(&diagnostics, label, operation, succeeded, failure);
    let visibility = window.clone(); let app = window.app_handle().clone();
    install_ready(window, diagnostics, label, 0, Some(Box::new(move || {
        match visibility.is_visible() {
            Ok(visible) => window_evidence(label, EventKind::Visibility { visible }),
            Err(error) => { let (category, code) = crate::ui_diagnostics::native_failure(&error);
                window_evidence(label, EventKind::WindowOperationFailed { operation: WindowOperation::Visibility, category, code }); }
        }
        if operation == WindowOperation::Show && succeeded { let _ = request(&app, label, false); }
    })))
}
pub(crate) fn publish_operation(diagnostics: &Diagnostics, label: WindowLabel, operation: WindowOperation,
    succeeded: bool, failure: Option<(WindowFailureCategory, Option<i32>)>) {
    let generation = entry(label).map(|e| e.probe.borrow().generation).unwrap_or_default();
    let publish = |kind| { diagnostics.submit(Event { sequence: 0, utc: chrono::Utc::now(), window: label, generation, kind }); };
    let kind = match operation { WindowOperation::Show => EventKind::Show { succeeded },
        WindowOperation::Focus => EventKind::Focus { succeeded }, WindowOperation::Visibility => return };
    publish(kind);
    if let Some((category, code)) = failure { publish(EventKind::WindowOperationFailed { operation, category, code }); }
}
pub fn shutdown() { remove(WindowLabel::Main, None); remove(WindowLabel::Analyst, None); }
pub(crate) fn finish_deadline_dispatch<E>(diagnostics: &Diagnostics, label: WindowLabel,
    ticket: probe::Ticket, result: std::result::Result<(), E>) {
    if result.is_err() {
        diagnostics.submit(Event { sequence: 0, utc: chrono::Utc::now(), window: label,
            generation: ticket.generation, kind: EventKind::ProbeDeadlineDispatchFailed { id: ticket.id } });
    }
}
pub fn unregister(app: &AppHandle, label: WindowLabel, generation: u64) -> tauri::Result<()> {
    app.run_on_main_thread(move || remove(label, Some(generation)))
}
fn remove(label: WindowLabel, generation: Option<u64>) {
    let Some(e) = entry(label).filter(|e| generation.is_none_or(|g| e.probe.borrow().generation.window == g)) else { return };
    e.emit(EventKind::Destroyed); e.closed.set(true);
    let current = e.probe.borrow().generation; e.probe.borrow_mut().invalidate(current);
    for (kind, token) in e.tokens.borrow_mut().drain(..) {
        let result = unsafe { match kind { 0 => e.webview.remove_ProcessFailed(token),
            1 => e.webview.remove_NavigationStarting(token), _ => e.webview.remove_NavigationCompleted(token) } };
        if let Err(error) = result {
            e.diagnostics.submit(Event { sequence: 0, utc: chrono::Utc::now(), window: label, generation: current,
                kind: EventKind::AdapterFailure { operation: AdapterOperation::Cleanup, code: Some(error.code().0) } });
        }
    }
    if let Some(identity) = e.environment.get() {
        let cleanup = ENVIRONMENTS.with(|all| {
            let mut all = all.borrow_mut();
            if let Some(environment) = all.get_mut(&identity) {
                if environment.members.detach(label, current.window) {
                    return Some((environment.interface.clone(), environment.token));
                }
            }
            None
        });
        // Do not hold a RefCell borrow across a COM call that may deliver a callback.
        if let Some((interface, token)) = cleanup {
            if let Err(error) = unsafe { interface.remove_BrowserProcessExited(token) } {
                e.diagnostics.submit(Event { sequence: 0, utc: chrono::Utc::now(), window: label, generation: current,
                    kind: EventKind::AdapterFailure { operation: AdapterOperation::Cleanup, code: Some(error.code().0) } });
                // Retain failed registration to avoid a duplicate on the next window.
            } else { ENVIRONMENTS.with(|all| all.borrow_mut().remove(&identity)); }
        }
    }
    e.finish();
    WINDOWS.with(|w| w.borrow_mut().remove(&key(label)));
}
/// UI-thread only: caller retrieves versions as part of the final tray adapter.
pub fn runtime_version(label: WindowLabel) -> Option<String> {
    entry(label).and_then(|e| e.version.borrow().clone())
}
/// Called only on the UI thread after a real OS power callback dispatch.
pub fn power(suspended: bool, utc: chrono::DateTime<chrono::Utc>) {
    let entries: Vec<_> = WINDOWS.with(|w| w.borrow().values().cloned().collect());
    for e in entries {
        e.probe.borrow_mut().power_observed(suspended, |generation| {
            e.diagnostics.submit(Event { sequence: 0, utc, window: e.label, generation,
                kind: if suspended { EventKind::Suspend } else { EventKind::Resume } });
        });
    }
}
