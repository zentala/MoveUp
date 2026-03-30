//! HourlyBreakCoverageMetric — fraction of work hours with a 5+ min screen break.

use super::{Metric, MetricLevel, MetricResult};
use crate::ergonomic_profile::ErgonomicProfile;
use crate::session_types::SessionState;

pub struct HourlyBreakCoverageMetric;

impl Metric for HourlyBreakCoverageMetric {
    fn id(&self) -> &str { "hourly_breaks" }
    fn label(&self) -> &str { "\u{2615} Breaks" }

    fn compute(&self, state: &SessionState, ergo: &ErgonomicProfile) -> MetricResult {
        let hours_active = state.hourly_breaks_active;

        if hours_active == 0 {
            return MetricResult {
                value: 0.0,
                display: "\u{2014}".to_string(),
                level: MetricLevel::Green,
                is_personal_best: false,
            };
        }

        let breaks = state.hourly_breaks_covered;
        let missed = hours_active.saturating_sub(breaks);

        let level = if missed == 0 { MetricLevel::Green }
            else if missed <= ergo.kpi.break_yellow_missed { MetricLevel::Yellow }
            else { MetricLevel::Red };

        MetricResult {
            value: breaks as f64 / hours_active as f64,
            display: format!("{}/{}", breaks, hours_active),
            level,
            is_personal_best: false,
        }
    }
}
