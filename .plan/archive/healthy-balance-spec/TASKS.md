# TASKS.md — Codex Implementation Plan

## Milestone 0 — Repository Hygiene

- [ ] Create project with Electron + React + TypeScript + Vite.
- [ ] Enable TypeScript strict mode.
- [ ] Add ESLint.
- [ ] Add Prettier.
- [ ] Add Vitest.
- [ ] Add path aliases.
- [ ] Add `src/shared` for shared types.
- [ ] Add basic README with dev commands.

Acceptance criteria:

- `pnpm install` works.
- `pnpm dev` starts the app.
- `pnpm test` runs.
- TypeScript builds without errors.

---

## Milestone 1 — Desktop Shell

- [ ] Create Electron main process.
- [ ] Create preload bridge.
- [ ] Create React renderer.
- [ ] Add tray/menu-bar icon.
- [ ] Add popover window.
- [ ] Open popover on tray/menu-bar click.
- [ ] Close popover on blur.
- [ ] Hide from taskbar/dock by default.
- [ ] Add platform detection.

Acceptance criteria:

- App launches.
- Tray/menu-bar icon is visible.
- Clicking icon opens popover.
- Clicking outside closes popover.
- Renderer accesses Electron only through preload API.

---

## Milestone 2 — Design Foundation

- [ ] Add `tokens.css`.
- [ ] Add `themes.css`.
- [ ] Add `glass.css`.
- [ ] Implement light theme.
- [ ] Implement dark theme.
- [ ] Implement system theme.
- [ ] Add `AppShell`.
- [ ] Add base typography.
- [ ] Add button styles.
- [ ] Add card styles.

Acceptance criteria:

- Theme can be switched.
- Theme persists.
- App visually matches Frosted System Minimalism direction.
- UI has glass surfaces and correct semantic colors.

---

## Milestone 3 — Data Types and Defaults

- [ ] Add `WorkMode`.
- [ ] Add `TimelineSegment`.
- [ ] Add `ReminderEvent`.
- [ ] Add `AppSettings`.
- [ ] Add `DailySummary`.
- [ ] Add default settings.
- [ ] Add notification copy constants.
- [ ] Add semantic color constants.

Acceptance criteria:

- Shared types are imported from `src/shared`.
- No duplicate domain types in renderer.
- Defaults are tested.

---

## Milestone 4 — Persistence Layer

- [ ] Implement settings read/write in main process.
- [ ] Implement timeline read/write in main process.
- [ ] Use `app.getPath('userData')`.
- [ ] Add corrupt JSON recovery.
- [ ] Add atomic write helper.
- [ ] Expose persistence through preload API.

Acceptance criteria:

- Settings persist after restart.
- Timeline persists after restart.
- Invalid JSON does not crash the app.
- Renderer does not use `fs`.

---

## Milestone 5 — App Store

- [ ] Add Zustand store.
- [ ] Add `initialize`.
- [ ] Load settings.
- [ ] Load today timeline.
- [ ] Track current mode.
- [ ] Track current segment.
- [ ] Implement `setMode`.
- [ ] Implement `pauseTracking`.
- [ ] Implement `resumeTracking`.
- [ ] Implement `updateSettings`.
- [ ] Save state after changes.

Acceptance criteria:

- Mode switching updates UI.
- Open segment is closed properly.
- New segment starts properly.
- Timeline is persisted.

---

## Milestone 6 — Domain Logic

- [ ] Implement `calculateDailySummary`.
- [ ] Implement `clampSegmentToDay`.
- [ ] Implement `switchMode`.
- [ ] Implement `calculateNextReminder`.
- [ ] Implement `shouldSendReminder`.
- [ ] Implement switch-count calculation.
- [ ] Add unit tests.

Acceptance criteria:

- All timeline calculation tests pass.
- Away/paused are excluded from standing percentage.
- Switch count only counts sitting/standing transitions.
- Midnight crossing is handled.

---

## Milestone 7 — Status Screen

