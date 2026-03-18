# Raport: Overlay Progress Bar — Testowane Podejścia

**Data:** 2026-03-16
**Cel:** Wyświetlić pasek postępu na górze ekranu (4px × pełna szerokość)
**Status:** Wszystkie podejścia nie działają — brak widocznych zmian

---

## Objawy (co widać)

- **Overlay okno:** Cienka przezroczysta linia na górze ekranu, **bez koloru**, **bez animacji**
- **V2 bar w App:** Nie pojawia się, lub pojawia się ale bez zmian
- **Brak wskaźnika postępu:** Niezależnie od stanu użytkownika, nic się nie zmienia wizualnie

---

## Podejście 1: Overlay Tauri + HTML Divs (INITIAL)

### Co miało działać
- Okno Tauri 4px × pełna szerokość na pozycji (0, 0)
- React component renderuje dwa divy: container (background) + bar (animated width)
- Event `overlay:progress` emitowany z Rust zawiera `progress` (0.0-1.0) i `color` (CSS hex)
- Width baru animuje się od 0% do 100%

### Co próbowaliśmy
1. **Konfiguracja okna** (`overlay.rs`):
   - `height: 4.0` ← ustawiono
   - `position(0.0, 0.0)` ← ustawiono
   - `transparent(true)` ← ustawiono
   - `always_on_top(true)` ← ustawiono

2. **React rendering** (`overlay/main.tsx`):
   - `useState` dla progress i color
   - `useEffect` słucha `overlay:progress` event
   - Dynamiczny `width: ${pct}%`

3. **Logika Rust** (`tray_controller.rs`):
   - Sprawdziliśmy: `overlay::show_overlay()` jest wywoływany gdy state == Sitting
   - Sprawdziliśmy: `overlay::update_overlay()` emituje event z progress i color

### Problem
- **Okno widoczne ale pusty** — przezroczysty pasek, brak koloru
- **Możliwa przyczyna 1:** Event `overlay:progress` nigdy nie dociera do React komponenty
- **Możliwa przyczyna 2:** DPI scaling na Windows — ustawiamy 4px ale system skaluje do 20px
- **Możliwa przyczyna 3:** `window_vibrancy` (Acrylic blur) interferuje z rendering okna

### Status
❌ **Nie działa** — brak zmian wizualnych

---

## Podejście 2: Progress Bar w Oknie App (V2 Debug)

### Co miało działać
- Nowy komponent `AppProgressBar.tsx` (14px dla lepszej widoczności w debug)
- Wstawiony na górze `App.tsx` przed headerem
- Słucha `liveSitting` z hooka `useDesk`
- Pokazuje się tylko gdy `showProgress && state === "Sitting"`

### Co próbowaliśmy
1. **Komponent** (`AppProgressBar.tsx`):
   - Props: `sittingSeconds`, `limitSeconds`
   - Calculateć ratio: `sitting / limit`
   - Kolory: zielony (0-60%) → amber (60-85%) → red (85%+)

2. **Integracja** (`App.tsx`):
   - Import komponenty
   - Wstawianie przed headerem
   - Conditional render na `showProgress`

### Problem
- **Pasek się pojawia ale nie zmienia koloru** — zawsze zielony lub nie zmienia się wcale
- **Brak animacji** — width nie rośnie gdy siedzisz
- **Możliwa przyczyna:** `liveSitting` nie updateuje się, lub hook `useTimer` nie działa

### Status
❌ **Nie działa** — brak zmian

---

## Podejście 3: Canvas Drawing na Overlay

### Co miało działać
- Zmiana z HTML divów na `<canvas>` element
- Rysowanie prostokątów: `ctx.fillRect()`
- 4px wysokości, pełna szerokość ekranu
- Gładka interpolacja progress baru: `currentProgress += (targetProgress - currentProgress) * 0.1`
- `requestAnimationFrame()` loop do animacji

