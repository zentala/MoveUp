# MoveUp — pełny przegląd: silnik, architektura, pre-release, plan

Data: 2026-09-06. Autor: agent (sesja planująca, nic nie wdrażano).
Źródła: pięć niezależnych przeglądów w [`_review-2026-09-06/`](_review-2026-09-06/)
(silnik, backend Rust, frontend, release, plany i historia). Każde znalezisko
tam niesie `path:line`. Ten dokument jest syntezą i rekomendacją.

## TLDR

1. **Twoja obserwacja jest prawdziwa i to jest błąd okablowania, nie brak
   funkcji.** Kredyt proporcjonalny (ADR 008) działa w silniku i widać go w
   Twoich logach (`CREDIT Partial dur=199s sitting=366`, nie zero). Ale duży
   timer i pasek w popupie czytają **inne pole** — `current_session_secs` —
   które z założenia zeruje się przy każdym powrocie do siedzenia
   (`session_types.rs:112-114`, `useDesk.ts:106`, `useRemoteDesk.ts:74`).
   Overlay i powiadomienia czytają pole właściwe. Naprawa: 3 punkty.
2. **Twój współczynnik 3:1 (45/15) to tylko konfiguracja.** Dziś domyślny
   mnożnik to 2.0 (`ergonomic_profile.rs:16`). Decyzja do podjęcia, patrz D1.
3. **Struktura jest lepsza niż zakładasz, ale testy nie są dowodem.** Moduły
   trzymają limit 250 linii, IPC jest cienkie i spójnie nazwane. 740 testów
   przechodzi, ale co najmniej jeden asertuje błąd jako poprawne zachowanie,
   główny hook danych ma 0,8 % pokrycia, a ścieżka „siadam → wstaję → siadam
   → co widzi UI" nie ma testu wcale. Zielony przebieg mówi, że kod robi to,
   co autor testu myślał, nie że robi to, co Ty chcesz. Prawdziwe problemy to
   **szwy**: dwa liczniki siedzenia bez właściciela (przepisywane 9 razy
   w 15 dni w marcu), cztery mechanizmy persystencji bez pierwszeństwa, dwa
   zduplikowane hooki stanu w TS, ręcznie kopiowane typy Rust→TS, dokumentacja
   opisująca inny produkt niż kod.
4. **Nie przepisuj, tylko wymieniaj kawałek po kawałku (strangler).**
   Sześć epików po kolei, razem 179 punktów, każdy przechodzi `ao ready`
   i ma pełne taski. Pierwsze 29 punktów (E016 + E015) załatwiają Twoją
   skargę i przywracają wiarygodność `.plan/`. Release publiczny blokują
   dokumenty, licencja, CI i podpis, nie kod.
5. **Cztery pliki udają backlog** (`BACKLOG.md`, `.plan/BACKLOG.md`,
   `TASKS.md`, `ORCHESTRATOR.md`), a `STATE.md` kłamie o wersji i epikach.
   To pierwsza rzecz do sprzątnięcia, bo bez tego żaden plan nie jest
   wiarygodny.

## 1. Jak aplikacja działa dziś (kontekst)

Czujnik ToF pod blatem wysyła odległość po USB (`serial.rs`). Wysokość
biurka + aktywność klawiatury dają stan `Sitting/Standing/Walking/Away`
z debounce 5 s (`session_reading.rs`). Silnik sesji (`session*.rs`) liczy
dwa liczniki siedzenia:

| Pole | Co liczy | Kto czyta |
|---|---|---|
| `sitting_seconds` | siedzenie **pomniejszone o kredyt przerw** (limit 40 min) | overlay, tray, `CommunicationPolicy` (powiadomienia), kolor widgetu |
| `current_session_secs` | sekundy od ostatniego wejścia w Sitting, **zawsze od zera** | duży timer i pasek w popupie, timeline, Debug |
| `sitting_seconds_total` | surowe siedzenie dzisiaj (KPI) | Analyst, PostureBalance |

