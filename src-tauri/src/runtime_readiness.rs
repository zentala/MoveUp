//! runtime_readiness.rs — Explicit readiness status for supervision.
//!
//! Provides typed validation of readiness state to be exposed through
//! `/display/api` (additive to existing schema). Answers whether the app
//! is initializing, ready, degraded, or failed, and why.
//!
//! Sensor absence → degraded (not failed). No outbound health calls.

use serde::{Deserialize, Serialize};

/// Readiness verdict for supervision and health probes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReadinessState {
    /// Still initializing, not yet ready for use.
    Initializing,
    /// All critical subsystems ready and healthy.
    Ready,
    /// Operating but degraded (e.g., sensor missing, non-critical service down).
    Degraded,
    /// Unable to operate (e.g., database corruption, crash loop).
    Failed,
}

impl std::fmt::Display for ReadinessState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Initializing => write!(f, "initializing"),
            Self::Ready => write!(f, "ready"),
            Self::Degraded => write!(f, "degraded"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// Reason for a degraded or failed readiness state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReadinessDegradationReason {
    /// No serial device detected; user interaction may be missing.
    SensorMissing,
    /// Database not yet opened or still initializing.
    DatabaseNotReady,
    /// Remote/health subsystems not yet initialized.
    RemoteNotReady,
    /// Tray initialization incomplete.
    TrayNotReady,
    /// Generic failure (see `failure_details` for more).
    OtherFailure,
}

impl std::fmt::Display for ReadinessDegradationReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SensorMissing => write!(f, "sensor_missing"),
            Self::DatabaseNotReady => write!(f, "database_not_ready"),
            Self::RemoteNotReady => write!(f, "remote_not_ready"),
            Self::TrayNotReady => write!(f, "tray_not_ready"),
            Self::OtherFailure => write!(f, "other_failure"),
        }
    }
}

/// Full readiness status reported by `/display/api`.
///
/// Includes app identity, version, process mode, and initialization stages
/// to allow supervisors to make informed restart decisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessStatus {
    /// Application identifier (e.g., "io.zntl.desk").
    pub app_id: String,
    /// Application version (e.g., "0.6.0").
    pub version: String,
    /// Build mode: "debug" or "release".
    pub build_mode: String,
    /// Current readiness state.
    pub state: ReadinessState,
    /// Reason for degradation (only set if state is Degraded or Failed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<ReadinessDegradationReason>,
    /// Additional details (e.g., error message from the failure).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_details: Option<String>,
    /// Tray window initialized and visible.
    pub tray_initialized: bool,
    /// Database connection open.
    pub database_initialized: bool,
    /// Serial device scanning completed (not necessarily found).
    pub serial_scanned: bool,
    /// Remote display server listening.
    pub remote_listening: bool,
}

impl ReadinessStatus {
    /// Create a new readiness status with the given app info.
    pub fn new(app_id: &str, version: &str, build_mode: &str) -> Self {
        Self {
            app_id: app_id.to_string(),
            version: version.to_string(),
            build_mode: build_mode.to_string(),
            state: ReadinessState::Initializing,
            reason: None,
            failure_details: None,
            tray_initialized: false,
            database_initialized: false,
            serial_scanned: false,
            remote_listening: false,
        }
    }

    /// Update readiness based on initialization flags.
    ///
    /// Logic:
    /// - If any required subsystem is not ready, state is Initializing.
    /// - If sensor is missing but others are ready, state is Degraded.
    /// - Otherwise, state is Ready.
    pub fn compute_state(&mut self, sensor_present: bool) {
        if !self.tray_initialized || !self.database_initialized || !self.remote_listening {
            self.state = ReadinessState::Initializing;
            self.reason = if !self.tray_initialized {
                Some(ReadinessDegradationReason::TrayNotReady)
            } else if !self.database_initialized {
                Some(ReadinessDegradationReason::DatabaseNotReady)
            } else {
                Some(ReadinessDegradationReason::RemoteNotReady)
            };
        } else if !sensor_present {
            self.state = ReadinessState::Degraded;
            self.reason = Some(ReadinessDegradationReason::SensorMissing);
        } else {
            self.state = ReadinessState::Ready;
            self.reason = None;
        }
    }

