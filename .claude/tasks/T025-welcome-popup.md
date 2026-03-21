# T025 — Welcome / onboarding popup on first launch

**Status:** open
**Priority:** P2
**Branch:** feat/T025-welcome-popup

---

## Goal

Show a friendly welcome popup when the app runs for the first time.

**Two purposes:**
1. Introduce the app warmly to the user
2. Validate that the Tauri WebviewWindow popup mechanism works (same approach as future custom notifications)

---

## Popup Content (exact text in Polish)

```
👋  Cześć! Jestem Twoim osobistym asystentem biurkowym.

Pomagam Ci zadbać o ciało podczas pracy — żebyś się częściej
ruszał i nie zapominał o przerwach na ruch.

Oto jak działam:

🟢  Pasek na górze ekranu pokazuje, jak długo siedzisz.
    Kolor zmienia się: zielony → żółty → czerwony.

🔔  Gdy przesiedzisz za długo, delikatnie Cię przypomnę.

🏆  Gdy wstajesz — nagradzam Cię złotym paskiem i punktami.

────────────────────────────────────────────────────────

Możesz przeciągnąć to okienko w dowolne miejsce na ekranie.
Kliknij ikonę w zasobniku systemowym, aby mnie ukryć lub pokazać.

[ ☐ Nie pokazuj przy następnym uruchomieniu ]

                  [ Gotowy! Zaczynamy! ✨ ]
```

---

## Implementation: Tauri WebviewWindow (NOT WinAPI)

Use Tauri's built-in `WebviewWindowBuilder` — React component rendered in a separate window.

**Why WebviewWindow over WinAPI:**
- Full CSS/React styling for a polished welcome experience
- Draggable via `data-tauri-drag-region` attribute
- Checkbox rendered as normal HTML
- Easier to design beautifully
- Alert popups (T013) use WinAPI because they must overlay fullscreen apps — welcome popup doesn't need that

### React component: `src/components/WelcomePopup.tsx`

```tsx
// Shown in a separate Tauri window (window label: "welcome")
// NOT rendered inside the main App window

import { invoke } from '@tauri-apps/api/core';
import { useState } from 'react';

export function WelcomePopup() {
  const [dontShowAgain, setDontShowAgain] = useState(false);

  const handleTestNotification = async () => {
    try {
      await invoke('trigger_test_notification');
    } catch (e) {
      console.error('Notification test failed:', e);
    }
  };

  const handleDismiss = async () => {
    await invoke('dismiss_welcome', { dontShowAgain });
    // window closes from Rust side
  };

  return (
    <div data-tauri-drag-region style={{ /* styles */ }}>
      <h2>👋 Cześć!</h2>
      {/* ... full content ... */}
      <label>
        <input
          type="checkbox"
          checked={dontShowAgain}
          onChange={e => setDontShowAgain(e.target.checked)}
        />
        Nie pokazuj przy następnym uruchomieniu
      </label>
      <button onClick={handleTestNotification} style={{ marginBottom: 8 }}>
        🔔 Przetestuj powiadomienia
      </button>
      <button onClick={handleDismiss}>Gotowy! Zaczynamy! ✨</button>
    </div>
  );
}
```

Entry point for the welcome window: `src/welcome.tsx` (separate from `src/main.tsx`):
```tsx
import { WelcomePopup } from './components/WelcomePopup';
ReactDOM.createRoot(document.getElementById('root')!).render(<WelcomePopup />);
```

Add `welcome.html` as additional Vite entry in `vite.config.ts`:
```ts
build: {
  rollupOptions: {
    input: {
      main: 'index.html',
      welcome: 'welcome.html',  // ← add this
    }
  }
}
```

### Rust command: `src-tauri/src/commands.rs`

```rust
#[tauri::command]
pub fn dismiss_welcome(
    dont_show_again: bool,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if dont_show_again {
        // Save to config store
        if let Some(store) = app.try_state::<tauri_plugin_store::Store<tauri::Wry>>() {
            let mut config = AppConfig::load(store.inner());
            config.show_welcome_on_startup = false;
            config.save(store.inner()).map_err(|e| e.to_string())?;
        }
    }
    // Close the welcome window
    if let Some(win) = app.get_webview_window("welcome") {
        let _ = win.close();
    }
    Ok(())
}
```

