//! session_tests_e015_credited.rs — E015: the credited counter is the only
//! number the UI reads.
//!
//! These tests run the real `on_reading` path and assert on the DTO the UI
//! consumes, not on an internal helper. Before E015 the DTO carried a second,
//! uncredited counter that was zeroed on every return to sitting; the user saw
//! a reset the engine never made.

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    use crate::session_manager::SessionManager;
    use crate::session_types::*;

    const SIT_MM: i32 = 800;
    const STAND_MM: i32 = 1200;

    fn feed(m: &mut SessionManager, mm: i32, count: usize) {
        for _ in 0..count {
            let _ = m.on_reading(mm, true);
            if let Some(ts) = m.state.last_accumulate_ts {
                m.state.last_accumulate_ts = Some(ts + Duration::seconds(1));
            }
        }
    }

    fn transition_to(m: &mut SessionManager, state: DeskState, mm: i32) {
        feed(m, mm, DEBOUNCE_COUNT as usize);
        assert_eq!(m.state.state, state, "expected {:?}", state);
    }

    /// Backdates the active timestamps so the manager sees `secs` of elapsed time.
    fn simulate_elapsed(m: &mut SessionManager, secs: i64) {
        let offset = Duration::seconds(secs);
        if let Some(ref mut ts) = m.state.sitting_started { *ts = *ts - offset; }
        if let Some(ref mut ts) = m.state.standing_bout_started { *ts = *ts - offset; }
        if let Some(ref mut ts) = m.state.standing_session_started { *ts = *ts - offset; }
        if let Some(ref mut ts) = m.state.break_started { *ts = *ts - offset; }
        m.state.last_tick_ts = Some(Utc::now());
    }

    /// Sit 30 min → stand 2 min → sit again. The DTO must report
    /// `1800 - 120 * multiplier`, never 0.
    #[test]
    fn e015_sit30_stand2_sit_dto_shows_credited_value() {
        let mut m = SessionManager::new();
        let multiplier = m.state.break_credit_multiplier as f64;

        transition_to(&mut m, DeskState::Sitting, SIT_MM);
        simulate_elapsed(&mut m, 1800);

        transition_to(&mut m, DeskState::Standing, STAND_MM);
        simulate_elapsed(&mut m, 120);

        transition_to(&mut m, DeskState::Sitting, SIT_MM);

        let expected = 1800 - (120.0 * multiplier) as i64;
        let dto = m.snapshot();
        assert_eq!(m.state.last_break_credit, BreakCredit::Partial);
        assert!(
            (dto.limit_used_secs - expected).abs() <= 15,
            "expected ~{} (1800 - 120*{}), got {}",
            expected, multiplier, dto.limit_used_secs
        );
        assert!(
            dto.limit_used_secs > 0,
            "the timer must not reset to 0 on returning to sitting"
        );
    }

    /// A break long enough to cancel the whole session still zeroes the timer —
    /// that is `BreakCredit::Full`, an earned reset, not the deleted one.
    #[test]
    fn e015_long_break_earns_a_full_reset() {
        let mut m = SessionManager::new();

        transition_to(&mut m, DeskState::Sitting, SIT_MM);
        simulate_elapsed(&mut m, 600);

        transition_to(&mut m, DeskState::Standing, STAND_MM);
        simulate_elapsed(&mut m, 900); // 900 * 2.0 = 1800 >= 600

        transition_to(&mut m, DeskState::Sitting, SIT_MM);

        assert_eq!(m.state.last_break_credit, BreakCredit::Full);
        assert!(
            m.snapshot().limit_used_secs <= 10,
            "a full credit resets the timer, got {}",
            m.snapshot().limit_used_secs
        );
    }

    /// The DTO the frontend deserialises carries exactly one session-progress
    /// counter. `secs_since_last_break` exists for the Debug tab and must stay
    /// distinct from it.
    #[test]
    fn e015_dto_json_has_no_second_counter() {
        let m = SessionManager::new();
        let json = serde_json::to_value(m.snapshot()).expect("DTO serialises");
        let obj = json.as_object().expect("DTO is a JSON object");

        assert!(
            !obj.contains_key("current_session_secs"),
            "the second counter must not come back"
        );
        assert!(obj.contains_key("limit_used_secs"), "credited counter missing");
        assert!(
            obj.contains_key("secs_since_last_break"),
            "debug-only diagnostic missing"
        );
    }

    /// `secs_since_last_break` measures time since the last position change and
    /// is unrelated to the credited counter — proving it is a diagnostic, not a
    /// second timer.
    #[test]
    fn e015_secs_since_last_break_tracks_position_change_only() {
        let mut m = SessionManager::new();
        assert_eq!(
            m.snapshot().secs_since_last_break,
            0,
            "no position change recorded yet"
        );

        transition_to(&mut m, DeskState::Sitting, SIT_MM);
        m.state.last_position_change_at = Some(Utc::now() - Duration::seconds(300));
        m.state.sitting_seconds = 42;

        let dto = m.snapshot();
        assert!(
            dto.secs_since_last_break >= 299 && dto.secs_since_last_break <= 301,
            "expected ~300, got {}",
            dto.secs_since_last_break
        );
        assert_eq!(dto.limit_used_secs, 42, "the two counters are independent");
    }
}
