# Story-Driven Launch & Product Strategy Update

**Date**: 2026-03-26
**Status**: Defined
**Supersedes parts of**: validation-and-gtm.md, distribution-and-tiers.md

---

## 1. Mission & Vision Statement

**Mission:** Change the sedentary habits of millions of computer workers by making
position awareness automatic and behavior change effortless.

**Vision:** A world where your workspace actively helps you stay healthy — starting
with knowing when to change position, expanding to full ergonomic intelligence
(eyes, movement, posture, breaks).

**Why this matters:** Sitting is the new smoking. Standing desks exist but don't
change behavior. We solve the gap between owning a standing desk and actually using it.

---

## 2. Landing Page: Founder's Story Format

The landing page is NOT a typical product page. It's a **personal story** that
naturally leads to a product. This is how indie makers sell.

### Story arc:

```
1. THE PROBLEM (personal)
   "I'm a programmer. I sit 10+ hours a day. I bought a standing desk
    two years ago thinking it would fix everything. It didn't."

2. THE REALIZATION
   "I realized I had no idea how much I actually stand. So I built
    a sensor to find out. The results were embarrassing: 8% standing time."
    [real data screenshot]

3. THE FIRST VERSION
   "I built a simple app to track it. It helped — but not enough.
    I needed something that would actually CHANGE my behavior, not just
    show me numbers."

4. THE BREAKTHROUGH
   "Then a kid sent me his simplified version. I thought: why am I
    overcomplicating this? I stripped it down, started experimenting
    with different UX approaches, and something clicked."

5. THE EXPERIMENTS
   "I tested: overlay bars, notifications, gamification, scoring with
    penalties, coaching messages. Some worked. Some were annoying.
    After 300+ hours of development and 500+ tests, I found what works."

6. THE RESULTS
   "My standing time went from 8% to 22%. Position changes from 1.2/day
    to 4.8/day. My back pain is gone."
    [before/after data, real screenshots]

7. THE PRODUCT
   "Now I want to share this with you. I've made the sensor simple
    enough to mount in 30 seconds, and the app is open source."

8. THE ASK
   "I'm not a hardware company. I'm a UX researcher who built something
    that works. To produce these at scale, I need your support.
    Pre-order now → when we hit the threshold, I start production."

9. THE BIGGER VISION
   "This is just the beginning. I want to build a system that helps you:
    - Change position more often (desk sensor ← YOU ARE HERE)
    - Rest your eyes (screen time tracking)
    - Move more (smartwatch integration)
    - Build lasting healthy habits (behavioral coaching in the cloud)
    Your purchase funds this research."
```

### How this differs from current plan:

- Current plan: product-first, features grid, comparison table
- New plan: story-first, emotion, personal journey, product emerges naturally
- The product sections (features, pricing, FAQ) still exist but come AFTER the story

---

## 3. Two Product Versions (from day one)

### Version 1: Basic Kit — "Awareness"

| What | Details |
|------|---------|
| Contents | VL53L1X sensor + MCU + carrier PCB + USB cable + tape |
| Price | €49 |
| Does | Measures desk height, tracks sitting/standing, overlay bar, notifications |
| Threshold | 200 pre-orders to start production |

### Version 2: Pro Kit — "Active Coaching"

| What | Details |
|------|---------|
| Contents | Everything in Basic + vibration motor + piezo/ADXL sensor |
| Price | €79 |
| Does | Everything in Basic + haptic alerts (desk vibrates) + presence detection |
| Threshold | 500 pre-orders to start production |
| Note | Higher threshold because more complex assembly + components |

### Version 3: Founder's Edition — "Full Investment"

| What | Details |
|------|---------|
| Contents | Pro Kit + 5 years Pro subscription (cloud + smartwatch) |
| Price | €149 |
| Does | Everything in Pro + cloud history + smartwatch integration when available |
| Threshold | Same as Pro (ships with Pro batch) |
| Note | No limit on quantity. Encourages buying multiple (for family/friends) |

