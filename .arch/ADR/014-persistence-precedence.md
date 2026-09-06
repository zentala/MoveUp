# ADR 014: Persistence precedence for "today's totals"

- **Status**: accepted
- **Date**: 2026-09-06
- **Epic**: E019 (backend hardening) — task T02
- **Related**: [ADR 008](008-proportional-break-credit.md),
  [ADR 012](012-analyst-dashboard-separate-window.md),
  [E015](../../.plan/epics/E015-2026-09-06-engine-single-truth/PLAN.md)

## Context

Three stores hold a piece of "what happened today", and none of them holds
all of it:

| Store | Holds | Written by |
|---|---|---|
| **SQLite** (`sessions` table) | every *completed* span: durations, state, break credit | `db_sessions::insert_session*` on each transition |
| **In-memory `SessionManager`** | the span still running, plus counters the DB cannot derive (`position_changes`) | `session_reading.rs` on each reading |
| **`tauri-plugin-store`** (`PersistedSessionState`) | notification flags, credited `sitting_seconds`, `daily_score` | `session_persistence.rs` at each save point |

Because they overlap, every caller that wanted a `TodaySummary` had to
combine them by hand — and they did not combine them the same way:

- `commands::get_today_summary` read the DB summary and then patched
  `position_changes` from the live session.
- `commands::ensure_initialized` read the DB summary and seeded
  `today_cache` **without** the patch, so the cache started every run
  claiming zero position changes.
- `db_queries::get_today_summary` hard-codes `position_changes: 0` with a
  comment saying the caller will fill it in — a contract enforced by
  nothing.

The DB genuinely cannot answer `position_changes` on its own: it only sees
closed sessions, so it is always one transition behind the desk.

A fourth reader looks like a duplicate and is not: Analyst's
`commands_analyst::collect_snapshots` walks the per-minute JSON snapshot
files. It answers a different question (per-minute series for charting),
not "what is the scalar total right now".

## Decision

**One composition point, one precedence rule.**

`src-tauri/src/today_totals.rs` owns the composition. `load_today_summary`
is the only supported way to obtain a display-facing `TodaySummary`.

Precedence, per field:

| Field | Authoritative store | Why |
|---|---|---|
| `sitting_secs`, `standing_secs`, `yesterday_*`, `sessions` | **SQLite** | durable, survives restarts, already aggregates closed spans |
| `position_changes` | **in-memory `SessionManager`** | the DB count excludes the open span and is always stale by one |
| notification flags, credited `sitting_seconds`, `daily_score` | **`tauri-plugin-store`** (out of scope for `TodaySummary`) | E015 owns this path; date-guarded on load |

`db_queries::get_today_summary` stays a pure DB query and keeps returning
`position_changes: 0`. That zero is not a value — it is "this store does
not know", and only `today_totals` is allowed to resolve it.

## Alternatives considered

- **Compute `position_changes` in SQL** (count state transitions among
  today's rows, as `db_sessions::load_today_totals` already does) —
  rejected: it still excludes the running span, so the number shown to the
  user would lag behind the desk they just moved. It would also make two
  differently-derived counters coexist, which is the exact bug class E015
  closed for session counters.
- **Make `TodaySummary.position_changes` an `Option<u32>`** so an
  unresolved value cannot be mistaken for a real zero — attractive, and
  the right answer if this field ever grows friends. Rejected for now
  because it is a wire type crossing IPC to TypeScript, and E018 is
  actively reworking that surface; changing its shape here would collide.
  Recorded as follow-up.
- **Leave the composition at each call site and just document it** —
  rejected: that is the current state, and it already produced one
  divergence (`ensure_initialized`) that nothing caught.

## Consequences

- `commands::get_today_summary` and `commands::ensure_initialized` both
  call `today_totals::load_today_summary`; the `ensure_initialized` seed
  now carries the live counter instead of zero.
- `commands_analyst::collect_snapshots` carries a doc comment pointing
  here, so it stops reading like a third implementation of the same
  scalar.
- **Not yet migrated:** `tray_helpers::refresh_today_cache` still composes
  the two stores by hand (`tray_helpers.rs:43-56`). It applies the same
  precedence, so behaviour is correct today, but it is a fourth hand-rolled
  copy. `tray_helpers.rs` is outside E019-T02's write set; migrating it is
  filed as follow-up work.
- Any new reader of today's totals adds a caller of `load_today_summary`,
  never a new `get_today_summary` + patch pair.
