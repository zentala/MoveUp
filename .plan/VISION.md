# Product vision

MoveUp helps its owner understand time spent sitting, standing and away from the desk, and experiment with configurable ways to take breaks.

## Direction recorded 2026-10-09

The compact popup and detailed Analyst should tell the same story using a reusable timeline, consistent state colors and a compact monospace charcoal design. A larger timeline should sit above connected smaller day tabs containing proportional daily donuts. Actual history, remaining sitting load after break credit, and daily score must be distinguishable. Data should appear on opening Analyst, and exact interval details should be inspectable.

These are recorded owner requirements, not a new product strategy. Existing ergonomic behavior and configurable motivation remain in scope.

## Direction clarified 2026-10-09

The selected day's heading and Range belong together above its timeline. A row of five to seven days uses one composition donut per day in place of grouped bars, with loading geometry visible immediately. Day at a Glance and desk-height history do not belong in the main view. Secondary trends must explain their meaning and their own scope; the value and retention of motivational score trajectory and break-credit histogram remain owner choices. Recorded machine states and events must not be presented as proven human posture or a normalized health score.

## Secondary detail decision, 2026-10-09

The owner accepted retaining both motivational score trajectory and break-credit histogram in an expandable details section, after correcting their calculations, missing-data handling and descriptions. The selected-day history and one composition donut per day remain the primary view.

## Core reliability and architecture direction — 2026-10-21

The owner explicitly commissioned implementation of the complete core-logic review, including its recommended proportional refactoring and the accepted undelivered Analyst details. The application should preserve observed time across gaps, day boundaries and restart; distinguish raw history from credited load; use one configurable decision for each notification; and carry local dates, observation quality and counters unchanged into the interface. Resolve uncertain cases through source and synthetic regressions rather than claiming defects without evidence. Prefer understandable existing modules and reusable current candidates over an application rewrite or new framework. Continue the shared timeline/day composition and expandable signed-score/recorded-credit direction. This is a reliability and comprehensibility goal, not a promise of medical or behavioral effectiveness. The owning execution plan is epics/E027-2026-10-09-analyst-timeline/PLAN.md; confirmed implementation authority and refusal boundaries are recorded privately by PO.
