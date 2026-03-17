# TASK 0001: Screen Overlay Progress Bar

**Status:** IN PROGRESS
**Date Started:** 2026-03-16
**Priority:** P0 (Critical Feature)

---

## 🎯 PRECYJNA DEFINICJA CELU

### Co ma się dziać
Wyświetlić **pasek postępu na GÓRZE EKRANU** — **całkowicie niezależny od okna aplikacji**.

### Specyfikacja
- **Umiejscowienie:** Top-left corner of PRIMARY MONITOR (0, 0)
- **Wymiary:** 4px wysokości × pełna szerokość ekranu
- **Brak dekoracji:** Brak ramki, brak cienia, brak zaokrąglenia, brak title bar
- **Zawsze na wierzchu:** Ponad innymi oknami (always-on-top)
- **Kolory:** Zielony (0-60%) → Amber (60-85%) → Czerwony (85%+)
- **Animacja:** Animowany width od 0% do 100% (0.5s ease)
- **Triggery:**
  - Pokazywać: Gdy użytkownik siedzi (state = Sitting) I sessionLimitSecs > 0
  - Ukrywać: Gdy użytkownik stoi/spaceruje/jest away

### Czego NIE chcemy
- ❌ Pasek wewnątrz okna aplikacji (będzie widoczny tylko gdy okno widoczne)
- ❌ Pasek w osobnym oknie Tauri (okno zawsze ma dekoracje/cień/zaokrąglenie)
- ❌ Pasek w overlay.html oknie (same problemy co wyżej)
- ❌ Fixed positioning w App.tsx (jest wewnątrz main okna Tauri)

---

## 📋 Próbowane Podejścia (FAILED)

### ❌ Podejście 1: Overlay Tauri WebviewWindow + HTML Divs
- **Lokacja:** `overlay.rs` + `overlay.html` + `overlay/main.tsx` (React divs)
- **Problem:**
  - Event `overlay:progress` nigdy nie docierał do React komponenty
  - Okno zawsze miało 20px wysokości zamiast 4px (DPI scaling)
  - Brak wizualnych zmian
- **Status:** ABANDONED

### ❌ Podejście 2: Delay na Event Listener
- **Zmiana:** Dodanie `setTimeout(..., 500ms)` przed `listen()`
- **Problem:**
  - Event dalej nie docierał
  - Tauri event system się nie komunikował między oknem a React
- **Status:** ABANDONED

### ❌ Podejście 3: Canvas Drawing na Overlay
- **Zmiana:** `overlay.html` → `<canvas>`, rysowanie na 2D context
- **Problem:**
  - Canvas był pusty (event nigdy nie dotarł)
  - Okno nadal miało dekoracje Tauri
- **Status:** ABANDONED

### ❌ Podejście 4: Non-Transparent Tauri Window
- **Zmiana:** `.transparent(false)` → zwykłe czarne okno
- **Problem:**
  - Okno miało zaokrąglenie Windows (narożniki)
  - Cień systemu
  - 10px offset od góry
  - Wysokość > 4px (system dodawał padding)
- **Status:** ABANDONED — **CRITICAL FINDING:** Tauri WebviewWindow zawsze ma właściwości systemu Windows

### ❌ Podejście 5: HTML Fixed Overlay w App.tsx (ScreenProgressBar)
- **Zmiana:** Nowy komponent z `position: fixed; top: 0; z-index: 999999`
- **Problem:**
  - Pasek pojawił się, ale WEWNĄTRZ głównego okna Tauri
  - Nie przebił się ponad oknem (z-index ignorowany przez main window)
  - Okno Tauri renderuje się w osobnym layer'ze który ignoruje CSS z HTML
- **Status:** ABANDONED — **CRITICAL FINDING:** Main Tauri window przysłania wszystko, nawet fixed positioning

---

## 🔍 Root Causes Discovered

### 1. Tauri Event System
- Event `overlay:progress` emituje się z Rust ✅
- Ale React `listen()` nigdy nie wyzwala callback'a ❌
- Możliwy bug Tauri v2 czy problem z komunikacją między oknem hidden a React

### 2. Tauri Windows zawsze mają System Decorations
- `.decorations(false)` nie usuwa zaokrąglenia/cienia
- `inner_size(4px)` system skaluje do 20px+ (DPI)
- Tauri WebviewWindow to ZAWSZE okno Windows z jego właściwościami

### 3. Fixed Positioning nie Przebija się Ponad Tauri Window
- HTML `position: fixed` jest ignorowany przez Tauri layer
- Z-index nie pomaga — main window jest renderowany w wyższej warstwie systemu

---

## ✅ Dalsze Kroki do Spróbowania

### Opcja A: Rysowanie w Rust (System-Level Overlay)
- Użyć `windows` crate do DirectX/Direct2D
- Lub `pixels` crate + `winit` do rysowania na system level
- Lub WinAPI do custom window bez dekoracji
- **Zaletą:** Prawdziwy overlay, niezwiązany z Tauri
- **Wadą:** Skomplikowana implementacja, mniej Cross-Platform

### Opcja B: Zmienić Architekturę Main Window
- Zrobić główne okno `fullscreen` lub borderless fullscreen
- Potem ScreenProgressBar byłby na górze tego okna
- **Zaletą:** Prostsze, wszystko w Tauri
- **Wadą:** Zmienia UX całej aplikacji

### Opcja C: Abandon Overlay, Alternatywne UI
- Dynamiczna ikona w system tray (zmienia kolor wg progress)
- Toast notification w rogu ekranu
- Panel na pasku Windows taskbar
- **Zaletą:** Możliwe do realizacji z Tauri
- **Wadą:** Inne UX niż overlay

### Opcja D: Czekać na Fix Tauri
- Event system może mieć bug w v2.10.3
- Czekać na następną wersję
- **Zaletą:** Może samo się naprawiać
- **Wadą:** Brak gwarancji, czeka się

---

## 📊 Podsumowanie Zasobów

- **Raport główny:** `.claude/raports/2026-03-16-overlay-progress-bar-approaches.md`
- **Research:** `.claude/raports/2026-03-16-tauri-event-research.md`
- **Kód:**
  - `src/components/ScreenProgressBar.tsx` — fixed overlay w React (FAILED)
  - `src-tauri/src/overlay.rs` — Tauri window (FAILED)
  - `overlay.html` + `overlay/main.tsx` — Canvas (FAILED)

---

## 🚀 Next Decision Point

**Pytanie:** Którą z opcji (A/B/C/D) chcesz spróbować następnie?

