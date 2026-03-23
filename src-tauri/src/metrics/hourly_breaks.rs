//! HourlyBreakCoverageMetric — fraction of work hours with a 5+ min screen break.

use super::{Metric, MetricLevel, MetricResult};
use crate::config::AppConfig;
use crate::session_types::SessionState;

pub struct HourlyBreakCoverageMetric;

impl Metric for HourlyBreakCoverageMetric {
    fn id(&self) -> &str { "hourly_breaks" }
    fn label(&self) -> &str { "Breaks" }

    fn compute(&self, state: &SessionState, config: &AppConfig) -> MetricResult {
        // This metric needs HourlyBreakTracker data which is not yet in SessionState.
        // For now, compute a placeholder based on position_changes.
        // The real implementation will be wired in T07 when HourlyBreakTracker is
        // integrated into the IPC layer.
        //
        // Temporary: use position_changes as a rough proxy for breaks.
        // Each position change implies some movement. This will be replaced.

        let hours_active = if let Some(first) = state.first_reading_at {
            let elapsed_secs = (chrono::Utc::now() - first).num_seconds().max(0) as f64;
            (elapsed_secs / 3600.0).floor() as u8
        } else {
            0
        };

        if hours_active == 0 {
            return MetricResult {
                value: 0.0,
                display: "\u{2014}".to_string(),
                level: MetricLevel::Green,
                is_personal_best: false,
            };
        }

        // Placeholder: assume each position_change counts as an hourly break (capped).
        // Real implementation uses HourlyBreakTracker (will be wired in T07).
        let breaks = state.position_changes.min(hours_active as u32) as u8;
        let missed = hours_active.saturating_sub(breaks);

        let level = if missed == 0 { MetricLevel::Green }
            else if missed <= config.kpi_break_yellow_missed { MetricLevel::Yellow }
            else { MetricLevel::Red };

        MetricResult {
            value: breaks as f64 / hours_active as f64,
            display: format!("{}/{}", breaks, hours_active),
            level,
            is_personal_best: false,
        }
    }
}
