//! Application-owned diagnostics. Only the worker calls disk-backed sinks.
use crate::ui_diagnostics_types::*;
use std::{io, path::PathBuf, sync::{Arc, atomic::{AtomicU64, Ordering}, mpsc::{self, Receiver, SyncSender, TrySendError}}};

impl DiagnosticState {
    pub fn new(identity: crate::app_lifecycle::RunId) -> Self {
        Self { format_version: 1, identity,
            windows: [WindowSnapshot::unobserved(WindowLabel::Main), WindowSnapshot::unobserved(WindowLabel::Analyst)],
            recent: Default::default(), counters: Default::default(), writer: WriteState::NotAttempted,
            report_writer: WriteState::NotAttempted, power: PowerState::default() }
    }
    pub fn observe(&mut self, mut event: Event) {
        // Validate before either generation checks or retained-event construction.
        // Even a stale result must not carry rejected values into a sink/report.
        let payload_rejected = matches!(&event.kind,
            EventKind::ProbeFinished { result: Collection::Received(page), .. } if !page.valid());
        if payload_rejected {
            if let EventKind::ProbeFinished { result, .. } = &mut event.kind {
                *result = Collection::CollectionFailed { code: None };
            }
            self.counters.invalid = self.counters.invalid.saturating_add(1);
        }
        let applied = self.apply(&event) && !payload_rejected;
        if let Some(last) = self.recent.iter_mut().rev().find(|last|
            last.first.window == event.window && last.first.generation == event.generation
                && last.first.kind == event.kind && last.applied == applied
                && last.payload_rejected == payload_rejected) {
                last.occurrences = last.occurrences.saturating_add(1);
                last.last_sequence = event.sequence;
                last.last_utc = event.utc;
                self.counters.coalesced = self.counters.coalesced.saturating_add(1);
                return;
        }
        if self.recent.len() == RECENT_CAPACITY { self.recent.pop_front(); }
        self.recent.push_back(EventGroup { last_sequence: event.sequence, last_utc: event.utc,
            first: event, occurrences: 1, applied, payload_rejected });
    }
    fn apply(&mut self, event: &Event) -> bool {
        // Process power evidence is independent of every window's generation/history.
        match event.kind {
            EventKind::PowerCapability { capability } => {
                self.power.capability = capability;
                if let Capability::RegistrationFailed { code } = capability {
                    self.power_failure(event, PowerOperation::Registration, code);
                }
                return true;
            }
            EventKind::PowerFailure { operation, code } => {
                if operation == PowerOperation::Unregistration { self.power.capability = Capability::RegistrationFailed { code }; }
                self.power_failure(event, operation, code);
                return true;
            }
            _ => {}
        }
        if matches!(event.kind, EventKind::Suspend | EventKind::Resume) {
            for window in &mut self.windows {
                if window.active_probe.take().is_some() { window.collection = Collection::Interrupted; }
            }
            return true;
        }
        let window = &mut self.windows[event.window.index()];
        let advances = matches!(event.kind, EventKind::Created | EventKind::NavigationStarted);
        if matches!(event.kind, EventKind::Created) && event.generation.window <= window.generation.window
            || matches!(event.kind, EventKind::NavigationStarted)
                && (event.generation.window != window.generation.window || window.present != Some(true)
                    || event.generation.navigation <= window.generation.navigation) {
            self.counters.stale = self.counters.stale.saturating_add(1); return false;
        }
        if event.generation < window.generation || (!advances && event.generation != window.generation) {
            self.counters.stale = self.counters.stale.saturating_add(1);
            return false;
        }
        if advances && event.generation > window.generation {
            let interrupted = window.active_probe.take().is_some();
            if matches!(event.kind, EventKind::Created) {
                *window = WindowSnapshot::unobserved(event.window);
                window.present = Some(true);
                window.collection = Collection::NotRequested;
            }
            window.generation = event.generation;
            window.collection = if interrupted { Collection::Interrupted } else { Collection::NotRequested };
        }
        match &event.kind {
            EventKind::Created => window.present = Some(true),
            EventKind::Destroyed => {
                window.present = Some(false); window.visible = None;
                window.active_probe = None; window.collection = Collection::NotPresent;
            }
            EventKind::Visibility { visible } => window.visible = Some(*visible),
            EventKind::ProcessCapability { capability } => window.process_events = *capability,
            EventKind::BrowserCapability { capability } => window.browser_events = *capability,
            EventKind::ProbeStarted { id } => {
                if window.present != Some(true) || window.active_probe.is_some() {
                    self.counters.invalid = self.counters.invalid.saturating_add(1); return false;
                }
                window.active_probe = Some(*id); window.collection = Collection::NotRequested;
            }
            EventKind::ProbeFinished { id, result } => {
                if window.active_probe != Some(*id) {
                    self.counters.stale = self.counters.stale.saturating_add(1); return false;
                }
                window.active_probe = None;
                window.collection = result.clone();
            }
            EventKind::ProbeDeadlineDispatchFailed { id } => {
                if window.active_probe != Some(*id) {
                    self.counters.stale = self.counters.stale.saturating_add(1); return false;
                }
                window.active_probe = None;
                window.collection = Collection::CollectionFailed { code: None };
            }
            _ => {}
        }
        true
    }
    fn power_failure(&mut self, event: &Event, operation: PowerOperation, code: Option<i32>) {
        self.power.failure_count = self.power.failure_count.saturating_add(1);
        self.power.last_failure = Some(PowerFailure { operation, code, utc: event.utc, sequence: event.sequence });
    }
}

