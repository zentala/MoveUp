# Premium Tier — Feature Definition & Implementation Plan

**Date**: 2026-03-25
**Status**: Defined (not yet scheduled)
**Depends on**: Validation (pre-orders prove demand)

---

## 1. Tier Comparison

| Feature | Free (local) | Pro (cloud) |
|---------|:---:|:---:|
| Sitting/standing auto-detection | ✅ | ✅ |
| Overlay progress bar | ✅ | ✅ |
| Tray icon with status | ✅ | ✅ |
| Daily KPIs (standing%, changes, session) | ✅ | ✅ |
| Gamification (daily score, streaks) | ✅ | ✅ |
| Smart notifications (escalation, coaching) | ✅ | ✅ |
| Plugin/skin system | ✅ | ✅ |
| Settings & configuration | ✅ | ✅ |
| Session timeline (today) | ✅ | ✅ |
| **History (7d / 30d / all time)** | ❌ | ✅ |
| **Cloud sync** | ❌ | ✅ |
| **Cross-device dashboard** | ❌ | ✅ |
| **Weekly/monthly reports (email)** | ❌ | ✅ |
| **AI coaching (personalized tips)** | ❌ | ✅ |
| **Smartwatch integration** | ❌ | ✅ |
| **Team dashboard** | ❌ | ✅ (Enterprise) |
| **Export (CSV/JSON)** | ❌ | ✅ |
| **Phone display (remote)** | ✅ (LAN) | ✅ (cloud relay) |

### Pricing

- **Pro:** 4.99 EUR/month or 39.99 EUR/year (~20% discount)
- **Enterprise:** TBD (per-seat, when demand exists)
- **Dev Kit buyers:** 6 months Pro free (included with hardware purchase)

### Key Principle

Free tier is FULLY FUNCTIONAL for daily use. Pro adds:
- **Time dimension** (history, trends, reports)
- **Convenience** (cloud sync, cross-device, export)
- **Intelligence** (AI coaching, personalized recommendations)
- **Integration** (smartwatch, calendar)

User should never feel "crippled" on free tier. Pro is "nice to have", not "need to have."

---

## 2. Implementation Breakdown

### Epic E010: Cloud Backend (Foundation)

Everything Pro depends on a cloud backend. This is the biggest investment.

**Stack decision (recommended):**
- Cloudflare Workers (compute) — matches zentala's preferred stack
- Cloudflare D1 (SQLite database)
- Cloudflare R2 (blob storage, if needed)
- Auth: Cloudflare Access or simple magic link email

**Tasks:**

| ID | Task | Description | Effort |
|----|------|-------------|--------|
| T01 | Auth system | Magic link email login (no password). JWT tokens. | M |
| T02 | User account model | D1 schema: users, subscriptions, devices | S |
| T03 | Data sync API | REST/tRPC endpoints: upload daily summaries, download history | L |
| T04 | Stripe integration | Subscription management, webhook handlers, billing portal | L |
| T05 | Desktop app: auth flow | Login UI in settings, token storage, sync toggle | M |
| T06 | Desktop app: data upload | Background sync of daily summaries to cloud (when Pro) | M |
| T07 | Desktop app: history download | Fetch historical data from cloud, display in UI | M |
| T08 | Rate limiting + security | API rate limits, input validation, abuse prevention | S |

**Wave structure:**
- Wave 1: T01, T02 (auth + schema) — foundation
- Wave 2: T03, T04 (API + billing) — can be parallel
- Wave 3: T05, T06, T07 (desktop integration) — depends on Wave 2
- Wave 4: T08 (hardening) — after everything works

### Epic E011: History & Trends (Pro Feature #1)

First Pro feature users will see. Requires E010 (cloud backend).

| ID | Task | Description | Effort |
|----|------|-------------|--------|
| T01 | Daily summary persistence | Save daily stats to SQLite on app close (local) | S |
| T02 | History view UI | 7d / 30d / all-time toggle above KPI strip | M |
| T03 | Trend charts | Line charts: standing%, changes/h, score over time | M |
| T04 | Weekly report email | Cloudflare Worker cron: compile weekly stats, send email | M |
| T05 | Export to CSV/JSON | Download button in history view | S |

**Wave structure:**
- Wave 1: T01 (local persistence — useful even without cloud)
- Wave 2: T02, T03 (UI — can show local history immediately)
- Wave 3: T04, T05 (cloud features — need E010)

### Epic E012: AI Coaching (Pro Feature #2)

