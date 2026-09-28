# PRD — IconForge

## Problem

Agent piszący ręcznie `path d` w SVG często tworzy nieczytelne, niespójne i trudne do korekty ikony. Potrzebny jest ograniczony język kompozycji, który agent umie poprawiać po obejrzeniu renderu.

## Użytkownicy i scenariusze

- Projektant lub developer opisuje ikonę słowami i dostaje SVG zgodne z istniejącym zestawem.
- Agent generuje serię ikon dla produktu i wykrywa odstępstwa od ustalonego stylu.
- Developer może edytować IconSpec ręcznie lub ponownie wygenerować SVG bez dostępu do modelu.

## Rezultat MVP

CLI przyjmuje prompt, profil stylu i opcjonalne 3–5 przykładów. Zwraca `spec.json`, `icon.svg`, podgląd PNG i raport walidacji. Potrafi przetworzyć jeden IconSpec bez połączenia z LLM. Zachowuje edytowalne linie i ścieżki ze stroke zamiast automatycznie zamieniać całą ikonę w kontury wypełnione. Pozwala wskazać istniejący plik JSON do rewizji.

## Wymagania funkcjonalne

1. Deterministyczny renderer SVG dla `line`, `polyline`, `circle`, `ellipse`, `rect`, `roundedRect`, `arc`, `curve` i `group`; bez SVG/XML przekazanego przez model.
2. Walidacja struktury i geometrii: skończone współrzędne, dopuszczalna liczba elementów, margines, rozmiar, ograniczony poziom zagnieżdżenia, bez negatywnych promieni.
3. Profil stylu kontroluje viewBox, stroke, cap/join, margines, siatkę, odstępy i dozwolone prymitywy. Model wybiera geometrię, profil narzuca styl.
4. Render PNG na ciemnym i jasnym tle oraz w rozmiarze docelowym, np. 24 px, i powiększonym 512 px.
5. Opcjonalny krok analizy wizualnej: agent opisuje konkretne defekty i zmienia wyłącznie IconSpec; limit iteracji i kosztów konfigurowalny.
6. Wygenerowanie wariantu `currentColor` oraz metadanych o promptach, wersji schematu, profilu i modelu; metadane nie muszą być osadzane w publicznym SVG.
7. CLI `validate`, `render`, `generate`, `revise`, `sheet`; arkusz porównawczy zestawu ikon.
8. Eksport bez skryptów, obcych referencji, `foreignObject`, obrazów osadzonych oraz nieautoryzowanych atrybutów.

## Poza MVP

Własny model, trening, figury semantyczne człowieka i mebli, rozbudowany edytor React, Figma, vectorizacja rastrowa, fonty ikonowe, animacje. Po sprawdzeniu MVP można dodać figury semantyczne, bez zmiany formatu prymitywów.

## Kryteria odbioru

- `render` daje ten sam znormalizowany SVG dla tego samego IconSpec i profilu.
- Poprawny przykład każdej figury daje SVG wyświetlające się w przeglądarce i ma test obrazu lub ręcznie zatwierdzony podgląd referencyjny.
- Niepoprawna geometria zwraca diagnostykę z lokalizacją elementu, nie uszkodzony SVG.
- Zestaw pilotażowy: 20 różnych pojęć z jednej domeny; co najmniej 16/20 zaakceptowanych po najwyżej 3 rundach, ocenionych ręcznie w 24 px przez osobę, która nie widzi promptu. To cel eksperymentu, nie obietnica skuteczności.
- 20 wyników jest stylistycznie porównywalnych na jednym arkuszu; odrzucenia są zapisywane z przyczyną.

## Pytania produktowe do rozstrzygnięcia po pilocie

Które pojęcia wymagają semantycznych szablonów? Jak duża jest poprawa po zastosowaniu przykładów? Czy vision-review daje mierzalną poprawę względem jednej generacji? Czy dany zestaw wymaga optycznego wyrównania, wykraczającego poza prostą siatkę?
