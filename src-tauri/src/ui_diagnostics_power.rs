//! Real process-level Windows suspend/resume notifications, opt-in at integration.
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use crate::{ui_diagnostics::Diagnostics, ui_diagnostics_types::*};
pub struct PowerSignal { active: AtomicBool, last: AtomicU8 }
impl Default for PowerSignal {
    fn default() -> Self { Self { active: AtomicBool::new(true), last: AtomicU8::new(0) } }
}
impl PowerSignal {
    pub fn receive(&self, event: u32) -> Option<bool> {
        if !self.active.load(Ordering::Acquire) { return None; }
        let state = match event { 4 => 1, 7 | 18 => 2, _ => return None };
        (self.last.swap(state, Ordering::AcqRel) != state).then_some(state == 1)
    }
    pub fn deactivate(&self) { self.active.store(false, Ordering::Release); }
}
/// Failed unregistration retains the token; retry never unregisters a new owner.
pub struct PowerLease { token: Option<isize> }
impl PowerLease {
    pub fn new(token: isize) -> Self { Self { token: Some(token) } }
    pub fn close(&mut self, unregister: impl FnOnce(isize) -> u32) -> Result<(), u32> {
        if let Some(token) = self.token {
            let code = unregister(token);
            if code != 0 { return Err(code); }
            self.token = None;
        }
        Ok(())
    }
}
pub(crate) fn publish_failure(diagnostics: &Diagnostics, operation: PowerOperation, code: Option<i32>) {
    diagnostics.submit(Event { sequence: 0, utc: chrono::Utc::now(), window: WindowLabel::Main,
        generation: Generation::default(), kind: EventKind::PowerFailure { operation, code } });
}
#[cfg(windows)]
mod native {
    use super::*;
    use tauri::AppHandle;
    use std::sync::{Arc, Mutex, atomic::{AtomicUsize, AtomicU64}};
    use windows::Win32::{Foundation::HANDLE, System::Power::{PowerRegisterSuspendResumeNotification,
        PowerUnregisterSuspendResumeNotification, DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS, HPOWERNOTIFY},
        UI::WindowsAndMessaging::DEVICE_NOTIFY_CALLBACK};
    struct Context { app: AppHandle, diagnostics: Diagnostics, signal: PowerSignal }
    static CONTEXT: Mutex<Option<(usize, Arc<Context>)>> = Mutex::new(None);
    static NEXT_ID: AtomicUsize = AtomicUsize::new(1);
    static DROPPED: AtomicU64 = AtomicU64::new(0);
    pub fn dropped_callbacks() -> u64 { DROPPED.load(Ordering::Relaxed) }
    unsafe extern "system" fn callback(context: *const core::ffi::c_void, event: u32, _: *const core::ffi::c_void) -> u32 {
        // Context is an opaque generation ID, never a dereferenced allocation.
        // A late callback cannot observe a freed context or a newer registration.
        let context = match CONTEXT.try_lock() {
            Ok(slot) => slot.as_ref().filter(|(id, _)| *id == context as usize).map(|(_, c)| c.clone()),
            Err(_) => { DROPPED.fetch_add(1, Ordering::Relaxed); None }
        };
        if let Some(context) = context {
            if let Some(suspended) = context.signal.receive(event) {
                let pending = context.clone();
                let utc = chrono::Utc::now();
                if context.app.run_on_main_thread(move || {
                    if pending.signal.active.load(Ordering::Acquire) { crate::ui_diagnostics_windows::power(suspended, utc); }
                }).is_err() {
                    DROPPED.fetch_add(1, Ordering::Relaxed);
                    publish_failure(&context.diagnostics, PowerOperation::Dispatch, None);
                }
            }
        }
        0
    }
    fn capability(diagnostics: &Diagnostics, capability: Capability) {
        diagnostics.submit(Event { sequence: 0, utc: chrono::Utc::now(), window: WindowLabel::Main,
            generation: Generation::default(), kind: EventKind::PowerCapability { capability } });
    }
    pub struct PowerRegistration { lease: PowerLease, id: usize, context: Arc<Context> }
    impl PowerRegistration {
        pub fn register(app: AppHandle, diagnostics: Diagnostics) -> Result<Self, u32> {
            let context = Arc::new(Context { app, diagnostics, signal: PowerSignal::default() });
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            {
                let mut slot = CONTEXT.lock().map_err(|_| {
                    capability(&context.diagnostics, Capability::RegistrationFailed { code: Some(6) }); 6u32
                })?;
                if slot.is_some() { capability(&context.diagnostics, Capability::RegistrationFailed { code: Some(183) }); return Err(183); }
                *slot = Some((id, context.clone()));
            }
            let parameters = DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS { Callback: Some(callback), Context: id as *mut core::ffi::c_void };
            let mut handle = std::ptr::null_mut();
            let code = unsafe { PowerRegisterSuspendResumeNotification(DEVICE_NOTIFY_CALLBACK,
                HANDLE((&parameters as *const DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS).cast_mut().cast()), &mut handle) }.0;
            if code != 0 {
                context.signal.deactivate();
                if let Ok(mut slot) = CONTEXT.lock() { if slot.as_ref().is_some_and(|(current, _)| *current == id) { *slot = None; } }
                capability(&context.diagnostics, Capability::RegistrationFailed { code: Some(code as i32) }); return Err(code);
            }
            capability(&context.diagnostics, Capability::Supported);
            Ok(Self { lease: PowerLease::new(handle as isize), id, context })
        }
        pub fn close(&mut self) -> Result<(), u32> {
            self.context.signal.deactivate();
            self.lease.close(|token| unsafe { PowerUnregisterSuspendResumeNotification(HPOWERNOTIFY(token)) }.0)
                .inspect_err(|code| publish_failure(&self.context.diagnostics, PowerOperation::Unregistration, Some(*code as i32)))?;
            if let Ok(mut slot) = CONTEXT.lock() { if slot.as_ref().is_some_and(|(id, _)| *id == self.id) { *slot = None; } }
            Ok(())
        }
    }
    impl Drop for PowerRegistration {
        fn drop(&mut self) { let _ = self.close(); }
    }

}
#[cfg(windows)] pub use native::{PowerRegistration, dropped_callbacks};
