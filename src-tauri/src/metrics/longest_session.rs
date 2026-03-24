//! LongestSessionMetric — longest continuous computer session (minutes).

use super::{Metric, MetricLevel, MetricResult, threshold_level_lower_better};
use crate::config::AppConfig;
use crate::session_types::SessionState;

pub struct LongestSessionMetric;

impl Metric for LongestSessionMetric {
    fn id(&self) -> &str { "longest_session" }
    fn label(&self) -> &str { "\u{1f441} Screen" }

    fn compute(&self, state: &SessionState, config: &AppConfig) -> MetricResult {
        let longest_secs = state.longest_computer_session_secs
            .max(state.continuous_computer_secs); // include current session
        let longest_mins = longest_secs as f64 / 60.0;

        if state.first_reading_at.is_none() {
            return MetricResult {
                value: 0.0,
                display: "\u{2014}".to_string(),
                level: MetricLevel::Green,
                is_personal_best: false,
            };
        }

        let display = if longest_mins >= 60.0 {
            let h = (longest_mins / 60.0).floor() as u32;
            let m = (longest_mins % 60.0).round() as u32;
            format!("{}h{}m", h, m)
        } else {
            format!("{}m", longest_mins.round() as u32)
        };

        let level = threshold_level_lower_better(
            longest_mins,
            config.kpi_session_green_mins as f64,
            config.kpi_session_yellow_mins as f64,
        );

        MetricResult {
            value: longest_secs as f64,
            display,
            level,
            is_personal_best: false,
        }
    }
}
