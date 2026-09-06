---
formatVersion: 1
type: handoff
epic: E014
created: 2026-09-05
plan: ./PLAN.md
points: 43
---

# E014 handoff — supervised release rollback

## TLDR

You are implementing MoveUp's half of a two-repo change. MoveUp keeps several
installed builds and knows which one last proved healthy. PM3 gets the
supervision loop, and that half is not yours: waves 3 and 4 are blocked until
two items land in the PM3 backlog. Start at wave 1, stop at the end of wave 2,
and do not invent a local watchdog to fill the gap.

Read [`PLAN.md`](./PLAN.md) for the decisions D1 to D7 and the alternatives
that were rejected. This file is the execution order.

## Mental model — how the thing works today

The app is a Tauri 2 desktop program. The Rust side lives in `src-tauri/src/`,
the React side in `src/`. One binary, `desk.exe`, is installed to
`%LOCALAPPDATA%\MoveUp\`.

Four facts drive every task below. Verify none of them by reading old
documents; they were checked on 2026-09-05.

1. **Login start is a registry value.** `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\MoveUp`
   holds `C:\Users\zentala\AppData\Local\MoveUp\desk.exe --minimized`. The app
   writes it itself in `ensure_autostart`, `src-tauri/src/setup_helpers.rs:104`.
   That function is a deliberate no-op in debug builds, so a dev run never
   touches the registry.
2. **Nothing supervises the process.** There is no `pm3.yaml` in this repo and
   no MoveUp service in PM3. If the app dies, it stays dead until the next
   logon.
3. **The dev launcher kills the installed build.** `scripts/tauri-dev.ps1`
   finds a running `desk.exe` and offers to kill it, because Windows cannot
   overwrite a running executable. Nothing brings the installed build back.
4. **Both builds serve the same port.** `setup_remote_display` is called
   unconditionally at `src-tauri/src/setup_helpers.rs:67`, outside any
   `debug_assertions` gate, so the dev build and the installed build both
   listen on `DESK_REMOTE_PORT`, default 3390, and both answer
   `GET /display/api` with JSON. This is why occupancy is detected by the
   health check and not by a Windows API. Do not add `FindWindowW` or
   `OpenMutexW` interop.

## Waves and order

### Wave 1 — release store (13 points, not blocked)

| Task | Points | Agent |
|---|---|---|
| E014-T09 epic setup: version bump to 0.6.0, two ADRs, `.plan/ARCH.md` | 5 | main |
| E014-T01 release store layout and retention | 5 | ts-dev |
| E014-T02 `last-known-good` marker file | 3 | ts-dev |

Do T09 first. It bumps the version in three files per
[`.claude/rules/versioning.md`](../../../.claude/rules/versioning.md) and writes
the two ADRs the plan names, so later tasks have a decision to cite.

T01 and T02 are independent of each other and can run in parallel.

### Wave 2 — what "good" means (8 points, not blocked)

| Task | Points | Agent |
|---|---|---|
| E014-T03 health-good probe | 5 | ts-dev |
| E014-T04 ordered candidate list writer | 3 | ts-dev |

T03 depends on T02, T04 depends on both T01 and T02.

### Wave 3 — PM3 consumption (6 points, BLOCKED)

T05 `pm3.yaml`, T06 stand-down wiring. Both wait on the PM3 backlog.

### Wave 4 — rollback and cutover (16 points, BLOCKED)

T07 demotion integration, T08 end-to-end verification, T10 retire the `Run`
key. T10 additionally waits on PM3's own at-logon Scheduled Task; see below.

## What blocks waves 3 and 4

Two items in `int://mATX.lan/C:/code/pm3-mcp/.plan/BACKLOG.md`, section
"2026-09-05 — Nadzór z powrotem do poprzedniego builda":

- **Kandydaci i degradacja do poprzedniego builda** (8 points) blocks T05 and T07.
- **Stand-down: przechodzący health check znaczy „usługa działa"** (5 points) blocks T06.

