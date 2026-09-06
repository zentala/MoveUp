//! session_tests_away_transitions.rs — Away state transition + credit tests (E002-T04).
//!
//! Readings are injected at explicit instants (E020-T01). The old form read the
//! system clock once per backdated timestamp and again inside every reading, so
//! a debounce batch appeared to take zero time; here a batch takes
//! `DEBOUNCE_COUNT` seconds, which is what the sensor actually does. Durations
//! are therefore stated relative to the instant a transition CONFIRMS, not to
//! the start of the batch.

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Duration, TimeZone, Utc};

    use crate::session_manager::SessionManager;
    use crate::session_types::*;

    /// A fixed instant, so nothing here depends on when the suite runs.
    fn base() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 1, 9, 0, 0).unwrap()
    }

    /// Feeds one debounce batch of readings, one second apart, starting at `at`.
    /// Returns the instant the NEXT batch should start from.
    fn advance_ticks(
        m: &mut SessionManager,
        mm: i32,
        active: bool,
        at: DateTime<Utc>,
    ) -> DateTime<Utc> {
        let (next, _) = batch(m, mm, active, at);
        next
    }

    /// Same as [`advance_ticks`], but also returns the last `ReadingResult` —
    /// the one carrying the transition, since the batch confirms on its final
    /// reading.
    fn batch(
        m: &mut SessionManager,
        mm: i32,
        active: bool,
        at: DateTime<Utc>,
    ) -> (DateTime<Utc>, ReadingResult) {
        let mut last = ReadingResult {
            state_change: None,
            completed_session: None,
            break_credit: None,
        };
        for i in 0..DEBOUNCE_COUNT as i64 {
            last = m.on_reading_at(mm, active, at + Duration::seconds(i));
        }
        (at + Duration::seconds(DEBOUNCE_COUNT as i64), last)
    }

    /// The instant the batch starting at `at` confirms its transition.
    fn confirms_at(at: DateTime<Utc>) -> DateTime<Utc> {
        at + Duration::seconds(DEBOUNCE_COUNT as i64 - 1)
    }

    // ─── Away → Sitting = New Session ───────────────────────────────────────

    #[test]
    fn away_to_sitting_full_credit() {
        // 1100s break → credit = 1100*2 = 2200 >= sitting (2000) → Full.
        let mut m = SessionManager::new();
        let t = advance_ticks(&mut m, 800, true, base());
        assert_eq!(m.state.state, DeskState::Sitting);
        m.state.sitting_seconds = 2000;
        m.state.sitting_started = Some(t - Duration::seconds(100));

        let t = advance_ticks(&mut m, 800, false, t);
        assert_eq!(m.state.state, DeskState::Away);

        m.state.sitting_seconds = 2000; // reseeded: the exit above committed the bout
        m.state.break_started = Some(confirms_at(t) - Duration::seconds(1100));
        advance_ticks(&mut m, 800, true, t);
        assert_eq!(m.state.state, DeskState::Sitting);

        assert_eq!(m.state.sitting_seconds, 0, "break credit >= sitting => full reset");
        assert!(m.state.sitting_started.is_some(), "new session started");
    }

    #[test]
    fn away_to_sitting_partial_credit() {
        let mut m = SessionManager::new();
        let t = advance_ticks(&mut m, 800, true, base());
        m.state.sitting_seconds = 2000;
        m.state.sitting_started = Some(t - Duration::seconds(100));

        let t = advance_ticks(&mut m, 800, false, t);
        assert_eq!(m.state.state, DeskState::Away);

        m.state.sitting_seconds = 2000;
        // 7 min break → credit 840 < 2000 → Partial.
        m.state.break_started = Some(confirms_at(t) - Duration::seconds(420));
        advance_ticks(&mut m, 800, true, t);
        assert_eq!(m.state.state, DeskState::Sitting);
        assert_eq!(m.state.last_break_credit, BreakCredit::Partial, "7 min = partial");
        assert_eq!(m.state.sitting_seconds, 2000 - 840);
    }

    #[test]
    fn away_to_sitting_no_credit() {
        // Threshold is BREAK_MIN_SECS=60 (1 min). Use 30s break to get None.
        let mut m = SessionManager::new();
        let t = advance_ticks(&mut m, 800, true, base());
        m.state.sitting_seconds = 2000;
        m.state.sitting_started = Some(t - Duration::seconds(100));

        let t = advance_ticks(&mut m, 800, false, t);
        assert_eq!(m.state.state, DeskState::Away);

        m.state.break_started = Some(confirms_at(t) - Duration::seconds(30));
        advance_ticks(&mut m, 800, true, t);
        assert_eq!(m.state.state, DeskState::Sitting);
        assert_eq!(m.state.last_break_credit, BreakCredit::None, "<1 min = no credit");
    }

    #[test]
    fn away_to_standing_resets_away_bout() {
        let mut m = SessionManager::new();
        let t = advance_ticks(&mut m, 800, true, base());
        m.state.sitting_started = Some(t);

        let t = advance_ticks(&mut m, 800, false, t);
        assert_eq!(m.state.state, DeskState::Away);
        m.state.away_bout_secs = 120;

        advance_ticks(&mut m, 1200, true, t);
        assert_eq!(m.state.state, DeskState::Standing);
        assert_eq!(m.state.away_bout_secs, 0, "away_bout must reset on standing");
    }

    #[test]
    fn sitting_to_away_creates_completed_session() {
        let mut m = SessionManager::new();
        let t = advance_ticks(&mut m, 800, true, base());
        m.state.sitting_started = Some(t - Duration::seconds(300));

        let (_, last) = batch(&mut m, 800, false, t);
        assert_eq!(m.state.state, DeskState::Away);
        assert!(
            last.completed_session.is_some(),
            "Sitting→Away must create CompletedSession for timeline"
        );
    }

    /// Regression test: Standing + inactive MUST transition to Away.
    /// Bug report: user was standing, walked away, app stayed in Standing.
    /// Verifies: (1) state transitions to Away, (2) break_started preserved,
    /// (3) state_change event is emitted, (4) break credit applied on return.
    #[test]
    fn standing_to_away_on_inactivity() {
        let mut m = SessionManager::new();

        // 1. Start Sitting
        let t = advance_ticks(&mut m, 800, true, base());
        assert_eq!(m.state.state, DeskState::Sitting);
        m.state.sitting_started = Some(t - Duration::seconds(600));
        m.state.sitting_seconds = 600;
        m.state.sitting_seconds_total = 600;

        // 2. Stand up (active) — should transition to Standing
        let t = advance_ticks(&mut m, 1200, true, t);
        assert_eq!(m.state.state, DeskState::Standing, "should be Standing when desk high + active");
        assert!(m.state.break_started.is_some(), "break_started must be set on Sitting→Standing");
        let break_start = m.state.break_started.unwrap();

        // 3. Walk away (inactive, desk still high) — MUST transition to Away
        let (t, last) = batch(&mut m, 1200, false, t);
        assert_eq!(
            m.state.state, DeskState::Away,
            "Standing + inactive MUST transition to Away (not stay Standing)"
        );
        assert!(
            last.state_change.is_some(),
            "state_change event must be emitted for Standing→Away"
        );
        assert_eq!(
            last.state_change.as_ref().unwrap().state,
            DeskState::Away,
            "emitted state must be Away"
        );
        // break_started must be preserved (break continues through Away)
        assert_eq!(
            m.state.break_started,
            Some(break_start),
            "break_started must be preserved on Standing→Away"
        );

        // 4. Stay away for 10+ minutes.
        m.state.break_started = Some(confirms_at(t) - Duration::seconds(700));

        // 5. Return and sit down — break credit should apply
        advance_ticks(&mut m, 800, true, t);
        assert_eq!(m.state.state, DeskState::Sitting);
        assert_eq!(
            m.state.last_break_credit, BreakCredit::Full,
            ">=10 min break (standing+away) = full credit"
        );
        assert_eq!(m.state.sitting_seconds, 0, "full credit resets sitting to 0");
    }

    // Sleep inflation tests moved to session_tests_sleep.rs

    #[test]
    fn standing_away_standing_sitting_only_counts_standing() {
        let mut m = SessionManager::new();
        // Start sitting
        let t = advance_ticks(&mut m, 800, true, base());
        m.state.sitting_started = Some(t);

        // Stand up, then pretend the bout has been running for 300s.
        let t = advance_ticks(&mut m, 1200, true, t);
        assert_eq!(m.state.state, DeskState::Standing);
        let first_bout_start = confirms_at(t) - Duration::seconds(300);
        m.state.break_started = Some(first_bout_start);
        m.state.standing_bout_started = Some(first_bout_start);

        // Go away — the standing bout is committed here, at exactly 300s.
        let t = advance_ticks(&mut m, 1200, false, t);
        assert_eq!(m.state.state, DeskState::Away);
        assert_eq!(m.state.standing_seconds, 300);

        // Come back standing, then pretend that bout has run for 180s.
        let t = advance_ticks(&mut m, 1200, true, t);
        assert_eq!(m.state.state, DeskState::Standing);
        m.state.standing_bout_started = Some(confirms_at(t) - Duration::seconds(180));

        // Sit down — commits the second bout.
        advance_ticks(&mut m, 800, true, t);
        assert_eq!(m.state.state, DeskState::Sitting);

        // Total standing is 300 + 180; the away stretch in between must not count.
        assert_eq!(
            m.state.standing_seconds, 480,
            "away time must not count towards standing"
        );
    }

    #[test]
    fn away_to_sitting_creates_completed_session() {
        let mut m = SessionManager::new();
        let t = advance_ticks(&mut m, 800, true, base());
        m.state.sitting_started = Some(t);

        let t = advance_ticks(&mut m, 800, false, t);
        assert_eq!(m.state.state, DeskState::Away);

        m.state.break_started = Some(confirms_at(t) - Duration::seconds(600));
        let (_, last) = batch(&mut m, 800, true, t);
        assert_eq!(m.state.state, DeskState::Sitting);
        assert!(
            last.completed_session.is_some(),
            "Away→Sitting must create CompletedSession for timeline"
        );
    }
}
