# ADR 010: Notification Escalating Silence

- **Status**: accepted
- **Date**: 2026-03-30
- **Epic**: E000-maintenance
- **Context**: Users reported notification spam when ignoring sit-limit alerts.
  Two separate notifications (Windows toast at limit, in-app popup at +5 min) fired
  with different messages, confusing users. Without cooldown, notifications repeated
  every second when ignored, creating strong incentive to disable notifications entirely.
- **Decision**:
  1. **Visual-only escalation**: the second escalation step (at +5 min) only intensifies
     visual signals (blink tray, pulse overlay) — no second notification type.
  2. **Escalating silence**: after each notification, a growing cooldown applies before
     the next one can fire (default: 0s → 5m → 15m → 30m → permanent silence).
  3. **Configurable per profile**: cooldown schedule lives in `snooze.notify_cooldowns_secs`
     so aggressive/gentle/silent profiles can tune independently.
  4. **Dismiss resets count**: user clicking Dismiss = engagement, so the notification
     schedule resets to zero (fresh start after snooze expires).
- **Alternatives**:
  - *Keep both toast + popup*: rejected — two channels with different messages confuses users.
  - *Fixed cooldown (e.g., always 5 min)*: rejected — doesn't account for "user is clearly ignoring".
  - *Never stop reminding*: rejected — drives users to disable notifications entirely,
    defeating the purpose. The app's philosophy is nudging, not nagging.
- **Consequences**:
  - Maximum 4 notifications per sitting session (unless dismissed).
  - After all reminders exhausted, only visual cues (overlay pulse, tray blink) remain.
  - Different profiles can have different patience levels (aggressive = shorter cooldowns).
  - The `notify: "popup"` option still works if a custom profile re-enables it,
    but built-in profiles no longer use it.

## Current presentation — 2026-10-28

[ADR 026](026-application-owned-reminders.md) supersedes only the delivery surface:
enabled ergonomic and normal device-disconnection notices use the existing main
window, including legacy toast/popup/both profile inputs. The cooldown schedule,
visual escalation, configurable silence and engagement rules above remain accepted.
Exceptional direct persistence/shutdown errors retain their separate paths.
