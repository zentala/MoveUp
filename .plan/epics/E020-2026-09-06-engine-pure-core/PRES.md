# E020 — Silnik: czysty rdzeń (prezentacja)

## TLDR

E015 naprawiło, które pole czyta popup. Ten epik naprawia PIĘĆ szwów pod
spodem: silnik czyta zegar sam z siebie, config jest kopiowany do stanu
(więc hot-reload profilu ergonomicznego po cichu nic nie robi — to
prawdziwy, nowo znaleziony błąd), persystencja to ręcznie duplikowany typ,
tray sam sobie dolicza to, co silnik już wie, a słowo „przerwa" znaczy
trzy różne rzeczy. 59 punktów, 7 fal, każda fala kończy się zielonym
`cargo test --lib`.

## 1. Kontekst

Po E015 popup pokazuje poprawny, skredytowany licznik. Ale silnik pod
spodem nadal ma pięć szwów, które przegląd `_review-2026-09-06/engine.md`
nazwał wprost: czyta `Utc::now()` w środku (testy backdatują czas i mają
nadzieję, że zegar się nie ruszy), sześć pól z profilu ergonomicznego jest
kopiowanych do stanu przy starcie i nigdy nie odświeżanych, plik
persystencji ręcznie duplikuje typ stanu, `tray_controller.rs` przelicza
na nowo to, co silnik już policzył, a „przerwa" oznacza trzy niepowiązane
koncepcje w trzech miejscach kodu.

## 2. Problem

Czytając kod pod ten epik znalazłem dodatkowy, nieudokumentowany błąd:
**hot-reload profilu ergonomicznego nic nie robi dla silnika sesji.**
`profile_reload.rs` podmienia profil tylko w `CommunicationPolicy` — sesja
trzyma swoją kopię sześciu liczb (próg przerwy, mnożnik kredytu, itd.) od
startu aplikacji i nigdy jej nie odświeża. Zmieniasz mnożnik w pliku JSON
profilu, aplikacja go ignoruje aż do restartu. Drugi znaleziony błąd:
dobowy reset porównuje datę UTC, a cała reszta aplikacji (baza, plik
stanu) liczy datę lokalną — dla Polski to kilka godzin przesunięcia.

## 3. Rozwiązanie

Osiem zadań w ścisłej kolejności (bo dotykają tych samych plików):
wstrzyknięcie zegara jako argumentu (+ naprawa daty lokalnej), wyjęcie
configu ze stanu (+ naprawa hot-reloadu), nazwanie trzech różnych
„przerw" osobno, jeden wersjonowany snapshot persystencji, silnik sam
liczy to, co dziś liczy tray na nowo, oraz dedykowany test tabelaryczny
łączący wszystkie scenariusze na realnej ścieżce DTO — odpowiedź na
werdykt przeglądu: „testy istnieją, ale testują złą rzecz".

## 4. Po zmianie

Zmieniasz mnożnik kredytu w profilu — działa od następnego ticka, bez
restartu. Dobowy reset odpala się o lokalnej północy. Jeden plik JSON
przeżywa restart aplikacji bez ręcznego duplikowania pól. `tray_controller.rs`
tylko wykonuje sygnały silnika — zgodnie z tym, co `CLAUDE.md` już dziś
deklaruje, ale kod jeszcze nie spełniał.

## 5. Zyski / Wady / Ryzyka

- Zysk: dwa realne, nieudokumentowane błędy naprawione przy okazji
  (hot-reload, strefa czasowa dobowego resetu); kompilator pilnuje reguł
  zamiast dokumentacji.
- Wada: dotyka niemal każdego pliku silnika sesji, więc pięć z ośmiu
  zadań musi iść po kolei, nie równolegle.
- Ryzyko: średnie. Strangler, nie przepisanie — każda fala kończy się
  zielonymi testami, żadna fala nie zostawia silnika w stanie pośrednim
  bez siatki bezpieczeństwa.

## 6. Punkty i decyzja

59 pkt, Medium (kod działa dziś dla użytkownika; to porządkowanie długu,
nie naprawa widocznej awarii). Zależy od E015 (musi być scalone jako
pierwsze) i sekwencyjnie po E017 wg rekomendacji z pełnego przeglądu.
Decyzja potrzebna: `go` na E020 po scaleniu E015 — plan stosuje D3
(naprawa strefy czasowej dobowego resetu) i numer ADR 015 bez pytania,
chyba że je zmienisz w [`PLAN.md`](PLAN.md).
