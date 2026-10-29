# MoveUp product purpose

Part of the product document set: [Vision](VISION.md) · [User experience](UX.md) · [Design contract](../DESIGN.md). Updated 2026-10-28.

## Why it exists

MoveUp helps a person using a sit/stand desk notice prolonged desk sessions, choose a change of position or a break, and understand the recorded day. It is also the owner's platform for experimenting with configurable, quiet motivation. The primary outcome is an understandable opportunity to move, rather than a larger dashboard or more notifications.

Three quantities must remain distinct: recorded chronological desk states, the current sitting balance after configured break credit, and optional motivational points. A break can reduce the balance without erasing earlier history. Missing observations remain unknown.

## Users and target audience

| Audience | Status and need | Product implications |
|---|---|---|
| The current owner, using one Windows computer and one motorized desk | Established user in the repository contract; wants reliable history, understandable reminders and fast experiments | Local operation, a small tray popup, explainable defaults and configurable profiles |
| Technically comfortable developers and other desk workers with compatible DIY USB hardware | Proposed early adopter audience, consistent with the existing dev-kit direction; broader demand is not established by this document | Discoverable assembly/setup guidance, calibration help and a useful first day without reading source code |
| People doing AI-assisted work, reading, thinking, listening or working intermittently away from the keyboard | Owner-reported current context; overlaps both audiences rather than forming a new commercial segment | Input inactivity must not be described as measured physical absence; work away from the computer should not require unnecessary interaction with the app |

The application currently serves one desk and one person. Employee monitoring, attendance measurement, productivity scoring and workplace fleet management are outside its current purpose. The recorded signals do not establish a person's physical posture, exercise, screen attention or health outcome.

## Working personas and their evidence — 2026-10-28

These are behavioral working personas, not research-validated market segments. They consolidate the existing owner reports and dev-kit direction; no interviews, demographic research or measured demand are implied. Technical fluency does not imply knowledge of MoveUp's accounting rules.

| Working persona | Evidence and status | Goal and underlying need | Typical flow and success signal |
|---|---|---|---|
| Focused desk worker | Primary current user: the owner using one Windows computer and adjustable desk | Notice a useful opportunity to move while keeping attention on work; trust the current estimate without managing a timer | Work → glance or receive a reminder → stand or leave → return. Can explain the displayed balance, overtime and next action without reading documentation |
| Motivation experimenter | Secondary mode of the same current user, established in the repository contract; not an independently validated second audience | Adjust reminder behavior and limits, see which rule is effective, and recover from an unsuccessful experiment | Inspect effective configuration → change one setting/profile → preview → work → inspect observations. Knows what changed, when it applied and how to revert |
| First-time DIY desk user | Proposed early adopter, supported by the owner's beginner setup request; usability and demand outside the owner remain unvalidated | Turn compatible parts into a useful desk aid without needing source-code knowledge; know when recording has actually started | Connect/assemble hardware → identify device → calibrate sitting/standing → confirm a real observation → understand first reminder. Can distinguish device detection, sensor measurement and inferred desk state |

AI-assisted work, reading, listening and voice interaction are contexts across these personas. A person may still be working while away from the computer, or present without input. Automation keeping a computer awake does not establish the person's presence. Do not create a separate AI market persona from this one owner's report.

## Needs, product goals and validation

The product goal is to make an understandable opportunity to change position available. The user's goal is to continue their work with fewer unwanted prolonged desk sessions and less interruption. Standing percentage, points, popup opens and reminder count are supporting signals, not proof of that outcome.

| Need | Product response to evaluate | Proposed acceptance observation; baseline not yet measured |
|---|---|---|
| Understand now, quickly | One named balance, separately named overtime, estimated state and recording quality | After a brief glance, the owner identifies what the number measures and whether the displayed information is current |
| Benefit from a break without app administration | Automatic observed-break accounting with an accessible explanation | On return, the owner can find the last observed break, qualifying rule and actual credit without asking an agent |
| Keep attention on work | Quiet background cues, configurable persistent reminders and explicit dismissal | A reminder remains readable without stealing typing focus; the person understands the consequence of its actions |
| Trust incomplete evidence | Separate inactivity, desk height, missing observations and supported pause reasons | The person can distinguish inferred Away from an unexplained gap; no unknown interval becomes confirmed posture |
| Control configuration | Visible effective settings, clear apply/save boundaries and recovery | Before leaving Settings, the person predicts what is already saved and what will be discarded |
| Start successfully | Hardware/setup guidance and a verifiable first observation | A new user reaches and recognizes actual recording, or gets a concrete explanation of the missing prerequisite |

Validate first with the current owner using task-based observation, then with real first-time dev-kit users when available. Record actual task completion and mistaken interpretations. No conversion, retention, behavior-change or medical-effectiveness measurements have been collected by this documentation update.

## AI-assisted work changes the interaction assumption

The owner's 2026-10-27 report says that AI-assisted work means less continuous time at the computer, and that Away also appeared while the owner was still present. This is a recorded requirement context, not a population-level market claim.

The existing signals are desk height and recent Windows input. They cannot distinguish reading an AI response while seated from leaving a low desk, or a voice conversation at the desk from an unattended computer. Keeping these limitations visible is accepted documentation direction. Selecting a new presence sensor, manual work mode, longer inactivity threshold or voice-based posture inference remains a product decision; none is implemented or silently approved here.

There is also an inverse risk: an agent or remote-control tool that generates native Windows input may keep the input-activity heuristic active while the person is away. Whether a particular tool does so depends on its input path; terminal output or browser-only events must not be assumed to reset Windows idle time. This is a scenario to verify, not an established cause of the owner's recorded day.

## Core jobs

1. While working: understand the current estimate and sitting balance in one glance, then decide whether to move.
2. When leaving and returning: receive the configured break credit without manually operating a timer, while understanding the limitations of inactivity-based detection.
3. When a number looks wrong: inspect the relevant interval, its source and missing coverage, and find out why credit was or was not applied.
4. When the app or sensor stops supplying observations: see that recording is uncertain, rather than a confident Sitting/Away claim.
5. When configuring the app: understand the effective thresholds, when changes take effect and how to recover from a failed load/save.

## Observable success

These are acceptance goals, not measured results:

- The owner can explain the difference between history, credited sitting balance and overtime from the UI without asking an agent.
- A qualifying observed break reduces the balance by the effective configured rule; a short break and an unknown gap have distinct explanations.
- A day view exposes actual time coverage and keeps both confirmed states and unknown time proportionate.
- Reading/listening or AI-assisted work does not silently become a claim of measured presence or physical absence.
- Normal use requires no configuration editing, mandatory cloud account or repeated app interaction during a break.
- A recording/UI failure has understandable local evidence and does not require a restart merely to find out that evidence is missing.

## Authority and scope

Existing operator direction is maintained in [VISION.md](VISION.md); visual choices belong to [DESIGN.md](../DESIGN.md). Existing business/dev-kit/open-core strategy remains in [BUSINESS_CONTEXT.md](BUSINESS_CONTEXT.md); this clarification does not alter pricing, licensing or delivery commitments. Current behavior and proposed improvements are separated in [UX.md](UX.md). The exact current operator request and runtime investigation are private PO records, not public product claims.
