# DESIGN_SYSTEM.md — Healthy Balance

## 1. Design Direction

Style name:

```txt
Frosted System Minimalism
```

The app should feel like a native desktop companion, not a web dashboard.

Core visual identity:

```txt
minimal desk
vertical movement
soft nudge / pulse
sit-stand rhythm
daily timeline
healthy balance cycle
```

The design must work across:

- Windows 11 tray
- macOS menu bar
- Website landing page
- Brand book / ebook
- Posters and presentation slides

---

## 2. Brand Personality

Keywords:

```txt
calm
minimal
premium
system-native
quiet
precise
healthy without being medical
productive without being corporate
```

Avoid:

```txt
cartoon
gym app
medical app
aggressive productivity
loud gamification
generic SaaS landing page
```

---

## 3. Color System

### 3.1 Semantic Colors

```ts
export const colors = {
  sitting: '#3FA7FF',
  standing: '#7ED957',
  reminder: '#FFB84D',
  danger: '#FF6868',
  away: '#9AA4B2',
  paused: '#8B8F98',
};
```

### 3.2 Light Theme

```ts
export const lightTheme = {
  bg: '#F6F8FB',
  surface: 'rgba(255, 255, 255, 0.68)',
  surfaceStrong: 'rgba(255, 255, 255, 0.84)',
  border: 'rgba(20, 30, 45, 0.10)',
  text: '#101828',
  muted: '#667085',
};
```

### 3.3 Dark Theme

```ts
export const darkTheme = {
  bg: '#070A0E',
  surface: 'rgba(12, 18, 24, 0.72)',
  surfaceStrong: 'rgba(16, 23, 32, 0.88)',
  border: 'rgba(255, 255, 255, 0.10)',
  text: '#F5F7FA',
  muted: '#AAB4C0',
};
```

---

## 4. Glass / Acrylic Surfaces

### 4.1 Shared Surface Principles

Surfaces should have:

- translucent background
- blur
- subtle border
- soft shadow
- inset highlight
- rounded corners

### 4.2 Windows Light

```css
.hb-surface.windows.light {
  background: rgba(255, 255, 255, 0.72);
  backdrop-filter: blur(32px) saturate(1.35);
  -webkit-backdrop-filter: blur(32px) saturate(1.35);
  border: 1px solid rgba(255, 255, 255, 0.75);
  box-shadow:
    0 28px 80px rgba(30, 60, 120, 0.16),
    inset 0 1px 0 rgba(255, 255, 255, 0.80);
}
```

### 4.3 Windows Dark

```css
.hb-surface.windows.dark {
  background: rgba(8, 12, 18, 0.76);
  backdrop-filter: blur(32px) saturate(1.25);
  -webkit-backdrop-filter: blur(32px) saturate(1.25);
  border: 1px solid rgba(255, 255, 255, 0.10);
  box-shadow:
    0 28px 90px rgba(0, 0, 0, 0.38),
    inset 0 1px 0 rgba(255, 255, 255, 0.08);
}
```

### 4.4 macOS Light

```css
.hb-surface.macos.light {
  background: rgba(255, 255, 255, 0.58);
  backdrop-filter: blur(40px) saturate(1.55);
  -webkit-backdrop-filter: blur(40px) saturate(1.55);
  border: 1px solid rgba(255, 255, 255, 0.55);
  box-shadow: 0 22px 70px rgba(20, 40, 80, 0.12);
}
```

### 4.5 macOS Dark

```css
.hb-surface.macos.dark {
  background: rgba(18, 22, 28, 0.58);
  backdrop-filter: blur(44px) saturate(1.45);
  -webkit-backdrop-filter: blur(44px) saturate(1.45);
  border: 1px solid rgba(255, 255, 255, 0.08);
  box-shadow: 0 24px 80px rgba(0, 0, 0, 0.32);
}
```

---

## 5. Typography

Use system fonts.

```css
font-family:
  system-ui,
  -apple-system,
  BlinkMacSystemFont,
  "Segoe UI",
  sans-serif;
```

### 5.1 Type Scale

