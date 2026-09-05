---
formatVersion: 1
type: epic
epic: E014
created: 2026-09-05
status: planning
readiness: drafting
points: 40
agent: ts-dev
depends-on: [E013]
---

# E014: Supervised release rollback

## TLDR

The installed MoveUp app has no crash recovery and no way to survive a bad
build. This epic splits the fix in two: MoveUp keeps a small store of past
builds and marks which one last proved healthy; PM3 gains a generic candidate
list so it can demote to an older build after repeated health failures and
promote back once a newer one proves good. MoveUp's half ships first and is
useful on its own. PM3's half is filed as its own backlog item and blocks two
of four waves here. Recommended approach: extend PM3 (Approach B), not a
private watchdog script.

## What

Two related capabilities, split by ownership:

1. **MoveUp** keeps a small, ordered, versioned release store under
   `%LOCALAPPDATA%\MoveUp\releases\` (or equivalent), writes a
   `last-known-good` marker only after a build proves itself, prunes to 3
   retained builds, and exposes a stable health contract
   (`GET /display/api` on `DESK_REMOTE_PORT`) that any supervisor can probe.
2. **PM3** gains an ordered candidate list per service, a rule that demotes to
   the next candidate after repeated health failure, promotion back to a
   newer candidate once it proves good, and a stand-down rule so it never
   fights a process that already holds the app's `tauri-plugin-single-instance`
   lock.

This epic depends on [E013](../E013-2026-08-28-signed-tauri-pm3-deployment/PLAN.md),
whose Wave 4 already proposes putting the installed MoveUp executable under
PM3 supervision with a health check against `/display/api`, autorestart, and
bounded backoff. E014 does not repeat that work — it adds candidate lists and
rollback on top of the plain single-candidate supervision E013 sets up.

## Why

Two things happen today, and nothing answers either of them.

First, `scripts/tauri-dev.ps1`'s process guard kills the installed
`desk.exe` before starting a dev build (see
[`.claude/rules/overlay.md`](../../../.claude/rules/overlay.md) and
`scripts/tauri-dev.ps1`), because Windows cannot overwrite a running
executable. Nothing brings the installed build back afterward. The developer
ends a dev session and the app is simply gone until the next login.

Second, there is no way to survive a bad build. If a newly installed
executable starts but never serves (a broken frontend bundle, a panicking
Rust command handler, a wrong `DESK_REMOTE_DIST`), nothing falls back to the
previous, working installer. The only recovery today is manual: notice the
app is broken, find an older installer, reinstall it by hand.

## Scope

| ID | Title | Wave | Points | Importance | Agent | TLDR |
|---|---|---|---|---|---|---|
| E014-T01 | Release store layout + retention | 1 | 5 | High | ts-dev | Add a versioned release directory under `%LOCALAPPDATA%\MoveUp\releases\<version>\`, retain 3, prune oldest on install. **Tests:** unit test asserts a 4th install prunes the oldest of 3 retained releases; unit test asserts pruning never removes the release marked `last-known-good`. |
| E014-T02 | `last-known-good` marker file | 1 | 3 | High | ts-dev | Write a `last-known-good` marker only after a build passes the health probe (D5); read it back to build the candidate order. **Tests:** unit test asserts the marker is absent until a probe succeeds; unit test asserts a failed probe leaves the previous marker untouched. |
| E014-T03 | Health-good probe (process-alive AND JSON) | 2 | 5 | High | ts-dev | Implement the "good build" check from D5: process alive 30s AND `GET /display/api` returns JSON within the health timeout. Neither alone is sufficient. **Tests:** unit test with a process that stays alive but never serves must NOT be marked good; unit test with a process that serves once then exits before 30s must NOT be marked good; unit test with both conditions met must be marked good. |
| E014-T04 | Ordered candidate list writer | 2 | 3 | High | ts-dev | Emit the ordered candidate list (newest installed, then last-known-good, then next older retained) as a file PM3 can read (D6). **Tests:** unit test asserts order with 3 distinct candidates; unit test asserts the list collapses correctly when newest IS last-known-good (no duplicate entry). |
| E014-T05 | `pm3.yaml` for MoveUp | 3 | 3 | High | pm | Add `pm3.yaml` declaring the MoveUp service, health check against `/display/api`, `boot: true`, and (once available) the candidate-list feature from the PM3 backlog. **BLOCKED on PM3 backlog item "Kandydaci i degradacja do poprzedniego builda".** **Tests:** manual — `pm3 reload` accepts the file with no schema error. |
| E014-T06 | Singleton stand-down wiring | 3 | 3 | High | pm | Wire MoveUp's service entry to the PM3 stand-down feature (D4) so PM3 checks `io.zntl.desk` singleton ownership, not executable path, before respawning. **BLOCKED on PM3 backlog item "Stand-down, gdy usługę trzyma cudzy proces".** **Tests:** manual — start a dev build, confirm PM3 does not spawn the installed build for at least two poll cycles (120s). |
| E014-T07 | Demotion/promotion integration | 4 | 8 | High | pm | Wire PM3's candidate demotion to consume MoveUp's ordered list (T04) and to call back into T02's marker on a successful promotion. **BLOCKED on the same PM3 backlog item as T05.** **Tests:** integration test installs a deliberately broken "newest" build, confirms PM3 demotes to `last-known-good` within two 60s poll cycles, and confirms the broken build is never re-promoted until fixed. |
| E014-T08 | End-to-end verification | 4 | 5 | High | verify | Verify the full loop: install a broken build over a working one, confirm two-miss detection (D3), confirm rollback to last-known-good, confirm no respawn while a dev build holds the singleton, confirm the Windows Run key is untouched. **Tests:** see `## Test strategy` below — this task executes that strategy end to end, not new tests of its own. |
| E014-T09 | Epic setup: version bump + ADRs + docs | 1 | 5 | Medium | main | Bump `package.json`/`src-tauri/tauri.conf.json`/`src-tauri/Cargo.toml` to `0.6.0` per [`.claude/rules/versioning.md`](../../../.claude/rules/versioning.md) (not performed by this plan — recorded as a task); write the two ADRs named below; update `.plan/ARCH.md` for the new release-store directory and the PM3/app supervision boundary. **Tests:** none — documentation/config task, verified by review only. |

