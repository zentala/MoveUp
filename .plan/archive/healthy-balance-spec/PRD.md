# PRD.md — Healthy Balance Tray App

## 1. Product Summary

**Healthy Balance** is a desktop tray/menu-bar app for a smart height-adjustable desk.

It tracks the user's sitting, standing, away, and paused time, then uses gentle notifications to help the user change position at the right moment.

Core promise:

> A quiet tray app for healthier desk rhythm.

Primary behavior:

- Track sitting and standing sessions.
- Show a daily timeline made of thin vertical bars.
- Notify the user when it is time to change position.
- Let the user manually switch between sitting, standing, away, and paused.
- Support mock desk height and manual raise/lower controls in MVP.
- Keep the same brand identity across Windows and macOS while respecting platform-native shell behavior.

---

## 2. Product Goals

### 2.1 Goals

- Create a calm, premium desktop tray app.
- Make sitting/standing rhythm visible at a glance.
- Reduce long uninterrupted sitting sessions.
- Avoid annoying or shame-based reminders.
- Provide a foundation for future smart desk hardware integration.
- Build a design system that works in dark and light themes.

### 2.2 MVP Scope

MVP must include:

- Windows tray / macOS menu-bar icon.
- Tray/menu-bar popover.
- Current mode display:
  - Sitting
  - Standing
  - Away
  - Paused
- Manual mode switching.
- Daily vertical-bar timeline.
- Sitting time, standing time, away time, paused time.
- Switch count.
- Next reminder countdown.
- Native OS notifications.
- Snooze and pause reminders.
- Settings screen.
- Light/dark/system theme.
- Local persistence.
- Demo mode for screenshots and presentations.

### 2.3 Non-goals for MVP

Do not implement in MVP:

- Real smart desk hardware control.
- Bluetooth or USB desk protocols.
- Cloud sync.
- Login/account system.
- Mobile app.
- Team dashboard.
- Health/medical claims.
- Complex gamification.
- Paid subscriptions.

---

## 3. Target Platforms

Primary:

- Windows 11
- macOS

Secondary / later:

- Linux tray app

Recommended stack:

```txt
Electron
React
TypeScript
Vite
Zustand
Vitest
local JSON persistence
CSS variables
```

MVP stack:

```txt
Electron + React + TypeScript + Vite + Zustand + Vitest
```

---

## 4. Product Principles

### 4.1 UX Tone

The product should feel like:

```txt
quiet helper > system utility > wellness app > fitness app
```

The app should not shame, pressure, or over-celebrate.

Good copy:

```txt
Time to move.
Stand and recharge.

Nice reset.
You changed position.

Welcome back.
Let’s restart your rhythm.

Good balance today.
You’re on track.
```

Avoid:

```txt
You failed.
You are unhealthy.
Stand up now!!!
Congratulations champion!!!
```

### 4.2 Product Safety

Do not claim that the app cures, treats, prevents, or fixes medical problems.

Use safe language:

```txt
healthier rhythm
better movement habit
less uninterrupted sitting
desk balance
work routine
```

Avoid:

```txt
fix back pain
prevent disease
medical treatment
guaranteed health improvement
```

---

## 5. Main User Stories

### 5.1 Open Tray Popover

As a user, I want to click the tray/menu-bar icon and see my current desk rhythm.

Acceptance criteria:

- Tray/menu-bar icon is visible after launch.
- Clicking icon opens the popover near the tray/menu-bar.
- Clicking outside closes the popover.
- Popover does not appear as a normal taskbar/dock app window by default.
- Platform behavior feels native.

### 5.2 Switch Current Mode

As a user, I want to manually mark whether I am sitting, standing, away, or paused.

Acceptance criteria:

- Current mode is visible.
- User can change mode from Status screen.
- Active timeline segment closes when mode changes.
- New timeline segment starts immediately.
- UI updates without reload.
- Data persists locally.

### 5.3 View Daily Timeline

As a user, I want to see my day as a clean sit/stand timeline.

Acceptance criteria:

- Timeline uses thin vertical bars.
- Segment colors:
  - Sitting = blue
  - Standing = green
  - Away = gray
  - Paused = muted gray
  - Reminder = amber marker
- Current time marker is visible.
- Time labels are shown.
- Legend is available.
- Timeline persists after restart.

### 5.4 Get Gentle Reminders

As a user, I want the app to remind me to change position without annoying me.

Acceptance criteria:

- Reminder fires after configured interval.
- Reminder is suppressed during paused mode.
- Reminder is suppressed during quiet hours.
- Snooze postpones the next reminder.
- Notification title/body use calm copy.
- Notification appears using native OS mechanism.

### 5.5 Complete Standing Session

As a user, I want positive feedback when I stand for a meaningful period.

Acceptance criteria:

- When standing session reaches configured target, show calm success notification.
- Example: `Nice reset. You stood for 15 minutes.`
- Update stats.
- Do not show loud celebration/confetti.

