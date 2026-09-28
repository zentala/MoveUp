# Instrukcje dla agentów — IconForge

## Cel

Zaimplementuj projekt opisany w `PRD.md` i `ARCHITECTURE.md`. Dokumenty są specyfikacją projektu; jeżeli istniejący kod wymaga zmiany kontraktu, najpierw uaktualnij `ICON_SPEC.md` z uzasadnieniem.

## Kolejność pracy

1. Zbadaj repo i środowisko, skonfiguruj TypeScript oraz minimalny workspace.
2. Wdroż schema + profil + parser, potem renderer + CLI `validate`/`render`.
3. Dodaj podglądy, arkusz porównawczy i testy jakości.
4. Dopiero wtedy dodaj adapter modelu, generowanie, krytykę i rewizję.
5. Przeprowadź pilota na 20 pojęciach; zapisz wyniki i niepowodzenia.

## Zasady

- Nie generuj `path d` ręcznie w promptach do LLM. Renderer może oczywiście wyprodukować SVG `path` dla łuków i krzywych.
- Nie wykonuj kodu, HTML ani SVG otrzymanego od modelu. Nie udostępniaj kluczy API w artefaktach.
- Utrzymuj style w profilach, a opis semantyczny w promptach; kształty w IconSpec.
- Testuj własności geometrii oraz wyjście serializerów. Nie zatwierdzaj SVG tylko dlatego, że przechodzi walidację składni.
- Otwórz podgląd ikon w rozmiarze docelowym i na dwóch tłach. Zanotuj w raporcie defekty, a nie ogólną ocenę „ładne”.
- Zachowaj surowe dane wejściowe i wynikowe dla reprodukowalności; nie przechowuj treści tajnych.
- Wybory bibliotek potwierdź małym prototypem i sprawdzeniem licencji, zanim dodasz zależności.

## Prompty agenta ikon

Planner: „Zwróć wyłącznie IconSpec v1 zgodny ze schematem. Zachowaj istotę: {brief}. Użyj profilu {profile} i przykładów referencyjnych. Projekt ma być czytelny w 24 px. Używaj maksymalnie 12 figur, chyba że zadanie wymaga więcej. Nie dodawaj tekstu i obcych atrybutów.”

Reviewer: „Porównaj obraz 24 px i 512 px z briefem i zestawem referencyjnym. Podaj co najwyżej trzy obserwowalne defekty; dla każdego wskaż `shape.id` i proponowaną zmianę współrzędnych lub elementu. Nie twierdź, że problem istnieje, jeśli nie widzisz go w renderze.”

Reviser: „Na podstawie IconSpec v1, briefu i listy defektów zwróć pełny poprawiony IconSpec v1. Zachowaj istniejące identyfikatory niezmienionych figur. Nie modyfikuj profilu.”

## Raport końcowy implementacji

Podaj komendy uruchomienia, zrealizowane wymagania, wyniki testów, ocenę pilota, znane ograniczenia oraz przykłady `spec.json`, `icon.svg`, `preview.png`. Bez deklaracji sukcesu przed obejrzeniem renderów.
