# MoveUp — full plan/backlog sweep and break-credit history (2026-09-06)

## TLDR

Four files claim to be "the backlog" and disagree with each other; `.plan/STATE.md`
contradicts the epic orchestrator it summarizes; the session/break-credit logic
was rewritten at least 9 times between 2026-03-16 and 2026-03-31 chasing the same
class of bug (a counter reduced by break credit being read where a raw counter
was needed, or vice versa) and the underlying design flaw -- two sitting counters
with overlapping purposes -- is still open today as a filed bug (PostureBalance
ratio, `.plan/BACKLOG.md:185-188`) and a known gap (`break_credit` not persisted
on `SessionRow`, `.plan/IMPROVEMENTS.md:17-20`). Nine of fifteen epics are fully
closed; E010 (marketing) is 19/22 done and stalled on human tasks; E011 is
code-complete but its closing ceremony was never finished, and `STATE.md` says
it is still "active" with tasks pending that are in fact done; E012's core is
done but has three unstarted follow-on tasks; E013 never got past planning;
E014 is half-blocked on another repo (`pm3-mcp`). Recommendation: close E011's
ceremony, delete the stale `.plan/STATE.md` claim, retire root `BACKLOG.md`/
`TASKS.md`/`ORCHESTRATOR.md` in favor of `.plan/BACKLOG.md`, and fix the
sitting-counter/break-credit double-counter design before adding another epic
on top of it -- every regression below traces back to that one design gap.

---

## 1. Ranked table of all open work

