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

## KPI "Session" Label — Confusing, Rename Required

- KPI badge labeled "Session" actually means "longest continuous screen time without 5+ min away"
- User interprets "Session" as sitting session or app uptime — both wrong
- **Rename to**: "Screen" or "Screen time" or "At desk" — something that communicates "continuous time at computer"
- **Possible bug**: Away detection doesn't work (Away state unreachable) → screen time counter NEVER resets → always red after 75 min. Fix Away state first (see "State Machine Redesign" below), then verify this KPI resets properly.
- **Not covered by tests**: no test verifying that 5+ min away resets the longest_session counter in practice (only unit tests on counter logic, but Away is never triggered by sensor)

---

## Max Continuous Computer Time Alert

Alert: max ciągła praca przy komputerze. Standing ≠ przerwa od ekranu.

- Osobny timer "czas przy komputerze" (niezależny od sitting session timer)
- Thresholds: <45m green, 45-75m yellow, >75m red
- Alert po przekroczeniu limitu (domyślnie 75m, konfigurowalne)
- Reset: ≥5 min away from computer
- Pokazywane w KPI strip jako "Screen time" (nie "Session" — mylące)

---

## Activity Tracking

- **Activity status in UI** — show keyboard/mouse activity status (active/idle) in floating window
  - `activity.rs` already detects idle ≥60s via `GetLastInputInfo`, but UI doesn't expose this
  - Show: "Active" / "Idle 2m" in widget footer or status bar
  - Useful for debugging Walking/Away state transitions

- **Cross-platform activity detection** — current implementation is Windows-only (`GetLastInputInfo`). Before release, need Linux/macOS support. Options: `rdev` crate (cross-platform input hooks), X11/Wayland idle APIs, macOS `CGEventSource`. Non-Windows currently returns `idle=0` (always active) — Away state will never trigger on Linux/macOS.

---

## Notification Centralization (pre-requisite for alert escalation)

> Before implementing full alert escalation flow (T017 Stages 3-5), centralize all notification sources.

- **NotificationService** — single Rust module that routes ALL notifications through one backend
  - Currently two parallel systems: native toasts (`tauri-plugin-notification`) scattered across `serial_periodic.rs` + custom WinAPI popup (`alert_popup_window.rs`)
  - 6 toast conditions spread across `serial_periodic.rs` and `session_breaks.rs` with no central coordination
  - `notification_backend` field exists in `config.rs` (`"toast" | "popup" | "both"`) but is **dead code** — never read
  - **Goal:** one `NotificationService` that:
    1. Collects all notification intents (inactivity, posture balance, praise, alerts, escalation)
    2. Routes through selected backend: native toast OR custom popup OR both
    3. Manages once-per-day / once-per-session gates centrally (not scattered flags)
    4. Makes T018 (A/B testing) trivial — just flip `notification_backend` in config
  - **Blocks:** T017 (Stages 3-5), T018 (A/B testing)
  - **Custom popup redesign** — migrate from raw WinAPI GDI (Win32 2003 look) to Tauri WebviewWindow (HTML/CSS, dark theme, acrylic blur). Already in backlog above under "Alert System — Future".

---

## Away Detection — Runtime Diagnostic (2026-03-25)

> State machine logic FIXED (commit c305f08): `!active → Away` regardless of desk height.
> Unit tests pass (340/340), including Standing→Away regression test.
> BUT: user reports Standing + inactive does NOT transition to Away at runtime.

**Status:** Diagnostic logging added to `serial_periodic.rs`. Next occurrence, check:
- Event log: `STATE Standing→Away idle=XXs` — if missing, `is_active()` never returned false
- Debug log: `Standing idle diagnostic: idle=XXs` — shows Windows API idle value

**Open questions:**
1. Does something on Windows reset `GetLastInputInfo` when overlay bar is in standing (gold) mode?
2. Is there a race condition where UI snapshot reads old state before transition?
3. Could the floating window (Tauri webview) or overlay (WinAPI) generate synthetic input?

**Next steps:**
- Run app with `RUST_LOG=desk_lib=debug`, reproduce, check logs
- If `is_active()` always returns true during standing: investigate WinAPI interactions
- Add integration test: `inject_reading(1200, false)` from frontend → verify UI shows Away

---

## DB Persistence in Dev Mode (2026-03-25)

> Database keeps getting reset during development. User needs persistent data even in dev mode.

**Current behavior:**
- DB path: `{AppData}/io.zntl.desk/desk.db`
- Lazy-initialized via `ensure_initialized()` — only opens when first IPC command fires
- `load_today_totals()` overwrites in-memory counters with DB values on init
- In dev mode (`pnpm tauri:dev`), rebuilds may change identifier → different `app_data_dir` → lost DB

**Proposed fixes:**
1. **Eager DB init** — open DB in `setup()`, not lazy on first IPC call. Prevents lost sessions before frontend loads.
2. **Log DB path at startup** — print `info!("DB: {}", db_path)` so user can verify path stays consistent.
3. **DB backup on startup** — copy `desk.db` → `desk.db.bak` before opening, protect against corruption.
4. **Pin `identifier` in dev mode** — ensure `tauri.conf.json` identifier doesn't change between builds.