Kredyt przerwy (`session_breaks.rs:126-158`): po powrocie do siedzenia
`sitting_seconds = max(0, sitting − przerwa × 2.0)`, przerwy < 60 s bez
kredytu. Przerwa ≥ 6 h dodatkowo zeruje flagi powiadomień (ADR 009).
`CommunicationPolicy` zamienia `(stan, czas)` na sygnały dla tray/overlay/
toastów wg profilu komunikacji (JSON, hot-reload). Stan przeżywa restart
przez `tauri-plugin-store`; sesje idą do SQLite; co minutę snapshot JSON;
zdarzenia do `events.log`.

## 2. Odpowiedź na pytanie o licznik

Śledzenie kroku „wstaję na 2 min i siadam":

1. Standing wchodzi, `sitting_seconds` stoi (`session_breaks.rs`).
2. Sitting wraca: `apply_break_credit` odejmuje `120 × 2.0 = 240 s` od
   `sitting_seconds` **i jednocześnie** `current_session_secs = 0`
   (`session_breaks.rs:34,63,85`).
3. `useDesk.ts:106` robi `setSittingSeconds(dto.current_session_secs)` —
   nazwa zmiennej w TS mówi „sittingSeconds", zawartość to licznik od zera.
4. `OneBarTimer.tsx:28` pokazuje to jako czas sesji. Kolor tego samego
   widgetu (`temperature.ts:29-32`) bierze pole kredytowane. Liczba i kolor
   w jednej klatce mówią co innego.
5. Test `session_tests_timers_live.rs:160-164` **asertuje ten reset jako
   poprawny** wewnątrz scenariusza z częściowym kredytem — dlatego nikt tego
   nie złapał.

Dowód z Twoich logów (`io.zntl.desk/logs/*/events.log`): wpisy `CREDIT
Partial` z niezerowym `sitting=` po krótkich przerwach. Pełny ślad:
[`engine.md`](_review-2026-09-06/engine.md) §Trace i §Log evidence.

Dlaczego to wracało: historia w
[`plans-and-history.md`](_review-2026-09-06/plans-and-history.md) §2 pokazuje
9 przepisań logiki kredytu między 16 a 31 marca. Za każdym razem ten sam
szew: dwa liczniki, brak reguły „kto czyta który". Szew nigdy nie został
zamknięty, więc każda kolejna funkcja (timeline, Analyst, persystencja)
wybierała pole losowo.

## 3. Werdykt architektoniczny

### Silnik (ocena: model spójny w 70 %, reszta to szwy)

| Znalezisko | Gdzie | Ważność | Pkt |
|---|---|---|---|
| Popup czyta `current_session_secs` zamiast pola kredytowanego | `useDesk.ts:106,170`, `OneBarTimer.tsx:28,59-68` | High | 3 |
| Test utrwala błąd | `session_tests_timers_live.rs:160-164` | High | 2 |
| Cztery pojęcia „elapsed" bez jednego źródła | `session_types.rs:82-114`, `tray_controller.rs:124-128` | Med | 5 |
| Zegar czytany wewnątrz silnika (`Utc::now()`), nie wstrzykiwany | `session_reading.rs:12` | Med | 8 |
| Config profilu kopiowany pole po polu do stanu | `session_types.rs:139-150` | Med | 8 |
| Ręcznie lustrzany typ persystencji zamiast serializacji stanu | `session_persistence.rs:78-107` | Med | 5 |
| PostureBalance miesza licznik kredytowany z surowym | `.plan/BACKLOG.md:185-188` | Med | 3 |
| `break_credit` nie zapisany na `SessionRow`, Analyst zgaduje | `.plan/IMPROVEMENTS.md:17-20` | Med | 3 |
| UX-FLOW opisuje stary 3-progowy model kredytu | `.arch/UX-FLOW.md:342-346` | Med | 1 |

Docelowy kształt (z `engine.md` §Target): jeden czysty rdzeń
`ErgoEngine::step(event, &cfg, now) -> Vec<Signal>` z jednym typowanym
stanem, jednym snapshotem DTO z **jednym** polem „ile zużyłem limitu",
zegarem jako argumentem, configiem jako referencją, sygnałami zamiast wiedzy
o tray/overlay. Migracja strangler, 8 kroków, 52 punkty. Kroki 1-3 (13 pkt)
zamykają Twoją skargę.

