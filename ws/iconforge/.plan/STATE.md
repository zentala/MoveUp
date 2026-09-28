---
updated: 2026-09-28
active_epic: none (E001 code-complete, E002 done)
---

# IconForge — state

E002 done 2026-09-28: `path` primitive, `join.near-miss`/`join.overshoot`, `icon improve` loop; 115 tests green.
First live OpenRouter run (`anthropic/claude-sonnet-5`, ~$0.12/icon, 2 rounds): heart-rate and settings clearly
improved and adopted into `examples/icons/moveup/`; height-sensor not improved (kept); sitting failed when credit ran out.
Structured output is off for IconSpec (providers reject the recursive schema and the unrolled one is too large);
the prompt + `parseIconSpec` + one retry carry the contract.

Next: widen `improve` to the remaining flagged icons after topping up OpenRouter credit (BACKLOG).
Key: Password Broker `openrouter-api-key`, inject with `-EnvName OPENROUTER_API_KEY`.
