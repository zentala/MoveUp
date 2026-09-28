# SKILL.md — Healthy Balance Tray App Designer & Builder

## Purpose

Use this skill when designing or implementing the **Healthy Balance** desktop tray/menu-bar app, website, icon system, presentation, or brand materials.

The product is a smart height-adjustable desk companion that tracks sitting/standing rhythm and gently nudges the user to change position.

Core idea:

```txt
Your desk gently helps you move at the right time.
```

---

## When to Use

Use this skill for:

- tray app UI
- macOS menu-bar UI
- Windows 11 tray UI
- sit/stand timeline
- notification design
- icon design
- landing page design
- brand book / ebook
- poster design
- presentation slides
- Codex implementation specs

---

## Design Direction

Style name:

```txt
Frosted System Minimalism
```

Design keywords:

```txt
calm
minimal
premium
system-native
glass
transparent
soft nudge
quiet utility
healthy rhythm
```

Avoid:

```txt
fitness app look
medical app look
cartoon style
loud gamification
generic SaaS landing page
heavy dashboards
```

---

## Visual Motifs

Always prefer:

```txt
minimal desk
vertical movement
sit/stand cycle
soft nudge pulse
daily timeline
healthy balance
small status dot
```

Avoid full clock circles around the desk.

Time should be shown through:

```txt
vertical progress bars
small progress line
subtle markers
timeline rhythm
```

---

## Icon Style

Icons must be:

```txt
monochrome
line-based
single color
rounded caps
rounded joins
intentional line gaps
readable at 16px
usable in Windows tray and macOS menu bar
```

SVG baseline:

```svg
stroke="currentColor"
stroke-width="1.8"
stroke-linecap="round"
stroke-linejoin="round"
fill="none"
```

---

## Timeline Grammar

```txt
blue    = sitting
green   = standing
gray    = away
amber   = reminder
red     = ignored too long
gap     = paused/unknown
dot     = event
```

Timeline should be made of thin vertical bars.

---

## Product Tone

Use calm, short copy.

Good:

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

Bad:

```txt
You failed.
You are unhealthy.
Stand up now!!!
Congratulations champion!!!
```

---

## Platform Rule

```txt
Brand = shared
Container = native
Content = adaptive
```

Windows:

```txt
Acrylic/Mica-inspired
larger popover
stronger shadow
taskbar tray behavior
```

macOS:

```txt
menu-bar popover
template icon
more compact
subtle translucency
```

---

## Core Components

Design and implement:

```txt
TrayPopover
CurrentModeCard
DeskHeightCard
DailyTimeline
SummaryCard
QuickActionBar
SettingsPanel
NotificationPreviewCard
GoalProgress
```

---

## Website Direction

The website should look like an extension of the app.

Hero:

```txt
Sit. Stand. Flow.
A quiet tray app for healthier desk rhythm.
```

Must show:

```txt
floating glass tray popover
Windows/macOS preview
daily vertical-bar timeline
gentle nudges
healthy balance
```

---

## Implementation Rules

For coding:

- Use TypeScript.
- Use Electron + React.
- Keep renderer away from Node APIs.
- Use preload IPC bridge.
- Keep timeline calculations pure.
- Use local persistence first.
- No cloud sync in MVP.
- No real hardware in MVP.
- Add demo mode for screenshots.
- Add tests for timeline calculations.

---

## Quality Checklist

Before accepting work:

- Is current mode visible immediately?
- Is timeline readable?
- Does UI work in light and dark theme?
- Does it feel native on Windows/macOS?
- Are icons readable at 16px?
- Is copy calm?
- Are medical claims avoided?
- Is the renderer isolated from Node?
- Are timeline calculations tested?