### Backend Rust (ocena: solidny, kilka prawdziwych ryzyk)

- `commands.rs`: 14 gołych `.unwrap()` na mutexach; jeden panic zatruwa
  lock i kładzie każde IPC. High, 2 pkt.
- Cztery mechanizmy persystencji (SQLite, store, snapshoty, events.log), trzy
  ścieżki liczące „dzisiejsze sumy", brak zapisanego pierwszeństwa. High, 5.
- `TrayController` liczy logikę (PolicyInput, laps, tooltip, broadcast),
  choć CLAUDE.md mówi „tylko wykonuje sygnały". Med, 5.
- `EventLogger::new` panikuje przy błędzie tworzenia katalogu, siostrzany
  `SnapshotLogger` tylko ostrzega. Med, 1.
- Lista `generate_handler!` ręcznie zdublowana debug/release. Med, 1.
- 8 nazw zdarzeń `desk:*` jako literały rozsiane po Rust i TS. Med, 2.
- Rekomendacja backendu: **nie** rób osobnego crate'a „domena bez Tauri";
  testy dowodzą, że silnik i tak testuje się bez Tauri. Popraw dokumenty,
  nie wymuszaj struktury. Pełna tabela: [`backend.md`](_review-2026-09-06/backend.md).

### Frontend (ocena: jeden dobry wzorzec plus nieusunięta poprzednia generacja)

- `pnpm lint` woła nieistniejący `eslint` (brak zależności i configu). High, 2.
- Bramka pokrycia 80 % istnieje w configu, ale nic jej nie woła; realne
  pokrycie 76/68/66 %. High, 2.
- `useDesk.ts` i `useRemoteDesk.ts` to ta sama maszyna stanu dwa razy
  (~500 linii); `useDesk` ma 0,78 % pokrycia. High, 8.
- `types.ts`/`SettingsTypes.ts` ręcznie kopiują DTO z Rusta, bez codegen
  i bez testu dryfu. Med, 5.
- 5 martwych komponentów sprzed OneBar, wciąż testowanych; martwe wejście
  `overlay.html`. Med, 3.
- UX-FLOW nie zna Steps/Google Fit ani „klik w timeline → Analyst". Med, 2.
- Brak `justfile`. Low, 1. Pełna tabela: [`frontend.md`](_review-2026-09-06/frontend.md).

### Plany i historia

- `STATE.md`: mówi v0.3.0 i E011 w toku; realnie v0.5.0, E011 i E012
  zrobione. High, 1.
- E011 ma kod, brak ceremonii zamknięcia. High, 3.
- Cztery backlogi, sprzeczne liczby zadań ludzkich E010 (3 vs 10). Med, 2.
- E013 utknął w planowaniu, E014 w połowie zablokowany przez `pm3-mcp`.
- Jedyna niescalona gałąź `feat/E013-T01-release-baseline` sama się oznacza
  jako porzucona. Pełna tabela 25 pozycji: [`plans-and-history.md`](_review-2026-09-06/plans-and-history.md).

## 4. Pre-release

Kod buduje się czysto (`tsc`, `vite build`, `cargo check`), telemetria nic
nie wysyła, sekretów w repo nie ma. Blokują dokumenty i proces:

| Bloker | Ważność | Pkt |
|---|---|---|
| Docs użytkownika opisują „zntlDesk", stare ścieżki, stare repo | High | 3 |
| `USER_UPDATES.md` opisuje auto-updater, którego nie ma w kodzie | High | 2 |
| Brak tożsamości do podpisu kodu (E013 fala 2) | High | 8 |
| Brak pliku LICENSE mimo modelu open-core | High | 2 |
| Google Fit wysyła dane do Google, `PRIVACY.md` twierdzi, że nic nie wychodzi | High | 2 |
| `.github/workflows/test.yml` celuje w nieistniejące `apps/desk/` | High | 3 |
| Firmware to goły `.ino` bez instrukcji flashowania | High | 3 |
| Sekcja o kablach USB nie trafiła do docs wsparcia | Med | 2 |
| `coverage/` i `test-performance-report/` śledzone w gicie | Med | 1 |
| Cargo metadata binarki mówi „zntl Desk" | Med | 1 |

