# ARCHITEKTURA: System-Level Progress Bar Overlay

**Data:** 2026-03-16
**Opcja:** A — Raw Windows API (system-level, poza Tauri)
**Status:** PLAN ZATWIERDZONY — gotowy do implementacji
**Reviewed:** plan-eng-review (SMALL CHANGE mode)

---

## 🎯 Problem

Tauri `WebviewWindow` zawsze ma dekoracje systemu Windows (zaokrąglenie narożników, cień, minimalną wysokość >4px, offset od góry). Nie możemy zrobić czystego 4px overlay przy użyciu Tauri.

## ✅ Rozwiązanie

Stworzyć **niezależne okno WinAPI** w Rust, **całkowicie poza Tauri**, przy użyciu `windows` crate który **już mamy** w `Cargo.toml`.

---

## 📊 Data Flow

```
Sensor (COM3)
    │ distance reading (mm)
    ▼
serial.rs → SessionManager → emits "desk:state-changed"
                                    │
                                    ▼
                          tray_controller.rs
                          ┌──────────────────────────┐
                          │ 1. update_tray()          │
                          │ 2. overlay.update(prog)   │ ← wywołuje
                          │ 3. overlay.show()/hide()  │    OverlayRenderer
                          └──────────────────────────┘
                                    │
                                    ▼
                          OverlayRenderer (Arc<Mutex<>>)
                          ┌──────────────────────────┐
                          │ shared OverlayState        │
                          │  - progress: f32           │
                          │  - color: (u8,u8,u8)       │
                          │  - visible: bool           │
                          │  - needs_redraw: bool      │ ← dirty flag
                          └──────────────────────────┘
                                    │ background thread
                                    ▼
                          WinAPI Event Loop (osobny wątek)
                          ┌──────────────────────────┐
                          │ CreateWindowExW()          │
                          │  WS_EX_TOPMOST             │
                          │  WS_EX_LAYERED             │
                          │  WS_EX_NOACTIVATE          │
                          │  WS_EX_TOOLWINDOW          │
                          │ SetTimer(16ms) → WM_TIMER  │
                          │ WM_PAINT → FillRect GDI    │
                          └──────────────────────────┘
                                    │
                                    ▼
                          4px × screen_width
                          position: (0, 0)
                          NO decorations, NO shadow
                          TOPMOST = above all windows
```

---

## 🛠️ Tech Stack

**Raw Windows API** — `windows` crate v0.61 (już w `Cargo.toml`)

```toml
# src-tauri/Cargo.toml — ŻADNYCH nowych zależności, już mamy:
[target.'cfg(windows)'.dependencies]
windows = { version = "0.61", features = [
    "Win32_UI_WindowsAndMessaging",   # CreateWindowExW, SetTimer, WM_PAINT
    "Win32_Graphics_Gdi",             # FillRect, PAINTSTRUCT, BeginPaint
    "Win32_Foundation",               # HWND, LPARAM, WPARAM
    # ... inne features
]}
```

**Uwaga:** Trzeba sprawdzić czy te konkretne features są już włączone. Jeśli nie — dodać do istniejącego `features = [...]`.

---

## 🏗️ Implementacja

### Krok 0: Zmiana `lib.rs` — stary setup overlay

```rust
// PRZED (lib.rs linia ~87):
overlay::setup_overlay(app.handle())?;

// PO — usunąć tę linię, zastąpić zarządzaniem przez AppState:
// OverlayRenderer::new() tworzy WinAPI window w tle
// Tauri overlay window jest niepotrzebne
```

Plik `overlay.rs` (stary) — można zachować jako archiwum lub usunąć. Nowy renderer będzie w `overlay_renderer.rs`.

---

### Krok 1: Nowy moduł `overlay_renderer.rs`

