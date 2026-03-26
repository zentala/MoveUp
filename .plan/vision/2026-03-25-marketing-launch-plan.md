# Marketing Launch Plan — zntlDesk

**Date**: 2026-03-25
**Status**: Active — launch plan + SEO + conversion + growth tactics
**Goal**: Launch desk.zentala.io (sales) + zentala.agency case study (brand positioning)
**Pricing**: Read from `config/pricing.json` (canonical source of truth)
**Supersedes**: Parts of `validation-and-gtm.md` (old 2-tier pricing, 100 threshold)

---

## 0. Strategic Context

**Primary goal:** Position zentala as UX/behavioral design innovator who created
a product that measurably changes people's health habits.

**Secondary goal:** Sell dev kits to cover 300+ hours of development work.
Target: 1000 units × 100 PLN margin = 100k PLN (4 months salary equivalent).

**Two websites, two audiences:**

| | desk.zentala.io | zentala.agency |
|---|---|---|
| **Purpose** | Sell dev kits | Personal brand / portfolio |
| **Audience** | Standing desk owners, devs, biohackers | Employers, grant committees, press |
| **Tone** | Product marketing, clear CTA | Innovation showcase, research, insights |
| **Traffic source** | Google/FB ads, Reddit product posts | Organic, LinkedIn, conference talks |
| **CTA** | "Pre-order Dev Kit" | "See my work" → links to desk.zentala.io |

---

## 1. Site B: desk.zentala.io (Sales Landing Page)

### Rebuild on existing Astro framework. English only.

### Page structure (top to bottom):

**Section 1: Hero**
```
Headline: "Your standing desk is useless if you never stand."
Subhead: "Automatic sit/stand tracking. Real-time coaching.
          Open source. No cloud. No account."
CTA: [Pre-order Dev Kit — €49] [See How It Works ↓]
Visual: Photo of sensor mounted under real desk + app screenshot overlay
```

**Section 2: Problem (emotional hook)**
```
"You spent €800 on a standing desk.
 You've used it standing... twice this month.

 Manual tracking apps? Abandoned in 3 days.
 Fitness watches? Don't know your desk height.

 You need something that works WITHOUT thinking about it."
```

**Section 3: How It Works (3 steps, with photos)**
```
1. Mount the sensor under your desk [photo: sensor on carrier PCB, taped under desk]
2. Plug USB into your computer [photo: USB-C cable to laptop]
3. The app does everything else [screenshot: overlay bar + popup with live data]

"No cloud. No account. No subscription required.
 Everything runs on YOUR computer."
```

**Section 4: Features (icon grid, 6 items)**
```
📊 Real-time overlay bar — green→red as you sit too long
🎯 Daily KPIs — standing %, position changes, longest session
🏆 Gamification — score, streaks, comeback mechanics
🔔 Smart alerts — escalating nudges, not annoying popups
🔌 Open source — customize everything, community skins
📱 Phone display — use old phone as desk dashboard (coming soon)
```

**Section 5: App Screenshots**
```
4-6 real screenshots from the app:
- Main popup (KPIs visible)
- Overlay bar on desktop (green state)
- Overlay bar (red state — sitting too long)
- Settings panel
- Timeline view (if exists)
```

**Section 6: What's In The Box (Dev Kit contents)**
```
✅ VL53L1X ToF sensor (pre-assembled on carrier PCB)
✅ Microcontroller (pre-flashed, ready to go)
✅ USB-C cable
✅ Mounting tape (stick under desk in 30 seconds)
✅ Desktop app (Windows, open source)
✅ Discord community access

Ships from Poland. Free EU shipping.
```

**Section 7: Pricing (two tiers)**
```
┌─────────────────┬──────────────────────────┐
│   Dev Kit        │   Founder's Edition      │
│   €49            │   €99                    │
├─────────────────┼──────────────────────────┤
│ ✅ Full kit      │ ✅ Full kit               │
│ ✅ App           │ ✅ App                    │
│ ✅ Community     │ ✅ Community              │
│                  │ ✅ Priority features      │
│                  │ ✅ Direct support         │
│                  │ ✅ 5 years Pro (free)     │
│                  │ ✅ Name in credits        │
│                  │ ✅ Beta access            │
├─────────────────┼──────────────────────────┤
│ [Pre-order]      │ [Pre-order]              │
└─────────────────┴──────────────────────────┘

"When 100 pre-orders are reached, production starts.
 Delivery: ~3 months. Full refund anytime before shipping."

Progress bar: "34 / 100 pre-orders"
```

**Section 8: DIY Option**
```
"Prefer to build it yourself?"
Full BOM list, firmware source, assembly guide — free on GitHub.
[GitHub Repository →]
```

**Section 9: Behavioral Research (light version — links to zentala.agency)**
```
"Why does this work?"
Brief: sitting is the new smoking, but standing desks alone
don't change behavior. Automatic tracking + coaching does.
[Read the full behavioral research →] (links to zentala.agency)
```

**Section 10: FAQ**
```
Q: Does it work with any desk?
A: Yes — any sit/stand desk. Sensor mounts underneath with tape.

Q: Do I need WiFi?
A: No. USB only. No cloud, no account needed.

Q: Is my data private?
A: 100%. Everything stays on your computer. Open source — verify yourself.

Q: When does it ship?
A: ~3 months after we reach 100 pre-orders.

Q: What if I want a refund?
A: Full refund anytime before shipping. No questions asked.

Q: Windows only?
A: Currently yes. Linux/macOS support is on the roadmap.
```

**Section 11: Footer**
```
[Pre-order Dev Kit — €49]  or  [Join Waitlist — free]
"No spam. Updates only when something ships."

GitHub | Discord | zentala.agency
```

### Tech stack:
- Existing Astro + Tailwind from desk.zentala.io
- Stripe Checkout for payments (one-time, handles EU VAT via Stripe Tax)
- Mailchimp free tier or simple Cloudflare Worker + D1 for waitlist emails
- Google Analytics (already configured)
- Pre-order counter: static JSON updated manually, or Cloudflare KV

---

## 2. Site A: zentala.agency (Brand / Case Study)

### Add a case study page/section for zntlDesk:

**Page: zentala.agency/projects/desk** (or similar)

**Content structure:**

```
Title: "Changing Sedentary Habits Through Behavioral UX Design"

The Problem:
- Standing desks don't change behavior
- People buy them, then sit all day
- Data: average standing desk user stands <15% of the day

My Approach:
- Automatic detection (no manual tracking)
- Coach, not tracker (tells you WHEN to change, not just shows data)
- Gamification with real consequences (negative scores = real motivation)
- Progressive escalation (gentle → firm nudges)
- Position CHANGES over standing duration

The Solution:
- Open source desktop app (Tauri + React + Rust)
- ToF sensor for automatic height detection
- Real-time overlay bar, KPI dashboard, smart alerts
- 500+ tests, modular architecture

Results:
- [zentala's own data: standing % before vs after]
- [position changes per day before vs after]
- [X users in beta, Y% report behavior change]

Key UX Innovations:
- Overlay progress bar (top of screen, non-intrusive)
- Coach messaging ("You're behind, but 10 min standing brings you to +5")
- Comeback mechanics in gamification (negative score is gameplay, not bug)
- Widget temperature system (visual urgency without text)

Technical Depth:
- 327 Rust tests, 169 TypeScript tests
- WinAPI overlay renderer (native, not Electron/web)
- Modular alert escalation (pure state machine)
- Open core model (app open, cloud closed)

Links:
- [Try it: desk.zentala.io] → product page
- [GitHub: source code]
- [Behavioral research writeup]
```

**Tone:** Professional, research-oriented, shows depth of thinking.
NOT a product page — a case study that happens to link to a product.

---

## 3. Reddit Posts (Full Copy)

**⚠️ WARNING: All numbers in these posts (8%, 3.5h, 1.2/day, etc.) are PLACEHOLDER
examples. They MUST be replaced with real data from zentala's actual usage before
publishing. Posting fabricated data destroys credibility permanently. See Section 9
for how to collect real data. Task #6 must be completed first.**

### Post 1: r/standingdesks (product angle)

**Title:** "I built a €12 sensor that tracks how much I actually use my standing desk — and the results were embarrassing"

**Body:**
```
I've had a standing desk for 2 years. I thought I was standing "a lot."

Then I built a sensor (VL53L1X laser distance sensor, ~€12) that mounts
under my desk and automatically measures height every second. Combined it
with a desktop app that tracks sitting/standing time, shows a progress bar,
and nudges me to change position.

After 30 days of data:
- I was standing only 8% of my work day
- My longest sitting session was 3.5 hours (I had no idea)
- I averaged 1.2 position changes per day (should be ~4-6)

The app changed my behavior:
- Standing % went from 8% to 22% in 2 weeks
- Position changes went from 1.2/day to 4.8/day
- Longest session dropped from 3.5h to 52 minutes

The whole thing is open source: [GitHub link]
The sensor + app as a ready-to-use kit: [desk.zentala.io]

Happy to answer questions about the tech, the data, or the UX design.
```

**Photo/video:** Before/after KPI screenshot from the app. Photo of sensor under desk.

---

### Post 2: r/quantifiedself (data/insights angle)

**Title:** "30 days of automatic sit/stand tracking — what I learned about my sedentary habits"

**Body:**
```
I built an automatic tracking system for my standing desk — a small sensor
under the desk measures height every second, and a desktop app logs
everything.

Here's what 30 days of data taught me:

1. I sit WAY more than I think. Self-reported: "I stand a lot."
   Actual data: 8% standing time.

2. Mornings are my worst time. 9-12 AM: zero position changes.
   I get "locked in" to work and forget to move.

3. Short breaks don't help. Standing for 2-3 minutes doesn't reset
   anything — physiologically or psychologically. 5+ minutes is the
   minimum for a real break.

4. Gamification works on me. When I added a scoring system
   (negative points for long sitting, positive for changes),
   my behavior changed within 3 days.

5. The biggest insight: it's not about standing duration.
   It's about CHANGE FREQUENCY. Sitting 4 hours is bad.
   Standing 4 hours is also bad. Changing every 45 minutes = healthy.

Tech: VL53L1X ToF sensor + ESP32 + Tauri desktop app (Rust + React).
Open source: [GitHub]
Ready-made kit: [desk.zentala.io]
```

---

### Post 3: Hacker News (Show HN — technical + philosophical)

**Title:** "Show HN: Open-source sit/stand tracker — automatic detection via ToF sensor"

**Body:**
```
I built an open-source desktop app that automatically tracks sitting and
standing using a VL53L1X Time-of-Flight sensor mounted under the desk.

Why: I had a standing desk for 2 years and discovered (via data) I was
standing only 8% of the time. Manual tracking apps get abandoned in days.
I wanted something fully automatic.

How it works:
- Sensor measures desk-to-floor distance every second via USB
- App detects sitting/standing based on calibrated height thresholds
- Real-time overlay bar (WinAPI, not Electron) shows session progress
- Gamification: scoring with penalties for long sitting + recovery mechanics
- Smart alerts: progressive escalation, not just "stand up!" popups

Stack: Tauri 2 (Rust backend + React frontend), SQLite, 500+ tests.
The overlay bar is a native WinAPI window (4px high, full screen width)
rendered via GDI — I went native because web-based overlays had latency.

Philosophy: "Coach, not tracker." The app tells you WHEN to change
position, not just shows you data. Negative scores are part of the game
(comeback mechanics), not a bug.

Source: [GitHub]
Product page (hardware kit): [desk.zentala.io]
Behavioral research writeup: [zentala.agency/projects/desk]
```

---

### Post 4: r/selfhosted (privacy angle)

**Title:** "Self-hosted sit/stand tracker — zero cloud, zero accounts, all data on your machine"

**Body:**
```
Built an automatic sitting/standing tracker for my standing desk.
Everything runs locally:

- Sensor (VL53L1X) under desk → USB → PC
- Desktop app (Tauri, ~60MB) tracks everything
- SQLite database on your machine
- No cloud, no account, no telemetry
- Open source (MIT)

It shows a thin overlay bar at the top of your screen (green→red as you
sit longer), daily KPIs, and smart notifications.

You can also use an old phone as a desk dashboard — the app runs an
embedded HTTP+WebSocket server on your LAN.

GitHub: [link]
Hardware kit (€49, pre-order): [desk.zentala.io]
```

