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

## Standing Mode Theme — Floating Window

When user stands, floating window changes background to subtle gold/green tint.
- Emit `desk:overlay-mode-changed` event from tray_controller.rs
- React component listens and applies CSS class `standing-mode` to root
- Style: `background: rgba(218, 165, 32, 0.05)` — barely visible warmth
- Implement after T028 (points in floating window) when UI is being touched anyway

---

## App Icon — True Vector SVG

- `src-tauri/icons/icon-source.svg` is NOT a real vector — it's a PNG rasterized image wrapped in an SVG container (base64-encoded `data:image/png` inside `<image href=...>`). Colors cannot be changed via CSS/attributes.
- **To do:** Replace with a proper SVG drawn with `<path>` / `<rect>` elements so it can be recolored and scaled infinitely.
- Search terms: Noun Project `standing desk`, SVG Repo `adjustable desk`, Flaticon `sit stand`
- Or: design from scratch in Inkscape/Figma — a simple desk silhouette (horizontal top + two legs at different heights) works at 16px.
- Once real SVG exists: use `pnpm tauri icon source.png` to auto-generate all required sizes.

---

## Alert System — Future

- **AlertManager message strings in settings panel** — `AlertConfig` holds default neutral/positive message arrays. When T001 settings panel ships, expose these as editable arrays so zentala can tune tone/wording without code changes. Depends on T001 + T015.

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

## Session Limit & Alerts

- **Bar flashing at limit** — when 45min reached, bar pulses/flashes red to grab attention
- **Red popup at limit** — "You've been sitting 45 min. Take a break!" with Dismiss button
  - Auto-dismiss when desk raised (Standing detected)
  - Snooze logic: Dismiss → remind after 2 hours. "Your body will thank you."
  - Must design the full UX flow: what happens on ignore? configurable?
- **Notification system architecture** — multiple notification types needed:
  - Colors: green (success), yellow (warning), red (alert), gray (info)
  - Two backends to prototype: (1) native Windows toast, (2) custom-drawn overlay popup
  - Need demo of both to compare before committing
  - Custom popup: animated progress bar at bottom, auto-dismiss timer, configurable
  - Some notifications should persist until action, some auto-dismiss — make it a setting
- **Success notifications** — gamification nudges:
  - "You stood 40min today — top 2% of users!" (green)
  - Daily standing goal milestones
  - Streak tracking ("3 days in a row of 40min+ standing")
  - Future: more gamification mechanics TBD through experimentation

## Tray Icon

- **Color dot on tray icon** — small circle (not whole icon) that matches bar color progression:
  - Green (OK) → Yellow (warning, 60%+) → Red (alert, 85%+)
  - Synced with overlay bar progress, same color_for_progress() logic
  - Note: T010 in main TASKS.md covers dynamic tray icon but needs redesign to be a small dot, not full icon recolor

## Inne

- Konfiguracja progów wysokości przez UI (kalibracja: "ustaw biurko na siedzącą pozycję i kliknij")
- Reguły sesji konfigurowalne z UI (limit minut, progi przerwy)
- Eksport danych do CSV