```rust
//! overlay_renderer.rs — System-level progress bar overlay using raw WinAPI.
//!
//! Creates a native Windows window (NOT Tauri WebviewWindow) at position (0,0).
//! Window is 4px tall × full screen width, always-on-top, no decorations.
//!
//! ⚠️ CRITICAL: CreateWindowExW() MUST be called in the SAME THREAD as the
//! message loop (PeekMessage/DispatchMessage). Never call it from another thread.
//! All WinAPI window operations stay inside run_event_loop().

use std::sync::{Arc, Mutex};
use log::info;

/// Shared state — written from Tauri thread, read from WinAPI thread.
pub struct OverlayState {
    pub progress: f32,           // 0.0 – 1.0
    pub color: (u8, u8, u8),     // RGB
    pub visible: bool,
    pub needs_redraw: bool,      // dirty flag — redraw only when changed
}

impl Default for OverlayState {
    fn default() -> Self {
        Self {
            progress: 0.0,
            color: (76, 175, 80), // green
            visible: false,
            needs_redraw: false,
        }
    }
}

pub struct OverlayRenderer {
    state: Arc<Mutex<OverlayState>>,
}

impl OverlayRenderer {
    pub fn new() -> Self {
        let state = Arc::new(Mutex::new(OverlayState::default()));
        let state_clone = Arc::clone(&state);

        // Spawn background thread — all WinAPI calls happen here
        std::thread::Builder::new()
            .name("overlay-renderer".into())
            .spawn(move || {
                // ⚠️ CreateWindowExW() and message loop MUST be in this same thread
                run_event_loop(state_clone);
            })
            .expect("Failed to spawn overlay thread");

        Self { state }
    }

    pub fn update(&self, progress: f32, color: (u8, u8, u8)) {
        if let Ok(mut s) = self.state.lock() {
            s.progress = progress.clamp(0.0, 1.0);
            s.color = color;
            s.needs_redraw = true;
        }
    }

    pub fn show(&self) {
        if let Ok(mut s) = self.state.lock() {
            s.visible = true;
            s.needs_redraw = true;
        }
    }

    pub fn hide(&self) {
        if let Ok(mut s) = self.state.lock() {
            s.visible = false;
            s.needs_redraw = true;
        }
    }
}

/// WinAPI event loop — runs in background thread.
///
/// ⚠️ CreateWindowExW MUST be called here, not in OverlayRenderer::new().
fn run_event_loop(state: Arc<Mutex<OverlayState>>) {
    use windows::Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        UI::WindowsAndMessaging::*,
    };

    // TODO: detect primary monitor width at runtime
    let screen_width = 1920i32;
    let height = 4i32;

    unsafe {
        // Register window class
        // CreateWindowExW() with WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW
        // SetTimer(hwnd, 1, 16, None)  ← 60fps timer

        // Message loop:
        // WM_TIMER → check needs_redraw, if true → InvalidateRect
        // WM_PAINT → BeginPaint → FillRect(bar_width) → EndPaint
        // WM_DESTROY → PostQuitMessage(0)
    }

    info!("🎨 WinAPI overlay window created: {}x{} @ (0,0)", screen_width, height);
}
```

---

### Krok 2: Zmiana `AppState` w `lib.rs`

```rust
// lib.rs — dodać pole overlay do AppState
pub struct AppState {
    pub conn: Arc<ConnectionState>,
    pub session: Arc<Mutex<SessionManager>>,
    pub db: Arc<Mutex<Option<Connection>>>,
    pub config: Arc<Mutex<Option<AppConfig>>>,
    pub overlay: Arc<OverlayRenderer>,   // ← NOWE
}

// W run():
let overlay = Arc::new(OverlayRenderer::new());
// Usunąć: overlay::setup_overlay(app.handle())?;

.manage(AppState {
    // ... existing fields
    overlay,
})
```

---

### Krok 3: Update `tray_controller.rs`

```rust
use crate::commands::AppState;

fn on_state_changed(app: &AppHandle, payload: &StateChangedPayload) {
    // ... existing tray update code ...

    // Update WinAPI overlay
    let app_state: tauri::State<'_, AppState> = app.state();
    let color = color_for_progress_rgb(progress);

    if payload.state == DeskState::Sitting {
        app_state.overlay.update(progress, color);
        app_state.overlay.show();
    } else {
        app_state.overlay.hide();
    }
}

/// Returns RGB tuple for progress ratio.
fn color_for_progress_rgb(progress: f32) -> (u8, u8, u8) {
    if progress < 0.60 { (76, 175, 80) }       // green
    else if progress < 0.85 { (255, 193, 7) }  // amber
    else { (244, 67, 54) }                      // red
}
```

---

## 🧪 Testing Strategy

### Unit Tests (bez WinAPI — pure logic)