- [ ] Build `Header`.
- [ ] Build `CurrentModeCard`.
- [ ] Build `DeskHeightCard`.
- [ ] Build `SummaryCard`.
- [ ] Build `QuickActionBar`.
- [ ] Add quick actions:
  - Sit
  - Stand
  - Away
  - Pause
  - Snooze
- [ ] Add current session timer.
- [ ] Add mock desk height.

Acceptance criteria:

- User can operate MVP from Status screen.
- Current session duration updates.
- Summary cards update.
- UI remains compact.

---

## Milestone 8 — Timeline Component

- [ ] Build `DailyTimeline`.
- [ ] Render vertical bars.
- [ ] Add semantic colors.
- [ ] Add compact mode.
- [ ] Add full mode.
- [ ] Add current time marker.
- [ ] Add time labels.
- [ ] Add legend.
- [ ] Add event markers placeholder.

Acceptance criteria:

- Mini timeline fits Status screen.
- Full timeline fits Timeline screen.
- Timeline updates when mode changes.
- Color grammar matches DESIGN_SYSTEM.md.

---

## Milestone 9 — Navigation and Screens

- [ ] Add tab navigation.
- [ ] Add Status screen.
- [ ] Add Timeline screen.
- [ ] Add Stats screen.
- [ ] Add Settings screen.
- [ ] Add screen-level layout components.
- [ ] Preserve selected tab while popover is open.

Acceptance criteria:

- All tabs work.
- No broken empty screens.
- Settings are editable.
- Stats use real calculated data.

---

## Milestone 10 — Notifications

- [ ] Add native notification support.
- [ ] Add reminder scheduler.
- [ ] Add time-to-move notification.
- [ ] Add stand-complete notification.
- [ ] Add welcome-back notification.
- [ ] Add snooze.
- [ ] Add quiet hours.
- [ ] Add notification toggle.
- [ ] Add tests for reminder logic.

Acceptance criteria:

- Reminder fires after configured interval.
- Snooze delays reminder.
- Pause suppresses reminders.
- Quiet hours suppress reminders.
- Notification copy is calm.

---

## Milestone 11 — Settings

- [ ] Theme setting.
- [ ] Reminder interval.
- [ ] Standing session goal.
- [ ] Daily standing target.
- [ ] Notifications toggle.
- [ ] Quiet hours.
- [ ] Idle detection toggle placeholder.
- [ ] Launch at startup.
- [ ] Show in taskbar/dock.
- [ ] Reset today.
- [ ] Reset all data.
- [ ] Privacy note.

Acceptance criteria:

- Settings persist.
- Changing settings affects behavior.
- Reset actions require confirmation.
- Privacy note is visible.

---

## Milestone 12 — Demo Mode

- [ ] Add demo data generator.
- [ ] Add scenarios:
  - balanced
  - too-much-sitting
  - great-day
  - new-user
- [ ] Add deterministic generation by date/scenario.
- [ ] Add settings UI for demo mode.
- [ ] Add reset demo data button.
- [ ] Add tests for generator.

Acceptance criteria:

- Demo mode can create realistic data.
- Screenshots look populated.
- Demo does not corrupt real user data without confirmation.

---

## Milestone 13 — Platform Polish

- [ ] Windows popover size and position.
- [ ] macOS popover size and position.
- [ ] Windows tray icon variants.
- [ ] macOS template icon.
- [ ] App startup behavior.
- [ ] Dock/taskbar visibility behavior.
- [ ] Reduced motion support.
- [ ] Keyboard navigation.

Acceptance criteria:

- Windows feels native.
- macOS feels native.
- Core content stays visually consistent.
- UI is accessible enough for MVP.

---

## Milestone 14 — QA

- [ ] Run unit tests.
- [ ] Run typecheck.
- [ ] Test launch/restart.
- [ ] Test invalid persistence recovery.
- [ ] Test theme switching.
- [ ] Test notifications.
- [ ] Test quiet hours.
- [ ] Test timeline after several mode switches.
- [ ] Test demo mode.
- [ ] Test popover close-on-blur.
- [ ] Test high DPI display.

Acceptance criteria:

- No critical runtime errors.
- No TypeScript errors.
- MVP flows are usable.
