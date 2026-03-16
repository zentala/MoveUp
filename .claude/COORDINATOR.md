# Desk App — Agent Coordinator Kickoff

Jesteś agentem koordynującym implementację funkcji dla aplikacji `apps/desk` —
ergonomicznego trackera biurka sit/stand w monorepo `zntl-tray`.

## Twoja rola

Nie implementujesz kodu bezpośrednio. Twoja rola to:
1. Stworzyć worktree dla każdego agenta w danej fali
2. Uruchomić agentów równolegle (w jednej wiadomości)
3. Poczekać na ich zakończenie
4. Zmergować worktree do main
5. Uruchomić następną falę

---

## Kontekst projektu

**Repo root**: `C:/code/zntl-tray`
**Główna gałąź**: `main`
**App dir**: `apps/desk/`
**Stack**: Tauri 2 + React + TypeScript (frontend) + Rust (backend)
**Package manager**: pnpm

### Kluczowe decyzje architektoniczne (WAŻNE — przeczytaj przed każdą falą)

**1. Rust is single source of truth.**
- `SessionManager` (Rust) posiada wszystkie countery sesji
- Na starcie seeduje dane z SQLite (via `rusqlite` — patrz niżej)
- Frontend NIE robi zapytań SQL bezpośrednio
- Wszystkie dane przychodzą przez Tauri commands lub eventy

**2. `rusqlite` zamiast `tauri-plugin-sql`.**
- `tauri-plugin-sql` exposes SQLite tylko do JavaScript — Rust nie ma do niego dostępu
- T002 zastępuje go `rusqlite` crate bezpośrednio w Rust
- `src/db.ts` (TypeScript) jest **usuwany całkowicie** — przez T002
- `tauri-plugin-sql` jest **usuwany** z `Cargo.toml` i `lib.rs`
- Schema init, session INSERT, today summary SELECT — wszystko w Rust

**3. `on_reading()` zwraca `ReadingResult` struct (nie tuple).**
```rust
pub struct ReadingResult {
    pub state_change: Option<StateChangedPayload>,
    pub completed_session: Option<CompletedSession>,
}
```
- `serial.rs` sprawdza `result.completed_session` i robi INSERT do SQLite
- Istniejące testy używają `.state_change` zamiast bezpośredniego zwracanego value

**4. Config commands w `config.rs`, nie w `commands.rs`.**
- `get_settings()` i `save_settings()` żyją w `src-tauri/src/config.rs`
- Import przez `commands::config::*` w `invoke_handler!`
- Zapobiega przekroczeniu limitu 250 linii w `commands.rs`

**5. `desk:db-error` event przy błędzie SQLite.**
- Jeśli `insert_session()` lub inne DB operacje failują → `log::error!` + emit `desk:db-error`
- `useDesk.ts` subskrybuje ten event i pokazuje error banner

### Pliki kluczowe
- `apps/desk/.claude/tasks/` — szczegółowe specyfikacje każdego taska
- `apps/desk/.interface-design/system.md` — design system (tokeny, komponenty)
- `apps/desk/TASKS.md` — board z priorytetami
- `apps/desk/CLAUDE.md` — konwencje projektu

---

## Fale implementacji

### Wave 1 — Fundament (1 agent, sekwencyjnie)

**Musisz zrobić to sam zanim uruchomisz Wave 2.**

```bash
cd /c/code/zntl-tray
git branch feat/T002-config-store main
git worktree add .claude/worktrees/T002-config-store feat/T002-config-store
```

Agent T002 implementuje (spec: `.claude/tasks/0002-settings-store.md`):

**Krok 0 — Fix test setup (PIERWSZE):**
- `pnpm add -D @testing-library/jest-dom` w `apps/desk/`
- Stwórz `src/test/setup.ts` z `import '@testing-library/jest-dom'`
- Dodaj `setupFiles: ['./src/test/setup.ts']` do `vite.config.ts`
- Usuń `@tauri-apps/plugin-sql` type dependency (bo plugin jest usuwany)

