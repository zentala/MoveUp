---
formatVersion: 1
type: handoff
status: todo
---

# E017 Handoff — Release readiness

## TLDR

Implementing session reads only this file plus [`PLAN.md`](PLAN.md). Eight
AO tasks in two waves (T01-T07 parallel, T08 closes them out), then a
separate "Outside AO" list this epic tracks but cannot automate: signing
decision, first-run browser pass, full build gate, tagging. Work in a
worktree via `wt-add`, branch `feat/E017-release-readiness`. Do not start
before E016 (plan hygiene) and E015 (engine single truth, bumps version to
0.6.0) have landed — this epic assumes `0.6.0` is already the version
everywhere.

## Decisions already made (apply unless Paweł overrides)

- D3: licence = MIT. Write the standard MIT license text with copyright
  holder "Paweł Żentała" and the current year.
- D4: this epic never runs before E015 — the engine fix ships first.
- Version 0.6.0 is already set by E015; this epic does not bump it again.
  Tagging (`git tag -a v0.6.0`) happens only in the Outside-AO build gate,
  after it passes — never as part of T01-T08.
- Verification scripts are real files (`scripts/check-e017-*.mjs`), not
  inline `node -e` one-liners — a repo hook blocks certain literal command
  content, and a file is easier for the next agent to read anyway. Each
  script is part of its task's `write_set`/`claims`.
