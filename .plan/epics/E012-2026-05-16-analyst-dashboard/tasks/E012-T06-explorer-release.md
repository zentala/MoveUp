---
id: E012-T06
epic: E012
status: pending
created: 2026-05-16
branch: feat/E012-T06-explorer-release
title: E012-T06 — Explorer tab live + tests + UX-FLOW + PROJECT.xml + version bump 0.5.0
---

# E012-T06 — Explorer tab live + tests + UX-FLOW + PROJECT.xml + version bump 0.5.0

## Goal
Wire the Explorer tab to live data from `get_snapshots_range` / `get_events_range` / `get_sessions_range`. Add tests, update docs, bump version to 0.5.0, tag release.

## Why
Last task in E012 — turns the mockup approved in T03 into a working analytics surface, then closes the epic per `.claude/rules/versioning.md`.

## Hooks
Create under `apps/desk/src/analyst/hooks/`:
- `useRangeQuery.ts` — generic hook that takes a date range + invoke command name, returns `{ status, data }`; debounces range changes by 250 ms
- `useSnapshotsRange(from, to)` — wraps `useRangeQuery` for snapshots
- `useEventsRange(from, to)` — same for events
- `useSessionsRange(from, to)` — same for sessions

## Charts (replace fixtures from T03 mockup)
For each of the 5 chart components, swap fixture prop for the relevant hook:
1. `DeskHeightTimeline` ← `useSnapshotsRange` (height_cm over time)
2. `StateGantt` ← `useSessionsRange` (start/end + state per day)
3. `DailyScoreTrajectory` ← `useSnapshotsRange` (daily_score per minute, group by hour)
4. `BreakCreditHistogram` ← `useEventsRange` (filter `kind === 'CREDIT'`, count none/partial/full)
5. `KpiTrend` ← `useSnapshotsRange` (last value per day → small multiples)

## Performance
- Don't re-render charts on every snapshot update — `useMemo` keyed on `(from, to, data.length)`
- For DeskHeightTimeline with 10 080 points: downsample to ~1000 buckets client-side (avg per 10 minutes)

## Tests
**TS unit:**
- One test per chart: renders with mocked hook returning fixture, asserts ≥ 1 SVG element / chart-specific marker
- `useRangeQuery` test: loading → ready transition, refetch on range change, debounce

**Integration:**
- Open Analyst window in test mode, change date range, assert charts re-render with new data (use existing tauri-mock infra if available)

**E2E (manual):**
- Open Analyst, switch to Explorer tab, all 5 charts render within 500 ms for 7-day window
- Change "to" date back 3 days → charts update

## Docs
- `apps/desk/.arch/UX-FLOW.md` — add "Analyst Window" section describing both tabs, when user opens it, what each chart means (semantic-color note included per `.claude/rules/timeline-theme` if relevant)
- `apps/desk/PROJECT.xml` — add new commands (T01+T02), new window (T04), new analyst module
- `apps/desk/CHANGELOG.md` (if present) — v0.5.0 entry: "Add Analyst window with Data Catalog and Explorer (E012)."

## Version sanity check + tag (per `.claude/rules/versioning.md`)
Version bump to `0.5.0` happened in the epic-setup commit (Wave 0). This task only:
- Verifies all three files still read `0.5.0` (package.json, tauri.conf.json, Cargo.toml)
- After merge to main: `git tag -a v0.5.0 -m "v0.5.0 — E012: Analyst Dashboard"` then `git push origin v0.5.0`

## Acceptance gates (pre-merge)
- [ ] `pnpm test:all` passes
- [ ] `cargo test --release` passes
- [ ] Coverage ≥ 80% across `analyst/`
- [ ] Manual smoke: open Analyst, both tabs work, no console errors
- [ ] UX-FLOW.md describes window
- [ ] PROJECT.xml lists new commands + window
- [ ] Version bumped + commit ready

## Files touched
- `apps/desk/src/analyst/hooks/useRangeQuery.ts` (new)
- `apps/desk/src/analyst/hooks/useSnapshotsRange.ts` (new)
- `apps/desk/src/analyst/hooks/useEventsRange.ts` (new)
- `apps/desk/src/analyst/hooks/useSessionsRange.ts` (new)
- `apps/desk/src/analyst/charts/*.tsx` — remove fixture imports, use hooks
- `apps/desk/src/analyst/ExplorerTab.tsx` — pass range from `<DateRangePicker>` down
- `apps/desk/.arch/UX-FLOW.md`
- `apps/desk/PROJECT.xml`
- (version files already at 0.5.0 from epic setup — verify only)

## Epic close (after merge)
- Triage `IMPROVEMENTS.md` with zentala
- Add `.arch/HISTORY.md` line (if file exists)
- Mark all 6 task checkboxes `[x]` in ORCHESTRATOR.md
- Update `apps/desk/.plan/DONE.md` index