Four backlog-like files exist: root `BACKLOG.md` (339 lines, legacy, last
substantively touched around the marketing-launch era), `.plan/BACKLOG.md`
(268 lines, actively maintained, has 2026-09-05/06 entries), root `TASKS.md`
(40 lines, already a redirect stub to `.plan/`, harmless), and root
`ORCHESTRATOR.md` (3 lines, dead pointer to an archived 2026-03-21 sprint file).
**Canonical recommendation: `.plan/BACKLOG.md`.** It is the one the
`plan-arch-structure.md` convention names, the one with current dates, and the
one epics actually link to. Root `BACKLOG.md` duplicates roughly 80% of its
section headers (Tauri Plugins, App Icon SVG, Alert System Future, Sensor and
Readings, Logging and Observability, Timeline Full Window, KPI Time Range
Selector, Activity Tracking, Future Features, "Removed") almost verbatim but
is missing everything from 2026-09 (autostart findings, PostureBalance bug,
sensor connection-flicker findings) and has its own 10-item "Collect from
User -- blocking launch" checklist that **contradicts** `STATE.md`'s claim of
"3 remaining human tasks" for E010. **Action: merge the "Collect from User"
section and anything else not already in `.plan/BACKLOG.md` into
`.plan/BACKLOG.md`, then delete root `BACKLOG.md` and root `ORCHESTRATOR.md`,
and leave `TASKS.md` as the redirect stub it already is** (or delete it too --
it adds nothing `.plan/STATE.md` doesn't already say).

| # | What | Source (path:line) | Importance | Points | Duplicate of |
|---|---|---|---|---|---|
| 1 | `.plan/STATE.md` claims E011 T01/T02 pending; E011's own ORCHESTRATOR.md and DONE.md show T01-T05 all [x] with commits -- pure documentation contradiction, but it is the file every session reads first | STATE.md:14-17 vs E011 ORCHESTRATOR.md:5-40 vs DONE.md:115-121 | High | 1 | -- |
| 2 | E011 closing ceremony never finished -- all 5 tasks done, but "Final integration" checklist still unchecked: build verified, manual smoke test, version bump to 0.4.0, tag, HISTORY.md entry, IMPROVEMENTS triage | E011 ORCHESTRATOR.md:44-51 | High | 3 | -- |
| 3 | PostureBalance ratio uses mixed counters -- sitting_seconds is break-credit-reduced, sitting_seconds_total is not; condition can never fire for a heavy break-taker on a genuinely sedentary day. Same two-counter confusion behind 3+ historical regressions (section 2) | .plan/BACKLOG.md:185-188, ADR 009 | Medium | 3 | root BACKLOG.md has no equivalent entry |
| 4 | break_credit not persisted on SessionRow -- Analyst dashboard guesses break credit from duration thresholds instead of the real proportional value from ADR 008 | E012 IMPROVEMENTS.md, promoted to .plan/IMPROVEMENTS.md:17-20 | Medium | 3 | -- |
| 5 | Sensor connect/lost flicker with a bad cable is invisible to the user -- serial.rs:161-200 treats each cycle as a normal connect, no threshold/warning | .plan/BACKLOG.md:114 | Medium | 3 | -- |
| 6 | Zero-COM-ports vs "sensor didn't answer" indistinguishable in logs -- serial.rs:161-200, violates "silence never means success" | .plan/BACKLOG.md:115 | Medium | 2 | -- |
| 7 | E014 Wave 3-4 blocked on pm3-mcp backlog (candidate list + demotion 8pt, singleton stand-down 5pt) -- MoveUp side (Waves 1-2, 21pt) unblocked and not started | .plan/BACKLOG.md:5, E014 HANDOFF.md:1-70 | Medium | 43 total (13 unblocked / 30 blocked) | -- |
| 8 | E013 stuck at "planning" -- HANDOFF.md says "Waves and task files will be defined after plan review"; only T01 exists and its branch is marked abandoned (superseded on main) | E013 HANDOFF.md:1-4, tasks/E013-T01-release-baseline.md:9 | Medium | ? | overlaps E003-T07 (row 9) |
| 9 | E003-T07 (GitHub Releases CI/CD) -- open since E003 (2026-03-16), explicitly folded into E013's scope but never marked superseded in its own line | .plan/BACKLOG.md:68 | Low | -- | duplicate of row 8 (E013) |
| 10 | StateGantt aggregates from snapshots, spec said sessions-driven -- semantically wrong chart in Analyst Explorer | .plan/IMPROVEMENTS.md:12-15 | Medium | 5 | -- |
| 11 | position_changes / longest_session_secs stubbed in per-day rollup (positionChanges: 0, longestSessionSecs: snap.standingSecs) -- same rollup-granularity gap as row 4 | .plan/IMPROVEMENTS.md:22-25 | Medium | 3 | overlaps .plan/BACKLOG.md:142-149 "split position_changes" |
| 12 | Root BACKLOG.md "Collect from user -- blocking launch", 10 unchecked human items -- contradicts STATE.md's "3 remaining" for E010 | BACKLOG.md:266-280 vs STATE.md:59-63 | Low (business) | -- (human) | contradicts STATE.md:59-63 |
| 13 | E002-T09 -- choose OPAQUE vs LAYERED overlay render mode needs a formal ADR (decision already made in practice) | .plan/BACKLOG.md:64 | Low | 1 | -- |
| 14 | E002-T11 -- overlay debug info needs manual QA sign-off | .plan/BACKLOG.md:65 | Low | 1 | -- |
| 15 | E004 T017 Stages 3-5 -- alert escalation stages 3-5, no task file exists yet | .plan/BACKLOG.md:72 | Low | 8 | -- |
| 16 | E004 T018 -- Notification A/B testing (profiles make this feasible, not yet exploited) | .plan/BACKLOG.md:73 | Low | 5 | also mentioned .plan/BACKLOG.md:223 |
| 17 | E004 T019 -- Success notifications + gamification, no task file | .plan/BACKLOG.md:74 | Low | 8 | also mentioned .plan/BACKLOG.md:224 |
| 18 | E004-T04 -- integration test for full alert flow, basic tests exist, time-simulation missing | .plan/BACKLOG.md:71 | Low | 3 | -- |
| 19 | E006-T14 -- connection status UI cleanup, functional only in Debug tab | .plan/BACKLOG.md:77 | Low | 2 | -- |
| 20 | E006-T15 -- tooltip + UI integration tests, e2e coverage still basic | .plan/BACKLOG.md:78 | Low | 3 | -- |
| 21 | E007 T047/T048 -- visual verification pass + fixes from that pass, never run | .plan/BACKLOG.md:81-82 | Low | 2+3 | -- |
| 22 | Profile editor UI (raw JSON edit only today) | .plan/BACKLOG.md:200 | Low | 8 | -- |
| 23 | useTimelineNav.isLive can report stale value between minute boundaries; a naive fix crashed vitest workers once already | .plan/IMPROVEMENTS.md:32-43 | Low | 3 | -- |
| 24 | Cross-platform activity detection (Windows-only GetLastInputInfo, Away never triggers on Linux/macOS) | .plan/BACKLOG.md:151 | Low | 8 | also root BACKLOG.md:124 |
| 25 | SQLite time-series storage (replace per-minute JSON files) -- prerequisite for items below | .plan/BACKLOG.md:121 | Low | 8 | also root BACKLOG.md:190 |

**Tail beyond top 25** (~22 more open items, by category): Profile System
scheduling/per-app/analytics (3 items, ~18pt, Low) -- Communication tuning
(profile-reload toast, neutral bar style, per-profile cooldown defaults -- 3
items, ~5pt, Low) -- Visualization spikes already delivered as reports but not
acted on (time-viz library evaluation, timeline visual upgrades -- 2 items,
decision pending, Low) -- Timeline Full Window + KPI Time Range Selector (2
items, ~13pt, Low, both gated on item 25) -- Content: one article write-up
(P2, 3pt) -- versioning.md should mention Cargo.lock in the bump checklist
(.plan/IMPROVEMENTS.md:27-30, trivial, 1pt) -- Landing page post-launch infra
(Cloudflare Worker waitlist, live counter, telemetry endpoint -- 3 items,
human/infra, not pointed) -- i18n (explicitly deferred until 100 pre-orders,
not actionable now) -- T-ARCH-001 extended body positions (vision-level,
unscoped, "?").

---

## 2. Break-credit / session-reset / sitting-counter timeline

Commit-by-commit, from `git log --oneline --follow` on `session.rs`,
`session_breaks.rs`, `session_daily.rs`, `hourly_break_tracker.rs`:

| Date | Commit | Intent |
|---|---|---|
| 2026-03-16 | f3a584d | Rust core created -- first session.rs, no break credit yet |
| 2026-03-16 | 9083e44 | Daily reset + standing-seconds counter added |
| 2026-03-20 | 649781a | Startup-race + first-run fixes |
| 2026-03-21 | 90bc888 | First split of session.rs (already too large) |
| 2026-03-21 | ca988d7 | "fix session timer + add transition banner" |
| 2026-03-21 | 74d559b | Points/score system added -- reads sitting/standing counters |
| 2026-03-23 | cd70caf | Regression: standing timer always zero -- serde case mismatch |
| 2026-03-23 | 663369b | E001-T04 SessionManager KPI extensions |
| 2026-03-23 | 7afe752 | HourlyBreakTracker + GapHandler introduced -- second break-tracking mechanism, parallel to session_breaks.rs |
| 2026-03-23 | 68ca0ef | "T042 session timer tests + fix break continuity" |
| 2026-03-23 | bb7561c | "set break_started on Away/Walking -> Standing" |
| 2026-03-23 | 7d46500 | "live break timer in tooltip + cleanup dead code" |
| 2026-03-23 | 9ab747f | "reset current_session_secs on state transitions" |
| 2026-03-24 | 72bb935 | Regression fix #1: "standing_pct uses raw sitting total, fix snapshot data" -- first appearance of the raw-vs-credited counter split (sitting_seconds vs sitting_seconds_total); standing% was showing 100% then 40% because break credit zeroed sitting_seconds, which KPI math also used as denominator |
| 2026-03-24 | 6fee55a | "standing % excludes Away time + wire HourlyBreakTracker" |
| 2026-03-24 | c40a3ca | "throttle accumulate_ongoing to 1 Hz + timestamp-based counters" |
| 2026-03-24 | f37a529 | "Away state now triggers on inactivity regardless of desk height" -- closes the open investigations/2026-03-25-away-detection-standing.md question about is_active() |
| 2026-03-24 | 6870982 | Dead-code removal in hourly_break_tracker.rs |
| 2026-03-30 | 3c09b3c | AppConfig fields moved into ergonomic profile (touches break-credit constants) |
| 2026-03-30 | e40fc0f | Second rewrite of break credit: 3-tier step function replaced by proportional credit (1 min break cancels 2 min sitting, configurable multiplier) -- ADR 008 |
| 2026-03-30 | 291911e | "apply break credit on sleep gap" -- closes a real regression: a machine-sleep gap inflated sitting_seconds because it rewound timestamps but never ran the credit calculation (41954s sitting after restart, per E008 journal) |
| 2026-03-30 | 1ba12ce | impro review fixes -- mutex safety, warnings, docs |
| 2026-03-30 | 09a1190 | Third rewrite/addition: Day Break Credit (breaks >= 6h reset notification flags + daily_score) layered on Session Break Credit -- ADR 009. Same session's journal already flags the PostureBalance mixed-counter bug (row 3 above) as a known consequence |
| 2026-03-31 | f57039d | HourlyBreakTracker persistence across restarts -- a fourth, independent persistence bug (break credit and flags were lost on every dev restart until session_persistence.rs landed) |

**Read as a whole**: the sitting/break-credit subsystem was touched in at
least **9 distinct fix/rewrite commits** across 15 days (2026-03-16 to
2026-03-31), and the recurring root cause each time was **the same design
seam** -- one raw accumulator (sitting_seconds_total) and one credit-adjusted
accumulator (sitting_seconds) serving different consumers (session-limit
logic vs. KPI/PostureBalance math) without a single place that says which
counter a given consumer should read. `72bb935` first split them,
`.plan/BACKLOG.md:185-188` shows the split is still incomplete for
PostureBalance five months later, and `.plan/IMPROVEMENTS.md:17-20` shows the
Analyst dashboard (E012, 2026-05-16) hit the identical seam again because the
persisted SessionRow never captured which counter's value it recorded. No
regression after 2026-03-31 has needed a rewrite -- the September-2026
findings (rows 3, 4, 11 above) are about the same gap never being closed, not
new breakage.

---

## 3. Epic status -- finished vs partial vs abandoned

| Epic | Status | Evidence |
|---|---|---|
| E000-maintenance | Open by design (permanent) | -- |
| E001 App Foundation | Done | TASKS.md:27, all tasks in DONE.md |
| E002 Overlay Progress Bar | Done, 2 loose ends | T01-T08,T10,T12 done; T09 (ADR) and T11 (QA sign-off) open, both trivial |
| E003 Installer & Distribution | Done, 1 loose end | T01-T06 done; T07 (GH Releases CI) deferred, now folded into E013 |
| E004 Session Alerts & Snooze | Partially done | T02,T03,T05 done; T04, T017, T018, T019 open -- roughly half the originally scoped alert-escalation work never shipped |
| E005 UX Communication + Widgets | Done | T01-T13 all in DONE.md |
| E006 Session Bugs & Polish | Done, 2 loose ends | T01-T13 done; T14 (connection UI) and T15 (tooltip e2e) partial |
| E007 KPI Dashboard + Timer UX | Done, 2 loose ends | T01-T11 done; T047/T048 verification pass never run |
| E008 UX Integrity | Done | T01-T13 all done |
| E009 Remote Display | Done | T01-T07 all done |
| E010 Marketing Launch | Stalled on human tasks, 19/22 | STATE.md:59-63 claims 3 remaining; root BACKLOG.md:266-280 lists 10 -- needs reconciliation |
| E011 Autostart Hardening | Code-complete, ceremony unfinished, STATE.md wrong | See ranked-table rows 1-2 |
| E012 Analyst Dashboard | Core done, 3 follow-on tasks open | Original 6 tasks (T01-T06) all [x]; T09-T11 (layout flip, donut KPIs, pulse animation) exist as task files but not started |
| E013 Signed Tauri/PM3 Deployment | Stuck in planning | HANDOFF.md explicitly incomplete; only task (T01) marked abandoned/superseded |
| E014 Supervised Release Rollback | Half-blocked, not started | Waves 1-2 (21pt) unblocked and idle; Waves 3-4 (22pt) blocked on pm3-mcp |

**Should be closed now** (small ceremony, big signal value): **E011** -- flip
the STATE.md contradiction, run the final-integration checklist, close it.
Its "active_epic" status is actively misleading anyone who reads STATE.md
first, which is everyone.

**Should be marked superseded, not left open**: E003-T07 (subsumed by E013).

**Genuinely abandoned in the git sense** (branch exists, work superseded on
main by a different commit): `feat/E013-T01-release-baseline` -- confirmed
via its own task frontmatter (tasks/E013-T01-release-baseline.md:9: "branch:
abandoned (superseded on main by 1484809)"). No other unmerged branches exist
(`git branch -a` shows only this one plus `main` and
`remotes/origin/moveup-migration`); no worktrees are open (`git worktree
list` shows only the main checkout).

---

## 4. Recommendation

**Do the STATE.md/E011 reconciliation and the four-file backlog merge first,
in one short session, before touching E013/E014 or any new epic.** Concretely:
(1) run E011's unfinished final-integration checklist and close it, (2)
rewrite `.plan/STATE.md` to reflect that (active epic should become E014 wave
1, or E000 if nothing is being actively worked), (3) merge root BACKLOG.md's
unique content into `.plan/BACKLOG.md` and delete root BACKLOG.md +
ORCHESTRATOR.md, (4) mark E003-T07 as superseded-by-E013 instead of a
standalone open line. This is roughly 1-2 points of work and it is the
prerequisite for the requested architecture review to trust anything else in
`.plan/` -- right now the single most-read status file in the repo is wrong
about the most recently closed epic.

Separately, and this is the answer to "why does the break-credit logic keep
regressing": **it is not regressing anymore in the sense of new breakage** --
the last rewrite was 2026-03-31, five months before this review. What is true
is that the underlying design gap that caused all nine 2026-03 fixes (two
sitting counters, no single owner of "which one does X read") was never
closed, only patched around each time it surfaced in a new consumer (KPI math
-> PostureBalance -> Analyst SessionRow). The next consumer to touch
sitting/break data (E012's remaining T09-T11, or anything in E014 that reads
session health) will hit the same seam a fourth time unless ranked-table rows
3 and 4 are fixed together, as one task, before more UI is built on top of
the ambiguous counters.

---

## 5. GAPS

- **No KB epics exist for MoveUp** -- `kb_plan_list` lists epics for ~15 other
  repos but nothing under MoveUp/desk; `kb_plan_repos` was not separately
  queried to confirm MoveUp is even registered as a discovered `.plan/` repo.
  If Pawel expected cross-repo epics here, they do not exist yet.
- **`kb_triage_list` and `kb_goal_list` were not queried** -- this review only
  covers repo-local `.plan/` state and `kb_plan_list`; any goal-level framing
  for MoveUp is not reflected in the table above.
- **Root BACKLOG.md's "Collect from user" (10 items) vs STATE.md's "3
  remaining" for E010 were not reconciled** -- I could not determine from the
  files alone which is current; flagged as a contradiction (row 12) rather
  than resolved.
- **`.plan/epics/E000-maintenance/JOURNAL.md` and IMPROVEMENTS.md were
  grepped, not read in full** -- only break-credit-related lines were pulled;
  E000 may carry other open small items not reflected here.
- **`.plan/reports/` (standalone research: gamification techniques, time-viz
  spikes, screen-time article research, sit-stand-walk-cycle research) were
  not opened individually** -- they are referenced from BACKLOG.md and
  counted as "tail" items, but their own content was not inspected.
- **`.plan/ADR/` was not swept file-by-file for status: proposed** -- only
  ADR 008 and 009 were read (via journal references) because they relate
  directly to break credit; other ADRs may carry open decisions.
- **Points for E013 (row 8) are marked "?"** -- the epic has no HANDOFF.md
  wave breakdown yet, so no honest point estimate exists; this is itself
  evidence the epic is not readiness: ready.
- **No verification that the two pm3-mcp backlog items blocking E014 are
  still open** -- `int://mATX.lan/C:/code/pm3-mcp/.plan/BACKLOG.md` is a
  different repo and was not read in this sweep; the block status is taken
  on the word of MoveUp's own HANDOFF.md, dated 2026-09-05.
