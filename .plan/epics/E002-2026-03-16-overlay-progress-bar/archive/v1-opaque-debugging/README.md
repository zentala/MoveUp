# V1: OPAQUE Mode Debugging — Archive

**Date:** 2026-03-19
**Status:** ARCHIVED — Lessons extracted to `../KNOWLEDGE-BASE.md`

## What Happened

Session with Haiku 4.5 agent attempting to implement demo mode for overlay progress bar.
4+ iterations, all failed. Root cause found by Opus 4.6: agent tested LAYERED mode while
user was seeing OPAQUE mode — two completely different code paths.

## Key Finding

Progress bar invisible because `visible=false` without desk sensor connected.
Test code (color cycling) was the only visible element because it renders unconditionally.

## Files

| File | What It Contains |
|------|------------------|
| `overlay_demo_troubleshooting.md` | 3 failed approaches with code snippets |
| `TESTING-OVERLAY.md` | Manual testing instructions (obsolete) |
| `OVERLAY-REPORT.md` | Iteration-by-iteration history |
| `test-harness.sh` | Old test script (replaced by auto-test.sh) |

## Lessons (see ../KNOWLEDGE-BASE.md Section 4)

1. Test the SAME code path user sees
2. Don't remove working code without proven replacement
3. Verify before claiming "should work"
4. Check all conditions (visible, bar_width, demo_mode) before changing code