E013/E014 to wdrożenie na **Twojej** maszynie (PM3, rollback), nie ścieżka
release'u dla użytkownika. Nie mieszaj ich z wydaniem. Pełna checklista 12
kroków: [`release.md`](_review-2026-09-06/release.md).

## 5. Decyzje, które są Twoje (z rekomendacją)

- **D1 — mnożnik kredytu.** Rekomenduję 3.0 (Twoje 45/15) jako domyślny
  w profilu `standard`, 2.0 zostaje w `relaxed`. Koszt odrzucenia: zero,
  to jedna liczba. Aktualizacja ADR 008 i UX-FLOW w tym samym commicie.
- **D2 — czy „czas od ostatniego wstania" ma gdzieś zostać.** Rekomenduję
  usunąć `current_session_secs` całkiem; jeśli chcesz taką liczbę w Debug,
  dostaje uczciwą nazwę `secs_since_last_break` i nigdy nie zasila paska.
- **D3 — licencja.** Rekomenduję MIT (spójne z ADR 005 open-core, najmniej
  tarcia dla dev kitu). Alternatywa Apache-2.0, jeśli zależy Ci na klauzuli
  patentowej.
- **D4 — kolejność: naprawa silnika przed czy po release?** Rekomenduję
  przed. Wydanie, w którym główny timer kłamie, kosztuje wiarygodność
  pierwszych użytkowników bardziej niż dwa tygodnie opóźnienia.
- **D5 — przepisać czy strangler.** Strangler. Przepisanie od zera
  odtworzyłoby dokładnie ten błąd (drugie „elapsed" wymyślone w połowie
  migracji) bez siatki 740 testów.

## 6. Plan: epiki i kolejność

Wszystkie sześć epików ma `PLAN.md`, `HANDOFF.md` z blokiem `## AO`,
`PRES.md` (po polsku) i pliki zadań `tasks/E0NN-T0N-*.md`. Każdy przeszedł
konwersję na manifest i `ao ready` → `status: ready` (2026-09-06). Zadania
ręczne (przeglądarka, podpis, tag) stoją w sekcji `## Outside AO`
danego handoffu, nie w YAML-u.

| Epik | Co | Pkt | Fale AO | Ważność | Zależy od |
|---|---|---|---|---|---|
| [E016 — Higiena planów i warunki AO](../epics/E016-2026-09-06-plan-hygiene-and-ao-readiness/PLAN.md) | STATE.md, scalenie 4 backlogów, zamknięcie E011, E003-T07 i E013 superseded, `coverage/` z gita, `.giter.yaml`, `justfile` | 8 | 2 | High | — |
| [E015 — Jedna prawda o liczniku siedzenia](../epics/E015-2026-09-06-engine-single-truth/PLAN.md) | usunięcie `current_session_secs`, popup na polu kredytowanym, test odwrócony + test na realnej ścieżce, PostureBalance, `break_credit` w bazie, ADR 008 rev (mnożnik 3.0) | 21 | 3 | High | E016 |
| [E017 — Gotowość do wydania](../epics/E017-2026-09-06-release-readiness/PLAN.md) | 7 docs na MoveUp, LICENSE MIT, PRIVACY + Google Fit, CI workflow, `firmware/README.md`; poza AO: podpis (8), przeglądarka, `tauri:build`, tag 0.6.0 | 18 (+8) | 2 | High | E015 |
| [E018 — Konsolidacja frontendu](../epics/E018-2026-09-06-frontend-consolidation/PLAN.md) | jeden reducer + 2 adaptery, codegen `ts-rs` (ADR 017), usunięcie martwego kodu, ESLint, bramka pokrycia, `formatXxx`, UX-FLOW; fala 3 = resztki E012 (T09-T11, recharts ADR 016) | 45 | 4 | Medium | E015 |
| [E019 — Utwardzenie backendu](../epics/E019-2026-09-06-backend-hardening/PLAN.md) | mutexy poison-safe, ADR 014 pierwszeństwo persystencji + jedna ścieżka „dzisiejsze sumy", stałe zdarzeń, EventLogger bez paniki, parytet handlerów, split google_fit i serial_periodic, poprawka opisów w CLAUDE.md | 28 | 7 | Medium | E015 |
| [E020 — Silnik: czysty rdzeń](../epics/E020-2026-09-06-engine-pure-core/PLAN.md) | wstrzyknięty zegar, config poza stanem, jeden model przerwy, jeden snapshot, sygnały z `step()` (ADR 015), tabela scenariuszy jako siatka regresji; przy okazji dwa nowe bugi: hot-reload profilu nie dociera do silnika, reset dzienny liczony po UTC | 59 | 7 | Medium | E015, E019 |

