# IconForge — dokumentacja projektu

Wersja: 0.1, 27.09.2026. Status: specyfikacja do implementacji.

Cel: agent generuje spójne, edytowalne ikony liniowe w formacie SVG z opisu słownego. Repozytorium ma być TypeScript-first i działać lokalnie oraz w CI. Nazwa IconForge jest robocza.

## Czy modele obrazu rysują skryptem?

Nie ma jednego mechanizmu dla „AI generuje grafikę”. Model obrazowy może wytwarzać reprezentację obrazu, którą system dekoduje do rastra; nie musi pisać skryptu SVG. Publiczny opis natywnego generowania obrazów GPT-4o określa ten model jako autoregresyjny, a DALL·E jako dyfuzyjny. Dokładny wewnętrzny pipeline aktualnej usługi nie jest w pełni publiczny. ChatGPT może też użyć narzędzi lub napisać kod, jeśli zadanie tego wymaga. Ładne napisy w wygenerowanym obrazie nie dowodzą, że powstały jako fonty lub wektory. W naszym projekcie jawnie wybieramy inny mechanizm: LLM → kontrolowana specyfikacja → deterministyczny renderer → SVG.

Podobna idea Vercela to `vercel-labs/json-render`: katalog dopuszczonych komponentów, JSON generowany przez model i renderer. `GenSvg` stosuje zbliżony wzorzec do SVG. Projekty stanowią odniesienie architektoniczne, nie obowiązkowe zależności.

## Dokumenty

- [PRD.md](PRD.md) — potrzeby, zakres, kryteria odbioru i etapy.
- [ARCHITECTURE.md](ARCHITECTURE.md) — komponenty, przepływ, wybory technologiczne.
- [ICON_SPEC.md](ICON_SPEC.md) — format danych i przykłady.
- [AGENTS.md](AGENTS.md) — instrukcje dla agentów wdrażających.
- [TASKS.md](TASKS.md) — kolejność zadań i definicja ukończenia.
- [RESEARCH.md](RESEARCH.md) — źródła i granice ustaleń.

Rozpoczęcie: przeczytaj `AGENTS.md`, wykonuj `TASKS.md` od fazy 0, a każdą zmianę formatu uzgadniaj ze specyfikacją `ICON_SPEC.md`.
