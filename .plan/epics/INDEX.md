# Epic index — the single source of truth for execution status

See [`.claude/rules/plan-arch-structure.md`](../../.claude/rules/plan-arch-structure.md)
("Status epiku mieszka w `epics/INDEX.md`") for why this file exists and how
it is enforced: `epic-index` (bang-command) checks it against the folders on
disk, `hooks/epic-index-guard.mjs` blocks a commit touching `PLAN.md`/
`HANDOFF.md`/`tasks/*.md` without also touching this file. `PLAN.md` headers
and task frontmatter are written at planning time and rot; this table is the
only place status is trusted.

Created 2026-09-08 (was missing since the repo's first epic — see
`.plan/BACKLOG.md` "`.plan/epics/INDEX.md` does not exist"). Rows below marked
`unknown` are honest gaps, not guessed `done` — a full audit against
`.plan/HISTORY.md` and `git log` is still open (`.plan/BACKLOG.md`).

| # | Folder | Status | Note |
|---|---|---|---|
| E000 | `E000-maintenance` | rolling | Permanent catch-all for small/misc work — never closes |
| E001 | `E001-2026-03-15-app-foundation` | unknown | Added by `epic-index --fix` — set the real status. |
| E002 | `E002-2026-03-16-overlay-progress-bar` | unknown | Added by `epic-index --fix` — set the real status. |
| E003 | `E003-2026-03-16-installer-distribution` | unknown | Added by `epic-index --fix` — set the real status. |
| E004 | `E004-2026-03-20-session-alerts` | unknown | Added by `epic-index --fix` — set the real status. |
| E005 | `E005-2026-03-21-ux-widgets` | unknown | Added by `epic-index --fix` — set the real status. |
| E006 | `E006-2026-03-22-session-bugs-polish` | unknown | Added by `epic-index --fix` — set the real status. |
| E007 | `E007-2026-03-23-kpi-dashboard` | unknown | Added by `epic-index --fix` — set the real status. |
| E008 | `E008-2026-03-24-ux-integrity` | unknown | Added by `epic-index --fix` — set the real status. |
| E009 | `E009-2026-03-24-remote-display` | unknown | Added by `epic-index --fix` — set the real status. |
| E010 | `E010-2026-03-25-marketing-launch` | unknown | Added by `epic-index --fix` — set the real status. |
| E011 | `E011-2026-05-07-autostart-hardening` | unknown | Added by `epic-index --fix` — set the real status. |
| E012 | `E012-2026-05-16-analyst-dashboard` | done | Shipped v0.5.0; last 3 tasks (T09-T11) landed later under E018 — `.plan/BACKLOG.md` |
| E013 | `E013-2026-08-28-signed-tauri-pm3-deployment` | superseded | Docs/licensing/CI absorbed by E017, rollback content by E014 — `.plan/BACKLOG.md` |
| E014 | `E014-2026-09-05-supervised-release-rollback` | blocked | Waves 1-2 done (promoted `d12412a`); waves 3-4 wait on `pm3-mcp` backlog — `.plan/BACKLOG.md` |
| E015 | `E015-2026-09-06-engine-single-truth` | in progress | Code-complete via AO, promoted `7a8bbf3`, v0.6.0; T05 browser-evidence gap still open — `.plan/BACKLOG.md` |
| E016 | `E016-2026-09-06-plan-hygiene-and-ao-readiness` | done | AO run E016-20260906-0348, promoted `0995f42` — `.plan/BACKLOG.md` |
| E017 | `E017-2026-09-06-release-readiness` | unknown | Added by `epic-index --fix` — set the real status. |
| E018 | `E018-2026-09-06-frontend-consolidation` | unknown | Added by `epic-index --fix` — set the real status. |
| E019 | `E019-2026-09-06-backend-hardening` | unknown | Added by `epic-index --fix` — set the real status. |
| E020 | `E020-2026-09-06-engine-pure-core` | unknown | Added by `epic-index --fix` — set the real status. |
| E021 | `E021-2026-09-06-smartwatch-integration` | done | All 9 tasks merged (AO run e021-run1), promoted, verified — 815 Rust tests green |
| E022 | `E022-2026-09-06-cross-device-phone-relay` | in progress | 14/16 tasks merged + security review (T13) done; T15 (browser-visual) partial, T16 (deploy) deferred — decision E022-D6 |
