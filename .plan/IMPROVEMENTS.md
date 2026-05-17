# Improvements — desk app (cross-cutting backlog)

Open quality/UX/DevEx/architecture TODOs that outlive any single epic. Triaged at epic close from per-epic `IMPROVEMENTS.md` files. Stop hook reads BOTH this file and the active epic's IMPROVEMENTS.md before flagging.

### [ ] StateGantt should consume sessions, not snapshots
- **Problem**: E012 ExplorerTab's StateGantt aggregates from snapshots; it was specified to be sessions-driven. `useSessionsRange` data passes through unused on Explorer for this chart. Visual fidelity OK on mockup fixtures, but live mode renders per-minute snapshot buckets rather than session ranges — semantically wrong.
- **Proposed fix**: Redesign chart to render `SessionRow` ranges as horizontal bars per day; needs new aggregation function in `explorer-derivations.ts` + a colour map per state.
- **Triggered by**: E012 (commit `aab3e7c`).

### [ ] `break_credit` not persisted on `SessionRow`
- **Problem**: ADR 008 defines proportional break credit; persisted sessions lose that exact value. `useSessionsRange` synthesises `breakCredit` from `durationSecs` thresholds (`deriveBreakCredit` in `explorer-derivations.ts`). Off by enough margin that BreakCreditHistogram is approximate.
- **Proposed fix**: Add `break_credit` (enum or computed float) to the SQLite sessions row; backfill is optional (skew tolerated for historical rows). Update `coerceSessionRow` to use the real value when present.
- **Triggered by**: E012 (commit `aab3e7c`).

### [ ] `position_changes` / `longest_session_secs` not in per-day snapshot rollup
- **Problem**: KpiTrend (E012 Explorer) stubs `positionChanges: 0` and `longestSessionSecs: snap.standingSecs` because the per-minute snapshot doesn't carry daily rollups. Live mode shows misleading values for these two KPIs.
- **Proposed fix**: Either (a) extend snapshot schema with `daily_position_changes` + `longest_session_secs_today` fields, or (b) compute in `useSnapshotsRange` from raw per-minute snapshots by detecting state transitions and the longest contiguous standing run.
- **Triggered by**: E012 (commit `aab3e7c`).

### [ ] `.claude/rules/versioning.md` checklist should mention `Cargo.lock`
- **Problem**: During E012 bootstrap, `Cargo.lock` was left at the previous version while `package.json` / `tauri.conf.json` / `Cargo.toml` all moved. Caught at first task merge (would have blocked auto-merge). Required a separate `chore(desk): sync Cargo.lock for 0.5.0` commit (`349d7e4`).
- **Proposed fix**: Add `Cargo.lock` line to the bump checklist in `.claude/rules/versioning.md` (project-management rule, technically `apps/desk/.claude/rules/versioning.md`). One-line edit.
- **Triggered by**: E012 (commit `349d7e4`).
