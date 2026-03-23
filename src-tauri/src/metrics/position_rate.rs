//! PositionChangeRateMetric — position changes per hour worked.

use super::{Metric, MetricLevel, MetricResult, threshold_level_higher_better};
use crate::config::AppConfig;
use crate::session_types::SessionState;

pub struct PositionChangeRateMetric;

impl Metric for PositionChangeRateMetric {
    fn id(&self) -> &str { "position_rate" }
    fn label(&self) -> &str { "Changes" }

    fn compute(&self, state: &SessionState, config: &AppConfig) -> MetricResult {
        // Need first_reading_at to compute hours_worked
        let hours_worked = if let Some(first) = state.first_reading_at {
            let elapsed_secs = (chrono::Utc::now() - first).num_seconds().max(0) as f64;
            elapsed_secs / 3600.0
        } else {
            0.0
        };

        let early_threshold_hours = config.kpi_early_data_threshold_mins as f64 / 60.0;
        if hours_worked < early_threshold_hours || hours_worked == 0.0 {
            return MetricResult {
                value: 0.0,
                display: "\u{2014}".to_string(),
                level: MetricLevel::Green,
                is_personal_best: false,
            };
        }

        let rate = state.position_changes as f64 / hours_worked;
        let level = threshold_level_higher_better(
            rate,
            config.kpi_changes_green as f64,
            config.kpi_changes_yellow as f64,
        );

        MetricResult {
            value: rate,
            display: format!("{:.1}/h", rate),
            level,
            is_personal_best: false,
        }
    }
}