    /// Mark the app as failed with an optional reason.
    pub fn mark_failed(&mut self, details: Option<&str>) {
        self.state = ReadinessState::Failed;
        self.reason = Some(ReadinessDegradationReason::OtherFailure);
        self.failure_details = details.map(|s| s.to_string());
    }
}

#[cfg(test)]
mod runtime_readiness_tests {
    use super::*;

    #[test]
    fn test_readiness_status_creation() {
        let status = ReadinessStatus::new("io.zntl.desk", "0.6.0", "release");
        assert_eq!(status.app_id, "io.zntl.desk");
        assert_eq!(status.version, "0.6.0");
        assert_eq!(status.build_mode, "release");
        assert_eq!(status.state, ReadinessState::Initializing);
    }

    #[test]
    fn test_compute_state_initializing() {
        let mut status = ReadinessStatus::new("io.zntl.desk", "0.6.0", "release");
        status.tray_initialized = false;
        status.database_initialized = true;
        status.remote_listening = true;

        status.compute_state(true);
        assert_eq!(status.state, ReadinessState::Initializing);
        assert_eq!(status.reason, Some(ReadinessDegradationReason::TrayNotReady));
    }

    #[test]
    fn test_compute_state_ready() {
        let mut status = ReadinessStatus::new("io.zntl.desk", "0.6.0", "release");
        status.tray_initialized = true;
        status.database_initialized = true;
        status.remote_listening = true;
        status.serial_scanned = true;

        status.compute_state(true);
        assert_eq!(status.state, ReadinessState::Ready);
        assert_eq!(status.reason, None);
    }

    #[test]
    fn test_compute_state_degraded_no_sensor() {
        let mut status = ReadinessStatus::new("io.zntl.desk", "0.6.0", "release");
        status.tray_initialized = true;
        status.database_initialized = true;
        status.remote_listening = true;
        status.serial_scanned = true;

        status.compute_state(false); // No sensor
        assert_eq!(status.state, ReadinessState::Degraded);
        assert_eq!(status.reason, Some(ReadinessDegradationReason::SensorMissing));
    }

    #[test]
    fn test_mark_failed() {
        let mut status = ReadinessStatus::new("io.zntl.desk", "0.6.0", "release");
        status.mark_failed(Some("database corrupted"));

        assert_eq!(status.state, ReadinessState::Failed);
        assert_eq!(status.reason, Some(ReadinessDegradationReason::OtherFailure));
        assert_eq!(
            status.failure_details,
            Some("database corrupted".to_string())
        );
    }

    #[test]
    fn test_readiness_state_display() {
        assert_eq!(ReadinessState::Initializing.to_string(), "initializing");
        assert_eq!(ReadinessState::Ready.to_string(), "ready");
        assert_eq!(ReadinessState::Degraded.to_string(), "degraded");
        assert_eq!(ReadinessState::Failed.to_string(), "failed");
    }

    #[test]
    fn test_degradation_reason_display() {
        assert_eq!(
            ReadinessDegradationReason::SensorMissing.to_string(),
            "sensor_missing"
        );
        assert_eq!(
            ReadinessDegradationReason::DatabaseNotReady.to_string(),
            "database_not_ready"
        );
    }

    #[test]
    fn test_serialization() {
        let mut status = ReadinessStatus::new("io.zntl.desk", "0.6.0", "release");
        status.tray_initialized = true;
        status.database_initialized = true;
        status.remote_listening = true;
        status.compute_state(true);

        let json = serde_json::to_string(&status).expect("should serialize");
        assert!(json.contains("io.zntl.desk"));
        assert!(json.contains("Ready"));
        assert!(json.contains("0.6.0"));

        let deserialized: ReadinessStatus = serde_json::from_str(&json)
            .expect("should deserialize");
        assert_eq!(deserialized.state, ReadinessState::Ready);
    }

    #[test]
    fn test_degradation_reason_skipped_in_ready() {
        let mut status = ReadinessStatus::new("io.zntl.desk", "0.6.0", "release");
        status.tray_initialized = true;
        status.database_initialized = true;
        status.remote_listening = true;
        status.compute_state(true);

        let json = serde_json::to_string(&status).expect("should serialize");
        assert!(!json.contains("reason"));
    }
}
