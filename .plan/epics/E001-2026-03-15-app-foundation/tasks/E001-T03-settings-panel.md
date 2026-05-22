---
id: E001-T03
epic: E001
status: completed
original_id: 0001-settings-panel
title: T001 — Settings Panel UI
---
# T001 — Settings Panel UI

**Priority**: P1
**Status**: open
**Depends on**: T002

## Goal
Single unified config screen that replaces `CalibrationWizard` as well.
No more two-screen config split — one panel covers limits, calibration, and notifications.

## CalibrationWizard → absorbed
`CalibrationWizard.tsx` is **deleted**. Its first-run flow becomes:
- On startup, if no config stored → Settings opens automatically
- User sets calibration heights via sliders (not a wizard, just numeric inputs or sliders)

## Scope

### New component: `src/components/SettingsPanel.tsx`

**Section 1 — Time Limits**
- Slider: "Remind me to stand after" — range 10–90, step 5, default 45
- Slider: "Remind me to sit after" — range 5–60, step 5, default 15

**Section 2 — Calibration**
- Number input: "Sitting height (mm)" — default 720, range 400–900
- Number input: "Standing height (mm)" — default 1050, range 900–1400
- Helper: "Set desk to sitting position and enter sensor reading"
- Validation: sitting_mm < standing_mm (inline error if inverted)

**Section 3 — Notifications** (3 toggles)
- `notify_inactivity` — "Alert if no position change for 90 min"
- `notify_daily_posture_balance` — "Alert if sitting dominates today"
- `notify_praise_halfway` — "Praise when halfway through standing goal"

**Footer**
- "Save" button → `invoke("save_settings", config)` + close panel
- "Back" / "Cancel" → discard local state + close

### Integration in `App.tsx`
- Settings button (gear icon or "settings" text) in header row
- `showSettings` boolean state → mounts `SettingsPanel` (replaces main content, not modal)
- On first run (no stored config): `showSettings = true` by default

### Design system tokens used
- Panel uses `.panel-row` surface, `.btn` / `.btn--danger` buttons
- Sliders: custom styled `<input type="range">` with `--beam` accent color
- Number inputs: `--panel-elevated` bg, `--rail-default` border
- Inline validation error: `--signal-alert` color

## Tests
- Vitest: Save invokes `save_settings` with correct payload
- Vitest: Back does not invoke `save_settings`
- Vitest: Inverted calibration (sitting_mm > standing_mm) shows inline error, disables Save
- Vitest: Slider clamps to min/max
- Snapshot: default state renders correctly with step=5, default=45

## Acceptance criteria
- [ ] CalibrationWizard deleted
- [ ] First run: Settings opens automatically
- [ ] Save persists via T002 backend
- [ ] Validation blocks save on inverted calibration
- [ ] All three notification toggles save/load correctly
