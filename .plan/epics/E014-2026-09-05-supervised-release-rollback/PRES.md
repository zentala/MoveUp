# E014 — nadzór z powrotem do poprzedniego builda

## TLDR

Zainstalowana aplikacja MoveUp nie ma dziś żadnej ochrony przed złym buildem
ani przed zabiciem jej podczas sesji deweloperskiej. Ten epik dzieli
naprawę na dwie części: MoveUp trzyma kilka ostatnich buildów i zna ten,
który ostatnio naprawdę działał; PM3 dostaje ogólny mechanizm cofania się do
poprzedniego kandydata, gdy nowy pada. Połowa po stronie MoveUp startuje od
razu. Połowa po stronie PM3 czeka na wpis w backlogu tamtego repo — epik to
mówi wprost, zamiast to ukrywać. Rekomendacja: rozbudować PM3 (Podejście B),
nie pisać osobnego, prywatnego strażnika w PowerShellu.

## Kontekst — jak to działa dziś

MoveUp startuje przy logowaniu przez wpis w rejestrze Windows (`HKCU\...\Run\MoveUp`),
który uruchamia zainstalowany plik `desk.exe --minimized`. To jest cała
ochrona, jaka istnieje. Gdy programista uruchamia wersję deweloperską przez
`pnpm tauri:dev`, skrypt startowy najpierw **zabija** zainstalowaną wersję —
bo Windows nie potrafi nadpisać działającego pliku `.exe`. Nic potem nie
przywraca zainstalowanej wersji z powrotem.

Osobno: PM3 (nasz menedżer procesów) dziś **w ogóle nie pilnuje** tej
aplikacji. Nie ma dla niej pliku `pm3.yaml`. Epik E013 dopiero to
planuje — ale w najprostszej formie: jeden proces, jeden health-check,
restart po padzie. Nic nie mówi o tym, co zrobić, gdy padnie **nowo
zainstalowana wersja**, a poprzednia działała dobrze.

## Problem — konkretna historia

Dwie prawdziwe sytuacje, które dziś kończą się tak samo: aplikacja stoi.

**Sytuacja 1.** Programista siada do pracy nad appką, odpala tryb dev.
Skrypt zabija zainstalowaną wersję, żeby zwolnić plik `.exe`. Sesja się
kończy, komputer się nie wyłącza — a zainstalowana wersja MoveUp już nie
wraca. Ergonomia przestaje być pilnowana, dopóki ktoś ręcznie tego nie
zauważy i nie odpali aplikacji z ręki.

**Sytuacja 2.** Nowy build trafia na dysk — powiedzmy, że zepsuty bundle
frontendu albo panikujący handler w Ruście. Aplikacja się uruchamia, ale
nigdy nie zaczyna faktycznie działać (nie odpowiada na `/display/api`).
Nic tego nie wykrywa automatycznie. Jedyna droga naprawy: zauważyć, że coś
jest nie tak, znaleźć starszy instalator na dysku, zainstalować go ręcznie.

## Rozwiązanie

Dwie osobne odpowiedzialności, każda po właściwej stronie:

- **MoveUp** trzyma kilka ostatnich wersji instalatora (3), zna tę, która
  ostatnio naprawdę zadziałała (`last-known-good`), i wystawia prosty
  sygnał zdrowia: `/display/api` zwraca JSON, gdy appka żyje.
- **PM3** dostaje ogólny mechanizm: co 60 sekund sprawdza, czy aplikacja
  odpowiada. Jeśli nie odpowiada **dwa razy z rzędu**, cofa się do
  poprzedniej działającej wersji z listy. Gdy nowsza wersja znowu zacznie
  działać, PM3 wraca do niej.

Kluczowy szczegół, który odróżnia to od naiwnego rozwiązania: PM3 musi
umieć rozpoznać, że aplikację **już trzyma ktoś inny** (na przykład sesja
deweloperska) — i wtedy nic nie robić, zamiast walczyć o tę samą aplikację
co programista.

## Po zmianie

