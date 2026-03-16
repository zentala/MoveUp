# Test Framework Improvements

This document describes the advanced testing features added to ensure code quality and catch regressions early.

## 1. Test Data Builders 🏗️

**Location**: `tests/fixtures/builders.ts`

Builders provide a fluent API for creating consistent test fixtures without repetitive setup.

### Example

```typescript
import { sessionStateBuilder, testScenarios } from "@/fixtures/builders";

it("should alert when sitting at 85%", () => {
  const state = sessionStateBuilder()
    .withState("sitting")
    .withSittingSeconds(2295) // 85% of 2700s limit
    .withSessionLimit(45)
    .build();

  expect(state.sitting_seconds).toBe(2295);
});

// Or use predefined scenarios
const critical = testScenarios.criticalSitting();
const balanced = testScenarios.balancedDay();
```

### Available Builders

- **SessionStateBuilder** — Build session state objects
  - `.withState()`, `.withSittingSeconds()`, `.withStandingSeconds()`
  - `.withSessionLimit()`, `.withPositionChanges()`

- **SettingsBuilder** — Build configuration objects
  - `.withSittingHeight()`, `.withStandingHeight()`
  - `.withNotifications()`, `.withSitLimit()`

- **TodaySummaryBuilder** — Build daily summary objects
  - `.withSittingTime()`, `.withStandingTime()`
  - `.withYesterdayTotals()`, `.addSession()`

### Predefined Test Scenarios

```typescript
testScenarios.halfwaySitting()    // 50% through session
testScenarios.criticalSitting()   // 85% (alert threshold)
testScenarios.shortBreak()        // <5 min standing
testScenarios.mediumBreak()       // 5-9 min standing
testScenarios.longBreak()         // 10+ min standing
testScenarios.walking()           // High desk, no activity
testScenarios.balancedDay()       // 4h sit, 2h stand
testScenarios.overSittingDay()    // 8h sit, 1h stand (poor)
```

---

## 2. Performance Benchmarks ⚡

**Location**: `src-tauri/benches/session_state_machine.rs`

Benchmark critical paths to catch performance regressions before they ship.

### Run Locally

```bash
# Run all benchmarks
cargo bench --bench session_state_machine

# Run specific benchmark
cargo bench --bench session_state_machine session_sitting

# Generate HTML report
cargo bench --bench session_state_machine -- --verbose
# Opens: target/criterion/report/index.html
```

### Benchmarks

| Benchmark | Purpose | Target |
|-----------|---------|--------|
| `on_reading (sitting, consistent)` | Normal sensor reading | < 1ms |
| `on_reading (state change, debounce)` | State transition with debounce | < 2ms |
| `should_alert` | Alert check (called frequently) | < 0.5ms |
| `apply_break_credit` | Break credit calculation | < 1ms |
| `check_daily_reset` | Daily reset check | < 0.5ms |
| `snapshot` | State DTO creation | < 1ms |

### Regression Tracking

Criterion compares against previous runs automatically:

```
on_reading (sitting, consistent)
  time:   [523.45 us 525.67 us 527.93 us]
  change: [-1.24% +0.33% +1.92%] (no change)
```

If performance degrades > 5%, the benchmark will warn you.

---

## 3. Mutation Testing 🧬

**Location**: CI only (runs on PRs)

Mutation testing modifies your code and checks if tests still pass. If they do, you have a problem — your tests aren't thorough enough.

### Example

```rust
// Original code
if sitting_seconds > limit {
  return true;
}

// Mutation 1: > becomes >=
if sitting_seconds >= limit {
  return true;
}
// → If test still passes, mutation wasn't caught! ❌

// Mutation 2: > becomes <
if sitting_seconds < limit {
  return true;
}
// → Test should fail (catch this mutation) ✅
```

### Run Locally

```bash
# Install tool
cargo install cargo-mutants

# Run mutations
cargo mutants --tests

# Output:
# 45 mutations created; 44 caught; 1 survived
# Mutation 23: Uncaught (survived)
#   → Line 543: condition always true
```

### Interpreting Results

```
mutations caught ÷ total mutations = test effectiveness

45 caught / 46 total = 97.8% effective ✅ (goal: >95%)
40 caught / 46 total = 87.0% effective ⚠️  (needs improvement)
```

### In CI/CD

- Runs automatically on all PRs
- Results posted to PR summary
- Uncaught mutations are reported

```
## Mutation Testing Results
- Total mutations: 46
- Caught: 44
- Survived: 2
  - Mutation #12 (Line 543): x > y → x >= y
  - Mutation #31 (Line 987): condition removal
```

