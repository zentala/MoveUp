# Overlay Progress Bar — Orchestration Plan

**For:** Orchestrator agent coordinating parallel work via worktrees
**Date:** 2026-03-19
**Status:** Ready to execute

---

## Pre-Flight Checklist

Before dispatching agents:
1. Ensure `main` branch is clean: `git status`
2. Ensure code compiles: `cd src-tauri && cargo check`
3. Ensure tests pass: `cd src-tauri && cargo test --lib`

---

## Wave Structure

```
WAVE 1 (parallel — 2 agents on worktrees)
├── T-OVR-001: Dev mode visible bar     [overlay_renderer.rs]
└── T-OVR-002: Device notification      [serial.rs, lib.rs]
         │
         ▼ merge both to main
WAVE 2 (parallel — 3 agents on worktrees)
├── T-OVR-003: Compare modes visually   [manual + docs]
├── T-OVR-005: Adjustable bar height    [overlay_renderer.rs]
└── T-OVR-006: Fix auto-test            [auto-test.sh]
         │
         ▼ merge all to main
WAVE 3 (parallel — 2 agents on worktrees)
├── T-OVR-004: Bar variants             [overlay_renderer.rs]
└── T-OVR-007: Rust visual regression   [overlay_renderer.rs tests]
         │
         ▼ merge to main
WAVE 4 (sequential — 1 agent)
├── T-OVR-008: Wire to session data     [tray_controller.rs]
└── T-OVR-009: Choose production mode   [decision + docs]
```

---

## Worktree Setup Commands

Run these from project root before dispatching Wave 1:

```bash
# Wave 1
git branch feat/T-OVR-001-dev-mode main
git worktree add .claude/worktrees/T-OVR-001-dev-mode feat/T-OVR-001-dev-mode

git branch feat/T-OVR-002-device-notif main
git worktree add .claude/worktrees/T-OVR-002-device-notif feat/T-OVR-002-device-notif
```

After Wave 1 merge:
```bash
# Wave 2
git branch feat/T-OVR-003-compare-modes main
git worktree add .claude/worktrees/T-OVR-003-compare-modes feat/T-OVR-003-compare-modes

git branch feat/T-OVR-005-bar-height main
git worktree add .claude/worktrees/T-OVR-005-bar-height feat/T-OVR-005-bar-height

git branch feat/T-OVR-006-fix-autotest main
git worktree add .claude/worktrees/T-OVR-006-fix-autotest feat/T-OVR-006-fix-autotest
```

### Merge Protocol

After each wave completes:
```bash
git checkout main
git merge feat/T-OVR-001-dev-mode --no-ff -m "feat(overlay): dev mode — always-visible progress bar"
git merge feat/T-OVR-002-device-notif --no-ff -m "feat(desk): device disconnected notification"
# Resolve conflicts if any (unlikely — different files)

# Cleanup
git worktree remove .claude/worktrees/T-OVR-001-dev-mode
git branch -d feat/T-OVR-001-dev-mode
git worktree remove .claude/worktrees/T-OVR-002-device-notif
git branch -d feat/T-OVR-002-device-notif
```

---

## Merge Conflict Risk Matrix

| Wave | Agent A | Agent B | Conflict Risk | Notes |
|------|---------|---------|---------------|-------|
| 1 | T-OVR-001 (overlay_renderer.rs) | T-OVR-002 (serial.rs, lib.rs) | **NONE** | Different files |
| 2 | T-OVR-005 (overlay_renderer.rs) | T-OVR-006 (auto-test.sh) | **NONE** | Different files |
| 2 | T-OVR-005 (overlay_renderer.rs) | T-OVR-003 (docs only) | **NONE** | Different files |
| 3 | T-OVR-007 (overlay_renderer.rs tests) | T-OVR-004 (overlay_renderer.rs) | **LOW** | Both touch same file but different sections |