**Krok 1 — Rusqlite + schema:**
- Dodaj `rusqlite = { version = "0.31", features = ["bundled"] }` do `Cargo.toml`
- Usuń `tauri-plugin-sql` z `Cargo.toml` i `lib.rs`
- Przepisz `src-tauri/src/db.rs`:
  - `init_schema(conn: &Connection)` — tworzy tabele IF NOT EXISTS
  - `insert_session(conn, row)` — INSERT z `log::error!` + emit `desk:db-error` na fail
  - `load_today_totals(conn)` → `(sitting_secs: i64, standing_secs: i64)`
  - `get_today_summary_sql(conn)` → `TodaySummary`
  - Testy in-memory: `test_schema_init_idempotent`, `test_insert_and_query_today`, `test_load_today_totals_empty`

**Krok 2 — SQLite file location:**
```rust
// W lib.rs setup():
let db_path = app.path().app_data_dir()?.join("desk.db");
let db_conn = rusqlite::Connection::open(&db_path)?;
db::init_schema(&db_conn)?;
```

**Krok 3 — `on_reading()` → `ReadingResult`:**
- Zmień typ zwracany z `Option<StateChangedPayload>` na `ReadingResult`
- Dodaj `CompletedSession { started_at, ended_at, duration_secs }` struct
- Na każdym Sitting→non-Sitting przejściu: wypełnij `completed_session`
- Zaktualizuj wszystkie istniejące testy: `.state_change` zamiast direct value

**Krok 4 — `config.rs`:**
- `AppConfig` struct z `#[serde(default)]` na wszystkich polach
- `AppConfig::clamped()` — clamp + inverted calibration check
- `AppConfig::load(store)` + `AppConfig::save(store)`
- Publiczne `get_settings()` i `save_settings()` komendy
- Testy: serde round-trip, clamping, inverted calibration reset

**Krok 5 — `session.rs`:**
- `SessionManager::new_from_config(config: &AppConfig)`
- `SessionManager::load_today_totals(sitting_secs, standing_secs)`
- `DEFAULT_SESSION_LIMIT_SECS` → 2700 (45 min)

**Krok 6 — `lib.rs` / startup:**
- Load config → create SessionManager from config → open SQLite → load today totals
- `AppState` dostaje dwa nowe pola: `db: Arc<Mutex<Connection>>`, `config: Arc<Mutex<AppConfig>>`
- `serial.rs` dostaje dostęp do `db` żeby robić INSERT po `on_reading()`

**Krok 7 — Usuń TypeScript DB:**
- Usuń `src/db.ts` (cały plik)
- Usuń import i call `saveSittingSession()` z `useDesk.ts` (~20 linii)
- Usuń `sittingStartedAt` i `prevSittingSeconds` refs z `useDesk.ts`
- Dodaj `desk:db-error` listener do `useDesk.ts` → setError()
- `TodayStats.tsx`: zastąp `getTodaySummary()` przez `invoke("get_today_summary")`

Po zakończeniu Wave 1: `cargo test` + `pnpm test` muszą przechodzić. Merge do main.

---

### Wave 2 — Równolegle (2 agenci jednocześnie)

Uruchom **oba agenty w jednej wiadomości** po zmergowaniu Wave 1.

#### Agent A — T009 + T006 (session.rs)
```bash
git branch feat/T009-T006-session-fixes main
git worktree add .claude/worktrees/T009-T006 feat/T009-T006-session-fixes
```
- Spec: `.claude/tasks/0009-daily-reset.md` + `.claude/tasks/0006-standing-secs.md`
- Dodaj `last_reset_date: NaiveDate` + `check_daily_reset()` do `SessionManager`
  - Sprawdzaj tylko co 60s (dodaj `last_reset_check: DateTime<Utc>` field)
  - Resetuj: `sitting_seconds`, `standing_seconds`, `position_changes` (T005 doda to pole), `alert_fired`, `stand_alert_fired`, notification flags
- Dodaj `standing_seconds: i64` do `SessionState` + `SessionStateDto`
  - Inkrementuj tylko gdy `state == Standing` (nie Walking, nie Away)
