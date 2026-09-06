---
formatVersion: 1
type: epic
status: todo
readiness: ready
points: 18
agent: ts-dev
wave: 3
parallel: []
depends-on: [E016, E015]
blocked-by: ""
---

# E017 — Release readiness

Status: planned (2026-09-06). Runs after E016 (plan hygiene) and E015 (engine
single truth, bumps version to 0.6.0). Source review:
[`../../reports/2026-09-06-pelny-przeglad-architektury-i-release.md`](../../reports/2026-09-06-pelny-przeglad-architektury-i-release.md)
§4, §6 (E017 table) and
[`../../reports/_review-2026-09-06/release.md`](../../reports/_review-2026-09-06/release.md)
(full 12-step checklist and GAPS). Board deck (Polish): [`PRES.md`](PRES.md).
Handoff: [`HANDOFF.md`](HANDOFF.md).

## TLDR

MoveUp builds clean and sends no telemetry, but every user-facing document
still describes a different, retired product ("zntlDesk", old paths, old
repo), claims an auto-updater that does not exist in code, and the app ships
with no LICENSE despite an open-core business model (ADR 005). CI has a
release-baseline workflow that already builds an unsigned installer
(E013 Wave 1, done) sitting next to a test workflow that targets a
`apps/desk/` monorepo layout that no longer exists. This epic rewrites the
five user docs, adds the missing license and Google Fit disclosure, fixes or
retires the broken CI workflow, documents firmware flashing, and closes the
metadata/version-hygiene gaps that block calling this shippable — 18 AO
points across two waves, plus a manual build/sign/verify pass explicitly
outside AO scope.

## Problem

The [full architecture review](../../reports/2026-09-06-pelny-przeglad-architektury-i-release.md)
§4 and the dedicated
[release review](../../reports/_review-2026-09-06/release.md) found ten
release blockers, almost all documentation and process, not code:

