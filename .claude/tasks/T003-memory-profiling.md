# T003: Memory Profiling Test

**Task ID:** T003
**Effort:** 4 hours
**Priority:** P0 (needs T001, blocks T005, T006)
**Owner:** Subagent

---

## Summary

Implement `tests/perf/memory-profile.test.ts` that measures runtime memory footprint via Tauri WebDriver, validates data, and saves baseline.

---

## Acceptance Criteria

### 1. tests/perf/memory-profile.test.ts (~200 lines)

**Key features:**
- Platform check: Windows-only (taskkill available)
- Kill existing zntlDesk processes before + after test
- Launch app via Tauri WebDriver
- Sample memory (RSS) every 500ms for 10s
- Validate samples: min 100MB, no NaN
- Record peak, stable (after 5s), trend
- Save to `.perf-baseline.json`
- Fail if peak >300MB or growth >20MB vs baseline
- Explicit timeout: 60s + diagnostics

**Stub:**

```typescript
import { describe, it, expect, beforeEach, afterEach, test } from 'vitest';
import { exec } from 'child_process';
import * as fs from 'fs';

const BASELINE_FILE = '.perf-baseline.json';

describe('Memory Profiling', () => {
  beforeEach(() => {
    // Kill existing zntlDesk (Windows: taskkill /IM zntlDesk.exe /F)
    killExistingApp();
  });

  afterEach(() => {
    // Cleanup
    killExistingApp();
  });

  test('Memory profile: app startup + idle', {
    timeout: 60000, // 60 seconds
  }, async () => {
    // 1. Launch app via WebDriver
    // 2. Sample memory every 500ms for 10s
    // 3. Validate samples
    // 4. Record: peak, stable, trend
    // 5. Compare vs baseline
    // 6. Fail if exceeded thresholds
    // 7. Save to .perf-baseline.json
  });
});

function killExistingApp() {
  // Windows: taskkill /IM zntlDesk.exe /F
  // Handle error gracefully (app not running is OK)
}
```

### 2. .perf-baseline.json Format

```json
{
  "timestamp": "2026-03-16T10:30:00.000Z",
  "peak_mb": 245,
  "stable_mb": 180,
  "trend": "stable",
  "samples_count": 20,
  "comment": "Samples taken every 500ms for 10s"
}
```

### 3. Package.json Script

Add to `apps/desk/package.json`:

```json
{
  "scripts": {
    "test:perf": "vitest run --config vitest.config.ts tests/perf"
  }
}
```

### 4. CLAUDE.md Integration

Document in CLAUDE.md:

```markdown
## Memory Profiling (On-Demand)

Before tagging a release, run memory profiling:

\`\`\`bash
pnpm test:perf
\`\`\`

This launches the full app, measures memory for 10 seconds, and compares against baseline.

**Targets:**
- Peak memory: <250 MB (alert: >300 MB)
- Stable memory: <200 MB
- Growth vs baseline: <20 MB

**Baseline:** `.perf-baseline.json`

If memory exceeds targets, review optimization strategies in `docs/OPTIMIZATION_GUIDE.md`.
```

---

## Implementation Checklist

- [ ] Create `tests/perf/memory-profile.test.ts`
- [ ] Implement app kill function (taskkill for Windows)
- [ ] Implement WebDriver launch + memory sampling
- [ ] Implement sample validation (min 100MB, no NaN)
- [ ] Implement timeout + diagnostics (60s)
- [ ] Implement peak + stable calculation
- [ ] Implement baseline comparison + alerts
- [ ] Implement atomic save to `.perf-baseline.json`
- [ ] Add `test:perf` to package.json
- [ ] Create initial `.perf-baseline.json` (empty or placeholder)
- [ ] Test: `pnpm test:perf` completes successfully
- [ ] Test: baseline file created/updated

---

## Review References

- **Eng Review:** Architecture Issue #3 (timing: on-demand)
- **Eng Review:** Code Quality Issue #2 (app cleanup)
- **Eng Review:** Test Review Issue #2 (timeout handling)
- **Eng Review:** Performance Issue #1 (targets)

---

## Blocks

- T005: CLAUDE.md (references memory profiling)
- T006: Optimization guidelines (uses baseline data)

## Depends On

- T001: Tauri config (app must launch stably)
