# Plan wdrożenia

## Faza 0 — sprawdzenie założeń

- [ ] Zbierz 20 pojęć z jednej rzeczywistej domeny i 3–5 ikon referencyjnych z prawem do użycia.
- [ ] Ustal styl, rozmiar docelowy i kryteria rozpoznawalności.
- [ ] Zrób spike: jeden IconSpec → SVG → PNG; oceń serializer prosty vs Paper.js przy krzywej.

## Faza 1 — rdzeń

- [ ] Pakiet `schema`: Zod i typy, profil, wersja; diagnostyka błędów.
- [ ] Pakiet `renderer`: osiem prymitywów i grupa, stała precyzja, safe allowlist.
- [ ] CLI: `validate`, `render`, czytelne kody wyjścia i przykłady.
- [ ] Testy: prymitywy, łuk 360°, granice stroke, złe współrzędne, nieznane pole, powtarzalność, odrzucenie niebezpiecznych pól.

## Faza 2 — pętla AI

- [ ] Adapter LLM z ustrukturyzowanym wyjściem i retry walidacji.
- [ ] `generate` z limitem kosztu, czasu i liczbą iteracji.
- [ ] `revise` z wizualną analizą, zapisem przyczyn zmian i historią wersji.
- [ ] `sheet` i artefakty podglądu w 24/512 px oraz na dwóch tłach.

## Faza 3 — ocena

- [ ] Wygeneruj 20 ikon bez ręcznego poprawiania surowego SVG.
- [ ] Ręcznie oceń rozpoznawalność i spójność bez znajomości promptów.
- [ ] Zapisz liczbę prób, koszt, czas, typy błędów, wskaźnik akceptacji.
- [ ] Zdecyduj, czy potrzebne są makra semantyczne, edytor i integracja Figma.

Koniec MVP: kryteria w `PRD.md` są spełnione albo wynik eksperymentu pokazuje, że trzeba zmienić podejście; w obu przypadkach zachowaj dane i wnioski.
