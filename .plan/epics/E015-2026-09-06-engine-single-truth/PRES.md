# E015 — Jedna prawda o liczniku siedzenia (prezentacja)

## TLDR

Popup pokazuje licznik, który zeruje się przy każdym wstaniu, choć silnik
od marca odejmuje kredyt za przerwy proporcjonalnie. To błąd okablowania:
UI czyta drugie, nieskredytowane pole. Proponuję usunąć to pole z systemu
typów, żeby czwarty raz nie dało się go użyć. 21 punktów, jedna fala.

## 1. Kontekst

Silnik liczy dwa liczniki siedzenia: `sitting_seconds` (po kredycie
przerw, zasila overlay i powiadomienia) i `current_session_secs` (od
ostatniego siadu, zawsze od zera). Timer i pasek w popupie czytają ten
drugi.

## 2. Problem

Wstajesz na 2 minuty, siadasz, timer pokazuje 0:00, a kolor tego samego
widgetu mówi „jesteś w 80 % limitu". Logika kredytu była przepisywana 9
razy w marcu 2026 i za każdym razem ten sam szew (dwa liczniki, brak
właściciela) zostawał otwarty. Test w repo asertuje reset jako poprawny.

## 3. Rozwiązanie

Usunąć drugi licznik z Rusta i TS, przekierować popup na pole
skredytowane, odwrócić test, dodać test na realnej ścieżce i test dryfu
typów Rust→TS. Przy okazji: PostureBalance porównuje surowe z surowym,
a `break_credit` trafia do bazy, żeby Analyst nie zgadywał.

## 4. Po zmianie

Timer, pasek, kolor, overlay i powiadomienia liczą z jednego pola. Twoje
2 minuty w kuchni odejmują 6 minut siedzenia (mnożnik 3.0, decyzja D1).

## 5. Zyski / Wady / Ryzyka

- Zysk: skarga zamknięta u źródła; kompilator pilnuje reguły.
- Wada: dotyka DTO, typów TS, kilku testów, trzech dokumentów.
- Ryzyko: średnie; strangler zamiast przepisania, siatka 740 testów zostaje.

## 6. Punkty i decyzja

21 pkt, High. Decyzje D1 (mnożnik 3.0) i D2 (usunąć pole) mają rekomendacje
w [`PLAN.md`](PLAN.md); wdrożenie stosuje je, chyba że je zmienisz.
Decyzja potrzebna: `go` na E015 po E016 (higiena planów, 3 pkt).
