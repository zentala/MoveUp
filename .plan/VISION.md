# Product vision

MoveUp helps its owner understand time spent sitting, standing and away from the desk, and experiment with configurable ways to take breaks.

Product entrypoints: [Purpose, current user and proposed audience](PURPOSE.md) and [User flows, current limitations and proposed experience](UX.md). The canonical visual contract remains [DESIGN.md](../DESIGN.md).

## Compact glance view and working personal sharing — 2026-10-29

After observing the delivered native popup, the owner requests a simpler main
hierarchy: top Share/Today/Settings/X actions, full estimated state and height,
no primary Recent input or permanent technical observation row, and a floating
window that stays above ordinary applications. Preserve truthful visible
stale/unknown/error states, accessible detail and focus-safe automatic reminders.
Closing the ordinary popup hides it to the tray while tracking continues. The
owner explicitly confirmed that clicking another application must leave the
popup visible above it until X, without taking typing focus back. The
owner also requests working social composer actions, personal habit-oriented
sharing text with actual daily statistics, and the new public website
`desk.labs.zentala.agency`. This corrects usability of the existing compact
instrument; it changes no ergonomic/presence/credit rules, historical data or
separate relay/telemetry service addresses. Owning follow-up:
`po/popup-sharing-review-2026-10-29.md`.

## Deliver all ten whole-product UX outcomes — 2026-10-28

The owner accepts implementation and delivery of all ten priority rows from the whole-product UX review. Improve understandable balance/overtime/actual break credit, settings recovery, observation freshness, low-input presence explanation, Welcome testing and first-day guidance, predictable settings/reminder actions, history/keyboard navigation, optional integration/sharing recovery and remote visibility. Preserve existing ergonomic/presence/credit rules, truthful historical gaps, network defaults and the compact instrument identity. Reuse the already accepted live-observation work rather than create another implementation. This is acceptance of the bounded UX recommendations, not sensor selection, retrospective history repair or validation of new market personas. Owning execution plan: epics/E031-2026-10-28-whole-product-ux/PLAN.md.

## Application-owned reminders and recurring bar visibility — 2026-10-28

The owner observed a Windows test banner but found its brief presentation unsuitable. Requested direction: show reminders as a MoveUp-owned message view in the existing compact tray popup, replacing its ordinary status content while a reminder is active, with distinct meaningful backgrounds. Preserve readable controls and an explicit way back to status; automatic presentation must not steal typing focus or disappear merely because another application has focus. Keep existing communication policy, Silent and snooze semantics authoritative. The owner also reports that the top bar disappeared again after the verified update: capture the current native/policy state before repair, distinguish intentionally hidden or visually neutral output from occlusion, and correct only established causes with regression coverage. This direction does not change presence inference, history or break credit.

## Replaceable presence inputs and preserved motivation — 2026-10-28

The owner proposes treating Away/presence as an output independent of the input adapter: Windows input activity today, possible USB ultrasonic or other physical presence sensors later, initially attached to the computer and potentially to a controller. Record this as an architectural direction to assess, not an implemented integration, selected device or approved replacement of current credit/presence rules. Keep physical desk presence distinct from keyboard activity, desk height and room occupancy; unavailable observations must remain unknown. The owner also reports missing top-of-screen status and stand-up reminders and requests investigation of their actual behavior and regression coverage. Preserve configurable motivation signals and distinguish policy decisions, native execution and observed delivery before claiming reliability.

## Known computer sleep versus missing observations — 2026-10-28

The owner requests analysis and a proposal that distinguish confirmed computer sleep from unexplained missing observations, input inactivity and sensor/application unavailability. Sleep describes the computer, not measured human posture. Preserve existing observed prefixes and Unknown gaps; record a known pause reason only when supported by qualified evidence. Showing that reason and granting break credit are separate product choices. The proposed direction is to evaluate confirmed sleep as a qualifying break from computer use using the existing credit rules, while unexplained missing time remains neutral. This is a proposal, not an implemented policy, retrospective history rewrite or claim of physical absence. The owner also reports notifications accumulated in Windows notification center without visible banners; app intent, system receipt and on-screen presentation must remain distinct evidence.

## Understandable accounting during AI-assisted work — 2026-10-27

The owner reports less continuous computer use during AI-assisted work, unexpected Away while still present, and a red history/large sitting balance that appears inconsistent with time spent away. The owner requests an evidence-based investigation, an explanation of the actual algorithm, and a whole-application UX review focused on the popup. Document who the application serves and its user flows; keep raw chronology, credit-adjusted balance, overtime and signed motivational score distinguishable.