### Rust startup check: `src-tauri/src/lib.rs`

In the app setup (after store is initialized):
```rust
// After store/config init:
let config = AppConfig::load(&store);
if config.show_welcome_on_startup {
    let _ = show_welcome_window(&app_handle);
}
```

```rust
fn show_welcome_window(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    tauri::WebviewWindowBuilder::new(
        app,
        "welcome",
        tauri::WebviewUrl::App("welcome.html".into()),
    )
    .title("zntlDesk — Witaj!")
    .inner_size(480.0, 420.0)
    .resizable(false)
    .always_on_top(true)
    .center()
    .decorations(true)
    .build()?;
    Ok(())
}
```

### Config change: `src-tauri/src/config.rs`

Add two fields to `AppConfig`:
```rust
/// Show welcome popup on startup. Set to false after first dismiss with "don't show again".
#[serde(default = "bool_true")]
pub show_welcome_on_startup: bool,
```

Note: `bool_true()` helper already exists in `config.rs` (returns `true`).

---

## tauri.conf.json — add welcome window

In `tauri.conf.json` under `windows`:
```json
{
  "label": "welcome",
  "title": "zntlDesk — Witaj!",
  "width": 480,
  "height": 420,
  "resizable": false,
  "alwaysOnTop": true,
  "center": true,
  "visible": false
}
```
Set `visible: false` — window is created by Rust code on demand, not at startup automatically.

---

## Settings hook — "Show welcome again"

In the Settings panel (`src/components/SettingsPanel.tsx`), add a button:
```tsx
<button onClick={() => invoke('show_welcome')}>
  Pokaż intro ponownie
</button>
```

Add a `show_welcome` command in `commands.rs` that calls `show_welcome_window()`.

---

## Key Files

| File | Change |
|------|--------|
| `src-tauri/src/config.rs` | Add `show_welcome_on_startup: bool` to `AppConfig` |
| `src-tauri/src/commands.rs` | Add `dismiss_welcome()` and `show_welcome()` commands |
| `src-tauri/src/lib.rs` | Check config on startup, call `show_welcome_window()` |
| `src/components/WelcomePopup.tsx` | New React component (welcome content + dismiss button) |
| `src/welcome.tsx` | New entry point for welcome window |
| `welcome.html` | New HTML entry for Vite |
| `vite.config.ts` | Add `welcome.html` as rollup input |
| `tauri.conf.json` | Add welcome window config |

---

## AppState structure (for reference)

```rust
pub struct AppState {
    pub session: Arc<Mutex<SessionManager>>,
    pub overlay: Arc<OverlayRenderer>,
    pub alert_manager: Arc<Mutex<AlertManager>>,
    pub alert_popup: Arc<Mutex<AlertPopup>>,
    // store is accessed via app.try_state::<Store<Wry>>()
}
```

`AppConfig` is NOT in `AppState` — it's loaded from the store on demand via `AppConfig::load(&store)`.

---

## Tests (~5)

- `show_welcome_on_startup` defaults to `true` in `AppConfig::default()`
- `dismiss_welcome(dont_show_again: false)` — config unchanged, window closes
- `dismiss_welcome(dont_show_again: true)` — `show_welcome_on_startup = false` saved to store
- Second launch with `show_welcome_on_startup = false` — window does NOT open
- Config serde roundtrip: `show_welcome_on_startup` serializes/deserializes correctly

---

## Acceptance Criteria

- [ ] Welcome popup appears on first run
- [ ] "Gotowy! Zaczynamy!" button closes the popup
- [ ] "Nie pokazuj" checkbox → no popup on next launch
- [ ] "Pokaż intro ponownie" button in Settings reopens popup
- [ ] Popup is draggable
- [ ] `cargo test` passes, `pnpm test:unit` passes
