---
id: E003-T04
epic: E003
status: completed
original_id: T004
title: User Documentation
---
# T004: User Documentation

**Task ID:** T004
**Effort:** 2 hours
**Priority:** P1 (needs T001, blocks T005)
**Owner:** Subagent

---

## Summary

Create user-facing docs for installation, updates, support, and privacy.

---

## Deliverables

### 1. docs/USER_INSTALL.md
- System requirements (Windows 10+)
- Download link (future: GitHub Releases)
- Step-by-step installer walkthrough
- Where data stored (AppData\Local\zntlDesk)
- Uninstall instructions

### 2. docs/USER_UPDATES.md
- Auto-update behavior
- Manual update check button
- Release notes link
- Rollback instructions

### 3. docs/USER_SUPPORT.md
- Common issues + fixes
- Where logs stored
- GitHub Issues link

### 4. docs/PRIVACY.md
- What data stored locally (SQLite)
- No telemetry, no cloud
- Export/backup data
- Reset app data procedure

### 5. Update docs/README.md
- Add links to all new docs

---

## Acceptance Criteria

- All 4 docs created + linked
- Language: clear for non-technical user
- No broken links
- Screenshots optional (text OK for MVP)

---

## Blocks

- T005: CLAUDE.md (links to user docs)

## Depends On

- T001: Installer must be configured
