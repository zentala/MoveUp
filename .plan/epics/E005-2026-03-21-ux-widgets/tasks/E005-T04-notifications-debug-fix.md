---
id: E005-T04
epic: E005
status: completed
created: 2026-03-21
completed: 2026-03-21
original_id: T024
title: T024 — Investigate + fix notifications
---
# T024 — Investigate + fix notifications

**Status:** open
**Priority:** P1 (bug — user never sees any notification)
**Branch:** feat/T024-notifications-fix

---

## Problem

User has never seen a notification from this app despite the notification infrastructure being implemented.

## Fix Plan

1. Test notification command (production-ready, with 60s rate-limit)
2. Lower thresholds (inactivity: 90 min -> 60 min)
3. Add `StandingTargetReached` notification event
4. Feature flag for notification backend ("toast", "popup", "both")

---

## Acceptance Criteria

- [ ] Click "Test notification" in debug UI -> Windows notification appears
- [ ] `trigger_test_notification` IPC command returns Ok (not Err)
- [ ] Inactivity threshold lowered to 60 min
- [ ] `StandingTargetReached` fires when standing target complete
- [ ] `notification_backend` config field added with default "toast"
- [ ] `cargo test` passes
