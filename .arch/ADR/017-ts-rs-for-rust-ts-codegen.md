# ADR 017: ts-rs generates the TypeScript mirror of the Rust DTOs

- **Status**: accepted
- **Date**: 2026-09-06
- **Epic**: E018 (task E018-T02, decision D3)

## Context

Rust owns the wire format. Every DTO that crosses the Tauri IPC boundary —
`SessionStateDto`, `StateChangedPayload`, `DashboardState`, `MetricSnapshot`,
`TodaySummary`, `AppConfig`, `PortInfo` — also existed as a hand-typed
interface in `src/types.ts` and `src/components/settings/SettingsTypes.ts`.
Nothing tied the two together, so a field renamed in Rust produced no compiler
error and no failing test; the frontend simply read `undefined` at runtime.

This was not hypothetical. When the codegen landed it immediately exposed
three live drifts that no test had caught:

- `TodaySummaryDto` was missing `yesterday_sitting_secs`,
  `yesterday_standing_secs` and `position_changes`.
- `SessionEntry` declared `duration_secs: number` where Rust sends
  `Option<i64>`, and omitted `break_credit` entirely — so a row that predates
  the column, whose duration is *unknown*, was typed as a number.
- `DeskSettings` still declared `sit_limit_mins`, `stand_limit_mins` and three
  `notify_*` flags that moved to the ergonomic and communication profiles.
  `get_settings` returns `AppConfig`, which has none of them, so the
  Notifications tab's checkboxes have been reading `undefined`.

E015 had already built a partial answer: a Rust test serialising the structs
into `src/test/fixtures/session-dto.json`, plus a TypeScript test asserting
the interface key sets matched it. It worked, but it covered two types by
hand, and it reported drift as a failing test rather than as a type error.

## Decision

Derive the TypeScript from the Rust structs with **ts-rs**, emit into
`src/generated/`, and re-export from `src/types.ts` and `SettingsTypes.ts`.
The generated files are committed; `.plan` treats them as source, not build
output, so a reviewer sees the wire format change in the diff.

Two details are deliberate:

- **ts-rs is a dev-dependency.** The derives are `#[cfg_attr(test, ...)]`, so
  nothing about the shipped binary changes.
- **The export runs from an explicit `ts_rs::Config`** (`lib.rs`, module
  `ts_export`) rather than from `#[ts(export)]`'s auto-generated tests. Those
  run under `Config::from_env()`, which renders `i64` as `bigint`; the DTOs
  reach JavaScript through `JSON.parse`, which yields `number`. A `bigint`
  binding would typecheck and then throw on the first arithmetic. A companion
  test asserts no `bigint` survives, so that trap cannot come back quietly.

Regenerate with:

```
cargo test --manifest-path src-tauri/Cargo.toml --lib -- ts_export
```

E015's fixture pair (`src/test/dto-drift.test.ts`,
`src/test/fixtures/session-dto.json`, `src-tauri/src/session_dto_fixture_tests.rs`)
is deleted. A compile error is strictly stronger than a runtime key-set diff,
and keeping both would mean two things to update on every field change.

## Alternatives

**specta + tauri-specta.** Its real strength is generating invoke-safe
*command* bindings — argument types and return types per `#[tauri::command]`,
so `invoke("get_settings")` stops being a stringly-typed call. That is worth
having, and this repo has a bug it would have caught: `SettingsPanel` calls
`invoke("save_settings", { settings })` while the Rust command's parameter is
named `config`, so saving settings cannot work. But adopting it means rewiring
every `invoke()` call site, which is a larger architectural commitment than
the DTO-mirroring problem in front of us. Revisit specta when full command
binding generation is a deliberate epic of its own.

**Keep E015's fixture approach and extend it by hand.** Rejected: it scales
linearly in hand-written key lists, and it fails at test time rather than at
compile time.

**Hand-typed interfaces plus review discipline.** Rejected — it is what we had,
and it produced the three drifts listed above.

## Consequences

- Renaming a Rust wire field and regenerating breaks `pnpm typecheck` on every
  consuming file. Verified by doing exactly that before this ADR was written.
- Nullability now crosses the boundary honestly: `Option<T>` becomes `T | null`,
  so consumers must say what an unknown value renders as. The Debug tab prints
  `unknown` for a session with no recorded duration instead of `0s`.
- `SessionEntry.state` is `string`, not `DeskState`, because the DB column is a
  string. `deskReducer` narrows it through `toDeskState`, mirroring Rust's
  `DeskState::from_db_str`: an unrecognised value is unknown, never coerced.
- Adding a DTO means adding two `cfg_attr` lines and one `export_all` call —
  a type that nobody exports silently stays hand-written, which is the failure
  mode to watch for at review.
- The stale `DeskSettings` fields are quarantined as optional and documented,
  not deleted: the settings UI that reads them is outside this task's scope.
  Deleting that dead UI, and fixing the `save_settings` argument-name bug, are
  follow-ups.

## References

- [E018 PLAN.md](../../.plan/epics/E018-2026-09-06-frontend-consolidation/PLAN.md) §Decisions and ADRs (D3)
- [ADR 008 — proportional break credit](008-proportional-break-credit.md) (why `break_credit` is nullable)
- E015 — the fixture-based drift check this supersedes
