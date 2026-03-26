---
name: ergo-review
description: Analyze desk app usage logs to evaluate motivation effectiveness, discuss ergonomic habits, and propose parameter tweaks. Use when user wants to review their sitting/standing patterns, discuss break credit logic, or iterate on motivation strategy.
allowed-tools: Read, Grep, Glob, Bash, AskUserQuestion
---

# Ergonomic Review — Usage Analysis & Motivation Discussion

Analyze real usage data, evaluate whether current motivation strategies are working, and propose improvements.

## Data Sources

All logs at: `C:/Users/zentala/AppData/Roaming/io.zntl.desk/logs/`

- **Minute snapshots**: `YYYY-MM-DD/HH-MM.json` — full session state every minute
- **Event log**: `YYYY-MM-DD/events.log` — state transitions, break credits, notifications, alerts

## Step 1: Load Recent Data

```bash
LOGS="C:/Users/zentala/AppData/Roaming/io.zntl.desk/logs"
# List available days
ls "$LOGS/" | sort | tail -7

# Today's data
TODAY=$(date +%Y-%m-%d)
ls "$LOGS/$TODAY/" 2>/dev/null | wc -l
```

Read at minimum:
- All event logs from last 3 days (or whatever is available)
- First, mid-day, and latest snapshot per day (for state comparison)

## Step 2: Extract Patterns

From event logs, build a timeline and compute:

### Sitting patterns
- Average sitting session length
- Longest unbroken sitting session
- How many sessions exceeded the 40-min limit?
- At what time of day does user sit longest? (morning vs afternoon)

### Break patterns
- How many breaks per day?
- Average break duration
- Break distribution: how many were <5min (no credit), 5-9min (partial), ≥10min (full reset)?
- Were there micro-breaks (1-4 min) that got zero credit?

### Notification effectiveness
- How many notifications were fired? (grep for NOTIF in event log)
- After each notification: did user stand/leave within 5 minutes?
- Notification-to-action latency (if trackable from state transitions)

### Position change rate
- Changes per hour worked
- Are changes voluntary (before notification) or prompted (after notification)?

## Step 3: Evaluate Current Parameters

Compare actual usage against current config:

| Parameter | Current Value | Evidence from logs |
|-----------|--------------|-------------------|
| Session limit | 40 min | How often exceeded? |
| Break threshold (short) | 5 min | How many breaks fall just below? |
| Break threshold (full) | 10 min | How many breaks are 5-9 min? |
| Short credit | -20 min | Does this feel right given patterns? |
| Full credit | reset to 0 | Too generous? Not enough? |

## Step 4: Identify Problems

Look for these anti-patterns:

1. **Micro-break punishment** — user takes 2-3 min breaks that get zero credit. These ARE healthy breaks. Current step function penalizes them.

2. **Notification fatigue** — notifications fire but user doesn't act. Either too frequent, wrong timing, or wrong medium (toast vs popup vs bar color).

3. **Gaming the system** — user stands for exactly 5 min then sits back. Technically a "position change" but not real movement.

4. **All-or-nothing sessions** — user either sits for 2h straight OR takes a proper 15min break. No middle ground. Break credit curve should reward partial breaks.

5. **Time-of-day blindspots** — e.g., morning meetings = 3h sitting, no breaks. App should know about these patterns.

## Step 5: Discuss with User

Present findings using AskUserQuestion. For each problem identified:

1. Show the data: "You had 7 breaks under 5 minutes on Tuesday. All got zero credit."
2. Propose a change: "Progressive credit: 2 min break = 4 min credit. Worth trying?"
3. Get user's perspective: "Does this match your experience? What would have helped?"

Key discussion points:
- Is the break credit curve working for you?
- Are notifications motivating or annoying?
- What makes you actually stand up? (notification? bar turning red? feeling stiff?)
- Any patterns you've noticed yourself?

## Step 6: Propose Changes

Based on discussion, propose specific parameter changes:

- **Quick tweaks** (config changes, no code): adjust thresholds, timing
- **Medium changes** (code needed): progressive break credit curve, notification outcome tracking
- **Long-term ideas** (future epic): adaptive coaching, A/B testing framework

For each proposal, note:
- What it changes
- Expected impact on user behavior
- How we'll measure if it worked (what to check in next /ergo-review)

## Context from Memory

Read these memory files for background:
- `project_progressive_break_credit.md` — proposed break credit curve
- `project_motivation_analytics_process.md` — long-term analytics vision
- `project_break_philosophy.md` — standing ≡ away, goal is changes not duration
- `project_product_vision.md` — coach not tracker philosophy
- `project_gamification_philosophy.md` — negative score valid, comeback mechanics

## Output Format

```
## Ergo Review — [date range]

### Usage Summary
- Days analyzed: N
- Total computer time: Xh
- Sitting: X% | Standing: X% | Away: X%
- Position changes: X/day avg (target: ≥6)
- Breaks: X/day avg

### Key Observations
1. [observation + data]
2. [observation + data]

### Problems Identified
1. [problem + evidence + proposed fix]

### Recommended Changes
- [ ] [change] — [expected impact]

### Next Review
- Check for: [what to measure next time]
```
