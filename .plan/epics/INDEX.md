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
| E014 | `E014-2026-09-05-supervised-release-rollback` | in progress | 53 points, attended AO in Herdr. Waves 1-2 integrated at `d12412a`; October continuation adds lifecycle/readiness, verifies PM3 prerequisites and completes integration. T05 includes independent loopback readiness. See [RESUME](E014-2026-09-05-supervised-release-rollback/RESUME.md). |
| E015 | `E015-2026-09-06-engine-single-truth` | done | Promoted `7a8bbf3`, v0.6.0; T05 closed on automated evidence (`e015_` Rust scenario test), Paweł accepted it instead of a browser pass 2026-09-13 |
| E016 | `E016-2026-09-06-plan-hygiene-and-ao-readiness` | done | AO run E016-20260906-0348, promoted `0995f42` — `.plan/BACKLOG.md` |
| E017 | `E017-2026-09-06-release-readiness` | done | AO run E017-20260906-0622, promoted `91b3572`, all 8 tasks merged, build gate ran for real — `.plan/STATE.md` |
| E018 | `E018-2026-09-06-frontend-consolidation` | done | AO run E018-20260906-1209, promoted `7566de7`, all 11 tasks merged, all 12 evidence records current — `.plan/STATE.md` |
| E019 | `E019-2026-09-06-backend-hardening` | done | AO run E019-20260906-0822, promoted `4649dc5`, all 8 tasks merged — `.plan/STATE.md` |
| E020 | `E020-2026-09-06-engine-pure-core` | done | AO run E020-20260906-1418, promoted `244de29`, all 8 tasks merged — `.plan/STATE.md` |
| E021 | `E021-2026-09-06-smartwatch-integration` | done | All 9 AO tasks (T02-T10) merged (run e021-run1), promoted, verified — 815 Rust tests green; T01 hardware spike outside AO never written up (`reports/2026-09-06-spike-results.md` missing), box left open; Health Connect question lives in BACKLOG as a candidate epic |
| E022 | `E022-2026-09-06-cross-device-phone-relay` | in progress | 14/16 tasks merged + security review (T13) done; T15 (browser-visual) partial, T16 (deploy) deferred — decision E022-D6 |
| E023 | `E023-2026-09-13-relay-security-hardening` | done | Six security fixes integrated as 0.7.0; 13 pts, **direct Herdr**, independent rounds 1–2 PASS, normal relay 180 tests and full project checks passed; no deployment — [JOURNAL](E023-2026-09-13-relay-security-hardening/JOURNAL.md) |
| E024 | `E024-2026-09-13-sensor-connection-diagnostics` | planned | Sensor diagnostics and emulator integration; 18 pts, **direct Herdr**, accepted after E025; overlapping old branches explicitly parked — [HANDOFF](E024-2026-09-13-sensor-connection-diagnostics/HANDOFF.md) |
| E025 | `E025-2026-09-14-alert-dismiss-and-yesterday-delta` | planned | Sensor-independent dismiss, alert-flow integration and yesterday KPI; 17 pts (originally 10; disclosed producer/callback scope), **direct Herdr**, accepted next after E023; overlapping old branches explicitly parked — [HANDOFF](E025-2026-09-14-alert-dismiss-and-yesterday-delta/HANDOFF.md) |
| E026 | `ws/icon-studio/.plan/epics/E026-2026-09-27-reference-icon-vector-library` | planning | Icon studio sub-plan: [INDEX](../../ws/icon-studio/.plan/epics/INDEX.md), [PLAN](../../ws/icon-studio/.plan/epics/E026-2026-09-27-reference-icon-vector-library/PLAN.md), [HANDOFF](../../ws/icon-studio/.plan/epics/E026-2026-09-27-reference-icon-vector-library/HANDOFF.md). Total **?** pending inventory; no dispatch. |

| E027 | `E027-2026-10-09-analyst-timeline` | in-progress | Accepted full core/Analyst implementation2026-10-21; full115p (retained47+provenance/day8+new60). Explicit operator-approved DIRECT Herdr exception2026-10-21; current115p after accepted5p adapter necessity2026-10-22; soleDM/max3/LunaHIGH/independent-beforemerge. Selected AO conflict historical/not executed; preserved candidates/dirty work, app healthy. [PLAN](E027-2026-10-09-analyst-timeline/PLAN.md) |

## Sub-plans

| Sub-plan | Scope |
|---|---|
| sub-plan: [`ws/iconforge/.plan/`](../../ws/iconforge/.plan/epics/INDEX.md) | IconForge — brief → IconSpec → deterministic SVG icons (incubated in the `ws/` monorepo, 2026-09-27) |

## Current continuation execution — 2026-10-21

| Epic | Full points | Dispatch | Admission | Done / not done |
|---|---:|---|---|---|
| E027 |115|DIRECT Herdr, explicit operator exception2026-10-21|Approved; first current15-path engine task (13p after source-qualified adapter amendment) released to sole DM for fresh preflight/regressions|Not done; persist through source/UI review, push and actual release/runtime|
