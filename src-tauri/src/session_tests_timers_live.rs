//! session_tests_timers_live.rs — T042: Live time counting and snapshot tests.
//!
//! **These tests still read the system clock, on purpose.** Most of them assert
//! on `SessionManager::snapshot()`, which calls `Utc::now()` internally.
//! E020-T01 injected the clock into the reading path, not into `snapshot()` —
//! that is E020-T05, which moves the derived fields into the engine. Until then
//! an injected instant here would be compared against the real clock inside
//! `snapshot()`, so every assertion would drift by the difference. Convert this
//! file together with T05, not before it.

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
    fn e015_t042_12_sitting_5min_live_credited_secs() {
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);
        m.state.sitting_started = Some(Utc::now() - chrono::Duration::seconds(300));
        let live = m.get_live_sitting_seconds(Utc::now());
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
    fn e015_t042_14_snapshot_after_60s_sitting() {
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);
        m.state.sitting_started = Some(Utc::now() - chrono::Duration::seconds(60));
        let snap = m.snapshot();
        assert!(snap.sitting_seconds >= 59 && snap.sitting_seconds <= 61);
        // The UI timer reads limit_used_secs — one field, same value.
        assert_eq!(snap.limit_used_secs, snap.sitting_seconds);
        assert_eq!(snap.break_seconds, 0);
    }

    #[test]
    fn e015_t042_15_snapshot_after_30s_standing() {
        let mut m = SessionManager::new();
        transition_to_standing(&mut m);
        m.state.sitting_seconds = 900;
        m.state.break_started = Some(Utc::now() - chrono::Duration::seconds(30));
        let snap = m.snapshot();
        assert!(snap.break_seconds >= 29 && snap.break_seconds <= 31);
        // Standing freezes the credited counter; it does not zero it.
        assert_eq!(snap.limit_used_secs, 900);
    }

    #[test]
    fn t042_15b_snapshot_standing_seconds_includes_live() {
        let mut m = SessionManager::new();
        m.state.standing_seconds = 600; // 10 min from earlier
        transition_to_standing(&mut m);
        let t = Utc::now() - chrono::Duration::seconds(120);
        m.state.break_started = Some(t);
        m.state.standing_bout_started = Some(t);
        let snap = m.snapshot();
        // standing_seconds should include base (600) + live standing bout (120)
        assert!(
            snap.standing_seconds >= 719 && snap.standing_seconds <= 721,
            "expected ~720, got {}",
            snap.standing_seconds
        );
    }

    #[test]
    fn t042_15c_snapshot_standing_seconds_frozen_while_sitting() {
        let mut m = SessionManager::new();
        m.state.standing_seconds = 600;
        transition_to_sitting(&mut m);
        m.state.sitting_started = Some(Utc::now() - chrono::Duration::seconds(60));
        let snap = m.snapshot();
        // While sitting, standing_seconds should NOT grow
        assert_eq!(snap.standing_seconds, 600);
    }

    #[test]
    fn t042_15d_state_changed_payload_uses_live_break_seconds() {
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);
        m.state.sitting_started = Some(Utc::now() - chrono::Duration::seconds(300));
        // Transition to standing — break_seconds should be 0 (just started)
        let _result = m.on_reading(1200, true); // first debounce reading
        // After debounce completes
        for _ in 1..DEBOUNCE_COUNT {
            let _ = m.on_reading(1200, true);
        }
        let result = m.on_reading(1200, true);
        if let Some(payload) = &result.state_change {
            // Break just started, so break_seconds should be ~0
            assert!(
                payload.break_seconds <= 1,
                "expected ~0, got {}",
                payload.break_seconds
            );
        }
    }

    #[test]
    fn t042_15e_standing_to_sitting_creates_completed_session() {
        let mut m = SessionManager::new();
        transition_to_standing(&mut m);
        let t = Utc::now() - chrono::Duration::seconds(300);
        m.state.break_started = Some(t);
        m.state.standing_bout_started = Some(t);
        // Transition back to sitting
        transition_to_sitting(&mut m);
        // The transition should have produced a completed session
        // We can't access the result directly from transition_to_sitting helper,
        // so verify via state: standing_seconds should have accumulated
        assert!(
            m.state.standing_seconds >= 299,
            "expected ~300, got {}",
            m.state.standing_seconds
        );
    }

    #[test]
    fn e015_t042_16_full_cycle_sit_stand_sit_timer_is_credited_never_reset() {
        let mut m = SessionManager::new();

        // Phase 1: Sit 5 min → sitting_seconds committed ≈ 300s on Stand transition
        transition_to_sitting(&mut m);
        m.state.sitting_started = Some(Utc::now() - chrono::Duration::seconds(300));
        let s1 = m.snapshot();
        assert!(s1.limit_used_secs >= 299 && s1.limit_used_secs <= 301);
        assert_eq!(s1.break_seconds, 0);

        // Phase 2: Stand 2 min (credit = 120s * 2.0 = 240s < ~300s sitting → Partial)
        transition_to_standing(&mut m);
        let t2 = Utc::now() - chrono::Duration::seconds(120);
        m.state.break_started = Some(t2);
        m.state.standing_bout_started = Some(t2);
        let s2 = m.snapshot();
        assert!(s2.break_seconds >= 119 && s2.break_seconds <= 121);
        // While standing the timer is frozen at what was consumed, not zeroed.
        assert!(
            s2.limit_used_secs >= 299 && s2.limit_used_secs <= 301,
            "expected ~300 while standing, got {}",
            s2.limit_used_secs
        );
        // standing_seconds should include live break
        assert!(
            s2.standing_seconds >= 119 && s2.standing_seconds <= 121,
            "expected ~120, got {}",
            s2.standing_seconds
        );

        // Phase 3: Sit again — 120s break * 2.0 = 240s credit < ~300s sitting → Partial.
        // This is the assertion E015 inverted: the old suite demanded ~0 here,
        // which is the reset the user complained about. The engine credits, it
        // does not reset, so the timer must show 300 − 240 ≈ 60.
        transition_to_sitting(&mut m);
        let s3 = m.snapshot();
        assert_eq!(m.state.last_break_credit, BreakCredit::Partial);
        assert!(
            s3.limit_used_secs >= 55 && s3.limit_used_secs <= 70,
            "expected ~60 (300 - 120*2.0), got {}",
            s3.limit_used_secs
        );
    }
}
