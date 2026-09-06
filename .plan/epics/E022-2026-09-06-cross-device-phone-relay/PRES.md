# E022 — Telefon jako pilot biurka spoza domu (prezentacja)

## TLDR

Dziś telefon pokazuje biurko tylko na tym samym Wi-Fi, bez żadnego logowania
i bez możliwości zmiany czegokolwiek. Ten epik dodaje **relay w chmurze**
(Cloudflare, jeden „pokój" na biurko, w pamięci tylko ostatni stan — żadna
historia nie wychodzi z komputera), **parowanie kodem z ekranu komputera**
(pierwsza autoryzacja w tej aplikacji: bez kont, bez haseł, bez e-maila,
z listą sparowanych telefonów i przyciskiem „odłącz") oraz **trzy komendy z
telefonu**: zamknij alert, zmień limity siedzenia/stania, przełącz profil.
Tryb LAN zostaje jako wersja darmowa, tylko do podglądu. 16 zadań, 111 pkt,
pełny Agent Orchestrator (trzy równoległe łańcuchy: relay / desktop / telefon).
Przegląd bezpieczeństwa jest osobnym zadaniem i nie ma drogi obejścia.
Decyzja potrzebna: `go` na plan albo zmiana którejś z pięciu decyzji niżej.

## 1. Kontekst

Aplikacja ma wbudowany serwer HTTP+WebSocket na porcie 3390
(`remote_server.rs`). Stary telefon z Fully Kiosk Browser wchodzi na
`http://<IP-komputera>:3390/display` i widzi ten sam React co popup na
desktopie — te same dane, ~1 zdarzenie/s. To ADR 001 z marca (E009). Tabela
tierów w wizji Pro mówi: **Free = podgląd w LAN, Pro = relay w chmurze**, a
stary dokument wizji rozpisał to jako cztery zadania z Cloudflare Durable
Objects w roli relayu.

## 2. Problem

Cztery braki, wszystkie potwierdzone w kodzie:

- **Tylko LAN.** Poza domem telefon nic nie pokazuje. Instrukcja każe szukać
  IP w `ipconfig` i otwierać port w zaporze.
- **Zero autoryzacji.** `ws_handler` sprawdza tylko licznik klientów. Każdy w
  tej samej sieci (współlokator, gość, biuro) widzi Twoje dane ergonomiczne.
- **Jednokierunkowo.** Pętla odbiorcza ignoruje wszystko, co przyjdzie z
  telefonu (`Some(Ok(_)) => {}`); `setSitLimit` po stronie telefonu to
  `console.warn`.
- **Bez stanu, gdy komputer śpi.** Telefon pokazuje „Reconnecting..." w
  nieskończoność, bo nie ma skąd wziąć ostatniego stanu.

Do tego aplikacja nie ma dziś **żadnego** systemu kont ani tokenów — cokolwiek
tu wybierzemy, będzie pierwszą powierzchnią autoryzacji tej appki i wzorcem dla
kolejnych funkcji chmurowych (historia, coaching).

## 3. Rozwiązanie — pięć decyzji

| # | Decyzja | Dlaczego tak, a nie inaczej |
|---|---|---|
| D1 | **Relay na Cloudflare Workers + Durable Objects (+ D1 na hashe tokenów)** | To usługa dla PŁACĄCEGO klienta. `my-severs.md` rezerwuje self-hosting dla własnych usług; `pve01` za NAT-em z autosleepem nie może być zależnością cudzego telefonu. Cloudflare już używasz (Pages, `desk.zentala.io`), wizja E010 i tak wybiera ten stos pod resztę Pro. Protokół to zwykły JSON po WebSocket z fixture'ami jako kontraktem — self-hostowany relay może powstać później jako druga implementacja tego samego kontraktu |
| D2 | **Parowanie kodem → długowieczne tokeny urządzeń, bez kont** | Komputer rejestruje się raz kluczem licencyjnym i dostaje `desk_token`; telefon wpisuje (albo skanuje QR) 8-znakowy kod ważny 5 minut i dostaje `viewer_token`. Relay trzyma tylko SHA-256 tokenów. Odłączanie z listy w Ustawieniach. Zero PII — zgodne z „privacy-first". Konta (magic link) to 5× więcej roboty i pierwszy e-mail w bazie; da się je dołożyć później jako „to, co posiada klucze licencyjne" |
| D3 | **Trzy komendy: `ack_alert`, `set_limits`, `switch_profile`** | Każda mapuje się 1:1 na funkcję, która już istnieje i już waliduje wejście (dismiss popupu, `set_session_limit`/`set_stand_limit` z clampami, `switch_*_profile`). Kalibracja z telefonu nie ma sensu (nie ma Cię przy biurku), a ogólny zapis ustawień z sieci to dokładnie to, co odrzuci przegląd bezpieczeństwa |
| D4 | **LAN zostaje: darmowy, tylko podgląd, przełącznik w Ustawieniach, ta sama koperta wiadomości** | Nie zabieramy działającej darmowej funkcji i nie robimy z lokalnego dashboardu zależności od chmury. Jeden klient React obsługuje oba transporty. Luka „brak auth w LAN" jest zwężona (można wyłączyć), nie zamknięta — parowanie w LAN idzie do backlogu |
| D5 | **Adres relayu: `relay.desk.zentala.io`** | Subdomena domeny produktu (`desk.zentala.io/CNAME`). Stała w kodzie, nadpisywalna w Ustawieniach — staging i self-host bez przebudowy. Wdrożenie idzie przez `consent-broker` |

Przepływ dla użytkownika: Ustawienia → Zdalny → „Sparuj telefon" → kod + QR
na ekranie → telefon skanuje → za 10 sekund widzi biurko i ma trzy przyciski.
Komputer łączy się z relayem **wyłącznie wychodząco** — koniec z zaporą i
przekierowaniem portów.

## 4. Po zmianie

- Telefon na LTE, w kawiarni: widzi stan biurka poniżej sekundy po zmianie.
- Komputer wyłączony: telefon pokazuje ostatni stan z banerem „biurko offline
  od 14:32" zamiast wiecznego „Reconnecting...".
- Z telefonu: zamknięcie alertu, zmiana limitu, zmiana profilu — z tymi
  samymi granicami co na desktopie, każda komenda w `events.log`.
- Zgubiony telefon: „Odłącz" w Ustawieniach, gniazdo zamyka się w sekundę,
  ponowne połączenie odrzucone.
- Wygasła licencja: Ustawienia mówią to słowami; aplikacja nie próbuje po
  cichu w kółko.
- Sąsiad na tym samym Wi-Fi: dalej widzi podgląd LAN, chyba że wyłączysz
  przełącznik — to jedyna rzecz, której ten epik nie domyka (backlog).

## 5. Zyski / Wady / Ryzyka

- **Zyski:** pierwsza realna funkcja Pro z tabeli tierów; wzorzec auth, na
  którym stanie historia/sync i coaching; relay bez danych — argument
  marketingowy zgodny z `PRIVACY.md`; zero konfiguracji sieci u użytkownika.
- **Wady:** relay w TypeScript obok aplikacji w Rust (dwa języki, jeden
  kontrakt — pilnują tego fixture'y testowane po obu stronach); nowe
  zależności (`tokio-tungstenite`, `keyring`, `zod`, `qrcode`, `wrangler`);
  koszt Cloudflare rośnie z liczbą aktywnych biurek (hibernacja WS trzyma
  bezczynne biurka za darmo); klucze licencyjne do czasu Stripe'a mintuje
  skrypt, nie sklep.
- **Ryzyka:** średnie technicznie (nowe runtime DO, pierwsze gniazdo
  wychodzące w Rust), **wysokie bezpieczeństwo z definicji** — dlatego T13
  (model zagrożeń + `security-reviewer` + `review-loop` + checklista z
  zapisaną obserwacją na każdy wiersz) jest zadaniem, nie kroką w commicie, i
  nie ma pola `Pipeline-skipped`. Ryzyko produktowe: 111 pkt to dużo; plan
  nazywa **cięcie minimalne** (79 pkt: podgląd zewsząd, parowanie, odłączanie —
  bez trzech komend), gdyby trzeba było wydać w dwóch krokach.

## 6. Punkty i decyzja

**111 pkt, Importance High** (jedyna funkcja z tabeli Pro, która nie
potrzebuje E010 w całości, a domyka realną dziurę bezpieczeństwa w LAN).
Zależy od E017/E018/E019 (scalone). Przez pełny AO: W0=5, W1=32 (4 zadania
równolegle), W2=29 (3), W3=18 (3), potem W4–W6 sekwencyjnie; T13/T15/T16 poza
AO (ręczne z natury). Wersja: 0.7.0 albo następny wolny minor.

Decyzja potrzebna: `go` na [`HANDOFF.md`](HANDOFF.md) — plan stosuje D1–D5
bez pytania, chyba że którąś zmienisz w [`PLAN.md`](PLAN.md). Jedyna rzecz,
którą i tak Ci pokażę przed zrobieniem: makiety sekcji Ustawienia → Zdalny
i ekranu parowania (T09/T10, reguła `ux-design-flow.md`) oraz bramka zgody na
wdrożenie relayu (T16).
