# E019 — JOURNAL

## 2026-09-06 — AO run, two operator interventions, promoted

Run `E019-20260906-0822` (wave 0: T01+T08 parallel, then T02→T03→T04→T05→T06→T07
sequential — see HANDOFF.md's "Mental model" for why). Both wave-0 workers
(`E019-T01`, `E019-T08`) came back `needs_attention` with
`cause: executor_result_error` within the same minute.

**Intervention 1 — Claude session limit, not a real failure.** Pulled
`state.stdout` out of the ledger per worker (`ao status --json` does not
surface it directly; had to script it via PowerShell since inline
double-quoted `ConvertFrom-Json` property access breaks on hyphenated task
IDs). Both workers' stdout read verbatim:
`"You've hit your session limit · resets 9:40am (Europe/Warsaw)"`. Waited
past the reset time, then `ao resume --manifest ... --run-id ... --ledger
...`, which completed T01, T08, T02, T03 cleanly and reached T04.

**Intervention 2 — Windows Smart App Control blocking Rust build DLLs.** T04's
code was correct (worker made the right edit), but its verification
(`cargo test`) failed with exit `101`. Re-running the exact verify command
by hand in the worker's worktree showed the real cause, which the ledger
does not store (only `ok`/`code`, no stdout): Windows Code Integrity
(`Microsoft-Windows-CodeIntegrity/Operational`, events 3077/3033/3118,
Policy ID `{0283ac0f-fff1-49ae-ada1-8a933130cad6}`) was blocking freshly
compiled, unsigned proc-macro DLLs (`darling_macro`, `tracing_attributes`,
`zerocopy`, ...) — 193 such block events logged since 10:45:32 that morning.
Confirmed this was not epic-specific: the same block hit a plain `cargo
build` in the main checkout, independent of any AO worktree.

Investigated with the user's explicit consent (`consent-broker`, since
disabling a Windows security feature is effectively irreversible) before
touching anything. Discovered mid-investigation that Windows had *already*
started disabling Smart App Control on its own (`VerifiedAndReputablePolicyState`
was `0`/Off in the registry, but `Get-MpComputerStatus` still reported `On` —
the classic "pending until reboot" state). Rather than reboot the machine
(which would have killed every other live session/process on it), tried
`CiTool.exe --refresh` — Windows' own live Code-Integrity-policy-reload tool —
first. It worked: `SmartAppControlState` flipped to `Off` immediately, no
reboot needed, and the T04 verification test passed on the next run.

Filed both root causes to `dispatch.internal/.plan/BACKLOG.md` (2026-09-06,
two new sections) as gaps in AO's `executor_result_error` classification: it
does not distinguish "executor gave a wrong answer" from "executor couldn't
run at all because of a session limit" from "the verification tool couldn't
run at all because the host OS blocked it" — all three land in the same
generic code today, and diagnosing each required manually pulling raw
`stdout`/exit codes out of the ledger rather than seeing them in `ao status`.

`ao resume` after the SAC fix completed T04 through T07 cleanly. `ao promote`
initially failed with `dirty_target` — the manifest JSON I had generated
lived under `.plan/ao/` inside the repo, so the working tree was never
actually clean. Moved the manifest to a scratch path outside the repo
(`--manifest` doesn't need to live inside the tree) and promoted again:
succeeded, merge commit `4649dc5`. Two of the eight worker worktrees needed
manual `wt-remove` (the standard `ao cleanup --force` defect, already filed
from E016/E017); the rest auto-cleaned.

## 2026-09-06 — post-promotion verification

`just check` on promoted `main` (`4649dc5`): 533 Rust unit tests + 3
integration tests + 261 TypeScript tests, all green, exit 0. Per HANDOFF's
"Outside AO" section, no browser pass is required for this epic — all eight
tasks are backend/non-UI, verified by `cargo test`/`vitest` alone.