---

### Post 5: r/homeautomation (smart desk angle)

**Title:** "Added automatic height tracking to my standing desk for €12"

**Body:**
```
Mounted a VL53L1X ToF sensor under my desk. It measures distance to floor
every second via USB. A desktop app tracks sitting/standing time,
shows stats, and reminds me to change position.

Total hardware cost: ~€12 (sensor + microcontroller from AliExpress).
App is open source and runs locally (no cloud).

The interesting part: it also runs a WebSocket server, so I have an old
phone mounted next to my desk showing real-time stats as a dashboard.

Next step: Home Assistant integration (the WebSocket protocol is simple
enough to bridge).

Photos: [sensor under desk] [phone dashboard] [app screenshot]
GitHub: [link]
```

---

## 4. Paid Ads Strategy (desk.zentala.io)

### Budget: 500 PLN/month (~115 EUR/month)

Split: 60% Google, 40% Facebook/Instagram

### Google Ads

**Campaign 1: Search (high intent)**
Budget: 200 PLN/month

Keywords:
- "standing desk tracker"
- "sit stand desk sensor"
- "how long do I stand at my desk"
- "standing desk reminder app"
- "desk height sensor"
- "sitting time tracker automatic"

Ad copy:
```
Headline: Track Sitting & Standing Automatically
Description: €12 sensor mounts under your desk. Open source app shows
real-time stats. No cloud. Pre-order the kit for €49.
URL: desk.zentala.io
```

**Campaign 2: Display/YouTube (awareness)**
Budget: 100 PLN/month

Target:
- Interest: standing desks, ergonomics, productivity, quantified self
- Custom audience: visitors of standing desk review sites

### Facebook/Instagram Ads

Budget: 200 PLN/month

**Ad 1: Problem hook (video/carousel)**
```
Image 1: "You bought a standing desk."
Image 2: "You've used it standing... twice this month."
Image 3: "This €12 sensor changes that." [photo of sensor]
Image 4: "Automatic tracking. Real coaching. Open source."
CTA: "Pre-order — €49"
```

**Ad 2: Data hook (single image)**
```
Image: Screenshot of app showing KPIs
Text: "I tracked my sitting for 30 days.
       I was standing only 8% of the time.
       Then I added coaching. Now it's 22%.
       The sensor costs €12. The app is free."
CTA: "Learn more"
```

Target audience:
- Age: 25-45
- Interests: standing desks, ergonomics, remote work, home office,
  developer tools, quantified self, biohacking
- Geo: EU (start with DE, NL, SE, DK — high standing desk adoption)

### Ad KPIs

| Metric | Target |
|--------|--------|
| CPC (Google Search) | < 2 PLN |
| CPC (Facebook) | < 1.5 PLN |
| CTR (Google Search) | > 3% |
| CTR (Facebook) | > 1.5% |
| Landing page → pre-order conversion | > 2% |
| Cost per pre-order | < 50 PLN (< €12) |

If cost per pre-order exceeds 50 PLN → pause ads, iterate messaging.

---

## 5. Content Calendar (First 4 Weeks)

### Week 1: Launch

| Day | Action | Platform |
|-----|--------|----------|
| Mon | Deploy desk.zentala.io landing page | — |
| Mon | Set up Stripe Checkout + waitlist form | — |
| Mon | Set up Google Analytics events (pre-order click, waitlist signup) | — |
| Tue | Post 1: r/standingdesks | Reddit |
| Tue | Post 4: r/selfhosted | Reddit |
| Wed | Post 2: r/quantifiedself | Reddit |
| Thu | Post 3: Show HN | Hacker News |
| Fri | Post 5: r/homeautomation | Reddit |
| Fri | Respond to ALL comments (critical!) | Reddit/HN |

### Week 2: Ads + Content

| Day | Action |
|-----|--------|
| Mon | Launch Google Search ads |
| Tue | Launch Facebook carousel ad |
| Wed | Blog post: "What I learned tracking my sitting for 30 days" (on desk.zentala.io/blog or medium) |
| Thu | Cross-post blog to Reddit (r/productivity, r/remotework) |
| Fri | Review ad performance, adjust targeting |

### Week 3: Depth

| Day | Action |
|-----|--------|
| Mon | Video: 60-second app demo (screen recording + voiceover) → YouTube |
| Tue | Share video on Reddit (r/standingdesks, r/quantifiedself) |
| Wed | zentala.agency case study page published |
| Thu | LinkedIn post linking to case study (for professional visibility) |
| Fri | Review metrics, write Week 3 summary |

### Week 4: Iterate

| Day | Action |
|-----|--------|
| Mon | A/B test landing page headline (if traffic > 500 visits) |
| Tue | New Reddit post with updated data (2 months tracking) |
| Wed | Try r/InternetIsBeautiful or r/dataisbeautiful (if data visualizations ready) |
| Thu | Adjust ad budget based on ROAS |
| Fri | Monthly report: visits, pre-orders, waitlist, ad spend, conversion rates |

---

## 6. KPI Dashboard & Measurement

### Primary KPIs (check weekly)

| KPI | How to measure | Week 1 target | Month 1 target |
|-----|---------------|---------------|----------------|
| Landing page visits | Google Analytics | 500 | 3,000 |
| Pre-order count | Stripe Dashboard | 5 | 30 |
| Waitlist signups | Mailchimp / form submissions | 20 | 100 |
| Pre-order conversion rate | pre-orders / visits | >1% | >2% |
| Waitlist conversion rate | signups / visits | >3% | >5% |
| Reddit post upvotes (total) | Manual check | 50 | — |
| HN points | Manual check | 10 | — |
| GitHub stars (desk repo) | GitHub | 10 | 50 |
| Ad spend total | Google+FB dashboards | 0 (week 1 organic) | 500 PLN |
| Cost per pre-order (ads) | ad spend / ad-driven pre-orders | — | <50 PLN |

### Secondary KPIs (check monthly)

