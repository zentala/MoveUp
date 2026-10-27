//! Logical deadlines never release a physical ExecuteScript slot.
use crate::ui_diagnostics_types::*;
use std::time::Duration;
pub const SCRIPT: &str = include_str!("ui_diagnostics_probe.js");
pub const TIMEOUT: Duration = Duration::from_secs(3);
pub const THROTTLE: Duration = Duration::from_secs(5);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ticket { pub id: u64, pub generation: Generation, pub started: Duration }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request { Start(Ticket), Reuse(Ticket), Throttled, Suspended }
pub struct Probe {
    pub generation: Generation,
    physical: Option<Ticket>, logical: bool, last_start: Option<Duration>, next_id: u64, suspended: bool,
}
impl Probe {
    pub fn new(generation: Generation) -> Self {
        Self { generation, physical: None, logical: false, last_start: None, next_id: 0, suspended: false }
    }
    pub fn request(&mut self, now: Duration, manual: bool) -> Request {
        if self.suspended { return Request::Suspended; }
        if let Some(ticket) = self.physical { return Request::Reuse(ticket); }
        if !manual && self.last_start.is_some_and(|last| now.saturating_sub(last) < THROTTLE) { return Request::Throttled; }
        self.next_id = self.next_id.checked_add(1).expect("probe ID exhausted");
        let ticket = Ticket { id: self.next_id, generation: self.generation, started: now };
        self.physical = Some(ticket); self.logical = true; self.last_start = Some(now);
        Request::Start(ticket)
    }
    pub fn timeout(&mut self, ticket: Ticket, now: Duration) -> bool {
        if self.physical != Some(ticket) || !self.logical || now.saturating_sub(ticket.started) < TIMEOUT { return false; }
        self.logical = false;
        true
    }
    pub fn invalidate(&mut self, generation: Generation) {
        self.generation = generation; self.logical = false;
    }
    pub fn power(&mut self, suspended: bool) { self.suspended = suspended; self.logical = false; }
    pub fn complete(&mut self, ticket: Ticket, result: Collection) -> Option<Collection> {
        if self.physical != Some(ticket) { return None; }
        self.physical = None;
        let current = self.logical && !self.suspended && ticket.generation == self.generation;
        self.logical = false;
        current.then_some(result)
    }
}
pub fn parse_response(text: &str) -> Collection {
    if text.len() > REPORT_LIMIT { return Collection::CollectionFailed { code: Some(-1) }; }
    match serde_json::from_str::<PageObservation>(text) {
        Ok(page) if page.valid() => Collection::Received(page),
        _ => Collection::CollectionFailed { code: Some(-2) },
    }
}

#[cfg(windows)]
mod completion {
    use super::*;
    use std::{cell::RefCell};
    use windows::core::{implement, HRESULT, PCWSTR};
    use webview2_com::Microsoft::Web::WebView2::Win32::{ICoreWebView2ExecuteScriptCompletedHandler, ICoreWebView2ExecuteScriptCompletedHandler_Impl};
    type Callback = Box<dyn FnOnce(Collection)>;
    #[implement(ICoreWebView2ExecuteScriptCompletedHandler)]
    struct Completion(RefCell<Option<Callback>>);
    impl ICoreWebView2ExecuteScriptCompletedHandler_Impl for Completion_Impl {
        fn Invoke(&self, errorcode: HRESULT, result: &PCWSTR) -> windows::core::Result<()> {
            let callback = self.0.borrow_mut().take();
            if let Some(callback) = callback {
                let collection = if errorcode.is_err() { Collection::CollectionFailed { code: Some(errorcode.0) } }
                    else { unsafe { bounded_result(*result) } };
                callback(collection);
            }
            Ok(())
        }
    }
    unsafe fn bounded_result(text: PCWSTR) -> Collection {
        if text.is_null() { return Collection::CollectionFailed { code: Some(-2) }; }
        let mut length = 0;
        while length <= REPORT_LIMIT && *text.0.add(length) != 0 { length += 1; }
        if length > REPORT_LIMIT { return Collection::CollectionFailed { code: Some(-1) }; }
        match String::from_utf16(std::slice::from_raw_parts(text.0, length)) {
            Ok(value) => parse_response(&value), _ => Collection::CollectionFailed { code: Some(-2) },
        }
    }
    pub fn handler(callback: impl FnOnce(Collection) + 'static) -> ICoreWebView2ExecuteScriptCompletedHandler {
        Completion(RefCell::new(Some(Box::new(callback)))).into()
    }
    /// No COM object crosses threads; the adapter owns this state on its STA.
    pub fn execute(webview: &webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2,
        callback: impl FnOnce(Collection) + 'static) -> windows::core::Result<()> {
        let script: Vec<u16> = SCRIPT.encode_utf16().chain(Some(0)).collect();
        let handler = handler(callback);
        unsafe { webview.ExecuteScript(PCWSTR(script.as_ptr()), &handler) }
    }
}
#[cfg(windows)] pub use completion::execute;
