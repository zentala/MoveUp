---
name: desk-debug
description: Analyze desk app snapshots and event logs to verify KPI metrics, detect data inconsistencies, and validate what the user sees in the popup. Use when debugging standing%, position changes, break tracking, or any KPI that looks wrong.
allowed-tools: Read, Grep, Glob, Bash
---

# Desk App Debug Analyzer

Analyze the running desk app's snapshot files and event logs to verify that computed metrics match reality.

## Data Sources

All logs are at: `C:/Users/zentala/AppData/Roaming/io.zntl.desk/logs/`

- **Minute snapshots**: `YYYY-MM-DD/HH-MM.json` — full session state every minute
- **Event log**: `YYYY-MM-DD/events.log` — all state transitions, credits, alerts, notifications

## Step 1: Load Today's Data

```bash
# List today's snapshot files
TODAY=$(date +%Y-%m-%d)
LOGS="C:/Users/zentala/AppData/Roaming/io.zntl.desk/logs/$TODAY"
ls "$LOGS/" | tail -5
```

Read the **latest snapshot** and the **event log** for today.

## Step 2: Reconstruct Timeline from Event Log

Parse events.log and build a timeline of state transitions:

| Time | From | To | Duration in state | Key fields |
|------|------|----|-------------------|-----------|

Track:
- Total ACTUAL Sitting time (sum of all Sitting bouts)
- Total ACTUAL Standing time (sum of all Standing bouts, NOT Walking/Away)
- Number of Sitting↔Standing transitions (= position_changes)
- Break credits applied (CREDIT events)
- Number of app restarts (START events)

## Step 3: Verify Snapshot Values

Compare the latest snapshot against your reconstructed timeline:

### sitting_seconds
- This is **credit-adjusted** — break credits reduce it
- Should equal: total sitting time MINUS credits applied

### sitting_seconds_total
- This is **raw** — never reduced by break credit
- Should equal: total actual Sitting time from timeline
- If missing from snapshot: the app version doesn't include this field yet

### standing_seconds
- Should equal: total Standing-only time (NOT Walking, NOT Away)
- Common bug: Away time counted as standing

### position_changes
- Should equal: number of Sitting↔Standing transitions
- After restart: should be seeded from DB (count of today's session rows - 1)
- Common bug: resets to 0 on restart

### Metrics (if present)
Check the `metrics` array in the snapshot:
- **standing_pct**: `standing_seconds / (sitting_seconds_total + standing_seconds) * 100`
  - If using `sitting_seconds` instead of `sitting_seconds_total`, it will be inflated after break credits
- **position_rate**: `position_changes / hours_worked`
- **hourly_breaks**: hours with ≥5min away break / total active hours
- **longest_session**: `longest_computer_session_secs` in minutes

## Step 4: Report

Format findings as:

### Data Summary
- App uptime: first START to last snapshot
- Restarts today: N
- States: Sitting Xh Ym, Standing Xh Ym, Away Xh Ym

### Metric Verification
| Metric | Snapshot Value | Calculated Value | Match? |
|--------|---------------|-----------------|--------|
| standing_pct | X% | Y% | YES/NO |
| position_changes | X | Y | YES/NO |
| ... | ... | ... | ... |

### Issues Found
List any discrepancies with root cause analysis.

### Recommendations
Specific fixes if issues found.

## Common Issues Checklist

- [ ] standing_seconds includes Away time (bug: `state != Sitting` instead of `state == Standing`)
- [ ] sitting_seconds reset by break credit inflates standing_pct
- [ ] position_changes = 0 after restart (not seeded from DB)
- [ ] metrics array empty in snapshots (Vec::new() passed instead of computed)
- [ ] daily_score negative (points calculation bug)
- [ ] Multiple restarts cause data loss (counters not restored)
- [ ] standing_target_reached fires on every restart (flag not persisted)
