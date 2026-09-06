//! session_tests_scenarios.rs — Realistic multi-cycle day scenarios (1-4).
//! Scenarios 5-8 in session_tests_scenarios_adv.rs.

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Duration, TimeZone, Utc};

    use crate::session_manager::SessionManager;
    use crate::session_types::*;

    const SIT_MM: i32 = 800;
    const STAND_MM: i32 = 1200;

    /// A fixed instant, so nothing here depends on when the suite runs.
    fn base() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 1, 8, 0, 0).unwrap()
    }

    /// Feeds `count` readings one second apart, advancing the scenario clock.
    fn feed(
        m: &mut SessionManager,
        now: &mut DateTime<Utc>,
        mm: i32,
        active: bool,
        count: i64,
    ) {
        for _ in 0..count {
            let _ = m.on_reading_at(mm, active, *now);
            *now += Duration::seconds(1);
            if let Some(ts) = m.state.last_accumulate_ts {
                // Keeps per-second accumulation throttled, as before: these
                // scenarios assert transitions, not tick counters.
                m.state.last_accumulate_ts = Some(ts + Duration::seconds(1));
            }
        }
    }

    /// Feeds a debounce batch and returns the reading that carried the
    /// transition (the last one).
    fn feed_last(
        m: &mut SessionManager,
        now: &mut DateTime<Utc>,
        mm: i32,
        active: bool,
    ) -> ReadingResult {
        let mut last = ReadingResult::default();
        for _ in 0..DEBOUNCE_COUNT {
            last = m.on_reading_at(mm, active, *now);
            *now += Duration::seconds(1);
        }
        last
    }

    fn transition_to(
        m: &mut SessionManager,
        now: &mut DateTime<Utc>,
        state: DeskState,
        mm: i32,
        active: bool,
    ) {
        feed(m, now, mm, active, DEBOUNCE_COUNT as i64);
        assert_eq!(m.state.state, state, "expected {:?}", state);
    }

    /// Moves the scenario clock forward by `secs` (E020-T01).
    ///
    /// The old form rewound every start timestamp instead, because `on_reading`
    /// always ran at the real "now". With the clock injected, moving forward is
    /// the honest operation. `last_tick_ts` moves with it: readings kept
    /// arriving through this stretch, and without that the next one would look
    /// like a sleep gap and earn break credit.
    fn simulate_elapsed(m: &mut SessionManager, now: &mut DateTime<Utc>, secs: i64) {
        *now += Duration::seconds(secs);
        m.state.last_tick_ts = Some(*now);
    }

    // ─── Scenario 1: Typical morning ─────────────────────────────────────
    // Sit 25min → Stand 8min → Away 12min → Sit 35min → Stand 5min → Sit
    // ~85 min total, 2 standing sessions, 1 long Away (full credit)

    #[test]
    fn scenario_typical_morning() {
        let mut m = SessionManager::new();
        let mut now = base();

        // Step 1: Start sitting
        transition_to(&mut m, &mut now, DeskState::Sitting, SIT_MM, true);
        simulate_elapsed(&mut m, &mut now, 25 * 60); // 25 min sitting
        assert_eq!(m.state.position_changes, 0);

        // Step 2: Stand up (break starts)
        transition_to(&mut m, &mut now, DeskState::Standing, STAND_MM, true);
        assert_eq!(m.state.position_changes, 1);
        assert!(m.state.break_started.is_some(), "break must start on Sit→Stand");
        let sitting_after_stand = m.state.sitting_seconds;
        assert!(sitting_after_stand >= 1400, "~25 min sitting committed: got {}", sitting_after_stand);
        simulate_elapsed(&mut m, &mut now, 8 * 60); // 8 min standing

        // Step 3: Walk away (standing continues as break)
        transition_to(&mut m, &mut now, DeskState::Away, STAND_MM, false);
        assert_eq!(m.state.position_changes, 1, "Stand→Away should NOT increment");
        let standing_after_away = m.state.standing_seconds;
        assert!(standing_after_away >= 470, "~8 min standing committed: got {}", standing_after_away);
        simulate_elapsed(&mut m, &mut now, 12 * 60); // 12 min away

        // Step 4: Return and sit down (break credit applied)
        transition_to(&mut m, &mut now, DeskState::Sitting, SIT_MM, true);
        // Total break = 8m stand + 12m away = 20m >= 10m → Full credit
        assert_eq!(m.state.last_break_credit, BreakCredit::Full);
        assert_eq!(m.state.sitting_seconds, 0, "full credit resets sitting");
        // Away→Sit is NOT a Sit↔Stand transition, so position_changes stays at 1
        assert_eq!(m.state.position_changes, 1, "only Sit↔Stand increments position_changes");

        // Step 5: Sit for 35 min
        simulate_elapsed(&mut m, &mut now, 35 * 60);

        // Step 6: Quick stand (5 min — partial credit on return)
        transition_to(&mut m, &mut now, DeskState::Standing, STAND_MM, true);
        let sitting_before = m.state.sitting_seconds;
        assert!(sitting_before >= 2050, "~35 min sitting: got {}", sitting_before);
        simulate_elapsed(&mut m, &mut now, 5 * 60);

        // Step 7: Sit back down
        transition_to(&mut m, &mut now, DeskState::Sitting, SIT_MM, true);
        // 5m break = partial credit (-1200s)
        assert_eq!(m.state.last_break_credit, BreakCredit::Partial);
        let final_sitting = m.state.sitting_seconds;
        assert!(final_sitting < sitting_before, "partial credit must reduce sitting");
        assert!(final_sitting >= 800, "sitting_before - 1200 ≈ 900: got {}", final_sitting);
    }

    // ─── Scenario 2: Standing with Away oscillation ──────────────────────
    // Real pattern: Stand→Away(60s idle)→Stand→Away→Stand→Sit
    // Standing time should accumulate only actual standing bouts.

    #[test]
    fn scenario_standing_away_oscillation() {
        let mut m = SessionManager::new();
        let mut now = base();

        // Start sitting for 20 min
        transition_to(&mut m, &mut now, DeskState::Sitting, SIT_MM, true);
        simulate_elapsed(&mut m, &mut now, 20 * 60);

        // Stand up
        transition_to(&mut m, &mut now, DeskState::Standing, STAND_MM, true);
        simulate_elapsed(&mut m, &mut now, 3 * 60); // 3 min standing

        // Away (idle kicks in after 60s)
        transition_to(&mut m, &mut now, DeskState::Away, STAND_MM, false);
        let standing_1 = m.state.standing_seconds;
        assert!(standing_1 >= 170, "~3 min standing bout 1: got {}", standing_1);
        simulate_elapsed(&mut m, &mut now, 2 * 60); // 2 min away

        // Back to standing (keyboard activity)
        transition_to(&mut m, &mut now, DeskState::Standing, STAND_MM, true);
        simulate_elapsed(&mut m, &mut now, 4 * 60); // 4 min standing

        // Away again
        transition_to(&mut m, &mut now, DeskState::Away, STAND_MM, false);
        let standing_2 = m.state.standing_seconds;
        // Should be ~3m + ~4m = ~7m = ~420s, NOT 3+2+4 = 9m
        assert!(standing_2 >= 400 && standing_2 <= 450,
            "standing should be ~420s (3+4m, no away time): got {}", standing_2);
        simulate_elapsed(&mut m, &mut now, 5 * 60); // 5 min away

        // Back to standing briefly
        transition_to(&mut m, &mut now, DeskState::Standing, STAND_MM, true);
        simulate_elapsed(&mut m, &mut now, 1 * 60); // 1 min

        // Sit down (total break = entire standing+away period)
        transition_to(&mut m, &mut now, DeskState::Sitting, SIT_MM, true);
        // Total break from first Stand to Sit = 3+2+4+5+1 = 15 min → Full credit
        assert_eq!(m.state.last_break_credit, BreakCredit::Full,
            "15+ min total break should be full credit");
        assert_eq!(m.state.sitting_seconds, 0, "full credit resets sitting");

        // Standing should be ~3+4+1 = ~8 min = ~480s (no away time)
        let final_standing = m.state.standing_seconds;
        assert!(final_standing >= 460 && final_standing <= 510,
            "standing total should be ~480s (3+4+1m): got {}", final_standing);
    }

    // ─── Scenario 3: Short breaks that give no credit ────────────────────
    // Sit 30min → Stand 30s → Sit 20min → Away 30s → Sit
    // Neither break reaches BREAK_MIN_SECS (60s), so no credit applied.

    #[test]
    fn scenario_short_breaks_no_credit() {
        let mut m = SessionManager::new();
        let mut now = base();

        // Sit 30 min
        transition_to(&mut m, &mut now, DeskState::Sitting, SIT_MM, true);
        simulate_elapsed(&mut m, &mut now, 30 * 60);

        // Stand 30s (< 60s threshold → no credit)
        transition_to(&mut m, &mut now, DeskState::Standing, STAND_MM, true);
        simulate_elapsed(&mut m, &mut now, 30);

        // Sit back (30s break < 60s → no credit)
        transition_to(&mut m, &mut now, DeskState::Sitting, SIT_MM, true);
        assert_eq!(m.state.last_break_credit, BreakCredit::None);
        let sitting_after = m.state.sitting_seconds;
        // Should be ~30min sitting (committed) + no credit reduction
        assert!(sitting_after >= 1780, "no credit: sitting should be ~1800: got {}", sitting_after);

        // Sit 20 more min
        simulate_elapsed(&mut m, &mut now, 20 * 60);

        // Away 30s (keyboard idle)
        transition_to(&mut m, &mut now, DeskState::Away, SIT_MM, false);
        simulate_elapsed(&mut m, &mut now, 30);

        // Back to sitting (30s < 60s → no credit)
        transition_to(&mut m, &mut now, DeskState::Sitting, SIT_MM, true);
        assert_eq!(m.state.last_break_credit, BreakCredit::None);
        // Sitting should be ~50min total (30 + 20)
        let final_sitting = m.state.sitting_seconds;
        assert!(final_sitting >= 2900, "50 min sitting, no credit: got {}", final_sitting);
    }

    // ─── Scenario 4: CompletedSession records for timeline ──────────────
    // Verify that each transition creates the right CompletedSession.

    #[test]
    fn scenario_timeline_records_all_sessions() {
        let mut m = SessionManager::new();
        let mut now = base();
        let mut completed_sessions: Vec<CompletedSession> = Vec::new();

        // Sit 10 min
        transition_to(&mut m, &mut now, DeskState::Sitting, SIT_MM, true);
        simulate_elapsed(&mut m, &mut now, 10 * 60);

        // Stand (creates CompletedSession for sitting)
        let mut last = feed_last(&mut m, &mut now, STAND_MM, true);
        if let Some(ref cs) = last.completed_session {
            completed_sessions.push(cs.clone());
        }
        assert_eq!(m.state.state, DeskState::Standing);
        assert_eq!(completed_sessions.len(), 1, "Sit→Stand must create 1 session record");
        assert!(completed_sessions[0].duration_secs >= 590,
            "sitting session ~10m: got {}s", completed_sessions[0].duration_secs);

        // Stand 7 min then sit
        simulate_elapsed(&mut m, &mut now, 7 * 60);
        last = feed_last(&mut m, &mut now, SIT_MM, true);
        if let Some(ref cs) = last.completed_session {
            completed_sessions.push(cs.clone());
        }
        assert_eq!(m.state.state, DeskState::Sitting);
        assert_eq!(completed_sessions.len(), 2, "Stand→Sit must create break session record");

        // Sit 15 min then away
        simulate_elapsed(&mut m, &mut now, 15 * 60);
        last = feed_last(&mut m, &mut now, SIT_MM, false);
        if let Some(ref cs) = last.completed_session {
            completed_sessions.push(cs.clone());
        }
        assert_eq!(m.state.state, DeskState::Away);
        assert_eq!(completed_sessions.len(), 3, "Sit→Away must create session record");

        // Away 12 min then sit
        simulate_elapsed(&mut m, &mut now, 12 * 60);
        last = feed_last(&mut m, &mut now, SIT_MM, true);
        if let Some(ref cs) = last.completed_session {
            completed_sessions.push(cs.clone());
        }
        assert_eq!(m.state.state, DeskState::Sitting);
        assert_eq!(completed_sessions.len(), 4, "Away→Sit must create break session record");

        // Verify all 4 records have positive durations
        for (i, cs) in completed_sessions.iter().enumerate() {
            assert!(cs.duration_secs > 0,
                "session {} has 0 duration", i);
            assert!(!cs.started_at.is_empty(),
                "session {} has empty started_at", i);
            assert!(!cs.ended_at.is_empty(),
                "session {} has empty ended_at", i);
        }
    }
    // Scenarios 5-8 in session_tests_scenarios_adv.rs
}
