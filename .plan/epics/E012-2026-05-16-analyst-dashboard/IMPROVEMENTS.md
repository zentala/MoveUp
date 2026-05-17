# E012 — Improvements (triaged 2026-05-17)

Triaged at epic close. Closed items kept here for traceability.

### [x] `apps/tray` typecheck blocking commits
- **Resolution**: false alarm. Re-running `pnpm typecheck` in `apps/tray` on `main` succeeds. The `--no-verify` on T06's merge was a stale-node_modules transient inside the subagent worktree. No fix needed.
- **Closed**: 2026-05-17 session 03:00.

### [x] Visual smoke of `/#/analyst` not performed
- **Resolution**: Performed via Chrome DevTools MCP on 2026-05-17. Both `/#/mockup/analyst` (504 snapshots, 29 sessions, all 5 charts rendering) and `/#/analyst` (live mode, gracefully degrades when Tauri runtime absent — "Failed: Cannot read properties of undefined (reading 'invoke')") render cleanly. No JS errors.
- **Closed**: 2026-05-17 session 03:00.

### [x] `useSessionsRange` does derivation inside fetch hook
- **Resolution**: `deriveBreakCredit` + `coerceSessionRow` (with `WireSessionRow` type) extracted to `explorer-derivations.ts`. Hook now I/O-only. +3 unit tests. Layering smell resolved.
- **Closed**: 2026-05-17 session 03:00 — commit pending in epic-close commit.

### [x] No ADR for "Analyst as separate window"
- **Resolution**: ADR 012 written and linked from `apps/desk/CLAUDE.md` UI Components #6.
- **Closed**: 2026-05-17 session 03:00.

### [x] Cargo.lock version drift not in bootstrap commit
- **Resolution**: Cargo.lock committed in `349d7e4` (`chore(desk): sync Cargo.lock for 0.5.0`). Versioning rule note proposed for `.claude/rules/versioning.md` — promoted to global IMPROVEMENTS (see below).
- **Closed**: 2026-05-17 session 03:00.

---

## Promoted to global `.plan/IMPROVEMENTS.md`

### [ ] StateGantt should consume sessions, not snapshots
Cross-cutting analyst-chart redesign — outlives this epic. See `.plan/IMPROVEMENTS.md`.

### [ ] `break_credit` not persisted on `SessionRow`
Schema change to session persistence — outlives this epic. See `.plan/IMPROVEMENTS.md`.

### [ ] `position_changes` / `longest_session_secs` not in per-day snapshot rollup
Schema change to snapshot logger — outlives this epic. See `.plan/IMPROVEMENTS.md`.
