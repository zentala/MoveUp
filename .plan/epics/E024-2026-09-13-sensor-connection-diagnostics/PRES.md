# E024 — Diagnostyka połączenia z czujnikiem (prezentacja)

## TLDR

Gdy czujnik nie działa, apka mówi tylko „brak sensora", choć przyczyny są trzy
i każda ma inną naprawę. Epik uczy apkę je nazywać i — zgodnie z warunkiem
Pawła — każdą diagnozę pokrywa testami integracyjnymi na emulatorze czujnika.
6 zadań, 18 punktów, wykonuje orkiestrator AO (powyżej progu 13).

## Kontekst

Czujnik to XIAO ESP32-C3 na USB. Apka co 10 s skanuje porty COM, wysyła
`PING` i czeka na `DEVICE: zntl-desk-sensor v1`. Ta płytka jest wrażliwa na
kable (CLAUDE.md → Hardware).

## Problem

| Przyczyna | Co naprawdę się dzieje | Co widzi użytkownik dziś |
|---|---|---|
| Brak portu COM | kabel tylko do ładowania, brak zasilania (2026-09-05) | „brak sensora" |
| Porty są, żaden nie odpowiada | zły firmware, port zajęty | „brak sensora" |
| Połączenie miga | cienki kabel, spadek napięcia, dźwięki podłączania w pętli (2026-09-06) | nic — każde podłączenie wygląda normalnie |

Dziś tej ścieżki nie da się przetestować: pętla woła prawdziwy port szeregowy,
a istniejący emulator w `tests/emulator/` tylko formatuje napisy.

## Rozwiązanie

| Zadanie | Co | Pkt |
|---|---|---|
| T01 | „Szew": port szeregowy za interfejsem, prawdziwa implementacja + emulator | 5 |
| T02 | Komenda debug `emulate_serial_scenario`, scenariusze w testach TS | 3 |
| T03 | Rozróżnienie: brak portów / brak dopasowania / błąd wyliczania | 3 |
| T04 | Wykrywanie migania (3 połączenia < 5 s w 60 s) → ostrzeżenie w tray i Debug | 5 |
| T05 | Dokumentacja | 1 |
| T06 | Weryfikacja na uruchomionej apce | 1 |

## Po zmianie

`events.log`: `DEVICE scan ports=0`, `DEVICE scan ports=2 [COM3, COM5] none
matched`, `DEVICE unstable short_connections=3 window=60s`. W tray: „Niestabilne
połączenie z czujnikiem — spróbuj innego kabla albo portu bez huba".

**Zyski:** diagnoza z logu zamiast zgadywania; emulator zostaje na każdy
kolejny błąd portu szeregowego. **Wady:** 3 nowe moduły Rust. **Ryzyka:**
przeniesienie kodu portu za interfejs może zmienić zachowanie z prawdziwym
czujnikiem — pilnują tego istniejące testy `serial_*` i kryterium 6.

Odrzucone: logi wprost w `serial.rs` (nie spełnia warunku testów na
emulatorze) i wirtualne porty com0com (sterownik jądra, nie odtworzy braku
portów ani spadku napięcia). Szczegóły: [`PLAN.md`](PLAN.md).

**Decyzja potrzebna:** żadna — zakres i warunek testów zlecone 2026-09-13.
