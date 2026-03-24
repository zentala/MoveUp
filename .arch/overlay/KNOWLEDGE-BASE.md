# Overlay Progress Bar — Knowledge Base

**Created:** 2026-03-19
**Purpose:** Kompletna wiedza z sesji debugowania. Czytaj przed każdą zmianą w overlay.

---

## 1. Architektura

### Plik: `src-tauri/src/overlay_renderer.rs`

Overlay to natywne okno Windows (4px × pełna szerokość ekranu) na górze ekranu.
Dwa tryby renderowania, wybierane env var `OVERLAY_MODE`:

| | OPAQUE (default) | LAYERED (experimental) |
|---|---|---|
| Env var | brak lub `OVERLAY_MODE=opaque` | `OVERLAY_MODE=layered` |
| Tło | Czarne (nieprzezroczyste) | Przezroczyste |
| Rendering | GDI: BeginPaint → FillRect | UpdateLayeredWindow z DIBSection |
| Wnd proc | `wnd_proc()` | `wnd_proc_layered()` |
| Event loop | `run_event_loop_opaque()` | `run_event_loop_layered()` |
| Redraw | InvalidateRect → WM_PAINT | WM_TIMER → draw_layered_frame() |
| Test code | ✅ TAK (color cycling) | ❌ NIE |
| Min bar width | 0px | 1px |

### Plik: `src-tauri/src/tray_controller.rs`

Nasłuchuje event `desk:state-changed` i wywołuje:
```rust
if DeskState::Sitting {
    overlay.update(progress, (r, g, b));  // ustawia progress + kolor
    overlay.show();                        // ustawia visible=true
} else {
    overlay.hide();                        // ustawia visible=false
}
```

### OverlayState (shared state)
```rust
pub struct OverlayState {
    pub progress: f32,           // 0.0–1.0
    pub color_rgb: (u8, u8, u8), // RGB kolor paska
    pub visible: bool,           // czy pasek widoczny
    pub needs_redraw: bool,      // dirty flag
    pub frame_count: u32,        // counter do animacji
    pub demo_mode: bool,         // OVERLAY_DEMO_MODE=true
}
```

---

## 2. Co Działa ✅

### Test code w OPAQUE mode
**Lokalizacja:** WM_PAINT handler, linie ~303-319

```rust
// Cykluje kolory co 3 frames (~48ms):
let test_colors = [(255,0,0), (139,0,0), (128,0,128), (240,230,200), (0,128,0)];
let color_idx = ((s.frame_count / 3) as usize) % test_colors.len();
FillRect(hdc, &rect, test_brush);  // Fills ENTIRE window
```

**Dlaczego działa:**
- Wykonuje się **bezwarunkowo** (nie sprawdza `visible`)
- Rysuje na **całą szerokość** okna
- User widzi: szybko zmieniające się kolory na pasku

### LAYERED mode z demo_mode
**Lokalizacja:** `draw_layered_frame()`, linie ~604-632

- Semi-przezroczysty biały pasek (alpha=128)
- Demo override: progress cykluje 0%→25%→50%→75%→100% co 5 sekund
- Minimum 1px bar width (zawsze coś widać)
- **Działa ale user tego nie testował** (wymaga `OVERLAY_MODE=layered`)

---

## 3. Co NIE Działa ❌

### Progress bar w OPAQUE mode
**Lokalizacja:** WM_PAINT handler, linie ~321-349

```rust
if s.visible {  // ← TU JEST PROBLEM: visible=false
    let bar_rect = RECT { left: 0, top: 0, right: bar_width, bottom: window_height };
    FillRect(hdc, &bar_rect, brush);
}
```

**Dlaczego nie działa:**
1. `visible` ustawiane na `true` TYLKO gdy `DeskState == Sitting`
2. `DeskState::Sitting` wymaga desk sensora (XIAO ESP32-C3) na COM3
3. **Bez sensora → brak eventów → visible NIGDY nie staje się true**
4. **Cały blok `if s.visible` się nie wykonuje**
5. **Progress bar NIGDY się nie rysuje**

### Demo mode w OPAQUE
Demo mode ustawia `visible=true` i `progress` cyklicznie w WM_TIMER.
ALE user nigdy nie uruchamiał z `OVERLAY_DEMO_MODE=true`.
Uruchamiał: `pnpm tauri:dev` (bez env vars) → demo_mode=false.

---

## 4. Fundamentalny Błąd Agenta

### Co agent robił źle:

1. **Testował inny code path niż user widział**
   - Auto-test: `OVERLAY_MODE=layered OVERLAY_DEMO_MODE=true` → LAYERED mode
   - User: `pnpm tauri:dev` → OPAQUE mode
   - Zupełnie inne funkcje, inne warunki, inny rendering

2. **Nie rozumiał dlaczego `visible=false`**
   - Myślał że problem to progress override lub bar_width
   - Nie sprawdził: skąd visible=true? (odpowiedź: desk sensor, którego nie ma)

3. **Usuwał działający kod bez zrozumienia**
   - Usunął test code → nic nie widać
   - Nie zrozumiał że test code był JEDYNYM widocznym elementem
   - Bo progress bar nigdy się nie rysował (visible=false)

4. **Testy nie testowały tego co powinny**
   - Sprawdzały wartości w logach (frame_count, progress, bar_width)
   - NIE sprawdzały czy coś jest faktycznie narysowane na ekranie
   - NIE sprawdzały OPAQUE mode (tylko LAYERED)

5. **Robił zmiany bez weryfikacji**
   - Proponował "fix" → kompilował → "should work" → nie działało
   - Powtarzał ten sam pattern wielokrotnie
   - Nie miał sposobu weryfikacji

---

## 5. Poprawne Rozwiązanie