- Oba nie kolidują: T009 dodaje pola reset, T006 dodaje standing counter

#### Agent B — T001 (Settings Panel UI — czysty frontend)
```bash
git branch feat/T001-settings-panel main
git worktree add .claude/worktrees/T001-settings-panel feat/T001-settings-panel
```
- Spec: `.claude/tasks/0001-settings-panel.md`
- Usuń `CalibrationWizard.tsx`, stwórz `SettingsPanel.tsx`
- Sekcje: Time Limits (sit/stand sliders step=5) + Calibration (mm inputs z walidacją) + Notifications (3 toggles)
- Design system: `.interface-design/system.md` — tokeny `--beam`, `--panel-*`, `--ink-*`
- Na first run (brak stored config): `showSettings = true` domyślnie w App.tsx
- `invoke("get_settings")` / `invoke("save_settings")` — backend już działa po Wave 1
- Testy Vitest: Save/Back behavior, inverted calibration blocks Save, slider defaults

**Agent A i B nie mają konfliktu plików** — A w `src-tauri/src/`, B w `src/components/`.

---

### Wave 3 — Równolegle (2 agenci) — UWAGA: merge conflict w session.rs

**PRZED mergem Wave 3:** oba agenty modyfikują `session.rs`. Merge resolution:
- Agent C dodaje: `last_position_change_at`, `stand_limit_secs`, `stand_alert_fired`, notification flags + `check_notification_conditions()`
- Agent D dodaje: `position_changes: u32` + inkrementację w `on_reading()` przy przejściu
- Oba zestawy zmian są **wyłącznie addytywne, różne nazwy pól** — weź oba przy konflikcie

#### Agent C — T003 + T004 (powiadomienia + stand limit)
```bash
git branch feat/T003-T004-notifications main
git worktree add .claude/worktrees/T003-T004 feat/T003-T004-notifications
```
- Spec: `.claude/tasks/0003-notification-prefs.md` + `.claude/tasks/0004-stand-limit.md`
- Powiadomienia: `notify_inactivity` (90 min bez zmiany), `notify_daily_posture_balance`, `notify_praise_halfway`
- Każde max raz na godzinę (debounce flag), reset na `check_daily_reset()`
- Stand limit: `stand_limit_secs`, `stand_alert_fired` — identyczny pattern jak `alert_fired`
- `check_notification_conditions()` wywoływane co 60s w serial reader lub osobnym timerze

#### Agent D — T005 (position changes counter)
```bash
git branch feat/T005-position-changes main
git worktree add .claude/worktrees/T005-position-changes feat/T005-position-changes
```
- Spec: `.claude/tasks/0005-position-changes.md`
- `position_changes: u32` w `SessionState` + `SessionStateDto` + `ReadingResult` (expose w StateChangedPayload)
- Inkrementuj w `on_reading()` na potwierdzonym przejściu (po debounce, tylko Sitting↔Standing)
- NIE inkrementuj na Standing→Walking (to nie jest zmiana pozycji)
- `TodayStats.tsx`: trzecia statystyka "changes"

---

### Wave 4 — Równolegle (3 agenci — czyste zmiany, brak konfliktów)

#### Agent E — T010 (Dynamic tray icon)
```bash
git branch feat/T010-tray-icon main
git worktree add .claude/worktrees/T010-tray-icon feat/T010-tray-icon
```
- Spec: `.claude/tasks/0010-dynamic-tray-icon.md`
- Pliki: `src-tauri/src/tray_controller.rs`, nowe PNG 32×32 w `src-tauri/icons/tray/`
- `update_tray_icon(app, ratio, state)` — ratio od SessionStateDto

#### Agent F — T011 (Rail pulse animation)
```bash
git branch feat/T011-rail-pulse main
git worktree add .claude/worktrees/T011-rail-pulse feat/T011-rail-pulse
```
- Spec: `.claude/tasks/0011-rail-pulse.md`
- Pliki: `src/styles/globals.css` (keyframe `rail-pulse`), `src/components/HeightRail.tsx`
- Prop `state` do HeightRail, useEffect wykrywa Sitting→Standing, dodaje klasę na 600ms

