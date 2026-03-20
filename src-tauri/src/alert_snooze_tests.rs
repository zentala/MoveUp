//! alert_snooze_tests.rs — Unit tests for AlertManager snooze + tone shift (T015).
//!
//! Tests cover: deescalating snooze durations, tone shift at dismiss #3,
//! snooze expiry, standing-while-snoozed, progress drop while snoozed.

use crate::alert_actions::{AlertAction, AlertStage};
use crate::alert_manager_tests::{manager, manager_fast, manager_fast_snooze};
use std::time::{Duration, Instant};

// ── T015 Test 1: dismiss #1 → 5 min snooze, snooze_index=1 ──────────────────

#[test]
fn dismiss_first_sets_5min_snooze() {
    let mut m = manager();
    m.tick(1.0); // Stage1
    m.dismiss();
    assert_eq!(m.stage(), AlertStage::Snoozed);
    assert_eq!(m.snooze_index(), 1);
    let until = m.snoozed_until.expect("snoozed_until must be set");
    let remaining = until.duration_since(Instant::now());
    assert!(remaining <= Duration::from_secs(5 * 60 + 1));
    assert!(remaining >= Duration::from_secs(5 * 60 - 1));
}

// ── T015 Test 2: dismiss #2 → 15 min snooze, snooze_index=2 ─────────────────

#[test]
fn dismiss_second_sets_15min_snooze() {
    let mut m = manager();
    m.tick(1.0); // Stage1
    m.dismiss(); // snooze_index → 1, 5 min
    m.stage = AlertStage::Stage2;
    m.dismiss(); // snooze_index → 2, 15 min
    assert_eq!(m.snooze_index(), 2);
    let until = m.snoozed_until.expect("snoozed_until must be set");
    let remaining = until.duration_since(Instant::now());
    assert!(remaining <= Duration::from_secs(15 * 60 + 1));
    assert!(remaining >= Duration::from_secs(15 * 60 - 1));
}

// ── T015 Test 3: dismiss #3 → 30 min snooze, snooze_index=3 ─────────────────

#[test]
fn dismiss_third_sets_30min_snooze() {
    let mut m = manager();
    m.tick(1.0);
    m.dismiss(); // index 0 → 5 min, snooze_index=1
    m.stage = AlertStage::Stage2;
    m.dismiss(); // index 1 → 15 min, snooze_index=2
    m.stage = AlertStage::Stage2;
    m.dismiss(); // index 2 → 30 min, snooze_index=3
    assert_eq!(m.snooze_index(), 3);
    let until = m.snoozed_until.expect("snoozed_until must be set");
    let remaining = until.duration_since(Instant::now());
    assert!(remaining <= Duration::from_secs(30 * 60 + 1));
    assert!(remaining >= Duration::from_secs(30 * 60 - 1));
}

// ── T015 Test 4: dismiss #4 → 60 min snooze (capped), snooze_index=4 ────────

#[test]
fn dismiss_fourth_sets_60min_snooze_capped() {
    let mut m = manager();
    m.tick(1.0);
    m.dismiss(); // index 0 → 5 min
    m.stage = AlertStage::Stage2;
    m.dismiss(); // index 1 → 15 min
    m.stage = AlertStage::Stage2;
    m.dismiss(); // index 2 → 30 min
    m.stage = AlertStage::Stage2;
    m.dismiss(); // index 3 → 60 min (last entry)
    assert_eq!(m.snooze_index(), 4);
    let until = m.snoozed_until.expect("snoozed_until must be set");
    let remaining = until.duration_since(Instant::now());
    assert!(remaining <= Duration::from_secs(60 * 60 + 1));
    assert!(remaining >= Duration::from_secs(60 * 60 - 1));
}

// ── T015 Test 5: standing while Snoozed → Idle, snooze_index=0 ───────────────

#[test]
fn standing_while_snoozed_resets_snooze_index() {
    let mut m = manager();
    m.tick(1.0); // Stage1
    m.dismiss(); // Snoozed, snooze_index=1
    assert_eq!(m.stage(), AlertStage::Snoozed);
    assert_eq!(m.snooze_index(), 1);
    assert!(m.snoozed_until.is_some());

    let actions = m.on_standing();
    assert_eq!(m.stage(), AlertStage::Idle);
    assert_eq!(m.snooze_index(), 0);
    assert!(m.snoozed_until.is_none());
    assert!(actions.contains(&AlertAction::StopPulse));
    assert!(actions.contains(&AlertAction::DismissPopup));
}