| KPI | How to measure |
|-----|---------------|
| zentala.agency case study views | Google Analytics |
| Discord members | Discord server |
| Organic search impressions | Google Search Console |
| Email open rate (waitlist updates) | Mailchimp |
| Referral sources | GA Acquisition report |

### Measurement Setup (one-time)

1. **Google Analytics 4** — already on desk.zentala.io. Add events:
   - `preorder_click` — when pre-order button clicked
   - `waitlist_signup` — when email submitted
   - `diy_click` — when GitHub/DIY link clicked
   - `faq_expand` — when FAQ item opened

2. **UTM parameters** — on every link you share:
   - Reddit: `?utm_source=reddit&utm_medium=post&utm_campaign=launch&utm_content=standingdesks`
   - HN: `?utm_source=hackernews&utm_medium=post&utm_campaign=launch`
   - Facebook: auto-tagged by FB ads
   - Google: auto-tagged by Google ads
   - zentala.agency: `?utm_source=agency&utm_medium=referral`

3. **Stripe Dashboard** — track pre-orders, revenue, refunds

4. **Weekly check routine** (15 min every Friday):
   - Open GA → check visits, sources, conversion events
   - Open Stripe → check pre-orders, revenue
   - Open Mailchimp → check signups
   - Open Reddit → check post karma, new comments
   - Log in spreadsheet or .plan/ file

### Go/No-Go Decisions

| Checkpoint | Metric | Decision |
|-----------|--------|----------|
| After Week 1 (organic only) | 0 pre-orders, <5 waitlist | Messaging problem → rewrite headline + hero |
| After Week 2 (ads started) | <3 pre-orders, CPC > 3 PLN | Ad targeting problem → narrow audience |
| After Month 1 | <10 pre-orders total | Serious demand question → pivot or pause |
| After Month 1 | 30+ pre-orders | On track → keep going, increase ad budget |
| After Month 1 | 100+ waitlist, <10 pre-orders | Price too high or trust issue → test lower price |
| 100 pre-orders reached | — | START PRODUCTION |

---

## 7. Pre-Order Counter Implementation

Simple approach (no backend needed):

1. Stripe webhook → Cloudflare Worker → updates a JSON file in R2 or KV
2. Landing page fetches JSON on load → displays counter
3. Fallback: manually update a `preorder-count.json` in the Astro repo
   and redeploy (takes 30 seconds, fine for <10 orders/day)

Counter display on landing page:
```
🎯 34 / 100 pre-orders
████████████░░░░░░░░░░░░░░░░░░░░ 34%
Production starts at 100. ~3 months to delivery.
```

---

## 8. Email Sequences

### Waitlist signup → Welcome email (immediate)
```
Subject: You're on the list — zntlDesk

Hey,

Thanks for joining the zntlDesk waitlist.

Here's what happens next:
- We're collecting pre-orders (currently at XX/100)
- When we hit 100, production starts
- You'll get an email when the kit is available

In the meantime:
- Check out the open source app: [GitHub]
- Join our Discord: [link]

If you want to skip the wait and lock in your kit now:
→ Pre-order for €49: [link]

— zentala
```

### Monthly update (to waitlist + pre-order customers)
```
Subject: zntlDesk update — XX/100 pre-orders

Quick update:
- Pre-orders: XX / 100
- New feature: [whatever shipped this month]
- Community: XX Discord members

[Pre-order if you haven't yet]

— zentala
```

---

## 9. Real Data Needed Before Launch

The Reddit posts reference specific numbers ("8% standing time", "3.5h longest session").
These MUST be real data from zentala's own usage.

**Action items before posting:**
1. Export 30 days of personal usage data from the app
2. Calculate: daily standing %, position changes/day, longest session
3. Calculate "before vs after" if possible (first week vs last week)
4. Screenshot the app showing real KPIs (not mock data)
5. Photo of sensor mounted under real desk (not render, not mockup)
6. Optional: 30-second screen recording of the app in use

**If you don't have 30 days of data yet:** Use whatever you have.
"2 weeks of data" is fine. Don't wait for perfect data.

---

## 10. SEO Strategy

### 10a. Technical SEO (landing page)

**On-page essentials (implement during landing page build):**

```html
<!-- Primary page -->
<title>zntlDesk — Automatic Sit/Stand Tracker for Height-Adjustable Desks</title>
<meta name="description" content="Track sitting and standing automatically
  with a €12 sensor. Open source app with real-time coaching, gamification,
  and privacy-first design. Pre-order the dev kit for €49." />

<!-- Open Graph (Facebook, LinkedIn) -->
<meta property="og:title" content="Your standing desk is useless if you never stand." />
<meta property="og:description" content="Automatic sit/stand tracking.
  Real-time coaching. Open source. No cloud." />
<meta property="og:image" content="https://desk.zentala.io/images/og-card.png" />
<meta property="og:type" content="product" />

<!-- Twitter Card -->
<meta name="twitter:card" content="summary_large_image" />
<meta name="twitter:title" content="zntlDesk — Automatic Sit/Stand Tracker" />
<meta name="twitter:image" content="https://desk.zentala.io/images/og-card.png" />

<!-- Schema.org Product markup -->
<script type="application/ld+json">
{
  "@context": "https://schema.org",
  "@type": "Product",
  "name": "zntlDesk Dev Kit",
  "description": "Automatic sitting/standing tracker for height-adjustable desks",
  "offers": {
    "@type": "Offer",
    "price": "49.00",
    "priceCurrency": "EUR",
    "availability": "https://schema.org/PreOrder"
  },
  "brand": { "@type": "Brand", "name": "zentala" }
}
</script>
```

**OG image:** Create a 1200×630px image with: app screenshot + sensor photo +
headline text. This is what shows when anyone shares the link on social media.
Critical for click-through. Use Figma or Canva.

**Additional technical:**
- Sitemap.xml (Astro generates automatically)
- robots.txt (allow all)
- Canonical URLs on all pages
- Alt text on ALL images (descriptive, include keywords naturally)
- Page speed: Astro is static = fast by default. Optimize images (WebP, lazy load)
- Mobile responsive (Tailwind handles this, but verify)

### 10b. Target Keywords

**Primary keywords (high intent, low-medium competition):**