#### Agent G — T012 (Yesterday delta)
```bash
git branch feat/T012-yesterday-delta main
git worktree add .claude/worktrees/T012-yesterday-delta feat/T012-yesterday-delta
```
- Spec: `.claude/tasks/0012-yesterday-delta.md`
- Pliki: `src-tauri/src/db.rs` (dodaj yesterday query), `src/components/TodayStats.tsx`
- Próg delta: ±300s (5 min) przed pokazaniem strzałki

E, F, G dotykają całkowicie różnych plików — merge bez konfliktów.

---

## Szablon komunikatu dla każdego agenta

```
Jesteś agentem implementującym [TASK_ID] w projekcie zntl-tray/apps/desk.

Pracujesz WYŁĄCZNIE w katalogu: /c/code/zntl-tray/.claude/worktrees/[WORKTREE_NAME]/
Ścieżka do worktree to TWÓJ katalog roboczy — używaj ścieżek absolutnych od tego katalogu.

Przeczytaj najpierw (ZANIM napiszesz linię kodu):
1. apps/desk/.claude/tasks/[TASK_FILE].md — pełna spec taska
2. apps/desk/CLAUDE.md — konwencje projektu (linting, format, commit style)
3. apps/desk/.interface-design/system.md — design system (jeśli robisz frontend)
4. Każdy plik który będziesz modyfikować — Read first, zawsze

Kluczowe decyzje architektoniczne:
- Rust is single source of truth (Rust owns DB + counters, frontend reads via commands)
- rusqlite w Rust, NIE tauri-plugin-sql (który jest tylko dla JS)
- on_reading() zwraca ReadingResult { state_change, completed_session }
- Nie rób bezpośrednich SQL queries w TypeScript
- Używaj tokenów CSS z design system (--beam, --panel-*, --ink-*) nie hardkoduj kolorów
- config commands (get_settings, save_settings) żyją w config.rs nie commands.rs

Po zakończeniu:
- cargo test (Rust) — musi przechodzić
- pnpm test (frontend) — musi przechodzić
- pnpm run typecheck — zero błędów w nowych plikach
- Zrób commit: feat(desk): [opis]
- NIE merguj sam — koordynator robi merge po zakończeniu całej fali
```

---

## Merge flow po każdej fali

```bash
cd /c/code/zntl-tray
git checkout main

# Merge:
git merge feat/T002-config-store --no-ff -m "feat(desk): T002 config store + rusqlite + calibration persistence"

# Cleanup:
git worktree remove .claude/worktrees/T002-config-store
git branch -d feat/T002-config-store
```

### Wave 3 merge conflict resolution guide

Oba agenty (C i D) modyfikują `session.rs`. Gdy `git merge` zgłosi conflict:
```
<<<<<<< feat/T003-T004-notifications
    last_position_change_at: Option<DateTime<Utc>>,
    stand_limit_secs: i64,
    stand_alert_fired: bool,
=======
    position_changes: u32,
>>>>>>> feat/T005-position-changes
```
Rozwiązanie: **weź oba zestawy pól**:
```rust
    last_position_change_at: Option<DateTime<Utc>>,
    stand_limit_secs: i64,
    stand_alert_fired: bool,
    position_changes: u32,
```
Nie ma logicznego konfliktu — to wyłącznie addytywne zmiany różnych pól.

---

## Checklist przed uruchomieniem Wave 1

- [ ] Jesteś na branchu `main` (`git status` = clean)
- [ ] `pnpm install` w `apps/desk/` jest zrobione
- [ ] `cargo check` w `apps/desk/src-tauri/` przechodzi
- [ ] Przeczytałeś `.claude/tasks/0002-settings-store.md` w całości
- [ ] Rozumiesz że `tauri-plugin-sql` jest **usuwany** i zastępowany `rusqlite`