### 5.6 Welcome Back

As a user, I want the app to recognize when I return after being away.

Acceptance criteria:

- Manual away mode exists.
- MVP may use OS idle time if available.
- On return, app can show `Welcome back. Let’s restart your rhythm.`
- User can select whether they returned sitting or standing.

---

## 6. Information Architecture

Main popover tabs:

```txt
Status
Timeline
Stats
Settings
```

### 6.1 Status Screen

Contains:

- Header with app name and mode status.
- Current mode card.
- Current session duration.
- Mock desk height card.
- Mini timeline.
- Summary cards:
  - Sitting
  - Standing
  - Switches
  - Next reminder
- Quick actions:
  - Sit
  - Stand
  - Away
  - Pause
  - Snooze
  - Settings

### 6.2 Timeline Screen

Contains:

- Full-day vertical timeline.
- Current time marker.
- Legend.
- Time labels.
- Event markers.
- Optional filter/toggle for workday/full day.

### 6.3 Stats Screen

Contains:

- Today summary.
- Standing percentage.
- Sitting percentage.
- Switch count.
- Daily goal progress.
- Weekly mini chart placeholder.

### 6.4 Settings Screen

Contains:

- Theme: System / Light / Dark.
- Reminder interval.
- Standing session goal.
- Daily standing target.
- Notifications toggle.
- Quiet hours.
- Idle detection toggle.
- Launch at startup.
- Show in taskbar/dock.
- Demo mode.
- Reset data.

---

## 7. Domain Model

```ts
export type WorkMode = 'sitting' | 'standing' | 'away' | 'paused';

export interface TimelineSegment {
  id: string;
  mode: WorkMode;
  startedAt: string;
  endedAt?: string;
  source: 'manual' | 'idle-detection' | 'desk-device' | 'mock';
}

export interface ReminderEvent {
  id: string;
  type: 'time-to-move' | 'stand-complete' | 'welcome-back' | 'goal-reached';
  createdAt: string;
  acknowledgedAt?: string;
  snoozedUntil?: string;
}

export interface AppSettings {
  theme: 'system' | 'light' | 'dark';
  reminderIntervalMinutes: number;
  standingSessionGoalMinutes: number;
  dailyStandingTargetPercent: number;
  quietHoursEnabled: boolean;
  quietHoursStart: string;
  quietHoursEnd: string;
  launchAtStartup: boolean;
  showInDockOrTaskbar: boolean;
  notificationsEnabled: boolean;
  idleDetectionEnabled: boolean;
  demoMode: boolean;
  demoScenario: DemoScenario;
}

export type DemoScenario =
  | 'balanced'
  | 'too-much-sitting'
  | 'great-day'
  | 'new-user';

export interface DailySummary {
  date: string;
  sittingMinutes: number;
  standingMinutes: number;
  awayMinutes: number;
  pausedMinutes: number;
  switches: number;
  standingPercent: number;
}
```

---

## 8. Calculation Rules

### 8.1 Duration

For each timeline segment:

- Clamp segment to the selected day.
- If `endedAt` is missing, use current time.
- Ignore invalid negative durations.
- Sum duration into the matching mode bucket.

### 8.2 Standing Percentage

```txt
standingMinutes / (sittingMinutes + standingMinutes) * 100
```

Away and paused time are excluded.

### 8.3 Switch Count

Count transitions only between:

```txt
sitting -> standing
standing -> sitting
```

Do not count:

```txt
away -> sitting
paused -> standing
sitting -> paused
```

### 8.4 Current Session

```txt
currentSessionDuration = now - currentSegment.startedAt
```

---

## 9. Notifications

### 9.1 Notification Types

```ts
export type NotificationType =
  | 'time-to-move'
  | 'stand-complete'
  | 'welcome-back'
  | 'goal-reached';
```

### 9.2 Copy

```txt
Time to move
Stand and recharge.

Nice reset
You stood for 15 minutes.

Welcome back
Let’s restart your rhythm.

Good balance today
You’re on track.
```

### 9.3 Rules

- No notifications during quiet hours.
- No reminders while paused.
- No reminders while away.
- Snooze suppresses reminders until `snoozedUntil`.
- Do not fire duplicate reminders within the same minute.
- Standing complete fires once per standing session.

---

## 10. Privacy

MVP rules:

- All data stays local.
- No analytics by default.
- No account required.
- No telemetry.
- Settings should state: `Your data stays on this device.`

---

## 11. Definition of Done

MVP is done when:

- App launches on Windows.
- App launches on macOS or has platform-ready abstractions.
- Tray/menu-bar icon works.
- Popover works.
- User can switch modes.
- Timeline records and persists segments.
- Status screen shows current state and summary.
- Timeline screen shows vertical bars.
- Notifications work.
- Snooze/pause works.
- Settings persist.
- Light/dark/system theme works.
- Demo mode works.
- Domain logic is tested.
- Renderer does not access Node APIs directly.
