---
formatVersion: 1
type: prd
status: todo
---

# E018 — Konsolidacja frontendu (prezentacja)

## TLDR

Frontend działa (248 zielonych testów), ale niesie balast poprzedniej
generacji: dwa haki stanu robiące to samo, ręcznie kopiowane typy z Rusta,
martwe komponenty wciąż kompilowane, `pnpm lint` wołający nieistniejący
program, i bramka pokrycia testów, która nigdy nikogo nie zablokowała. Ten
epik zamyka te szwy (45 punktów, 3 fale) i dodatkowo kończy trzy zadania UI
zostawione przez E012. Decyzja do podjęcia: **D4** — czy dociągnąć pokrycie
do 80%, czy uczciwie obniżyć próg z notatką w backlogu.

## 1. Jeden reducer zamiast dwóch haków stanu (8 pkt, High)

**Kontekst.** Aplikacja ma dwa sposoby dostarczania danych do popupu:
`useDesk.ts` (gdy działa jako appka desktopowa przez Tauri) i
`useRemoteDesk.ts` (gdy telefon ogląda ten sam ekran przez WebSocket).

**Problem.** Oba pliki przepisują tę samą logikę — te same pola stanu, te
same wyliczenia „poprzednia sesja"/„dzisiejsze sesje" — osobno. Gdy dodano
`continuous_computer_secs` (ADR 011), trzeba było wpisać to ręcznie w obu
miejscach. `useDesk.ts` ma dziś 0,78% pokrycia testami, bo środowisko
testowe nie potrafi udawać prawdziwego Tauri.

**Rozwiązanie.** Jeden „reducer" (czysta funkcja licząca nowy stan) plus
dwa cienkie adaptery transportu — jeden gada z Tauri, drugi z WebSocketem.
Reducer testuje się bez żadnego z nich.

**Po zmianie.** Nowe pole dodaje się raz, nie dwa razy. `useDesk.ts` ma
prawdziwe testy, nie przypadkowe 0,78%.

**Zyski/Wady/Ryzyka.** Zysk: koniec podwójnych edycji, realna testowalność.
Wada: więcej plików do przeczytania na start. Ryzyko: średnie — refaktor
dotyka kodu, na którym stoi cały popup; siatka 248 testów łapie regresje.

**Punkty + decyzja: 8, High. Rekomendacja: zrobić w Fali 1.**

## 2. Typy generowane z Rusta zamiast przepisywane ręcznie (5 pkt, Medium)

**Kontekst.** `types.ts` i `SettingsTypes.ts` to ręczne kopie struktur
Rustowych (`session_types.rs`, `config.rs`).

**Problem.** Zmiana pola w Ruście nie daje żadnego sygnału po stronie TS —
ani błędu kompilacji, ani testu — dopóki ktoś nie zauważy tego ręcznie w
działającej appce.

**Rozwiązanie.** Biblioteka `ts-rs` generuje pliki `.ts` wprost ze
struktur Rustowych (decyzja D3, alternatywa `specta` odrzucona — robi
więcej niż potrzeba: cały system bindowania komend, nie tylko typów DTO).

**Po zmianie.** Zmiana nazwy pola w Ruście = błąd kompilacji TypeScript,
nie cichy błąd w runtime.

**Zyski/Wady/Ryzyka.** Zysk: dryf typów staje się niemożliwy. Wada: nowa
zależność (`ts-rs`) w `Cargo.toml`. Ryzyko: niskie — zmiana mechaniczna,
nie zmienia żadnej logiki.

**Punkty + decyzja: 5, Medium. Rekomendacja: Fala 2, po reducerze.**

## 3. Usunięcie martwego kodu (3 pkt, Medium)

**Kontekst.** Pięć komponentów sprzed obecnego widgetu (`OneBarWidget`) —
`AppProgressBar`, `HeightRail`, `SessionProgress`, `TodayStats`,
`TransitionBanner` — i osobne wejście `overlay.html` z marca 2026 wciąż
siedzą w repo i są kompilowane.

**Problem.** Nic ich nie używa, ale ich testy wliczają się do „248 testów
przechodzi", co zawyża wrażenie pokrycia. `overlay.html` to atrapa —
prawdziwy overlay to natywne okno WinAPI w Ruście.

**Rozwiązanie.** Usunąć wszystkie siedem plików + wpis w `vite.config.ts`,
po wcześniejszym grepie potwierdzającym brak żywych importów.

**Po zmianie.** Mniej plików, uczciwsza liczba testów, mniejszy build.