Input inactivity and desk height do not establish physical presence or human posture. Preserve the existing truthful-observation direction: investigate actual data, retain Unknown gaps and do not rewrite history from recollection. New presence modes, thresholds, recovery-credit rules and UX changes remain proposals until accepted; the investigation itself changes no algorithm or live data. Existing dev-kit/open-core direction is retained rather than replaced by an invented AI market strategy.

## Beginner DIY setup direction — 2026-10-23

The owner wants a beginning programmer to obtain an RP2040-Zero, a VL53L0X
module and a 3 x 7 cm perfboard, then follow a photo-backed assembly article
and ask an agent from a cloned repository to prepare the USB device. The
repository should supply firmware, a discoverable setup skill, stepwise
wiring/flashing guidance and separate checks for the controller and the sensor.
Current work documents the module path and checks the already programmed board;
physical assembly and measurements wait for the ordered parts. A fresh-session
trial of the skill is planned. Firmware publication, whether it needs its own
repository, and flashing through MoveUp are future decisions/work, not delivered
capabilities. Hardware photos and the article belong to the companion
`desk.zentala.io/hardware/module-sensor/` owner, without replacing its separate
custom-PCB project.

## Direction recorded 2026-10-09

The compact popup and detailed Analyst should tell the same story using a reusable timeline, consistent state colors and a compact monospace charcoal design. A larger timeline should sit above connected smaller day tabs containing proportional daily donuts. Actual history, remaining sitting load after break credit, and daily score must be distinguishable. Data should appear on opening Analyst, and exact interval details should be inspectable.

These are recorded owner requirements, not a new product strategy. Existing ergonomic behavior and configurable motivation remain in scope.

## Direction clarified 2026-10-09

The selected day's heading and Range belong together above its timeline. A row of five to seven days uses one composition donut per day in place of grouped bars, with loading geometry visible immediately. Day at a Glance and desk-height history do not belong in the main view. Secondary trends must explain their meaning and their own scope; the value and retention of motivational score trajectory and break-credit histogram remain owner choices. Recorded machine states and events must not be presented as proven human posture or a normalized health score.

## Secondary detail decision, 2026-10-09

The owner accepted retaining both motivational score trajectory and break-credit histogram in an expandable details section, after correcting their calculations, missing-data handling and descriptions. The selected-day history and one composition donut per day remain the primary view.

## Core reliability and architecture direction — 2026-10-21

The owner explicitly commissioned implementation of the complete core-logic review, including its recommended proportional refactoring and the accepted undelivered Analyst details. The application should preserve observed time across gaps, day boundaries and restart; distinguish raw history from credited load; use one configurable decision for each notification; and carry local dates, observation quality and counters unchanged into the interface. Resolve uncertain cases through source and synthetic regressions rather than claiming defects without evidence. Prefer understandable existing modules and reusable current candidates over an application rewrite or new framework. Continue the shared timeline/day composition and expandable signed-score/recorded-credit direction. This is a reliability and comprehensibility goal, not a promise of medical or behavioral effectiveness. The owning execution plan is epics/E027-2026-10-09-analyst-timeline/PLAN.md; confirmed implementation authority and refusal boundaries are recorded privately by PO.

## Automatic restart safety — 2026-10-24

The owner expects ongoing automatic persistence even without sensor traffic, small bounded loss after an abrupt process exit or computer reset, and understandable actionable save failures. Improve the existing shared settings/engine/policy checkpoint and automatic consistent database backups; preserve known observations and mark offline time Unknown. This extends reliability of the delivered application rather than introducing a new product or promising zero loss. Windows supervisor configuration remains a separate operational responsibility and actual prior access refusals remain binding.

## Ongoing observations and understandable gaps — 2026-10-25

The owner explicitly approved planning and subagent implementation after seeing Unknown while currently sitting. Popup and Analyst must show the same ongoing confirmed observations through the last actual supported endpoint, distinguish real missing data from ongoing recording, and explain sensor connection, sample freshness and view update/error separately. No guessed past posture or filling across disconnect/sleep/restart. This extends the existing truthful-history direction without changing ergonomics/credit or rewriting history. Owning plan: epics/E028-2026-10-25-live-observation-timeline/PLAN.md.

## Understandable UI failures — 2026-10-27

The owner accepted implementing local per-window diagnostics after a blank main popup and Settings while Analyst remained usable. Collect native WebView2 failures, frontend liveness/mount/query hints and suspend/resume evidence, and save a report from the native tray without relying on the broken view. Reuse existing logs and preserve ergonomic observations, persistence and backend readiness. Observation must not silently become an automatic reload, restart policy or claim of proven cause. Owning execution plan: epics/E029-2026-10-27-ui-runtime-diagnostics/PLAN.md.
