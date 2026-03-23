//! Unit tests for HeightStabilizer.

use super::*;

// ─── mm_to_cm_rounded ────────────────────────────────────────────────────────

#[test]
fn round_721mm_to_72cm() {
    assert_eq!(mm_to_cm_rounded(721.0), 72);
}

#[test]
fn round_725mm_to_73cm() {
    assert_eq!(mm_to_cm_rounded(725.0), 73);
}

#[test]
fn round_719mm_to_72cm() {
    assert_eq!(mm_to_cm_rounded(719.0), 72);
}

#[test]
fn round_zero() {
    assert_eq!(mm_to_cm_rounded(0.0), 0);
}

#[test]
fn round_exact_cm() {
    assert_eq!(mm_to_cm_rounded(720.0), 72);
}

// ─── Moving average ──────────────────────────────────────────────────────────

#[test]
fn average_of_typical_readings() {
    let mut s = HeightStabilizer::new();
    let readings = [720, 721, 719, 720, 722, 720, 719, 721, 720, 720];
    for r in readings {
        s.push(r);
    }
    let avg = s.average_mm().expect("should have average");
    // Sum = 7202, avg = 720.2
    assert!((avg - 720.2).abs() < 0.1, "avg={avg}, expected ~720.2");
}

#[test]
fn average_with_fewer_than_window_size() {
    let mut s = HeightStabilizer::new(); // window=10
    s.push(700);
    s.push(710);
    let avg = s.average_mm().expect("should have average");
    assert!((avg - 705.0).abs() < 0.1);
}

#[test]
fn average_slides_window() {
    let mut s = HeightStabilizer::with_window_size(3);
    s.push(100);
    s.push(200);
    s.push(300);
    // avg = 200
    assert!((s.average_mm().unwrap() - 200.0).abs() < 0.1);
    s.push(600);
    // window: [200, 300, 600], avg = 366.7
    assert!((s.average_mm().unwrap() - 366.7).abs() < 0.2);
}

#[test]
fn empty_stabilizer_returns_none() {
    let s = HeightStabilizer::new();
    assert!(s.average_mm().is_none());
    assert!(s.stabilized_cm().is_none());
}

// ─── stabilized_cm (integration of avg + rounding) ──────────────────────────

#[test]
fn stabilized_cm_from_typical_readings() {
    let mut s = HeightStabilizer::new();
    let readings = [720, 721, 719, 720, 722, 720, 719, 721, 720, 720];
    for r in readings {
        s.push(r);
    }
    // avg ~720.2mm -> 72cm
    assert_eq!(s.stabilized_cm(), Some(72));
}

#[test]
fn stabilized_cm_single_reading() {
    let mut s = HeightStabilizer::new();
    s.push(1085);
    // 1085mm -> 108.5 -> rounds to 109cm
    assert_eq!(s.stabilized_cm(), Some(109));
}

// ─── Trend lock ──────────────────────────────────────────────────────────────

#[test]
fn stable_readings_engage_lock() {
    let mut s = HeightStabilizer::with_window_size(10);
    // Push 10 nearly identical readings to fill buffer and engage lock.
    for _ in 0..10 {
        s.push(720);
    }
    assert!(s.is_locked(), "should be locked after stable readings");
    assert_eq!(s.stabilized_cm(), Some(72));
}

#[test]
fn small_outlier_stays_locked() {
    let mut s = HeightStabilizer::with_window_size(10);
    for _ in 0..10 {
        s.push(720);
    }
    assert!(s.is_locked());

    // Push a reading within unlock tolerance (<=5mm from locked value).
    s.push(723);
    assert!(s.is_locked(), "should stay locked for small deviation");
    // Display should still show locked value (72cm).
    assert_eq!(s.stabilized_cm(), Some(72));
}

#[test]
fn large_deviation_unlocks() {
    let mut s = HeightStabilizer::with_window_size(10);
    for _ in 0..10 {
        s.push(720);
    }
    assert!(s.is_locked());

    // Push a reading that deviates >5mm from locked value.
    s.push(730);
    assert!(!s.is_locked(), "should unlock on >5mm deviation");
}

#[test]
fn relocks_after_new_stable_period() {
    let mut s = HeightStabilizer::with_window_size(5);
    // Lock at ~720mm.
    for _ in 0..6 {
        s.push(720);
    }
    assert!(s.is_locked());

    // Unlock by jumping to 750.
    s.push(750);
    assert!(!s.is_locked());

    // Stabilize at new value: fill buffer fully, then accumulate stable count.
    for _ in 0..12 {
        s.push(750);
    }
    assert!(s.is_locked(), "should relock at new stable value");
    assert_eq!(s.stabilized_cm(), Some(75));
}

// ─── Reset ───────────────────────────────────────────────────────────────────

#[test]
fn reset_clears_state() {
    let mut s = HeightStabilizer::new();
    for _ in 0..10 {
        s.push(720);
    }
    assert!(s.is_locked());

    s.reset();
    assert!(!s.is_locked());
    assert!(s.average_mm().is_none());
    assert!(s.stabilized_cm().is_none());
}

// ─── Edge cases ──────────────────────────────────────────────────────────────

#[test]
fn window_size_one() {
    let mut s = HeightStabilizer::with_window_size(1);
    s.push(721);
    assert_eq!(s.stabilized_cm(), Some(72));
    s.push(735);
    assert_eq!(s.stabilized_cm(), Some(74));
}

#[test]
fn window_size_zero_treated_as_one() {
    let mut s = HeightStabilizer::with_window_size(0);
    s.push(721);
    assert_eq!(s.stabilized_cm(), Some(72));
}