| Keyword | Monthly searches (est.) | Difficulty | Priority |
|---------|----------------------|------------|----------|
| standing desk tracker | 100-500 | Low | ★★★ |
| sit stand desk sensor | 50-200 | Low | ★★★ |
| standing desk reminder | 500-1000 | Medium | ★★★ |
| how long do I stand at my desk | 50-200 | Low | ★★★ |
| automatic standing desk tracking | 10-50 | Very low | ★★ |
| desk height sensor | 50-200 | Low | ★★ |

**Content keywords (informational, for blog — high volume):**

| Keyword | Monthly searches (est.) | Use for |
|---------|----------------------|---------|
| sitting disease | 1000-5000 | Blog article |
| standing desk benefits | 5000-10000 | Blog article |
| how long should I stand at my desk | 1000-5000 | Blog article |
| sitting too long health risks | 1000-5000 | Blog article |
| standing desk vs sitting | 5000-10000 | Blog comparison |
| best standing desk accessories | 1000-5000 | Blog + affiliate |
| how to use a standing desk properly | 500-2000 | Blog article |
| standing desk posture | 500-2000 | Blog article |
| sitting breaks at work | 500-2000 | Blog article |

**Long-tail keywords (blog articles that link to product):**

| Keyword | Article idea |
|---------|-------------|
| "I sit all day even with a standing desk" | Problem-aware article → product as solution |
| "standing desk not using it" | Same angle, different phrasing |
| "how to remember to stand up" | Solutions article → zntlDesk as automatic option |
| "standing desk habit tracker" | Product-aware → direct to landing page |
| "DIY standing desk sensor" | Tutorial → link to GitHub + dev kit |

### 10c. Blog Content Plan (SEO-driven)

**Location:** desk.zentala.io/blog/ (Astro content collections)

**Article 1 (Week 2): "What I Learned Tracking My Sitting for 30 Days"**
- Keywords: sitting tracker, how long do I sit, sitting habits
- Content: Personal data story, real numbers, before/after
- CTA: "Want to track yours? Here's how" → product link
- Length: 1500-2000 words
- This is also the Reddit cross-post content

**Article 2 (Week 4): "The Sitting Disease: Why Your Standing Desk Isn't Enough"**
- Keywords: sitting disease, standing desk benefits, standing desk not using
- Content: Health research summary + why manual solutions fail + automatic tracking
- CTA: Link to product
- Length: 2000-2500 words
- High-volume keyword, competes with health sites but our angle is unique (data-driven)

**Article 3 (Week 6): "How to Actually Use Your Standing Desk (Data-Backed Guide)"**
- Keywords: how to use standing desk, standing desk tips, sit stand schedule
- Content: Optimal sit/stand ratios, position change frequency, break rules
- Reference real research + zentala's own data
- CTA: "Or automate it with a sensor" → product link
- Length: 2000 words

**Article 4 (Week 8): "DIY Standing Desk Sensor: Build Your Own for €12"**
- Keywords: DIY standing desk sensor, desk height sensor, VL53L1X desk
- Content: Full tutorial with photos, wiring, firmware, app setup
- CTA: "Or buy the pre-assembled kit for €49" → product link
- Length: 2500+ words (tutorial format, lots of photos)
- This will rank for long-tail DIY queries AND drive dev kit sales

**Article 5 (Week 10): "Standing Desk Accessories That Actually Change Your Habits"**
- Keywords: standing desk accessories, best standing desk accessories
- Content: Review of accessories (mats, boards, timers) + zntlDesk as the tracking layer
- Affiliate links for mats/boards (supplementary revenue)
- CTA: product link
- Length: 2000 words

**Publishing cadence:** 1 article every 2 weeks. Each article targets a specific
keyword cluster. Internal links between articles + to product page.

### 10d. Link Building (Organic)

- Every Reddit/HN post with upvotes = backlink (nofollow but drives traffic)
- GitHub repo README links to desk.zentala.io (dofollow from github.com)
- zentala.agency links to desk.zentala.io (cross-site, both owned)
- Blog articles get shared on Reddit = more backlinks
- Pitch to: standing desk review sites, ergonomics blogs, maker blogs
- Submit to: Product Hunt (when ready for broader launch)
- Respond on Quora/StackExchange to "standing desk" questions with helpful answers + link

### 10e. Google Search Console Setup

1. Verify desk.zentala.io in Google Search Console
2. Submit sitemap.xml
3. Monitor: impressions, clicks, average position for target keywords
4. Track which blog articles rank and for what queries
5. Add to weekly KPI check routine

---

## 11. Social Proof Strategy

### 11a. Social Proof Types (ordered by credibility)

**Tier 1 — Quantitative results (MOST powerful)**
Real data showing behavior change. This is what sells.

```
Section on landing page: "Real Results"

┌────────────────────────────────────────────────┐
│  Before zntlDesk          After 30 days        │
│  ─────────────           ──────────────        │
│  Standing: 8%      →     Standing: 22%         │
│  Changes/day: 1.2  →     Changes/day: 4.8      │
│  Longest sit: 3.5h →     Longest sit: 52min    │
│                                                │
│  "I had no idea I was sitting this much.       │
│   The data changed my behavior in 3 days."     │
│                           — zentala (creator)   │
└────────────────────────────────────────────────┘
```

Start with YOUR OWN data. You're user #1. Your numbers are the first proof.

**Tier 2 — User testimonials (target profiles)**

These are TEMPLATES for what to collect from early users. Not fake —
these are the TYPES of testimonials to actively seek from dev kit buyers.

Profile A: The Remote Developer
```
"I work from home 5 days a week. I bought a standing desk 2 years ago
 but honestly I was only using it standing maybe once a week. After
 installing zntlDesk, I saw I was standing only 6% of the time. Now
 it's 25% and I change position 5 times a day. My back pain is gone."

 — [Name], Software Engineer, Berlin
 Photo: desk setup with sensor visible
```

Profile B: The Quantified Self Enthusiast
```
"I track everything — sleep, calories, steps. But I had zero data on
 my desk habits. zntlDesk filled that gap. The gamification is what
 hooked me — seeing my score drop when I sit too long actually makes
 me stand up. It's like a Fitbit for your desk."

 — [Name], Data Scientist, Amsterdam
 Photo: app screenshot with KPIs
```

