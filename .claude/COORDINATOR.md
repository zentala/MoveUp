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

### Kluczowa decyzja architektoniczna (WAŻNE)
**Rust is single source of truth.**
- `SessionManager` (Rust) posiada wszystkie countery sesji
- Na starcie seeduje dane z SQLite
- Frontend NIE robi zapytań SQL bezpośrednio (usuń `getTodaySummary()` z `db.ts`)
- Wszystkie dane przychodzą przez Tauri commands lub eventy

### Pliki kluczowe
- `apps/desk/.claude/tasks/` — szczegółowe specyfikacje każdego taska
- `apps/desk/.interface-design/system.md` — design system (tokeny, komponenty)
- `apps/desk/TASKS.md` — board z priorytetami
- `apps/desk/CLAUDE.md` — konwencje projektu

---

## Fale implementacji

### Wave 1 — Fundament (1 agent, sekwencyjnie)

**Musisz zrobić to sam zanim uruchomisz Wave 2.**

Stwórz worktree i uruchom agenta dla T002:

```bash
cd /c/code/zntl-tray
git branch feat/T002-config-store main
git worktree add .claude/worktrees/T002-config-store feat/T002-config-store
```

Agent T002 ma zaimplementować:
- Plik: `.claude/tasks/0002-settings-store.md` (pełna spec)
- Nowy `src-tauri/src/config.rs` z `AppConfig` + serde defaults + `clamped()`
- `SessionManager::new_from_config()` + `load_today_totals()`
- `get_settings()` + `save_settings()` + prawdziwy `get_today_summary()` w `commands.rs`
- Usuń placeholder `standing_secs: 0` i `getTodaySummary()` z frontendu (TypeScript)
- `DEFAULT_SESSION_LIMIT_SECS` → 2700 (45 min)
- Fix test setup: zainstaluj brakujące `@testing-library/jest-dom` i dodaj type stubs dla `@tauri-apps/plugin-sql`

Po zakończeniu Wave 1: merge do main, cleanup worktree.

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
- Dodaj `standing_seconds: i64` do `SessionState` + `SessionStateDto`
- Obydwa nie kolidują: 0009 dodaje pola reset, 0006 dodaje standing counter

#### Agent B — T001 (Settings Panel UI — czysty frontend)
```bash
git branch feat/T001-settings-panel main
git worktree add .claude/worktrees/T001-settings-panel feat/T001-settings-panel
```
- Spec: `.claude/tasks/0001-settings-panel.md`
- Usuń `CalibrationWizard.tsx`, stwórz `SettingsPanel.tsx`
- Design system: `.interface-design/system.md` (użyj tokenów `--beam`, `--panel-*` itd.)
- Na first run (brak stored config): otwórz Settings automatycznie
- `invoke("get_settings")` / `invoke("save_settings")` — NIE implementuj backendu, to już jest po Wave 1

**Agent A i B nie mają konfliktu plików** — A w `src-tauri/src/`, B w `src/components/`.

Po zakończeniu Wave 2: merge oba do main. Jeśli merge conflict (mało prawdopodobne) — rozwiąż ręcznie.

---

### Wave 3 — Równolegle (2 agenci)

#### Agent C — T003 + T004 (powiadomienia + stand limit — session.rs)
```bash
git branch feat/T003-T004-notifications main
git worktree add .claude/worktrees/T003-T004 feat/T003-T004-notifications
```
- Spec: `.claude/tasks/0003-notification-prefs.md` + `.claude/tasks/0004-stand-limit.md`
- Dodaj `last_position_change_at`, `check_notification_conditions()` do `session.rs`
- Dodaj `stand_limit_secs`, `stand_alert_fired` do `SessionManager`
- 3 typy powiadomień: `notify_inactivity`, `notify_daily_posture_balance`, `notify_praise_halfway`

#### Agent D — T005 (position changes counter — session.rs + frontend)
```bash
git branch feat/T005-position-changes main
git worktree add .claude/worktrees/T005-position-changes feat/T005-position-changes
```
- Spec: `.claude/tasks/0005-position-changes.md`
- Dodaj `position_changes: u32` do `SessionState` + `SessionStateDto`
- Inkrementuj przy potwierdzonych przejściach (po debounce)
- Dodaj do `TodayStats.tsx` jako trzecia statystyka

