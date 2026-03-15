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

---

## Inne

- Konfiguracja progów wysokości przez UI (kalibracja: "ustaw biurko na siedzącą pozycję i kliknij")
- Reguły sesji konfigurowalne z UI (limit minut, progi przerwy)
- Eksport danych do CSV
