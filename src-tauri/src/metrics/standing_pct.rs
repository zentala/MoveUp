//! StandingPercentMetric — percentage of "at desk" time spent standing.

use super::{Metric, MetricLevel, MetricResult, threshold_level_higher_better};
use crate::config::AppConfig;
use crate::session_types::SessionState;

pub struct StandingPercentMetric;

impl Metric for StandingPercentMetric {
    fn id(&self) -> &str { "standing_pct" }
    fn label(&self) -> &str { "Standing" }

    fn compute(&self, state: &SessionState, config: &AppConfig) -> MetricResult {
        let sitting = state.sitting_seconds as f64;
        let standing = state.standing_seconds as f64;
        let total = sitting + standing;

        // Too early or no data
        let early_threshold_secs = config.kpi_early_data_threshold_mins as f64 * 60.0;
        if total < early_threshold_secs || total == 0.0 {
            return MetricResult {
                value: 0.0,
                display: "\u{2014}".to_string(),
                level: MetricLevel::Green,
                is_personal_best: false,
            };
        }

        let pct = (standing / total) * 100.0;
        let level = threshold_level_higher_better(
            pct,
            config.kpi_standing_green_pct as f64,
            config.kpi_standing_yellow_pct as f64,
        );

        MetricResult {
            value: pct,
            display: format!("{:.0}%", pct),
            level,
            is_personal_best: false,
        }
    }
}
