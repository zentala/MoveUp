# MoveUp product purpose

Part of the product document set: [Vision](VISION.md) · [User experience](UX.md) · [Design contract](../DESIGN.md). Updated 2026-10-27.

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
