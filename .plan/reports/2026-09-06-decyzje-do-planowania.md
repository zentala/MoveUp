# Decyzje przed sesją planistyczną — MoveUp, 2026-09-06

## TLDR

Trzy epiki (E018, E019, E020) i połowa E014 są dziś zamknięte i wypchnięte
na `origin/main`. Zostały cztery realne pytania, na które potrzebna jest
Twoja decyzja, zanim ktokolwiek napisze kolejny `PLAN.md`. Odpowiedz na
nie (wystarczy "1a, 2b..." w czacie) — zapiszę je i przygotuję `HANDOFF.md`
dla sesji planistycznej z najlepszym modelem.

---

## Pytanie 1 — E015-T05: zaakceptować dowód testowy czy naprawić lukę wizualną?

**Kontekst.** E015 naprawiło błąd, że popup pokazywał zły licznik czasu
siedzenia (resetował się przy każdym wstaniu, zamiast liczyć z kredytem za
przerwę). Poprawka jest udowodniona testem Rust (`session_tests_timers_live.rs`),
ale nigdy nie zobaczono jej na żywo w przeglądarce — dev-mode ma dwie
niezależne luki: brak proxy Vite dla trasy `/display`, i tryb mock nie
steruje prawdziwym silnikiem sesji.

**Opcje:**
- **(1a) Zaakceptuj dowód testowy jako wystarczający.** Zamknij E015-T05
  formalnie z notatką "zweryfikowane testem, nie renderem". Zero dodatkowej
  roboty.
- **(1b) Napraw dev-mode gap, potem zrób prawdziwy przegląd w przeglądarce.**
  Szacunek: ~9 punktów (3 pkt proxy Vite + 1 pkt mock→silnik + 5 pkt sam
  przegląd i ew. poprawki).

**Moja rekomendacja: 1a.** Zachowanie jest odtworzone scenariuszem
testowym (sit→stand→sit z kredytem), a naprawienie dev-mode to inwestycja
w infrastrukturę testową, nie w produkt — wart zrobienia kiedyś, ale nie
pilny, i nie blokuje niczego innego.

Źródła: [`.plan/STATE.md:12-17`](../STATE.md), [`.plan/BACKLOG.md` sekcja "Dev-mode remote display"](../BACKLOG.md), [`epics/E015-2026-09-06-engine-single-truth/HANDOFF.md`](../epics/E015-2026-09-06-engine-single-truth/HANDOFF.md)

---

## Pytanie 2 — Które z dwóch funkcji Pro planować teraz?

**Kontekst.** Dokument wizji biznesowej z marca 2026 wymienia dwie funkcje
płatnego tieru, które dziś istnieją tylko jako akapit koncepcyjny — zero
researchu API/urządzeń, zero `PLAN.md`.

| Funkcja | Co to jest | Szacunek | Ważność wg wizji |
|---|---|---|---|
| **(2a) Smartwatch integration** | Odczyt kroków/tętna z zegarka (Apple Watch / Wear OS / Garmin) jako alternatywa dla Google Fit | ~21+ pkt (nieznany zakres API) | Pro feature #3 |
| **(2b) Cross-device & phone relay** | Podgląd/kontrola aplikacji z telefonu przez sieć lokalną, rozszerzenie dzisiejszego remote display | ~13+ pkt (nieznany zakres) | Pro feature #4 |

**Opcje:**
- Zaplanuj **2a** teraz
- Zaplanuj **2b** teraz
- Zaplanuj **obie**
- **Żadnej teraz** — zostają w wizji jako "post-product-market-fit", zgodnie
  z oryginalnym dokumentem wizji, dopóki nie ma potwierdzonego popytu

**Moja rekomendacja: żadnej teraz.** Oba dokumenty wizji same mówią, że to
funkcje na etap po walidacji popytu (Kickstarter/pre-ordery), a MoveUp nie
ma dziś ani jednego zewnętrznego użytkownika. Planowanie 21+13 punktów
funkcji bez potwierdzonego zapotrzebowania to praca, która może się okazać
niepotrzebna. Warto natomiast utrzymać `remote display` (już istnieje) jako
fundament pod 2b, gdyby popyt się pojawił.

