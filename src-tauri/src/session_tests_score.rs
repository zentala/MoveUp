//! session_tests_score.rs — Unit tests for the points/score system (T028).

#[cfg(test)]
mod tests {
    use crate::config::AppConfig;
    use crate::session_manager::SessionManager;
    use crate::session_types::DeskState;

    fn default_config() -> AppConfig {
        AppConfig::default()
    }

    fn standing_manager() -> SessionManager {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Standing;
        m
    }

    fn sitting_manager() -> SessionManager {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Sitting;
        m
    }

    #[test]
    fn score_starts_at_zero() {
        let m = SessionManager::new();
        assert!((m.state.daily_score - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn sitting_60_ticks_decreases_score() {
        let mut m = sitting_manager();
        let config = default_config();
        for _ in 0..60 {
            m.accumulate_score_tick(&config);
        }
        // 60 ticks * (-0.5 / 60) = -0.5
        let expected = -0.5_f32;
        assert!((m.state.daily_score - expected).abs() < 0.01,
            "expected ~{}, got {}", expected, m.state.daily_score);
    }

    #[test]
    fn standing_60_ticks_increases_score() {
        let mut m = standing_manager();
        let config = default_config();
        for _ in 0..60 {
            m.accumulate_score_tick(&config);
        }
        // 60 ticks * (1.0 / 60) = 1.0 (no bonus yet, only 60s standing)
        let expected = 1.0_f32;
        assert!((m.state.daily_score - expected).abs() < 0.02,
            "expected ~{}, got {}", expected, m.state.daily_score);
    }

    #[test]
    fn walking_ticks_neutral() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Walking;
        let config = default_config();
        for _ in 0..120 {
            m.accumulate_score_tick(&config);
        }
        assert!((m.state.daily_score - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn away_ticks_neutral() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Away;
        let config = default_config();
        for _ in 0..120 {
            m.accumulate_score_tick(&config);
        }
        assert!((m.state.daily_score - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn lap_bonus_awarded_at_target() {
        let mut m = standing_manager();
        let config = default_config(); // standing_target_mins = 15
        let target_ticks = 15 * 60; // 900 ticks
        for _ in 0..target_ticks {
            m.accumulate_score_tick(&config);
        }
        // Score = 15 min * 1.0 pts/min + 5.0 bonus = 20.0
        assert_eq!(m.state.lap_bonus_awarded_for_lap, 1);
        let expected = 20.0_f32;
        assert!((m.state.daily_score - expected).abs() < 0.5,
            "expected ~{}, got {}", expected, m.state.daily_score);
    }

    #[test]
    fn no_bonus_before_target() {
        let mut m = standing_manager();
        let config = default_config();
        let ticks = 14 * 60; // 840s, target is 900s
        for _ in 0..ticks {
            m.accumulate_score_tick(&config);
        }
        assert_eq!(m.state.lap_bonus_awarded_for_lap, 0);
    }

    #[test]
    fn no_double_bonus_same_lap() {
        let mut m = standing_manager();
        let config = default_config();
        // Go to exactly the target, then 1 more tick.
        for _ in 0..(15 * 60 + 1) {
            m.accumulate_score_tick(&config);
        }
        assert_eq!(m.state.lap_bonus_awarded_for_lap, 1,
            "bonus should be awarded exactly once for lap 1");
    }

    #[test]
    fn two_laps_two_bonuses() {
        let mut m = standing_manager();
        let config = default_config();
        let ticks = 30 * 60; // 2 full laps
        for _ in 0..ticks {
            m.accumulate_score_tick(&config);
        }
        assert_eq!(m.state.lap_bonus_awarded_for_lap, 2);
        // 30 min * 1.0 + 2 * 5.0 = 40.0
        let expected = 40.0_f32;
        assert!((m.state.daily_score - expected).abs() < 1.0,
            "expected ~{}, got {}", expected, m.state.daily_score);
    }

    #[test]
    fn standing_session_resets_on_sit_allow_new_bonus() {
        let mut m = standing_manager();
        let config = default_config();
        // Complete 1 lap.
        for _ in 0..(15 * 60) {
            m.accumulate_score_tick(&config);
        }
        assert_eq!(m.state.lap_bonus_awarded_for_lap, 1);

        // Simulate sitting transition: reset per-session fields.
        m.state.standing_session_secs = 0;
        m.state.lap_bonus_awarded_for_lap = 0;
        m.state.state = DeskState::Standing;

        // Another full lap.
        for _ in 0..(15 * 60) {
            m.accumulate_score_tick(&config);
        }
        assert_eq!(m.state.lap_bonus_awarded_for_lap, 1,
            "new standing session should earn lap 1 bonus again");
    }

    #[test]
    fn daily_reset_clears_score() {
        let mut m = standing_manager();
        m.state.daily_score = 42.0;
        m.state.standing_session_secs = 900;
        m.state.lap_bonus_awarded_for_lap = 1;

        // Force a day change by backdating last_reset_date.
        m.last_reset_date = m.last_reset_date.pred_opt().unwrap();
        m.last_reset_check = chrono::Utc::now() - chrono::Duration::seconds(120);

        let did_reset = m.check_daily_reset();
        assert!(did_reset);
        assert!((m.state.daily_score - 0.0).abs() < f32::EPSILON);
        assert_eq!(m.state.standing_session_secs, 0);
        assert_eq!(m.state.lap_bonus_awarded_for_lap, 0);
    }

    #[test]
    fn snapshot_includes_daily_score() {
        let mut m = standing_manager();
        m.state.daily_score = 12.5;
        m.state.standing_session_secs = 300;
        let snap = m.snapshot();
        assert!((snap.daily_score - 12.5).abs() < f32::EPSILON);
        assert_eq!(snap.standing_session_secs, 300);
    }

    #[test]
    fn config_defaults_correct() {
        let config = AppConfig::default();
        assert!((config.pts_standing_per_min - 1.0).abs() < f32::EPSILON);
        assert!((config.pts_session_bonus - 5.0).abs() < f32::EPSILON);
        assert!((config.pts_sitting_per_min - (-0.5)).abs() < f32::EPSILON);
    }
}
