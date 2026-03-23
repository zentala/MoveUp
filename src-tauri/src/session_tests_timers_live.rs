//! session_tests_timers_live.rs — T042: Live time counting and snapshot tests.

#[cfg(test)]
mod timer_live_tests {
    use chrono::Utc;

    use crate::session_manager::SessionManager;
    use crate::session_types::*;

    fn transition_to_sitting(m: &mut SessionManager) {
        for _ in 0..DEBOUNCE_COUNT {
            let _ = m.on_reading(800, true);
        }
        assert_eq!(m.state.state, DeskState::Sitting);
    }

    fn transition_to_standing(m: &mut SessionManager) {
        for _ in 0..DEBOUNCE_COUNT {
            let _ = m.on_reading(1200, true);
        }
        assert_eq!(m.state.state, DeskState::Standing);
    }

    // ─── Time Counting (tests 12–16) ─────────────────────────────────────

    #[test]
    fn t042_12_sitting_5min_live_session_secs() {
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);
        m.state.sitting_started = Some(Utc::now() - chrono::Duration::seconds(300));
        let live = m.get_live_current_session_secs(Utc::now());
        assert!(live >= 299 && live <= 301, "expected ~300, got {}", live);
    }

    #[test]
    fn t042_13_standing_5min_live_break_secs() {
        let mut m = SessionManager::new();
        transition_to_standing(&mut m);
        m.state.break_started = Some(Utc::now() - chrono::Duration::seconds(300));
        let live = m.get_live_break_seconds(Utc::now());
        assert!(live >= 299 && live <= 301, "expected ~300, got {}", live);
    }

    #[test]
    fn t042_14_snapshot_after_60s_sitting() {
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);
        m.state.sitting_started = Some(Utc::now() - chrono::Duration::seconds(60));
        let snap = m.snapshot();
        assert!(snap.sitting_seconds >= 59 && snap.sitting_seconds <= 61);
        assert!(snap.current_session_secs >= 59 && snap.current_session_secs <= 61);
        assert_eq!(snap.break_seconds, 0);
    }

    #[test]
    fn t042_15_snapshot_after_30s_standing() {
        let mut m = SessionManager::new();
        transition_to_standing(&mut m);
        m.state.break_started = Some(Utc::now() - chrono::Duration::seconds(30));
        let snap = m.snapshot();
        assert!(snap.break_seconds >= 29 && snap.break_seconds <= 31);
        assert_eq!(snap.current_session_secs, 0);
    }

    #[test]
    fn t042_16_full_cycle_sit_stand_sit_snapshots() {
        let mut m = SessionManager::new();

        // Phase 1: Sit 5 min
        transition_to_sitting(&mut m);
        m.state.sitting_started = Some(Utc::now() - chrono::Duration::seconds(300));
        let s1 = m.snapshot();
        assert!(s1.current_session_secs >= 299 && s1.current_session_secs <= 301);
        assert_eq!(s1.break_seconds, 0);

        // Phase 2: Stand 5 min
        transition_to_standing(&mut m);
        m.state.break_started = Some(Utc::now() - chrono::Duration::seconds(300));
        let s2 = m.snapshot();
        assert!(s2.break_seconds >= 299 && s2.break_seconds <= 301);
        assert_eq!(s2.current_session_secs, 0);

        // Phase 3: Sit again (partial credit)
        transition_to_sitting(&mut m);
        let s3 = m.snapshot();
        assert_eq!(m.state.last_break_credit, BreakCredit::Partial);
        assert!(s3.current_session_secs < 5, "should be near 0, got {}", s3.current_session_secs);
    }
}