enum Work {
    Event(Event),
    Snapshot(mpsc::Sender<DiagnosticState>),
    Report { directory: PathBuf, checkpoint: CheckpointObservation, runtime: Option<String>,
        reply: mpsc::Sender<Result<PathBuf, crate::ui_diagnostics_store::ReportError>> },
}
#[derive(Default)]
struct Loss { dropped: AtomicU64, disconnected: AtomicU64 }
#[derive(Clone)]
pub struct Diagnostics { sender: SyncSender<Work>, loss: Arc<Loss> }
#[derive(Debug, PartialEq, Eq)]
pub enum RequestFailure { InvalidMetadata, QueueUnavailable }

impl Diagnostics {
    /// Construction only; final application activation belongs to the tray adapter.
    pub fn start(identity: crate::app_lifecycle::RunId,
        mut sink: impl FnMut(&str) -> io::Result<()> + Send + 'static) -> io::Result<Self> {
        if identity.run_id.len() > 128 || identity.version.len() > 128 || identity.build_mode.len() > 32
            || identity.supervisor_run_id.as_ref().is_some_and(|s| s.len() > 128) {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        let (sender, receiver) = mpsc::sync_channel(QUEUE_CAPACITY);
        let loss = Arc::new(Loss::default());
        let worker_loss = loss.clone();
        std::thread::Builder::new().name("ui-diagnostics".into()).spawn(move || {
            let mut state = DiagnosticState::new(identity);
            let mut sequence = 0u64;
            while let Ok(work) = receiver.recv() {
                state.counters.dropped = worker_loss.dropped.load(Ordering::Relaxed);
                state.counters.disconnected = worker_loss.disconnected.load(Ordering::Relaxed);
                match work {
                    Work::Event(mut event) => {
                        // Order is the worker's receipt order, independent of source clocks.
                        sequence = sequence.saturating_add(1);
                        event.sequence = sequence;
                        let before = state.counters.coalesced;
                        state.observe(event);
                        // Repeated observations remain counted in memory; avoid repeated IO.
                        if state.counters.coalesced == before {
                            let line = serde_json::to_string(state.recent.back().unwrap());
                            let result = line.map_err(io::Error::other)
                                .and_then(|s| sink(&format!("UI_DIAGNOSTICS run={} {s}", state.identity.run_id)));
                            state.writer = match result {
                                Ok(()) => WriteState::Written,
                                Err(error) => {
                                    state.counters.write_failures = state.counters.write_failures.saturating_add(1);
                                    WriteState::Failed { kind: error.kind().into() }
                                }
                            };
                        }
                    }
                    Work::Snapshot(reply) => { let _ = reply.send(state.clone()); }
                    Work::Report { directory, checkpoint, runtime, reply } => {
                        let report = Report { collected_utc: chrono::Utc::now(), facts: state.clone(),
                            checkpoint, webview_runtime_version: runtime, hypotheses: Vec::new() };
                        let result = crate::ui_diagnostics_store::save_report(&directory, &report);
                        state.report_writer = match &result {
                            Ok(_) => WriteState::Written,
                            Err(error) => {
                                state.counters.report_failures = state.counters.report_failures.saturating_add(1);
                                WriteState::Failed { kind: error.kind }
                            }
                        };
                        let _ = reply.send(result);
                    }
                }
            }
        })?;
        Ok(Self { sender, loss })
    }
    pub fn with_logger(identity: crate::app_lifecycle::RunId,
        logger: Arc<crate::event_logger::EventLogger>) -> io::Result<Self> {
        Self::start(identity, move |line| logger.log_result(line))
    }
    fn send(&self, work: Work) -> bool {
        match self.sender.try_send(work) {
            Ok(()) => true,
            Err(TrySendError::Full(_)) => { self.loss.dropped.fetch_add(1, Ordering::Relaxed); false }
            Err(TrySendError::Disconnected(_)) => { self.loss.disconnected.fetch_add(1, Ordering::Relaxed); false }
        }
    }
    /// No disk IO, session/JS locks, sleeps or waits on the producer path.
    pub fn submit(&self, event: Event) -> bool { self.send(Work::Event(event)) }
    pub fn dropped(&self) -> u64 { self.loss.dropped.load(Ordering::Relaxed) }
    pub fn snapshot(&self) -> Option<Receiver<DiagnosticState>> {
        let (tx, rx) = mpsc::channel();
        self.send(Work::Snapshot(tx)).then_some(rx)
    }
    pub fn report(&self, directory: PathBuf, checkpoint: CheckpointObservation,
        runtime: Option<String>) -> Result<Receiver<Result<PathBuf, crate::ui_diagnostics_store::ReportError>>, RequestFailure> {
        if runtime.as_ref().is_some_and(|s| s.len() > 128 || !s.chars().all(|c| c.is_ascii_digit() || c == '.'))
            || matches!(&checkpoint, CheckpointObservation::Available { last_success_utc: Some(s), .. } if s.len() > 64) {
            return Err(RequestFailure::InvalidMetadata);
        }
        let (reply, rx) = mpsc::channel();
        if self.send(Work::Report { directory, checkpoint, runtime, reply }) { Ok(rx) }
        else { Err(RequestFailure::QueueUnavailable) }
    }
}

/// Read-only, nonblocking access; never includes raw persistence errors.
pub fn checkpoint(lifecycle: Option<&crate::app_lifecycle::AppLifecycle>) -> CheckpointObservation {
    let Some(lifecycle) = lifecycle else { return CheckpointObservation::Unavailable };
    match lifecycle.checkpoint_status.try_lock() {
        Ok(status) => CheckpointObservation::Available {
            last_success_utc: status.last_success_utc.as_ref().filter(|s| s.len() <= 64).cloned(),
            last_save_failed: status.last_error.is_some(),
        },
        Err(std::sync::TryLockError::WouldBlock) => CheckpointObservation::Contended,
        Err(std::sync::TryLockError::Poisoned(_)) => CheckpointObservation::Unavailable,
    }
}
