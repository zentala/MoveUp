# Session Engine Review — Break Credit Reset Bug + Architecture Audit

## TLDR

**Yes, it resets — but only the number the owner actually watches.** The
internal engine field `sitting_seconds` (`src-tauri/src/session_breaks.rs:126-158`)
does apply proportional credit correctly, matching ADR-008 (verified live in
`events.log`: `CREDIT Partial dur=199s sitting=366`, not zero). But the main
popup's big timer and its progress bar (`OneBarTimer.tsx:28`) are wired to a
*different* field, `current_session_secs`, which is unconditionally zeroed on
every single Standing/Walking/Away → Sitting transition regardless of break
length (`session_breaks.rs:34`, `:63`, `:85`) and never receives any credit —
by design, per its own doc comment (`session_types.rs:112-114`). The overlay
bar and the notification/escalation engine correctly use the credited
`sitting_seconds` (`tray_controller.rs:61-62,124-128`), so two visible UI
surfaces disagree with each other about whether a break "did anything." This
is a wiring bug (wrong field bound to the wrong display), not a missing
feature — the proportional-credit math the owner is asking for already
exists in the codebase, one field over from where the UI reads it.

## Trace: what happens on stand-up

State machine entry point: `SessionManager::on_reading()` —
`src-tauri/src/session_reading.rs:11-162`. On a confirmed transition it calls
`handle_state_exit()` (`session_reading.rs:112` → `session_breaks.rs:11-118`).

### Sitting → Standing (leaving Sitting)

`session_breaks.rs:19-46` (`DeskState::Sitting` arm):
- `session_breaks.rs:20-24`: commits elapsed time — `sitting_seconds +=
  elapsed`, `sitting_seconds_total += elapsed`. **Not zeroed.**
- `session_breaks.rs:33-38`: on any exit from Sitting — `current_session_secs
  = 0` (unconditional, happens the instant you stand up, before any break
  duration is known), `break_started = Some(now)`, `break_seconds = 0`,
  `last_break_credit = BreakCredit::None`.
- `sitting_seconds` is **untouched** here — it keeps its committed value.

### Standing/Walking/Away → Sitting (returning from a break)

`session_breaks.rs:47-76` (`DeskState::Standing` arm, `Standing→Sitting`
sub-case) and `session_breaks.rs:77-114` (`Walking|Away` arm,
`→Sitting` sub-case):
- `session_breaks.rs:59-62` / `:81-84`: computes `break_dur`, calls
  `apply_break_credit(break_dur)`.
