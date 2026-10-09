//! Strict readiness validation for new supervisors; the legacy probe is unchanged.
use crate::runtime_readiness::{ReadinessState, ReadinessStatus, APP_ID};

pub struct ExpectedReadiness<'a> {
    pub version: &'a str,
    pub pid: Option<u32>,
    pub run_id: Option<&'a str>,
    pub supervisor_run_id: Option<&'a str>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ReadinessError { InvalidSchema, WrongApp, WrongVersion, WrongRun, NotReady }

pub fn validate_readiness(body: &str, expected: &ExpectedReadiness<'_>)
    -> Result<ReadinessStatus, ReadinessError> {
    if body.len() > 4096 { return Err(ReadinessError::InvalidSchema); }
    let status: ReadinessStatus = serde_json::from_str(body)
        .map_err(|_| ReadinessError::InvalidSchema)?;
    if status.app_id != APP_ID { return Err(ReadinessError::WrongApp); }
    if status.version != expected.version { return Err(ReadinessError::WrongVersion); }
    if status.pid == 0 || uuid::Uuid::parse_str(&status.run_id).is_err()
        || !matches!(status.build_mode.as_str(), "debug" | "release")
        || status.supervisor_run_id.as_ref().is_some_and(|id| id.is_empty() || id.len() > 128) {
        return Err(ReadinessError::InvalidSchema);
    }
    if expected.pid.is_some_and(|pid| pid != status.pid)
        || expected.run_id.is_some_and(|run| run != status.run_id)
        || expected.supervisor_run_id.is_some_and(|run|
            status.supervisor_run_id.as_deref() != Some(run)) {
        return Err(ReadinessError::WrongRun);
    }
    let mut computed = status.clone();
    computed.compute_state();
    if !status.ready || !computed.ready
        || status.state != computed.state || status.reason != computed.reason
        || !matches!(status.state, ReadinessState::Ready | ReadinessState::Degraded) {
        return Err(ReadinessError::NotReady);
    }
    Ok(status)
}
