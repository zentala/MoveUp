//! alert_manager_tests.rs — Unit tests for AlertManager basic escalation (T013+T014).
//!
//! Tests 1-10: idle→stage1→stage2 transitions, on_standing reset, progress oscillation.
//! Snooze tests (T015) live in alert_snooze_tests.rs.

use crate::alert_actions::{AlertAction, AlertStage};
use crate::alert_config::AlertConfig;
use crate::alert_manager::AlertManager;
use std::time::{Duration, Instant};

pub(crate) fn manager() -> AlertManager {
    AlertManager::new(AlertConfig::default())
}

pub(crate) fn manager_fast() -> AlertManager {
    AlertManager::new(AlertConfig {
        stage2_delay_secs: 0, // Instant Stage2 for tests
        ..AlertConfig::default()
    })
}

pub(crate) fn manager_fast_snooze() -> AlertManager {
    AlertManager::new(AlertConfig {
        stage2_delay_secs: 0,
        snooze_durations: vec![
            Duration::from_millis(1),
            Duration::from_millis(1),
            Duration::from_millis(1),
            Duration::from_millis(1),
        ],
        ..AlertConfig::default()
    })
}

// ── Test 1: progress < 1.0 stays Idle ─────────────────────────────────────────

#[test]
fn idle_below_threshold_no_actions() {
    let mut m = manager();
    let actions = m.tick(0.5);
    assert!(actions.is_empty());
    assert_eq!(m.stage(), AlertStage::Idle);
}

#[test]
fn idle_at_99pct_no_actions() {
    let mut m = manager();
    let actions = m.tick(0.99);
    assert!(actions.is_empty());
    assert_eq!(m.stage(), AlertStage::Idle);
}

// ── Test 2: progress >= 1.0 → Stage1, emits PulseBar ─────────────────────────

#[test]
fn idle_at_threshold_enters_stage1() {
    let mut m = manager();
    let actions = m.tick(1.0);
    assert_eq!(actions, vec![AlertAction::PulseBar]);
    assert_eq!(m.stage(), AlertStage::Stage1);
}

#[test]
fn idle_above_threshold_enters_stage1() {
    let mut m = manager();
    let actions = m.tick(1.2);
    assert_eq!(actions, vec![AlertAction::PulseBar]);
    assert_eq!(m.stage(), AlertStage::Stage1);
}

// ── Test 3: Stage1 for < 2 min → stays Stage1, no new actions ────────────────

#[test]
fn stage1_before_delay_no_actions() {
    let mut m = manager();
    m.tick(1.0); // enter Stage1
    let actions = m.tick(1.0); // still in Stage1, not enough time passed
    assert!(actions.is_empty());
    assert_eq!(m.stage(), AlertStage::Stage1);
}

// ── Test 4: Stage1 for >= 2 min → Stage2, emits ShowPopup ────────────────────

#[test]
fn stage1_after_delay_enters_stage2() {
    let mut m = manager_fast(); // stage2_delay_secs = 0
    m.tick(1.0); // enter Stage1
    m.stage_entered_at = Instant::now() - Duration::from_secs(1);
    let actions = m.tick(1.0);
    assert_eq!(m.stage(), AlertStage::Stage2);
    assert!(actions.iter().any(|a| matches!(a, AlertAction::ShowPopup(_))));
}

// ── Test 5: on_standing from Stage1 → Idle, emits StopPulse ─────────────────

#[test]
fn on_standing_from_stage1_resets() {
    let mut m = manager();
    m.tick(1.0); // enter Stage1
    let actions = m.on_standing();
    assert!(actions.contains(&AlertAction::StopPulse));
    assert_eq!(m.stage(), AlertStage::Idle);
}

// ── Test 6: on_standing from Stage2 → Idle, emits StopPulse + DismissPopup ──

#[test]
fn on_standing_from_stage2_dismisses_popup() {
    let mut m = manager_fast();
    m.tick(1.0); // Stage1
    m.stage_entered_at = Instant::now() - Duration::from_secs(1);
    m.tick(1.0); // Stage2
    assert_eq!(m.stage(), AlertStage::Stage2);

    let actions = m.on_standing();
    assert!(actions.contains(&AlertAction::StopPulse));
    assert!(actions.contains(&AlertAction::DismissPopup));
    assert_eq!(m.stage(), AlertStage::Idle);
}

// ── Test 7: dismiss → Snoozed; next tick after expiry → Stage1 ───────────────

#[test]
fn dismiss_then_tick_reenters_stage1_after_snooze_expiry() {
    let mut m = manager_fast_snooze();
    m.tick(1.0); // Stage1
    m.stage_entered_at = Instant::now() - Duration::from_secs(1);
    m.tick(1.0); // Stage2

    let dismiss_actions = m.dismiss();
    assert!(dismiss_actions.contains(&AlertAction::StopPulse));
    assert!(dismiss_actions.contains(&AlertAction::DismissPopup));
    assert_eq!(m.stage(), AlertStage::Snoozed);
    assert_eq!(m.snooze_index(), 1);

    std::thread::sleep(Duration::from_millis(5));

    let actions = m.tick(1.0);
    assert_eq!(m.stage(), AlertStage::Stage1);
    assert!(actions.contains(&AlertAction::PulseBar));
}

// ── Test 8: progress < 1.0 while Stage2 → Idle, emits StopPulse + Dismiss ───

#[test]
fn stage2_progress_drops_enters_idle() {
    let mut m = manager_fast();
    m.tick(1.0); // Stage1
    m.stage_entered_at = Instant::now() - Duration::from_secs(1);
    m.tick(1.0); // Stage2
    assert_eq!(m.stage(), AlertStage::Stage2);

    let actions = m.tick(0.5); // progress drops
    assert!(actions.contains(&AlertAction::StopPulse));
    assert!(actions.contains(&AlertAction::DismissPopup));
    assert_eq!(m.stage(), AlertStage::Idle);
}

// ── Test 9: progress oscillation 0.99→1.01→0.99 stays correct state ──────────

#[test]
fn progress_oscillation_correct_states() {
    let mut m = manager();
    assert!(m.tick(0.99).is_empty()); // Idle
    m.tick(1.01); // Stage1
    assert_eq!(m.stage(), AlertStage::Stage1);

    let drop_actions = m.tick(0.99); // back below threshold
    assert!(drop_actions.contains(&AlertAction::StopPulse));
    assert_eq!(m.stage(), AlertStage::Idle);
}

// ── Test 10: rapid sit/stand/sit ends in correct state ────────────────────────

#[test]
fn rapid_sit_stand_sit_ends_in_stage1() {
    let mut m = manager();
    m.tick(1.0); // Sitting → Stage1
    m.on_standing(); // Standing → Idle
    m.tick(1.0); // Sitting again → Stage1
    assert_eq!(m.stage(), AlertStage::Stage1);
}
