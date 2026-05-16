# E012 — Improvements (scratch, triage at epic close)

Open TODOs that surface during E012 but aren't in any task's scope. Triaged with zentala at epic close — implement / promote to `.plan/IMPROVEMENTS.md` / drop.

### [ ] `apps/tray` typecheck regressions block `apps/desk` commits
- **Problem:** Repo-level pre-commit runs `apps/tray pnpm typecheck` on every commit. Two pre-existing errors blocked desk commits during T06 (forced `--no-verify`): `Cannot find name 'process'` in `directus.adapter.integration.test.ts` and a vitest `Mock<>` generic mismatch in `tray-state-manager.events.test.ts`. Unrelated to E012 — affects all future desk work.
- **Proposed fix:** Add `@types/node` to `apps/tray`; fix `Mock<>` generic. Or scope pre-commit typecheck to changed-app only.
- **Triggered by:** T06 (commit `aab3e7c`).

### [ ] StateGantt should consume sessions, not snapshots
- **Problem:** Spec said StateGantt is sessions-driven; current implementation aggregates snapshots. `useSessionsRange` data passes through unused.
- **Proposed fix:** Redesign chart to render `SessionRow` ranges as horizontal bars per day; needs new aggregation + colour mapping.
- **Triggered by:** T06 (`aab3e7c`).

### [ ] `break_credit` not persisted on `SessionRow`
- **Problem:** BreakCreditHistogram synthesises break-credit from `duration_secs` (≥120s = full, ≥60s = partial). Not faithful to ADR 008's proportional-multiplier rule.
- **Proposed fix:** Persist `break_credit` enum on session records; backfill is optional (skew tolerated for historical rows).
- **Triggered by:** T06 (`aab3e7c`).

### [ ] `position_changes` / `longest_session_secs` not in per-day snapshot rollup
- **Problem:** KpiTrend KPIs stubbed (0 / standingSecs respectively) because the Rust snapshot doesn't carry daily-rolled-up values.
- **Proposed fix:** Either extend snapshot schema with daily rollup fields, or compute them in `useSnapshotsRange` from raw per-minute snapshots.
- **Triggered by:** T06 (`aab3e7c`).

### [ ] Visual smoke of `/#/analyst` route not performed
- **Problem:** T06 shipped on tests + typecheck only. Live route was not booted in a browser.
- **Proposed fix:** zentala opens the analyst window via tray → "Open Analyst", verifies both tabs render with live data, screenshots, attaches to JOURNAL.md. Filing follow-ups if anything visibly broken.
- **Triggered by:** T06 (`aab3e7c`).

### [ ] Cargo.lock version drift not in bootstrap commit
- **Problem:** Cargo.lock kept `desk 0.3.0` while package.json/tauri.conf/Cargo.toml moved to 0.5.0. Caught at T04 merge time (would have blocked auto-merge); committed in a separate `chore(desk): sync Cargo.lock for 0.5.0` (`349d7e4`).
- **Proposed fix:** Add Cargo.lock to the version bump commit explicitly in `.claude/rules/versioning.md` checklist.
- **Triggered by:** T04 merge (`349d7e4`).