A third, pre-existing PM3 item blocks T10: **"`pm3d` has no ONLOGON Scheduled
Task on mATX"**, owner E000-A4. Until PM3's daemon starts at logon on its own,
removing the `Run` key leaves a five-minute hole after every login, because the
daemon is currently revived by the `pm3-doctor` watchdog every five minutes.

**Do not work around any of these.** Paweł decided on 2026-09-05 that PM3 is
the only daemonizer. A private PowerShell watchdog in this repo was considered
as Approach A and rejected. If you find yourself writing a scheduled task here,
stop and re-read `PLAN.md`.

## Where to edit

- Release store, retention, marker, candidate list: new Rust modules under
  `src-tauri/src/`. Keep each file at or under 250 lines and each function at
  or under 50, per `.claude/rules/code-style.md` and `rules/rust.md`.
- Autostart behaviour (T10 only): `src-tauri/src/setup_helpers.rs`, the
  `ensure_autostart` pair at lines 92 and 104.
- Service declaration (T05): a new `pm3.yaml` in the repo root.

## What not to touch

- `scripts/tauri-dev.ps1`. Its kill behaviour is correct and is the premise of
  the two-miss rule.
- The `Run` key, until T10 and only after PM3 starts at logon.
- Anything under `.plan/epics/E013-.../`. E014 depends on E013 and does not
  edit it.
- `src-tauri/src/google_fit.rs` beyond what a test needs; it was touched on
  2026-09-05 to fix a parallel-test env race and is otherwise out of scope.

## Test layout in this repo

- Rust: a sibling file `<module>_tests.rs`, registered as
  `#[cfg(test)] mod <module>_tests;` in `lib.rs`. This keeps modules under the
  250-line cap.
- TypeScript: co-located `<module>.test.ts(x)`, run by `pnpm test:unit`.

Build gate: `pnpm tauri:build` runs `pnpm test:all` first and halts on failure.
Set `CI=true` when running it non-interactively, or pnpm refuses to purge
`node_modules` without a terminal.

## How to know each wave is done

Wave 1: a fourth install prunes the oldest of three retained releases, and
pruning never removes the release marked `last-known-good`.

Wave 2: a build that stays alive but never serves is NOT marked good, and a
build that serves once then exits before thirty seconds is NOT marked good.
Both assertions fail today because neither the probe nor the marker exists.

Waves 3 and 4: see `PLAN.md` section `## Test strategy`. Wave 4 ends with a
deliberately broken newest build that must roll back to `last-known-good`
within two poll cycles.

## Note added 2026-09-06 (before AO dispatch)

- **Version is already 0.6.0.** T09's original brief included "version bump
  to 0.6.0 per `.claude/rules/versioning.md`" — E015 already bumped and
  tagged `v0.6.0` on 2026-09-06. T09 below does the two ADRs and the
  `.arch/ARCHITECTURE.md` update only; skip the version bump, it is a no-op.
  (This repo also uses `.arch/ARCHITECTURE.md` and `.arch/ADR/`, not
  `.plan/ARCH.md`/`.plan/ADR/` — PLAN.md's references to the latter are
  stale from before that convention was fixed in E016.)
- **T01/T02 sequenced, not parallel**, unlike the wave table above. Every
  epic run through AO since E018 hit the same collision: two tasks that both
  need a `mod` line in `lib.rs` cannot run in the same wave without a
  guaranteed `write_set_out_of_scope_write`/file-claim clash. T01 and T02
  both add a new module, so T02 now `depends_on: ["E014-T01"]` below. Same
  reasoning chains T03 and T04 after T02. The wave-1/wave-2 split above is
  still accurate for POINTS bookkeeping; the AO block below is the real,
  sequential execution order.
- **Module names below are new choices**, not named in PLAN.md (which only
  names test files): `release_store.rs` (T01), `last_known_good.rs` (T02),
  `health_probe.rs` (T03), `candidate_list.rs` (T04) — each with its sibling
  `_tests.rs`, per this repo's test-file convention.

## AO

