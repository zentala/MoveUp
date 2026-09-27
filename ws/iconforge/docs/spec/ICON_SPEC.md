# IconSpec v1

Dokument JSON opisuje geometrię. Identyfikator profilu stylu odwołuje się do pliku kontrolowanego przez projekt; LLM nie ustawia swobodnie szerokości linii dla każdej ikony.

```ts
type Point = readonly [number, number];
type Base = { id: string };
type Shape = Base & (
  | { type: 'line'; from: Point; to: Point }
  | { type: 'polyline'; points: Point[]; closed?: boolean }
  | { type: 'circle'; center: Point; radius: number }
  | { type: 'ellipse'; center: Point; rx: number; ry: number }
  | { type: 'rect'; origin: Point; width: number; height: number }
  | { type: 'roundedRect'; origin: Point; width: number; height: number; radius: number }
  | { type: 'arc'; center: Point; radius: number; startDeg: number; sweepDeg: number }
  | { type: 'curve'; from: Point; control1: Point; control2: Point; to: Point }
  | { type: 'group'; children: Shape[]; translate?: Point; rotationDeg?: number }
);
type IconSpec = {
  version: 1;
  name: string;
  profile: string;
  shapes: Shape[];
};
```

Przykład `desk`:

```json
{
  "version": 1,
  "name": "desk",
  "profile": "outline-24-v1",
  "shapes": [
    { "id": "top", "type": "roundedRect", "origin": [3, 9], "width": 18, "height": 2, "radius": 0.5 },
    { "id": "left-leg", "type": "line", "from": [6, 11], "to": [6, 20] },
    { "id": "right-leg", "type": "line", "from": [18, 11], "to": [18, 20] }
  ]
}
```

Proponowany profil `outline-24-v1`: viewBox `[0,0,24,24]`; stroke `currentColor`; strokeWidth `1.5`; cap/join `round`; fill `none`; margines optyczny 2; siatka orientacyjna 0.5; minGap 1.0. Walidator sprawdza faktyczny obrys z uwzględnieniem połowy szerokości stroke. Nie kwantyzuj bezwarunkowo punktów kontrolnych krzywych. Ograniczenia odstępów mogą zgłaszać ostrzeżenie, bo zamierzone przecięcia są częste.

Kontrakt łuku: kąty w stopniach, 0° na prawo, dodatni obrót zgodny z ruchem wskazówek zegara w SVG; `0 < abs(sweepDeg) <= 360`. Pełne 360° kompiluj jako okrąg lub dwa łuki. Obrót grupy wokół początku grupy; w późniejszej wersji jawny pivot. Wszystkie `id` unikalne w dokumencie. `closed` nie oznacza automatycznego fill. Maksymalne wymiary i liczby wynikają z profilu i walidatora.

Schema Zod ma odrzucać nieznane pola (`strict`), NaN, Infinity i niepoprawne tablice punktów. Wersję schematu migratuj jawnie; nie naprawiaj po cichu współrzędnych modelu. Serializer ma stałą precyzję, kolejność atrybutów i identyczny output dla identycznego wejścia. Metadane generacji zapisuj obok JSON, nie w samym kontrakcie geometrii.
