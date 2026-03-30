//! tests.rs — unit tests for MetricEngine and all 4 metrics.

#[cfg(test)]
mod tests {
    use crate::ergonomic_profile::ErgonomicProfile;
    use crate::metrics::*;
    use crate::session_types::*;
    use chrono::Utc;

    fn default_state() -> SessionState {
        SessionState {
            state: DeskState::Sitting,
            sitting_started: None,
            sitting_seconds: 0,
            standing_seconds: 0,
            break_started: None,
            break_seconds: 0,
            session_limit_secs: 2700,
            stand_limit_secs: 0,
            desk_height_cm: 0.0,
            last_position_change_at: None,
            position_changes: 0,
            last_break_secs: 0,
            last_sitting_secs: 0,
            last_break_credit: BreakCredit::None,
            daily_score: 0.0,
            standing_session_secs: 0,
            standing_session_started: None,
            lap_bonus_awarded_for_lap: 0,
            current_session_secs: 0,
            continuous_computer_secs: 0,
            longest_computer_session_secs: 0,
            away_bout_secs: 0,
            first_reading_at: None,
            last_tick_ts: None,
            last_accumulate_ts: None,
            standing_bout_started: None,
            hourly_breaks_covered: 0,
            hourly_breaks_active: 0,
            sitting_seconds_total: 0,
            break_min_secs: 60,
            break_credit_multiplier: 2.0,
        }
    }

    fn ergo() -> ErgonomicProfile {
        ErgonomicProfile::default()
    }

    // ─── Engine tests ────────────────────────────────────────────────────

    #[test]
    fn engine_compute_all_returns_4() {
        let engine = MetricEngine::with_defaults();
        let state = default_state();
        let e = ergo();
        let results = engine.compute_all(&state, &e);
        assert_eq!(results.len(), 4);
    }

    // ─── StandingPercentMetric ───────────────────────────────────────────

    #[test]
    fn standing_pct_no_data() {
        let metric = standing_pct::StandingPercentMetric;
        let state = default_state();
        let e = ergo();
        let result = metric.compute(&state, &e);
        assert_eq!(result.display, "\u{2014}");
    }

    #[test]
    fn standing_pct_normal() {
        let metric = standing_pct::StandingPercentMetric;
        let mut state = default_state();
        // Total must exceed early_data_threshold (30 min = 1800s)
        state.sitting_seconds_total = 1500;
        state.standing_seconds = 500;
        let e = ergo();
        let result = metric.compute(&state, &e);
        assert!((result.value - 25.0).abs() < 0.1);
        assert_eq!(result.display, "25%");
    }

    #[test]
    fn standing_pct_green() {
        let metric = standing_pct::StandingPercentMetric;
        let mut state = default_state();
        state.sitting_seconds_total = 1700;
        state.standing_seconds = 300; // 300/(1700+300) = 15%
        let e = ergo();
        let result = metric.compute(&state, &e);
        assert_eq!(result.level, MetricLevel::Green);
    }

    #[test]
    fn standing_pct_red() {
        let metric = standing_pct::StandingPercentMetric;
        let mut state = default_state();
        state.sitting_seconds_total = 1900;
        state.standing_seconds = 100; // 100/(1900+100) = 5%
        let e = ergo();
        let result = metric.compute(&state, &e);
        assert_eq!(result.level, MetricLevel::Red);
    }

    #[test]
    fn standing_pct_unaffected_by_break_credit() {
        let metric = standing_pct::StandingPercentMetric;
        let mut state = default_state();
        state.sitting_seconds_total = 7200;
        state.sitting_seconds = 0;
        state.standing_seconds = 600;
        let e = ergo();
        let result = metric.compute(&state, &e);
        assert!((result.value - 7.69).abs() < 0.1);
    }

    // ─── PositionChangeRateMetric ────────────────────────────────────────

    #[test]
    fn position_rate_no_data() {
        let metric = position_rate::PositionChangeRateMetric;
        let state = default_state();
        let e = ergo();
        let result = metric.compute(&state, &e);
        assert_eq!(result.display, "\u{2014}");
    }

    #[test]
    fn position_rate_normal() {
        let metric = position_rate::PositionChangeRateMetric;
        let mut state = default_state();
        state.first_reading_at = Some(Utc::now() - chrono::Duration::hours(2));
        state.position_changes = 4;
        let e = ergo();
        let result = metric.compute(&state, &e);
        assert!((result.value - 2.0).abs() < 0.1);
        assert_eq!(result.level, MetricLevel::Green);
    }

    // ─── LongestSessionMetric ────────────────────────────────────────────

    #[test]
    fn longest_session_no_data() {
        let metric = longest_session::LongestSessionMetric;
        let state = default_state();
        let e = ergo();
        let result = metric.compute(&state, &e);
        assert_eq!(result.display, "\u{2014}");
    }

    #[test]
    fn longest_session_green() {
        let metric = longest_session::LongestSessionMetric;
        let mut state = default_state();
        state.first_reading_at = Some(Utc::now() - chrono::Duration::hours(2));
        state.longest_computer_session_secs = 30 * 60; // 30 min
        let e = ergo();
        let result = metric.compute(&state, &e);
        assert_eq!(result.display, "30m");
        assert_eq!(result.level, MetricLevel::Green);
    }

    #[test]
    fn longest_session_red() {
        let metric = longest_session::LongestSessionMetric;
        let mut state = default_state();
        state.first_reading_at = Some(Utc::now() - chrono::Duration::hours(2));
        state.longest_computer_session_secs = 80 * 60; // 80 min
        let e = ergo();
        let result = metric.compute(&state, &e);
        assert_eq!(result.display, "1h20m");
        assert_eq!(result.level, MetricLevel::Red);
    }

    // ─── Utility functions ───────────────────────────────────────────────

    #[test]
    fn threshold_level_higher_better_works() {
        assert_eq!(threshold_level_higher_better(20.0, 15.0, 10.0), MetricLevel::Green);
        assert_eq!(threshold_level_higher_better(12.0, 15.0, 10.0), MetricLevel::Yellow);
        assert_eq!(threshold_level_higher_better(5.0, 15.0, 10.0), MetricLevel::Red);
        assert_eq!(threshold_level_higher_better(15.0, 15.0, 10.0), MetricLevel::Green);
        assert_eq!(threshold_level_higher_better(10.0, 15.0, 10.0), MetricLevel::Yellow);
    }

    #[test]
    fn threshold_level_lower_better_works() {
        assert_eq!(threshold_level_lower_better(30.0, 45.0, 75.0), MetricLevel::Green);
        assert_eq!(threshold_level_lower_better(60.0, 45.0, 75.0), MetricLevel::Yellow);
        assert_eq!(threshold_level_lower_better(90.0, 45.0, 75.0), MetricLevel::Red);
        assert_eq!(threshold_level_lower_better(45.0, 45.0, 75.0), MetricLevel::Green);
        assert_eq!(threshold_level_lower_better(75.0, 45.0, 75.0), MetricLevel::Yellow);
    }
}