Razem 179 punktów (+8 podpis). Kolejność: E016 → E015 → E017 → E018 ∥ E019
→ E020. E018 i E019 nie dzielą plików, mogą iść równolegle.

### Stare epiki — co z nimi

| Epik | Stan | Decyzja |
|---|---|---|
| E011 autostart | kod gotowy, ceremonia nie | zamyka E016-T03 |
| E012 Analyst | rdzeń gotowy, T09-T11 nietknięte | wchłonięte jako fala 3 E018 |
| E013 signed release + PM3 | utknął w planowaniu | superseded: fale 1-2 → E017, fale 3-5 → E014 (E016-T03 oznacza) |
| E014 rollback pod PM3 | fale 1-2 odblokowane, 3-4 czekają na `pm3-mcp` | zostaje; HANDOFF nie ma jeszcze bloku `## AO` — do dopisania przed dispatchem (wpis w backlogu) |
| E004 alerty T017-T019 | pomysły bez plików zadań | zostają w backlogu jako kandydat na E021 „gamifikacja", nie planowane teraz |
| E010 marketing | 3 albo 10 zadań ludzkich (sprzeczność) | E016-T02 rozstrzyga przy scalaniu backlogów; to nie jest robota agenta |

### Co znaczy „strangler"

Wzorzec strangler fig (Fowler): nowy kod owija stary i przejmuje po jednej
odpowiedzialności naraz, a stary jest usuwany dopiero wtedy, gdy nowa część
udowodniła, że działa. W praktyce tutaj: nie piszemy nowego silnika obok,
tylko w każdym kroku E020 zmieniamy jedną rzecz (zegar, config, snapshot,
sygnały), po każdym kroku `cargo test` jest zielony, a adapter do starego
kształtu żyje, dopóki ostatni konsument nie przejdzie. Przepisanie od zera
odtworzyłoby dokładnie ten błąd, od którego zacząłeś: ktoś w połowie
migracji wymyśliłby trzeci licznik, a stara siatka testów nie łapałaby
nowego kodu.

## 7. Co dalej — jedna rekomendacja

Nowa sesja z czystym kontekstem robi E016 (godzina), potem bierze
`E015/HANDOFF.md`. Ta sesja planująca nie wdraża.

## GAPS

- Nikt nie odpalił aplikacji w przeglądarce ani popupu na żywo; werdykt o
  timerze stoi na kodzie i logach, nie na pikselach.
- Pełny `pnpm tauri:build` nie był uruchomiony.
- `cargo clippy` niedostępny w toolchainie, nie liczono ostrzeżeń.
- `useRemoteDesk.ts` ma ten sam błąd (potwierdzone grepem), ale ścieżka
  telefonu nie była śledzona dalej.
- Blokery E014 w `pm3-mcp` wzięte z HANDOFF-u, nie zweryfikowane w tamtym repo.
- Sprzeczność E010 (3 vs 10 zadań ludzkich) nierozstrzygnięta.
