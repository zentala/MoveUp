# BACKLOG.md — Desk App

Items not yet scheduled, before refinement.

---

## Tauri Plugins — do dodania

- **`window-vibrancy`** (crate) — Windows Acrylic/Mica blur effect na floating window
  - wymaga `"transparent": true` w `tauri.conf.json` + `background: transparent` w CSS

- **`tauri-plugin-autostart`** — start aplikacji przy logowaniu do Windows

---

## App Icon — True Vector SVG

- `src-tauri/icons/icon-source.svg` is NOT a real vector — it's a PNG rasterized image wrapped in an SVG container (base64-encoded `data:image/png` inside `<image href=...>`). Colors cannot be changed via CSS/attributes.
- **To do:** Replace with a proper SVG drawn with `<path>` / `<rect>` elements so it can be recolored and scaled infinitely.
- Search terms: Noun Project `standing desk`, SVG Repo `adjustable desk`, Flaticon `sit stand`
- Or: design from scratch in Inkscape/Figma — a simple desk silhouette (horizontal top + two legs at different heights) works at 16px.
- Once real SVG exists: use `pnpm tauri icon source.png` to auto-generate all required sizes.

---

## Alert System — Future

- **AlertManager message strings in settings panel** — `AlertConfig` holds default neutral/positive message arrays. Expose as editable arrays so zentala can tune tone/wording without code changes. Depends on T015.
- **Notification strategy as pluggable system** — like widgets but for notifications. Different backends (toast, custom popup, both), different escalation patterns. Deferred until widget system proves the pattern.
- **Redesign alert popups — Tauri WebviewWindow instead of raw WinAPI** — current popups (`alert_popup_window.rs`) use raw WinAPI GDI, which looks like a 2003 Win32 dialog. Migrate to a Tauri WebviewWindow so we can style alerts with HTML/CSS using the instrument panel design system (--panel-*, --beam-*, --ink-*). This enables: dark themed popups matching the main window, animated transitions, rich content (progress bars, coach messages in popup), and the acrylic blur effect. Implementation: create a `popup.html` route, spawn via `WebviewWindowBuilder` (like welcome popup), communicate via Tauri events.

---

## Dokumentacja

- **Overlay Progress Bar Architecture Document** — pełny opis systemu paska na górze ekranu

---

## Sensor & Readings

- **Height stabilization algorithms** — moving average, 1cm rounding, trend locking (scheduled: T037)
- **Sensor diagnostics panel** — show raw vs smoothed readings, debounce state, threshold visualization
  - Helps debug standing detection issues like T036

---

## Persistent KPI Strip

Always-visible KPI strip on every view — fixed position, same place. Shows 4 daily ergonomics metrics with color coding.

### KPIs (all RELATIVE, not raw counts):

| KPI | Format | Green | Yellow | Red |
|-----|--------|-------|--------|-----|
| **Standing %** | `12%` | ≥15% | 10-15% | <10% |
| **Position changes/h** | `1.2/h` | ≥1.0 | 0.5-1.0 | <0.5 |
| **Hourly breaks** | `5/7h` | all hours covered | 1-2 missed | ≥3 missed |
| **Longest session** | `47m` | <45m | 45-75m | >75m |

- Position change count (raw number) = secondary/small, shown alongside changes/h
- No raw counts without context — everything relative to time worked

### Design rules:

- Widoczne na KAŻDYM widoku, stałe miejsce (np. dolny pasek, header strip)
- Kompaktowe — 4 liczby + kolory, zero tekstu opisowego
- Kolor komunikuje stan (zielony/żółty/czerwony) — nie trzeba czytać

### Position change definition:

- 5+ minut stania LUB 5+ minut away = 1 zmiana pozycji
- Standing i away są wymienne jako "przerwa"
- Cel: ~1 zmiana/godzinę

### Hourly break definition:

- Binarne sprawdzenie per godzina: "Czy była ≥5 min przerwa od ekranu?"
- KPI = ile godzin miało przerwę / ile godzin pracowaliśmy
- Jedna 30-min przerwa w jednej godzinie = 1/1 dla tej godziny, NIE nadrabia za inne
- Away = odpoczynek dla oczu (nawet stojąc, oczy pracują)

---

## Coach Messages → Progress Bar

Zamienić tekstowe coach messages ("Half limit used", "On track", etc.) w OneBarCoach na wizualne elementy. Istniejący 4px progress bar w OneBarTimer działa dobrze — coach message pod nim jest niepotrzebny/nieczytelny.

- Progress bar już istnieje i jest dynamiczny (OneBarTimer.tsx) — reużyć/rozszerzyć
- Usunąć lub uprościć coach text messages — bar sam komunikuje stan
- Ewentualnie: coach message jako tooltip, nie stały tekst

---

## Max Continuous Computer Time Alert

Alert: max ciągła praca przy komputerze. Standing ≠ przerwa od ekranu.

- Osobny timer "czas przy komputerze" (niezależny od sitting session timer)
- Thresholds: <45m green, 45-75m yellow, >75m red
- Alert po przekroczeniu limitu (domyślnie 75m, konfigurowalne)
- Reset: ≥5 min away from computer
- Pokazywane w KPI strip jako "Longest session"

---

## Activity Tracking

- **Activity status in UI** — show keyboard/mouse activity status (active/idle) in floating window
  - `activity.rs` already detects idle ≥60s via `GetLastInputInfo`, but UI doesn't expose this
  - Show: "Active" / "Idle 2m" in widget footer or status bar
  - Useful for debugging Walking/Away state transitions

---

## Logging & Observability

- **SQLite time-series storage** — replace file-per-minute snapshots with queryable SQLite table. Enables: search across days, trend analysis (sitting % over weeks), anomaly detection (daily score dropping), dashboard visualization. Natural phase 2 after T044 file-based logging proves useful. Effort: M, Priority: P3.

---

## Future Features

- **Notification A/B testing** — two backends simultaneously with feature flag (T018)
- **Success notifications + gamification** — streak tracking, milestone celebrations (T019)
- **Notification strategy plugins** — like widget system but for how/when to nudge
- **Phone-as-hub** — old phone + BLE sensor, works without desktop app
- **Smartwatch integration** — proximity detection, walking state, HRV
- Eksport danych do CSV
- Konfiguracja progów wysokości przez UI (kalibracja z UI)

---

## Removed (already implemented or superseded)

## Sensor & Stabilization

- **Extended height stabilization** — when desk is stationary for extended period (no significant movement), increase smoothing window 2x to eliminate residual jitter (e.g. 92→91→92→91 flickering). Current HeightStabilizer uses 10-sample window; for stationary desk, double to 20 samples or use exponential moving average with lower alpha. Detect "stationary" = all readings within ±3mm for last N seconds.

---

- ~~`tauri-plugin-notification`~~ — already integrated
- ~~`tauri-plugin-store`~~ — already integrated
- ~~Dynamiczna ikona tray~~ — superseded by T016 (color dot approach)
- ~~Standing Mode Theme~~ — superseded by T032 (widget temperature system)
- ~~Bar flashing at limit~~ — implemented in T013 (alert Stage 1, variant 2 pulsing)
- ~~Red popup at limit~~ — implemented in T013+T014 (AlertPopup)
- ~~Snooze logic~~ — implemented in T015 (deescalating snooze)
- ~~Color dot on tray icon~~ — scheduled as T016