```css
--font-xs: 11px;
--font-sm: 12px;
--font-md: 14px;
--font-lg: 18px;
--font-xl: 24px;
--font-2xl: 36px;
```

### 5.2 Typography Rules

- Use short labels.
- Avoid long paragraphs in popover.
- Use `font-weight: 600` for card titles.
- Use uppercase kicker labels with letter spacing.
- Keep notification copy concise.

---

## 6. Spacing and Radius

```css
--space-1: 4px;
--space-2: 8px;
--space-3: 12px;
--space-4: 16px;
--space-5: 24px;
--space-6: 32px;
--space-7: 40px;

--radius-sm: 8px;
--radius-md: 14px;
--radius-lg: 20px;
--radius-xl: 28px;
--radius-pill: 999px;
```

Popover radius:

```txt
Windows: 24–28px
macOS: 18–22px
```

---

## 7. Icon System

### 7.1 Icon Style

Use monochrome line icons:

```txt
single color
outline only
rounded caps
rounded joins
intentional line gaps
no fill
no cartoon face
no excessive detail
readable at 16px
```

SVG baseline:

```svg
stroke="currentColor"
stroke-width="1.8"
stroke-linecap="round"
stroke-linejoin="round"
fill="none"
```

### 7.2 Required Icon Concepts

Core icons:

```txt
desk
raise desk
lower desk
sit
stand
sit/stand cycle
time to move
snooze
pause
settings
connected
disconnected
away
welcome back
goal reached
standing complete
healthy balance
```

Tray icon states:

```txt
neutral
sitting
standing
time-to-move
paused
disconnected
goal-reached
```

### 7.3 macOS Template Icon

macOS menu-bar icon must support template rendering:

- black/transparent source
- system controls color
- no embedded brand color
- no tiny details

---

## 8. Timeline Component

Timeline is the signature component.

### 8.1 Timeline Grammar

```txt
blue    = sitting
green   = standing
gray    = away
amber   = reminder/warning
red     = ignored too long
gap     = tracking paused/unknown
dot     = event
tall bar = active confirmed segment
short bar = passive/inferred segment
```

### 8.2 Visual Rules

- Use many thin vertical bars.
- Bars should feel like rhythm, not chart noise.
- Rounded bar caps.
- Compact version must work at 260–340px width.
- Full version must work at 600–900px width.
- Always include legend on detailed views.

### 8.3 Example Layout

```txt
| | | | | | | | | | | | | | |
B B B B G G G B B A A G G G P
```

---

## 9. Component Style

### 9.1 Current Mode Card

Should include:

- mode label
- session duration
- color-coded status dot
- optional icon
- primary action

### 9.2 Summary Card

Should include:

- compact label
- value
- optional mini icon
- subtle border

### 9.3 Buttons

Primary:

- filled or strong glass
- rounded pill
- high contrast

Secondary:

- transparent
- subtle border
- hover surface

Danger:

- use sparingly
- only for reset/delete actions

---

## 10. Platform Adaptation

Rule:

```txt
Brand = shared
Container = native
Content = adaptive
```

Windows:

- slightly larger cards
- stronger shadow
- Acrylic/Mica-like surfaces
- anchored above taskbar tray

macOS:

- more compact popover
- more translucency
- lighter shadow
- anchored below menu bar
- template menu-bar icon

---

## 11. Website Direction

Landing page should not look like generic SaaS.

It should show:

- floating glass tray app
- Windows and macOS platform preview
- daily timeline as hero asset
- three feature cards:
  - Gentle nudges
  - Sit/stand rhythm
  - Healthy balance
- minimal copy
- calm background glow

Hero copy:

```txt
Sit. Stand. Flow.
A quiet tray app for healthier desk rhythm.
```

Alternative:

```txt
A better rhythm for your desk.
Gentle reminders to sit, stand and move in balance.
```

---

## 12. Design QA Checklist

Before accepting UI:

- Does it work in both dark and light mode?
- Does it feel like a desktop utility, not a website?
- Is the current mode visible in 2 seconds?
- Is the timeline understandable?
- Are icons readable at 16px?
- Is copy calm and short?
- Is there enough empty space?
- Are colors semantic and consistent?
- Does the design avoid medical claims?