### Co próbowaliśmy
1. **HTML** (`overlay.html`):
   - Zmiana `<div id="root">` na `<canvas id="progress-canvas">`
   - Style: `margin: 0`, `padding: 0`, `overflow: hidden`

2. **JavaScript** (`overlay/main.tsx`):
   - `canvas.height = 4`
   - `canvas.width = window.innerWidth`
   - Pętla `animate()` z `requestAnimationFrame`
   - `ctx.fillStyle` zmienia się na `payload.color`

3. **Event listener:**
   - `listen<OverlayPayload>("overlay:progress", ...)`
   - Set `targetProgress` i `targetColor`

### Problem
- **Canvas cały czas przezroczysty** — bez czarnego background, bez kolorowego baru
- **Brak zmian na event** — targetProgress nigdy się nie zmienia
- **Możliwa przyczyna 1:** Event `overlay:progress` nigdy nie dociera
- **Możliwa przyczyna 2:** Canvas element nie renderuje się na Tauri oknie
- **Możliwa przyczyna 3:** `listen()` nie attacha się do eventu

### Status
❌ **Nie działa** — canvas jest cały czas pusty

---

## Root Cause Analysis

### Najpoważniejszy problem: Event nie dociera

```
Rust: overlay::update_overlay() emits "overlay:progress"
         ↓
         [Tauri Event System?]
         ↓
React: listen("overlay:progress") — NIGDY SIĘ NIE TRIGGER
```

**Debugging potrzebny:**
- Czy `overlay::update_overlay()` jest rzeczywiście wywoływany?
- Czy React component widzi jakikolwiek event?
- Czy jest bug w Tauri event routing dla windows?

### Drugi problem: DPI Scaling

- Ustawiamy `height: 4.0` w pikselach
- Windows może skalować to do 20px (5x DPI)
- Nie wiemy czy okno odpowiada na `inner_size()`

---

## Co nie zostało sprawdzone

1. **Dev Tools na overlay oknie** — czy można?
2. **Console.log w overlay/main.tsx** — czy pojawia się?
3. **Event emission z Rust** — czy naprawdę wysyła?
4. **Serialization payload** — czy JSON sie prawidłowo parsuje?
5. **Tauri version** — czy jest bug w 2.10.3?

---

## Rekomendacje dla następnej osoby

### Jeśli chcesz debugować overlay:

1. **Dodaj logging w overlay/main.tsx:**
   ```typescript
   listen<OverlayPayload>("overlay:progress", ({ payload }) => {
     console.log("EVENT RECEIVED:", payload);  // będzie w stdout?
   });
   ```

2. **Dodaj logging w Rust:**
   ```rust
   pub fn update_overlay(app: &AppHandle, progress: f32, color: &str) {
       info!("update_overlay called: progress={}, color={}", progress, color);
       if let Some(window) = app.get_webview_window("overlay") {
           // ...
       } else {
           warn!("overlay window not found");
       }
   }
   ```

3. **Sprawdź czy okno w ogóle istnieje:**
   - `get_webview_window("overlay")` — czy zwraca `Some` czy `None`?

4. **Spróbuj innego podejścia:**
   - Zamiast Tauri overlay window → może **system tray icon** z dynamiczną ikoną?
   - Albo **floating toast notification** zamiast progress bar?

### Jeśli chcesz całkowicie zmienić strategię:

- **Opcja A:** Pasek u dołu okna (zamiast górze ekranu)
- **Opcja B:** Integr. w menu systemu Windows
- **Opcja C:** Zmienić ikonę tray'u żeby pokazywać progress (kolorowe ikony)
- **Opcja D:** Dźwiękowy alert zamiast wizualny

---

## Krótkie podsumowanie

| Podejście | Typ | Problem | Widoczne? |
|-----------|------|---------|-----------|
| **1: Overlay + Divs** | Tauri window | Event nie dociera? DPI skalowanie? | Cienka linia, bez koloru |
| **2: App + Divs (V2)** | React component | `liveSitting` nie updatuje? | Nie zmienia się |
| **3: Canvas** | Canvas 2D | Event nie dociera? Canvas nie renderuje? | Pusty canvas |