Źródła: [`.plan/vision/2026-03-25-premium-tier-definition.md:113,130`](../vision/2026-03-25-premium-tier-definition.md), [`.plan/vision/2026-03-24-business-vision.md`](../vision/2026-03-24-business-vision.md)

---

## Pytanie 3 — Co zrobić z resztą drobnego długu technicznego?

**Kontekst.** W backlogu leży kilka drobnych, nieplanowanych pozycji, żadna
nie jest pilna, żadna nie blokuje niczego:

| Pozycja | Punkty | Źródło |
|---|---|---|
| `cargo-clippy` nie zainstalowany dla tego toolchaina | 1 | [`BACKLOG.md:479`](../BACKLOG.md) |
| 6 reguł ESLint obniżonych do `warn` (E018-T04), do podciągnięcia do `error` | 3+5+2+2+1+1 = 14 | [`BACKLOG.md`](../BACKLOG.md), sekcja "ESLint baseline downgrades" |
| Próg pokrycia testów obniżony do 73/72 (E018-T05, decyzja D4), do podniesienia z powrotem | 5 | [`BACKLOG.md`](../BACKLOG.md), sekcja "E018-T05 lowered coverage thresholds" |
| "Smart Desk" branding w `ShareStats.tsx` (stare nazewnictwo) | 1 | [`BACKLOG.md:513`](../BACKLOG.md), [`src/components/ShareStats.tsx`](../src/components/ShareStats.tsx) |

**Razem: ~21 punktów**, wszystko niskiego/średniego priorytetu.

**Opcje:**
- **(3a) Zbierz to w jeden mały epik "engine/frontend debt"** i puść przez AO
  przy okazji następnej sesji.
- **(3b) Zostaw w backlogu**, poprawiane opportunistycznie przy okazji innych
  epików (tak jak dziś poprawiałem błędy znalezione po drodze).

**Moja rekomendacja: 3b.** To są klasyczne "przy okazji" poprawki — żadna
nie uzasadnia osobnego epiku i osobnej sesji AO (koszt orkiestracji
przewyższa wartość), a każda i tak zostanie zamieciona przy następnym
epiku, który dotknie te same pliki.

---

## Pytanie 4 — Code signing

Odłożone przez Ciebie dziś ("odkładamy") — zostaje otwarte w
[`BACKLOG.md`](../BACKLOG.md), bez terminu, bez dalszych pytań.

---

## Co NIE wymaga decyzji (już wiadomo, co robić)

- **E014, fale 3-4** — czekają na `pm3-mcp`, nie jest to decyzja tego
  repo. Nic do zaplanowania tutaj.
- Reszta backlogu (~60 pozycji) to pojedyncze bugi/poprawki punktowe, nie
  epiki — nie wymagają sesji planistycznej.

## Ile to będzie punktów, jeśli zaplanujemy wszystko na "tak"

| Decyzja | Jeśli "zrób to" | Jeśli "nie teraz" |
|---|---|---|
| E015-T05 | ~9 pkt (fix + przegląd) | 0 pkt (zamknięcie formalne) |
| Smartwatch (2a) | ~21+ pkt | 0 pkt |
| Cross-device (2b) | ~13+ pkt | 0 pkt |
| Dług techniczny (3a) | ~21 pkt | 0 pkt (opportunistycznie) |

**Przy moich rekomendacjach (1a, żadna z 2, 3b): 0 punktów do zaplanowania
teraz** — repo jest faktycznie w pełni domknięte na dziś, poza czekaniem na
`pm3-mcp`. Jeśli zdecydujesz się na obie funkcje Pro: **~34-43+ punktów**
do rozpisania w nowej sesji planistycznej.