**Zyski/Wady/Ryzyka.** Zysk: czystość, prawdziwa liczba testów. Wada:
żadna. Ryzyko: niskie, pod warunkiem grepa przed usunięciem.

**Punkty + decyzja: 3, Medium. Rekomendacja: Fala 1, zrobić od razu.**

## 4. ESLint naprawdę zainstalowany + bramka pokrycia podpięta (3+5 pkt, High)

**Kontekst.** `pnpm lint` istnieje w `package.json`, ale program `eslint`
nigdy nie został zainstalowany — komenda pada natychmiast. Bramka pokrycia
80% w `vite.config.ts` jest prawdziwa, ale `pnpm build` jej nie woła.

**Problem.** Lint nigdy w praktyce nie działał. Pokrycie dziś wynosi
76,5/68,1/66,0% — poniżej progu na wszystkich trzech osiach — a nic tego
nie sygnalizuje.

**Rozwiązanie.** Zainstalować ESLint z nowoczesnym (flat) configiem;
podpiąć sprawdzanie pokrycia pod `pnpm build`; dodać `justfile` (konwencja
z innych repo), którego to repo dziś nie ma.

**Po zmianie.** `pnpm lint` naprawdę coś sprawdza. Build faktycznie
odmawia przejścia, gdy pokrycie spadnie poniżej progu.

**Zyski/Wady/Ryzyka.** Zysk: dwie martwe bramki jakości ożywają. Wada:
build będzie wolniejszy (liczy pokrycie). Ryzyko: średnie — jeśli po
Fali 1 pokrycie nadal jest poniżej 80%, trzeba świadomie obniżyć próg
(decyzja D4) zamiast udawać, że jest inaczej.

**Punkty + decyzja: 3+5=8, High. D4 wymaga Twojego potwierdzenia, jeśli
próg trzeba obniżyć — zobacz `## Outside AO` w HANDOFF.**

## 5. Porządki: formatowanie i dwa dokumenty (3+1+1 pkt, Low/Medium)

**Kontekst.** Sześć niezależnych funkcji `formatXxx` robi to samo
(formatuje datę/czas) w czterech różnych plikach zamiast korzystać ze
wspólnego `utils/format.ts`. Dokument `.claude/rules/overlay.md` opisuje
nieistniejący skrypt `.sh` zamiast realnego `.ps1`. `.arch/UX-FLOW.md` nie
wie nic o widgecie Steps (Google Fit) ani o kliknięciu w timeline
otwierającym Analyst.

**Rozwiązanie.** Przenieść formatowanie w jedno miejsce; poprawić oba
dokumenty, żeby opisywały to, co naprawdę dzieje się w kodzie.

**Po zmianie.** Jedno miejsce do naprawy błędu formatowania. Dokumenty
znowu są prawdą, nie historią.

**Punkty + decyzja: 3+1+1=5, Low/Medium. Fala 1, przy okazji innych zmian.**

## 6. Dokończenie Analyst UI z E012 (5+8+3 pkt, Medium)

**Kontekst.** E012 zostawił trzy gotowe, opisane, ale niewdrożone zadania:
przełożenie nagłówka daty na górę i nawigacji dni na dół (T09), donuty KPI
zamiast słupków (T10, biblioteka Recharts), i pulsowanie kart KPI przy
zmianie dnia (T11).

**Rozwiązanie.** Wdrożyć wszystkie trzy w kolejności T09→T10→T11 (każde
kolejne buduje na plikach poprzedniego), z obowiązkowym mockupem przed
wdrożeniem (zasada `ux-design-flow.md`) i przejściem przeglądarkowym po.

**Po zmianie.** Analyst wygląda tak, jak ostatnio zaakceptowałeś w
rozmowie o E012, zamiast czekać dalej w niewdrożonym backlogu.

**Zyski/Wady/Ryzyka.** Zysk: zamyka ostatni otwarty wątek E012. Wada:
Recharts to nowa zależność (uzasadniona w ADR 016). Ryzyko: niskie — trzy
zadania są już w pełni zaprojektowane, tylko niewdrożone.

**Punkty + decyzja: 5+8+3=16, Medium. Fala 3, po Falach 1-2.**

## Razem

45 punktów, 3 fale (19 / 10 / 16), zależność od E015 (musi wylądować
pierwszy — ten epik zakłada, że `limit_used_secs` jest już jedynym liczonym
polem). Jedna decyzja wymaga Twojego potwierdzenia: D4, obniżenie progu
pokrycia, jeśli Fala 1 go nie dociągnie.
