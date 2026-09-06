---
formatVersion: 1
type: report
created: 2026-09-06
---

# Pre-release readiness review — MoveUp (2026-09-06)

## TLDR

MoveUp is not ready to ship to a stranger today, but the blockers are almost
all **documentation and process**, not code. The app builds clean (`tsc`,
`vite build`, `cargo check` all pass in well under two minutes), telemetry is
genuinely off by default and sends nothing over the network, and no real
secret is committed. The two things that would embarrass a first release are:
(1) every user-facing doc (`README` inside `docs/`, `USER_INSTALL.md`,
`USER_UPDATES.md`, `PRIVACY.md`, `docs/USER_SUPPORT.md`) still describes a
different, older product — name "zntlDesk", data path
`AppData\Local\zntlDesk\`, repo `github.com/zentala/zntl-tray` — and describes
a fully automatic 24h update flow that **does not exist in code** (no
updater plugin is wired at all); (2) there is no code-signing identity, no
public release artifact, and no LICENSE file despite the business plan
committing to an open-core, MIT/Apache app. E013 (signing + PM3) and E014
(rollback supervision) are explicitly scoped as **developer-machine
deployment work** (PM3 on `moveup.internal`), not the public-release path —
conflating the two would be a planning error. Google Fit integration also
sends data to a third party (Google) that the current Privacy Policy denies
happens at all ("does NOT send any data to servers"). Fix the docs, decide
signing/licensing, and re-verify first-run/no-sensor UX in a real browser
before calling this shippable.

## Release blockers

| # | Blocker | Evidence (path:line) | Importance | Points |
|---|---|---|---|---|
| 1 | User-facing docs describe a different product (name, install path, repo URL) than what ships | `docs/USER_INSTALL.md:3,15,65`, `docs/PRIVACY.md:3,16,145`, `docs/USER_UPDATES.md:3,21,45`, `docs/USER_SUPPORT.md` (10+ hits), `docs/README.md:1,68`, `docs/REMOTE_DISPLAY.md:8,15,57,67` — all say "zntlDesk" / `AppData\Local\zntlDesk\` / `github.com/zentala/zntl-tray`; actual `productName` is `MoveUp` (`src-tauri/tauri.conf.json:3`), actual data dir is `%LOCALAPPDATA%\MoveUp\` (`.plan/epics/E014.../HANDOFF.md:27`), actual remote is `git@github.com:zentala/MoveUp.git` | High | 3 |
| 2 | `USER_UPDATES.md` describes a working automatic-update system (24h check, tray notification, "Install Now") that is not implemented — no `tauri-plugin-updater` in `Cargo.toml`/`tauri.conf.json`/`package.json`, no updater config, no pubkey | `docs/USER_UPDATES.md:5-45`; absence confirmed via `grep updater src-tauri/Cargo.toml src-tauri/tauri.conf.json package.json` (zero hits); `.claude/rules/installer.md` "Auto-Update (Scaffolded) — Disabled in `tauri.conf.json`" | High | 2 |
| 3 | No code-signing identity exists; E013 Wave 2 (signing) has not started | `.plan/epics/E013-2026-08-28-signed-tauri-pm3-deployment/JOURNAL.md:8` ("No code-signing identity has been selected or exposed to this repository"), `scripts/sign-installer.sh` (placeholder), `.claude/rules/installer.md` "Code Signing (Scaffolded) — Not yet active" | High | 8 (procurement + CI wiring, per E013 Wave 2) |
| 4 | No LICENSE file and no `license` field in `package.json`/`Cargo.toml`, yet the business plan and Privacy Policy both assert an open-source/open-core model | `ls LICENSE*` → none found; `package.json` (no `license` key); `src-tauri/Cargo.toml:1-6` (no `license` key); `docs/PRIVACY.md:110,143` ("open-source", "verify the code yourself"); CLAUDE.md → ADR 005 "open-core software model" | High | 2 |
| 5 | Google Fit integration sends data to Google (a third-party server) on every steps fetch, but `PRIVACY.md` explicitly claims "does NOT send any data to servers (cloud, analytics, telemetry)" and never mentions Google Fit | `src-tauri/src/google_fit.rs` (OAuth2 + `googleapis.com` HTTP client), `CLAUDE.md` "Google Fit Integration" section; `docs/PRIVACY.md:20-25` contradicts this | High | 2 |
| 6 | `docs/USER_INSTALL.md`/`USER_SUPPORT.md` give only generic "check USB cable" troubleshooting; they omit the documented, verified failure mode (charge-only cables enumerate zero COM ports; C-to-C link needs the board's 5.1k CC resistors; thin/long cables brown out mid-enumeration) — a first-time user with the wrong cable will see nothing and no doc explains why | `CLAUDE.md` "Hardware" section (cable sensitivity, verified 2026-09-06); `docs/USER_INSTALL.md:92-98`, `docs/USER_SUPPORT.md:15-27` (generic only) | Medium | 2 |
| 7 | No CI job builds/tests the repo at its current root layout — `.github/workflows/test.yml` still uses `working-directory: apps/desk` and `apps/desk/pnpm-lock.yaml`, a monorepo path that no longer exists (repo root has no `apps/` dir); this job runs on every push/PR to `main` | `.github/workflows/test.yml:26,33,38,43,48,53,58,63,70,80,86,95,99,133,146,161` (repeated `apps/desk` refs); `ls apps` → not found; last modified 2026-09-04 (`git log -1 -- .github/workflows/test.yml`) without being fixed | High | 3 |
| 8 | Firmware for the required hardware ships as a bare `.ino` sketch with no README, flashing instructions, or version pin — a buyer of the "Dev Kit" (per business vision) has no documented path to a working sensor | `firmware/tof_reader/tof_reader.ino` (only file in `firmware/`); no `firmware/README.md` | High | 3 |
| 9 | `src-tauri/Cargo.toml` embeds the stale product description in the shipped binary's own metadata | `src-tauri/Cargo.toml:4` (`description = "zntl Desk — VL53L1X ToF sensor reader"`) vs `productName: "MoveUp"` in `tauri.conf.json:3` | Medium | 1 |
| 10 | `.plan/STATE.md` (the file agents read to know epic status) is stale by two epics and one version: says `active_epic: E011` with T01/T02 pending and `Current: v0.3.0`, while `.plan/DONE.md` shows E011 AND E012 fully done and the real version everywhere is `0.5.0` | `.plan/STATE.md:1-30` vs `.plan/DONE.md` (E011-T01..T05, E012-T01..T06 all `[x]`), `package.json:4`, `src-tauri/tauri.conf.json:4`, `src-tauri/Cargo.toml:3` (`0.5.0`) | Medium | 1 (fix STATE.md) |

## Should-fix before release

- **Version/tag hygiene**: only `v0.5.0` is tagged (`git tag -l`); no `v0.4.0` tag exists even though `DONE.md` and `E011-2026-05-07` clearly closed an epic that per `rules/versioning.md` should have bumped+tagged `0.4.0` before `E012` bumped to `0.5.0`. Not a functional blocker, but the "every epic = version bump + tag" contract in `.claude/rules/versioning.md` was skipped once; confirm E011/E012 history is otherwise intact.
- **`.build-sizes.json` vs installer.md target mismatch**: current NSIS installer is `4.32 MB` / MSI `6.21 MB` (`.build-sizes.json:2-3`), far below the documented 60-70 MB target in `.claude/rules/installer.md`. Likely means the target number itself is stale (was probably written before the app dropped a bundled WebView2 runtime or similar) — verify which number is right before quoting either to a buyer.
- **`.perf-baseline.json` is 6 months stale** (`timestamp: 2026-03-16`, before Analyst window, Google Fit, remote display expansion) — re-run `pnpm test:perf` before claiming the documented `<250MB peak` target still holds.
- **`catalog-info.yaml` declares `lifecycle: production`** while `README.md:26-29` says "MoveUp is in active development and is not yet distributed as a public installer" — pick one; Backstage catalog entries are read by other repos/agents as ground truth.
- **E011 epic close was never finished per its own checklist** — `ORCHESTRATOR.md`'s "Final integration" section still has `pnpm tauri:build` succeeds, manual E2E smoke, `.arch/HISTORY.md` update, and IMPRO/IMPROVEMENTS triage unchecked, even though the individual tasks all merged. `.plan/HISTORY.md` is empty (no epic narrative recorded anywhere despite 12+ closed epics in `DONE.md`).
- **Coverage/perf artifacts tracked in git** (see repo hygiene below) bloat the repo and go stale between commits — either regenerate on CI or gitignore them, don't hand-commit.
- **E013/E014 scope note (do not conflate with public release)**: E013's stated scope is "PM3 supervision on the developer machine" + `moveup.internal` for the *developer's own* browser dashboard; its own "Out of scope" section explicitly excludes "Publishing MoveUp to external users, automatic updater infrastructure, and mobile distribution" (`E013 PLAN.md`). E014's rollback/candidate-list work is likewise about the installed build on the developer's own machine, blocked on two `pm3-mcp` backlog items. Neither epic gets a stranger a working installer — they are dev-ops hardening for zentala's own dogfood machine.

## Nice-to-have

- Rust `cargo check` emits 2 dead-code warnings (`exe_from_command_line` in `setup_helpers.rs:341`, `tray_menu_item_ids` in `tray.rs:229`) — harmless, but worth deleting per the repo's own "delete fully" code-style rule.
- `docs/OPTIMIZATION_GUIDE.md` still says "zntlDesk" and references `taskkill /IM zntlDesk.exe` — low-traffic doc, fix in the same pass as the other four.
- `docs/superpowers/plans/2026-03-23-*.md` reference the old monorepo path `C:/code/zntl-tray/apps/desk/...` — historical planning artifacts, fine to leave as-is (they're dated records, not live docs) but worth a one-line note if anyone reuses them as a template.

## Verbatim command results

### `git tag -l`
```
v0.5.0
```

### Version consistency
```
package.json:            "version": "0.5.0"
src-tauri/tauri.conf.json: "version": "0.5.0"
src-tauri/Cargo.toml:    version = "0.5.0"
```
All three agree with each other and with the one existing tag. `.plan/STATE.md` alone disagrees (`Current: v0.3.0`) — see blocker #10.

### `node_modules/.bin/tsc --noEmit`
```
(no output — exit 0, ~2.3s)
```

### `node_modules/.bin/vite build` (tail)
```
✓ 112 modules transformed.
dist/welcome.html                       0.49 kB │ gzip:  0.30 kB
dist/overlay.html                       0.64 kB │ gzip:  0.33 kB
dist/index.html                         0.79 kB │ gzip:  0.37 kB
dist/assets/AnalystWindow-CAZOqctF.css  0.45 kB │ gzip:  0.18 kB
dist/assets/main-B_GJJsYT.css          22.70 kB │ gzip:  4.86 kB
dist/assets/client-Ah1pgjPn.js        192.35 kB │ gzip: 60.29 kB
✓ built in 1.22s
EXIT:0
```

### `cargo check` (from `src-tauri/`, tail)
```
   Compiling desk v0.5.0 (C:\Users\zentala\code\MoveUp\src-tauri)
    Checking tauri-plugin-single-instance v2.4.3
