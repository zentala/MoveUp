# E016 — Higiena planów i gotowość pod AO

## TLDR

Pełny przegląd architektury z 2026-09-06 znalazł, że `.plan/` w tym repo jest
niewiarygodne: cztery pliki udają backlog, `STATE.md` sam sobie zaprzecza,
epik E011 jest zrobiony w kodzie, ale nigdy formalnie zamknięty, dwa foldery
z artefaktami testów siedzą w gicie, a repo nie ma żadnego z plików, których
potrzebuje Agent Orchestrator (AO), żeby wykonać kolejny epik bez nadzoru.
E016 naprawia wszystkie pięć rzeczy w jednej krótkiej sesji, bez dotykania
kodu aplikacji — 8 punktów, jedna fala. To musi pójść PRZED epikiem E017
(gotowość do wydania), bo E017 ma być pierwszym epikiem odpalonym przez AO.

## Pozycje do decyzji

### 1. `.plan/STATE.md` sam sobie zaprzecza

**Kontekst.** `STATE.md` to plik, który każdy agent czyta jako pierwszy,
żeby wiedzieć, co się dzieje w repo. Ma dwie części: nagłówek YAML
(maszynowy) i tekst pod spodem (dla człowieka i dla agenta).

**Problem.** Nagłówek już mówi „E011 i E012 zrobione", ale tekst pod spodem
nadal mówi „E011 aktywny, T03 zrobiony, fala 1 i 2-3 w toku" — mimo że
wszystkie pięć zadań E011 ma status zrobione w `.plan/DONE.md`. Dodatkowo:
plik mówi, że zostały 3 zadania ludzkie z E010, a stary plik `BACKLOG.md`
w korzeniu repo wymienia 10 — nikt tego nie pogodził.

**Rozwiązanie.** Task T01: przepisać sekcję statusu E011 na zgodną z
nagłówkiem, dopisać jedno zdanie tłumaczące różnicę 3 vs 10.

**Po zmianie.** Każdy agent, który otworzy `STATE.md`, dostanie jedną,
spójną historię — bez dwóch sprzecznych wersji naraz.

**Zyski / Wady / Ryzyka.** Zysk: zaufanie do najważniejszego pliku w repo.
Wady: brak. Ryzyko: brak — to czysta korekta faktów, zero decyzji
projektowej.

**Punkty i decyzja.** 1 punkt. Rekomendacja: zrobić od razu, bez pytania.

---

### 2. Cztery pliki udają backlog

**Kontekst.** Oprócz `.plan/BACKLOG.md` (aktywnie prowadzony, ma wpisy z
2026-09) istnieją jeszcze root `BACKLOG.md` (339 linii, stary), root
`TASKS.md` (już tylko przekierowanie) i root `ORCHESTRATOR.md` (3 linie,
martwy wskaźnik na sprint z marca).

**Problem.** Root `BACKLOG.md` powiela 80% treści `.plan/BACKLOG.md`, ale ma
jedną unikalną sekcję: 10-punktową listę „rzeczy tylko dla Pawła" (zdjęcia,
wideo, konto Stripe, Plausible, Google Search Console) — nigdzie indziej jej
nie ma. Jeśli ktoś usunie root `BACKLOG.md` bez przeniesienia tej sekcji,
ta lista przepadnie.

**Rozwiązanie.** Task T02: przenieść tę sekcję (i wszystko inne unikalne)
dosłownie do `.plan/BACKLOG.md`, dopiero potem skasować trzy stare pliki.

**Po zmianie.** Jeden plik backlogu, `.plan/BACKLOG.md`, zero duplikatów,
zero martwych wskaźników.

**Zyski / Wady / Ryzyka.** Zysk: jedno źródło prawdy. Wady: brak. Ryzyko:
niskie — jeśli coś zostanie pominięte przy scalaniu, to strata informacji;
dlatego task ma osobny skrypt weryfikujący, że kluczowa sekcja faktycznie
tam wylądowała.

**Punkty i decyzja.** 2 punkty. Rekomendacja: zrobić od razu.

---

### 3. E011 zrobiony w kodzie, nigdy formalnie zamknięty

**Kontekst.** Epik E011 (autostart) ma wszystkie 5 zadań zrobionych i
scalonych. Ale jego własna checklista „Final integration" (build, tag,
wpis w historii, przegląd usprawnień) nigdy nie została odhaczona — a
`.plan/HISTORY.md` w ogóle nie istnieje, mimo 12+ zamkniętych epików.

