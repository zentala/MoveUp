//! metrics/mod.rs — MetricEngine platform: trait, engine, result types.

pub mod standing_pct;
pub mod position_rate;
pub mod hourly_breaks;
pub mod longest_session;
#[cfg(test)]
mod tests;

use crate::ergonomic_profile::ErgonomicProfile;
use crate::session::SessionStateDto;
use crate::session_types::SessionState;
use serde::Serialize;

/// Severity level for a KPI metric.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MetricLevel {
    Green,
    Yellow,
    Red,
}

/// Result of computing a single metric.
#[derive(Debug, Clone, Serialize)]
pub struct MetricResult {
    pub value: f64,
    pub display: String,
    pub level: MetricLevel,
    pub is_personal_best: bool,
}

/// A named metric snapshot for IPC transport.
#[derive(Debug, Clone, Serialize)]
pub struct MetricSnapshot {
    pub id: String,
    pub label: String,
    pub result: MetricResult,
}

/// Combined dashboard response: session state + computed KPI metrics.
#[derive(Debug, Clone, Serialize)]
pub struct DashboardState {
    pub session: SessionStateDto,
    pub metrics: Vec<MetricSnapshot>,
}

/// Each metric implements this trait.
pub trait Metric: Send + Sync {
    fn id(&self) -> &str;
    fn label(&self) -> &str;
    fn compute(&self, state: &SessionState, ergo: &ErgonomicProfile) -> MetricResult;
}

/// Engine that holds all registered metrics and computes them all.
pub struct MetricEngine {
    metrics: Vec<Box<dyn Metric>>,
}

impl MetricEngine {
    pub fn new() -> Self {
        Self { metrics: Vec::new() }
    }

    /// Creates engine with all 4 default metrics registered.
    pub fn with_defaults() -> Self {
        let mut engine = Self::new();
        engine.register(Box::new(standing_pct::StandingPercentMetric));
        engine.register(Box::new(position_rate::PositionChangeRateMetric));
        engine.register(Box::new(hourly_breaks::HourlyBreakCoverageMetric));
        engine.register(Box::new(longest_session::LongestSessionMetric));
        engine
    }

    pub fn register(&mut self, metric: Box<dyn Metric>) {
        self.metrics.push(metric);
    }

    pub fn compute_all(
        &self,
        state: &SessionState,
        ergo: &ErgonomicProfile,
    ) -> Vec<MetricSnapshot> {
        self.metrics.iter().map(|m| MetricSnapshot {
            id: m.id().to_string(),
            label: m.label().to_string(),
            result: m.compute(state, ergo),
        }).collect()
    }
}

/// Utility: determine level from value against green/yellow thresholds.
/// For "higher is better" metrics (standing_pct, position_rate).
pub fn threshold_level_higher_better(value: f64, green: f64, yellow: f64) -> MetricLevel {
    if value >= green { MetricLevel::Green }
    else if value >= yellow { MetricLevel::Yellow }
    else { MetricLevel::Red }
}

/// Utility: determine level from value against green/yellow thresholds.
/// For "lower is better" metrics (longest_session).
pub fn threshold_level_lower_better(value: f64, green: f64, yellow: f64) -> MetricLevel {
    if value <= green { MetricLevel::Green }
    else if value <= yellow { MetricLevel::Yellow }
    else { MetricLevel::Red }
}
