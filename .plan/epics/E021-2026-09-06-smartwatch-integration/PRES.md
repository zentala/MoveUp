# E021 — Zegarek i dyktowanie głosem (prezentacja)

## TLDR

Chciałeś „pełne 2a": zegarek podpięty przez oficjalne SDK i dyktafon z
nadgarstka. Research dał trzy twarde fakty: Twój Xiaomi Watch 4 nie
uruchomi żadnej aplikacji, **żadne** oficjalne SDK zegarka (Apple, Wear
OS, Garmin) nie daje mikrofonu apce trzeciej, a Google Fit — dziś jedyne
źródło kroków — **Google wyłącza pod koniec 2026**, czyli za ~4 miesiące.
Plan robi więc to, co przeżyje: telefon jest mostem (dane + głos),
zegarek jest ekranem podglądowym (dostaje odpowiedzi jako lustrzane
powiadomienia — to już dziś działa), a aplikacja dostaje wejście na dane
zdrowotne niezależne od Google. 51 punktów, 10 zadań, 5 fal, pełny AO.

## 1. Kontekst

Dziś MoveUp czyta kroki z Google Fit REST (ADR 012), do którego Mi Fitness
synchronizuje Twój Watch 4. Serwer `:3390` serwuje ten sam React na
telefon (`/display`), ale tylko do odczytu — nic nie da się do aplikacji
wysłać. Wizja od marca mówi „zegarek steruje agentem głosem" i „ciało:
smartwatch (ruch, HRV)"; żadne z tych zdań nie ma jeszcze linijki kodu.

## 2. Problem

Trzy rzeczy naraz:

- **Google Fit REST gaśnie pod koniec 2026** (nowe rejestracje zablokowane
  od maja 2024, dokładnej daty Google nie podał). Bez zmian aplikacja
  straci kroki w ciągu kwartału. Zastępca — Health Connect — działa tylko
  na Androidzie, na urządzeniu; z Windowsa nie da się go odczytać.
- **Watch 4 to zamknięty RTOS** (`wearables.md`): brak aplikacji, brak
  API, tylko lustrzane powiadomienia z telefonu.
- **Mikrofon z nadgarstka jest niemożliwy pod żadnym oficjalnym SDK** —
  także gdybyś kupił Wear OS. To ograniczenie platform, nie Xiaomi.

## 3. Rozwiązanie — i konflikt rozstrzygnięty

Brief zostawił wybór A/B/C. Rozstrzygam: **A (telefon jako most), z D
(oficjalna apka Android z Health Connect) jako następnym epikiem**.

| Opcja | Werdykt | Dlaczego |
|---|---|---|
| **A. Telefon = most, zegarek = podgląd** | **wybrana** | zero nowego sprzętu; każdy element to standard (HTTP, dyktowanie systemowe, Web Speech, ntfy); przeżywa śmierć Fit; kontrakt push jest dokładnie tym, czego potrzebuje D |
| B. Minimum: tylko Fit + HR, głos „kiedyś, jak będzie Wear OS" | odrzucona | przesłanka fałszywa (Wear OS też nie ma mikrofonu w SDK); przy wyłączeniu Fit apka zostaje bez źródła; prosiłeś o pełne 2a |
| C. Własny firmware na Watch 4 | **jawnie odrzucona** | sprzeczna z „oficjalne SDK, out of the box"; nie ma na czym testować; ryzyko cegły |
| D. Apka Android (Kotlin/Tauri mobile) z Health Connect + `SpeechRecognizer` | cel na później (kandydat E022) | najczystsza „oficjalna" historia, ale inny toolchain, 10-20 pkt hydrauliki zanim cokolwiek dowiezie, i tak potrzebuje wejścia z A |

Co powstaje:

