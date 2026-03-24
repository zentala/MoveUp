---
id: E002-T00
epic: E002
status: done
original_id: 0001-screen-overlay-progress-bar
---
# TASK 0001: Screen Overlay Progress Bar

**Status:** IN PROGRESS
**Date Started:** 2026-03-16
**Priority:** P0 (Critical Feature)

---

## PRECYJNA DEFINICJA CELU

### Co ma sie dziac
Wyswietlic **pasek postepu na GORZE EKRANU** — **calkowicie niezalezny od okna aplikacji**.

### Specyfikacja
- **Umiejscowienie:** Top-left corner of PRIMARY MONITOR (0, 0)
- **Wymiary:** 4px wysokosci x pelna szerokosc ekranu
- **Brak dekoracji:** Brak ramki, brak cienia, brak zaokraglenia, brak title bar
- **Zawsze na wierzchu:** Ponad innymi oknami (always-on-top)
- **Kolory:** Zielony (0-60%) -> Amber (60-85%) -> Czerwony (85%+)
- **Animacja:** Animowany width od 0% do 100% (0.5s ease)
- **Triggery:**
  - Pokazywac: Gdy uzytkownik siedzi (state = Sitting) I sessionLimitSecs > 0
  - Ukrywac: Gdy uzytkownik stoi/spaceruje/jest away

### Czego NIE chcemy
- Pasek wewnatrz okna aplikacji (bedzie widoczny tylko gdy okno widoczne)
- Pasek w osobnym oknie Tauri (okno zawsze ma dekoracje/cien/zaokraglenie)
- Pasek w overlay.html oknie (same problemy co wyzej)
- Fixed positioning w App.tsx (jest wewnatrz main okna Tauri)

---

## Probowane Podejscia (FAILED)

### Podejscie 1: Overlay Tauri WebviewWindow + HTML Divs
- **Lokacja:** `overlay.rs` + `overlay.html` + `overlay/main.tsx` (React divs)
- **Problem:** Event `overlay:progress` nigdy nie docieral do React komponenty. Okno zawsze mialo 20px wysokosci zamiast 4px (DPI scaling).
- **Status:** ABANDONED

### Podejscie 2: Delay na Event Listener
- **Zmiana:** Dodanie `setTimeout(..., 500ms)` przed `listen()`
- **Problem:** Event dalej nie docieral
- **Status:** ABANDONED

### Podejscie 3: Canvas Drawing na Overlay
- **Zmiana:** `overlay.html` -> `<canvas>`, rysowanie na 2D context
- **Problem:** Canvas byl pusty (event nigdy nie dotarl)
- **Status:** ABANDONED

### Podejscie 4: Non-Transparent Tauri Window
- **Zmiana:** `.transparent(false)` -> zwykle czarne okno
- **Problem:** Okno mialo zaokraglenie Windows (narozniki), cien systemu, 10px offset od gory, wysokosc > 4px
- **Status:** ABANDONED — **CRITICAL FINDING:** Tauri WebviewWindow zawsze ma wlasciwosci systemu Windows

### Podejscie 5: HTML Fixed Overlay w App.tsx (ScreenProgressBar)
- **Zmiana:** Nowy komponent z `position: fixed; top: 0; z-index: 999999`
- **Problem:** Pasek pojawil sie, ale WEWNATRZ glownego okna Tauri
- **Status:** ABANDONED — **CRITICAL FINDING:** Main Tauri window przyslania wszystko

---

## Root Causes Discovered

### 1. Tauri Event System
- Event `overlay:progress` emituje sie z Rust
- Ale React `listen()` nigdy nie wyzwala callback'a

### 2. Tauri Windows zawsze maja System Decorations
- `.decorations(false)` nie usuwa zaokraglenia/cienia
- `inner_size(4px)` system skaluje do 20px+ (DPI)

### 3. Fixed Positioning nie Przebija sie Ponad Tauri Window
- HTML `position: fixed` jest ignorowany przez Tauri layer

---

## Rozwiazanie: Opcja A — Rysowanie w Rust (System-Level Overlay)
- Uzyc `windows` crate do WinAPI custom window bez dekoracji
- **Zaleta:** Prawdziwy overlay, niezwiazany z Tauri
- Implemented as `overlay_renderer.rs` — see E002 PLAN.md
