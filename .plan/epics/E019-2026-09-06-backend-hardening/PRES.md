# E019 — Utwardzenie backendu (prezentacja)

Powiązane: [`PLAN.md`](PLAN.md) (pełny plan techniczny), [`HANDOFF.md`](HANDOFF.md)
(kolejność zadań). Źródło: [`../../reports/_review-2026-09-06/backend.md`](../../reports/_review-2026-09-06/backend.md).

## TLDR

Przegląd architektury backendu (2026-09-06) znalazł 8 konkretnych ryzyk w
kodzie Rust — żadne nie psuje dziś działania appki (492/492 testów zielone),
ale każde jest realnym zagrożeniem albo źródłem przyszłego bugu. Epik
naprawia wszystkie 8, plus jedno fałszywe zdanie w `CLAUDE.md`. 28 punktów,
8 zadań, bez ryzykownych decyzji architektonicznych do podjęcia przez Ciebie
— rekomendacja jest jedna i recenzent się z nią zgadza.

## 1. Zamki (mutexy) w `commands.rs` nie przeżywają panicu

**Kontekst.** Backend trzyma stan aplikacji (sesja, baza, config) za zamkami
(`Mutex`). Gdy jeden wątek panikuje trzymając zamek, ten zamek staje się
"zatruty" — każde kolejne `.lock()` też panikuje, chyba że kod jawnie to
obsłuży.

**Problem.** `commands.rs` — plik obsługujący wszystkie 34 komendy z frontendu
— ma 14 miejsc, gdzie zamek jest odblokowywany na sztywno (`.unwrap()`). Jeśli
gdziekolwiek w appce coś panikuje trzymając ten sam zamek, każde kolejne
kliknięcie w UI zwraca błąd zamiast danych, aż do restartu appki.

**Rozwiązanie.** Zamień 14 miejsc na wzorzec, który już działa w trzech innych
plikach (`tray_controller.rs`, `remote_server.rs`) — zamek przeżywa
zatrucie i zwraca dane zamiast panikować.

**Po zmianie.** Panic gdziekolwiek indziej w appce nie blokuje już IPC.

**Zyski/Wady/Ryzyka.** Zysk: jedno realne ryzyko stabilności zniknięte.
Wada: brak — to jest kopiowanie wzorca, który już działa gdzie indziej. Ryzyko: niskie.

**Punkty: 3. Decyzja: brak do podjęcia — czysto techniczna poprawka.**

## 2. Cztery sposoby liczenia "dzisiejszych sum", żaden nie jest oficjalny

**Kontekst.** Appka liczy, ile dziś siedziałeś/stałeś na cztery różne sposoby:
baza SQLite, plik configu, snapshoty co minutę, log zdarzeń tekstowych.

**Problem.** Nie ma dokumentu mówiącego, który wygrywa, gdy się nie zgadzają.
Dwa realne miejsca w kodzie liczą tę samą liczbę osobno (`get_today_summary`
robi to inaczej niż inicjalizacja przy starcie appki) — rozjazd między nimi
jest dziś niewykrywalny.

**Rozwiązanie.** Jeden dokument decyzyjny (ADR) mówiący, co jest źródłem
prawdy, plus jedna funkcja w kodzie, z której korzystają oba miejsca zamiast
liczyć osobno.

**Po zmianie.** Jeden kod liczy "dzisiejsze sumy"; snapshoty i log zdarzeń są
jawnie oznaczone jako "tylko do wglądu", nie jako alternatywne źródło prawdy.

**Zyski/Wady/Ryzyka.** Zysk: koniec z niewidzialnym rozjazdem liczb. Wada:
największe zadanie w epiku (8 pkt) — dotyka kilku plików naraz. Ryzyko:
średnie, ale nie dotyka plików, które E015 właśnie przerabia (sprawdzone).

**Punkty: 8. Decyzja: brak do podjęcia — rekomendacja recenzenta jest jedna
i jest w planie.**

## 3. Nazwy zdarzeń jak `desk:state-changed` są rozsiane jako gołe teksty

**Kontekst.** Backend i frontend komunikują się m.in. przez zdarzenia z
nazwami typu `"desk:state-changed"`. Dziś każde miejsce w kodzie wpisuje tę
nazwę ręcznie, jako zwykły tekst.

**Problem.** Literówka w jednym z ~15 miejsc nie daje błędu kompilacji — po
prostu appka cicho przestaje reagować na to zdarzenie.

**Rozwiązanie.** Jedna lista nazw w Rust (`desk_events.rs`) i jej lustrzane
odbicie po stronie TypeScript (`src/events.ts`) — na razie tylko jako nowy
plik, bez przepinania istniejącego kodu frontendu (to zrobi osobny epik
E018, żeby uniknąć konfliktu).

**Po zmianie.** Literówka w nazwie zdarzenia nie kompiluje się.

**Zyski/Wady/Ryzyka.** Zysk: cała klasa cichych bugów znika. Wada: żadna.
Ryzyko: niskie.

**Punkty: 5. Decyzja: brak do podjęcia.**

## 4. Jeden logger panikuje, bliźniaczy tylko ostrzega

**Kontekst.** Dwa niemal identyczne moduły piszą logi na dysk — jeden log
zdarzeń tekstowych, jeden snapshoty JSON co minutę.

