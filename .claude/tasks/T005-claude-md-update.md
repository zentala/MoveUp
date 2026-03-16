# T005: Update CLAUDE.md

**Task ID:** T005
**Effort:** 1 hour
**Priority:** P1 (depends on T001-T004)
**Owner:** Zentala

---

## Summary

Update apps/desk/CLAUDE.md with installer, distribution, code signing, auto-update, icon requirements, and pre-release checklist sections.

---

## New Sections to Add

1. **## Icon Requirements**
   - List required icons (32x32, 128x128, icon.ico)
   - Link to generation guide

2. **## Installer & Distribution**
   - Build command: `pnpm tauri:build`
   - Output locations
   - Size targets + baselines
   - Memory profiling: `pnpm test:perf`

3. **## Code Signing (Scaffolded)**
   - Certificate requirements
   - GitHub Secrets setup (template)
   - Implementation timeline

4. **## Auto-Update Mechanism (Scaffolded)**
   - How it works (future)
   - Implementation timeline

5. **## Pre-Release Checklist**
   ```
   Before tagging release:
   1. Run `pnpm test:all`
   2. Run `pnpm test:perf`
   3. Review .build-sizes.json
   4. Review .perf-baseline.json
   5. Tag: git tag v0.1.0
   ```

6. **## Key Files (additions)**
   - scripts/build-report.js
   - scripts/sign-installer.sh
   - .build-sizes.json
   - .perf-baseline.json
   - docs/USER_*.md files

---

## Acceptance Criteria

- All sections written
- Links to user docs work
- Links to script files accurate
- Pre-release checklist clear
- No broken references

---

## Depends On

- T001: Icon + signing docs
- T002: build-report.js docs
- T003: Memory profiling docs
- T004: User docs
