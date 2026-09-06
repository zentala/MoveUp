# Brief for planning: Smartwatch Integration + AI Voice Dictation

## TLDR

This is a **research brief**, not a plan — a fresh planning session (using
the `Plan` agent) reads this, then writes `PLAN.md` + `HANDOFF.md` +
`PRES.md` under a new epic `E021-2026-09-06-smartwatch-integration/`. Two
research passes already happened (cheap model, file discovery + web
research); their findings are compiled below with file:line pointers so
the planning session does not re-discover them.

**Decision from Paweł**: yes, plan this fully — "będę to wydawał" (I will
ship this), "rozplanuj pełny 2a". He wants minimal further intervention:
prefer official SDKs so the result works "out of the box", tested against
real vendor SDKs where one exists.

## Ground truth — read these first, in this order

| File | Lines | Why |
|---|---|---|
| `C:\Users\zentala\.claude\skills\homelab\ref\wearables.md` | 1-36 | **Canonical, foundational.** Paweł owns a **Xiaomi Watch 4** — HyperOS on Xiaomi Vela/NuttX RTOS, a **closed system with zero third-party app API**. It only mirrors phone notifications. Contrast note in the same file: Xiaomi Watch 2/2 Pro run **Wear OS** and could run real apps, if he ever swaps hardware. |
| `.arch/ADR/012-google-fit-integration.md` | 1-42 | Current health-data integration decision (why Google Fit, what was rejected) — the pattern any new source should follow or explicitly diverge from |
| `src-tauri/src/google_fit.rs` | 1-100 | OAuth2 client, error classification (`auth_revoked` vs `transient`) |
| `src-tauri/src/google_fit_service.rs` | 1-80 | Caching, 5-min background poll, in-flight dedup — the shape a second health source would need to fit into (or replace) |
| `src-tauri/src/commands_google_fit.rs` | 1-24 | IPC contract: `get_steps_today`, `refresh_steps_now` |
| `src/components/StepsWidget.tsx` | 1-80 | UI render states (unconfigured / fresh / stale / auth_revoked / transient error), reconnect backoff |
| `CLAUDE.md` | "Google Fit Integration" section (search for it) | `.env` keys, OAuth helper script, data-source discovery/override |
| `.plan/vision/2026-03-24-business-vision.md` | 180-250 | Business framing: smartwatch integration + HRV as a Pro-tier feature |
| `.plan/vision/2026-03-25-premium-tier-definition.md` | 1-100 | Pro tier feature list; smartwatch integration positioned as closed-source paid feature |

## Hard facts already established — do not re-research these

1. **Step/activity data is already solved for Paweł's actual watch.** Xiaomi
   Watch 4 syncs steps to Google Fit via the Mi Fitness companion app
   (confirmed via Google Fit community support threads, 2026). The
   existing `google_fit_service.rs` pipeline already picks this up with
   **zero new code**. Do not plan a new Xiaomi-specific data path.
2. **No official SDK — on any of the three major platforms — exposes
   microphone access to a third-party app.** Checked all three:
   - **Apple Watch (HealthKit)**: no mic API for third parties; Voice
     Memos is a built-in-only capability.
   - **Wear OS (Health Services)**: the health/activity framework has no
     mic access; raw Android `MediaRecorder` exists on Wear OS but sits
     entirely outside Health Services and outside "official health SDK"
     framing.
   - **Garmin (Connect IQ SDK)**: no mic API despite newer hardware (Venu
     3, Fenix 8) having a microphone; repeatedly requested by developers
     on Garmin's own forums, never enabled.
3. **Consequence: voice-in cannot originate from a smartwatch under any
   official SDK, full stop** — not a Xiaomi limitation specifically, a
   platform-wide one. This holds even if Paweł buys a Wear OS watch.

## What this brief does NOT resolve — the planning session must decide

This is a genuine architecture choice with real trade-offs — the plan's
own "Alternatives considered" section (per `how-we-build/patterns.md`
convention: 2-3 named approaches, one minimum, one target) should cover:

- **(A) Voice capture on the phone, watch as glance/trigger surface only.**
  Feasible today with zero new hardware. The watch would show agent
  replies/nudges via notification mirroring (already works per
  `wearables.md`); voice goes in via a phone app (existing Android/iOS
  speech APIs), independent of watch brand.
- **(B) Defer voice-in entirely for this epic; ship only formalized
  multi-source step/activity aggregation** (Google Fit today, room for a
  second source later) as the actual E021 scope, and record voice-in as a
  separate, later epic once a Wear OS device is actually owned.
- **(C) Custom/unofficial firmware on the Xiaomi Watch 4** (Paweł mentioned
  he could flash custom software) — flag this as **explicitly rejected**
  or **explicitly out of scope**, since it contradicts his own "official
  SDK, out of the box" requirement and has no vendor support surface to
  test against. Name it in Alternatives so it's not silently forgotten,
  but the plan should not choose it as the recommended path.

State a clear recommendation among these (or a variant) — do not leave it
open-ended back to Paweł a second time; that decision belongs to this
planning session now that the facts are established.

## Constraints from CLAUDE.md / repo conventions to honor in the plan

- File ≤ 250 lines, function ≤ 50 lines (`.claude/rules/code-style.md`).
- New external dependency → needs an ADR (`.arch/ADR/`, next number after
  019 — check `.arch/ADR/` for the current highest before assigning).
- Test file conventions: Rust sibling `<module>_tests.rs`, TS co-located
  `<module>.test.ts(x)` (see `CLAUDE.md` "Conventions" section).
- Evidence contract required (`rules/evidence.md`) for every check the plan
  claims is verifiable.
- If the feature has any UI surface, the plan's acceptance criteria must
  include a browser-agent visual pass (`rules/testing.md`,
  `CLAUDE.md` "Never claim it works" section) — not just unit tests.
- Business context: `.plan/vision/config/pricing.json` is canonical for
  any pricing/tier claims — don't hardcode numbers from memory.

## Deliverables — where to save them

Create `.plan/epics/E021-2026-09-06-smartwatch-integration/` with:
- `PLAN.md` — full plan per this repo's convention (TLDR, Decisions and
  ADRs, Alternatives considered with the A/B/C above resolved into a
  recommendation, Architecture impact section, Test strategy, Acceptance
  criteria, Evidence contract).
- `HANDOFF.md` — task breakdown with points, `agent:` field per task, and
  a `## AO` YAML block ready for Agent Orchestrator dispatch (see
  `~/.claude/skills/ao/SKILL.md` §6 for the exact shape this repo now
  expects — and §0 for whether this epic even needs full AO or should be
  dispatched directly, based on its final point total and whether its
  waves have real parallelism).
- `PRES.md` — board-style presentation per `rules/workflows.md` "PRES.md"
  section (context/problem/solution/gains-losses-risks/points for each
  major decision), since Paweł will read this to approve, not the PLAN.md.

This session plans only — do not implement. Close by pointing Paweł at
`PRES.md` for approval.
