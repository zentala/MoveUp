---
id: E012-T07
epic: E012
status: in-progress
created: 2026-05-17
decisions_locked: 2026-05-18
branch: feat/E012-T07-timeline-strip
title: Redesign State Gantt → Daily Timeline Strip
---

## Brainstorm output (2026-05-18)

Original design stress-tested via brainstorming skill. Critical concerns:
**C1** time-axis distortion (12%/6h compresses sides 2×), **C2** abrupt midnight
auto-advance, **C3** Sleep-band false positives, **C4** range-picker conflict,
**C6** loss of multi-day pattern visibility (StateGantt regression).

User redirected design to **Hybrid Alt-B** (brushable context + detail strip)
with proportional time scaling and graceful overnight handling.

## Final design (locked 2026-05-18)

### Architecture: TWO components (locked 2026-05-18, after visual review)

```
┌─────────────────────────────────────────────────────────────────┐
│  DATE NAVIGATOR — range header + 14 day tabs with mini bars     │
│  Range May 04 → May 17           [7d] [14d✓] [30d] [custom]   │
│                                                       today      │
│  8h ┊                                                   ↓        │
│     ┊  ▌  ▌  ▌  ▌  ▌  ▌  ▌  ▌  ▌  ▌  ▌  ▌  ▌  ▌                │
│  4h ┊ ▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌                  │
│   0 ┊─────────────────────────────────────────────                │
│       Mon Tue Wed Thu Fri Sat Sun Mon Tue Wed Thu Fri Mon[Tue]   │
│       04  05  06  07  08  09  10  11  12  13  14  15  16  17    │
│  each day = 4 mini bars: Sit · Stand · Walk · Away              │
├─────────────────────────────────────────────────────────────────┤
│  TIMELINE DETAIL — one continuous proportional timeline          │
│  selected day centered; clicking another day = smooth-scroll     │
│  animation across intermediate days                              │
│ ┌─ Tue · May 17 ─ Last session ended 01:42 ─────────────────┐  │
│ │ ◀ │ Mon 18-24 │  Tue 00-24 (selected, centered) │ Wed 00-6 │▶│
│ │ ░░ │ faded 50% │ full opacity, hero            │  faded   │  │
│ └────────────────────────────────────────────────────────────┘  │
│  uniform pixel scale: 1 min = same width across whole strip     │
└─────────────────────────────────────────────────────────────────┘
```

### Component 1: `DateNavigator.tsx`

Merges previous "range picker" + "context strip" into a single navigation card.

**Header row**:
- Range display: `May 04 → May 17` with clickable date pills (opens date picker)
- Preset chips: `7d` / `14d` / `30d` / `custom`

**Body**: grouped mini bar chart — 1 column per day in range:
- 4 thin bars side by side: **Sit · Stand · Walk · Away**
- Each bar height = hours in that state (shared Y scale, 0–8h with dashed gridlines)
- Y axis labelled (`0` / `4h` / `8h`)
- Below each column: day name (`Mon`) + day number (`04`), date acts as the tab
- Weekend day labels dimmed
- "TODAY" tag above the current day
- Selected day = blue outline + bg tint + bold blue date
- Hover any column = subtle bg highlight
- Click any column = sets `selectedDay`, triggers smooth-scroll in detail strip

**Footer**: legend (Sit/Stand/Walk/Away color chips) + interaction hint.

### Component 2: `TimelineDetail.tsx`

A **single continuous horizontal timeline** that spans the whole selected range.
Conceptually it's one long scrollable strip — clicking a day in DateNavigator
triggers a **smooth scroll animation** centring that day in the viewport. Days
the user "passes through" are visible during the animation.

- **Uniform pixel scale**: 1 minute = same width across the whole strip. No fisheye.
- **Side previews**: when a day is centred, the strip naturally shows ~6h of
  neighbour days on each side (faded at 50% opacity, subtle bg tint).
- **Day dividers**: thin vertical line at every 00:00 boundary.
- **Now indicator**: blue line + dot at the real "now" position. Only present
  when current scroll position covers `Date.now()`.
- **Header**: `◀` / `selected date (clickable picker)` / `▶` + neutral late-night
  label "Last session ended 01:42" when applicable.
- **Tooltip on hover**: timestamp · state · duration of contiguous run · score.
- **Scroll animation**: triggered by DateNavigator clicks (~400–700ms ease-in-out,
  duration proportional to jump distance, capped). Native scroll-snap-to-day
  on user drag.