1. `docs/README.md`, `USER_INSTALL.md`, `USER_SUPPORT.md`, `PRIVACY.md`,
   `REMOTE_DISPLAY.md` say "zntlDesk" / `AppData\Local\zntlDesk\` /
   `github.com/zentala/zntl-tray` (`release.md` blocker #1, 93 hits across 6
   files, `docs/PRIVACY.md:3,16,145`, `docs/USER_INSTALL.md:3,15,65`, etc.).
2. `docs/USER_UPDATES.md` describes a working 24h auto-update flow; no
   `tauri-plugin-updater` exists anywhere in `Cargo.toml`, `tauri.conf.json`,
   or `package.json` (`release.md` blocker #2).
3. No LICENSE file, no `license` field in `package.json` or
   `src-tauri/Cargo.toml`, despite ADR 005's open-core commitment
   (`release.md` blocker #4).
4. `docs/PRIVACY.md` claims the app "does NOT send any data to servers", but
   `src-tauri/src/google_fit.rs` makes real OAuth2 + `googleapis.com` calls
   (`release.md` blocker #5).
5. `.github/workflows/test.yml` still references `apps/desk/` — a monorepo
   path this repo no longer has (`ls apps` → not found) — on every push/PR to
   `main` (`release.md` blocker #7). `release-baseline.yml` (E013 Wave 1) is
   already the correct release-build CI job and needs no rework here.
6. `firmware/tof_reader/tof_reader.ino` ships with no README, no flashing
   instructions, and no cable-sensitivity warning, though
   `CLAUDE.md` → Hardware documents a verified, non-obvious cable failure mode
   (release.md blocker #8, and the architecture report's own "Sekcja o
   kablach USB nie trafiła do docs wsparcia" row).
7. `src-tauri/Cargo.toml:4` still reads `description = "zntl Desk — VL53L1X
   ToF sensor reader"` while `tauri.conf.json` already says `MoveUp`
   (`release.md` blocker #9).
8. `.perf-baseline.json` is dated 2026-03-16, six months before Analyst,
   Google Fit, and remote display shipped; `.claude/rules/installer.md`
   quotes a 60-70 MB installer target against an observed ~4.3 MB NSIS /
   6.2 MB MSI (`release.md` "Should-fix" section).
9. No first-run browser verification and no full `pnpm tauri:build` have
   ever been run against this revision (`release.md` GAPS) — the "never
   claim it works" rule blocks calling this release-ready without both.
10. Code-signing has no chosen provider; E013 Wave 2 has not started
    (`release.md` blocker #3). This is the one multi-day blocker and is
    explicitly a human decision, not something this epic's AO tasks can
    resolve.

## Decisions and ADRs

- **D3 (owner decision, 2026-09-06): licence = MIT.** Implements
  [ADR 005](../../../.arch/ADR/005-open-core-software-model.md)'s open-core
  model, which named MIT or Apache 2.0 as the open-source half of the split
  and left the choice open. T04/T08 record `D3` in `.plan/decisions.jsonl`
  (created by E015-T04 if that epic ran first; created here if not — see
  Mental model in HANDOFF.md).
- **D4 (owner decision, 2026-09-06): fix the engine (E015) before release.**
  This epic is sequenced after E015 in `depends-on`; nothing here should run
  on a branch where E015 has not landed, since the popup timer bug would
  still be visible to a first user.
- **Version 0.6.0** is bumped by E015 (`.claude/rules/versioning.md`,
  "before starting the first task of a new epic"); this epic's Outside-AO
  step tags it (`git tag -a v0.6.0`) only after the full-build go/no-go gate
  passes — never before.
- `.plan/decisions.jsonl` — may not exist yet at plan time (E015-T04 creates
  it with D1/D2 if E015 ran first). This epic's T08 appends D3, creating the
  file if it is still absent.
- No other applicable ADR beyond 005; this epic does not choose an
  architecture, only a license and a documentation baseline.

## Alternatives

| | A. Minimum | B. Full checklist (recommended) | C. Build the updater and signing now |
|---|---|---|---|
| Summary | Fix only naming (blocker #1) and add LICENSE (#4); ship with everything else as-is | Fix all ten blockers from `release.md` except signing (owner decision, out of AO) plus the two Should-fix items (perf/build-size refresh, `test.yml`) | Implement a real `tauri-plugin-updater` and finish E013 Wave 2 signing inside this epic, so `USER_UPDATES.md` never needs a "manual for now" caveat |
| Effort | S (5) | M (18 AO + ~19 outside AO) | XL (34+, blocked indefinitely on a signing-provider decision that has no owner yet) |
| Risk | H — a stranger still hits a broken CI badge, an undocumented cable failure, an unlicensed repo claiming open-core, and a privacy policy that is factually false about Google Fit | M — sequenced, bounded, defers signing to its own decision instead of blocking on it | H — conflates a documentation/process epic with two multi-week feature builds (updater infra, signing infra), directly against D4 ("fix the engine before release", not "build everything before release") |
| Pros | Fast; answers the two most visible embarrassments | Every `release.md` blocker gets either fixed or an explicit Outside-AO owner; CI stops lying about its own layout; firmware buyers get a real flashing guide | Removes the "manual update" caveat permanently |
| Cons | Leaves a legally false privacy claim, a broken CI workflow lying to every PR, and an unflashable firmware kit | Ships with a manual update path (acceptable per the owner's explicit scope: "no updater in code — do not plan implementing the updater, file it as a follow-on") | Scope creep the owner explicitly rejected for this pass; would delay release by weeks waiting on a signing certificate |
| Reuses | `docs/` structure, existing `release-baseline.yml` | same, plus `CLAUDE.md` Hardware section content for the cable warning, ADR 005 for the licence choice | everything from B, plus new updater/signing code |

**Recommendation: B.** The owner's own scope statement rules out C
explicitly (updater is a follow-on, signing is a separate decision-gated
task), and A leaves a factually false privacy policy live, which is worse
than shipping a day later with it fixed.

## Scope

**In (this epic's AO tasks):** `docs/README.md`, `docs/USER_INSTALL.md`,
`docs/USER_SUPPORT.md`, `docs/USER_UPDATES.md`, `docs/PRIVACY.md`,
`docs/REMOTE_DISPLAY.md`, `docs/OPTIMIZATION_GUIDE.md`, `LICENSE`,
`package.json` (license field only), `src-tauri/Cargo.toml` (license and
description fields only), `.github/workflows/test.yml`,
`firmware/README.md`, `.perf-baseline.json`, `.plan/decisions.jsonl` (append
D3), plus one `scripts/check-e017-*.mjs` verification script per task.

**In (Outside AO, this epic's HANDOFF tracks but does not automate):**
signing-provider decision and integration (E013 Wave 2), first-run browser
pass, full `pnpm tauri:build` go/no-go gate, `.build-sizes.json` refresh and
reconciling the `installer.md` size target against real numbers, tagging
`v0.6.0`.

**Out:** implementing an auto-updater (explicit owner instruction: file as
follow-on, do not build); rewriting the engine (E015); frontend
consolidation (E018); backend hardening (E019); E013 Waves 3-5 (PM3
deployment on the developer's own machine — a different audience than a
public release, see `release.md` "Should-fix" §E013/E014 scope note); E014
(supervised rollback, blocked on `pm3-mcp`, unrelated to public release).

## Tasks

| ID | Task | Points | Agent | Wave |
|---|---|---|---|---|
| T01 | Rewrite `docs/README.md`, `USER_INSTALL.md`, `USER_SUPPORT.md`, `REMOTE_DISPLAY.md`, `OPTIMIZATION_GUIDE.md` to MoveUp naming/paths/repo; add the USB cable-sensitivity section from `CLAUDE.md` Hardware to install/support docs | 3 | ts-dev | 1 |
| T02 | Rewrite `docs/USER_UPDATES.md` to describe the real manual update path (download from GitHub Releases, no auto-check); note the updater as a filed follow-on, not implemented here | 2 | ts-dev | 1 |
| T03 | Rewrite `docs/PRIVACY.md` to MoveUp naming and add a truthful Google Fit disclosure (what is sent to Google, when, opt-in nature) | 2 | ts-dev | 1 |
| T04 | Add `LICENSE` (MIT, D3), `license` field in `package.json` and `src-tauri/Cargo.toml`, fix `Cargo.toml` `description` from "zntl Desk" to MoveUp wording | 2 | main | 1 |
| T05 | Fix `.github/workflows/test.yml` to run from the repo root (remove all `apps/desk` references); confirm `release-baseline.yml` remains the sole release-build CI job | 3 | ts-dev | 1 |
| T06 | Write `firmware/README.md`: flashing instructions for the XIAO ESP32-C3 + VL53L1X sketch, and the verified cable-sensitivity warning from `CLAUDE.md` Hardware | 3 | main | 1 |
| T07 | Refresh `.perf-baseline.json` via `pnpm test:perf` | 1 | main | 1 |
| T08 | Aggregate verification across T01-T07 (no forbidden strings remain, LICENSE + license fields present, workflow clean); append D3 to `.plan/decisions.jsonl` | 2 | main | 2 |

Wave 1 = 16 points (T01-T07, disjoint files, no shared claims). Wave 2 = 2
points (T08, depends on all of Wave 1). Total AO: 18 points, well under the
40-point wave ceiling.

**Outside AO** (see HANDOFF.md → "Outside AO" for detail): signing-provider
decision + integration (~13 pts, E013 Wave 2), first-run browser pass
(3 pts), full `pnpm tauri:build` go/no-go + `.build-sizes.json` refresh +
`installer.md` reconciliation (2 pts), tag `v0.6.0` (1 pt).

## Test strategy

Every task here is a documentation/metadata task; "test" means a script that
fails today against the current repo state and passes once the task's
content lands. Each task writes its own `scripts/check-e017-<task>.mjs`
(part of its `write_set`) rather than an inline `node -e` one-liner, per the
repo's command-content hook.

| Task | Kind | Assertion that FAILS today | Script path |
|---|---|---|---|
| T01 | doc-content | `docs/README.md` etc. contain `zntlDesk`/`zntl-tray`/`Smart Desk` (case-insensitive); `USER_INSTALL.md`/`USER_SUPPORT.md` lack a cable-sensitivity section | `scripts/check-e017-t01.mjs` |
| T02 | doc-content | `USER_UPDATES.md` describes an automatic 24h check with no mention of "manual" or "GitHub Releases" | `scripts/check-e017-t02.mjs` |
| T03 | doc-content | `PRIVACY.md` contains the unqualified "does NOT send any data to servers" claim and no "Google Fit" section | `scripts/check-e017-t03.mjs` |
| T04 | file/metadata | `LICENSE` does not exist; `package.json`/`Cargo.toml` have no `license` field; `Cargo.toml` description still says "zntl Desk" | `scripts/check-e017-t04.mjs` |
| T05 | CI config | `.github/workflows/test.yml` contains the string `apps/desk` | `scripts/check-e017-t05.mjs` |
| T06 | file-existence | `firmware/README.md` does not exist / lacks "flash" and "cable" sections | `scripts/check-e017-t06.mjs` |
| T07 | file-freshness | `.perf-baseline.json` timestamp is `2026-03-16...` (stale) | `scripts/check-e017-t07.mjs` |
| T08 | aggregate | any of T01-T07's checks still fail, or `.plan/decisions.jsonl` has no `D3` entry | `scripts/check-e017-t08.mjs` |

Four-path coverage note: these are content/metadata checks, not runtime code
paths — there is no nil/empty/error branch to test here beyond "string
present or absent" and "file exists or not", which each script asserts
directly.

## Evidence contract

| check_id | class | procedure | expected | record |
|---|---|---|---|---|
| docs-naming | test | `node scripts/check-e017-t01.mjs` | pass | `evidence/records/T01-docs-naming.json` |
| updates-path | test | `node scripts/check-e017-t02.mjs` | pass | `evidence/records/T02-updates-path.json` |
| privacy-disclosure | test | `node scripts/check-e017-t03.mjs` | pass | `evidence/records/T03-privacy-disclosure.json` |
| license-metadata | test | `node scripts/check-e017-t04.mjs` | pass | `evidence/records/T04-license-metadata.json` |
| ci-workflow-root | test | `node scripts/check-e017-t05.mjs` | pass | `evidence/records/T05-ci-workflow-root.json` |
| firmware-readme | test | `node scripts/check-e017-t06.mjs` | pass | `evidence/records/T06-firmware-readme.json` |
| perf-refresh | test | `node scripts/check-e017-t07.mjs` | pass | `evidence/records/T07-perf-refresh.json` |
| aggregate-verify | test | `node scripts/check-e017-t08.mjs` | pass | `evidence/records/T08-aggregate-verify.json` |
| first-run-browser | visual | browser agent: welcome → calibration → no-sensor state | three characteristic UI states observed, console clean | `evidence/records/outside-ao-first-run.json` (manual, recorded by whoever runs it) |
| build-gate | manual | `pnpm tauri:build` from repo root | exits 0, produces NSIS+MSI under `src-tauri/target/release/bundle/` | `evidence/records/outside-ao-build-gate.json` |

## Architecture impact

None. No change to `.plan/ARCH.md` or any component/data-flow. This epic
touches documentation, licensing metadata, and CI configuration only. The
one durable-decision record is `.plan/decisions.jsonl` gaining `D3`
(licence = MIT) — a decision, not an architecture change, per
`rules/workflows.md` → "ADR trigger rules" (licence choice is not listed
there; `decisions.jsonl` is the correct home).

## Acceptance criteria

1. No occurrence of `zntlDesk`, `zntl-tray`, or `Smart Desk` remains in
   `docs/README.md`, `USER_INSTALL.md`, `USER_SUPPORT.md`, `USER_UPDATES.md`,
   `PRIVACY.md`, `REMOTE_DISPLAY.md`, `OPTIMIZATION_GUIDE.md`,
   `firmware/README.md`, `LICENSE`, `package.json`, or `src-tauri/Cargo.toml`.
2. `USER_INSTALL.md` and `USER_SUPPORT.md` each contain the cable-sensitivity
   troubleshooting content from `CLAUDE.md` Hardware.
3. `USER_UPDATES.md` describes the real manual update path; no claim of an
   automatic updater remains.
4. `PRIVACY.md` discloses the Google Fit data flow and drops the blanket
   "no data leaves this machine" claim.
5. `LICENSE` exists (MIT text); `package.json` and `src-tauri/Cargo.toml`
   both declare `license: "MIT"`; `Cargo.toml` `description` says MoveUp.
6. `.github/workflows/test.yml` contains no `apps/desk` reference and can run
   from the repo root; `release-baseline.yml` is unchanged and remains the
   release-build CI job.
7. `firmware/README.md` exists with flashing instructions and the cable
   warning.
8. `.perf-baseline.json` timestamp reflects a run made during or after this
   epic, not 2026-03-16.
9. `.plan/decisions.jsonl` contains a `D3` entry recording licence = MIT.
10. Signing-provider decision, first-run browser pass, and the full
    `pnpm tauri:build` go/no-go gate are recorded as done in HANDOFF.md's
    "Outside AO" section before anyone tags `v0.6.0` — none of them is
    silently skipped.