Profile C: The Health-Conscious Professional
```
"My physiotherapist told me to stand more. But 'stand more' isn't
 actionable. zntlDesk made it concrete: I can see exactly how long
 I've been sitting, and the gentle nudge at 40 minutes actually works.
 I went from 0 position changes to 4 per day."

 — [Name], Product Manager, Stockholm
 Photo: standing at desk
```

Profile D: The DIY Maker
```
"Built it from the GitHub instructions in an afternoon. The sensor is
 €12, the app is free, and it works flawlessly. I added my own skin
 to match my desktop theme. This is what open source hardware should be."

 — [Name], Maker / Hardware Hacker, Warsaw
 Photo: custom build
```

**How to collect these testimonials:**
1. First 10 dev kit buyers → email after 2 weeks: "How's it going? Would you share your experience?"
2. Offer: name in credits + 3 months extra Pro for a testimonial with photo
3. Reddit commenters who say positive things → DM: "Mind if I quote you on the site?"
4. Discord members who share screenshots → ask permission to use

**Tier 3 — Trust badges / social signals**

```
Section on landing page: below hero or above pricing

[⭐ XX GitHub stars] [👥 XX pre-orders] [🧪 500+ tests] [🔓 Open Source]
```

Add as numbers grow:
- "As seen on: Hacker News, r/standingdesks" (after posts get traction)
- "XX countries" (when you have orders from multiple countries)

**Tier 4 — Video testimonial (ultimate proof, later)**

30-second video of a real user showing their setup + sharing one number
("I went from 5% to 28% standing"). This converts better than anything.
Collect from Founder's Edition buyers (they're most engaged).

### 11b. "Why Not Just Use...?" Comparison Section

Add to landing page between Features and Pricing:

```
Section: "Why zntlDesk?"

┌─────────────────┬──────────┬───────────┬──────────┬──────────┐
│                  │ zntlDesk │ Phone app │ Smart-   │ Desktop  │
│                  │          │ (manual)  │ watch    │ timer    │
├─────────────────┼──────────┼───────────┼──────────┼──────────┤
│ Automatic        │ ✅       │ ❌ manual │ ❌ no    │ ❌ no    │
│ detection        │          │ logging   │ desk     │ desk     │
│                  │          │           │ height   │ data     │
├─────────────────┼──────────┼───────────┼──────────┼──────────┤
│ Knows desk       │ ✅       │ ❌        │ ❌       │ ❌       │
│ height           │          │           │          │          │
├─────────────────┼──────────┼───────────┼──────────┼──────────┤
│ Real-time        │ ✅       │ ❌        │ partial  │ ❌       │
│ coaching         │          │           │          │          │
├─────────────────┼──────────┼───────────┼──────────┼──────────┤
│ Privacy          │ ✅ local │ ❌ cloud  │ ❌ cloud │ ✅ local │
│                  │          │           │          │          │
├─────────────────┼──────────┼───────────┼──────────┼──────────┤
│ Open source      │ ✅       │ ❌        │ ❌       │ some     │
├─────────────────┼──────────┼───────────┼──────────┼──────────┤
│ No subscription  │ ✅       │ freemium  │ ❌       │ ✅       │
│ required         │          │           │          │          │
├─────────────────┼──────────┼───────────┼──────────┼──────────┤
│ Gamification     │ ✅       │ basic     │ basic    │ ❌       │
├─────────────────┼──────────┼───────────┼──────────┼──────────┤
│ Works without    │ ✅       │ ❌        │ ✅       │ ✅       │
│ thinking         │          │           │          │          │
└─────────────────┴──────────┴───────────┴──────────┴──────────┘

"The only solution that knows your actual desk height
 and coaches you automatically."
```

---

## 12. Conversion Optimization

### 12a. Exit Intent Popup

When mouse moves toward browser close/back button:

```
┌─────────────────────────────────────────────────┐
│                                                 │
│  Wait — before you go:                          │
│                                                 │
│  Join 142 people waiting for zntlDesk.          │
│  We'll email you once when it ships.            │
│                                                 │
│  [your@email.com] [Join Waitlist]               │
│                                                 │
│  No spam. Unsubscribe anytime.                  │
│                                                 │
└─────────────────────────────────────────────────┘
```

Implementation: simple JS (mouseleave on document), show once per session,
cookie to not show again for 7 days. No external library needed.

### 12b. Urgency & Scarcity Triggers

**Pre-order counter (already planned):**
```
🎯 34 / 100 pre-orders — production starts at 100
```

**Recent activity (when pre-orders start coming in):**
```
"Someone from Berlin just pre-ordered" (shows for 5 seconds, then fades)
```
Implementation: Stripe webhook → store last 5 orders (city only, from Stripe
billing address) → display rotating. If no recent orders, don't show.

**Founder's Edition urgency:**
```
"Founder's Edition: 12 claimed this week"
```
No artificial scarcity (no limit), but showing momentum creates social proof.

### 12c. "As Seen On" Section

Empty at launch. Fill as traction builds:

```
After HN front page:  [Hacker News logo]
After Reddit viral:   [Reddit logo] "Top post on r/standingdesks"
After blog features:  [blog logos]
After Product Hunt:   [PH badge]
```

Place between hero and problem section (or below hero). Even 1 logo helps.

### 12d. Sticky CTA (mobile)

On mobile, fixed bottom bar:
```
┌──────────────────────────────────────┐
│ Pre-order Dev Kit — €49  [Order →]   │
└──────────────────────────────────────┘
```

Always visible while scrolling. Disappears when pricing section is in view
(to avoid redundancy).

---

## 13. Competitive Positioning

### 13a. Competitor Landscape

| Competitor type | Examples | Their weakness |
|----------------|---------|---------------|
| **Manual tracking apps** | StandUp!, BreakTimer, Stretchly | No desk height data, user forgets to log, abandoned in days |
| **Smartwatch apps** | Apple Watch Stand reminders | Doesn't know desk height, just reminds hourly, no coaching |
| **Smart desks (built-in)** | IKEA BEKANT, Uplift | Expensive ($800+), tracks height but no coaching/gamification |
| **Pomodoro timers** | Forest, Focus To-Do | Time-based not position-based, no desk awareness |
| **Nothing** (most people) | — | They bought a desk, don't use it standing |

