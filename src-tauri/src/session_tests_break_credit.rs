//! session_tests_break_credit.rs — Tests for proportional break credit.
//!
//! Every reading is injected at an explicit instant (E020-T01). The old form
//! backdated `break_started` against one `Utc::now()` and then let `on_reading`
//! read a second, slightly later one, so the break duration these tests
//! asserted was "at least N seconds" rather than exactly N.

#[cfg(test)]
mod break_credit_tests {
    use chrono::{DateTime, Duration, TimeZone, Utc};

    use crate::session_manager::SessionManager;
    use crate::session_types::*;

    /// A fixed instant, so nothing here depends on when the suite runs.
    fn base() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 1, 9, 0, 0).unwrap()
    }

    /// Feeds coherent readings over `DEBOUNCE_COUNT` elapsed seconds and
    /// returns the instant after the endpoint sample.
    fn feed(
        m: &mut SessionManager,
        mm: i32,
        at: DateTime<Utc>,
    ) -> (DateTime<Utc>, ReadingResult) {
        let mut result = ReadingResult {
            state_change: None,
            completed_session: None,
            break_credit: None,
        };
        for i in 0..=DEBOUNCE_COUNT as i64 {
            result = m.on_reading_at(mm, true, at + Duration::seconds(i));
        }
        (at + Duration::seconds(DEBOUNCE_COUNT as i64 + 1), result)
    }

    fn transition_to_sitting(m: &mut SessionManager, at: DateTime<Utc>) -> DateTime<Utc> {
        let (next, _) = feed(m, 800, at);
        assert_eq!(m.state.state, DeskState::Sitting);
        next
    }

    fn transition_to_standing(m: &mut SessionManager, at: DateTime<Utc>) -> DateTime<Utc> {
        let (next, _) = feed(m, 1200, at);
        assert_eq!(m.state.state, DeskState::Standing);
        next
    }

    /// Holds `mm` for `secs` starting at `from`, ticking once a minute.
    ///
    /// The minute cadence matters: a single jump longer than
    /// `SLEEP_GAP_THRESHOLD_SECS` would make the engine treat the interval as
    /// machine sleep and apply break credit for it, which is a different code
    /// path (see `session_tests_sleep.rs`).
    fn hold(m: &mut SessionManager, mm: i32, from: DateTime<Utc>, secs: i64) -> DateTime<Utc> {
        let end = from + Duration::seconds(secs);
        let mut t = from;
        while t + Duration::seconds(60) < end {
            t += Duration::seconds(60);
            let _ = m.on_reading_at(mm, true, t);
        }
        end
    }

    /// Returns to sitting so that the break started at `break_started` lasts
    /// exactly `break_secs`. The confirming reading is the last of the batch.
    fn sit_back_after(
        m: &mut SessionManager,
        break_started: DateTime<Utc>,
        break_secs: i64,
    ) -> ReadingResult {
        let lead = DEBOUNCE_COUNT as i64;
        let batch_start = hold(m, 1200, break_started, break_secs - lead);
        let (_, result) = feed(m, 800, batch_start);
        result
    }

    #[test]
    fn break_credit_full_after_long_break() {
        // 20 min break with multiplier 2.0 = 40 min credit, cancelling 40 min
        // of sitting exactly → Full reset.
        let mut m = SessionManager::new();
        let t = transition_to_sitting(&mut m, base());
        let t = transition_to_standing(&mut m, t);
        // Seeded after the transition: handle_state_exit commits the
        // elapsed sitting bout, which would otherwise add to this.
        m.state.sitting_seconds = 2400; // 40 min sitting

        m.state.break_started = Some(t);
        let result = sit_back_after(&mut m, t, 20 * 60);

        let (credit, dur) = result.break_credit
            .expect("should carry break_credit after long break");
        assert_eq!(credit, BreakCredit::Full);
        assert_eq!(dur, 20 * 60, "break must be exactly 20 minutes");
        assert_eq!(m.state.sitting_seconds, 0, "sitting should be reset to 0");
    }

    #[test]
    fn break_credit_none_for_very_short_break() {
        // Break < 1 min (BREAK_MIN_SECS) → no credit
        let mut m = SessionManager::new();
        let t = transition_to_sitting(&mut m, base());
        let t = transition_to_standing(&mut m, t);
        // Seeded after the transition: handle_state_exit commits the
        // elapsed sitting bout, which would otherwise add to this.
        m.state.sitting_seconds = 1200;

        m.state.break_started = Some(t);
        let result = sit_back_after(&mut m, t, 30);

        assert!(
            result.break_credit.is_none(),
            "Break < 1 min should NOT set break_credit"
        );
        let fact = m.state.last_break_credit_facts.as_ref().unwrap().last().unwrap();
        assert_eq!(fact.break_started_at, t.to_rfc3339());
        assert_eq!(fact.observed_break_secs, 30);
        assert_eq!(fact.minimum_break_secs, 60);
        assert_eq!(fact.applied_at, (t + Duration::seconds(30)).to_rfc3339());
        assert_eq!(fact.applied_credit_secs, 0);
        assert_eq!(fact.no_credit_reason.as_deref(), Some("break_below_minimum"));
    }

    #[test]
    fn unqualified_credit_application_does_not_leave_stale_source_facts() {
        let mut m = SessionManager::new();
        m.state.last_break_credit_facts = Some(vec![LastBreakCreditFacts {
            break_started_at: "2026-06-01T08:00:00+00:00".to_string(),
            observed_break_secs: 300,
            minimum_break_secs: 60,
            multiplier: 2.0,
            requested_credit_secs: 600,
            prior_applied_credit_secs: 0,
            sitting_before_secs: 600,
            applied_credit_secs: 600,
            sitting_after_secs: 0,
            applied_at: "2026-06-01T08:05:00+00:00".to_string(),
            no_credit_reason: None,
        }]);

        m.apply_break_credit(300);

        assert!(m.state.last_break_credit_facts.is_none());
    }

    #[test]
    fn break_credit_partial_proportional() {
        // 5 min break with multiplier 2.0 = 10 min (600s) credit.
        // 40 min sitting - 10 min credit = 30 min remaining → Partial
        let mut m = SessionManager::new();
        let t = transition_to_sitting(&mut m, base());
        let t = transition_to_standing(&mut m, t);
        // Seeded after the transition: handle_state_exit commits the
        // elapsed sitting bout, which would otherwise add to this.
        m.state.sitting_seconds = 2400; // 40 min sitting

        m.state.break_started = Some(t);
        let result = sit_back_after(&mut m, t, 5 * 60);

        let (credit, dur) = result.break_credit
            .expect("should carry break_credit for 5 min break");
        assert_eq!(credit, BreakCredit::Partial);
        assert_eq!(dur, 300, "break must be exactly 5 minutes");
        // 2400 - (300 * 2.0) = 2400 - 600 = 1800
        assert_eq!(m.state.sitting_seconds, 1800, "sitting should be 1800s (30 min)");
    }

    #[test]
    fn completed_break_snapshot_contains_actual_capped_credit_facts() {
        let mut m = SessionManager::new();
        let t = transition_to_sitting(&mut m, base());
        let t = transition_to_standing(&mut m, t);
        m.state.sitting_seconds = 480;
        m.state.break_started = Some(t);

        let observed_break_secs = 300;
        let applied_at = t + Duration::seconds(observed_break_secs);
        let result = sit_back_after(&mut m, t, observed_break_secs);
        assert_eq!(result.break_credit, Some((BreakCredit::Full, observed_break_secs)));
        assert_eq!(m.state.sitting_seconds, 0);

        let snapshot = serde_json::to_value(m.snapshot_at(applied_at)).unwrap();
        assert_eq!(
            snapshot["last_break_credit_facts"],
            serde_json::json!([{
                "break_started_at": t.to_rfc3339(),
                "observed_break_secs": 300,
                "minimum_break_secs": 60,
                "multiplier": 2.0,
                "requested_credit_secs": 600,
                "prior_applied_credit_secs": 0,
                "sitting_before_secs": 480,
                "applied_credit_secs": 480,
                "sitting_after_secs": 0,
                "applied_at": applied_at.to_rfc3339(),
                "no_credit_reason": null
            }]),
            "snapshot must expose facts from the engine's actual capped application"
        );
    }

    #[test]
    fn break_credit_10min_cancels_20min_sitting() {
        // 10 min break * 2.0 = 20 min credit
        let mut m = SessionManager::new();
        let t = transition_to_sitting(&mut m, base());
        let t = transition_to_standing(&mut m, t);
        // Seeded after the transition: handle_state_exit commits the
        // elapsed sitting bout, which would otherwise add to this.
        m.state.sitting_seconds = 2400; // 40 min

        m.state.break_started = Some(t);
        let _ = sit_back_after(&mut m, t, 10 * 60);

        // 2400 - (600 * 2.0) = 2400 - 1200 = 1200 (20 min)
        assert_eq!(m.state.sitting_seconds, 1200);
    }

    #[test]
    fn break_credit_2h_away_resets_fully() {
        // 2 hour break * 2.0 = 4 hours credit → always Full reset
        let mut m = SessionManager::new();
        let t = transition_to_sitting(&mut m, base());
        let t = transition_to_standing(&mut m, t);
        // Seeded after the transition: handle_state_exit commits the
        // elapsed sitting bout, which would otherwise add to this.
        m.state.sitting_seconds = 2400; // 40 min

        m.state.break_started = Some(t);
        let _ = sit_back_after(&mut m, t, 2 * 60 * 60);

        assert_eq!(m.state.sitting_seconds, 0, "2h break should fully reset sitting");
    }
}
