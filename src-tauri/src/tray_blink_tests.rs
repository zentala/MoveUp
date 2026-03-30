//! Unit tests for the TrayBlinker engine.

#[cfg(test)]
mod tests {
    use crate::tray_blink::{BlinkPattern, TrayBlinker};

    fn pattern_2x() -> BlinkPattern {
        BlinkPattern { on_ms: 100, off_ms: 100, count: 2, pause_ms: 500 }
    }

    fn pattern_1x() -> BlinkPattern {
        BlinkPattern { on_ms: 100, off_ms: 100, count: 1, pause_ms: 500 }
    }

    // ── Test 1: new() is inactive, tick returns false ──────────────────────────

    #[test]
    fn new_is_inactive_and_tick_returns_false() {
        let mut b = TrayBlinker::new();
        assert!(!b.is_active());
        assert!(!b.tick(50));
        assert!(!b.tick(1000));
    }

    // ── Test 2: start() activates, first tick returns true (On phase) ──────────

    #[test]
    fn start_activates_and_first_tick_returns_true() {
        let mut b = TrayBlinker::new();
        b.start(pattern_2x());
        assert!(b.is_active());
        // First tick in On phase — dot should be visible
        assert!(b.tick(10));
    }

    // ── Test 3: Full burst cycle: On→Off→On→Off→Pause ─────────────────────────

    #[test]
    fn full_burst_cycle_two_blinks() {
        let mut b = TrayBlinker::new();
        b.start(pattern_2x()); // on=100, off=100, count=2, pause=500

        // — Blink 1: On phase (100ms) —
        assert!(b.tick(50));   // 50ms in On: still on
        assert!(b.tick(50));   // 100ms: transition to Off (returns true for this tick)
        // — Blink 1: Off phase (100ms) —
        assert!(!b.tick(10)); // 10ms in Off: dot hidden
        assert!(!b.tick(90)); // 100ms: transition to On (next blink)
        // — Blink 2: On phase (100ms) —
        assert!(b.tick(50));  // 50ms in On: dot visible
        assert!(b.tick(50));  // 100ms: transition to Off
        // — Blink 2: Off phase (100ms) — burst complete after this
        assert!(!b.tick(50));
        assert!(!b.tick(50)); // 100ms: burst done → transition to Pause
        // — Pause phase (500ms) —
        assert!(!b.tick(100));
        assert!(!b.tick(100));
        assert!(!b.tick(100));
    }

    // ── Test 4: After pause, new burst starts (On phase) ──────────────────────

    #[test]
    fn after_pause_new_burst_starts() {
        let mut b = TrayBlinker::new();
        b.start(pattern_2x()); // on=100, off=100, count=2, pause=500

        // Fast-forward through one full cycle: 2×(100+100) + 500 = 900ms
        // Blink 1 On
        b.tick(100);
        // Blink 1 Off
        b.tick(100);
        // Blink 2 On
        b.tick(100);
        // Blink 2 Off (burst ends)
        b.tick(100);
        // Pause (500ms elapsed → next burst)
        b.tick(500);

        // Now we should be back in On phase — dot visible
        assert!(b.tick(10));
    }

    // ── Test 5: stop() mid-blink → immediately returns false ──────────────────

    #[test]
    fn stop_mid_blink_returns_false() {
        let mut b = TrayBlinker::new();
        b.start(pattern_2x());
        assert!(b.tick(10)); // dot visible
        b.stop();
        assert!(!b.is_active());
        assert!(!b.tick(10));
        assert!(!b.tick(500));
    }

    // ── Test 6: start() replaces active pattern ────────────────────────────────

    #[test]
    fn start_replaces_active_pattern() {
        let mut b = TrayBlinker::new();

        // Start a slow pattern
        b.start(BlinkPattern { on_ms: 1000, off_ms: 1000, count: 5, pause_ms: 5000 });
        b.tick(500); // mid-On phase

        // Replace with fast pattern
        let fast = BlinkPattern { on_ms: 100, off_ms: 100, count: 2, pause_ms: 500 };
        b.start(fast.clone());

        // Should now be following the fast pattern
        assert!(b.is_active());
        // First tick is On
        assert!(b.tick(10));
        // After 100ms total, transitions to Off
        b.tick(90);
        assert!(!b.tick(10));
    }

    // ── Test 7: count=1 pattern: On→Off→Pause→On→... ──────────────────────────

    #[test]
    fn single_blink_per_burst_cycles_correctly() {
        let mut b = TrayBlinker::new();
        b.start(pattern_1x()); // on=100, off=100, count=1, pause=500

        // On phase
        assert!(b.tick(50));
        assert!(b.tick(50)); // 100ms: → Off
        // Off phase (only 1 blink, so burst ends after Off)
        assert!(!b.tick(50));
        assert!(!b.tick(50)); // 100ms: → Pause
        // Pause phase
        assert!(!b.tick(200));
        assert!(!b.tick(300)); // 500ms: → On again
        // New burst starts
        assert!(b.tick(10));
    }

    // ── Test 8: Dot is hidden while in Pause phase ─────────────────────────────

    #[test]
    fn dot_hidden_during_pause() {
        let mut b = TrayBlinker::new();
        b.start(pattern_1x()); // on=100, off=100, count=1, pause=500

        // Complete burst: On + Off
        b.tick(100); // On → Off
        b.tick(100); // Off → Pause

        // Entire pause should return false
        for _ in 0..4 {
            assert!(!b.tick(100));
        }
        // At 400ms of pause, still hidden
        assert!(!b.tick(99));
        // At exactly 500ms, transitions to On — next tick returns true
        b.tick(1); // hits 500ms → transitions to On
        assert!(b.tick(10)); // now in On phase
    }

    // ── Test 9: Large delta spans multiple phase transitions ───────────────────

    #[test]
    fn large_delta_advances_to_pause_correctly() {
        let mut b = TrayBlinker::new();
        b.start(BlinkPattern { on_ms: 100, off_ms: 100, count: 1, pause_ms: 500 });

        // Single massive tick spanning On + Off (200ms total needed to reach Pause)
        // Tick 200ms: should end in Pause or at transition
        b.tick(100); // On → Off
        b.tick(100); // Off → Pause
        // Now in Pause
        assert!(!b.tick(50));
    }

    // ── Test 10: is_active() follows start/stop correctly ──────────────────────

    #[test]
    fn is_active_follows_start_stop() {
        let mut b = TrayBlinker::new();
        assert!(!b.is_active());
        b.start(pattern_2x());
        assert!(b.is_active());
        b.stop();
        assert!(!b.is_active());
        b.start(pattern_2x());
        assert!(b.is_active());
    }
}