### 13b. Our Unique Position

```
"zntlDesk is the only product that:
 1. Knows your actual desk height (automatic, not manual)
 2. Coaches you to change position (not just shows data)
 3. Runs 100% locally (no cloud, no account)
 4. Is open source (verify privacy yourself)
 5. Costs €49, not €800+"
```

### 13c. Positioning Statement (for all marketing)

**For standing desk owners** who bought a desk but still sit all day,
**zntlDesk** is an **automatic tracking sensor + coaching app**
that **measurably changes your sitting habits**.

Unlike manual tracking apps that get abandoned in days,
**zntlDesk detects your position automatically** and coaches you
with real-time feedback, gamification, and smart alerts.

**Open source. Privacy-first. €49.**

---

## 14. "Share My Stats" — Viral Growth Engine

### Concept

In-app button that generates a shareable image/card showing the user's ergonomic stats.
Like Spotify Wrapped, but for desk habits. Users post it on social media → free advertising.

### Shareable Card Design

```
┌──────────────────────────────────────────┐
│  🏆 My Desk Stats This Week             │
│                                          │
│  Standing time:  28%  ███████░░░░░       │
│  Position changes: 23                    │
│  Longest sit: 42 min                     │
│  Daily score: +47                        │
│  Streak: 🔥 12 days                      │
│                                          │
│  desk.zentala.io — track yours free      │
└──────────────────────────────────────────┘
```

### Where users can share:

| Platform | Format | How |
|----------|--------|-----|
| **Twitter/X** | Image + text | "Share" button → opens X with pre-filled text + image |
| **LinkedIn** | Image + text | Same flow, professional framing |
| **Facebook** | Image | Share button → FB share dialog |
| **Forum signatures** | Dynamic image URL | `![My Desk Stats](desk.zentala.io/stats/USER_ID.png)` — image regenerated daily |
| **Discord** | Image | Copy image → paste in channel |
| **Reddit** | Image | Download → upload to post |
| **Instagram Stories** | Image | Download → share to story |

### Forum Signature Badge (dynamic)

Special feature: a URL that returns a live-updating PNG of your stats.
Like GitHub contribution badges or Steam profile cards.

```
[img]https://desk.zentala.io/badge/abc123.png[/img]
```

Shows: standing %, streak, daily score. Updates daily.
Requires: cloud endpoint (Cloudflare Worker + D1) — ties into Pro/telemetry.

**Free tier:** Static image (generated on share, snapshot of that moment).
**Pro tier:** Dynamic badge URL (updates daily, requires cloud account).

### Pre-filled share text:

**Twitter/X:**
```
This week I changed position 23 times at my standing desk.
Standing time: 28%. Longest sit: 42 min.

Tracking automatically with @zntlDesk — open source, privacy-first.
desk.zentala.io
```

**LinkedIn:**
```
I've been tracking my desk habits automatically for 3 weeks.

Results:
✅ Standing time: 8% → 28%
✅ Position changes: 1.2/day → 4.8/day
✅ Longest sitting session: 3.5h → 42 min

The tool is open source and runs 100% locally.
If you have a standing desk, this is worth trying.

#ergonomics #standingdesk #health #remotework
desk.zentala.io
```

### Implementation (app side):

1. Button in app: "Share My Stats" (in popup, near KPI strip)
2. Generates PNG image (HTML → canvas → PNG, or server-side via CF Worker)
3. Options: "Copy image", "Share to Twitter", "Share to LinkedIn", "Get badge URL"
4. Pre-filled text with user's real numbers
5. Link always points to desk.zentala.io (with UTM: `?utm_source=share&utm_medium=social`)

### Why this matters more than ads:

- **Trust:** People trust peers, not ads. A friend posting their stats > any ad.
- **Specificity:** Real numbers ("28% standing") are more compelling than marketing copy.
- **Cost:** €0 per impression. Ads cost €0.50-2 per click.
- **Compound:** Each user who shares brings more users who share.
- **Content:** User-generated content for landing page social proof section.

---

## 15. Product Hunt Launch Strategy

### When to launch:

After Reddit/HN posts (Wave 3) — when you have:
- ✅ Live landing page with pre-orders working
- ✅ At least 10 pre-orders (social proof)
- ✅ GitHub repo with stars
- ✅ At least 1 Reddit/HN post with traction
- ✅ Demo video (15-60 seconds)

### Preparation (1-2 weeks before):

1. **Find a "Hunter"** — someone with PH reputation who will submit your product
   - Search PH for people who hunt hardware/developer tools
   - DM them: "I built this, would you be interested in hunting it?"
   - Alternative: submit yourself (lower initial visibility but still works)

2. **Prepare PH listing:**
   - Tagline (60 chars max): "Automatic sit/stand tracker — open source, privacy-first"
   - Description (260 chars): "A €12 sensor under your desk tracks sitting/standing
     automatically. Open source app with real-time coaching, gamification, and
     smart alerts. Your data stays on your computer."
   - Topics: Developer Tools, Health, Open Source, Hardware, Productivity
   - Maker comment: your founder story (abbreviated)
   - Gallery: 5 images (sensor photo, app screenshots, before/after data, demo GIF)
   - First comment: longer version of founder story + link to GitHub

3. **Rally upvotes (first hour is critical):**
   - Email waitlist morning of launch: "We're live on Product Hunt!"
   - Post in Discord
   - Share on Twitter/X
   - Ask friends/colleagues to upvote (NOT fake accounts)
   - Goal: 10+ upvotes in first hour → enters trending

4. **Best timing:**
   - Day: Tuesday, Wednesday, or Thursday
   - Time: 00:01 PST (Product Hunt resets daily at midnight PST)
   - Post early so you have full day to accumulate votes

### After PH launch:

- Respond to EVERY comment on PH (fast responses boost ranking)
- Add "Featured on Product Hunt" badge to landing page
- Blog post: "Our Product Hunt launch — what we learned"
- Track: PH views → landing page visits → pre-orders (UTM tagged)