```yaml
project: MoveUp
epic: E014
base_ref: main
tasks:
  - id: E014-T09
    repo: MoveUp
    executor: main
    depends_on: []
    write_set: [".arch/ADR/018-pm3-app-ownership-split.md", ".arch/ADR/019-release-store-layout.md", ".arch/ARCHITECTURE.md", ".plan/sessions/**", ".plan/BACKLOG.md", ".plan/IMPRO.md", ".plan/epics/E014-2026-09-05-supervised-release-rollback/JOURNAL.md", ".plan/epics/E014-2026-09-05-supervised-release-rollback/IMPRO.md"]
    claims: [".arch/ADR/018-pm3-app-ownership-split.md", ".arch/ADR/019-release-store-layout.md", ".arch/ARCHITECTURE.md"]
    verification: "node -e \"const fs=require('fs');for (const f of ['.arch/ADR/018-pm3-app-ownership-split.md','.arch/ADR/019-release-store-layout.md']) { if (!fs.existsSync(f)) { console.error('missing '+f); process.exit(1); } } console.log('ok');\""
    budget_minutes: 45
  - id: E014-T01
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E014-T09"]
    write_set: ["src-tauri/src/release_store.rs", "src-tauri/src/release_store_tests.rs", "src-tauri/src/lib.rs", ".plan/sessions/**", ".plan/BACKLOG.md", ".plan/IMPRO.md", ".plan/epics/E014-2026-09-05-supervised-release-rollback/JOURNAL.md", ".plan/epics/E014-2026-09-05-supervised-release-rollback/IMPRO.md"]
    claims: ["src-tauri/src/release_store.rs", "src-tauri/src/release_store_tests.rs", "src-tauri/src/lib.rs"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- release_store"
    budget_minutes: 90
  - id: E014-T02
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E014-T01"]
    write_set: ["src-tauri/src/last_known_good.rs", "src-tauri/src/last_known_good_tests.rs", "src-tauri/src/release_store.rs", "src-tauri/src/lib.rs", ".plan/sessions/**", ".plan/BACKLOG.md", ".plan/IMPRO.md", ".plan/epics/E014-2026-09-05-supervised-release-rollback/JOURNAL.md", ".plan/epics/E014-2026-09-05-supervised-release-rollback/IMPRO.md"]
    claims: ["src-tauri/src/last_known_good.rs", "src-tauri/src/last_known_good_tests.rs", "src-tauri/src/lib.rs"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- last_known_good"
    budget_minutes: 60
  - id: E014-T03
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E014-T02"]
    write_set: ["src-tauri/src/health_probe.rs", "src-tauri/src/health_probe_tests.rs", "src-tauri/src/lib.rs", ".plan/sessions/**", ".plan/BACKLOG.md", ".plan/IMPRO.md", ".plan/epics/E014-2026-09-05-supervised-release-rollback/JOURNAL.md", ".plan/epics/E014-2026-09-05-supervised-release-rollback/IMPRO.md"]
    claims: ["src-tauri/src/health_probe.rs", "src-tauri/src/health_probe_tests.rs", "src-tauri/src/lib.rs"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- health_probe"
    budget_minutes: 60
  - id: E014-T04
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E014-T03"]
    write_set: ["src-tauri/src/candidate_list.rs", "src-tauri/src/candidate_list_tests.rs", "src-tauri/src/release_store.rs", "src-tauri/src/lib.rs", ".plan/sessions/**", ".plan/BACKLOG.md", ".plan/IMPRO.md", ".plan/epics/E014-2026-09-05-supervised-release-rollback/JOURNAL.md", ".plan/epics/E014-2026-09-05-supervised-release-rollback/IMPRO.md"]
    claims: ["src-tauri/src/candidate_list.rs", "src-tauri/src/candidate_list_tests.rs", "src-tauri/src/release_store.rs", "src-tauri/src/lib.rs"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- candidate_list"
    budget_minutes: 60
```

Waves 3-4 (T05-T08, T10) stay out of this manifest — they are blocked on
`pm3-mcp`'s backlog (see above) and are not dispatched by this block.