### Shared utility: `timeline-utils.ts`

- `dayToPx(day, totalRangeStart, pxPerMinute)`
- `pxToDay(px, totalRangeStart, pxPerMinute)`
- Scroll math for smooth-scroll animation (target offset, easing).
- Snap calculations.

### Interaction model

| Action | Result |
|---|---|
| Click day in context strip | `selectedDay` = that day |
| `◀` / `▶` buttons in detail header | selectedDay ± 1 day |
| Keyboard `← →` on focus | selectedDay ± 1 day |
| Date trigger ("Tue · May 17 ▾") | Opens native date picker |
| Drag detail strip horizontally | Free pan; snap-to-hour at slow velocity, snap-to-day at fast |
| Mouse wheel on detail strip | Same as drag |
| Hover any segment | Tooltip |
| Click a segment | (Future: cross-filter sessions table — out of scope for T07) |

### Auto-advance rule

- **Live mode** = user hasn't pressed nav button in last ~60 min AND `selectedDay` is today (or yesterday before 02:00).
- **At midnight (00:00)**: `selectedDay` does NOT change. Detail strip naturally shows ~6h of new day in right preview. User sees the overlap. No jump.
- **At 02:00 local time**: if user is still in live mode, `selectedDay` flips to new calendar day. Strip now centres on new day; ~3-6h of previous day visible in left preview. Smooth label transition, no content jump (preview was already showing this region).
- Manual nav (◀▶/keyboard/click context strip) → exits live mode until user clicks "Today".

### Range picker coexistence (C4)

- Range picker (existing) → defines **range for context strip + other charts**.
- Detail strip → independent `selectedDay`, initialised to last day of range.
- Brushing context strip → updates `selectedDay`, NOT range.
- Changing range → if `selectedDay` falls outside new range, snap to nearest in-range day.

### Removed from MVP

- **Sleep band** (was C3): cut. "Away" stays honest. → see backlog.
- **Late-night red flag** (C7): replaced by neutral "Last session ended HH:MM" label in detail header, no colour.
- **Click-segment-cross-filter** (C9): deferred to follow-up task — Explorer doesn't yet have a sessions table.
- **Week-view toggle** (Q5): deferred to E012-T08.

### Data

- Snapshots already fetched via `useSnapshotsRange` over user-selected range.
- Detail strip uses snapshots filtered to `[selectedDay - 6h, selectedDay + 30h]` client-side.
- Context strip uses full range bucketed per hour (14×24 = 336 buckets).

### Accessibility

- Strip is `role="region"` with `aria-label="Daily activity timeline"`.
- ARIA live region announces `selectedDay` changes ("Showing Tuesday, May 17").
- All interactive elements keyboard-reachable; focus visible.
- Tooltips also rendered as `<title>` SVG for screen readers.

### Performance

- Pre-aggregate snapshots to 5-minute buckets in the detail strip (max 36h × 12 = 432 rects).
- Context strip uses hour buckets (14 × 24 = 336 rects).
- Now-indicator timer uses single `setInterval(60_000)` at parent level.

### Layout in ExplorerTab

```
┌─ Date range + status line ───────────────────────┐
│ TimelineContextStrip (full width, ≈40px tall)    │  ← new
│ TimelineDetailStrip (full width, ≈220px tall)    │  ← new (replaces StateGantt)
├──────────────────────────────────────────────────┤
│ KpiTrend │ BreakCreditHistogram │ DeskHeightTL  │  ← existing grid
├──────────────────────────────────────────────────┤
│ DailyScoreTrajectory (full width)                │  ← existing
└──────────────────────────────────────────────────┘
```

### File plan

| File | LoC budget |
|---|---|
| `analyst/charts/DateNavigator.tsx` | ≤220 |
| `analyst/charts/TimelineDetail.tsx` | ≤230 |
| `analyst/charts/timeline-utils.ts` (scale, smooth-scroll math) | ≤140 |
| `analyst/hooks/useTimelineNav.ts` (selectedDay state, auto-advance, scroll trigger) | ≤120 |
| Tests for each above (`*.test.ts(x)`) | mirroring |

Total new code ≈700 LoC across 4 source files. All files ≤250 lines per project rule.

### Acceptance criteria

