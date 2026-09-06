---
id: E018-T02
epic: E018
status: pending
priority: medium
effort: medium
dependencies: ["E018-T01"]
tags: [frontend, rust, codegen]
created_at: 2026-09-06
points: 5
agent: ts-dev
branch: feat/E018-T02-ts-rs-codegen
---

# E018-T02: Rust→TS type codegen with ts-rs

## Objective

Replace the hand-copied DTO interfaces in `src/types.ts` and
`src/components/settings/SettingsTypes.ts` with types generated from the
Rust structs they mirror, using `ts-rs` (decision D3 — see PLAN.md
Alternatives; `specta` rejected as more machinery than a DTO-only mirror
needs).

## Context

`src/types.ts:59-89` (`SessionStateDto`, `StateChangedPayload`,
`DashboardState`, `MetricSnapshot`, `TodaySummaryDto`) and
`src/components/settings/SettingsTypes.ts:10-26` (`DeskSettings` mirroring
`src-tauri/src/config.rs`'s `AppConfig`) are hand-typed with no compiler or
test signal when the Rust side changes
([`frontend.md`](../../../reports/_review-2026-09-06/frontend.md) finding #5).

**Do not duplicate E015.** E015-T02 adds a manual JSON-fixture key-set
drift test between a Rust `serde_json::to_string` output and `types.ts`.
Before writing any code, grep `src/` for that test (search for
`dto-drift`, or for a `.test.ts` importing a fixture emitted via
`serde_json::to_string`) and delete it once generated types make it
redundant — a compile error is strictly stronger than a runtime key-set
diff.

## Tasks

- [ ] Add `ts-rs` to `src-tauri/Cargo.toml` (dev-dependency is enough if
  export only runs under `cargo test`).
- [ ] Add `#[derive(ts_rs::TS)] #[ts(export, export_to = "../src/generated/")]`
  to the DTO structs in `src-tauri/src/session_types.rs`,
  `src-tauri/src/config.rs`, and any DTO struct returned by commands in
  `src-tauri/src/commands.rs` / `commands_analyst.rs`.
- [ ] Regenerate via
  `cargo test --manifest-path src-tauri/Cargo.toml --lib -- export_bindings`
  (ts-rs's own export-on-test-run convention) — confirm `src/generated/*.ts`
  is produced. Do not add a new `package.json` script for this; T05 in the
  same wave (0/2) already touches `package.json` and the two must not
  collide.
- [ ] `src/types.ts`: re-export the generated interfaces instead of
  hand-typing them; keep any pure-frontend types (e.g. `WidgetProps`,
  `DeskWidget`) as-is.
- [ ] `src/components/settings/SettingsTypes.ts`: same treatment for
  `DeskSettings`/`AppConfig`.
- [ ] Delete the superseded E015 manual drift test (see Context above).
- [ ] Write ADR `.arch/ADR/017-ts-rs-for-rust-ts-codegen.md` covering the
  ts-rs vs specta tradeoff: ts-rs is a derive-only, DTO-shape exporter with
  no runtime dependency and a stable, narrow API surface; specta's real
  strength — auto-generating invoke-safe command bindings via
  `tauri-specta` — is a larger architectural commitment (rewiring every
  `invoke()` call site) than this epic's DTO-mirroring problem calls for.
  Revisit specta if/when full command-binding generation becomes a
  separate, deliberate epic.

## Four shadow paths

- **happy** — a generated `SessionStateDto` type in `src/generated/` has
  the same field set as the Rust struct; `pnpm typecheck` passes.
- **nil** — an `Option<T>` Rust field renders as `T | null`, not
  `T | undefined`, in the generated output.
- **empty** — a struct with zero optional fields still generates a valid,
  non-empty interface (sanity check against a degenerate case).
- **error (the drift case itself)** — renaming a Rust field without
  regenerating breaks `pnpm typecheck` on the consuming TS files; this IS
  the check that proves the codegen boundary is load-bearing, not
  decorative.

## Acceptance Criteria

- No DTO interface in `types.ts`/`SettingsTypes.ts` is hand-typed anymore
  — each re-exports from `src/generated/`.
- `pnpm typecheck` passes.
- E015's manual DTO-drift test file no longer exists.
- ADR 017 written and linked from `.plan/decisions.jsonl` (D3).
- All new/changed files ≤ 250 lines.

## Cross-references

- [`PLAN.md`](../PLAN.md) §Decisions and ADRs (D3), §Alternatives.
- [`HANDOFF.md`](../HANDOFF.md) §Mental model — codegen boundary.
- E015: [`../E015-2026-09-06-engine-single-truth/PLAN.md`](../../E015-2026-09-06-engine-single-truth/PLAN.md) (manual drift test this supersedes).
- Source review: [`frontend.md`](../../../reports/_review-2026-09-06/frontend.md) finding #5, §Recommended target structure item 4.