---

## 4. Platform-Specific Test Skipping 🖥️

**Location**: `src-tauri/src/activity.rs`

Some tests only make sense on certain platforms.

### Example

```rust
#[test]
#[cfg(windows)]  // Only run on Windows
fn idle_seconds_returns_u64_windows() {
  let secs = get_idle_seconds();
  assert!(secs >= 0);
}

#[test]
#[cfg(not(windows))]  // Run on Linux/Mac
fn idle_seconds_returns_zero_non_windows() {
  let secs = get_idle_seconds();
  assert_eq!(secs, 0); // Not implemented on non-Windows
}
```

### Test Output on Linux

```
test activity::tests::idle_seconds_returns_u64_windows ... skipped
test activity::tests::is_active_consistent_with_idle_seconds_windows ... skipped
test activity::tests::idle_seconds_returns_zero_non_windows ... ok
```

---

## 5. Test Performance Reports 📊

**Location**: CI workflow + local script

Generate reports to identify slow tests.

### Generate Report Locally

```bash
# Using the helper script
./scripts/test-performance.sh

# Output:
# 📊 Running tests with performance metrics...
# 🧪 Running Rust tests...
# ✅ Test run complete!
#
# 📈 Performance Analysis:
# ═════════════════════════
# Slowest Tests (>100ms):
# ─────────────────────────
#   234.5 ms  test session::tests::should_send_praise_halfway_fires_once_per_day
#   156.2 ms  test db::tests::test_aggregate_multiple_sitting_sessions
```

### In CI/CD

- Runs on every push to `main`
- Results stored as artifacts (30 days retention)
- Benchmark results available as: `benchmark-results/`
- Open in browser: `criterion/report/index.html`

### Performance Tips

1. **Tests > 500ms**: Likely need optimization
   - Use mocks instead of real I/O
   - Reduce test data size
   - Parallelize independent tests

2. **Database tests slow?**
   - Use in-memory SQLite (`:memory:`)
   - Create indices on frequently queried columns
   - Batch inserts

3. **State machine tests slow?**
   - Reduce property test iterations
   - Use smaller input ranges

---

## 🎯 Testing Workflow

### Local Development

```bash
# Run unit tests (fast)
pnpm test:unit
cargo test --lib

# Run benchmarks (slow, 1-2 min)
cargo bench --bench session_state_machine

# Check mutations locally (very slow, 5-10 min)
cargo mutants --tests --timeout 10

# Generate performance report
./scripts/test-performance.sh
```

### Before Committing

```bash
pnpm test:all          # Unit + Rust tests
pnpm typecheck         # TypeScript
pnpm lint              # ESLint
```

### On Pull Request

CI will automatically:
- Run unit tests ✅
- Run Rust tests ✅
- Check coverage (80%+ required) ✅
- Run mutation testing ⚠️ (info only)
- Run benchmarks (main branch only) ⚠️

---

## 📈 Metrics & Goals

### Coverage

- **Frontend**: ≥80% (enforced by pre-build gate)
- **Rust**: Currently 74 tests (aim for 80+ as codebase grows)

### Mutation Testing

- **Goal**: >95% of mutations caught
- **Current**: Monitor on PRs (not yet reported)

### Performance

- **All unit tests**: <5s total
- **All benchmarks**: <30s total
- **Mutation testing**: <10 min (CI only)

---

## 🔗 References

- [Criterion.rs](https://bheisler.github.io/criterion.rs/book/)
- [Cargo Mutants](https://mutants.live/)
- [Property Testing with Proptest](https://docs.rs/proptest/)
- [Mutation Testing Theory](https://en.wikipedia.org/wiki/Mutation_testing)

---

## 📝 Examples

### Using Builders in Integration Tests

```typescript
// Before: Repetitive
const state = {
  state: "sitting",
  sitting_seconds: 1800,
  standing_seconds: 0,
  break_seconds: 0,
  session_limit_secs: 2700,
  stand_limit_secs: 0,
  desk_height_cm: 75.0,
  position_changes: 2,
};

// After: Clear intent
const state = sessionStateBuilder()
  .withSittingSeconds(1800)
  .withPositionChanges(2)
  .build();
```

### Interpreting Benchmark Output

```
on_reading (state change, debounce)
  time:   [1.234 ms 1.256 ms 1.280 ms]
  change: [-2.3% +1.2% +4.8%] (within noise)

Verdict: No significant change — regression not detected ✅
```

If change shows >10%:
```
Verdict: Likely increased — investigate! ⚠️
```

---

Last updated: 2026-03-16