- [ ] Context strip renders configured days in range; brushing changes selectedDay.
- [ ] Detail strip uses proportional pixel-per-minute scale; no time distortion.
- [ ] `◀` `▶`, keyboard `← →`, date picker, and context-strip brush all update selectedDay consistently.
- [ ] Now indicator visible when `[selectedDay - 6h, selectedDay + 30h]` covers `Date.now()`.
- [ ] At 02:00 local, live-mode selectedDay auto-advances to new day with smooth label change.
- [ ] Tooltips show timestamp + state + duration + score on hover.
- [ ] Late-night neutral label in header when last session ended after 00:00.
- [ ] StateGantt removed; ExplorerTab grid recomposed per layout above.
- [ ] All new files under LoC budget; functions ≤50 lines.
- [ ] Tests: ≥80% line coverage on new code; explicit cases for midnight auto-advance, brush math, proportional scale, snap velocity.
- [ ] Mockup page `/#/mockup/analyst` updated to show new components with fixture data.

### Tests outline

| Test | Asserts |
|---|---|
| `timeline-utils.test.ts` — scale | 1 min = same px in center and previews |
| `timeline-utils.test.ts` — snap | slow velocity → hour snap; fast → day snap |
| `useTimelineNav.test.ts` — midnight | At 00:00, selectedDay unchanged |
| `useTimelineNav.test.ts` — 02:00 flip | After 02:00 with no manual nav, selectedDay = new day |
| `useTimelineNav.test.ts` — manual exit | Pressing ◀ disables auto-advance for 60min |
| `TimelineContextStrip.test.tsx` | Brush click → onSelectDay called |
| `TimelineDetailStrip.test.tsx` | Renders 36h proportionally; side bands at 50% opacity |
| `TimelineDetailStrip.test.tsx` | Now indicator only when range covers now |
| `TimelineHourAxis.test.tsx` | Day-boundary divider at 00:00 transitions |

## Plan

1. ✅ Lock decisions (this section)
2. **frontend-design pass** — produce visual mockup of the hybrid; append "After" panel to `E012-T07-preview.html` for A/B comparison
3. User reviews HTML A/B → approves visual
4. Implementation in worktree `feat/E012-T07-timeline-strip`
5. Tests + Mockup page update
6. Merge → close task


# E012-T07: Redesign State Gantt → Daily Timeline Strip

## Context

Obecny `StateGantt` rysuje N wierszy (jeden per dzień) na ~⅔ szerokości
dashboardu, obok niego stoi `DeskHeightTimeline`. Problem:

1. To **najważniejszy** wykres — pokazuje surowy "co robiłem w ciągu dnia" —
   ale leży zakopany jako jeden z 5 boxów.
2. Wiele wierszy pod sobą jest mało czytelnych — nie widać kontekstu
   przejścia dnia w dzień (wieczór → noc → wczesny ranek).
3. Brakuje fabuły — user chce zobaczyć JEDEN dzień centralnie + skrawki
   wczoraj/jutro żeby zauważyć "siedziałem do 2 w nocy" albo "wstałem o 5".

## Goal

Zastąpić obecny `StateGantt` komponentem **Daily Timeline Strip** na pełną
szerokość, jako pierwszy box w `ExplorerTab`.

## Ideas — pierwszy szkic (do przepuszczenia przez design review)

### A. Layout

```
┌──────────────────────────────────────────────────────────────────────┐
│  ◀  Tue May 17 ▾                                              ▶     │
├──────────────────────────────────────────────────────────────────────┤
│ │   yesterday (last 6h)    │      TODAY 00:00 → 24:00         │ tom │
│ │░░░░░░ standing ░░░░░░░░│░░░ sit ░░░░░░ stand ░░ away ░░░░│░    │
│ │      18:00      22:00  │ 00  06  12  18  24                │ 06 │
│  ╰ dimmed / 50% opacity ╯ ╰ full opacity ╯                    ╰dim╯
└──────────────────────────────────────────────────────────────────────┘
       drag to scroll · ◀▶ to jump days · click hour to focus
```

- Środek (full opacity): pełne 24h **wybranego dnia**.
- Lewy bok (~12% width, dimmed): ostatnie ~6h dnia poprzedniego.
- Prawy bok (~12% width, dimmed): pierwsze ~6h dnia następnego.
- Subtle pionowe dividery na granicach 00:00 + delikatne tło zmieniające
  ton dla "yesterday" / "today" / "tomorrow".
- Pasek godzin pod spodem ciągły (zaczyna się od 18:00 wczoraj, kończy 06:00 jutro).