```rust
#[test]
fn state_update_sets_needs_redraw() {
    let renderer = OverlayRenderer::new();
    renderer.update(0.5, (255, 193, 7));
    let state = renderer.state.lock().unwrap();
    assert_eq!(state.progress, 0.5);
    assert!(state.needs_redraw, "update() must set needs_redraw");
}

#[test]
fn hide_sets_needs_redraw() {
    let renderer = OverlayRenderer::new();
    renderer.hide();
    let state = renderer.state.lock().unwrap();
    assert!(!state.visible);
    assert!(state.needs_redraw);
}

#[test]
fn color_green_below_60pct() {
    assert_eq!(color_for_progress_rgb(0.3), (76, 175, 80));
}

#[test]
fn color_amber_between_60_and_85() {
    assert_eq!(color_for_progress_rgb(0.7), (255, 193, 7));
}

#[test]
fn color_red_above_85pct() {
    assert_eq!(color_for_progress_rgb(0.9), (244, 67, 54));
}
```

### WinAPI Geometry Tests (Windows-only, integration)

```rust
#[cfg(target_os = "windows")]
#[test]
fn overlay_window_has_correct_dimensions() {
    // After OverlayRenderer::new() + short sleep for thread startup:
    // GetWindowRect(hwnd) → assert height == 4
    // GetWindowLong(hwnd, GWL_STYLE) → assert (style & WS_CAPTION) == 0
    // GetWindowLong(hwnd, GWL_EXSTYLE) → assert (exstyle & WS_EX_TOPMOST) != 0
}
```

---

## 📊 Ryzyka & Mitigacje

| Ryzyko | Mitigacja |
|--------|-----------|
| WinAPI call nie w swoim wątku | `run_event_loop()` jest jedynym miejscem WinAPI — komentarz WARNING |
| DPI scaling → 4px staje się 8px | Użyć `SetProcessDpiAwarenessContext` + `PhysicalSize` zamiast logical |
| Multi-monitor: bar tylko na jednym | V1: primary monitor OK; V2: rozszerzyć na wszystkie monitory |
| Thread crash → app działa bez overlaya | `thread::Builder::spawn` z named thread, logować panic |
| `windows` crate features brakuje | Sprawdzić Cargo.toml przed implementacją, dodać brakujące features |

---

## ❌ NOT IN SCOPE

- **macOS / Linux support** — Raw WinAPI, app jest Windows-only
- **Multi-monitor spanning** — V1 tylko primary monitor
- **Animacja interpolowana** — V1 natychmiastowa zmiana; płynna animacja to V2
- **Dirty flag tuning** — Implementujemy dirty flag, nie optymalizujemy throttlingu
- **Usuwanie `overlay.rs`** — Stary plik zostawiamy (nie blokuje), cleanup osobny PR

---

## 🔍 What Already Exists

| Komponent | Status | Reuse? |
|-----------|--------|--------|
| `overlay.rs` show/hide/update API | Istnieje (Tauri) | ❌ Zastępujemy |
| `tray_controller.rs` state→overlay routing | Istnieje | ✅ Minimalna zmiana |
| `color_for_progress()` logika kolorów | Istnieje | ✅ Przenosimy do `overlay_renderer.rs` |
| `windows` crate v0.61 | W Cargo.toml | ✅ Zero nowych deps |
| `AppState` struct | Istnieje | ✅ Dodajemy pole `overlay` |

---

## ✅ Definition of Done

- [ ] Overlay window pojawia się na (0, 0) primary monitora
- [ ] Okno ma 4px wysokości, pełna szerokość ekranu
- [ ] Brak dekoracji / cienia / zaokrąglenia
- [ ] Always-on-top (ponad innymi oknami)
- [ ] Bar rośnie od 0% → 100% gdy siedzisz
- [ ] Kolory: green (0-60%) → amber (60-85%) → red (85%+)
- [ ] Pokazuje się gdy Sitting, ukrywa się gdy inne stany
- [ ] Dirty flag — redraw tylko gdy state się zmienił
- [ ] Unit testy dla logiki stanu i kolorów
- [ ] WinAPI geometry test (height=4, no WS_CAPTION, WS_EX_TOPMOST)
- [ ] Stary `overlay::setup_overlay()` usunięty z `lib.rs`

---

## 🚀 Implementacja

```
Branch: feat/0001-system-overlay
Files touched:
  src-tauri/src/overlay_renderer.rs  ← NOWY
  src-tauri/src/lib.rs               ← remove setup_overlay, add overlay to AppState
  src-tauri/src/tray_controller.rs   ← use overlay from AppState
  src-tauri/Cargo.toml               ← verify windows features
```

