# E025 — Zamykanie alertu bez czujnika + porównanie z wczoraj (prezentacja)

## TLDR

Audyt starych epików znalazł dwie rzeczy odhaczone jako zrobione, których w
działającej apce nie ma. 4 zadania, 10 punktów, wykonują subagenci.

| # | Kontekst | Problem | Rozwiązanie | Pkt |
|---|---|---|---|---|
| 1 | Po 40 min siedzenia wyskakuje okienko alertu z przyciskiem Dismiss | Bez podłączonego czujnika kliknięcie nic nie robi — apka sprawdza je tylko przy nowym odczycie wysokości. Po podłączeniu czujnika okienko wraca od razu, bez drzemki | Okienko wysyła zdarzenie „zamknięte" w chwili kliknięcia; nie zależy od czujnika | 3 |
| 2 | E004-T04 obiecywał test całego przepływu alertu | Testu nigdy nie napisano | `tests/integration/alert_flow.test.ts`: limit → alert → zamknięcie → drzemka → powrót | 3 |
| 3 | W marcu był widok z strzałką ↑/↓ „dziś vs wczoraj" | Przebudowa na widżety go odpięła, potem skasowano go jako martwy kod. Apka **nadal co 10 s liczy i pobiera** wczorajsze sumy — i nigdzie ich nie pokazuje | Nowy wskaźnik w pasku KPI: „vs wczoraj −25 min", kolor zielony/żółty/czerwony, dymek z opisem; ten sam na telefonie | 3 |
| 4 | — | — | Weryfikacja, dokumentacja, poprawienie `DONE.md` | 1 |

**Co jest dziś na froncie z danych dziennych:** pasek KPI pokazuje wskaźniki
liczone w Rust (procent stania, tempo zmian pozycji, przerwy godzinowe,
najdłuższa sesja) plus zdrowie. Dane z `get_today_summary` (sumy dnia,
wczoraj, liczba zmian pozycji, lista sesji) trafiają do stanu Reacta i nie
czyta ich żaden komponent.

**Zyski:** alert przestaje wracać bez drzemki; wraca motywujące porównanie z
wczoraj. **Wady:** kolejny badge w pasku KPI. **Ryzyka:** zmiana wątku okienka
WinAPI (przekazanie uchwytu apki) — pilnuje tego test T01.

Szczegóły i odrzucone warianty: [`PLAN.md`](PLAN.md).
