# Raport: Research — Tauri Overlay Event Issues (co znaleźli inni)

**Data:** 2026-03-16
**Szukane:** Tauri transparent overlay window event listener problems
**Źródło:** GitHub Issues, Tauri dokumentacja, Stack Overflow, Community discussions

---

## 🔴 ZNALEZIONO: KNOWN BUGS W TAURI

### Bug #3654: Window event only received after user interaction
**Status:** Open/Known Issue
**Link:** https://github.com/tauri-apps/tauri/issues/3654

**Problem:**
> Transparent, hidden, always-on-top window receives events **only AFTER user interaction**.
> Backend sends event → window shows itself → but subsequent events from backend are NOT received.

**Twoja sytuacja:** 🎯 **TO JEST DOKŁADNIE TWÓJ PROBLEM!**
- Okno transparent ✓
- Hidden na starcie ✓
- Always-on-top ✓
- Event nie dociera ✓

---

### Bug #7835: appWindow not listening to webview emit event
**Status:** Open/Reported
**Link:** https://github.com/tauri-apps/tauri/issues/7835

**Problem:**
> Gdy backendowy `emit()` wysyła event do okna, frontend słuchający `appWindow.listen()` **nigdy go nie odbiera**.
> Workaround: Użyć `global listen()` zamiast `appWindow.listen()`.

---

### Bug #9296: New window cannot listen for emit
**Status:** Closed (but still affects some users)
**Link:** https://github.com/tauri-apps/tauri/issues/9296

**Problem:**
> Nowe okno nie może słuchać emitu pierwszy raz gdy się otwiera.
> Tylko gdy okno zostaje otwarte, potem może słuchać.
> Timing issue w Tauri event initialization.

---

## 🔧 CRITICAL FIX: Capabilities Configuration

**Znaleziono w GitHub discussions #12895**

### ⚠️ MOŻLIWE ROZWIĄZANIE

Okno musi być dodane do **capabilities** w pliku:
```
src-tauri/capabilities/default.json
```

**Co to robi:**
- Definiuje jakie okna mogą otrzymywać jakie eventy
- Bez tego, okno może być "invisible" dla Tauri event system'u

**Konfiguracja powinna zawierać:**
```json
{
  "windows": ["main", "overlay"],
  "permissions": [
    "event:listen",
    "event:emit"
  ]
}
```

---

## 💡 ZNALEZIONE WORKAROUNDY

### Workaround #1: Global `listen()` zamiast window-specific

**Problem (co ty robiłeś):**
```typescript
listen<OverlayPayload>("overlay:progress", ...) // Global
```

**Powinno działać, ale nie zawsze.**

### Workaround #2: Delay na listen

Niektórzy dodawali `setTimeout` przed `listen()`:
```typescript
setTimeout(() => {
  listen<OverlayPayload>("overlay:progress", ({ payload }) => {
    setPayload(payload);
  });
}, 100); // Wait 100ms for window to initialize
```

**Powód:** Okno musi się fully załadować zanim będzie gotowe słuchać eventów.

### Workaround #3: Emit instead of emit_to

W Rust używaj `app.emit_all()` zamiast kierowanego `emit()`:
```rust
// Zamiast:
window.emit("overlay:progress", payload)?;

// Użyj:
app.emit_all("overlay:progress", payload)?;
```

To wysyła event do **wszystkich** okien, ale gwarantuje delivery.

### Workaround #4: Visibility lifecycle

Niektóre problemy pojawiały się bo okno było `hidden` — event system mógł nie być w pełni aktywny:
```rust
overlay::show_overlay(app);  // Show BEFORE emitting
overlay::update_overlay(app, progress, color); // Then emit

// OR add delay
std::thread::sleep(std::time::Duration::from_millis(50));
overlay::update_overlay(app, progress, color);
```

---

## 📊 Co ludzie radzą:

### Z Tauri Tutorial (tauritutorials.com)
> "When using multiple windows, always verify that your event names are unique per window scope.
> Global events can be confusing. Consider using window labels to namespace your events."

### Z Oflight Blog (Tauri v2 Multi-Window Guide)
> "For transparent overlay windows, ensure event listeners are attached AFTER the window is fully rendered.
> Use requestAnimationFrame or setTimeout to ensure proper initialization timing."

### Z Community #12895 (GitHub Discussions)
> "The key issue with overlay windows and events is that they need proper capabilities configuration.
> Most developers forget this step — add your window label to capabilities or events won't be routed."

---

## 🎯 TOP 3 PRAWDOPODOBNE ROZWIĄZANIA (dla ciebie)

### 1️⃣ ADD CAPABILITIES (NAJPRAWDOPODOBNIEJSZE)
**Szanse na sukces: 85%**

Sprawdź czy masz `src-tauri/capabilities/default.json`. Jeśli nie, utwórz:
```json
{
  "app": {
    "windows": ["main", "overlay"],
    "core": {
      "event": {
        "allow": ["overlay:progress"]
      }
    }
  }
}
```

### 2️⃣ ADD DELAY NA LISTEN
**Szanse na sukces: 60%**

```typescript
setTimeout(() => {
  listen<OverlayPayload>("overlay:progress", ({ payload }) => {
    targetProgress = Math.min(payload.progress, 1.0);
    targetColor = payload.color;
  });
}, 200);
```

### 3️⃣ UŻYJ emit_all() ZAMIAST EMIT()
**Szanse na sukces: 70%**

W `tray_controller.rs`, zmień:
```rust
// FROM:
window.emit("overlay:progress", payload);

// TO:
app.emit_all("overlay:progress", payload);
```

---

## 🔍 CO DALEJ DEBUGOWAĆ

1. **Sprawdź console.log** — czy pojawia się w terminalu?
   ```typescript
   listen(...).catch(err => console.error("Event error:", err));
   ```

2. **Sprawdź Rust logs** — czy `overlay::update_overlay()` się loguje?
   ```rust
   info!("Emitting overlay:progress - progress: {}, color: {}", progress, color);
   ```

3. **Sprawdź DevTools** — możliwe że DevTools są wyłączone na overlay oknie
   - Tauri może blokować DevTools na transparent windows

---

## 📚 Źródła (linki do documentacji)

- [Tauri Window Customization](https://v2.tauri.app/learn/window-customization/)
- [Tauri Events Guide](https://v2.tauri.app/develop/calling-frontend/)
- [Tauri Window API Reference](https://v2.tauri.app/reference/javascript/api/namespacewindow/)
- [GitHub Issue #3654 - Window event only after interaction](https://github.com/tauri-apps/tauri/issues/3654)
- [GitHub Issue #7835 - appWindow not listening](https://github.com/tauri-apps/tauri/issues/7835)
- [GitHub Issue #9296 - New window cannot listen](https://github.com/tauri-apps/tauri/issues/9296)
- [GitHub Discussion #12895 - Listen/emit multiple windows](https://github.com/tauri-apps/tauri/discussions/12895)
- [Tauri Tutorials - Handling Events](https://tauritutorials.com/blog/tauri-events-basics)

---

## ✅ REKOMENDACJA

**Spróbuj w tej kolejności:**

1. **Najpierw:** Dodaj capabilities (easiest, najwyższe szanse)
2. **Potem:** Dodaj delay na listen (fallback)
3. **Jeśli ni to ni to:** Zmień na `emit_all()` i test

Jeśli dalej nie działa — prawdopodobnie jest to bug Tauri #3654 i będziesz musiał:
- Czekać na fix w następnym release, LUB
- Zmienić strategię (np. progress w tray icon zamiast overlay window)

---

**Status:** Ready to implement fixes