- Zamknięcie sesji dev nie zostawia komputera bez działającej ergonomii na
  dłużej niż dwie minuty — PM3 wykrywa brak aplikacji i odpala ostatnią
  zainstalowaną wersję.
- Zepsuty build nie zostaje na produkcji. PM3 wykrywa dwa nieudane
  sprawdzenia z rzędu i wraca do wersji, która ostatnio realnie działała —
  bez ręcznej interwencji.
- Wpis w rejestrze Windows (`Run`) **znika**. Decyzja Pawła z 2026-09-05:
  jeden nadzorca, nie dwóch. Aplikacja przestaje sama wpisywać się do
  rejestru, a start przy logowaniu przejmuje PM3.
  Uwaga, to nas o coś blokuje: demon PM3 nie ma dziś własnego zadania przy
  logowaniu i wstaje dopiero z watchdoga `pm3-doctor`, czyli w ciągu pięciu
  minut. Dopóki to nie zostanie naprawione po stronie PM3, usunięcie klucza
  `Run` oznaczałoby, że aplikacja wstaje pięć minut po zalogowaniu. Dlatego
  klucz zostaje jako tymczasowy most i znika w tym samym zadaniu, które
  potwierdzi, że PM3 startuje sam.
- Sesja deweloperska nie jest zakłócana. PM3 rozpoznaje zajętość nie po
  ścieżce pliku, tylko po tym, że health check już przechodzi. Oba buildy,
  deweloperski i zainstalowany, serwują ten sam port, więc odpowiedź na
  porcie jest dowodem, że ktoś aplikację trzyma. Nie potrzeba do tego
  żadnego grzebania w API Windowsa.

## Zyski / Wady / Ryzyka

**Zyski.** Każda przyszła aplikacja pod PM3 na tym komputerze dostaje
rollback za darmo, nie tylko MoveUp — bo mechanizm siedzi w PM3, nie w
prywatnym skrypcie tej jednej appki. Programista przestaje musieć pamiętać
o ręcznym odpalaniu appki po sesji dev.

**Wady.** Połowa rozwiązania (mechanizm w PM3) czeka na pracę w innym
repozytorium, na cudzym harmonogramie. Dopóki nie powstanie, `pm3.yaml`
tej appki i wiążący się z nim rollback nie działają — tylko magazyn
buildów po stronie MoveUp jest gotowy wcześniej.

**Ryzyka.** Jeśli wykrywanie „kto trzyma aplikację" pomyli się, PM3 mógłby
zagłodzić zainstalowaną wersję nawet po zakończeniu sesji dev — dlatego
weryfikacja epiku wprost sprawdza powrót do normalnego działania po
zamknięciu sesji dev, nie tylko brak przeszkadzania w jej trakcie. Build,
który działa na starcie, ale psuje się po jakimś czasie, nie zostanie
wykryty — sonda „czy build jest dobry" sprawdza raz, przy instalacji i
awansie, nie w sposób ciągły; to świadomie poza zakresem tego epiku.

## Punkty + Importance + potrzebna decyzja

Suma epiku: **39 punktów** (skala Fibonacciego), 9 zadań. Rozbicie: 15
punktów (fale 1-2, magazyn buildów po stronie MoveUp) **nie jest
zablokowane** i może ruszyć od razu; 24 punkty (fale 3-4, integracja z PM3)
**są zablokowane** na dwóch wpisach w backlogu `pm3-mcp` — Importance
Medium/8 i Medium/5 tam.

**Decyzja potrzebna od Pawła:** żadna — podejście B (rozbudowa PM3) jest
rekomendowane i przyjęte jako decyzja projektowa (D1-D7 w `PLAN.md`).
Jedyne, co Paweł może chcieć zobaczyć, to priorytet dwóch wpisów w backlogu
`pm3-mcp`, bo od nich zależy, kiedy fale 3-4 tego epiku w ogóle ruszą.

Pełny plan techniczny: [`PLAN.md`](./PLAN.md).
Wpisy w backlogu PM3, na których czeka ten epik:
`int://mATX.lan/C:/code/pm3-mcp/.plan/BACKLOG.md`.
