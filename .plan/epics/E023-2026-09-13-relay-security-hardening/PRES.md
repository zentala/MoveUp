# E023 — Utwardzenie relay (prezentacja)

## TLDR

Przegląd bezpieczeństwa E022 znalazł 8 problemów w relay (serwer pośredniczący
między biurkiem a telefonem). Dwa naprawiliśmy od razu, sześć odłożyliśmy, bo
nie blokowały wdrożenia. Relay nadal nie jest wdrożony (decyzja E022-D6), więc
teraz naprawa jest najtańsza: nie ma użytkowników ani migracji. 6 poprawek +
weryfikacja, 13 punktów, wykonują subagenci.

## Pozycje

| # | Kontekst | Problem | Rozwiązanie | Pkt / Ważność |
|---|---|---|---|---|
| F1 | Biurko łączy się z relay adresem z Ustawień | Adres `ws://` wysyła token biurka otwartym tekstem | `ws://` tylko dla localhost (tak działa `just relay-dev`), reszta musi być szyfrowana | 2 / Medium |
| F2 | Licencja ma limit sparowanych telefonów | Dwa równoczesne parowania przekraczają limit o jeden | Jedno warunkowe `INSERT` w bazie, przegrany dostaje 409 | 3 / Medium |
| F3 | Pole klucza licencji w Ustawieniach | Klucz widać na ekranie i w historii autouzupełniania | `type="password"` | 1 / Low |
| F4 | Limit zapytań liczony per adres IP | Nagłówek `X-Forwarded-For` ustawia klient; brak IP = wspólny kubełek dla wszystkich | Ufamy tylko `CF-Connecting-IP`; brak = błąd konfiguracji | 2 / Low |
| F5 | Ramki WebSocket w pokoju biurka | Brak limitu rozmiaru przed parsowaniem JSON | Limit 16 KiB, większa ramka zamyka połączenie | 2 / Low |
| F6 | Rejestracja biurka kluczem licencji | Różne błędy dla „nie ma klucza" i „wygasł" | Jeden, ten sam błąd | 1 / Low |
| T07 | — | — | Pełne testy + ponowny przegląd bezpieczeństwa | 2 |

**Zyski:** relay gotowy do wdrożenia bez znanych luk. **Wady:** nic nie
wdrażamy, więc zysk jest odroczony. **Ryzyka:** F4 może zepsuć lokalny
`relay-dev`, jeśli miniflare nie ustawia nagłówka — test to wyłapie.

Odrzucone: przeniesienie całej kontroli dostępu do Durable Object (za duże na
dwa Low) i reguły WAF Cloudflare (wymagają wdrożonej strefy). Szczegóły:
[`PLAN.md`](PLAN.md).

**Decyzja potrzebna:** żadna — zakres zlecony 2026-09-13.
