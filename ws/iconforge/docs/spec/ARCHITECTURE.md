# Architektura

## Przepływ

`brief + przykłady + profil` → `planner LLM` → `IconSpec JSON` → `walidacja` → `kompilator` → `SVG` → `PNG` → `ocena wizualna` → `rewizja IconSpec` → `zatwierdzenie`.

Generator i recenzent mogą używać jednego modelu na początek. Kompilacja, walidacja i eksport są lokalne i deterministyczne. Żaden wynik modelu nie jest wykonywany jako kod.

## Moduły

```text
packages/
  schema/       # typy TS, Zod, profil, wersjonowanie
  renderer/     # prymitywy -> bezpieczne SVG, normalizacja
  quality/      # reguły geometrii i raporty
  agent/        # adapter LLM, prompt planner/reviewer, limity
  cli/          # validate/render/generate/revise/sheet
examples/       # profile, zaakceptowane ikony, dane pilota
```

## Wybory

- TypeScript + Zod: kontrakt danych i jawna walidacja. Przed użyciem sprawdzić wersję i zgodność strict structured outputs wybranego API.
- Renderer MVP: bezpośredni, mały serializer prymitywów. Do prostych figur Paper.js i Maker.js nie są konieczne. Paper.js można dodać jako adapter dla `smooth` i trudniejszych krzywych; Maker.js dla konstrukcji geometrycznych. Eksport obu nie rozwiązuje sam spójności stylu.
- `@resvg/resvg-js` lub kompatybilny renderer do PNG po ocenie działania w docelowym środowisku. Pipeline musi działać offline w trybie `render`.
- `json-render` i GenSvg: inspiracje, decyzja o integracji dopiero po krótkim proof of concept; nie uzależniać formatu IconSpec od konkretnego frameworka.
- Adapter modelu oddziela provider od domeny. Structured output preferowany; przy błędzie schematu jeden jawny retry z diagnostyką.

## Ochrona i limity

Waliduj również po odpowiedzi modelu: maks. 64 prymitywy, głębokość grup 3, wartości współrzędnych w zakresie ustalonym przez profil, maks. 3 rundy rewizji. Escape tekstu XML, choć MVP nie przewiduje tekstu jako prymitywu. SVG buduj z allowlisty tagów/atrybutów; odrzuć URI, CSS, event handlers i nieznane właściwości. Traktuj przykłady i prompty jako dane, nie instrukcje uruchamiania kodu. Ustaw budżet API na zadanie.

## Kontrola jakości

Statycznie: granice, liczba elementów, wartości, stroke, odstępy i ostrzeżenia o możliwym nakładaniu. Wizualnie: czy znak jest rozpoznawalny w 24 px, czy ma właściwą liczbę detali i nie kłóci się z arkuszem ikon. Ocena vision jest pomocnicza; ostateczną jakość pilota ocenia człowiek. Zapisuj wejście, wersję spec, wynik i decyzję w artefaktach lokalnych bez sekretów API.

## Rozwój

Po pilocie dodaj semantyczne szablony jako makra kompilowane do prymitywów, np. `person`, `chair`, `monitor`, z parametrami pozy i rozmiaru. Każde makro musi mieć test referencyjny i nie może wymuszać migracji istniejących ikon. Edytor React pokazujący JSON i podgląd jest kolejnym etapem.
