# E002 journal

## 2026-09-27

- Schema `path` primitive added by coordinator (additive, version stays 1); spec addendum written first.
- OpenRouter key not in Password Broker (store empty on this machine, `bw` not installed) — asked via broker form `openrouter-api-key`.
- Wave 1 dispatched: renderer path, quality path + join rules, agent/CLI improve loop.
- Wave 1 done: renderer path (35 tests), quality path + `flatten` + `join.near-miss`/`join.overshoot` (37 tests), agent prompts + `improveIcon` + CLI `improve` + `revise` test (42 tests). Typecheck clean.
- Full suite: first run 1 failed / 114 (failure not captured), then 7 runs 114/114. Recorded as `failed` then flaky — BACKLOG.
- Join rules on the pilot set confirm the operator's "ragged" observation: settings 8 near-misses (rays stop 1.0–1.24 short of hub), steps 7, break 2, heart-rate 1 (pulse 0.38 short of heart), height-sensor 1; overshoots in standing/walking (torso 1.5 past arm) and posture-balance.
