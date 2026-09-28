# TEST_CASES.md — Healthy Balance

## 1. Timeline Calculation Tests

### 1.1 One Sitting Segment

Input:

```txt
08:00–09:00 sitting
```

Expected:

```txt
sittingMinutes = 60
standingMinutes = 0
awayMinutes = 0
pausedMinutes = 0
standingPercent = 0
switches = 0
```

---

### 1.2 Sitting Then Standing

Input:

```txt
08:00–09:00 sitting
09:00–09:30 standing
```

Expected:

```txt
sittingMinutes = 60
standingMinutes = 30
standingPercent = 33.33
switches = 1
```

---

### 1.3 Standing Then Sitting Then Standing

Input:

```txt
08:00–08:30 standing
08:30–09:30 sitting
09:30–10:00 standing
```

Expected:

```txt
sittingMinutes = 60
standingMinutes = 60
standingPercent = 50
switches = 2
```

---

### 1.4 Away Excluded From Balance

Input:

```txt
08:00–09:00 sitting
09:00–10:00 away
10:00–11:00 standing
```

Expected:

```txt
sittingMinutes = 60
standingMinutes = 60
awayMinutes = 60
standingPercent = 50
switches = 1
```

Away time does not affect standing percentage.

---

### 1.5 Paused Excluded From Balance

Input:

```txt
08:00–09:00 sitting
09:00–10:00 paused
10:00–11:00 standing
```

Expected:

```txt
sittingMinutes = 60
standingMinutes = 60
pausedMinutes = 60
standingPercent = 50
switches = 1
```

Paused time does not affect standing percentage.

---

### 1.6 Open Current Segment

Input:

```txt
08:00–open sitting
now = 08:45
```

Expected:

```txt
sittingMinutes = 45
```

---

### 1.7 Clamp Segment Crossing Midnight

Selected date:

```txt
2026-06-08
```

Input:

```txt
2026-06-07 23:30 – 2026-06-08 00:30 sitting
```

Expected:

```txt
sittingMinutes = 30
```

---

### 1.8 Ignore Invalid Negative Segment

Input:

```txt
09:00–08:00 sitting
```

Expected:

```txt
duration ignored
no crash
```

---

## 2. Mode Switching Tests

### 2.1 Switch Sitting to Standing

Initial:

```txt
current segment: sitting 08:00–open
now = 09:00
```

Action:

```txt
setMode('standing')
```

Expected:

```txt
sitting segment endedAt = 09:00
new standing segment startedAt = 09:00
currentMode = standing
```

---

### 2.2 Switch to Same Mode

Initial:

```txt
currentMode = sitting
```

Action:

```txt
setMode('sitting')
```

Expected:

```txt
no duplicate segment
no data corruption
```

---

### 2.3 Pause Tracking

Initial:

```txt
currentMode = sitting
```

Action:

```txt
pauseTracking()
```

Expected:

```txt
sitting segment closed
paused segment started
reminders suppressed
```

---

## 3. Reminder Tests

### 3.1 Time To Move

Settings:

```txt
reminderIntervalMinutes = 30
```

Input:

```txt
currentMode = sitting
current segment started 31 minutes ago
notifications enabled
quiet hours disabled
```

Expected:

```txt
shouldSendReminder = true
type = time-to-move
```

---

### 3.2 Do Not Remind Too Early

Input:

```txt
currentMode = sitting
current segment started 20 minutes ago
interval = 30
```

Expected:

```txt
shouldSendReminder = false
```

---

### 3.3 Do Not Remind While Paused

Input:

```txt
currentMode = paused
```

Expected:

```txt
shouldSendReminder = false
```

---

### 3.4 Do Not Remind While Away

Input:

```txt
currentMode = away
```

Expected:

```txt
shouldSendReminder = false
```

---

### 3.5 Quiet Hours Suppress Reminder

Settings:

```txt
quietHoursEnabled = true
quietHoursStart = 22:00
quietHoursEnd = 07:00
now = 23:00
```

Expected:

```txt
shouldSendReminder = false
```

---

### 3.6 Snooze Suppresses Reminder

Input:

```txt
snoozedUntil = 10:30
now = 10:15
```

Expected:

```txt
shouldSendReminder = false
```

---

### 3.7 Snooze Expired

Input:

```txt
snoozedUntil = 10:30
now = 10:31
```

Expected:

```txt
shouldSendReminder = true if other conditions match
```

---

## 4. Standing Completion Tests

### 4.1 Standing Session Goal Reached

Settings:

```txt
standingSessionGoalMinutes = 15
```

Input:

```txt
currentMode = standing
standing session duration = 15 minutes
```

Expected:

```txt
show stand-complete notification once
```

---

### 4.2 Do Not Duplicate Standing Completion

Input:

```txt
standing session duration = 20 minutes
notification already fired for this segment
```

Expected:

```txt
no duplicate notification
```

---

## 5. Persistence Tests

### 5.1 Missing Settings

Input:

```txt
settings.json missing
```

Expected:

```txt
return default settings
create settings on save
```

---

### 5.2 Corrupt Settings

Input:

```txt
settings.json contains invalid JSON
```

Expected:

```txt
rename to .corrupt
return defaults
no crash
```

---

### 5.3 Timeline Persistence

Action:

```txt
switch mode twice
restart app
```

Expected:

```txt
timeline restored
current day summary restored
```

---

## 6. UI Acceptance Tests

### 6.1 Status Screen

Expected visible:

```txt
current mode
session duration
desk height
mini timeline
sitting summary
standing summary
switch count
next reminder
quick actions
```

---

### 6.2 Timeline Screen

Expected visible:

```txt
full timeline
legend
current time marker
time labels
```

---

### 6.3 Settings Screen

Expected editable:

```txt
theme
reminder interval
standing session goal
daily standing target
notifications
quiet hours
demo mode
```

---

## 7. Accessibility Tests

- User can tab through controls.
- Buttons have accessible labels.
- Timeline has legend.
- Color is not the only mode indicator.
- Reduced motion setting disables unnecessary animation.
- Text contrast is readable in light and dark modes.
