# E017 — Gotowość do wydania (prezentacja)

## TLDR

Aplikacja buduje się czysto i nic nie wysyła bez zgody, ale każdy dokument
dla użytkownika opisuje inny, stary produkt ("zntlDesk"), obiecuje
auto-update, którego nie ma w kodzie, i nie ma licencji mimo modelu
open-core. To są dokumenty i proces, nie kod. 18 punktów AO w dwóch falach,
plus cztery kroki poza AO (podpis, przeglądarka, pełny build, tag).

## 1. Kontekst

`docs/README.md`, `USER_INSTALL.md`, `USER_SUPPORT.md`, `USER_UPDATES.md`,
`PRIVACY.md` i dwa mniejsze pliki wciąż mówią "zntlDesk", stara ścieżka
danych, stare repo `zntl-tray`. `PRIVACY.md` twierdzi, że nic nie wychodzi z
komputera, mimo że integracja Google Fit realnie łączy się z Google. Firmware
to goły plik `.ino` bez instrukcji. Nie ma pliku LICENSE. CI ma dwa
workflow'y — jeden już dobry (`release-baseline.yml`, zrobiony w E013),
drugi (`test.yml`) celuje w folder `apps/desk`, którego nie ma od dawna.

## 2. Problem

Ktoś obcy, kto pobierze tę aplikację, przeczyta dokumenty o innym programie,
zobaczy fałszywą deklarację prywatności i nie znajdzie licencji przy
open-core'owym modelu biznesowym. To kosztuje wiarygodność pierwszego
kontaktu — dokładnie to, czego ma unikać release.

## 3. Rozwiązanie

Przepisać pięć dokumentów na prawdziwe nazewnictwo i ścieżki, dodać sekcję
o kablach USB (masz już zweryfikowaną wiedzę w `CLAUDE.md`), ujawnić
Google Fit w polityce prywatności, dodać LICENSE (MIT — Twoja decyzja D3),
naprawić `test.yml`, napisać `firmware/README.md`, odświeżyć metryki
wydajności. Podpis kodu, przebieg w przeglądarce i pełny build zostają jako
osobne kroki poza automatyzacją — bo wymagają Twojej decyzji albo realnego
sprzętu/przeglądarki.

## 4. Po zmianie

Dokumenty mówią prawdę o produkcie, który realnie ship'ujesz. Repo ma
licencję. CI nie kłamie o własnej strukturze. Firmware ma instrukcję
flashowania. Zostaje jedna prawdziwa przeszkoda wielodniowa: wybór dostawcy
podpisu kodu — i to jest decyzja, nie zadanie do zautomatyzowania.

## 5. Zyski / Wady / Ryzyka

- Zysk: usuwa dziesięć realnych blokerów release'u za 18 punktów AO.
- Wada: nie buduje auto-updatera — zostaje ręczna instrukcja pobrania z
  GitHub Releases (świadoma decyzja: updater to osobny follow-on, nie ten
  epik).
- Ryzyko: niskie — to dokumenty i metadane, nie zmiana zachowania aplikacji.

## 6. Punkty i decyzja

18 pkt AO (2 fale) + ~19 pkt poza AO (podpis, przeglądarka, build, tag),
High. Decyzja potrzebna: `go` na E017 po domknięciu E016 i E015. Osobna
decyzja, którą musisz podjąć podczas realizacji: dostawca podpisu kodu
(E013 Wave 2) — nie ma dziś rekomendacji, bo zależy od dostępnych kont
organizacyjnych, których agent nie widzi.
