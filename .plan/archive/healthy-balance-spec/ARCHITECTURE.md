# ARCHITECTURE.md — Healthy Balance

## 1. Architecture Summary

Healthy Balance is an Electron desktop app with:

```txt
Electron main process
Electron preload bridge
React renderer process
Local persistence
Native OS notifications
Tray/menu-bar integration
```

Strict rule:

> Renderer must not access Node.js APIs directly.

All file system, tray, notification, and OS integration must go through Electron main process using a typed IPC/preload API.

---

## 2. Process Responsibilities

### 2.1 Main Process

Responsibilities:

- Create tray/menu-bar icon.
- Manage popover window.
- Position popover near tray/menu-bar.
- Handle native notifications.
- Read/write local files.
- Manage launch-at-startup setting.
- Detect platform.
- Optional: detect OS idle time.
- Provide IPC handlers.

Files:

```txt
src/main/index.ts
src/main/tray.ts
src/main/windows.ts
src/main/notifications.ts
src/main/persistence.ts
src/main/platform.ts
src/main/idle.ts
```

### 2.2 Preload

Responsibilities:

- Expose safe typed API to renderer.
- Hide raw IPC details.
- Validate basic arguments where practical.

Files:

```txt
src/preload/index.ts
src/preload/electron-api.ts
```

### 2.3 Renderer

Responsibilities:

- React UI.
- State management.
- Theme switching.
- Timeline rendering.
- Settings UI.
- Calls to `window.hb`.

Files:

```txt
src/renderer/app
src/renderer/components
src/renderer/features
src/renderer/styles
```

---

## 3. Recommended Folder Structure

```txt
src/
  main/
    index.ts
    tray.ts
    windows.ts
    notifications.ts
    persistence.ts
    platform.ts
    idle.ts

  preload/
    index.ts
    electron-api.ts

  shared/
    types/
      tracking.types.ts
      settings.types.ts
      reminders.types.ts
      platform.types.ts
    constants/
      copy.ts
      theme.ts
    utils/
      date.ts
      clamp.ts

  renderer/
    app/
      App.tsx
      store.ts
      routes.tsx

    components/
      AppShell/
      Header/
      CurrentModeCard/
      DeskHeightCard/
      DailyTimeline/
      TimelineLegend/
      SummaryCard/
      QuickActionBar/
      SettingsPanel/
      GoalProgress/
      Icon/

    features/
      tracking/
        tracking.store.ts
        tracking.selectors.ts
        tracking.utils.ts
        tracking.demo.ts
      reminders/
        reminders.scheduler.ts
        reminders.copy.ts
      settings/
        settings.defaults.ts
        settings.store.ts

    styles/
      tokens.css
      global.css
      glass.css
      themes.css

    assets/
      icons/
      tray/
      logo/
```

---

## 4. IPC Contract

The renderer should access Electron through:

```ts
window.hb
```

Use this interface:

```ts
export interface HealthyBalanceApi {
  platform: {
    getPlatform(): Promise<AppPlatform>;
    getThemeSource(): Promise<'system' | 'light' | 'dark'>;
  };

  settings: {
    get(): Promise<AppSettings>;
    save(settings: AppSettings): Promise<void>;
    reset(): Promise<AppSettings>;
    setLaunchAtStartup(enabled: boolean): Promise<void>;
  };

  timeline: {
    getToday(): Promise<TimelineSegment[]>;
    saveToday(segments: TimelineSegment[]): Promise<void>;
    getByDate(date: string): Promise<TimelineSegment[]>;
    saveByDate(date: string, segments: TimelineSegment[]): Promise<void>;
    resetToday(): Promise<void>;
  };

  reminders: {
    showNotification(input: NotificationInput): Promise<void>;
  };

  window: {
    closePopover(): Promise<void>;
    resizePopover(size: { width: number; height: number }): Promise<void>;
  };

  demo: {
    generateToday(scenario: DemoScenario): Promise<TimelineSegment[]>;
  };
}
```

Types:

```ts
export type AppPlatform = 'windows' | 'macos' | 'linux';

export interface NotificationInput {
  type: 'time-to-move' | 'stand-complete' | 'welcome-back' | 'goal-reached';
  title: string;
  body: string;
  silent?: boolean;
}
```

---

## 5. Persistence

### 5.1 Location

Use:

```ts
app.getPath('userData')
```

