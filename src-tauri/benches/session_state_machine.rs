/**
 * session_state_machine.rs — Benchmarks for session state machine.
 *
 * Run with: cargo bench --bench session_state_machine
 * Results: target/criterion/
 */

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use desk_lib::session::SessionManager;

fn bench_on_reading_sitting(c: &mut Criterion) {
    let mut m = SessionManager::new();

    c.bench_function("on_reading (sitting, consistent)", |b| {
        b.iter(|| {
            let mm = black_box(800);
            m.on_reading(mm, black_box(true))
        })
    });
}

fn bench_on_reading_state_change(c: &mut Criterion) {
    let mut m = SessionManager::new();
    let mut counter = 0;

    c.bench_function("on_reading (state change, debounce)", |b| {
        b.iter(|| {
            counter += 1;
            let mm = if counter % 10 < 5 { 800 } else { 1200 };
            m.on_reading(black_box(mm), black_box(true))
        })
    });
}

fn bench_should_alert(c: &mut Criterion) {
    let mut m = SessionManager::new();
    m.state.sitting_seconds = 2750; // Just over limit
    m.state.state = desk_lib::session::DeskState::Sitting;

    c.bench_function("should_alert", |b| {
        b.iter(|| m.should_alert())
    });
}

fn bench_apply_break_credit(c: &mut Criterion) {
    let mut m = SessionManager::new();
    m.state.sitting_seconds = 3000;

    c.bench_function("apply_break_credit (7 min break)", |b| {
        b.iter(|| {
            m.apply_break_credit(black_box(420));
        })
    });
}

fn bench_check_daily_reset(c: &mut Criterion) {
    let mut m = SessionManager::new();
    m.state.sitting_seconds = 5000;

    c.bench_function("check_daily_reset (same day)", |b| {
        b.iter(|| m.check_daily_reset())
    });
}

fn bench_snapshot(c: &mut Criterion) {
    let m = SessionManager::new();

    c.bench_function("snapshot (state DTO)", |b| {
        b.iter(|| m.snapshot())
    });
}

criterion_group!(
    benches,
    bench_on_reading_sitting,
    bench_on_reading_state_change,
    bench_should_alert,
    bench_apply_break_credit,
    bench_check_daily_reset,
    bench_snapshot
);

criterion_main!(benches);
