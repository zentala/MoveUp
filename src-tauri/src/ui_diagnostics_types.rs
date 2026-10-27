//! Bounded, content-free observations; neither health verdicts nor engine inputs.
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BootStage { BeforeApp, Mounted, Fallback, Unknown }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewCategory { Widget, Settings, Analyst, Other }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentReady { Loading, Interactive, Complete, Unknown }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryOutcome { NotObserved, Succeeded, Failed }
fn nullable<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<Option<T>, D::Error> {
    Option::<T>::deserialize(d)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryHint {
    pub outcome: QueryOutcome,
    #[serde(deserialize_with = "nullable")] pub last_result_ms: Option<u64>,
    #[serde(deserialize_with = "nullable")] pub last_success_ms: Option<u64>,
}
impl QueryHint {
    pub fn unobserved() -> Self { Self { outcome: QueryOutcome::NotObserved, last_result_ms: None, last_success_ms: None } }
    fn valid(&self) -> bool {
        [self.last_result_ms, self.last_success_ms].iter().all(|t| t.is_none_or(|n| n <= 8_640_000_000_000_000))
            && match self.outcome {
                QueryOutcome::NotObserved => self.last_result_ms.is_none() && self.last_success_ms.is_none(),
                QueryOutcome::Succeeded => self.last_result_ms.is_some() && self.last_success_ms.is_some(),
                QueryOutcome::Failed => self.last_result_ms.is_some(),
            }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Queries { pub dashboard: QueryHint, pub today_summary: QueryHint }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureSource { Render, Window, Rejection }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafeErrorKind { Error, TypeError, RangeError, ReferenceError, SyntaxError, URIError,
    EvalError, AggregateError, DOMException, NonError, UnknownError }
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SafeLocation {
    pub module: String,
    #[serde(deserialize_with = "nullable")] pub line: Option<u32>,
    #[serde(deserialize_with = "nullable")] pub column: Option<u32>,
}
impl SafeLocation {
    fn valid(&self) -> bool {
        let Some((base, extension)) = self.module.rsplit_once('.') else { return false };
        !base.is_empty() && base.len() <= 128 && self.module.len() <= 133
            && base.as_bytes()[0].is_ascii_alphanumeric()
            && base.bytes().all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
            && matches!(extension.to_ascii_lowercase().as_str(), "js"|"jsx"|"ts"|"tsx"|"cjs"|"cjsx"|"cts"|"ctsx"|"mjs"|"mjsx"|"mts"|"mtsx")
            && match (self.line, self.column) { (None, None) => true,
                (Some(l), Some(c)) => l > 0 && c > 0 && l <= 1_000_000_000 && c <= 1_000_000_000, _ => false }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SafeError {
    pub source: FailureSource, pub kind: SafeErrorKind,
    #[serde(deserialize_with = "nullable")] pub location: Option<SafeLocation>,
}
/// Exact version1 shape; native collection never retains raw page data.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageObservation {
    pub schema_version: u8,
    pub ready: DocumentReady, pub visible: bool, pub diagnostics_available: bool,
    pub boot: BootStage, pub view: ViewCategory, pub root_present: bool,
    pub root_children: u32, pub width_px: u32, pub height_px: u32,
    pub display_none: bool, pub visibility_hidden: bool, pub zero_opacity: bool,
    pub queries: Queries,
    #[serde(deserialize_with = "nullable")] pub last_error: Option<SafeError>,
}
impl PageObservation {
    pub fn valid(&self) -> bool {
        self.schema_version == 1 && self.root_children <= 100_000 && self.width_px <= 100_000 && self.height_px <= 100_000
            && self.queries.dashboard.valid() && self.queries.today_summary.valid()
            && self.last_error.as_ref().is_none_or(|e| e.location.as_ref().is_none_or(SafeLocation::valid))
            && (self.diagnostics_available || (self.boot == BootStage::Unknown && self.view == ViewCategory::Other
                && self.queries.dashboard.outcome == QueryOutcome::NotObserved
                && self.queries.today_summary.outcome == QueryOutcome::NotObserved && self.last_error.is_none()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "event")]
pub enum EventKind {
    Created, Destroyed, NavigationStarted, NavigationCompleted { succeeded: bool },
    Show { succeeded: bool }, Focus { succeeded: bool }, Visibility { visible: bool },
    ProcessCapability { capability: Capability }, BrowserCapability { capability: Capability },
    NativeFailure { kind: u32, code: Option<i32> },
    ProcessFailed { kind: Option<i32>, reason: Option<i32>, exit_code: Option<i32> },
    BrowserExited { kind: Option<i32>, process_id: Option<u32> },
    PowerCapability { capability: Capability },
    PowerFailure { operation: PowerOperation, code: Option<i32> },
    AdapterFailure { operation: AdapterOperation, code: Option<i32> },
    ProbeStarted { id: u64 }, ProbeFinished { id: u64, result: Collection },
    ProbeDeadlineDispatchFailed { id: u64 },
    Suspend, Resume,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterOperation { Acquisition, NavigationRegistration, Cleanup, Script, Dispatch, RuntimeVersion }
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PowerOperation { Registration, Unregistration, Dispatch }
#[derive(Clone, Debug, Serialize)]
pub struct PowerFailure {
    pub operation: PowerOperation, pub code: Option<i32>, pub utc: DateTime<Utc>, pub sequence: u64,
}
#[derive(Clone, Debug, Serialize)]
pub struct PowerState {
    pub capability: Capability, pub last_failure: Option<PowerFailure>, pub failure_count: u64,
}
impl Default for PowerState {
    fn default() -> Self { Self { capability: Capability::NotRequested, last_failure: None, failure_count: 0 } }
}
#[derive(Clone, Debug, Serialize)]
pub struct DiagnosticState {
    pub format_version: u8,
    pub identity: crate::app_lifecycle::RunId,
    pub windows: [WindowSnapshot; 2],
    pub power: PowerState,
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