---

## 16. Drip Email Sequence (Waitlist → Pre-order Conversion)

### 5-email sequence, sent every 3-4 days after signup:

**Email 1: Welcome + The Problem (Day 0)**
```
Subject: The standing desk lie

Hey [name],

You bought a standing desk to be healthier.
But let me ask you: how much did you actually stand today?

Most people guess "a lot." The data says otherwise.

I tracked my own desk habits for 30 days. I was standing 8% of the time.
Eight percent. On a desk I spent €800 on.

That's why I built zntlDesk — but more on that in the next email.

— zentala

P.S. If you're curious about your own standing time,
the app is free and open source: [GitHub link]
```

**Email 2: The Discovery (Day 3)**
```
Subject: I tracked my sitting for 30 days — here's what I found

[name],

Here are my actual numbers from 30 days of automatic tracking:

Before zntlDesk:
• Standing time: 8%
• Position changes per day: 1.2
• Longest continuous sitting: 3.5 hours (!!)

The scariest part? I had NO IDEA.
I genuinely thought I was "using my standing desk."

I was not.

Next email: what I changed (and how my numbers look now).

— zentala
```

**Email 3: The Solution (Day 7)**
```
Subject: 8% → 28% standing — here's how

[name],

After seeing my embarrassing numbers, I experimented
with different approaches to change my behavior:

❌ Timer apps — I ignored them after 2 days
❌ Sticky notes — "stand more" is not actionable
❌ Smartwatch reminders — doesn't know my desk height
✅ Automatic tracking + real-time coaching

The combination that worked:
1. A sensor that KNOWS if I'm sitting or standing (automatic)
2. A progress bar that slowly turns red (visual pressure)
3. Gamification with real penalties (can't ignore it)
4. Smart alerts that escalate (gentle → firm)

My numbers after 30 days:
• Standing time: 8% → 28%
• Position changes: 1.2/day → 4.8/day
• Longest sit: 3.5h → 52 minutes

The sensor costs €12. The app is free.
Or you can pre-order the ready-made kit:
→ [Pre-order — €49]

— zentala
```

**Email 4: Social Proof (Day 11)**
```
Subject: What others are saying about zntlDesk

[name],

Quick update: [XX] people have already pre-ordered zntlDesk.

Here's what early users are telling me:

"[real testimonial when available]"
— [Name], [City]

"[another testimonial]"
— [Name], [City]

We're at [XX]/200 pre-orders for the Basic Kit.
When we hit 200, production starts.

→ [Secure your spot — €49]

— zentala
```

**Email 5: Last Call (Day 15)**
```
Subject: [XX] spots left before production starts

[name],

We're at [XX]/200 pre-orders for the Basic Kit.

If you've been thinking about it — now is the time.
After we hit the threshold:
• Production starts (~3 months to delivery)
• Price may increase for the next batch

Your options:
• Basic Kit (€49) — sensor + app [Pre-order]
• Pro Kit (€79) — sensor + vibration motor + presence sensor [Pre-order]
• Founder's Edition (€149) — everything + 5yr cloud subscription [Pre-order]

Full refund available anytime before shipping. Zero risk.

→ [Choose your kit]

Questions? Reply to this email — I read everything personally.

— zentala
```

### Automation:

Use Mailchimp free tier (up to 500 contacts):
- Automation: "Waitlist signup" trigger → 5 emails, 3-4 day intervals
- Personalization: [name] from signup form (optional field)
- Unsubscribe in every email (required by law)
- Track: open rate, click rate, pre-order conversion per email

---

## 17. Referral Program

### Mechanism:

After pre-order purchase, customer receives email with unique referral link:

```
Subject: Thanks for your pre-order! Share the love ❤️

Hey [name],

Your zntlDesk kit is reserved! 🎉

Know someone who'd benefit from better desk habits?
Share your personal link — if they pre-order, you BOTH save:

Your link: desk.zentala.io/?ref=ABC123

• Your friend: €5 off their first kit
• You: €5 credited back (or applied to upgrade)

Share with: [Twitter] [LinkedIn] [Copy link] [Email a friend]

— zentala
```

### Implementation:

1. Stripe Checkout with coupon codes (generated per customer)
2. Unique referral URL with `?ref=CODE` parameter
3. Plausible/CF Analytics tracks referral source
4. On purchase with valid ref code → apply €5 discount + flag referrer for credit
5. Manual process at first (check Stripe, send credit). Automate later if volume justifies.

### Economics:

- Cost per referral: €5 discount + €5 credit = €10
- Cost per acquisition via ads: ~€12-50
- Referral is CHEAPER and MORE trusted than ads

---

## 18. Social Wall — Live Mentions on Landing Page

### Concept:

Section on landing page that shows real-time social media mentions of zntlDesk.
When someone tags @zntlDesk or uses #zntlDesk on Twitter/X, their post appears
on the landing page automatically.

### Implementation options:

| Option | Cost | Effort | Notes |
|--------|------|--------|-------|
| **Twitter embed + manual curation** | Free | Low | Manually pick best tweets, embed via Twitter widget |
| **Curator.io** | Free (small plan) | Low | Aggregates social mentions, embeddable widget |
| **Walls.io** | €49/mo | Zero | Full social wall, auto-updates, multiple platforms |
| **Custom CF Worker** | Free | Medium | Poll Twitter API, cache results, serve to frontend |

**Recommendation for start:** Manual curation (free).
- Create a section: "What people are saying"
- Manually embed 3-5 best tweets/posts
- Update weekly as new mentions come in
- Switch to automated tool when mentions exceed 10/week

### Monitoring mentions:

1. **Twitter/X:** Search `zntlDesk OR desk.zentala.io` daily
2. **Reddit:** Search across standing desk subreddits
3. **Google Alerts:** Set up alert for "zntlDesk"
4. **GitHub:** Watch for stars, issues, forks (already visible)

### Encouraging mentions:

- "Share My Stats" feature (Section 14) generates posts that tag @zntlDesk
- Pre-filled share text includes #zntlDesk hashtag
- After purchase email: "Tell the world! Tag @zntlDesk"
- Discord community: encourage sharing desk setup photos
