---
id: E005-T13
epic: E005
status: done
created: 2026-03-21
completed: 2026-03-22
original_id: T033
---
# T033 — Widget "Timeline Zen"

**Priority:** P3
**Depends on:** T031 (widget architecture)
**Wave:** 5 (parallel with T032)

---

## Goal

Minimalist widget. Timeline big, numbers small, zero text.
For users who don't want coaching — just a visual rhythm check.

## Key Differences from One Bar

- Timeline height: 48px (hero) vs 28px
- No coach sentence
- No previous session card
- Subtle temperature (only bar color)
- Total height ~140px vs ~200px

---

## Acceptance Criteria

- [ ] Widget renders in all 3 states
- [ ] Timeline is 48px tall (hero element)
- [ ] No coach text, no previous session card
- [ ] Timer shows limitRemaining
- [ ] Total widget height <= 140px
- [ ] No file > 250 lines
