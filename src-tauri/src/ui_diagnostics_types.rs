//! Bounded, content-free observations; neither health verdicts nor engine inputs.
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::VecDeque;

pub const QUEUE_CAPACITY: usize = 128;
pub const RECENT_CAPACITY: usize = 64;
pub const REPORT_LIMIT: usize = 256 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowLabel { Main, Analyst }
impl WindowLabel {
    pub fn index(self) -> usize { match self { Self::Main => 0, Self::Analyst => 1 } }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Generation { pub window: u64, pub navigation: u64 }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "state")]
pub enum Capability { NotRequested, Supported, Unsupported, RegistrationFailed { code: Option<i32> } }

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "state", content = "observation")]
pub enum Collection {
    Received(PageObservation), Timeout, NotRequested, NotPresent,
    Unsupported, Interrupted, CollectionFailed { code: Option<i32> },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BootStage { BeforeApp, Mounted, Fallback, Unknown }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewCategory { Widget, Settings, Analyst, Other }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentReady { Loading, Interactive, Complete, Unknown }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryOutcome { NotRequested, Succeeded, Failed }

/// No page text, HTML, input, storage or query data crosses this boundary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PageObservation {
    pub ready: DocumentReady,
    pub visible: bool,
    pub diagnostics_available: bool,
    pub boot: BootStage,
    pub view: ViewCategory,
    pub root_present: bool,
    pub root_children: u32,
    pub width_px: u32,
    pub height_px: u32,
    pub display_none: bool,
    pub visibility_hidden: bool,
    pub zero_opacity: bool,
    pub query: QueryOutcome,
    /// Page clock, never compared with native UTC or checkpoint time.
    pub last_query_success_ms: Option<u64>,
    pub captured_error: bool,
}
impl PageObservation {
    pub fn valid(&self) -> bool {
        self.root_children <= 100_000 && self.width_px <= 100_000 && self.height_px <= 100_000
            && self.last_query_success_ms.is_none_or(|n| n <= 8_640_000_000_000_000)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "event")]
pub enum EventKind {
    Created, Destroyed, NavigationStarted, NavigationCompleted { succeeded: bool },
    Show { succeeded: bool }, Focus { succeeded: bool }, Visibility { visible: bool },
    ProcessCapability { capability: Capability }, BrowserCapability { capability: Capability },
    NativeFailure { kind: u32, code: Option<i32> },
    ProbeStarted { id: u64 }, ProbeFinished { id: u64, result: Collection },
    Suspend, Resume,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Event {
    pub sequence: u64,
    pub utc: DateTime<Utc>,
    pub window: WindowLabel,
    pub generation: Generation,
    pub kind: EventKind,
}
#[derive(Clone, Debug, Serialize)]
pub struct EventGroup {
    pub first: Event,
    pub last_sequence: u64,
    pub last_utc: DateTime<Utc>,
    pub occurrences: u64,
    pub applied: bool,
    pub payload_rejected: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IoFailure { PermissionDenied, NotFound, AlreadyExists, InvalidData, Other }
impl From<std::io::ErrorKind> for IoFailure {
    fn from(kind: std::io::ErrorKind) -> Self {
        use std::io::ErrorKind::*;
        match kind { PermissionDenied => Self::PermissionDenied, NotFound => Self::NotFound,
            AlreadyExists => Self::AlreadyExists, InvalidData => Self::InvalidData, _ => Self::Other }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "state")]
pub enum WriteState { NotAttempted, Written, Failed { kind: IoFailure } }

#[derive(Clone, Debug, Serialize)]
pub struct WindowSnapshot {
    pub label: WindowLabel,
    pub generation: Generation,
    pub present: Option<bool>,
    pub visible: Option<bool>,
    pub process_events: Capability,
    pub browser_events: Capability,
    pub active_probe: Option<u64>,
    pub collection: Collection,
}
impl WindowSnapshot {
    pub fn unobserved(label: WindowLabel) -> Self {
        Self { label, generation: Generation::default(), present: None, visible: None,
            process_events: Capability::NotRequested, browser_events: Capability::NotRequested,
            active_probe: None, collection: Collection::NotRequested }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Counters {
    pub dropped: u64, pub disconnected: u64, pub coalesced: u64,
    pub stale: u64, pub invalid: u64, pub write_failures: u64, pub report_failures: u64,
}
#[derive(Clone, Debug, Serialize)]
pub struct DiagnosticState {
    pub format_version: u8,
    pub identity: crate::app_lifecycle::RunId,
    pub windows: [WindowSnapshot; 2],
    pub recent: VecDeque<EventGroup>,
    pub counters: Counters,
    pub writer: WriteState,
    pub report_writer: WriteState,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case", tag = "state")]
pub enum CheckpointObservation {
    Available { last_success_utc: Option<String>, last_save_failed: bool },
    Unavailable, Contended,
}
#[derive(Clone, Debug, Serialize)]
pub struct Report {
    pub collected_utc: DateTime<Utc>,
    pub facts: DiagnosticState,
    pub checkpoint: CheckpointObservation,
    pub webview_runtime_version: Option<String>,
    /// Empty unless a later adapter has explicit, source-qualified hypotheses.
    pub hypotheses: Vec<String>,
}