- `apply_break_credit()` (`session_breaks.rs:126-158`):
  - `session_breaks.rs:129-132`: break `< break_min_secs` (default 60s,
    `session_types.rs:14`, `ergonomic_profile.rs:15`) → `BreakCredit::None`,
    **`sitting_seconds` is left unchanged** (correct — a 30s break should not
    reduce it, and per code it doesn't).
  - `session_breaks.rs:133-140`: otherwise `credit = break_secs *
    multiplier` (default 2.0, `session_types.rs:17`); `sitting_seconds =
    max(0, sitting_seconds - credit)`. **This is genuinely proportional and
    matches ADR-008.** For a 2-min (120s) break: credit = 240s subtracted —
    not the owner's requested 3:1 ratio (would be 360s), but proportional,
    not a reset (see Model Inconsistencies for the ratio mismatch).
  - `session_breaks.rs:145-157`: separately, if break ≥ `day_break_min_secs`
    (default 6h) — a *different* reset (notification flags + `daily_score` to
    0, NOT `sitting_seconds`, which was already handled by the multiplier
    line above — 6h × 2.0 will already have zeroed it via the `max(0, ...)`
    anyway).
- `session_breaks.rs:63-64` / `:85-86`: **immediately after** crediting,
  `current_session_secs = 0` and `break_seconds = 0` are set again — this is
  the second, independent write that hard-resets the UI-facing counter, with
  no relationship to the credit just computed.
- `session_breaks.rs:71` / `:97`: `sitting_started = Some(now)` — the elapsed
  computation baseline restarts (correct: it must, since it's a delta timer).

### Why the owner sees a reset

`SessionState.current_session_secs` doc comment, `session_types.rs:112-114`:
> "Seconds in the current sitting session only (resets on Sitting entry,
> after break credit). Use this for the session progress timer in the UI."

This field is *designed* to always start at 0 after any break — it was never
meant to carry credit. The bug is that it is the field piped to the
user-visible timer:

1. `SessionManager::snapshot()` (`session_manager.rs:187-213`) emits **both**
   `sitting_seconds` (credited, `:195`) and `current_session_secs`
   (uncredited, `:205`) in the same `SessionStateDto`.
2. Frontend `useDesk.ts:106` (initial fetch) and `:170` (live event handler)
   both do `setSittingSeconds(dto.current_session_secs)` /
   `setSittingSeconds(payload.current_session_secs)` — the hook's
   `sittingSeconds` state variable is **misleadingly named**; it actually
   holds the uncredited per-bout counter, not the credited daily counter.
3. `useWidgetData.ts:39`: `currentSessionSecs: desk.sittingSeconds` — passes
   it straight through to widget props.
4. `OneBarTimer.tsx:28`: `elapsed = isSitting ? props.currentSessionSecs :
   props.breakSecs` — this is the **big number** shown to the user
   ("25:00 / 40:00") and, at `:59-68`, the exact same `elapsed` value drives
   the `ProgressBar`'s fill. Both slam to `0 / limit` the instant you sit
   back down, no matter how short or long the break was.

Meanwhile the same information, computed correctly, is available but unused
by this widget: `limitUsedSecs` (= live `sitting_seconds`,
`session_manager.rs:202,215-217`) is exposed by `useDesk.ts:48,112,236-238`
and only consumed by `DebugSection.tsx:73,88` (Settings → Debug tab) and by
`computeTemperature()` (`temperature.ts:29-32`), which sets the widget's
background color/glow from `props.limitRatio` — the **correct**, credited
ratio. This produces a directly observable self-contradiction: after a
2-minute kitchen break, the widget's background can still show "warm" (color
correctly reflects partial credit) while the big number and the bar under it
both show `0:00 / 40:00` (uncredited hard reset) in the same frame.

The overlay bar (top-of-screen native window) and the notification/alert
engine do **not** have this bug:
- `tray_controller.rs:61-62`: `progress = payload.sitting_seconds as f32 /
  snapshot.session_limit_secs as f32` — feeds `overlay.update()` — **credited**.
- `tray_controller.rs:124-128`: `elapsed_secs = match state { Sitting =>
  snapshot.sitting_seconds, Standing => snapshot.break_seconds, _ => 0 }` —
  this feeds `CommunicationPolicy::evaluate()` (`communication_policy.rs:55`),
  which drives escalation, tray color, and notifications — **credited**.

So: the engine's credit math is correct and is honored by the overlay bar and
the notification system; only the popup's headline number and its own
progress bar read the wrong field.

## Log evidence