### Cel: Progress bar widoczny w OPAQUE mode bez desk sensora

**Podejście:** Zmodyfikować test code tak, aby:
1. Zachować bezwarunkowe renderowanie (nie wymagać visible=true)
2. Zamiast kolorów na pełną szerokość → kolor na bar_width
3. W demo mode: cyklicznie zmieniać bar_width (0%, 25%, 50%, 75%, 100%)
4. Zwolnić animację (co 5 sekund zmiana, nie co 48ms)

**Kod do zmiany:** WM_PAINT handler w `wnd_proc()`, linie ~296-349

**Logika:**
```
1. Wypełnij tło czarnym
2. Oblicz bar_width:
   - Jeśli demo_mode: stage = (frame_count / 300) % 5; progress = stage / 4.0
   - Jeśli nie demo: progress z state (od tray_controller)
   - bar_width = window_width * progress
   - Minimum 1px jeśli progress > 0 lub demo_mode
3. Oblicz kolor:
   - Jeśli demo_mode: cykluj kolory co 300 frames (co 5 sekund)
   - Jeśli nie demo: kolor z state.color_rgb
4. Rysuj FillRect od (0,0) do (bar_width, window_height)
   - BEZ warunku visible (zawsze rysuj w demo_mode)
   - Z warunkiem visible tylko w normalnym mode
```

### Dlaczego to zadziała:
- Test code już UDOWODNIŁ że bezwarunkowy FillRect jest widoczny
- Zmieniamy tylko KSZTAŁT (partial width zamiast full width)
- I TEMPO (co 5 sekund zamiast co 48ms)
- Reszta mechanizmu renderowania jest taka sama

---

## 6. Jak Testować

### Test automatyczny (auto-test.sh):
- Uruchamia z `OVERLAY_DEMO_MODE=true` (ALE BEZ `OVERLAY_MODE=layered`!)
- Parsuje logi: sprawdza bar_width, progress, frame_count
- **WAŻNE:** Testuj OPAQUE mode, nie LAYERED

### Test wizualny (user):
- `pnpm tauri:dev` — normalny mode, bez sensora → powinien widzieć demo
- `OVERLAY_DEMO_MODE=true pnpm tauri:dev` — demo mode z cycling

### Czego szukać w logach:
```
WM_PAINT: bar_width=480 (progress=0.25), visible=true   ← 25% width
WM_PAINT: bar_width=960 (progress=0.50), visible=true   ← 50% width
```

### Czego szukać wizualnie:
- Pasek rośnie od lewej strony
- Zmienia kolor (zielony→żółty→czerwony) co 5 sekund
- Reszta okna czarna
- Brak szybkiego migania

---

## 7. Reguły

1. **NIE usuwaj test code** dopóki progress bar nie jest udowodniony jako widoczny
2. **NIE testuj LAYERED gdy user widzi OPAQUE** — to inne code paths
3. **NIE zakładaj visible=true** — bez sensora jest false
4. **NIE rób zmian bez weryfikacji** — sprawdź logi PO zmianie
5. **Jedna zmiana na raz** — zmień jedną rzecz, przetestuj, dopiero potem następna
6. **Dokumentuj każdą iterację** — w OVERLAY-REPORT.md

---

## 8. Powiązane Pliki

| Plik | Opis |
|------|------|
| `src-tauri/src/overlay_renderer.rs` | Główny kod overlay |
| `src-tauri/src/tray_controller.rs` | Steruje overlay (show/hide/update) |
| `src-tauri/src/colors.rs` | Mapowanie progress → kolor |
| `src-tauri/src/session.rs` | State machine: Sitting/Standing/Walking |
| `.claude/tasks/OVERLAY-REPORT.md` | Historia iteracji |
| `.claude/auto-test.sh` | Automatyczny test loop |
| `.claude/AUTONOMOUS-WORKFLOW.md` | Workflow iteracyjny |
| `.claude/TESTING-SYSTEM.md` | System testowania |
| `.claude/TESTING-OVERLAY.md` | Instrukcje ręcznego testowania |
| `tests/overlay-screenshots.test.ts` | Screenshot testing (Playwright) |

---

## 9. Komendy

```bash
# Normalny mode (OPAQUE, bez demo — wymaga sensora)
pnpm tauri:dev

# Demo mode OPAQUE (bez sensora, z cycling)
OVERLAY_DEMO_MODE=true pnpm tauri:dev

# Demo mode LAYERED (transparentny, z cycling)
OVERLAY_MODE=layered OVERLAY_DEMO_MODE=true pnpm tauri:dev

# Auto-test (OPAQUE demo)
bash .claude/auto-test.sh

# Kompilacja
cd src-tauri && cargo check

# Testy Rust
cd src-tauri && cargo test overlay_renderer --lib
```

---

## 10. Git History (tej sesji)

| Commit | Opis | Status |
|--------|------|--------|
| `f5b885f` | Remove debug logging, restore state logic | ✅ Cleanup |
| `46e9fe2` | Add demo mode for progress bar testing | ⚠️ Demo nie testowało OPAQUE |
| `4ec7e62` | 5x slower demo cycling, always visible | ⚠️ Zmieniało LAYERED, nie OPAQUE |
| `0ff291f` | Demo progress override at render time | ⚠️ Override w LAYERED draw_layered_frame |
| `066a4c2` | Add comprehensive logging | ✅ Logging |
| `5af96e7` | Testing guide for log-based debugging | ✅ Docs |
| `708e69a` | Automated log + screenshot testing | ✅ Infra |
| `bceb943` | Autonomous test & iteration system | ✅ Infra |
| `ed1933b` | Remove test color cycling from OPAQUE | ❌ BŁĄD — usunięto jedyny widoczny element |
| `e8d6294` | Revert removal of test code | ✅ Naprawiono |
