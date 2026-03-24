---
id: E003-T03
epic: E003
status: done
original_id: T003
---
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

`"test:perf": "vitest run --config vitest.config.ts tests/perf"`

---

## Blocks

- T005: CLAUDE.md (references memory profiling)
- T006: Optimization guidelines (uses baseline data)

## Depends On

- T001: Tauri config (app must launch stably)
