//! Source-agnostic health DTOs (E021, ADR 020).
//!
//! One shape for every health source — the Google Fit pull, the LAN push
//! inlet a phone writes to, and whatever comes after Fit's shutdown. The
//! frontend reads [`HealthView`] and never learns which source produced it
//! beyond the `source_id` label.

use serde::Serialize;

/// Failure classes a health source can report.
///
/// Deliberately an alias of the Google Fit [`ErrorKind`](crate::google_fit_models::ErrorKind)
/// rather than a second enum: the two variants ("user must re-authorize"
/// vs "retry later") are the only distinction any source has ever needed,
/// and duplicating them would let the two drift.
pub type HealthErrorKind = crate::google_fit_models::ErrorKind;

/// One reading of today's health metrics from a single source.
///
/// `steps_today` is the only required metric; a source that cannot supply
/// heart rate or HRV reports `None` rather than a zero, so "no sensor" and
/// "measured zero" stay distinguishable.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export_to = "../../src/generated/"))]
pub struct HealthSnapshot {
    pub steps_today: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub heart_rate_bpm: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub hrv_rmssd_ms: Option<f32>,
    /// Stable identifier of the producing source, e.g. `google_fit`.
    pub source_id: String,
    /// Wall-clock unix-milliseconds at which this reading was produced.
    pub fetched_at_ms: i64,
}

/// What both health IPC commands and the remote snapshot carry.
///
/// Semantics, unchanged from the Google-Fit-only predecessor:
/// - `configured = false` → no source is set up; show the setup hint
/// - `configured = true, snapshot = None, error_kind = None` → loading
/// - `snapshot = Some(_), error_kind = None` → fresh data
/// - `error_kind = Some(AuthRevoked)` → show the reconnect CTA
/// - `error_kind = Some(Transient)` → keep the snapshot, flag the failure
#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export_to = "../../src/generated/"))]
pub struct HealthView {
    pub configured: bool,
    pub snapshot: Option<HealthSnapshot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub error_kind: Option<HealthErrorKind>,
    /// Human-readable error text — sanitized, never carries a secret.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub error_message: Option<String>,
}

impl HealthView {
    /// The view a source reports when it holds no credentials at all.
    pub fn unconfigured() -> Self {
        Self {
            configured: false,
            snapshot: None,
            error_kind: None,
            error_message: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap() -> HealthSnapshot {
        HealthSnapshot {
            steps_today: 4321,
            heart_rate_bpm: Some(72),
            hrv_rmssd_ms: None,
            source_id: "google_fit".into(),
            fetched_at_ms: 1_700_000_000_000,
        }
    }

    #[test]
    fn view_omits_none_fields_in_json() {
        let v = HealthView {
            configured: true,
            snapshot: Some(snap()),
            error_kind: None,
            error_message: None,
        };
        let s = serde_json::to_string(&v).expect("serializes");
        assert!(!s.contains("error_kind"), "got: {s}");
        assert!(!s.contains("hrv_rmssd_ms"), "got: {s}");
        assert!(s.contains("\"heart_rate_bpm\":72"), "got: {s}");
    }

    #[test]
    fn unconfigured_view_has_no_snapshot_and_no_error() {
        let v = HealthView::unconfigured();
        assert!(!v.configured);
        assert!(v.snapshot.is_none());
        assert!(v.error_kind.is_none());
    }

    #[test]
    fn error_kind_serializes_as_snake_case() {
        let v = HealthView {
            configured: true,
            snapshot: None,
            error_kind: Some(HealthErrorKind::AuthRevoked),
            error_message: Some("google fit: refresh token revoked".into()),
        };
        let s = serde_json::to_string(&v).expect("serializes");
        assert!(s.contains("\"error_kind\":\"auth_revoked\""), "got: {s}");
    }
}
