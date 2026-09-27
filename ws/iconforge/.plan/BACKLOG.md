# IconForge — backlog

| Item | Importance | Points |
|---|---|---|
| Spec phase 3: blind 24 px evaluation of the pilot set, record acceptance, cost, time, failure types | High | 3 |
| Collect 3–5 licensed reference icons for few-shot examples | Medium | 2 |
| Spike: Paper.js adapter for smooth curves vs. the direct serializer | Low | 3 |
| Semantic macros (`person`, `chair`, `monitor`) compiled to primitives, each with a reference test | Low | 8 |
| Pilot defects (coordinator review 2026-09-27): sitting reads as "2", steps look like carrots, heart-rate is a diamond (needs arcs), settings reads as sun, snooze moon reads as "C", walking ≈ standing at 24 px | High | 3 |
| First live `generate` run against OpenRouter; record cost/time per icon | High | 2 |
| Automated test for `icon revise` (mock provider, like generate) | Medium | 1 |
| Decide `ajv` as a devDependency vs. the hand-rolled mini validator in agent tests | Low | 1 |
| Flaky test: first full `vitest run` after E002 wave 1 reported 1 failed / 114; 7 reruns green, failing test not captured (suspect 5 s timeout in a PNG-rendering CLI test under load) | Medium | 1 |