- The updater is explicitly NOT built in this epic. `USER_UPDATES.md`
  describes the manual path; file the real updater as a follow-on backlog
  item in `.plan/BACKLOG.md` during T02 (one entry, `path:line` to
  `USER_UPDATES.md`, Importance Medium, Points 8 per the "target
  architecture" scale used elsewhere in the review).

## Mental model

- **Product identity, source of truth**: `src-tauri/tauri.conf.json:3`
  (`productName: "MoveUp"`), data dir `%LOCALAPPDATA%\MoveUp\`, repo
  `git@github.com:zentala/MoveUp.git`. Every doc must match these three
  facts, never the old `zntlDesk` / `AppData\Local\zntlDesk\` /
  `github.com/zentala/zntl-tray` values.
- **The five docs to rewrite**: `docs/README.md`, `docs/USER_INSTALL.md`,
  `docs/USER_SUPPORT.md`, `docs/USER_UPDATES.md`, `docs/PRIVACY.md`. Two
  more get the same naming sweep but no content rewrite:
  `docs/REMOTE_DISPLAY.md`, `docs/OPTIMIZATION_GUIDE.md`.
- **Cable-sensitivity content to copy (not summarize)**: `CLAUDE.md` →
  Hardware → "Cable sensitivity" paragraph, verbatim, into
  `USER_INSTALL.md`/`USER_SUPPORT.md`'s troubleshooting sections and into
  `firmware/README.md`. It documents three real, verified failure modes
  (charge-only cables, C-to-C CC-resistor dependency, voltage drop on thin/
  long cables) — do not paraphrase into something vaguer.
- **Google Fit facts to disclose**: `src-tauri/src/google_fit.rs` makes real
  OAuth2 calls to `googleapis.com`; scope is
  `fitness.activity.read`; opt-in (missing `.env` keys = feature no-ops per
  `CLAUDE.md` → "Google Fit Integration"). `PRIVACY.md` must say this
  plainly and drop the blanket "no data leaves this machine" claim.
- **License text lives at repo root**: `LICENSE` (no extension), standard
  MIT boilerplate. `package.json` top-level `"license": "MIT"`.
  `src-tauri/Cargo.toml` `[package]` gets `license = "MIT"` next to the
  existing `description` line — fix that line's text too
  (`"zntl Desk — VL53L1X ToF sensor reader"` → something naming MoveUp).
- **`.github/workflows/test.yml`** (`.github/workflows/test.yml:33,38,43,
  48,53,58,63,70,80,86,95,99,133,146,161`) has every `working-directory` and
  `cache-dependency-path` pointed at `apps/desk` — a path that does not
  exist (`ls apps` → not found). Change every one to the repo root (drop the
  `apps/desk` segment entirely). Do not touch `release-baseline.yml` — it
  already works and is the correct release-build job (E013 Wave 1, already
  merged to `main`).
- **Firmware**: only file is `firmware/tof_reader/tof_reader.ino`. New
  `firmware/README.md` needs: what board (XIAO ESP32-C3), what sensor
  (Grove VL53L1X v2, mounted under the desk pointing down), how to flash
  (Arduino IDE or PlatformIO, board package URL, baud rate if non-default),
  what output confirms success (`DEVICE: zntl-desk-sensor v1` on connect,
  per `CLAUDE.md` → Hardware → "Auto-detection"), and the cable warning.
- **`.perf-baseline.json`** is written by `pnpm test:perf`
  (`vitest run --config vitest.perf.config.ts tests/perf`) — no full Tauri
  build needed, safe to run as an AO task.
- **`.build-sizes.json`** is written by `scripts/build-report.cjs`, which
  only runs as part of `pnpm tauri:build` (`package.json:14`) — that
  requires a full release build and is Outside AO (see below). Do not try to
  refresh it from an AO task.

## Tasks

- [ ] **T01** (3, ts-dev) — rewrite `docs/README.md`, `USER_INSTALL.md`,
  `USER_SUPPORT.md`, `REMOTE_DISPLAY.md`, `OPTIMIZATION_GUIDE.md` to MoveUp
  naming/paths/repo; add the cable-sensitivity section to
  `USER_INSTALL.md`/`USER_SUPPORT.md`. Verify:
  `node scripts/check-e017-t01.mjs`.
- [ ] **T02** (2, ts-dev) — rewrite `docs/USER_UPDATES.md` for the real
  manual update path; file the updater follow-on in `.plan/BACKLOG.md`.
  Verify: `node scripts/check-e017-t02.mjs`.
- [ ] **T03** (2, ts-dev) — rewrite `docs/PRIVACY.md`: naming sweep + Google
  Fit disclosure section. Verify: `node scripts/check-e017-t03.mjs`.
- [ ] **T04** (2, main) — add `LICENSE` (MIT), `license` field in
  `package.json` and `src-tauri/Cargo.toml`, fix `Cargo.toml` `description`.
  Verify: `node scripts/check-e017-t04.mjs`.
- [ ] **T05** (3, ts-dev) — fix `.github/workflows/test.yml` to run from the
  repo root; leave `release-baseline.yml` untouched. Verify:
  `node scripts/check-e017-t05.mjs`.
- [ ] **T06** (3, main) — write `firmware/README.md` (flashing + cable
  warning). Verify: `node scripts/check-e017-t06.mjs`.
- [ ] **T07** (1, main) — refresh `.perf-baseline.json` via
  `pnpm test:perf`. Verify: `node scripts/check-e017-t07.mjs`.
- [ ] **T08** (2, main) — aggregate check across T01-T07; append `D3` to
  `.plan/decisions.jsonl` (create the file if E015 has not already created
  it). Verify: `node scripts/check-e017-t08.mjs`.

## Outside AO

These are not in the YAML contract below — each needs a human decision, a
real browser, or a multi-minute release build that does not fit a 30-90
minute AO task budget.

- **Signing-provider decision + integration** (E013 Wave 2, ~8 pts decision
  + ~5 pts CI wiring). Go through `consent-broker` or a direct conversation
  with Paweł: inventory available signing services (see
  `.plan/epics/E013-2026-08-28-signed-tauri-pm3-deployment/PLAN.md` Wave 2
  for the constraints — no exportable private key in the repo, must support
  unattended CI signing). Once chosen, a follow-on task adds the signing
  step to `release-baseline.yml` (or a new signed-release workflow) and
  verifies `Get-AuthenticodeSignature` passes on the output.
- **First-run browser pass** (3 pts, agent `browser`): welcome window →
  calibration → no-sensor state, on the real rendered app, per the global
  "never claim it works" rule. Dispatch to the `browser` agent — do not do
  this from the main loop.
- **Full `pnpm tauri:build` go/no-go gate** (2 pts, main): run the complete
  build from repo root. On success, `scripts/build-report.cjs` regenerates
  `.build-sizes.json` — use its fresh numbers to fix the 60-70 MB target in
  `.claude/rules/installer.md` against the real ~4-6 MB output (or confirm
  the new build has grown closer to that historical target — do not assume
  the old number is simply wrong without checking the fresh build's actual
  size first).
- **Tag `v0.6.0`**: only after the build gate above passes. `git tag -a
  v0.6.0 -m "v0.6.0 — E015 engine fix + E017 release readiness"`, then
  `git push origin --tags`, per `.claude/rules/versioning.md`.

None of these four items is optional — they are release blockers, just ones
that cannot be expressed as an AO `verification` command. Track them here as
a checklist so `done.` on this epic checks all four were actually completed,
not silently dropped.

## Done means

All ten acceptance criteria in PLAN.md hold, all eight evidence records for
T01-T08 are `current`, the four Outside-AO items above are checked off with
a link to their evidence, `.plan/HISTORY.md` gets an E017 entry, and
`STATE.md` is updated to show E017 done and the tag pushed.

## AO

```yaml
project: MoveUp
epic: E017
base_ref: main
tasks:
  - id: E017-T01
    repo: MoveUp
    executor: ts-dev
    depends_on: []
    write_set: ["docs/README.md", "docs/USER_INSTALL.md", "docs/USER_SUPPORT.md", "docs/REMOTE_DISPLAY.md", "docs/OPTIMIZATION_GUIDE.md", "scripts/check-e017-t01.mjs"]
    claims: ["docs/README.md", "docs/USER_INSTALL.md", "docs/USER_SUPPORT.md", "docs/REMOTE_DISPLAY.md", "docs/OPTIMIZATION_GUIDE.md", "scripts/check-e017-t01.mjs"]
    verification: "node scripts/check-e017-t01.mjs"
    budget_minutes: 60
  - id: E017-T02
    repo: MoveUp
    executor: ts-dev
    depends_on: []
    write_set: ["docs/USER_UPDATES.md", "scripts/check-e017-t02.mjs", ".plan/BACKLOG.md"]
    claims: ["docs/USER_UPDATES.md", "scripts/check-e017-t02.mjs", ".plan/BACKLOG.md"]
    verification: "node scripts/check-e017-t02.mjs"
    budget_minutes: 45
  - id: E017-T03
    repo: MoveUp
    executor: ts-dev
    depends_on: []
    write_set: ["docs/PRIVACY.md", "scripts/check-e017-t03.mjs"]
    claims: ["docs/PRIVACY.md", "scripts/check-e017-t03.mjs"]
    verification: "node scripts/check-e017-t03.mjs"
    budget_minutes: 45
  - id: E017-T04
    repo: MoveUp
    executor: ts-dev
    depends_on: []
    write_set: ["LICENSE", "package.json", "src-tauri/Cargo.toml", "scripts/check-e017-t04.mjs"]
    claims: ["LICENSE", "package.json", "src-tauri/Cargo.toml", "scripts/check-e017-t04.mjs"]
    verification: "node scripts/check-e017-t04.mjs"
    budget_minutes: 30
  - id: E017-T05
    repo: MoveUp
    executor: ts-dev
    depends_on: []
    write_set: [".github/workflows/test.yml", "scripts/check-e017-t05.mjs"]
    claims: [".github/workflows/test.yml", "scripts/check-e017-t05.mjs"]
    verification: "node scripts/check-e017-t05.mjs"
    budget_minutes: 45
  - id: E017-T06
    repo: MoveUp
    executor: ts-dev
    depends_on: []
    write_set: ["firmware/README.md", "scripts/check-e017-t06.mjs"]
    claims: ["firmware/README.md", "scripts/check-e017-t06.mjs"]
    verification: "node scripts/check-e017-t06.mjs"
    budget_minutes: 45
  - id: E017-T07
    repo: MoveUp
    executor: ts-dev
    depends_on: []
    write_set: [".perf-baseline.json", "scripts/check-e017-t07.mjs"]
    claims: [".perf-baseline.json", "scripts/check-e017-t07.mjs"]
    verification: "node scripts/check-e017-t07.mjs"
    budget_minutes: 30
  - id: E017-T08
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E017-T01", "E017-T02", "E017-T03", "E017-T04", "E017-T05", "E017-T06", "E017-T07"]
    write_set: [".plan/decisions.jsonl", "scripts/check-e017-t08.mjs"]
    claims: [".plan/decisions.jsonl", "scripts/check-e017-t08.mjs"]
    verification: "node scripts/check-e017-t08.mjs"
    budget_minutes: 45
```
