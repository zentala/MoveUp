# E017 — JOURNAL

## 2026-09-06 — AO run, one stale merge_conflict, promoted

Run `E017-20260906-0622` (wave 0: T01-T07 parallel, wave 1: T08). Six of
seven wave-0 tasks merged clean on the first pass. `E017-T01` came back
`needs_attention` with `cause: merge_conflict`.

**Investigated before retrying, per the AO skill's own rule** (`ao/SKILL.md`
§3: "anything else → stop and report, do not invent a recovery" — this
looked like the table's `merge_conflict` row but didn't match either of its
two sub-cases). `git status --porcelain` and `git ls-files -u` in the
integration worktree showed nothing conflicted, no `MERGE_HEAD`. A manual
`git merge --no-commit --no-ff ao/worker/E017-T01/...` against the current
integration HEAD applied cleanly (`Automatic merge went well`) and the
task's own verification script passed on the result. So the conflict report
was stale/transient, not real — reset the diagnostic merge
(`git reset --hard` back to the pre-test integration HEAD) and ran `ao
resume --manifest ... --run-id ... --ledger ...`, which gave T01 a fresh
`attempt-1` and completed the whole run (`execution: completed`,
`promotion: ready_for_promotion`). Filed nothing new to the AO defect
backlog for this one — without a reproduction it would be a guess, and the
skill explicitly warns against inventing causes.

`ao promote` succeeded in one call this time (no `dirty_target`, unlike
E016). `ao cleanup --force` again reported `cleanup: pending` without
removing anything (same known defect already filed to
`dispatch.internal/.plan/BACKLOG.md` from E016) — cleaned the two worktrees
that remained (`integration`, `E017-T01-attempt-0`; the rest were already
gone, unlike E016 where all six lingered) via `wt-remove`, which asserted
`node_modules`/`target` guards survived.

## 2026-09-06 — post-promotion verification and Outside-AO items

Ran `just check` on promoted `main` (91b3572) independently of AO's per-task
gates: 504 Rust + 3 integration + 259 TS tests green (TS count differs from
E015's 255 because T05's fixes below add 4), typecheck clean. Ran each of
the 8 `scripts/check-e017-t0N.mjs` scripts individually on the merged tree —
all pass. Wrote evidence records for all 8 via `verify-evidence`.

**Build gate (Outside AO)**: `pnpm tauri:build` from repo root, real run —
111.9s, produced `MoveUp_0.6.0_x64_en-US.msi` (6.2 MB) and
`MoveUp_0.6.0_x64-setup.exe` (4.3 MB). This falsifies the `installer.md`
60-70 MB target from `E017-D2`'s era — the real app has never been in that
range. Corrected the rule file rather than leaving a stale target in a file
agents read as ground truth.

**First-run browser pass (Outside AO)**: dispatched the `browser` agent.
Result: PARTIAL, not fabricated as a full pass. Welcome screen confirmed
rendered (`welcome.html`, characteristic button text). Calibration could not
be reached — not a failure to find it, but a deliberate `display: none` on
`.one-bar__settings-btn` under `html.remote-display`
(`globals.css:866`), confirmed via `getBoundingClientRect`: calibration is
desktop-only by design and cannot be evidenced from a browser session at
all. No-sensor state could not be reached because the real XIAO sensor was
physically connected and reporting live readings — simulating "no sensor"
would require unplugging hardware, outside a browser agent's reach.

The pass did surface three real bugs, all fixed same-session (not left as
backlog-only, since each was small and I was highly confident in the fix —
per the "found a bug → fix or backlog" rule's confidence/points table):

1. `WelcomePopup.tsx` rendered literal `\uXXXX` escape sequences instead of
   Polish diacritics — plain JSX text is never JS-interpreted, so a
   source file carrying literal escape sequences (not real UTF-8
   characters) rendered as-is. Fixed with real UTF-8 text + a regression
   test asserting no `\uXXXX` pattern survives.
2. `remote_server.rs`'s `frontend_service()` resolved its default static
   dist path (`"../../dist"`) against the process's **current working
   directory**, not the executable's location — 404 on every route
   depending on launch context (confirmed: root gave 404 until the env var
   was forced). Added `default_dist_path()` deriving the path from
   `std::env::current_exe()`; `DESK_REMOTE_DIST` override still works.
3. `App.tsx`'s `listen()` calls and `StepsWidget.tsx`'s `invoke()` call were
   not gated by the existing `isTauri` check used elsewhere in the same
   file, throwing on every remote/browser page load. Gated both.

Dispatched to `ts-dev` for the fix (a second agent, not the browser agent
that found them — separation of finder and fixer). Verified myself after:
`git log` shows 5 real commits, `pnpm test:unit` re-run independently came
back 259/259 green (a `just check` run immediately after landing showed 255
passed + 1 error — re-ran `pnpm test:unit` directly and got a clean
259/259, confirming the first failure was transient, matching what ts-dev's
own report already flagged as an unrelated Windows "app control policy"
blip on an empty test binary, reproduced identically on a stashed baseline).
`just check` on a second run: clean, exit 0.

**Code-signing decision (Outside AO)**: presented to Paweł via
`AskUserQuestion` (three options: defer / buy an EV-OV cert / Azure Trusted
Signing) — a genuine decision with a real cost/vendor trade-off, not
something to auto-decide. Chose to defer; recorded as an open
`.plan/BACKLOG.md` item with both candidate options and their trade-offs, so
whoever revisits it doesn't start from zero.

**Tag**: `v0.6.0` was already created during E015 and was not re-tagged —
the build gate above re-confirms it's still the right version for the
promoted tree.

## Not done this session

- Pushing `main`/tags to `origin` — local is 20+ commits ahead of
  `origin/main` accumulated across E016/E015/E017; push was not requested
  and is a visible-to-others action, so left for Paweł to trigger explicitly.