### B. Nawigacja

- Przyciski `◀` / `▶` w nagłówku — skok o 1 dzień.
- Date picker w środku (klik na "Tue May 17 ▾" → date input).
- Klawiatura: `←` / `→` = dzień, `Home` / `End` = start/koniec zakresu.
- Scroll horyzontalny myszką/touchpadem — płynne przewijanie.
  - Snap do pełnej godziny przy puszczeniu (CSS `scroll-snap-type: x proximity`).
- Drag (mouse down + drag) — bezpośrednie przewijanie taśmy.

### C. Wizualne wskazówki

- **Now indicator**: pionowa kreska + dot na "teraz" w widoku dzisiejszym
  (jeśli wybrany dzień = today). Aktualizowana co 60s.
- **Sleep band**: jeśli wykryjemy ≥4h ciągłego `Away` w godz. 22:00–08:00 →
  delikatne ciemniejsze tło + ikona księżyca (sygnał: spałeś).
- **Late-night flag**: jeśli ostatni session-end po 00:00 — czerwona kropka
  w nagłówku ("after midnight").
- **Hover tooltip**: timestamp + state + score + długość ciągłego segmentu.

### D. Interakcje

- Klik segment → focus na tym session row w tabeli niżej (cross-filtering).
- Klik godzina w pasku → ustawia hover-cursor na tej minucie, pokazuje
  tooltip "14:23 — Standing for 12 min".

### E. Data model / perf

- Renderujemy ~36h danych (18h yesterday + 24h today + 6h tomorrow), ale
  z paginacją per day — fetch tylko gdy user przewinie poza zakres.
- Buckets per minuta (1440/dzień × 1.5 = 2160 rect) — to się da SVG-em.
- Alternative: `<canvas>` jeśli FPS leci.

### F. Multi-day view (opcjonalnie, jako tryb)

Toggle "Day | Week" w nagłówku:
- **Day** (default): jak wyżej.
- **Week**: 7 wierszy małych pasków, ale **z zachowaniem strip-style**
  (lewa/prawa krawędź wyblakła), żeby user widział spójność tygodnia.

## Open questions (do design review)

1. **Width of side previews** — 12% / 15% / 20%? Trade-off: więcej kontekstu
   vs mniej miejsca na "dzisiaj".
2. **Czy snap on scroll release** ma snapować do hour, day, czy nic? Day-snap
   może być irytujący przy przewijaniu długich zakresów.
3. **Color encoding** — czy zostawiamy 4 kolory (Sit/Stand/Walk/Away) czy
   redukujemy do 2 (active/away)? 4 mogą "krzyczeć" na full-width.
4. **Co z Sleep band na nieznanym retention** — pokazywać "?" jeśli brak
   danych z 22:00–08:00?
5. **Czy "Day | Week" toggle** to scope tego taska, czy follow-up?

## Plan

1. **Design review** — odpalić skill `frontend-design` (lub `brainstorming`)
   na tym dokumencie, zebrać alternatywy + uwagi.
2. **Decyzje** — user + agent ustalają finalny look.
3. **Implementation** — nowy komponent
   `apps/desk/src/analyst/charts/DailyTimelineStrip.tsx` zastępuje `StateGantt`
   w `ExplorerTab.tsx`. Stary `StateGantt` usuwamy (history w git).
4. **Tests** — unit (segment rendering, scroll math), E2E smoke (Chrome
   DevTools MCP: render, scroll, day-skip, hover tooltip).
5. **Mockup page update** — `AnalystMockup.tsx` pokazuje nowy widok.

## Acceptance criteria

- [ ] Komponent zajmuje pełną szerokość kontenera (CSS `width: 100%`).
- [ ] Renderuje 1 dzień centralnie + ~6h previews po bokach.
- [ ] `◀` / `▶` zmieniają dzień; klawiatura `← →` działa gdy focus.
- [ ] Now indicator widoczny gdy selected day = today.
- [ ] Wszystkie segmenty mają tooltip z timestamp + state + duration.
- [ ] Test coverage: ≥80% nowego kodu.
- [ ] Pierwszy box w `ExplorerTab`, powyżej KPI grid.

## Constraints

- Plik ≤ 250 lines (split na sub-komponenty jeśli trzeba).
- Funkcje ≤ 50 lines.
- No new deps (SVG / native scroll only).
- Brak emoji w UI — ikony jako inline SVG.