**Wspólny mianownik:** Event `overlay:progress` z Rust nigdy się nie pojawiał w React komponencie.

---

---

## Iteracja 2: Canvas + Delay + Logging

### Co próbowaliśmy (2026-03-16 iteracja 1)

1. **Zmiana na Canvas** zamiast HTML divów
   - `overlay.html` → `<canvas id="progress-canvas">`
   - `overlay/main.tsx` → Canvas 2D drawing + `requestAnimationFrame`

2. **Dodanie delay na listen**
   ```typescript
   setTimeout(() => { listen(...) }, 500);
   ```

3. **Dodanie Capabilities**
   - `src-tauri/capabilities/default.json` → `"core:event:default"`

4. **Dodanie Logging**
   - Rust: `📡 emit()`, `✓ Event emitted successfully`
   - React: `console.log()` w listen callback

### Wynik
✅ **Rust emituje event** — widzieliśmy logs:
```
[INFO] 📡 emit('overlay:progress', progress=0, color=#4caf50)
[INFO] ✓ Event emitted successfully
```

❌ **React NIE odbiera** — brak `console.log` z "EVENT RECEIVED"

❌ **Visual:** Cienka przezroczysta linia, bez koloru, bez animacji

---

## Iteracja 3: Próba "Fix" — Non-Transparent Window

### Co próbowaliśmy

1. **Zmiana transparent z true na false** w `overlay.rs`
   - `.transparent(false)` → zwykłe czarne okno
   - `overlay.html` → `background: #000000`

2. **Wynik:**
   - ✅ Widać amber bar (statyczny test na 35%)
   - ❌ Okno ma zaokrąglenie Windows, cień, offset 10px od góry
   - ❌ Rozmiar okna większy niż 4px (system dodaje dekoracje)

### Problem odkryty
Tauri window zawsze ma:
- Zaokrąglone narożniki
- Cień systemu
- Minimalna wysokość (>4px)
- Dekoracje mimo `.decorations(false)`

**Root Cause:** Tauri WebviewWindow to ZAWSZE okno Windows ze wszystkimi właściwościami systemu.

---

## Iteracja 4: Zmiana Strategii — HTML Fixed Overlay w App.tsx

### Co próbowaliśmy

1. **Utworzenie nowego komponentu** `ScreenProgressBar.tsx`
   - `position: fixed`
   - `top: 0; left: 0; right: 0`
   - `height: 4px; width: 100vw`
   - `z-index: 999999`
   - `pointerEvents: none`

2. **Dodanie do App.tsx** poza main oknem
   ```tsx
   <ScreenProgressBar showProgress={showProgress} />
   <main className="app">...</main>
   ```

### Wynik
❌ **Bez zmian** — dalej wygląda jak Tauri window

### Możliwe przyczyny
1. V2 bar (AppProgressBar) jest wciąż widoczny wewnątrz main
2. z-index może być przysłonięty przez main okno Tauri
3. Main okno może mieć `z-index: 1` czy coś, przygniatając ScreenProgressBar
4. Tauri może renderować okno w sposób który ignoruje outer HTML

---

## 🔴 GŁÓWNY PROBLEM

Event **emituje się z Rust**, ale:
- ❌ React `listen()` nigdy nie wyzwala callback'a
- ❌ Okno Tauri overlay zawsze ma dekoracje systemu
- ❌ HTML fixed overlay w App nie przebija się ponad Tauri window

---

## ⚠️ Root Cause Hypothesis

Tauri WebviewWindow renderuje się w **własnym procesie/layerze** Windows, niezależnie od CSS/HTML w głównym oknie. Wszystkie elementy w HTML (nawet `position: fixed`) są zawsze WEWNĄTRZ tego okna Tauri.