**Problem.** Dwa dodatkowe luźne wątki wiszą bez oznaczenia: `E003-T07`
(automatyzacja GitHub Releases) miał zostać wchłonięty przez E013, ale nigdy
nie oznaczono go jako „zastąpiony"; a sam E013 utknął w planowaniu i jego
zakres dzielimy teraz na dwa nowe epiki (E017 — gotowość do wydania, E014 —
nadzór nad wdrożeniem).

**Rozwiązanie.** Task T03: napisać `.plan/HISTORY.md` z wpisami dla E011 i
E012, odhaczyć/wyjaśnić checklistę E011 (część punktów jest już
bezprzedmiotowa — np. wersja poszła dalej niż 0.4.0, retroaktywny tag nic
nie da), oznaczyć E003-T07 i E013 jako zastąpione.

**Po zmianie.** E011 jest formalnie zamknięty na papierze, a nie tylko w
kodzie; nikt nie zapyta drugi raz „czy E013 jeszcze żyje".

**Zyski / Wady / Ryzyka.** Zysk: czysta historia projektu. Wady: brak.
Ryzyko: brak — żadna z tych adnotacji nie zmienia zachowania aplikacji.

**Punkty i decyzja.** 2 punkty (zależy od T02, bo oba dotykają tego samego
pliku backlogu). Rekomendacja: zrobić od razu, w tej kolejności.

---

### 4. Dwa foldery z artefaktami testów w gicie

**Kontekst.** `coverage/` (raport pokrycia testami) i
`test-performance-report/` — 18 plików łącznie — są scommitowane, mimo że
`.gitignore` już wyklucza `dist/`.

**Problem.** Te pliki zmieniają się przy każdym uruchomieniu testów, więc
każdy commit, który nie dotyczy testów wcale, i tak pokazuje zmiany w tych
folderach — szum w historii i większe klony repo.

**Rozwiązanie.** Task T04: `git rm --cached` na obu folderach + dopisanie
ich do `.gitignore`. Pliki zostają na dysku, znikają tylko z gita.

**Po zmianie.** `git status` po `pnpm test` nie pokazuje już 18 zmienionych
plików, których nikt nie chciał commitować.

**Zyski / Wady / Ryzyka.** Zysk: czystsza historia gita. Wady: brak.
Ryzyko: brak.

**Punkty i decyzja.** 1 punkt. Rekomendacja: zrobić od razu.

---

### 5. Repo nie ma niczego, czego potrzebuje AO

**Kontekst.** Agent Orchestrator (AO) to system, który ma odpalać kolejne
epiki (E017 i dalej) bez ciągłego nadzoru — ale ma pięć warunków wstępnych,
sprawdzanych przed każdym uruchomieniem.

**Problem.** Ten warunek nie jest spełniony: repo nie ma `.giter.yaml`
(mówi AO, jakie pliki skopiować do izolowanego worktree) ani `justfile`
(jednolity interfejs komend `just build`/`just test`, zamiast pamiętać, że
to konkretnie `pnpm`). Bez nich AO odmówi startu.

**Rozwiązanie.** Task T05: dopisać oba pliki. Sprawdzono też czwarty warunek
AO (czy jakiś hook w repo nie brudzi drzewa roboczego po commicie worker'a)
— w tym repo nie ma żadnych haków ani `.husky/`, więc ten warunek jest już
spełniony, zapisujemy to jako fakt, nie budujemy zabezpieczenia przed czymś,
czego nie ma.

**Po zmianie.** E017 (i każdy kolejny epik) może pójść przez `ao run` zamiast
przez ręczne wdrażanie zadanie po zadaniu.

**Zyski / Wady / Ryzyka.** Zysk: odblokowuje nadzorowaną automatyzację dla
przyszłych epików. Wady: brak. Ryzyko: niskie — `justfile` tylko opakowuje
istniejące skrypty `pnpm`, nie zmienia ich.

**Punkty i decyzja.** 2 punkty. Rekomendacja: zrobić od razu — to blokuje
start E017 przez AO.

---

## Razem

8 punktów, jedna fala (T03 czeka na T02), zero zmian w kodzie aplikacji.
Plan: [`PLAN.md`](PLAN.md). Wdrożenie: [`HANDOFF.md`](HANDOFF.md).
