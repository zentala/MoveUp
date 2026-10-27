# MoveUp repository instructions

Read the existing [repository contract](CLAUDE.md) before planning or editing. It contains legacy project instructions that have not yet been consolidated here; preserve them when a separate migration is performed.

The canonical visual contract is [DESIGN.md](DESIGN.md). Read it before changing popup, Analyst, settings, charts, colors, typography, spacing or icons. Record new visual decisions there before implementation. Compact and detailed timelines share chronological intervals and posture colors; credited sitting load remains separate from elapsed history.

Read `.plan/VISION.md` before revising product plans. Preserve actual observations and identify missing data; never turn an unobserved interval into Away or use credited sitting load as a historical duration.

## Window runtime diagnostics

Follow [ADR 025](.arch/ADR/025-window-runtime-diagnostics.md), preserving
[the separate Analyst window](.arch/ADR/012-analyst-dashboard-separate-window.md),
[engine purity](.arch/ADR/015-pure-ergo-engine.md) and
[backend readiness](.arch/ADR/024-application-lifecycle-readiness.md).
UI/COM callbacks enqueue bounded typed observations without waiting on disk,
JavaScript or session locks. Diagnostics cannot reload/restart the application
or infer painting/crash causes from missing responses or DOM presence.

## Startup supervision

Prefer installed PM3 on development machines where available. Once registered,
PM3 alone owns the MoveUp process: use its ordinary start/stop/restart and logs;
do not launch a competing release executable. Preparation is documented in
`docs/USER_INSTALL.md`; generating a file is not registration or runtime proof.
Use the installed public CLI contract, not unsupported candidates, suspend,
promotion callbacks or a second supervisor. Ordinary stop removes boot-restore
intent; boot=true alone does not restore a deliberately stopped service.
Quit, development replacement and login activation require separately verified
flush-before-stop and ownership behavior before installation. Keep the existing
direct login entry until supervised readiness and recovery have been proved.

Readiness returns 503 when an existing essential prerequisite fails, and 200
for core-ready operation even without a sensor. Its session_store_available
field is separate evidence: core readiness does not certify session Store
persistence. Do not repair or conceal that separate limitation in startup work.
Retain the persistent application event log and PM3 disk lifecycle records;
Windows GUI builds need not emit usable stdout/stderr. Do not enable a console
to make the log tail look populated. Run `just check`, then obtain independent
source review and the applicable actual Windows lifecycle checks before delivery.
