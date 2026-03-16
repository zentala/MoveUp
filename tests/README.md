# Testing Guide — Desk App

Complete test coverage across 3 layers: unit, integration, E2E.

## Quick Start

```bash
# Run all tests (unit + Rust)
pnpm test:all

# Or individually:
pnpm test:unit          # Frontend unit tests (Vitest)
pnpm test:unit:rust     # Rust unit tests (cargo)
pnpm test:integration   # Integration tests (Vitest + node)
pnpm test:e2e           # E2E tests (Playwright)
```

## Layer 1: Unit Tests

### Frontend (Vitest + jsdom)

**Location**: `src/**/*.test.tsx`

```bash
pnpm test:unit                # Run once
pnpm test:unit:watch          # Watch mode
pnpm test:coverage            # With coverage report
```

**Examples**:
- `src/components/ProgressBar.test.tsx` — renders progress, updates on state change
- `src/hooks/useDesk.test.ts` — mocked Tauri API, state synchronization
- `src/utils/format.test.ts` — duration formatting edge cases

### Rust (#[test])

**Location**: `src-tauri/src/session.rs` (85+ test cases), `src-tauri/src/db.rs` (5+ tests)

Key test suites:
- **Break credit**: <5min, 5-9min, ≥10min scenarios
- **Debounce**: requires 5 consistent readings for state transition
- **Alerts**: fires once, suppressed until break ends
- **Daily reset**: counters reset at midnight, throttled to 60s checks
- **Standing accumulation**: only in Standing state, not Walking

```bash
cd src-tauri
cargo test                          # All tests
cargo test --lib session            # Session tests only
cargo test break_credit             # Specific test name
cargo test -- --nocapture          # Show println! output
```

## Layer 2: Integration Tests

**Requires**: Tauri app running in dev mode

### Setup

Terminal 1 — Start the app:
```bash
pnpm tauri:dev
```

Terminal 2 — Run integration tests:
```bash
pnpm test:integration
```

### What They Test

Each test uses `inject_reading(mm, active)` to simulate sensor readings and verifies app state changes via IPC.

| Test | Scenario | Verification |
|------|----------|--------------|
| `basic-sitting-session` | Send sitting readings (750mm, active=true) | state=Sitting |
| `standing-long-break` | Sit → Stand for 10+min → Sit | sitting_seconds reset |
| `standing-short-break` | Sit → Stand 5-9min → Sit | sitting_seconds -= 1200s |
| `walking-no-credit` | Stand + inactive (active=false) | state=Walking, standing_seconds unchanged |
| `debounce-5-readings` | Send 4 readings, then 5th | transition only at 5th |
| `position-changes` | Sit↔Stand↔Sit transitions | position_changes += 2 |
| `daily-reset` | (mocked date change) | counters = 0 |
| `device-reconnect` | Start/stop auto-connect | events: connected/lost |

**Example test**:
```typescript
// tests/integration/session-flow.test.ts
await sendRepeated(SITTING_DISTANCE_MM, DEBOUNCE_COUNT + 2, true, 100);
const state = await invoke("get_session_state");
expect(state.state).toBe("sitting");
```

### Emulator Helpers

**DeskDeviceEmulator.ts**:
- `formatDistance(mm)` → "distance: 750 mm\n"
- `generateReadings(mm, count)` → array of formatted strings

**scenarios.ts**:
- `SITTING_DISTANCE_MM = 750`
- `STANDING_DISTANCE_MM = 1080`
- `sendRepeated(mm, count, active)` → injects N readings
- `simulateSession(config)` → sit → stand → sit flow

## Layer 3: E2E Tests

**Requires**: `@playwright/test`, `tauri-driver`

### Setup

Install Tauri driver (one-time):
```bash
cargo install tauri-driver
```

Terminal 1 — Start WebDriver mode:
```bash
tauri driver --compatibility-mode
```

Terminal 2 — Run tests:
```bash
pnpm test:e2e              # Headless
pnpm test:e2e:ui           # Interactive UI
```

### Test Structure

**app-startup.test.ts**:
- Main window loads and is visible
- Connection status indicator present
- Settings button opens panel

**session-progress.test.ts**:
- Inject readings via `invokeCommand` helper
- UI displays "Sitting" / "Standing" state
- Progress bar visible during sitting
- Session timer updates over time
- Break indicator appears on standing

**test-helpers.ts**:
- `invokeCommand(page, "inject_reading", {mm, active})` — execute Tauri command from test
- `waitForEvent(page, "desk:state-changed", timeout)` — listen for emitted events
- `sendReadings(page, mm, count)` — helper for multi-reading sequences

## Coverage Requirements

Minimum **80%** coverage enforced by:
```typescript
// vite.config.ts
coverage: {
  thresholds: {
    lines: 80,
    functions: 80,
    branches: 75,
  }
}
```

Check coverage:
```bash
pnpm test:coverage
# Opens coverage/index.html in browser
```

## Pre-Build Validation

Tests run automatically before building:

```bash
pnpm build         # Runs: test:all → tsc → vite build
pnpm tauri:build   # Runs: test:all → tauri build
```

If any test fails, the build stops. Fix tests and retry.

## Debugging Tests

### Print debugging in tests
```typescript
console.log("State:", state);  // Visible with: cargo test -- --nocapture
```

### Run a single test
```bash
# Rust
cargo test test_name -- --exact

# TypeScript
pnpm test:unit -- session-flow.test.ts
```

### Watch mode (auto-rerun on file change)
```bash
pnpm test:unit:watch
vitest --config vitest.config.ts tests/integration --watch
```

### View test output interactively
```bash
pnpm test:e2e:ui    # Opens Playwright UI with video recordings
```

## Writing New Tests

### Rust Unit Test Template
```rust
#[test]
fn test_new_scenario() {
    let mut m = SessionManager::new();
    // Arrange
    m.state.sitting_seconds = 3000;

    // Act
    m.apply_break_credit(7 * 60);  // 7-minute break

    // Assert
    assert_eq!(m.state.sitting_seconds, 3000 - 1200);
}
```

### TypeScript Integration Test Template
```typescript
it("scenario-name: description", async () => {
  // Inject readings
  await sendRepeated(SITTING_DISTANCE_MM, 6, true, 100);

  // Query state
  const state = await invoke("get_session_state");

  // Assert
  expect(state.state).toBe("sitting");
});
```

### Playwright E2E Test Template
```typescript
test("ui-interaction: description", async ({ page }) => {
  await page.goto("http://localhost:1443");
  await page.waitForLoadState("networkidle");

  // Inject via command
  await invokeCommand(page, "inject_reading", { mm: 750, active: true });

  // Query UI
  const element = page.locator("[data-testid=state-display]");
  await expect(element).toContainText("Sitting");
});
```

## Troubleshooting

**"inject_reading not found" in dev mode**:
- Ensure app is running in debug mode (`pnpm tauri:dev`)
- Command is cfg-gated to debug builds only

**Integration tests timeout**:
- Check that `pnpm tauri:dev` is still running in another terminal
- Increase timeout: `const state = await invoke(...).catch(e => { /* retry */ })`

**Playwright WebDriver mode won't start**:
- Ensure `tauri-driver --compatibility-mode` is running
- Check that port 4444 is not in use

**Coverage below 80%**:
- Run `pnpm test:coverage` to find untested functions
- Add tests for missing branches/functions
- Commit the coverage badge to git
