# ADR 011: Unified Sit-Stand-Walk Cycle (Screen Time Integration)

- **Status**: accepted
- **Date**: 2026-03-30
- **Epic**: E000 (maintenance — design phase)
- **Context**: The app tracks sitting time (40 min limit) and needs to also track continuous screen/computer time (60 min limit per medical consensus). Initial design proposed two independent escalation systems (sitting alerts + computer time alerts) using the same visual channels (tray color, overlay bar). This creates UX confusion — user sees yellow tray and doesn't know if it's sitting limit or screen limit. Two overlapping alert systems on the same channels = chaos.

  Additionally, research shows standing at a desk is NOT a screen break — the user is still fixated on the monitor at the same distance with the same blink suppression (Cornell, CCOHS, multiple CVS studies). The existing system treats standing as a "break" but it only breaks the sitting posture, not the screen exposure.

  Research basis: [Screen Time & Eye Health Report](../../.plan/reports/screen-time-eye-health-research.md), [Sit-Stand-Walk Cycle Research](../../.plan/reports/sit-stand-walk-cycle-research.md)

- **Decision**: One unified three-phase cycle instead of two independent alert systems.

  **Three phases of a work cycle:**
  1. **Sit** (default limit: 40 min) — tracked by `sitting_seconds` as today
  2. **Stand** (no hard limit for standing itself, but computer_time keeps ticking) — standing provides sitting break credit but NOT screen break credit
  3. **Walk away** (goal: 5 min away from screen) — resets BOTH sitting credit and computer time

  **How it works:**
  - `continuous_computer_secs` tracks total time at computer (Sitting + Standing). Resets only after 5+ min Away.
  - At sitting limit (40 min): message offers a choice — "Stand up **or** walk away from the screen for a few minutes"
  - Walking away is presented as the **better** option (resets everything)
  - Standing is still valid (resets sitting timer via break credit) but computer_time continues
  - When user is already standing AND `continuous_computer_secs` approaches/exceeds 60 min: gentle nudge messages encouraging a screen break ("Grab a glass of water", "Quick 2-min stretch", "Your eyes need a break too")
  - These nudges use a **separate, lighter channel** — not tray/overlay escalation (which is reserved for sitting limit). Implementation TBD: could be a subtle text in the popup widget, a one-time gentle toast, or a message in the standing UI.

  **Key principle**: the desk MUST go up (or user must leave) — there's no "dismiss and keep sitting" option. The choice is stand OR walk away, both are valid responses. Walking away is encouraged as the premium choice because it serves both posture and eye health.

  **Configurable in ergonomic profile:**
  - `max_continuous_computer_secs`: 3600 (60 min default, based on medical consensus)
  - `computer_break_reset_secs`: 300 (5 min away = full reset)

- **Alternatives**:
  1. **Two independent escalation systems** (sitting + computer time) — rejected because same visual channels (tray yellow/red) would conflict. User can't distinguish which limit is being signaled.
  2. **Separate ComputerTimeMonitor module** — rejected because it duplicates escalation logic and creates signal conflicts on tray/overlay.
  3. **Only KPI badge color change** (no alerts) — rejected because it's too passive; medical research says 60 min is a real threshold that needs active intervention.

- **Consequences**:
  - CommunicationPolicy needs awareness of `continuous_computer_secs` to adjust standing-phase messages
  - Standing UI gets a new "nudge" layer — gentle encouragement to take screen break, not a hard escalation
  - Ergonomic profile gains `max_continuous_computer_secs` and `computer_break_reset_secs` fields
  - Communication profile gains screen-break nudge messages (configurable per profile)
  - Feature toggle: `enable_computer_time_tracking` in AppConfig (default: on)
  - Sitting limit toast message changes from "Time to stand" to "Time to stand up or step away from the screen"
  - This is a philosophy shift: the app moves from "sit/stand tracker" to "full work cycle coach" (sit → stand → move)