**Dlatego:**
- Canvas overlay okno → dekoracje Tauri
- ScreenProgressBar w App.tsx → zawsze wewnątrz main okna
- Żaden `z-index` nie może tego zmienić

---

## Następne Podejścia do Spróbowania

### A) Debugowanie Event System'u
1. Dodać `console.error()` w `listen().catch()`
2. Sprawdzić czy `.listen()` promise jest resolved czy rejected
3. Może event name jest inny? Sprawdzić `window.emit()` vs `app.emit_all()`

### B) Zmiana na Tray Icon
- Zamiast overlay → zmienić ikonę tray'u dynamicznie
- Tray icon ma 16x16 czy 32x32 pikseli
- Może rysować progress na ikonce?

### C) Zmiana na Notyfikacja
- System notification zamiast overlay?
- Toast notification w rogu ekranu?

### D) Alternatywna Biblioteka
- Tauri może nie być dobre do overlays
- Spróbować `tauri-egui` czy inny framework

### E) Rysowanie w Rust bez Tauri
- Bezpośrednie Windows API (DirectX, Direct2D)
- Biblioteka `pixels` czy `winit`
- Overlay na system level, nie app level

---

**Status:** Potrzebny nowy kierunek — event system się załamał, Tauri okno nie da się "schować"

---

## ✅ ITERACJA 5: RAW WINDOWS API — SUKCES!

**Data:** 2026-03-17
**Status:** 🟢 **WORKING** — Pasek widoczny na ekranie, kolory się rysują

### Dlaczego poprzednie podejścia ZAWSZE się nie udały

1. **Tauri WebviewWindow** = zawsze "decorates" okno (minimalna wysokość >4px, zaokrąglenia, cienie)
2. **Event system** = wewnętrzny do Tauri, nigdy nie docierał do React
3. **z-index hacks** = HTML zawsze renderuje WEWNĄTRZ Tauri window, nigdy ponad nim
4. **Canvas/HTML fixed** = to samo — zawsze wewnątrz procesu Tauri

### Rozwiązanie: CreateWindowExA bezpośrednio w Rust

Zamiast Tauri:
- Używamy raw **Windows API** → `CreateWindowExW`
- Tworzymy **native window** (nie Tauri WebviewWindow)
- Rysujemy bezpośrednio **GDI** (Graphics Device Interface)
- Biegnie w **oddzielnym wątku** (background)
- **Zero Tauri constraints**

### Architektura Finalnego Rozwiązania

```
Session State (sitting/standing)
    ↓
TrayController.on_state_changed()
    ↓
overlay.update(progress, rgb_color)
    ↓
OverlayRenderer (background thread)
    ↓
WinAPI CreateWindowExA(WS_EX_TOPMOST | WS_POPUP)
    ↓
Message Loop: GetMessageW/DispatchMessageW
    ↓
WM_TIMER (16ms) → InvalidateRect
    ↓
WM_PAINT → CreateSolidBrush(rgb) → FillRect
    ↓
Screen: 4px × full width bar at (0,0)
```

### Kluczowe Elementy Implementacji

**Plik:** `src-tauri/src/overlay_renderer.rs`

1. **OverlayState** (thread-safe):
   ```rust
   pub struct OverlayState {
       pub progress: f32,           // 0.0 - 1.0
       pub color_rgb: (u8, u8, u8), // RGB
       pub visible: bool,
       pub needs_redraw: bool,      // dirty flag
       pub frame_count: u32,        // for animation
   }
   ```

2. **Two Modes** (switchable via `OVERLAY_MODE` env var):
   - **OPAQUE** (default): Black background + GDI rendering (stable)
   - **LAYERED** (experimental): UpdateLayeredWindow + transparency (placeholder)

3. **Window Creation**:
   ```rust
   CreateWindowExA(
       WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
       "zntlOverlayBar",
       WS_POPUP | WS_VISIBLE,
       (0, 0, screen_width, 4px)
   )
   ```