**UWAGA**: C i D oboje modyfikują `session.rs`. Użyj worktrees — merge będzie miał konflikt w `session.rs`, ale jest przewidywalny (różne pola/metody). Rozwiąż przez `git merge` z ręcznym merge conflict resolution.

---

### Wave 4 — Równolegle (3 agenci — czyste zmiany, brak konfliktów)

#### Agent E — T010 (Dynamic tray icon)
```bash
git branch feat/T010-tray-icon main
git worktree add .claude/worktrees/T010-tray-icon feat/T010-tray-icon
```
- Spec: `.claude/tasks/0010-dynamic-tray-icon.md`
- Pliki: `src-tauri/src/tray_controller.rs`, nowe PNG w `src-tauri/icons/tray/`

#### Agent F — T011 (Rail pulse animation)
```bash
git branch feat/T011-rail-pulse main
git worktree add .claude/worktrees/T011-rail-pulse feat/T011-rail-pulse
```
- Spec: `.claude/tasks/0011-rail-pulse.md`
- Pliki: `src/styles/globals.css` (keyframe), `src/components/HeightRail.tsx`

#### Agent G — T012 (Yesterday delta)
```bash
git branch feat/T012-yesterday-delta main
git worktree add .claude/worktrees/T012-yesterday-delta feat/T012-yesterday-delta
```
- Spec: `.claude/tasks/0012-yesterday-delta.md`
- Pliki: `src-tauri/src/db.rs`, `src/components/TodayStats.tsx`

E, F, G dotykają całkowicie różnych plików — merge bez konfliktów.

---

## Szablon komunikatu dla każdego agenta

Każdy agent powinien dostać komunikat w tym formacie:

```
Jesteś agentem implementującym [TASK_ID] w projekcie zntl-tray/apps/desk.

Pracujesz WYŁĄCZNIE w katalogu: /c/code/zntl-tray/.claude/worktrees/[WORKTREE_NAME]/

Przeczytaj najpierw:
1. apps/desk/.claude/tasks/[TASK_FILE].md — pełna spec
2. apps/desk/CLAUDE.md — konwencje projektu
3. apps/desk/.interface-design/system.md — design system (jeśli frontend)
4. Pliki które będziesz modyfikować (Read first!)

Kluczowa decyzja architektoniczna:
- Rust is single source of truth (Rust posiada countery, frontend czyta przez commands)
- Nie dodawaj bezpośrednich SQL queries w TypeScript
- Używaj tokenów CSS z design system zamiast hardkodowanych wartości

Po zakończeniu:
- Uruchom testy: `pnpm test` (frontend) lub `cargo test` (Rust)
- Zrób commit: `feat(desk): [opis]` lub `fix(desk): [opis]`
- NIE merguj sam — koordynator zrobi merge po zakończeniu fali
```

---

## Merge flow po każdej fali

```bash
# Po zakończeniu fali:
cd /c/code/zntl-tray
git checkout main

# Merge każdego worktree:
git merge feat/T002-config-store --no-ff -m "feat(desk): T002 config store + calibration persistence"

# Cleanup:
git worktree remove .claude/worktrees/T002-config-store
git branch -d feat/T002-config-store
```

---

## Checklist przed uruchomieniem Wave 1

- [ ] Jesteś na branchu `main` i jest on aktualny
- [ ] `pnpm install` w `apps/desk/` jest zrobione
- [ ] `cargo check` w `apps/desk/src-tauri/` przechodzi bez błędów
- [ ] Przeczytałeś `.claude/tasks/0002-settings-store.md` w całości

---

## Znane problemy do naprawienia w Wave 1 (przy okazji)

1. **Test setup broken** — `@testing-library/jest-dom` nie zainstalowane, brak type stubs dla `@tauri-apps/plugin-sql`. Agent T002 naprawia to jako część swojej pracy.
2. **`HeightReadingRow` dead code** — zaznaczone TODO w `db.rs:44` — można zostawić lub usunąć przy okazji T002.
3. **Duplicate `getTodaySummary` in TypeScript** — usuń `getTodaySummary()` i `saveSittingSession()` z `src/db.ts` po tym jak T002 wdroży backend version.
