# Vision: UX Communication — How the App Talks to the User

**Date:** 2026-03-20
**Status:** Vision — multiple tasks derived

---

## Core Philosophy

This app is a **personal movement coach**, not a productivity monitor. It should feel like a friendly assistant that genuinely cares — not a nagging boss. Two modes, two emotional tones:

- **Sitting mode**: gentle watch, escalating nudge when limit reached
- **Standing mode**: celebration, reward, gold visuals

The app should be **proactive but not annoying**. If the user dismisses 3× in a row, back off — change from nagging to inspiring.

---

## Welcome / Onboarding Popup (T025)

First impression matters. On first launch, a friendly popup introduces the app:

> **Cześć! 👋**
>
> Jestem twoim osobistym asystentem biurkowym.
> Pomagam Ci zadbać o ciało podczas pracy — żebyś się częściej ruszał i nie zapominał o przerwach.
>
> **Oto jak działam:**
> 🟢 Pasek na górze ekranu — widać, jak długo siedzisz. Zielony → żółty → czerwony.
> 🔔 Gdy przesiedzisz za długo — delikatnie Cię przypomnę.
> 🏆 Gdy wstajesz — nagradzam Cię złotym paskiem i punktami.
>
> Możesz przeciągnąć to okienko w dowolne miejsce na ekranie.
> Kliknij ikonę w zasobniku systemowym, żeby mnie ukryć lub pokazać.
>
> **Gotowy? Zacznijmy!** ✨

Popup shows:
- Only on first launch (flag stored in `tauri-plugin-store`)
- Has "Got it!" button to dismiss
- Has "Don't show again" checkbox
- Always-on-top, non-modal, draggable
- Friendly tone: emoji, warm text, not corporate

**Secondary purpose:** This popup also validates that WinAPI/custom popups work at all. Same infrastructure as alert popup (T013).

### Configurable
- `show_welcome_on_startup: bool` (default: true, can disable in settings)
- Can be re-triggered from settings: "Show welcome again"

---

## Notification Strategy (T024 + T018)

### Problem with current state
- Windows toasts rarely visible (Focus Assist blocks them, thresholds too high)
- Custom popup (T013) works but only for alert escalation
- No feedback loop — user never sees notifications → app feels dead

### Two-track approach with feature flag

```
NOTIFICATION_BACKEND=toast   → use tauri_plugin_notification (Windows toast)
NOTIFICATION_BACKEND=popup   → use custom WinAPI popup (same as alert popup)
NOTIFICATION_BACKEND=both    → show both (for A/B comparison during dev)
```

This is a developer feature flag for testing. Long-term: custom popup wins (full design control).

### When notifications should fire

**Too aggressive is as bad as no notifications.** Proposed thresholds:

| Event | Trigger | Type | Configurable |
|-------|---------|------|--------------|
| Sit limit reached (Stage 1) | sitting ≥ limit | pulsing bar (no popup) | no |
| Sit limit +2 min (Stage 2) | alert_manager | popup | yes |
| Standing 5 min | first 5 min standing in session | toast/popup "Keep going!" | yes |
| Standing target reached | 15 min standing | toast "Great break!" | yes |
| Daily posture balance | sitting > 2× standing | toast (once/day) | yes |
| Inactivity | no movement for **60 min** (not 90) | toast | yes |

---

## Tray Icon Design (T016 redesign)

### Concept
White base icon (desk silhouette or up/down arrow with desk) + small colored dot in corner.

**Dot colors:**
- 🟢 Green — sitting, <60% session limit
- 🟡 Amber — sitting, 60–85%
- 🔴 Red — sitting, >85% (or alert active)
- 🟡 Gold — standing (progress toward target)
- ⚪ Gray — away / disconnected

**Standing state** should NOT be gray (neutral). Gold communicates "good, you're doing the right thing."

### App icon design direction
The icon should show a **desk** — the product's identity. Options to explore:
- Silhouette of adjustable desk (high/low positions as two states)
- Up arrow + desk surface (suggests "raise your desk")
- Abstract: two horizontal lines at different heights (sit vs stand levels)

Search terms to find inspiration:
- "standing desk icon SVG"
- "adjustable desk icon minimal"
- "sit stand desk icon"
- Noun Project: "standing desk"

Implemented as SVG → converted to PNG at 16x16, 32x32, 256x256, .ico.

---

## Body Position Extensibility (Future Architecture — T-ARCH-001)

### Current model
Binary: Sitting / Standing (+ Walking + Away as derived states).

### Future model
The desk height sensor gives us a **continuous spectrum**. We can map that to more positions:

```
Height (cm)   Position
──────────────────────────────────────────────
< 60 cm       Kneeling (e.g. kneeling chair)
60–80 cm      Sitting (standard chair)
81–95 cm      Leaning / Perch (leaning stool, saddle chair)
96–120 cm     Standing (standard standing)
> 120 cm      Standing tall / treadmill desk
```

This is just a **height threshold mapping** — same sensor, richer semantics.

### Architectural requirements (to keep open)
- `DeskPosition` enum should have a `Custom(cm: u32, label: String)` variant or be extensible
- Session stats should track **all** positions, not just sitting/standing total
- Points system should assign different values per position:
  - Sitting: −0.5/min
  - Leaning/perch: −0.1/min (better than sitting, not as good as standing)
  - Standing: +1/min
  - Kneeling: +0.5/min
- Thresholds should be user-configurable (you might have a non-standard desk)
- `position_label()` helper returns the human-readable name for a given height

### Standing: max session duration
Standing too long is also bad. Max continuous standing session: **90 min** (configurable, option: 60 min).
After max: gentle nudge "Consider sitting or walking for a bit."
This is symmetric to the sitting limit — the app cares about movement, not just standing.

### When to implement
Not now — requires:
1. UI for configuring height thresholds per position
2. Session model refactor (sitting_secs → Map<Position, u32>)
3. Points system extension

Keep this in vision. Mark `DeskPosition` enum with `// EXTENSIBLE: see vision doc` comment when next touching session.rs.

---

## Related Tasks
- `T023` — Fix tooltip: show standing time, update every second
- `T024` — Investigate + fix notifications
- `T025` — Welcome/onboarding popup on first launch
- `T026` — App icon design (SVG + PNG assets)
- `T016` — Tray icon: white base + colored dot, gold when standing
- `T018` — Notification A/B: both backends with feature flag