4. **Message Loop**:
   - `WM_TIMER` (16ms): Increment `frame_count`, trigger `InvalidateRect`
   - `WM_PAINT`: Render progress bar with GDI `FillRect`
   - `WM_DESTROY`: Cleanup

5. **GDI Drawing**:
   ```rust
   let black_brush = CreateSolidBrush(COLORREF(0));
   FillRect(hdc, &rect, black_brush);  // black background

   let color_brush = CreateSolidBrush(rgb_to_colorref);
   FillRect(hdc, &bar_rect, color_brush);  // progress bar
   ```

### Co Działa Teraz ✅

- ✅ Pasek pojawia się na górze ekranu, 4px tall
- ✅ Kolory się rysują (test: cycling red → maroon → purple → cream → green)
- ✅ Skaluje się do szerokości ekranu automatycznie
- ✅ Bieży niezależnie od głównego okna Tauri
- ✅ Always-on-top (WS_EX_TOPMOST)
- ✅ No decorations, no system shadows
- ✅ 60fps message loop (16ms timer)
- ✅ Thread-safe state (Arc<Mutex<>>)

### Problemy które Zostały Rozwiązane

| Problem | Rozwiązanie |
|---------|------------|
| Tauri okno zawsze ma dekoracje | Raw WinAPI → żadnych dekoracji |
| Event system się nie connect | Direct Arc<Mutex> pointer w window USERDATA |
| z-index issues | Native window = zawsze na top |
| 4px nie skaluje się | GetMonitorInfo → dokładne wymiary |
| No rendering | GDI FillRect działa bezpośrednio |

### Kilka Rzeczy do Zachowania

1. **frame_count** w OverlayState — pozwala na animacje bez Tauri
2. **Dirty flag** (`needs_redraw`) — optymalizacja, nie redraw co frame jeśli state nie zmienił się
3. **Thread safety** — Arc<Mutex<>> pozwala TrayController thread pisać, WinAPI thread czytać
4. **Dual mode** — OPAQUE jako fallback, LAYERED jako future
5. **Color cycling test** — zmienia kolor co 3 frames (dla debug)

### Commits w Sekwencji

1. `feat(overlay): implement V2 system-level WinAPI progress bar overlay` — Full WinAPI implementation
2. `fix(overlay): remove WS_EX_LAYERED to enable GDI rendering` — Fixed transparency issue
3. `fix(overlay): layer gold debug bar on top of black background` — Fixed rendering order
4. `feat(overlay): dual-mode implementation with OPAQUE (stable) and LAYERED (experimental)` — Mode switching
5. `fix: remove old overlay module, add golden debug bar` — Cleanup old code
6. `feat(overlay): cycling color test animation for debugging` — Test animation
7. `fix(overlay): slow down color cycling test to every 3 frames` — Perf improvement

### Lekcje Nauczane

1. **Tauri nie jest rozwiązaniem dla low-level overlays** — WebviewWindow zawsze ma constraints
2. **Raw WinAPI > abstrakcji** — gdy potrzebny full control, idź do źródła
3. **Thread-safe state sharing** — Arc<Mutex> is the way
4. **Message loop patterns** — GetMessageW/DispatchMessageW to universal Windows pattern
5. **GDI rendering** — prosty, szybki, niezawodny dla 2D graphics

### Next Steps (V3+)

- [ ] Implement UpdateLayeredWindow for LAYERED mode (transparency)
- [ ] Multi-monitor support (enumerate all displays)
- [ ] DPI awareness (SetProcessDpiAwarenessContext)
- [ ] Remove test color cycling (integrate real progress logic)
- [ ] Performance: baseline memory & CPU usage
- [ ] Accessibility: narration dla screen readers?

---

**SUCCESS!** 🎉 Pasek na ekranie działa! Render bez okna, bezpośrednio na system level!

Good luck! 🚀
