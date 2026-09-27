# Źródła i interpretacja (27.09.2026)

- OpenAI, opis GPT-4o image generation: https://openai.com/index/introducing-4o-image-generation/ — generowanie natywne i tekst w obrazie.
- OpenAI, addendum system card: https://cdn.openai.com/11998be9-5319-4302-bfbf-1167e093f1fb/Native_Image_Generation_System_Card.pdf — publiczne rozróżnienie modelu autoregresyjnego i dyfuzyjnego.
- Vercel, json-render: https://github.com/vercel-labs/json-render — katalog komponentów i ograniczony JSON renderowany przez kod.
- GenSvg: https://github.com/pq-dong/GenSvg — podobny pomysł dla SVG, do ewaluacji, bez założenia jakości lub utrzymania projektu.
- Paper.js Path: https://paperjs.org/reference/path/ — wygładzanie i eksport SVG.
- Maker.js Exporting: https://maker.js.org/docs/exporting/ — eksport modelu geometrycznego do SVG.

Wniosek projektowy: narzucenie katalogu prymitywów i renderowanie danych po walidacji powinno ułatwić kontrolę stylu. To hipoteza do zmierzenia pilotażem, nie własność każdego modelu. Zdanie, że współczesny model obrazu „rysuje napis fontem” albo „pisze skrypt SVG”, nie wynika z podanych źródeł. Nie należy też zakładać, że bezpośredni serializer zawsze wygra z Paper.js; spike rozstrzyga konkretny przypadek.
