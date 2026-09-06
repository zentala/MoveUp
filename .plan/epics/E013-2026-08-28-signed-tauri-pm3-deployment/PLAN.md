---
epic: E013
created: 2026-08-28
status: superseded
---

# E013: Signed Tauri release and PM3 deployment

> **Superseded 2026-09-06**: this epic's planned waves 1-2 (release readiness:
> docs, license, CI, signing decision) move to
> [E017](../E017-2026-09-06-release-readiness/); waves 3-5 (PM3 supervision,
> rollback, `moveup.internal` health checks) move to
> [E014](../E014-2026-09-05-supervised-release-rollback/). This file is
> retained for its acceptance-criteria research; do not resume implementation
> against it directly.
>
> The frontmatter says `superseded` while the body below still reads as an
> active plan — that disagreement is deliberate during a split. The body is
> kept verbatim as source material for E017 and E014; only the status field
> and this note describe where the work actually lives now.

## Goal

Deliver the current MoveUp Tauri application as a signed Windows release,
supervised by PM3 on the developer machine and reachable at
`moveup.internal`.

## Scope

In scope: reproducible Windows CI, release provenance, code-signing
integration, installation/update procedure, PM3 supervision, internal-domain
registration, and end-to-end verification of the browser remote display.

The work applies only to the current Tauri codebase. No legacy application or
archive is in scope. This epic supersedes and expands deferred
`E003-T07` (GitHub Releases CI/CD Automation).

## Acceptance criteria

- [ ] A GitHub Actions Windows job builds a release from a pinned source revision.
- [ ] CI runs the existing frontend and Rust test gates before publishing an artifact.
- [ ] The release executable and installer have a valid Authenticode signature,
      timestamp, and verified publisher identity.
- [ ] Signing material is held in an approved secret/signing service, never in
      git, workflow logs, artifacts, or a developer workstation checkout.
- [ ] The signed installer can be installed on the developer machine without
      disabling Smart App Control or adding a broad Cargo/target-directory rule.
- [ ] PM3 is the sole supervisor of the installed MoveUp executable, with
      autorestart and a health check against `/display/api`.
- [ ] `moveup.internal/display` renders the remote display and
      `moveup.internal/display/api` returns JSON after restart and after reboot.
- [ ] README remains end-user decision documentation; CONTRIBUTING contains the
      developer/agent build and deployment runbook.

## Constraints

- Do not weaken or turn off Smart App Control.
- Do not add a path rule for Cargo's writable `target` directory.
- Use PM3 only for long-lived process control; do not hand-launch a duplicate.
- Register `moveup.internal` through the internal-domain source of truth, not
  by editing a reverse proxy manually.
- The selected signing service must support unattended CI signing without
  exporting a private key into the repository.

## Out of scope

- Any legacy/archive application.
- Publishing MoveUp to external users, automatic updater infrastructure, and
  mobile distribution.
- Changing ergonomics, sensor, or dashboard product behaviour except where
  required to make the release health check reliable.

## Architecture impact

Pattern: **one address, two audiences**. The desktop app remains a native Tauri
application, while its embedded remote-display server provides the browser
surface at `moveup.internal`.

Conventions applied: an internal service is reached by a `.internal` domain,
not an exposed port; PM3 owns the long-running process; README and
CONTRIBUTING serve different readers.

Before implementation, create ADRs for:

1. the selected code-signing provider and secret custody model;
2. the release distribution location and retention policy;
3. any change to the default remote-display port or deployment layout.

## Delivery waves

### Wave 1 - Release baseline

Audit the current Tauri build on a clean `windows-latest` runner. Add a
manually-triggered GitHub Actions workflow with dependency caching, a pinned
Node/pnpm/Rust toolchain, frontend build/test, Rust test, `tauri build`, and a
short-lived unsigned artifact for diagnostics. Record exact output filenames,
hashes, and build revision.

**Exit condition:** CI produces an unsigned installer/executable and test logs
from the current revision. It is not deployed to Smart App Control machines.

### Wave 2 - Signing decision and integration

Inventory existing organisational signing capability. Choose a service that
works with GitHub Actions and permits timestamping and certificate verification
without placing an exportable private key in the repository. Configure secrets
only after the provider and repository access scope have been approved. Add
signature verification as a CI postcondition.

**Exit condition:** a CI release artifact passes `Get-AuthenticodeSignature`,
has a trusted publisher chain on the target machine, and is timestamped.

### Wave 3 - Release installation contract

Define an immutable release directory, installer invocation, versioned release
metadata, SHA-256 verification, rollback procedure, and retention policy. Make
the remote frontend path explicit via `DESK_REMOTE_DIST`; do not rely on CI or
interactive working directories.

**Exit condition:** a clean target machine can install the signed release and
start it manually once; its remote-display API returns JSON.

### Wave 4 - PM3 and internal domain

Reserve a port through the domain registry, register `moveup.internal`, and add
a machine-local PM3 service pointing to the installed signed executable. Set
literal `DESK_REMOTE_PORT` and `DESK_REMOTE_DIST`, HTTP health check
`/display/api`, autorestart, bounded backoff, and a sufficient start period.

**Exit condition:** PM3 reports the service healthy; an unexpected process exit
is recovered; no duplicate MoveUp process exists.

### Wave 5 - End-to-end and operations handoff

Verify the domain from the browser, API response, WebSocket connection, tray
behaviour, reboot persistence, and PM3 logs. Update documentation and write a
brief release/rollback operator runbook.

**Exit condition:** the developer can open `moveup.internal/display`, and a new
agent can reproduce or diagnose the deployment without changing Windows
security settings.

## Test strategy

- CI: `pnpm` frontend checks, Rust unit tests, release build, artifact hash,
  Authenticode signature and timestamp verification.
- Installation: install on a clean Windows profile; verify signature before
  launch; confirm no Smart App Control bypass was used.
- Service: PM3 status and HTTP health check; force one controlled restart and
  confirm bounded recovery.
- Browser: open `/display`, assert `/display/api` returns JSON, establish one
  WebSocket connection to `/display/ws`.
- Reboot: verify PM3 daemon and MoveUp service recover, then re-run the browser
  checks.

## Risks and gates

| Risk | Mitigation / gate |
| --- | --- |
| No suitable signing identity | Stop after Wave 1; do not deploy unsigned output as a workaround. |
| Certificate private key exposed | Use a managed signing service or CI secret scope; rotate immediately if exposure is suspected. |
| Signed binary still lacks reputation | Test on the target device before rollout; do not weaken Smart App Control. |
| Static remote display path is wrong | Make `DESK_REMOTE_DIST` explicit and assert `/display` in the release smoke test. |
| PM3 starts a duplicate process | PM3 is the only launch path after Wave 4; verify process identity before enabling autorestart. |