1. **Wejście na dane zdrowotne** (`HealthSource`): Google Fit zostaje za
   interfejsem (z podpowiedzią „kończy się pod koniec 2026"), obok
   pojawia się `POST /display/health` z tokenem — kroki/HR może wpychać
   dowolny most z telefonu (Health Connect w E022, Tasker, HA, curl).
2. **Dyktowanie na telefonie** w istniejącym `/display`: pole tekstowe z
   dyktowaniem klawiatury (działa zawsze, także po `http://`) + przycisk
   mikrofonu Web Speech, gdy przeglądarka pozwoli. Zadanie T01 sprawdza
   to na Twoim telefonie ZANIM ktoś napisze komponent.
3. **Intencje**: „drzemka 10" odkłada przypomnienie (istniejącą ścieżką
   snooze), „notatka: ból pleców" ląduje w bazie i `events.log`, „idę na
   spacer / wróciłem" zapisuje się jako zdarzenie. **Silnik sesji nie jest
   dotykany** (ADR 015) — ręczna nadpisywanie stanu „chodzę" to osobna
   decyzja na później.
4. **Webhook powiadomień** (format ntfy): alert siedzenia i potwierdzenie
   dyktowania idą na telefon → lustrzane na zegarek. Tak zegarek „mówi".
5. **Odpowiedź AI** (opcja): własny klucz OpenRouter w `.env`, ≤60 słów
   coachingu; bez klucza — cisza, nie błąd.

## 4. Po zmianie

Idziesz na spacer, mówisz do telefonu „drzemka 15" — zegarek pokazuje
„OK, przypomnę o 14:35". Wracasz, popup na PC i `/display` na telefonie
pokazują kroki z tego, co je wpycha (Fit dopóki żyje, potem most z
telefonu), plus tętno, jeśli źródło je ma. Gdy Google wyłączy Fit,
zmieniasz konfigurację, nie kod.

## 5. Zyski / Wady / Ryzyka

- **Zysk**: aplikacja przestaje zależeć od API z datą śmierci; pierwszy
  kanał zwrotny (głos → aplikacja) i pierwszy kanał na nadgarstek; wszystko
  lokalnie, bez chmury; kontrakt gotowy pod oficjalną apkę Android.
- **Wada**: głos wchodzi przez telefon, nie z nadgarstka — bo inaczej się
  nie da; jedno urządzenie więcej w pętli.
- **Ryzyko średnie**: Web Speech może być zablokowane na `http://` (plan
  ma ścieżkę główną bez niego — D5); tętno z Watch 4 w Google Fit jest
  niepewne (T01 sprawdza, pole jest opcjonalne); pierwsze POST-y na `:3390`
  to nowa powierzchnia — token, limity rozmiaru, walidacja schematu, brak
  interpolacji transkryptu gdziekolwiek.

## 6. Punkty i decyzja

**51 pkt, High** (bez tego kroki znikają w Q4 2026). Zależy od E020
(scalone). Fale: W0 spike 2 · W1 18 (3 równoległe) · W2 13 (2) · W3 13 (2)
· W4 3 · W5 2. Pełny AO — trzy fale z realnym fan-outem.

Decyzje, które podjąłem za Ciebie i które możesz odwrócić jednym zdaniem:

| # | Decyzja | Cena odrzucenia |
|---|---|---|
| D1 | telefon jest mostem, zegarek podglądem; firmware odrzucony | brak alternatywy pod oficjalnym SDK |
| D4 | wszystko lokalne w otwartej apce; AI-odpowiedź BYOK (własny klucz); hostowane AI i chmura zostają Pro | jeśli chcesz AI-odpowiedź tylko w Pro — T08 dostaje flagę licencyjną zamiast klucza, reszta bez zmian |
| D5 | dyktowanie klawiatury jako ścieżka główna, Web Speech jako dodatek | jeśli T01 pokaże, że Web Speech działa na Twoim telefonie po http, dodatek staje się domyślny — zero zmian w planie |
| D6 | Google Fit zostaje z podpowiedzią o końcu, nie jest usuwany | usunięcie to jedna linia po tym, jak most z telefonu działa |
| E022 | apka Android z Health Connect jako osobny epik, nie część tego | jeśli chcesz ją TERAZ — dopisuję do tego epiku ~21 pkt i inny toolchain |

Potrzebne: `go` na E021 (nowa sesja bierze [`HANDOFF.md`](HANDOFF.md));
T01 wymaga Twojego telefonu w ręku na ~15 minut. Pełny plan:
[`PLAN.md`](PLAN.md).