**Problem.** Gdy folder logów nie da się utworzyć (np. problem z
uprawnieniami), jeden z nich **wywraca całą appkę przy starcie**, drugi
tylko ostrzega i appka działa dalej bez tego loga.

**Rozwiązanie.** Ujednolić: oba tylko ostrzegają, żaden nie wywraca appki.

**Po zmianie.** Problem z uprawnieniami do folderu logów nie blokuje
uruchomienia appki.

**Zyski/Wady/Ryzyka.** Zysk: mniej sposobów na to, żeby appka w ogóle nie
wystartowała. Wada: żadna. Ryzyko: niskie.

**Punkty: 2 (razem z pkt. 5 poniżej — to jedno zadanie w planie: T04).
Decyzja: brak do podjęcia.**

## 5. Lista komend Tauri jest ręcznie zdublowana (debug vs release)

**Kontekst.** Appka rejestruje ~34 komendy IPC osobno dla wersji
deweloperskiej i wersji wydawanej użytkownikom — dwie prawie identyczne
listy w kodzie.

**Problem.** Nowa komenda dodana do jednej listy i zapomniana w drugiej nie
daje błędu — po prostu istnieje tylko w jednej wersji appki, cicho.

**Rozwiązanie.** Test, który psuje się, jeśli listy się rozjadą.

**Po zmianie.** Zapomniana komenda w jednej z list = czerwony test, nie
cichy brak funkcji w wydaniu.

**Zyski/Wady/Ryzyka.** Zysk: koniec z cichym rozjazdem między wersją dev i
release. Wada: żadna. Ryzyko: niskie.

**Punkty: (wliczone w T04 powyżej). Decyzja: brak do podjęcia.**

## 6. Dwa pliki integracji Google Fit są za duże wg naszej własnej konwencji

**Kontekst.** Repo ma zasadę: żaden plik kodu nie przekracza 250 linii —
większe pliki są dzielone na plik główny + plik testów + plik pomocniczy.
Prawie wszędzie ta zasada jest przestrzegana.

**Problem.** Dwa pliki integracji z Google Fit (473 i 328 linii) łamią tę
zasadę — jedyne dwa w całym repo, mimo że są najlepiej udokumentowanym
modułem w projekcie.

**Rozwiązanie.** Podzielić dokładnie wg wzorca, który już działa gdzie
indziej (osobny plik na testy, osobny na "surowe" wywołania HTTP).

**Po zmianie.** Oba pliki (i wszystkie nowe) ≤250 linii.

**Zyski/Wady/Ryzyka.** Zysk: spójność z resztą repo, łatwiejsze czytanie.
Wada: żadna — to czysto mechaniczny podział, zero zmiany zachowania.
Ryzyko: niskie.

**Punkty: 3. Decyzja: brak do podjęcia.**

## 7. Jedna funkcja robi pięć rzeczy naraz przy każdym odczycie z czujnika

**Kontekst.** Co ok. sekundę i co minutę backend przetwarza odczyt z
czujnika wysokości biurka — zmiana stanu, zapis do bazy, powiadomienia, log,
reset dobowy.

**Problem.** Dwie funkcje (`check_periodic`, `handle_reading`) robią to
wszystko naraz, w jednym ciągu — sprzecznie z własnym komentarzem w kodzie
mówiącym "wydzielone, żeby serial.rs zajmował się tylko I/O".

**Rozwiązanie.** Rozbić na osobne funkcje/pliki wg odpowiedzialności —
dokładnie tak, jak już zrobiono z `tray_controller.rs`.

**Po zmianie.** Każda odpowiedzialność w osobnym, nazwanym miejscu.

**Zyski/Wady/Ryzyka.** Zysk: łatwiej dodać/zmienić jedną rzecz bez ryzyka
zepsucia pozostałych czterech. Wada: żadna. Ryzyko: niskie — zero zmiany
publicznego zachowania.

**Punkty: 3. Decyzja: brak do podjęcia.**

## 8. Dwa miejsca liczą to samo dla zdalnego wyświetlacza (telefon/przeglądarka)

**Kontekst.** Appka wystawia stan sesji na telefon/przeglądarkę przez
WebSocket i REST. Dwa niezależne miejsca w kodzie budują tę samą strukturę
danych z tych samych trzech źródeł.

**Problem.** Zmiana sposobu liczenia metryk musi być zrobiona w dwóch
miejscach naraz — łatwo o jedno zapomnieć.

**Rozwiązanie.** Jedna wspólna funkcja, dwa miejsca ją wywołują.

**Po zmianie.** Jedna zmiana = jedno miejsce do edycji.

**Zyski/Wady/Ryzyka.** Zysk: mniej okazji do rozjazdu WS vs REST. Wada:
żadna. Ryzyko: niskie.

**Punkty: 3. Decyzja: brak do podjęcia.**

## Bonus — jedno fałszywe zdanie w `CLAUDE.md`

`CLAUDE.md` twierdzi, że `TrayController` "tylko wykonuje sygnały". To
nieprawda — liczy też logikę (progres okrążeń, tekst tooltipa, broadcast).
Poprawka to jedno zdanie. **1 punkt, zero decyzji do podjęcia.**

## Razem

28 punktów, 8 zadań, kolejność w [`HANDOFF.md`](HANDOFF.md). Żadna pozycja
nie wymaga Twojej decyzji — wszystkie są jednoznacznymi poprawkami
technicznymi z jedną rekomendacją, z którą zgadza się recenzent.
