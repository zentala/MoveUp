# Epic index — the single source of truth for execution status

See [`.claude/rules/plan-arch-structure.md`](../../.claude/rules/plan-arch-structure.md)
("Status epiku mieszka w `epics/INDEX.md`") for why this file exists and how
it is enforced: `epic-index` (bang-command) checks it against the folders on
disk, `hooks/epic-index-guard.mjs` blocks a commit touching `PLAN.md`/
`HANDOFF.md`/`tasks/*.md` without also touching this file. `PLAN.md` headers
and task frontmatter are written at planning time and rot; this table is the
only place status is trusted.

Created 2026-09-08 (was missing since the repo's first epic — see
`.plan/BACKLOG.md` "`.plan/epics/INDEX.md` does not exist"). E001-E009 were
audited 2026-09-13 against each epic's `ORCHESTRATOR.md`, `HISTORY.md`,
`git log` and the current source; leftovers that are genuinely not done went
to `.plan/BACKLOG.md` §"E001-E009 audit leftovers".

| # | Folder | Status | Note |
|---|---|---|---|
| E000 | `E000-maintenance` | rolling | Permanent catch-all for small/misc work — never closes |
| E001 | `E001-2026-03-15-app-foundation` | done | 10/12 ticked, T10 cancelled (superseded by `icon_for_state_and_progress`); T12 (yesterday delta) reopened — UI deleted later, planned in E025 |
| E002 | `E002-2026-03-16-overlay-progress-bar` | done | 11/12 ticked; T09 resolved by the `opaque` default in `overlay_renderer.rs`; T11 debug overlay line unconfirmed, low value |
| E003 | `E003-2026-03-16-installer-distribution` | done | 6/7 ticked; T07 (tag-triggered GitHub Releases) still open in BACKLOG, waits on code signing |
| E004 | `E004-2026-03-20-session-alerts` | in progress | T02/T03 done; T04 alert-flow integration test never written; T05 dismiss-without-sensor still unfixed despite `DONE.md` — both planned in E025 |
| E005 | `E005-2026-03-21-ux-widgets` | done | 13/13; `WelcomePopup.tsx`, `OneBarWidget.tsx` present |
| E006 | `E006-2026-03-22-session-bugs-polish` | done | 15/15 ticked; T14/T15 marked superseded by `ConnectionOverlay.tsx` (E009) and E008-T08 tooltips |
| E007 | `E007-2026-03-23-kpi-dashboard` | done | 11/11 with commit shas, final `9384824` |
| E008 | `E008-2026-03-24-ux-integrity` | done | 13/13; Away state, KPI icons, `notification_service.rs` present |
| E009 | `E009-2026-03-24-remote-display` | done | Boxes never ticked but shipped: `remote_server.rs`, `ws_broadcaster.rs`, commits `dc2dbd9`..`f17dc0e`, v0.3.0 |
| E010 | `E010-2026-03-25-marketing-launch` | in progress | 19/22 tasks done; 3 remaining need Paweł (photos/video/GIF), not an agent — `.plan/STATE.md` |
| E011 | `E011-2026-05-07-autostart-hardening` | done | Code-complete 2026-05-07, ceremony closed — `.plan/STATE.md` |
| E012 | `E012-2026-05-16-analyst-dashboard` | done | Shipped v0.5.0; last 3 tasks (T09-T11) landed later under E018 — `.plan/BACKLOG.md` |
| E013 | `E013-2026-08-28-signed-tauri-pm3-deployment` | superseded | Docs/licensing/CI absorbed by E017, rollback content by E014 — `.plan/BACKLOG.md` |
| E014 | `E014-2026-09-05-supervised-release-rollback` | blocked | Waves 1-2 done (promoted `d12412a`); waves 3-4 wait on `pm3-mcp` backlog — `.plan/BACKLOG.md` |
| E015 | `E015-2026-09-06-engine-single-truth` | done | Promoted `7a8bbf3`, v0.6.0; T05 closed on automated evidence (`e015_` Rust scenario test), Paweł accepted it instead of a browser pass 2026-09-13 |
| E016 | `E016-2026-09-06-plan-hygiene-and-ao-readiness` | done | AO run E016-20260906-0348, promoted `0995f42` — `.plan/BACKLOG.md` |
| E017 | `E017-2026-09-06-release-readiness` | done | AO run E017-20260906-0622, promoted `91b3572`, all 8 tasks merged, build gate ran for real — `.plan/STATE.md` |
| E018 | `E018-2026-09-06-frontend-consolidation` | done | AO run E018-20260906-1209, promoted `7566de7`, all 11 tasks merged, all 12 evidence records current — `.plan/STATE.md` |
| E019 | `E019-2026-09-06-backend-hardening` | done | AO run E019-20260906-0822, promoted `4649dc5`, all 8 tasks merged — `.plan/STATE.md` |
| E020 | `E020-2026-09-06-engine-pure-core` | done | AO run E020-20260906-1418, promoted `244de29`, all 8 tasks merged — `.plan/STATE.md` |
| E021 | `E021-2026-09-06-smartwatch-integration` | done | All 9 AO tasks (T02-T10) merged (run e021-run1), promoted, verified — 815 Rust tests green; T01 hardware spike outside AO never written up (`reports/2026-09-06-spike-results.md` missing), box left open; Health Connect question lives in BACKLOG as a candidate epic |
| E022 | `E022-2026-09-06-cross-device-phone-relay` | in progress | 14/16 tasks merged + security review (T13) done; T15 (browser-visual) partial, T16 (deploy) deferred — decision E022-D6 |
| E023 | `E023-2026-09-13-relay-security-hardening` | planned | 6 deferred E022 security findings + verify; 13 pts, **subagents**; wave 6, parallel with E024 — [HANDOFF](E023-2026-09-13-relay-security-hardening/HANDOFF.md) |
| E024 | `E024-2026-09-13-sensor-connection-diagnostics` | planned | No-port vs no-match vs flapping sensor diagnostics, emulator integration tests; 18 pts, **AO**; wave 6, parallel with E023 — [HANDOFF](E024-2026-09-13-sensor-connection-diagnostics/HANDOFF.md) |
| E025 | `E025-2026-09-14-alert-dismiss-and-yesterday-delta` | planned | Audit leftovers: popup dismiss ignored without sensor (E004-T05), alert-flow integration test (E004-T04), yesterday comparison KPI (E001-T12); 10 pts, **subagents**; wave 6 — [HANDOFF](E025-2026-09-14-alert-dismiss-and-yesterday-delta/HANDOFF.md) |
