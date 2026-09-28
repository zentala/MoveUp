# E002 journal

## 2026-09-27

- Schema `path` primitive added by coordinator (additive, version stays 1); spec addendum written first.
- OpenRouter key not in Password Broker (store empty on this machine, `bw` not installed) — asked via broker form `openrouter-api-key`.
- Wave 1 dispatched: renderer path, quality path + join rules, agent/CLI improve loop.
- Wave 1 done: renderer path (35 tests), quality path + `flatten` + `join.near-miss`/`join.overshoot` (37 tests), agent prompts + `improveIcon` + CLI `improve` + `revise` test (42 tests). Typecheck clean.
- Full suite: first run 1 failed / 114 (failure not captured), then 7 runs 114/114. Recorded as `failed` then flaky — BACKLOG.
- Join rules on the pilot set confirm the operator's "ragged" observation: settings 8 near-misses (rays stop 1.0–1.24 short of hub), steps 7, break 2, heart-rate 1 (pulse 0.38 short of heart), height-sensor 1; overshoots in standing/walking (torso 1.5 past arm) and posture-balance.

## 2026-09-28 — T04 live run

- OpenRouter key: not in any local store (mATX broker, mITX, homelab-secrets empty; `secrets.internal` is an unbuilt app whose Caddy vhost fails TLS). Captured once via broker as `openrouter-api-key`; injected with `-EnvName OPENROUTER_API_KEY`.
- Run 1 failed: 400 "Circular reference detected" — strict json_schema from the recursive zod export is rejected by Anthropic/Google/Azure. Rewrote `agent/src/jsonSchema.ts` as an acyclic, constraint-free schema (groups unrolled to depth 3, `path` added).
- Run 2 failed: 400 "compiled grammar is too large". Dropped the provider-enforced schema from `askForSpec` (`loop-helpers.ts`); the prompt carries the type description, `parseIconSpec` + one retry enforce the contract (spec allows this).
- Run 3 failed: 402 — no `max_tokens`, so OpenRouter reserved the model default 65 536 against ~$0.65 of remaining credit. Added `SPEC_MAX_TOKENS=8000`, `REVIEW_MAX_TOKENS=1500`.
- Run 4 (2 rounds, $0.50 cap, `anthropic/claude-sonnet-5`): heart-rate $0.142, height-sensor $0.100, settings $0.127, all `max-rounds`; sitting failed "response missing choices[0].message.content" (credit ran out) and aborted the batch — fixed: a per-icon provider failure now records `history.json {error}`, continues, still writes the sheet, exits 3 (new CLI test).
- My earlier "exit code 0" on run 1 was my own `| tail` hiding the CLI's exit 3, not a CLI bug.
- Coordinator evaluation of before/after (512 and 24 px, `ws/out/e002-compare.png`):
  - heart-rate: better — diamond became a smooth arc heart; at 24 px the pulse still crowds the inside. Adopted.
  - settings: much better — rays-on-a-hub "sun" became a cog with rounded teeth, reads as settings at 24 px. Adopted.
  - height-sensor: not better — model added a desk top and down-pointing waves (semantically right) but the waves overlap the stem; at 24 px it reads as a trident/tree. Kept the original.
- Decision on widening: YES, 2 of 3 clearly improved at ~$0.12/icon. Next batch: sitting, steps, snooze, walking, standing, height-sensor (retry with a sharper brief) — needs ~$0.75 of credit. Recorded in BACKLOG.
