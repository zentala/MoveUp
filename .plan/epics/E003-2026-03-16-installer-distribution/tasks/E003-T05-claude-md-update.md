---
id: E003-T05
epic: E003
status: completed
original_id: T005
title: Update CLAUDE.md
---
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

1. **Icon Requirements** — List required icons (32x32, 128x128, icon.ico)
2. **Installer & Distribution** — Build command, output locations, size targets
3. **Code Signing (Scaffolded)** — Certificate requirements, GitHub Secrets setup
4. **Auto-Update Mechanism (Scaffolded)** — How it works (future)
5. **Pre-Release Checklist** — Steps before tagging release
6. **Key Files (additions)** — scripts/build-report.js, .build-sizes.json, etc.

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