### Landing page pricing section:

```
┌─────────────┬─────────────┬──────────────────────┐
│   Basic Kit │   Pro Kit   │ Founder's Edition    │
│   €49       │   €79       │ €149                 │
├─────────────┼─────────────┼──────────────────────┤
│ ✅ Sensor    │ ✅ Sensor    │ ✅ Sensor             │
│ ✅ App       │ ✅ App       │ ✅ App                │
│ ✅ USB cable │ ✅ USB cable │ ✅ USB cable          │
│              │ ✅ Vibration │ ✅ Vibration          │
│              │   motor     │   motor              │
│              │ ✅ Presence  │ ✅ Presence           │
│              │   sensor    │   sensor             │
│              │              │ ✅ 5yr cloud sub     │
│              │              │ ✅ Smartwatch (when   │
│              │              │   available)         │
│              │              │ ✅ Priority features  │
│              │              │ ✅ Name in credits    │
├─────────────┼─────────────┼──────────────────────┤
│ 200 needed  │ 500 needed  │ Ships with Pro       │
│ [Pre-order] │ [Pre-order] │ [Pre-order]          │
└─────────────┴─────────────┴──────────────────────┘

💡 "Buy extra for family & friends — healthier habits for everyone"
```

---

## 4. "DIY Kickstarter" Model

NOT using Kickstarter platform (they take 5-10% + fees).
Instead: own website with Stripe, same crowdfunding psychology.

**How it works on the landing page:**

```
┌──────────────────────────────────────────────┐
│ BASIC KIT (€49)                              │
│ 🎯 87 / 200 pre-orders                      │
│ ████████████████████░░░░░░░░░░░░  44%       │
│ [Pre-order Basic Kit]                        │
│                                              │
│ PRO KIT (€79)                                │
│ 🎯 34 / 500 pre-orders                      │
│ ██████░░░░░░░░░░░░░░░░░░░░░░░░░░   7%      │
│ [Pre-order Pro Kit]                          │
│                                              │
│ FOUNDER'S EDITION (€149)                     │
│ 🎯 12 backers                               │
│ [Back This Project]                          │
│                                              │
│ Total raised: €X,XXX                         │
└──────────────────────────────────────────────┘
```

---

## 5. Viral Marketing Strategy

### YouTube / Influencer Outreach

Send free prototype units (5-10) to:

**Target profiles:**
- Standing desk reviewers (search: "standing desk review" on YouTube)
- Quantified self / biohacking YouTubers
- Developer setup / productivity YouTubers
- Ergonomics professionals / physiotherapists with channels
- Tech maker / DIY channels

**Specific channels to research:**
- Search YouTube: "standing desk accessories", "desk setup tour", "ergonomic setup"
- Look for channels with 10k-500k subscribers (big enough for reach, small enough to respond)
- Maker channels: ones that review open source hardware, Arduino projects

**What to send:**
- Assembled dev kit (Basic version)
- USB cable
- One-page "quick start" card
- Personal letter: "I built this because..."
- Pre-written key points (not a script — just context they can use)

**What to ask:**
- "Try it for 2 weeks, share your honest opinion"
- NOT "please promote this" — let them decide
- Offer: "If you want, I can share your review on our site"

### Video Content (by zentala)

1. **"How I track my sitting habits automatically" (2-3 min)**
   - Show real desk, mount sensor live, show app working
   - Show real data: standing %, position changes
   - End: "If you want one → link in description"

2. **"I sat for 10 hours and my desk told me off" (1 min, short-form)**
   - TikTok/Reels/Shorts format
   - Show overlay bar going red, notification popping up
   - Humorous tone

3. **"Building a smart desk sensor for €12" (5-10 min, maker content)**
   - Assembly process, soldering, flashing firmware
   - For maker audience
   - Links to GitHub + buy page

---

## 6. Analytics: Privacy-First (NOT Google Analytics)

