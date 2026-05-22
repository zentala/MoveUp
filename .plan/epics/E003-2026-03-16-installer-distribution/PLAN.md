---
created: 2026-03-16
status: completed
title: Installer & Distribution
---
# Installer & Distribution Plan — zntlDesk

**Status:** Plan Complete (Approved by CEO + Eng Review)
**Target:** 2 weeks (Wave 1 + Wave 2)
**Owner:** Zentala
**Date:** 2026-03-16

---

## Overview

Ship zntlDesk as a production-ready Windows installer (.msi + .exe) with:
- Automated size monitoring (detect bloat early)
- Runtime memory profiling (prevent regressions)
- User documentation (installation, updates, privacy)
- Code signing + auto-update scaffolding (for future implementation)
- Developer tooling (pre-release checklist, optimization guidelines)

---

## Core Objectives

1. **Distribution** — Users can download + install app like any Windows app
2. **Quality Assurance** — Installer size <70MB, memory <250MB peak
3. **Future-Ready** — Infrastructure for code signing, auto-update, releases
4. **Developer Experience** — Clear build reports, memory baselines, release checklist

---

## Success Criteria

- [x] Tauri configured for NSIS/MSI bundling
- [x] Build size reported after every `pnpm tauri:build`
- [x] Memory profiling available via `pnpm test:perf`
- [x] User docs cover installation, updates, privacy
- [x] CLAUDE.md documents entire build + release process
- [x] Pre-release checklist prevents regressions
- [x] Code signing + auto-update have clear implementation path

---

## Tasks (T001-T007)

### Wave 1 (Parallel, ~1 week)

**T001:** Configure Windows installer (tauri.conf.json, NSIS, signing scaffolding)
-> Blocks: T002, T003, T004

**T002:** Build size monitoring (build-report.js script + tests)
-> Blocks: T005

**T003:** Memory profiling (memory-profile.test.ts + baseline)
-> Blocks: T005, T006

### Wave 2 (Sequential, ~1 week)

**T004:** User documentation (USER_INSTALL.md, USER_UPDATES.md, etc)
-> Blocks: T005

**T005:** Update CLAUDE.md (documents T001-T004)
-> Depends on: T001, T002, T003, T004

**T006:** Optimization guidelines (based on T003 baseline)
-> Depends on: T003

### Wave 3 (Future, ~1-2 months after T001-T006)

**T007:** GitHub Releases automation (release.yml workflow)
-> Depends on: T001, T002

---

## Review Decisions

### Architecture
- **Exit codes (T002):** 0=success, 1=fatal error. Warnings logged but don't block.
- **Build report location (T002):** Both local + CI (part of tauri:build script)
- **Memory profiling timing (T003):** On-demand (`pnpm test:perf`), before release
- **Baseline tracking:** Both .build-sizes.json + .perf-baseline.json git-tracked

### Code Quality
- **Error handling (T002):** Fail fast with clear messages to stderr
- **App cleanup (T003):** Kill existing process before test
- **Icons (T001):** Document in CLAUDE.md (32x32, 128x128, icon.ico required)

### Testing
- **build-report.js tests (T002):** Focused (4-5 tests: happy + main failures)
- **memory-profile timeout (T003):** Explicit 60s + diagnostics
- **Memory validation (T003):** Min 100MB, fail on NaN

### Performance
- **Hash strategy (T002):** File size + mtime (skip SHA256 for speed)

### Release
- **Pre-release checklist (T005):** Document in CLAUDE.md before tagging

---

## Not in Scope (Deferred)

- Code signing **implementation** (scaffolding only)
- Auto-update **implementation** (scaffolding only)
- GitHub Releases automation (separate task T007)
- Metrics aggregation dashboard (TODO for 6mo)
- Cross-platform (Windows-only)
- Telemetry (future, opt-in if implemented)

---

## Design Diagrams

### Build Flow

```
Developer: pnpm tauri:build
    |
[Pre-build: tests pass]
    |
[Tauri bundles Rust + JS]
    |
src-tauri/target/release/bundle/
|-- nsis/ -> zntlDesk_*.exe (65 MB)
|-- msi/ -> zntlDesk_*.msi (62 MB)
    |
[scripts/build-report.js scans]
|-- Calculate sizes
|-- Compare vs .build-sizes.json baseline
|-- Alert if growth >10% or size >80MB
|-- Update baseline (atomic write)
    |
Console output:
  Build complete
  Installer: 65.3 MB (nsis), 58.2 MB (msi)
  Growth: +2.1 MB vs baseline
```

### Memory Profiling

```
Developer: pnpm test:perf
    |
[Kill existing zntlDesk processes]
    |
[Launch app via Tauri WebDriver]
    |
[Sample memory every 500ms for 10s]
|-- Validate: min 100MB, no NaN
|-- Record: peak, stable, trend
|-- Compare vs .perf-baseline.json
    |
[Fail if peak >300MB or growth >20MB]
    |
Console output:
  Memory Profile:
    Peak: 245 MB (first 2s)
    Stable: 180 MB (after 5s)
    Trend: stable
```

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Build fails silently (no artifacts) | Post-build check in CI/CD validates .exe/.msi exist |
| Installer bloat undetected | build-report.js warns if >80MB or growth >10% |
| Memory regression missed | Memory profiling baseline + test fails if >300MB |
| Developer forgets pre-release checks | Checklist in CLAUDE.md + pre-release task reminder |
| Concurrent builds corrupt baseline | Atomic file write (temp -> rename) |
| WebDriver test hangs | Explicit 60s timeout + kill process on timeout |

---

## Effort Estimate

| Task | Effort | Owner | Timeline |
|------|--------|-------|----------|
| T001 | 3h | You | Week 1 |
| T002 | 4h | Subagent | Week 1 (parallel) |
| T003 | 4h | Subagent | Week 1 (parallel) |
| T004 | 2h | Subagent | Week 2 |
| T005 | 1h | You | Week 2 |
| T006 | 2h | Subagent | Week 2 |
| **Total** | **~16h** | | **~2 weeks** |

---

## References

- **CEO Review:** `.claude/reviews/2026-03-16-installer-ceo-review.md`
- **Eng Review:** `.claude/reviews/2026-03-16-installer-eng-review.md`
- **Tasks:** T001-T007 (detailed specs in this directory)