Real logs from `C:\Users\zentala\AppData\Roaming\io.zntl.desk\logs\2026-09-06\`.

`events.log` (excerpt, times UTC+2 local as logged):
```
01:01:57 STATE Away→Sitting h=64cm idle=0s
01:01:57 CREDIT Partial dur=199s sitting=366
```
This proves the internal engine applied **partial** credit (not a reset) for
a 199s (~3.3 min) break — `sitting=366` is the post-credit `sitting_seconds`
value, non-zero.

Minute snapshots straddling that transition:

`01-01.json` (just before the transition, state `Away`):
```json
"sitting_seconds": 764,
"current_session_secs": 0,
```
`01-02.json` (24s after returning to `Sitting`):
```json
"sitting_seconds": 389,
"current_session_secs": 23,
```
`366` (post-credit, from the event log) `+ 23`s elapsed since re-entering
Sitting `= 389` — matches `sitting_seconds` in the snapshot exactly,
confirming the credited counter is live-accumulating correctly.
`current_session_secs` however reads `23` — i.e. it only ever counted the 23
seconds since the user sat back down; the 366s of "surviving" sitting credit
is invisible to it. Any UI bound to `current_session_secs` at `01-02.json`'s
timestamp would show ~23 seconds elapsed and a near-empty progress bar, while
the true credited state was 389/2400s (16%) — this is the exact symptom the
owner describes as "resets to zero."

Multiple further `CREDIT Partial` lines in the same log (`01:54:22 dur=115s
sitting=3047`, `02:07:00 dur=65s sitting=477`, `02:29:28 dur=194s
sitting=1190`) all show non-zero post-credit `sitting_seconds`, confirming
the pattern is consistent, not a one-off.

No `CREDIT Full` (reset-to-zero) events appear in the reviewed window,
because breaks in this log are all short (65-199s) relative to the
accumulated sitting time — under the current formula a short break simply
cannot zero out a large `sitting_seconds`. The zeroing the owner experiences
is happening in the UI layer (`current_session_secs`), not in this backend
counter.

## Model inconsistencies

| Finding | Path:line | Importance | Points |
|---|---|---|---|
| Popup's big timer + progress bar bound to `current_session_secs` (always hard-reset) instead of credited `sitting_seconds`/`limitUsedSecs` — root cause of the owner's complaint | `src/hooks/useDesk.ts:106,170`; `src/hooks/useWidgetData.ts:39`; `src/widgets/one-bar/OneBarTimer.tsx:28,59-68` | High | 3 |
| `useDesk.ts` state variable named `sittingSeconds` actually holds `current_session_secs` (uncredited per-bout timer), not sitting seconds — misleading name masks the bug | `src/hooks/useDesk.ts:42,106,170` | High | 1 |
| Widget background color (`temperature.ts`) and widget headline number (`OneBarTimer.tsx`) derive from two different, disagreeing counters — directly observable self-contradiction in one frame | `src/widgets/one-bar/temperature.ts:29-32` vs `src/widgets/one-bar/OneBarTimer.tsx:28` | High | 2 |
| Test suite actively asserts the buggy behavior as correct: `current_session_secs` is asserted near-zero right after a `Partial` credit is applied, in a test explicitly modeling the proportional-credit scenario | `src-tauri/src/session_tests_timers_live.rs:160-164` | High | 2 |
| `session_types.rs` doc comment explicitly designs `current_session_secs` to reset "after break credit" for "the session progress timer in the UI" — contradicts ADR-008's promise that breaks reduce, not reset, the session timer | `src-tauri/src/session_types.rs:112-114` | High | — (doc, see migration) |
| Break credit multiplier default is 2.0 (1s break cancels 2s sitting), not the 3:1 ratio the owner describes in the complaint — even after the wiring fix, the ratio itself under-credits relative to owner's stated expectation | `src-tauri/src/ergonomic_profile.rs:16`; `src-tauri/src/session_types.rs:17`; ADR `.arch/ADR/008-proportional-break-credit.md:22` | Medium | 1 |
| `UX-FLOW.md` documents the pre-ADR-008 fixed 3-tier credit table (5-9 min → subtract flat 1200s, ≥10 min → full reset) as current behavior — stale relative to both the code and ADR-008 | `.arch/UX-FLOW.md:342-346` | Medium | 1 |
| `UX-FLOW.md` also claims "Progress = `sitting_seconds / session_limit_secs`" (correct, credited) directly contradicting its own later section describing `current_session_secs` reset semantics, and contradicting what `OneBarTimer.tsx` actually renders | `.arch/UX-FLOW.md:113` vs `:318-333` | Medium | — (doc) |
| Four different "elapsed" notions coexist with no single source of truth: `sitting_seconds` (credited, daily), `current_session_secs` (uncredited, per-bout), `break_seconds`/`get_live_break_seconds` (break elapsed), `elapsed_secs` in `PolicyInput` (context-dependent alias built ad hoc per tick) | `session_types.rs:82-114`; `tray_controller.rs:124-128`; `communication_policy.rs:14` | Medium | 5 |
| `SessionState` mixes mutable runtime counters with static config copied in from the ergonomic profile (`break_min_secs`, `break_credit_multiplier`, `day_break_min_secs`, `posture_balance_min_sitting_secs`, `max_continuous_computer_secs`, `computer_break_reset_secs`) — config and state are one struct, so profile hot-reload must reach into live session state field-by-field | `src-tauri/src/session_types.rs:139-150` | Medium | 5 |
| Persistence layer hand-rolls a second, parallel snapshot type (`PersistedSessionState`) instead of (de)serializing `SessionState` itself — `SessionState` has no `Serialize`/`Deserialize` derive at all; every new persisted field must be added in two places | `src-tauri/src/session_persistence.rs:78-107` vs `src-tauri/src/session_types.rs:77-153` | Medium | 5 |
| Engine is not a pure `(state, event, now) → (state, signals)` function — several methods call `Utc::now()` internally (`on_reading`, `check_daily_reset`, `check_notification_conditions`) while sibling methods (`handle_state_exit`, live getters) take `now` as a parameter; time source is inconsistent across the same module | `src-tauri/src/session_reading.rs:12`; `src-tauri/src/session_daily.rs:13,23`; `src-tauri/src/session_breaks.rs:170` vs `src-tauri/src/session_breaks.rs:14` (param) | Medium | 8 |
| Tests work around the lack of an injectable clock by backdating timestamps (`Utc::now() - Duration::seconds(N)`) rather than injecting a fake `now` — works, but couples every test to wall-clock `Utc::now()` at call time and cannot deterministically test DST/leap-second edges | `src-tauri/src/session_tests_break_credit.rs:33-34,60-61,86-87,113-114,131-132` | Low | — |
| "Break" exists as at least three separate models that don't share a definition: `BreakCredit` (proportional, `session_breaks.rs`), `HourlyBreakTracker` (per-clock-hour boolean coverage, `hourly_break_tracker.rs`), and the Day Break Credit special-case inside `apply_break_credit` (resets notification flags/score, unrelated to sitting-time credit) | `src-tauri/src/hourly_break_tracker.rs`; `src-tauri/src/session_breaks.rs:126-158` | Low | 5 |
| Engine entangled with IO/tray/overlay indirectly: `tray_controller.rs` re-derives session-facing fields (`elapsed_secs`, standing-lap progress) from a fresh `snapshot()` on every tick rather than the engine emitting a single self-contained signal set; the "communication" layer duplicates knowledge of which field means what per state | `src-tauri/src/tray_controller.rs:98-151` | Low | — |

## Test coverage verdict

The proportional-credit math **is** tested — but only on the internal
`sitting_seconds` field, never on the field the UI actually renders:

- `session_tests_break_credit.rs` (all 5 tests) asserts exclusively on
  `m.state.sitting_seconds` (`:49,102,120,137`) and on
  `result.break_credit` (`:44,71,97`). None of these tests touch
  `current_session_secs`, the `SessionStateDto`, or the `StateChangedPayload`
  that the frontend actually consumes.
- The only tests that *do* touch `current_session_secs`
  (`session_tests_timers.rs:56-62,74-84,102-111`;
  `session_tests_timers_live.rs:27-33,44-63,136-165`;
  `session_tests_floating.rs:46-53`; `session_tests_scenarios_adv.rs:141-172`)
  all assert it resets to (or near) zero — including
  `session_tests_timers_live.rs:160-164`, a test explicitly built around a
  "120s break * 2.0 = 240s credit < ~300s sitting → Partial" scenario
  (comment at `:145`), which nonetheless asserts
  `s3.current_session_secs < 5, "should be near 0"` right after that partial
  credit. This test *proves* the developer knew a partial-credit scenario was
  in play and still coded the assertion for the reset behavior — the bug is
  not an oversight in test coverage, it is a codified expectation that
  contradicts the ADR the same repo claims to follow.
- No Rust test exercises the full `on_reading()` → `StateChangedPayload` →
  (simulated) frontend path to check that the number a user would see after
  a short break reflects the credit. No TypeScript test in
  `OneBarTimer.test.tsx` or `useDesk`-adjacent tests asserts that
  `currentSessionSecs` carries credit either (not independently re-verified
  file-by-file beyond the `useDesk`/`useWidgetData` wiring already traced
  above — see GAPS).
- Verdict: **coverage tests the helper (`sitting_seconds`), not the real
  path the user experiences.** The real path (engine → DTO →
  `current_session_secs` → `OneBarTimer` big number) has explicit tests that
  lock in the wrong behavior.

## Target architecture and migration steps

### Target shape

A single pure core:

```rust
struct ErgoState { /* one typed state: position, timers, credits, flags */ }
struct ErgoConfig { /* everything from ergonomic_profile.rs — no mutable copy inside ErgoState */ }
enum ErgoEvent { Reading { mm: i32, active: bool, now: DateTime<Utc> }, DayRollover(NaiveDate), ... }
enum Signal { Notify(NotificationEvent), TrayUpdate{..}, OverlayUpdate{..}, PopupUpdate{..} }