warning: function `exe_from_command_line` is never used
   --> src\setup_helpers.rs:341:4
warning: function `tray_menu_item_ids` is never used
   --> src\tray.rs:229:8
warning: `desk` (lib) generated 2 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 46.26s
EXIT:0
```
Completed in 46s, well under the 2-minute budget. Note: E013's journal reports Smart App Control blocking a full `cargo build`/release build and Vitest's `esbuild` spawn locally — `cargo check` (no code-gen for build scripts requiring signing) was not blocked in this session, so this does not contradict that finding; it is a narrower check.

### Tracked build artifacts
```
git ls-files | grep -E '^(coverage|dist|test-performance-report)/' | wc -l
18
```
Sample: `coverage/index.html`, `coverage/coverage-final.json`, `test-performance-report/2026-03-16_19-24-57/index.html`, `.../rust-tests.log`. `.gitignore` covers `dist/` but **not** `coverage/` or `test-performance-report/` — these 18 files are genuinely committed, will drift from reality on every test run, and bloat every clone.

### `.gitignore` (full)
```
node_modules/
.pnpm-store/
dist/
src-tauri/target/
.claude/worktrees/
.build-sizes.json
.build-log.txt
.plan/sessions/
```

### Secret scan
```
git grep -iE 'client_secret|refresh_token' -- ':!*.md'
```
All 25 hits are source code (`google_fit.rs`, `google_fit_service_tests.rs`, `google-fit-auth.cjs`) referencing the **names** of env vars or test fixture strings (`"test-secret"`, `"secret"`, `"tok"`) — no real credential value is committed. `.env` is not tracked (`git ls-files` has no `.env` entry).

### License
```
ls LICENSE*  → no matches
grep "license" package.json → no matches
grep "license" src-tauri/Cargo.toml → no matches
```

### Firmware
```
find firmware -type f
firmware/tof_reader/tof_reader.ino
```
Single sketch file, no README, no version, no wiring diagram, no flashing guide.

### Telemetry
`src-tauri/src/telemetry.rs`: gated by `config.telemetry_enabled` (default off — confirmed by `SettingsTypes.ts`/`TelemetrySection.tsx` opt-in UI); when enabled it only builds and **logs** the payload (`debug!("telemetry: would send {}", json)`) — the actual `reqwest` POST is a `TODO`, never implemented, and the endpoint constant is named `_TELEMETRY_URL` (unused, prefixed to silence the dead-code lint). **Nothing is ever sent over the network by this module today**, opt-in or not.

## Proposed release checklist (ordered)

1. Fix `.plan/STATE.md` to reflect reality (E011/E012 done, v0.5.0) — 5 minutes, unblocks trustworthy planning for whoever ships next.
2. Rewrite the five stale docs (`docs/README.md`, `USER_INSTALL.md`, `USER_UPDATES.md`, `USER_SUPPORT.md`, `PRIVACY.md`) to MoveUp naming/paths/repo, and **either** implement the updater **or** rewrite `USER_UPDATES.md` to describe the real (manual, GitHub-Releases-download) update path until a real updater ships.
3. Add Google Fit disclosure to `PRIVACY.md` (what's sent to Google, when, opt-in nature) — this is a factual accuracy fix, not a feature.
4. Decide and record the license (LICENSE file + `license` field in both manifests) — required before "open-core" can be a true claim to anyone outside zentala.
5. Fix or retire `.github/workflows/test.yml` — either update its paths to repo root or delete it if `release-baseline.yml` has superseded it; a workflow that can't succeed on every PR is worse than none because it's silent noise, not a gate.
6. Decide the E013 Wave 2 signing provider and execute it (this is the actual multi-day blocker — everything else above is same-day fixable).
7. Add a `firmware/README.md` with flashing instructions and the cable-sensitivity warning from `CLAUDE.md`, since the Dev Kit business model requires a buyer to flash this themselves.
8. Move `coverage/` and `test-performance-report/` out of git tracking (`git rm -r --cached coverage test-performance-report` + add to `.gitignore`).
9. Re-run `pnpm test:perf` and `pnpm build:report` to refresh `.perf-baseline.json` / `.build-sizes.json`, and reconcile the 60-70 MB installer-size target in `installer.md` against the observed ~4-6 MB.
10. Do a real browser-driven first-run pass (welcome window → calibration → no-sensor state) with the `browser` agent before calling first-run UX verified — this review did not drive the browser (see GAPS).
11. Close out E011's own "Final integration" checklist (tag `v0.4.0` retroactively is not useful now, but do write the missing `.plan/HISTORY.md` epic entries for E011/E012/E013 so the record isn't silently missing).
12. Only after 1-11: run the full `pnpm tauri:build` (not attempted in this session — see GAPS) as the final go/no-go gate, since it is what actually produces the artifact a user would download.

## GAPS

- **Did not run `pnpm tauri:build` or `cargo build --release`** — explicitly out of scope per the task's "too slow" instruction. `cargo check` and `vite build` passing does not guarantee the full release build (icon bundling, NSIS packaging, LTO) succeeds; E013's journal records that Smart App Control has blocked a *release* Cargo build on this machine before. This is the single most important unverified fact in this report.
- **Did not verify the app in a real browser** — first-run welcome flow, calibration UI, no-sensor UX, and the remote dashboard were assessed by reading source (`WelcomePopup.tsx`, `CalibrationSection.tsx`, `useDesk.ts`/`useRemoteDesk.ts` device-lost handling exists) but never rendered or clicked through. Per this repo's own `ux-design-flow.md` and the global "never claim it works" rule, this is a real gap, not a nit — recommend a `browser`-agent pass before shipping.
- **Did not confirm whether `github.com/zentala/MoveUp` is public or private**, nor whether `github.com/zentala/zntl-tray` (the repo every doc still links to) still exists or 404s — only checked the local `git remote -v`. If `zntl-tray` still resolves to something, users following stale doc links could land on the wrong/dead project.
- **Did not check `TASKS.md`, `DDD.md`, `DESIGN.md`, `.interface-design/`, or `BACKLOG.md` in full** — read only the sections directly relevant to the review questions (autostart findings, planned epics). There may be additional open items in the fuller backlog not surfaced here.
- **Did not run the Rust test suite** (`cargo test`, `pnpm test:unit:rust`) or the TypeScript unit tests (`pnpm test:unit`) — only `cargo check`/`tsc --noEmit`/`vite build` per the task's time constraint. Test totals in `STATE.md` (590 total) are themselves from a stale snapshot and were not re-verified.
- **Did not independently verify the `.build-sizes.json` numbers** against a fresh build — they are read from a file dated 2026-09-05, one day before this review, so likely current, but not regenerated here.
- **CI workflow status unknown** — did not query GitHub Actions to see whether `test.yml` is actually failing on every run (would confirm blocker #7 empirically) or whether GitHub simply has it disabled/unused; inferred purely from the stale `apps/desk` path against the current repo layout.
