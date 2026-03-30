//! session_tests_score.rs — Unit tests for the points/score system (T028).

#[cfg(test)]
mod tests {
    use crate::ergonomic_profile::ErgonomicProfile;
    use crate::session_manager::SessionManager;
    use crate::session_types::DeskState;

    fn default_ergo() -> ErgonomicProfile {
        ErgonomicProfile::default()
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
        let ergo = default_ergo();
        for _ in 0..60 {
            m.accumulate_score_tick(&ergo);
        }
        // 60 ticks * (-0.5 / 60) = -0.5
        let expected = -0.5_f32;
        assert!((m.state.daily_score - expected).abs() < 0.01,
            "expected ~{}, got {}", expected, m.state.daily_score);
    }

    #[test]
    fn standing_60_ticks_increases_score() {
        let mut m = standing_manager();
        let ergo = default_ergo();
        for _ in 0..60 {
            m.accumulate_score_tick(&ergo);
        }
        // 60 ticks * (1.0 / 60) = 1.0 (no bonus — standing_session_started not set)
        let expected = 1.0_f32;
        assert!((m.state.daily_score - expected).abs() < 0.02,
            "expected ~{}, got {}", expected, m.state.daily_score);
    }

    #[test]
    fn walking_ticks_neutral() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Walking;
        let ergo = default_ergo();
        for _ in 0..120 {
            m.accumulate_score_tick(&ergo);
        }
        assert!((m.state.daily_score - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn away_ticks_neutral() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Away;
        let ergo = default_ergo();
        for _ in 0..120 {
            m.accumulate_score_tick(&ergo);
        }
        assert!((m.state.daily_score - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn lap_bonus_awarded_at_target() {
        let mut m = standing_manager();
        let ergo = default_ergo(); // standing_target_secs = 900
        // Simulate 15 minutes of standing via timestamp.
        m.state.standing_session_started =
            Some(chrono::Utc::now() - chrono::Duration::seconds(15 * 60));
        let target_ticks = 15 * 60;
        for _ in 0..target_ticks {
            m.accumulate_score_tick(&ergo);
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
        let ergo = default_ergo();
        // Simulate 14 minutes of standing (below 15 min target).
        m.state.standing_session_started =
            Some(chrono::Utc::now() - chrono::Duration::seconds(14 * 60));
        let ticks = 14 * 60;
        for _ in 0..ticks {
            m.accumulate_score_tick(&ergo);
        }
        assert_eq!(m.state.lap_bonus_awarded_for_lap, 0);
    }

    #[test]
    fn no_double_bonus_same_lap() {
        let mut m = standing_manager();
        let ergo = default_ergo();
        // Simulate 15 min + 1s of standing.
        m.state.standing_session_started =
            Some(chrono::Utc::now() - chrono::Duration::seconds(15 * 60 + 1));
        for _ in 0..(15 * 60 + 1) {
            m.accumulate_score_tick(&ergo);
        }
        assert_eq!(m.state.lap_bonus_awarded_for_lap, 1,
            "bonus should be awarded exactly once for lap 1");
    }

    #[test]
    fn two_laps_two_bonuses() {
        let mut m = standing_manager();
        let ergo = default_ergo();
        // Simulate 30 minutes of standing (2 full laps).
        m.state.standing_session_started =
            Some(chrono::Utc::now() - chrono::Duration::seconds(30 * 60));
        let ticks = 30 * 60;
        for _ in 0..ticks {
            m.accumulate_score_tick(&ergo);
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
        let ergo = default_ergo();
        // Complete 1 lap: 15 min of standing.
        m.state.standing_session_started =
            Some(chrono::Utc::now() - chrono::Duration::seconds(15 * 60));
        for _ in 0..(15 * 60) {
            m.accumulate_score_tick(&ergo);
        }
        assert_eq!(m.state.lap_bonus_awarded_for_lap, 1);

        // Simulate sitting transition: reset per-session fields.
        m.state.standing_session_secs = 0;
        m.state.standing_session_started = None;
        m.state.lap_bonus_awarded_for_lap = 0;
        m.state.state = DeskState::Standing;

        // Another full lap: 15 min from now.
        m.state.standing_session_started =
            Some(chrono::Utc::now() - chrono::Duration::seconds(15 * 60));
        for _ in 0..(15 * 60) {
            m.accumulate_score_tick(&ergo);
        }
        assert_eq!(m.state.lap_bonus_awarded_for_lap, 1,
            "new standing session should earn lap 1 bonus again");
    }

    #[test]
    fn daily_reset_clears_score() {
        let mut m = standing_manager();
        m.state.daily_score = 42.0;
        m.state.standing_session_secs = 900;
        m.state.standing_session_started = Some(chrono::Utc::now());
        m.state.lap_bonus_awarded_for_lap = 1;

        // Force a day change by backdating last_reset_date.
        m.last_reset_date = m.last_reset_date.pred_opt().unwrap();
        m.last_reset_check = chrono::Utc::now() - chrono::Duration::seconds(120);

        let did_reset = m.check_daily_reset();
        assert!(did_reset);
        assert!((m.state.daily_score - 0.0).abs() < f32::EPSILON);
        assert_eq!(m.state.standing_session_secs, 0);
        assert!(m.state.standing_session_started.is_none());
        assert_eq!(m.state.lap_bonus_awarded_for_lap, 0);
    }

    #[test]
    fn snapshot_includes_daily_score() {
        let mut m = standing_manager();
        m.state.daily_score = 12.5;
        m.state.standing_session_secs = 300;
        m.state.standing_session_started =
            Some(chrono::Utc::now() - chrono::Duration::seconds(300));
        let snap = m.snapshot();
        assert!((snap.daily_score - 12.5).abs() < f32::EPSILON);
        assert!(snap.standing_session_secs >= 299 && snap.standing_session_secs <= 301,
            "expected ~300, got {}", snap.standing_session_secs);
    }

    #[test]
    fn ergo_profile_defaults_correct() {
        let ergo = ErgonomicProfile::default();
        assert!((ergo.scoring.pts_standing_per_min - 1.0).abs() < f32::EPSILON);
        assert!((ergo.scoring.pts_session_bonus - 5.0).abs() < f32::EPSILON);
        assert!((ergo.scoring.pts_sitting_per_min - (-0.5)).abs() < f32::EPSILON);
    }
}
