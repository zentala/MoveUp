---
id: E003-T02
epic: E003
status: completed
original_id: T002
title: Build Size Monitoring Script
---
# T002: Build Size Monitoring Script

**Task ID:** T002
**Status:** Pending
**Effort:** 4 hours
**Priority:** P0 (needs T001, blocks T005)
**Owner:** Subagent
**Date Created:** 2026-03-16

---

## Summary

Create `scripts/build-report.js` that reports installer size, tracks growth vs baseline, and logs to console + file. Add unit tests. Integrate with `pnpm tauri:build`.

---

## Acceptance Criteria

### 1. scripts/build-report.js (100-150 lines)

**Features:**
- Platform check: Windows-only
- Scan for NSIS and MSI artifacts
- Load baseline from `.build-sizes.json`
- Compare and alert (>80MB or >10% growth)
- Log to console and `.build-log.txt`
- Atomic file write (temp -> rename)
- Exit code: 0 (success), 1 (fatal error)

### 2. Package.json Integration

```json
{
  "scripts": {
    "tauri:build": "pnpm test:all && tauri build && node scripts/build-report.js"
  }
}
```

### 3. Unit Tests: tests/scripts/build-report.test.js (~150 lines)

4-5 focused tests: scan artifacts, fail on missing, corrupted baseline, growth alert, atomic write.

### 4. .build-sizes.json Format

Git-tracked baseline with timestamp, nsis_bytes, msi_bytes.

---

## Blocks

- T005: Update CLAUDE.md (documents build-report.js)

## Depends On

- T001: Tauri config (artifacts must exist)