Total: 9 tasks, **40 points**. Wave 1 is 13 points, wave 2 is 8, wave 3 is 6,
wave 4 is 13, so no wave exceeds the 40-point split threshold.

Waves 1-2 (T01-T04, T09) are **not blocked** — they build MoveUp's half, which
is independently useful even before PM3 gains candidate lists. Waves 3-4
(T05-T08) are **blocked** on the two PM3 backlog items filed alongside this
plan (see `## Decisions and ADRs`).

## Decisions and ADRs

This plan applies decisions D1-D7, settled before planning and not reopened
here:

- **D1 — Ownership split.** PM3 owns the supervision loop (poll, two-miss
  rule, stand-down, demotion/promotion). MoveUp owns the release store,
  `last-known-good` marker, health contract, retention, and the launch
  contract (`--minimized`). The loop is generic process supervision and
  belongs to the process manager; keeping builds and deciding what "good"
  means is app knowledge.
- **D2 — The Windows `Run` key stays the login path.** PM3 does not replace
  it; it fills the gap after login and performs rollback. When PM3 polls and
  the app is already running, it does nothing.
- **D3 — Two consecutive misses, not one, at 60s spacing.** A single-miss
  rule would race the dev launcher's kill-then-start sequence and relaunch
  the installed build into the dev build's face.
- **D4 — Occupancy is detected by singleton ownership, not executable path.**
  Dev and installed builds share bundle id `io.zntl.desk` and therefore share
  `tauri-plugin-single-instance` (`src-tauri/src/lib.rs:136`). A supervisor
  matching only the installed path would respawn the installed build every
  poll during a dev session, and every such spawn would exit immediately
  against the singleton.
- **D5 — Definition of a good build.** Process alive 30s AND `GET
  http://127.0.0.1:<DESK_REMOTE_PORT>/display/api` returns JSON within the
  health timeout. Process-alive alone is not sufficient.
- **D6 — Candidate order and retention.** Newest installed build, then
  last-known-good, then the next older retained build. Retain 3 builds.
  Retention and ordering are MoveUp's responsibility; PM3 only consumes the
  ordered list.
- **D7 — Sequencing.** MoveUp's half is built first (Waves 1-2) because it is
  independently useful and unblocks nothing else; the loop (Waves 3-4) waits
  on PM3.

New ADRs required before implementation of Waves 3-4:

1. **Ownership split between PM3 and the app** — records D1/D2/D4/D7 as an
   architectural decision, not just a plan note, because it sets a precedent
   other apps on this machine will follow.