Suggested structure:

```txt
Healthy Balance/
  settings.json
  timeline/
    2026-06-08.json
    2026-06-09.json
  reminders/
    2026-06-08.json
```

### 5.2 Atomic Save

Use safe write:

```txt
write file.tmp
fsync if practical
rename file.tmp -> file.json
```

### 5.3 Validation

On read:

- If file missing, return defaults.
- If JSON invalid, rename to `.corrupt` and return defaults.
- Never crash renderer because of bad persistence.

---

## 6. State Management

Use Zustand in renderer.

Store responsibilities:

- current mode
- current segment
- timeline segments
- settings
- derived summaries
- reminder state
- demo mode

Suggested store shape:

```ts
interface AppStore {
  initialized: boolean;

  currentMode: WorkMode;
  currentSegment: TimelineSegment | null;
  segments: TimelineSegment[];
  settings: AppSettings;
  nextReminderAt: string | null;

  initialize(): Promise<void>;
  setMode(mode: WorkMode, source?: TimelineSegment['source']): Promise<void>;
  snoozeReminder(minutes: number): void;
  pauseTracking(): Promise<void>;
  resumeTracking(mode: 'sitting' | 'standing'): Promise<void>;
  updateSettings(patch: Partial<AppSettings>): Promise<void>;
  resetToday(): Promise<void>;
  loadDemoScenario(scenario: DemoScenario): Promise<void>;
}
```

Domain logic should be pure and testable.

Do not put timeline calculation directly inside React components.

---

## 7. Domain Utilities

Create pure functions:

```ts
calculateDailySummary(segments, date, now): DailySummary
closeOpenSegment(segments, now): TimelineSegment[]
startSegment(mode, now, source): TimelineSegment
switchMode(segments, nextMode, now, source): TimelineSegment[]
clampSegmentToDay(segment, date): TimelineSegment | null
calculateNextReminder(currentSegment, settings, now): Date | null
shouldSendReminder(state, now): boolean
```

These functions require unit tests.

---

## 8. Popover Window Behavior

### 8.1 Shared

- Frameless.
- Transparent or translucent where supported.
- Resizable only internally if needed.
- Closes on blur.
- Does not steal focus unnecessarily.
- Opens near tray/menu-bar icon.

### 8.2 Windows

- Position above taskbar tray.
- Use larger size:
  - compact: 360 × 520
  - expanded: 420 × 620
- Stronger shadow.
- Acrylic-like UI created in CSS.

### 8.3 macOS

- Position below menu-bar icon.
- Use compact size:
  - compact: 340 × 500
  - expanded: 390 × 580
- Template menu-bar icon.
- Subtler borders and more translucency.

---

## 9. Notifications Architecture

Main process owns native notifications.

Renderer schedules intent; main process displays notification.

Renderer may call:

```ts
window.hb.reminders.showNotification(input)
```

MVP reminder scheduler can run in renderer store, but long-term should move to main process.

Rules:

- No duplicate notification in the same minute.
- Respect quiet hours.
- Respect pause.
- Respect away.
- Respect notification toggle.

---

## 10. Demo Mode

Demo mode is required.

Purpose:

- screenshots
- landing page
- presentations
- testing UI density

Scenarios:

```txt
balanced
too-much-sitting
great-day
new-user
```

Demo data generator must create realistic segments for today.

The generator must be deterministic if given a date/scenario.

---

## 11. Testing Strategy

Use Vitest.

Test pure domain logic first:

```txt
tracking.utils.test.ts
reminders.scheduler.test.ts
demo.generator.test.ts
settings.defaults.test.ts
```

Renderer component tests are optional in MVP.

Manual QA required for tray behavior.

---

## 12. Technical Constraints

- TypeScript strict mode.
- Avoid `any`.
- Keep shared types in `src/shared`.
- No direct Node access from renderer.
- No cloud calls.
- No telemetry.
- No hardcoded absolute paths.
- Use CSS variables for theme.
- Avoid large UI libraries for MVP unless necessary.

---

## 13. Future Extensions

Possible later modules:

- Desk hardware adapter.
- Bluetooth/USB/HID support.
- Auto-detect mode from desk height.
- Keyboard/mouse idle detection.
- Weekly/monthly stats.
- Export data.
- Device calibration.
- Multiple desks.
- Full dashboard window.