impl ErgoEngine {
    fn step(&mut self, event: ErgoEvent, cfg: &ErgoConfig) -> Vec<Signal>;
    fn snapshot(&self, now: DateTime<Utc>) -> ErgoSnapshot; // ONE DTO, one "elapsed", one "credited sitting"
}
```

Rules the target enforces that the current code violates:
1. **One canonical "time remaining in this sitting session" number.** Every
   consumer (overlay, popup, tray, notifications) reads the same
   `snapshot().sitting_used_secs` (today's `sitting_seconds`/`limitUsedSecs`).
   `current_session_secs` as a *separate*, uncredited concept is deleted; if
   a "time since last stood up" display is wanted, it is clearly a different,
   explicitly-named field (`secs_since_last_break`) never used for the limit
   bar.
2. **Time is injected, never read from the wall clock inside the engine.**
   `step()` takes `now` as an argument everywhere, including daily-reset and
   notification checks; adapters (serial thread, periodic tick) are the only
   places that call `Utc::now()`.
3. **One persistence snapshot type**, derived (`Serialize`/`Deserialize`)
   directly from `ErgoState`, versioned with a schema tag — no hand-mirrored
   `PersistedSessionState`.
4. **Config is not mixed into state.** `ErgoConfig` (from the ergonomic
   profile) is passed into `step()`/`snapshot()`, never copied field-by-field
   into the state struct.
5. **The engine returns `Signal`s; it does not know about tray/overlay/Tauri
   events.** `tray_controller.rs` becomes a thin adapter that maps `Signal`
   to WinAPI/Tauri calls, not a place that re-derives "elapsed" per tick.

### Migration path: strangler, not rewrite

A full rewrite risks re-introducing exactly this class of bug (a second
"elapsed" concept invented mid-migration) without the safety net of the
existing (if misdirected) test suite. Recommended path:

| Step | Description | Points |
|---|---|---|
| 1. Fix the reported bug directly (no architecture change) | Point `OneBarTimer`'s `elapsed` (Sitting branch) and `ProgressBar` at `limitUsedSecs`/`sitting_seconds` instead of `current_session_secs`; rename `useDesk.ts`'s `sittingSeconds` state to something honest if kept at all; fix or delete the tests that assert the reset (`session_tests_timers_live.rs:160-164` and siblings) to assert credited behavior instead | 3 |
| 2. Reconcile the ratio with the owner's stated expectation | Confirm with the owner whether 2.0 (current default) or 3.0 (owner's example) is the intended multiplier; if 3.0, update `ergonomic_profile.rs` defaults + `.arch/ADR/008-proportional-break-credit.md` + `UX-FLOW.md:113,342-346` in the same change (docs currently describe three different models) | 2 |
| 3. Collapse "elapsed" to one field | Delete `current_session_secs` and `SessionStateDto.current_session_secs`/`StateChangedPayload.current_session_secs`; update every consumer (`OneBarTimeline.tsx:49-52`, `useWidgetData.ts:39`, `DebugSection.tsx`) to the credited value; keep a distinctly-named "seconds since last position change" only if a real UI need for it is confirmed | 5 |
| 4. Inject the clock | Change `on_reading`, `check_daily_reset`, `check_notification_conditions` to accept `now: DateTime<Utc>` instead of calling `Utc::now()` internally; update call sites (`serial_periodic.rs` or wherever `on_reading` is invoked) to pass one `now` per tick; rewrite the backdating-based tests to inject `now` directly | 8 |
| 5. Single persistence snapshot | Derive `Serialize`/`Deserialize` on `SessionState` (or a purpose-built `ErgoState`), replace `PersistedSessionState` with a versioned wrapper around it, migrate `session_persistence.rs` to round-trip the real type | 8 |
| 6. Extract config from state | Move `break_min_secs`, `break_credit_multiplier`, `day_break_min_secs`, `posture_balance_min_sitting_secs`, `max_continuous_computer_secs`, `computer_break_reset_secs` out of `SessionState` into a `&ErgonomicProfile`/`&Limits` reference threaded through the methods that need them; profile hot-reload becomes "swap the reference," not "copy six fields" | 8 |
| 7. Signals instead of ad hoc tray re-derivation | Have the engine emit `Signal`s (or at minimum a single `PolicyInput`-equivalent struct) from `step()` itself rather than `tray_controller.rs` reconstructing `elapsed_secs`/lap progress from a fresh snapshot every tick; `CommunicationPolicy` becomes a pure consumer of engine output, not a second derivation site | 13 |
| 8. Unify "break" models | Decide whether `HourlyBreakTracker` and the Day Break Credit special-case are facets of one `Break` concept or genuinely separate metrics, and name them accordingly in code and docs so "break" stops meaning three unrelated things | 5 |

Total: **52 points** if done end-to-end; steps 1-3 (13 points) resolve the
owner's actual complaint and the worst doc/code contradictions without
touching clock injection or persistence, and can ship independently. Steps
4-8 are the structural cleanup and can be sequenced later without blocking
the fix.

## GAPS

- Did not read every one of the ~40 `session_tests_*.rs` files line-by-line —
  focused on the ones matching `current_session_secs`, `sitting_seconds =`,
  `reset`, and break-credit greps. It is possible another test file asserts
  something relevant that was missed; the grep-based sweep covered all files
  the pattern search returned, but a file using a synonym (e.g. a local
  variable also called `elapsed`) without touching the literal field names
  searched would not surface.
- Did not open `db_sessions.rs` beyond the first ~60 lines (insert/load
  scaffolding) — did not verify whether `TodayTotals` loading or the SQLite
  `sessions` table itself has any reset-adjacent logic beyond what
  `load_today_totals` (`session_manager.rs:165-176`) already showed.
- Did not read `communication_policy.rs` in full (only the `PolicyInput`
  struct and `evaluate` signature) — the escalation-step logic
  (`step_to_signals`, `maybe_apply_screen_nudge`) was not traced line by
  line; it's plausible additional inconsistencies live there, but the field
  it consumes (`elapsed_secs`, built correctly in `tray_controller.rs:124-128`)
  was confirmed correct at the boundary.
- Did not read the TypeScript test files (`OneBarTimer.test.tsx`,
  `SessionProgress.test.tsx`, `useRemoteDesk.test.ts`, etc.) to confirm
  whether any of them already asserts (or contradicts) the
  `currentSessionSecs`-drives-the-bar wiring; the frontend trace was done via
  source reading, not by running the existing test suite.
- Did not run `cargo test` or `pnpm test:unit` to confirm the currently
  documented test behavior (e.g. that `session_tests_timers_live.rs:160-164`
  actually passes today) — this is inferred from reading the assertion
  against the traced code, not from an executed test run.
- Did not check `useRemoteDesk.ts` (the phone/remote-display path) for
  whether it has the same `current_session_secs` wiring as `useDesk.ts` —
  greps showed it also exposes `limitUsedSecs` (`useRemoteDesk.ts:54,230-232`)
  but it was not opened to check what its own "big timer" equivalent (if any)
  binds to.
- Did not verify against a live, running instance of the app (no browser/UI
  verification performed) — per this repo's own trust rules, this is a
  code-and-log-based finding, not one confirmed by watching the popup render
  live. The log evidence (§ Log evidence) is real historical data from the
  user's own machine, which is the strongest available substitute, but it is
  not the same as observing the popup in real time.
- The owner's stated 3:1 ratio example ("2-minute walk extends allowed
  sitting by 6 minutes") was taken as an illustrative example of the desired
  *shape* of the fix (subtract, don't reset), not necessarily a literal
  request to change the multiplier from 2.0 to 3.0 — Step 2 of the migration
  explicitly flags this as needing a decision, not silently changed here.