2. **Release-store layout and retention policy** — records the directory
   layout, the 3-build retention count, and the `last-known-good` marker
   format from D5/D6.

Both ADRs go in `.plan/ADR/` per [`plan-arch-structure.md`](../../../../.claude/rules/plan-arch-structure.md);
task E014-T09 writes them.

This plan depends on [E013 — Signed Tauri release and PM3
deployment](../E013-2026-08-28-signed-tauri-pm3-deployment/PLAN.md), whose
Wave 4 puts the installed executable under plain PM3 supervision (autorestart,
health check, no candidate list). E014 extends that with rollback; it does
not duplicate it.

## Architecture impact

Two additions to the system:

- A new **release store directory** on the MoveUp side
  (`%LOCALAPPDATA%\MoveUp\releases\<version>\`, plus a `last-known-good`
  marker and a candidate-order file PM3 reads).
- A new **supervision boundary** between PM3 and the app: PM3 owns the
  poll/demote/promote loop and singleton-aware stand-down; MoveUp owns what
  "good" means and which builds exist. This boundary is the precedent
  Approach B extends to every future PM3-supervised app on this machine.

E014-T09 updates `.plan/ARCH.md` to describe both additions.

## Alternatives considered

**A. Minimum — PowerShell watchdog + Scheduled Task.**
Summary: a PowerShell script polls the app, restarts it, and keeps its own
rollback list, driven by a Windows Scheduled Task.
Effort: S. Risk: M.
Reuses: nothing.
Pros: ships without touching PM3; no dependency on another repo's roadmap.
Cons: puts a second, competing process supervisor on a machine whose
convention is that PM3 owns long-lived processes; its rollback logic is
private to this one app and every future app repeats the same watchdog from
scratch.

**B. Target (recommended) — PM3 gains candidate lists.**
Summary: PM3's service schema gains an ordered candidate list, demotion on
repeated health failure, promotion on a proven-good newer candidate, and a
singleton-aware stand-down check. MoveUp becomes a thin consumer with a
`pm3.yaml`.
Effort: M to L. Risk: M.
Reuses: PM3's existing `health`, `autorestart`, and `backoff` machinery
(`src/config-schema.ts:47-62,76-78`) and E013 Wave 4's planned health check
against `/display/api`.
Pros: every future app supervised by PM3 on this machine gets rollback for
free, not just MoveUp.
Cons: blocked on work in another repo (`pm3-mcp`), on a timeline this plan
does not control.

**C. In-app self-supervision.**
Rejected outright. A process that has died cannot restart itself, and a build
that fails to serve cannot judge itself good or bad from the inside — the
judge has to be a separate, still-alive process. This is not a tradeoff
against A or B; it does not solve the problem at all.

**Recommendation: B**, with MoveUp's half (Waves 1-2) built first per D7 so
value ships even while PM3's half is pending.

## Acceptance criteria

- [ ] A release store under `%LOCALAPPDATA%\MoveUp\releases\` holds at most 3
      versioned builds, oldest pruned on install of a 4th.
- [ ] A build is marked `last-known-good` only after passing the D5 health
      probe (process alive 30s AND `/display/api` returns JSON).
- [ ] The ordered candidate list (D6) is available for PM3 to read.
- [ ] `pm3.yaml` declares the MoveUp service with `health` against
      `/display/api`, once E014-T05/T06/T07 are unblocked.
- [ ] PM3 does not respawn the installed build while a dev build holds the
      `io.zntl.desk` singleton, verified over at least two 60s poll cycles.
- [ ] Installing a deliberately broken "newest" build results in PM3 demoting
      to `last-known-good` within two poll cycles (roughly 2 minutes), and the
      broken build is not re-promoted until it is fixed.
- [ ] The Windows `Run` key registration is untouched by this epic (D2).

## Test strategy

Per feature, the kind of test, the assertion that would fail today, and where
the test file goes. Rust tests are a sibling `<module>_tests.rs` registered
in `lib.rs`; TypeScript tests are co-located `<module>.test.ts(x)`.

- **Release store retention** — Rust unit test in a new
  `release_store_tests.rs` (registered in `lib.rs`). Assertion that fails
  today: there is no release store at all, so any retention assertion fails
  by definition (`cargo test release_store` finds no such module).
- **`last-known-good` marker** — Rust unit test, same file. Assertion that
  fails today: no marker file exists, and no code path writes one after a
  probe succeeds.
- **Health-good probe (D5)** — Rust unit test in `health_probe_tests.rs`,
  three cases: process-alive-only (must fail), serves-then-dies-early (must
  fail), both conditions met (must pass). All three fail today because no
  probe function exists.
- **Candidate order writer** — Rust unit test, same suite as release store.
  Assertion that fails today: no candidate-list file is produced.
- **PM3 candidate list + demotion/promotion** — depends on the PM3-side
  feature; once available, an integration test in `pm3-mcp`
  (`src/candidate-supervision.test.ts` or equivalent, per that repo's
  convention) asserts demotion after N consecutive health failures and
  promotion after a newer candidate proves good. That test file belongs to
  the PM3 backlog item, not to this plan.
- **Singleton stand-down** — manual verification task (E014-T06): start a dev
  build, observe PM3's poll log for at least two cycles, confirm no spawn
  attempt against the installed build while the dev build holds the
  singleton.
- **End-to-end rollback (E014-T08)** — manual/scripted verification: install
  a build whose `/display/api` deliberately returns a non-JSON body, confirm
  PM3 demotes to `last-known-good` within two poll cycles, confirm the
  Windows Run key registry value is unchanged before and after.

Four shadow paths (happy/nil/empty/error), applied to the health probe and
candidate-list reader specifically:

- **happy** — process alive 30s, `/display/api` returns valid JSON → marked
  good, promoted.
- **nil** — no release store exists yet (fresh install) → candidate list has
  exactly one entry, no crash.
- **empty** — release store directory exists but is empty (pruning bug, or
  first run before any build completes) → candidate list is empty, PM3's
  consumer must not crash on an empty list, must report "no candidate
  available" rather than silently doing nothing.
- **error** — `/display/api` call throws (port not listening, connection
  reset) → treated as a failed probe, not as a good build, and not as a
  crash of the probing code itself.

## Out of scope

- Building a PM3 candidate-list feature inside this repo. That code lives in
  `pm3-mcp` and is tracked in that repo's own backlog
  (`int://mATX.lan/C:/code/pm3-mcp/.plan/BACKLOG.md`).
- Replacing the Windows `Run` key as the login mechanism (D2).
- Any change to ergonomics, sensor, or dashboard product behavior.
- Code signing, CI release pipeline, and `moveup.internal` domain
  registration — those are E013's scope, not repeated here.
- Cross-platform (non-Windows) release supervision.

## Risks

| Risk | Mitigation / gate |
| --- | --- |
| PM3's candidate-list feature never lands, or lands with a different shape than assumed here | Waves 1-2 still ship value alone (release store, last-known-good marker); Waves 3-4 stay blocked and visible in this plan rather than silently stalling |
| Singleton stand-down misdetects and starves the installed build even after a dev session ends | E014-T06's manual test explicitly confirms recovery after the dev build exits, not only non-interference during it |
| A "good" build that later degrades (serves JSON at startup, then silently breaks) is never caught | Out of scope for this epic — D5's probe runs once per install/promotion, not continuously; a future epic could extend health monitoring, noted here so it is not lost |
| Release store grows unbounded if pruning has a bug | E014-T01's test asserts the 4th install prunes; add a periodic disk-usage sanity check as a follow-up if a bug is found in practice |

## Files referenced

- [`.plan/epics/E013-2026-08-28-signed-tauri-pm3-deployment/PLAN.md`](../E013-2026-08-28-signed-tauri-pm3-deployment/PLAN.md) — depended-on epic, Wave 4
- `src-tauri/src/lib.rs:136` — `tauri-plugin-single-instance` wiring
- `src-tauri/src/setup_helpers.rs` — `ensure_autostart` no-op in debug builds
- `scripts/tauri-dev.ps1` — dev launcher process guard (kills installed build)
- `int://mATX.lan/C:/code/pm3-mcp/src/config-schema.ts:47-62,76-78` — PM3's
  existing `health`/`autorestart`/`backoff` schema, today single-candidate only
- `int://mATX.lan/C:/code/pm3-mcp/.plan/BACKLOG.md` — the two PM3 backlog
  items this epic's Waves 3-4 are blocked on
- [`.claude/rules/versioning.md`](../../../.claude/rules/versioning.md) —
  epic version-bump convention (0.6.0)
- [`.plan/BACKLOG.md`](../../BACKLOG.md) — `## Planned Epics` and `##
  Autostart — findings 2026-09-05` entries updated alongside this plan
