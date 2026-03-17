# BACKLOG.md — Desk App

Items not yet scheduled, before refinement.

---

## Tauri Plugins — do dodania

- **`window-vibrancy`** (crate) — Windows Acrylic/Mica blur effect na floating window
  - wymaga `"transparent": true` w `tauri.conf.json` + `background: transparent` w CSS

- **`tauri-plugin-notification`** — natywne powiadomienia Windows ("czas wstać")

- **`tauri-plugin-autostart`** — start aplikacji przy logowaniu do Windows

- **`tauri-plugin-store`** — persystencja konfiguracji (limit sesji, progi wysokości, kalibracja)

- **Dynamiczna ikona tray** — wbudowane Tauri 2 (`tray-icon` feature), `set_icon()` — zmiana koloru/ikony wg stanu (OK / warning / overdue)
  - **Status:** Scheduled for future (po stabilizacji progress bar overlay)

---

## Dokumentacja

- **Overlay Progress Bar Architecture Document** — pełny opis systemu paska na górze ekranu
  - Architektura: OverlayRenderer, OverlayState, thread-safe state management
  - Synchronizacja z session state (Sitting/Standing/Walking)
  - Logika kolorów (green → amber → red)
  - Linkowanie do testów: `overlay_renderer::tests`, `colors::tests`
  - Linkowanie do plików: `overlay_renderer.rs`, `colors.rs`, `tray_controller.rs`
  - Diagram przepływu: State → TrayController → OverlayRenderer → WinAPI Window
  - Aby uniknąć rozjazdów: dokumentacja = Single Source of Truth dla jak pasek działa

---

## Inne

- Konfiguracja progów wysokości przez UI (kalibracja: "ustaw biurko na siedzącą pozycję i kliknij")
- Reguły sesji konfigurowalne z UI (limit minut, progi przerwy)
- Eksport danych do CSV