Personalized tips based on user's patterns. Requires E010 + E011.

| ID | Task | Description | Effort |
|----|------|-------------|--------|
| T01 | Pattern detection | Analyze 7d data: worst hours, best days, streaks | M |
| T02 | Coaching engine | Rule-based tips (not ML): "You sit longest 14-16h, try standing then" | M |
| T03 | Coaching UI | In-app tips panel + optional push notification | S |
| T04 | LLM-generated insights | Optional: Claude API for natural language weekly summary | M |

**Wave structure:**
- Wave 1: T01, T02 (backend logic)
- Wave 2: T03, T04 (UI + optional LLM)

### Epic E013: Smartwatch Integration (Pro Feature #3)

Detect walking, provide wrist notifications. Highest complexity.

| ID | Task | Description | Effort |
|----|------|-------------|--------|
| T01 | Research: watch APIs | Apple HealthKit, Garmin Connect IQ, Fitbit Web API | M |
| T02 | Activity data ingestion | Cloud endpoint to receive step/HR data from watch | L |
| T03 | Walking state detection | Correlate watch movement with desk height → true Walking state | L |
| T04 | Wrist notifications | Push "time to stand" to watch (platform-specific) | L |
| T05 | Desktop integration | Show watch data in app (steps, HR during standing) | M |

**Wave structure:**
- Wave 1: T01 (research — may change approach)
- Wave 2: T02, T03 (backend)
- Wave 3: T04, T05 (integration)

### Epic E014: Cross-Device & Phone Relay (Pro Feature #4)

Cloud relay for phone display (works outside LAN). Builds on E009 (web kiosk).

| ID | Task | Description | Effort |
|----|------|-------------|--------|
| T01 | WebSocket relay server | Cloudflare Durable Objects: relay desk events to phone | M |
| T02 | Desktop: cloud relay mode | Send events to cloud relay instead of (or in addition to) LAN | S |
| T03 | Phone: cloud connection | Connect to cloud relay when not on same LAN | S |
| T04 | Latency optimization | Minimize delay between desk event and phone display | S |

---

## 3. Prioritized Roadmap

```
E010 Cloud Backend ──────────────────────────────────────────────
     │
     ├── E011 History & Trends (first Pro feature visible)
     │        │
     │        ├── E012 AI Coaching (builds on history data)
     │        │
     │        └── E014 Cross-Device Relay (cloud infrastructure)
     │
     └── E013 Smartwatch (independent research, cloud needed)
```

### Phase order:
1. **E010** — Cloud backend (must be first, everything depends on it)
2. **E011** — History & trends (easiest to build, highest perceived value)
3. **E012** — AI coaching (differentiator, uses Claude API)
4. **E014** — Cross-device relay (incremental on E009)
5. **E013** — Smartwatch (hardest, most research, do last)

### Estimated total effort:
- E010: ~6-8 tasks, 2-3 weeks focused work
- E011: ~5 tasks, 1-2 weeks
- E012: ~4 tasks, 1-2 weeks
- E013: ~5 tasks, 3-4 weeks (most unknowns)
- E014: ~4 tasks, 1 week

**Total: ~8-12 weeks** to have a shippable Pro tier with history + AI + cross-device.
Smartwatch adds 3-4 weeks on top.

---

## 4. Build vs. Skip Analysis

### Worth building (high value / reasonable effort):
- ✅ Cloud sync + history — table stakes for any "tracking" app
- ✅ Weekly email reports — very low effort, high perceived value
- ✅ Export (CSV/JSON) — trivial, users expect it
- ✅ AI coaching — differentiator, Claude API makes this easy

### Skip for now (high effort / uncertain value):
- ⏸️ Smartwatch — complex, platform-specific, unclear if users want it
- ⏸️ Team/enterprise — no enterprise customers yet, premature
- ⏸️ Calendar integration — nice idea, unclear demand

### zentala's concern: "SaaS is a lot of work"

Honest assessment: YES, it is. E010 (cloud backend) alone is 2-3 weeks.
But consider:
- Without Pro, the only revenue is hardware (thin margin, logistics pain)
- 100 Pro subscribers × 5 EUR/month = 500 EUR/month RECURRING
- 100 dev kits × 49 EUR = 4,900 EUR ONE-TIME
- After month 10, SaaS revenue exceeds hardware revenue forever

**Recommendation:** Build E010 + E011 (cloud + history) as MINIMUM viable Pro.
Skip E012-E014 until Pro has paying subscribers proving demand for premium.