### Decision: No Google Analytics

Reasons:
- Privacy-first product can't use privacy-invasive analytics
- Users will notice and call it out (especially HN/Reddit audience)
- GDPR cookie consent adds friction to landing page
- Hypocritical: "your data stays local" + Google tracking pixel

### Alternatives (pick one):

| Tool | Hosting | Cost | GDPR-safe | Notes |
|------|---------|------|-----------|-------|
| **Plausible** | Cloud or self-host | €9/mo or free (self) | ✅ No cookies | Recommended |
| **Umami** | Self-host | Free | ✅ No cookies | More DIY |
| **Fathom** | Cloud | $14/mo | ✅ No cookies | Premium option |
| **Cloudflare Analytics** | Cloudflare | Free | ✅ | Basic, already available |

**Recommendation:** Plausible (cloud, €9/mo) — simplest, privacy-aligned, no cookie banner needed.
Or Cloudflare Analytics (free) if budget is tight.

### App Telemetry (separate from website analytics)

**Opt-in only.** User explicitly enables data sharing in Settings.

**What we collect (when opted in):**
- Daily aggregates: standing %, position changes, longest session, score
- App version, OS
- NO: raw sensor data, timestamps, keyboard activity, personally identifiable info

**What we show the user:**
- Clear toggle: "Share anonymized usage data to help improve the app"
- "What data?" link → full list of exactly what's sent
- Default: OFF

**Why collect:**
- "Average standing % for our users: X%" → social proof
- Identify which features actually change behavior
- NOT for monetization — for product improvement

**Implementation:** Cloudflare Worker endpoint, D1 storage. Simple POST of daily summary.
No user accounts needed — random device ID, no email/name.

---

## 7. Sedentary Lifestyle Research Report

### Task: Create research report

**Output:** `.plan/reports/sedentary-lifestyle-research.md`

**Contents:**
1. Health consequences of prolonged sitting (with citations)
   - Cardiovascular disease risk
   - Diabetes risk
   - Back pain / musculoskeletal disorders
   - Mental health impact
   - Mortality statistics

2. Standing desk effectiveness research
   - Do standing desks actually help? (mixed evidence)
   - Why people stop using them (behavior gap)
   - Optimal sit/stand ratios from research

3. Position change recommendations (evidence-based)
   - How often to change position
   - Minimum break duration
   - Standing vs walking vs stretching
   - Eye rest recommendations (20-20-20 rule)

4. Behavioral intervention research
   - What makes people change habits?
   - Nudge theory applied to desk behavior
   - Gamification effectiveness research
   - Automatic vs manual tracking effectiveness

5. Sources (links to actual papers/studies)
   - PubMed, Google Scholar
   - WHO reports
   - Ergonomics journals

**Use for:**
- Landing page: cite real research to support claims
- Blog articles: reference in SEO content
- App: evidence-based default values (40 min session limit, 5 min break minimum)
- zentala.agency case study: research depth shows expertise
- Potential EU grant applications: research foundation

---

## 8. Updated Task List for E010

New/changed tasks:

| Task | Description | Priority |
|------|-------------|----------|
| **Rewrite landing page as founder story** | Story arc from Section 2, not product grid | CRITICAL |
| **Three product tiers with separate thresholds** | Basic €49/200, Pro €79/500, Founder €149 | HIGH |
| **Sedentary lifestyle research report** | Scientific sources, recommendations, citations | HIGH |
| **Switch to Plausible/CF Analytics** | Remove GA references, no cookie banner | HIGH |
| **Design app telemetry opt-in** | Settings toggle, data policy, CF Worker endpoint | MEDIUM |
| **Create 2-3 marketing videos** | Screen recording + sensor mounting + data reveal | HIGH |
| **Influencer outreach list** | Research 10-20 YouTube channels, prepare kit + letter | MEDIUM |
| **"Buy for family" upsell** | Multi-quantity discount or bundle option in Stripe | LOW |
