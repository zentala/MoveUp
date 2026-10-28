# Product vision

MoveUp helps its owner understand time spent sitting, standing and away from the desk, and experiment with configurable ways to take breaks.

Product entrypoints: [Purpose, current user and proposed audience](PURPOSE.md) and [User flows, current limitations and proposed experience](UX.md). The canonical visual contract remains [DESIGN.md](../DESIGN.md).

## Replaceable presence inputs and preserved motivation — 2026-10-28

The owner proposes treating Away/presence as an output independent of the input adapter: Windows input activity today, possible USB ultrasonic or other physical presence sensors later, initially attached to the computer and potentially to a controller. Record this as an architectural direction to assess, not an implemented integration, selected device or approved replacement of current credit/presence rules. Keep physical desk presence distinct from keyboard activity, desk height and room occupancy; unavailable observations must remain unknown. The owner also reports missing top-of-screen status and stand-up reminders and requests investigation of their actual behavior and regression coverage. Preserve configurable motivation signals and distinguish policy decisions, native execution and observed delivery before claiming reliability.

## Understandable accounting during AI-assisted work — 2026-10-27

The owner reports less continuous computer use during AI-assisted work, unexpected Away while still present, and a red history/large sitting balance that appears inconsistent with time spent away. The owner requests an evidence-based investigation, an explanation of the actual algorithm, and a whole-application UX review focused on the popup. Document who the application serves and its user flows; keep raw chronology, credit-adjusted balance, overtime and signed motivational score distinguishable.

Input inactivity and desk height do not establish physical presence or human posture. Preserve the existing truthful-observation direction: investigate actual data, retain Unknown gaps and do not rewrite history from recollection. New presence modes, thresholds, recovery-credit rules and UX changes remain proposals until accepted; the investigation itself changes no algorithm or live data. Existing dev-kit/open-core direction is retained rather than replaced by an invented AI market strategy.

## Direction recorded 2026-10-09

The compact popup and detailed Analyst should tell the same story using a reusable timeline, consistent state colors and a compact monospace charcoal design. A larger timeline should sit above connected smaller day tabs containing proportional daily donuts. Actual history, remaining sitting load after break credit, and daily score must be distinguishable. Data should appear on opening Analyst, and exact interval details should be inspectable.

These are recorded owner requirements, not a new product strategy. Existing ergonomic behavior and configurable motivation remain in scope.

## Direction clarified 2026-10-09

The selected day's heading and Range belong together above its timeline. A row of five to seven days uses one composition donut per day in place of grouped bars, with loading geometry visible immediately. Day at a Glance and desk-height history do not belong in the main view. Secondary trends must explain their meaning and their own scope; the value and retention of motivational score trajectory and break-credit histogram remain owner choices. Recorded machine states and events must not be presented as proven human posture or a normalized health score.

## Secondary detail decision, 2026-10-09

The owner accepted retaining both motivational score trajectory and break-credit histogram in an expandable details section, after correcting their calculations, missing-data handling and descriptions. The selected-day history and one composition donut per day remain the primary view.

## Core reliability and architecture direction — 2026-10-21

The owner explicitly commissioned implementation of the complete core-logic review, including its recommended proportional refactoring and the accepted undelivered Analyst details. The application should preserve observed time across gaps, day boundaries and restart; distinguish raw history from credited load; use one configurable decision for each notification; and carry local dates, observation quality and counters unchanged into the interface. Resolve uncertain cases through source and synthetic regressions rather than claiming defects without evidence. Prefer understandable existing modules and reusable current candidates over an application rewrite or new framework. Continue the shared timeline/day composition and expandable signed-score/recorded-credit direction. This is a reliability and comprehensibility goal, not a promise of medical or behavioral effectiveness. The owning execution plan is epics/E027-2026-10-09-analyst-timeline/PLAN.md; confirmed implementation authority and refusal boundaries are recorded privately by PO.