// ── T015 Test 6: progress < 1.0 while Snoozed → Idle (cancel snooze) ────────

#[test]
fn progress_drops_while_snoozed_enters_idle() {
    let mut m = manager();
    m.tick(1.0); // Stage1
    m.dismiss(); // Snoozed
    assert_eq!(m.stage(), AlertStage::Snoozed);

    let actions = m.tick(0.5); // progress below threshold
    assert_eq!(m.stage(), AlertStage::Idle);
    assert!(actions.contains(&AlertAction::StopPulse));
    // DismissPopup not expected — popup was already dismissed on dismiss()
    assert!(!actions.contains(&AlertAction::DismissPopup));
}

// ── T015 Test 7: dismiss returns [StopPulse, DismissPopup]; expiry → PulseBar

#[test]
fn dismiss_actions_and_snooze_expiry_emits_pulse_bar() {
    let mut m = manager_fast_snooze();
    m.tick(1.0); // Stage1
    m.stage_entered_at = Instant::now() - Duration::from_secs(1);
    m.tick(1.0); // Stage2

    let dismiss_actions = m.dismiss();
    assert_eq!(
        dismiss_actions,
        vec![AlertAction::StopPulse, AlertAction::DismissPopup]
    );
    assert_eq!(m.stage(), AlertStage::Snoozed);

    std::thread::sleep(Duration::from_millis(5));

    let actions = m.tick(1.0);
    assert_eq!(m.stage(), AlertStage::Stage1);
    assert_eq!(actions, vec![AlertAction::PulseBar]);
}

// ── T015 Test 8: snooze expiry → Stage1 (not Stage2 directly) ────────────────

#[test]
fn snooze_expiry_enters_stage1_not_stage2() {
    let mut m = manager_fast_snooze();
    m.tick(1.0); // Stage1
    m.stage_entered_at = Instant::now() - Duration::from_secs(1);
    m.tick(1.0); // Stage2
    m.dismiss(); // Snoozed

    std::thread::sleep(Duration::from_millis(5));

    let actions = m.tick(1.0);
    assert_eq!(m.stage(), AlertStage::Stage1);
    assert!(actions.contains(&AlertAction::PulseBar));
    assert!(!actions.iter().any(|a| matches!(a, AlertAction::ShowPopup(_))));
}

// ── T015 Test bonus: tone shift — neutral before #3, positive at #3+ ──────────

fn extract_popup_msg(actions: &[AlertAction]) -> String {
    actions
        .iter()
        .find_map(|a| {
            if let AlertAction::ShowPopup(s) = a {
                Some(s.clone())
            } else {
                None
            }
        })
        .expect("ShowPopup must be present")
}

#[test]
fn popup_message_tone_shifts_at_threshold() {
    let mut m = manager_fast();
    let neutral_msgs: Vec<String> = m.config.neutral_messages.clone();
    let positive_msgs: Vec<String> = m.config.positive_messages.clone();

    m.tick(1.0); // Stage1
    m.stage_entered_at = Instant::now() - Duration::from_secs(1);
    let actions = m.tick(1.0); // Stage2, snooze_index=0 → neutral
    let msg = extract_popup_msg(&actions);
    assert!(neutral_msgs.iter().any(|n| n == &msg), "expected neutral message, got: {msg}");

    m.dismiss();
    m.stage = AlertStage::Stage1;
    m.stage_entered_at = Instant::now() - Duration::from_secs(1);
    let actions2 = m.tick(1.0); // Stage2, snooze_index=1
    let msg2 = extract_popup_msg(&actions2);
    assert!(neutral_msgs.iter().any(|n| n == &msg2), "expected neutral at index 1, got: {msg2}");

    m.dismiss();
    m.stage = AlertStage::Stage1;
    m.stage_entered_at = Instant::now() - Duration::from_secs(1);
    let actions3 = m.tick(1.0); // Stage2, snooze_index=2
    let msg3 = extract_popup_msg(&actions3);
    assert!(neutral_msgs.iter().any(|n| n == &msg3), "expected neutral at index 2, got: {msg3}");

    m.dismiss();
    m.stage = AlertStage::Stage1;
    m.stage_entered_at = Instant::now() - Duration::from_secs(1);
    let actions4 = m.tick(1.0); // Stage2, snooze_index=3 = tone_shift_threshold → positive
    let msg4 = extract_popup_msg(&actions4);
    assert!(positive_msgs.iter().any(|p| p == &msg4), "expected positive after tone shift, got: {msg4}");
}
