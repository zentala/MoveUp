# Security Review Report — E022 Cross-Device Phone Relay

## TLDR

Przejrzano cały podsystem relay (Cloudflare Worker + Durable Objects + D1) oraz
stronę desktopową (Rust) i frontend (TS). Architektura jest starannie
zaprojektowana pod kątem bezpieczeństwa: hashe SHA-256 zamiast plaintextów,
porównania rzeczywiście stałoczasowe, allowlista komend egzekwowana identycznie
po obu stronach, parametryzowane zapytania D1, brak sekretów w URL/logach, TLS
z rustls+webpki. Znaleziono **0 Critical, 1 High, 3 Medium, 4 Low**. Żadne ze
znalezisk nie jest banalnym exploitem gotowym do użycia od ręki, ale High i
część Medium wymagały zamknięcia przed publicznym uruchomieniem
`relay.desk.zentala.io`.

## Findings

### High

1. **Brak wymuszania wygaśnięcia/odwołania licencji na już otwartym
   połączeniu** — `relay/src/room/desk-room.ts`, `relay/src/room/sweep.ts`.
   `verifyToken` (`relay/src/auth/tokens.ts:124-160`) sprawdza
   `isActive(license, Date.now())` wyłącznie w momencie `hello`. `sweep()`
   obsługuje tylko idle-timeout (60s), nie licencję. Desk pinguje co 25s, więc
   połączenie z wygasłą licencją mogło żyć w nieskończoność.
   - **Status: NAPRAWIONE** — `alarm()` teraz re-waliduje `isActive` co cykl i
     zamyka połączenie kodem `4402` przy wygaśnięciu, analogicznie do
     istniejącego `RPC.revoke`.

### Medium

2. **Replay komend w 30-sekundowym oknie bez deduplikacji nonce/id** —
   `src-tauri/src/relay_commands.rs:107-116` (`admit`) sprawdzał tylko
   `|now_ms - ts| <= 30_000`, bez śledzenia wykonanych `id`. Model zagrożeń
   projektu (ADR 023) zakłada, że relay jest niezaufany — skompromitowany
   relay mógł dwukrotnie przesłać ten sam `command` frame w oknie 30s.
   - **Status: NAPRAWIONE** — krótkotrwały zbiór (TTL 30s) ostatnio wykonanych
     `env.id` po stronie desktopa, duplikaty odrzucane jako `replayed_command`.

3. **Brak automatycznego wymuszania TLS dla połączenia desk→relay** —
   `src-tauri/src/relay_client.rs:82-90` (`ws_url`) akceptuje dowolny schemat
   z Settings; `ws://` zamiast `wss://` wysłałoby token przez czysty TCP.
   Domyślny URL jest bezpieczny (`https://relay.desk.zentala.io`), ale nic nie
   blokuje ręcznego/błędnego ustawienia `ws://`.
   - **Status: BACKLOG** — niskie ryzyko operacyjne, nie blokuje deployu.

4. **TOCTOU na limicie widzów (`max_viewers`)** —
   `relay/src/http/pairing.ts:61-65` sprawdza limit przed zużyciem kodu
   parowania bez serializacji dwóch równoczesnych żądań. Może przekroczyć
   `max_viewers` o jeden.
   - **Status: BACKLOG** — ograniczenie biznesowe/licencyjne, nie luka
     poufności danych.

### Low

5. **Klucz licencyjny bez maskowania w UI** —
   `src/components/settings/RemoteSection.tsx:52-60`, pole bez
   `type="password"`. — **BACKLOG**.
6. **`clientIp` fallback do wspólnego kubełka `"unknown"`** —
   `relay/src/http/rate-limit.ts:53-54`, teoretyczne na Cloudflare Workers
   (nagłówek zawsze ustawiany przez edge). — **BACKLOG**.
7. **Brak twardego limitu rozmiaru ramki WS w `DeskRoom`** —
   `relay/src/room/desk-room.ts`/`messages.ts`, REST już ma limit 2 KiB, WS
   nie. — **BACKLOG**.
8. **Niespójne kody błędów `register` (nieistniejący vs wygasły klucz)** —
   `relay/src/http/desks.ts:56-64`, minimalne user enumeration na kluczach o
   256-bit entropii. — **BACKLOG**, kosmetyczne.

## Co uznano za solidne (bez zastrzeżeń)

- Tokeny: 256-bit entropia, SHA-256 at rest, stałoczasowe porównania, nigdy
  w URL.
- Allowlist komend egzekwowana identycznie w TS i Rust, w tym walidacja
  `switch_profile.name` blokująca path traversal.
- WS handshake: token weryfikowany PRZED dopuszczeniem do logiki pokoju;
  jeden Durable Object na `desk_id` daje twardą izolację między biurkami.
- D1: wszystkie zapytania parametryzowane, brak SQL injection.
- Pairing: kod 8 znaków, TTL 5 min, blokada po 10 błędnych próbach/15 min,
  dodatkowy IP rate-limit.
- LAN inlet (`:3390`): brak tokenu = zawsze `503`.
- TLS: `rustls-tls-webpki-roots`, brak wyłączonej weryfikacji certyfikatu.
- RPC wewnętrzny (`/__room/*`) nieosiągalny z zewnątrz.

## Verdict

**Po naprawie High #1 i Medium #2 (ta sesja, 2026-09-07): bezpiecznie
deployować.** Medium #3, #4 i wszystkie Low idą do `.plan/BACKLOG.md` do
naprawy iteracyjnej po starcie — nie blokują deployu.
