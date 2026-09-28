# IconForge — backlog

| Item | Importance | Points |
|---|---|---|
| Spec phase 3: blind 24 px evaluation of the pilot set, record acceptance, cost, time, failure types | High | 3 |
| Collect 3–5 licensed reference icons for few-shot examples | Medium | 2 |
| Spike: Paper.js adapter for smooth curves vs. the direct serializer | Low | 3 |
| Semantic macros (`person`, `chair`, `monitor`) compiled to primitives, each with a reference test | Low | 8 |
| Pilot defects (coordinator review 2026-09-27): sitting reads as "2", steps look like carrots, heart-rate is a diamond (needs arcs), settings reads as sun, snooze moon reads as "C", walking ≈ standing at 24 px | High | 3 |
| First live `generate` run (planner from a brief) against OpenRouter; `improve` ran live 2026-09-28 at ~$0.12/icon | High | 2 |
| Decide `ajv` as a devDependency vs. the hand-rolled mini validator in agent tests | Low | 1 |
| Flaky test: first full `vitest run` after E002 wave 1 reported 1 failed / 114; 7 reruns green, failing test not captured (suspect 5 s timeout in a PNG-rendering CLI test under load) | Medium | 1 |
| Widen `improve` to sitting, steps, snooze, walking, standing, height-sensor (sharper brief: waves must not overlap the stem) — decided 2026-09-28 after 2/3 improved; top up OpenRouter credit (~$0.75) first. Inputs: `ws/out/e002-in/briefs.json` pattern, `ws/iconforge/packages/cli/src/commands/improve.ts` | High | 2 |
| `improve` reports "4 round(s)" for `--rounds 2` — history counts review and revise entries separately; make the summary count review rounds (`ws/iconforge/packages/cli/src/commands/improve.ts`, summary line) | Low | 1 |
| Adopted heart-rate/settings specs carry off-grid warnings and a 0.06 pulse overshoot (`ws/iconforge/examples/icons/moveup/{heart-rate,settings}.json`) — snap to 0.5 grid in a later revise round | Low | 1 |