**Priority:** HIGH — data loss during development is unacceptable when testing break patterns

---

## Logging & Observability

- **SQLite time-series storage** — replace file-per-minute snapshots with queryable SQLite table. Enables: search across days, trend analysis (sitting % over weeks), anomaly detection (daily score dropping), dashboard visualization. Natural phase 2 after T044 file-based logging proves useful. Effort: M, Priority: P3.

---

## Timeline Full Window — Expandable History View

- Click on timeline in popup → opens a dedicated window with full-day/multi-day timeline
- Scroll through days, zoom in/out, click sessions for details
- Shows: all states with durations, breaks, position changes, daily score
- Currently popup is ephemeral (opens/closes) — history needs a persistent view
- Implementation: Tauri WebviewWindow (like welcome popup), separate route
- Priority: P3 — needs SQLite history infrastructure first

---

## KPI Time Range Selector

- Switch to view stats for: Today / 7 days / 30 days
- Requires: SQLite time-series storage (see Logging & Observability below)
- Currently all KPIs are today-only (in-memory, reset at midnight)
- For 7d/30d: need historical daily summaries in DB
- UI: small toggle/tabs above KPI strip ("Today | 7d | 30d")
- Priority: P3 — needs DB history infrastructure first

---

## Remote Display — Phone as Desk Dashboard

**Epic E009** — full spec at `.plan/epics/E009-2026-03-24-remote-display/PLAN.md`

Phase 1 (E009): Web kiosk — PC serves React+WS to phone browser. 7 tasks, ~16h.
Phase 2 (future): Tauri Mobile native Android app.
Phase 3 (future): Standalone — sensor communicates wirelessly (BLE/WiFi) with phone, no PC needed.

See `.plan/vision/2026-03-15-desk-app-vision.md` → "Remote Display" section for full vision.

---

## Motivation Analytics & Adaptive Coaching (ongoing process)

- **Progressive break credit curve** — replace step function (<5m=0, 5-9m=-20m, ≥10m=reset) with smooth curve where every minute of break gives increasing credit. Short breaks (1-4 min) should give *some* reward. See memory: `project_progressive_break_credit.md`.
- **`/ergo-review` skill** — agent reads minute snapshots + event log, analyzes sitting/standing patterns, discusses UX effectiveness with user, proposes parameter tweaks. Created as `.claude/skills/ergo-review/`.
- **Notification outcome tracking** — log whether a notification led to action within 5 min (standing/away). Currently we fire notifications but don't track if they worked. Needed for measuring motivation effectiveness.
- **Configurable break credit parameters** — move hardcoded `BREAK_SHORT_SECS`, `BREAK_LONG_SECS`, `SHORT_BREAK_CREDIT_SECS` to `AppConfig` so they can be tuned without code changes.
- **Adaptive motivation engine (long-term)** — A/B test different notification strategies, learn what works for this user, optimize automatically. Needs: notification outcomes, sufficient history, parameter framework.

---

## Fullscreen Debug Dashboard (2026-03-26)

Dedicated fullscreen window for debugging and verifying app behavior. NOT the popup — a separate Tauri WebviewWindow.

### Must show:
1. **Timeline visualization** — full-day timeline bar (like popup but bigger), color-coded by state (green=standing, red=sitting, gray=away, gold=break credit applied)
2. **Event log overlay** — event log entries mapped to timeline positions. Each STATE/CREDIT/ALERT/NOTIF event visible as markers on the timeline
3. **Counter dashboard** — all live counters with their data source explained:
   - `sitting_seconds` — "Total sitting today (committed + live elapsed)"
   - `standing_seconds` — "Total standing today (committed + live bout)"
   - `break_seconds` — "Current break duration (from break_started)"
   - `position_changes` — "Sit↔Stand transitions only"
   - `daily_score` — "Points formula: -0.5/min sit, +1.0/min stand, +5.0/lap"
   - `continuous_computer_secs` — "Time at keyboard without 5min break"
4. **Raw state dump** — all SessionState fields, updated live (1s polling)
5. **DB session history** — list of CompletedSession rows from SQLite for today, with started_at, ended_at, state, duration_secs

### Purpose:
When user sees timeline not matching reality, they can open this view and immediately compare: "timeline shows X, but event log says Y, and DB has Z." No more guessing.

### Implementation:
- New Tauri WebviewWindow (like welcome popup pattern)
- Route: `/#/debug-dashboard`
- Read-only — no mutations
- IPC: `get_dashboard_state` (existing) + new `get_event_log` + `get_db_sessions_today`

---

## Future Features

- **Notification A/B testing** — two backends simultaneously with feature flag (T018)
- **Success notifications + gamification** — streak tracking, milestone celebrations (T019)
- **Notification strategy plugins** — like widget system but for how/when to nudge
- **Phone-as-hub** — old phone + BLE sensor, works without desktop app → **see E009 Phase 3**
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
