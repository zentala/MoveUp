# ADR 019: Release store layout, retention, and what marks a build good

- **Status**: accepted
- **Date**: 2026-09-06
- **Epic**: E014 (task E014-T09, decisions D5, D6)

## Context

[ADR 018](018-pm3-app-ownership-split.md) gives PM3 the supervision loop and
leaves MoveUp two jobs: keep the builds worth falling back to, and say which one
is worth falling back to. Neither exists today. `%LOCALAPPDATA%\MoveUp\` holds
exactly one `desk.exe`, the installer overwrites it, and the previous build is
gone the moment a new one lands.

A supervisor cannot invent that state. It needs three concrete things from the
app: a place where past builds live, a durable statement of which one last
proved itself, and an ordered list of what to try next. All three are file
contracts across a process boundary, so their shape is a decision, not an
implementation detail.

The hard part is the middle one. "The process is running" is a bad definition of
a good build: a binary with a broken frontend bundle starts, stays up, and never
serves a byte. Marking that build good would pin the rollback chain to something
useless.

## Decision

**A versioned release store, three builds retained, and a marker written only
after a build proves itself twice over.**

**Layout.** Builds live under `%LOCALAPPDATA%\MoveUp\releases\<version>\`, one
directory per version. Two sibling files carry the state PM3 reads: a
`last-known-good` marker naming a version, and an ordered candidate list.

**Retention (D6).** Three builds are retained. Installing a fourth prunes the
oldest — **except** the build named by `last-known-good`, which is never pruned.
Retention that could delete the fallback is not retention.

**What "good" means (D5).** A build is marked good when **both** hold: the
process stayed alive for 30 seconds, **and**
`GET http://127.0.0.1:<DESK_REMOTE_PORT>/display/api` returned JSON within the
health timeout. Neither alone qualifies. Alive-but-never-serving is the exact
failure this epic exists for; serve-once-then-exit is a crash loop that happens
to have answered once.

**Marker semantics.** The marker is absent until a probe succeeds. A failed
probe leaves the previous marker untouched — a bad new build must never demote
the record of the last good one.

**Candidate order (D6).** Newest installed build, then `last-known-good`, then
the next older retained build. When the newest *is* the last-known-good the list
collapses to one entry rather than repeating it. The list is a file; PM3 reads
it and never computes it.

**The four states of that list are distinct.** A fresh install with no store
yields exactly one entry and must not crash. An existing but empty store yields
an empty list, and the consumer reports "no candidate available" rather than
silently doing nothing. A probe that throws — port not listening, connection
reset — is a failed probe, not a good build and not a crash of the probing code.

## Alternatives

**Keep the installer's single-slot layout and roll back by re-running an old
installer.** Rejected: it is the manual recovery this epic replaces, and it
requires a human to notice the app is broken first.

**Mark a build good on process-alive alone.** Simpler, and it is what a plain
supervisor already knows. Rejected because it cannot see the failure that
motivated the epic — a build that runs happily and serves nothing would be
promoted over the working one it replaced.

**Let PM3 compute the candidate order from directory mtimes.** Rejected: it
moves app knowledge into the supervisor, contradicting ADR 018, and mtimes lie
after a copy, a restore, or a backup tool touching the tree.

**Retain more than three builds.** Not chosen. Three covers newest,
last-known-good, and one older, which is the deepest the candidate order goes.
More builds cost disk and prove nothing extra.

## Consequences

- Rollback depth is bounded by retention: if the newest build and
  `last-known-good` are both broken, exactly one older build remains to try.
- The marker is the single durable statement of health across restarts, so its
  file format is a compatibility surface. Changing it without a migration
  strands the fallback chain.
- The probe is a **one-shot** check at install or promotion, not continuous
  monitoring. A build that serves JSON at startup and silently degrades an hour
  later is not caught. This is a known limit, recorded in E014's risk table
  rather than solved here.
- Retention now has a rule that can hide a leak: pruning skips the
  last-known-good build, so a store that stops rotating looks like normal
  behaviour. E014-T01's test asserts the fourth install prunes.
- The store's path is Windows-specific (`%LOCALAPPDATA%`). Cross-platform
  supervision is explicitly out of scope for E014.

## References

- [E014 PLAN.md](../../.plan/epics/E014-2026-09-05-supervised-release-rollback/PLAN.md)
  §Decisions and ADRs (D5, D6), §Test strategy
- [ADR 018 — PM3 owns supervision](018-pm3-app-ownership-split.md) — the other
  side of this boundary
- [ADR 001 — remote display web kiosk](001-remote-display-web-kiosk.md) —
  `/display/api` and `DESK_REMOTE_PORT`
- `src-tauri/src/setup_helpers.rs:67` — `setup_remote_display`, why both builds
  answer the health endpoint
