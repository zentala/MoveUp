# Validation Strategy & Go-to-Market Plan

**Date**: 2026-03-25
**Status**: PARTIALLY SUPERSEDED by `story-driven-launch.md` (2026-03-26)
**Superseded sections**: Pricing (was 2 tiers, now 3 — see `config/pricing.json`), threshold (was 100 global, now per-tier), landing page structure (now story-driven)
**Still valid**: Fulfillment plan, shipping timeline, assembly checklist, validation thresholds
**Depends on**: Business Vision (2026-03-24)

---

## 1. Core Hypothesis to Validate

> "People with sit/stand desks will pay 30-50 EUR for a sensor kit that
> automatically tracks their sitting/standing habits and motivates them
> to take breaks."

Secondary hypotheses:
- People care enough about sitting habits to install hardware
- The "automatic tracking" value proposition resonates (vs manual apps)
- Developers AND non-developers are interested
- People will pay upfront (pre-sale), not just "join waitlist"

---

## 2. Pricing Analysis

### Cost structure
- BOM from China: ~40-60 PLN (~10-15 EUR)
- Plexi/PCB mount: ~20-30 PLN (~5-7 EUR)
- Assembly time: ~30 min per unit
- Shipping (EU): ~15-30 PLN (~4-7 EUR)
- **Total cost per unit: ~75-120 PLN (~18-30 EUR)**

### Pricing decision
- **Minimum viable price: 30 EUR (~130 PLN)** — barely covers costs + shipping
- **Target price: 49 EUR (~210 PLN)** — covers costs, leaves margin for iteration
- **Maximum: 59 EUR (~250 PLN)** — with 3 months Pro included

### Why NOT cheaper:
- Dropping from 49 EUR to 25 EUR will NOT 2x the buyers in this niche
- The audience is small and specific (sit/stand desk owners who care about ergonomics)
- Price elasticity is low — people who want this will pay 49 EUR; people who don't, won't buy at 15 EUR either
- Lower price = more units = more support = more shipping headaches
- zentala doesn't want to run a hardware logistics business

### Why NOT more expensive:
- Above 59 EUR, comparison with commercial gadgets kicks in
- Dev kit positioning doesn't support premium pricing
- Goal is user acquisition, not hardware profit

### Recommended: Two tiers on landing page

| Tier | Price | What's included |
|------|-------|-----------------|
| **DIY** | Free | Open source firmware + app + BOM list + assembly guide |
| **Dev Kit** | 49 EUR | Assembled sensor + mount + USB cable + app |

The DIY tier exists to:
- Prove open source commitment
- Let price-sensitive devs participate
- Create content (assembly videos, blog posts)
- NOT cannibalize sales (assembly takes 5-10h, most will buy)

---

## 3. Landing Page Strategy

### URL: desk.zentala.io

Reuse existing Astro + Tailwind infrastructure from current desk.zentala.io project.

### Page Structure (top to bottom)

```
┌─────────────────────────────────────────────┐
│ HERO                                        │
│ "Track sitting & standing automatically."   │
│ "Build healthier work habits."              │
│                                             │
│ [Pre-order Dev Kit — 49 EUR] [DIY Guide →]  │
│                                             │
│ 📸 Photo: sensor mounted under real desk    │
│ 📸 Screenshot: app popup with real data     │
└─────────────────────────────────────────────┘

┌─────────────────────────────────────────────┐
│ PROBLEM                                     │
│ "You bought a standing desk. You still sit  │
│  all day. Sound familiar?"                  │
│                                             │
│ • Average standing desk user stands <15%    │
│ • Manual tracking apps get abandoned in 3d  │
│ • You need AUTOMATIC detection + coaching   │
└─────────────────────────────────────────────┘

┌─────────────────────────────────────────────┐
│ HOW IT WORKS (3 steps)                      │
│                                             │
│ 1. Mount sensor under desk (30 seconds)     │
│    [photo: sensor on plexi under desk]      │
│                                             │
│ 2. Plug USB into computer                   │
│    [photo: USB cable to laptop]             │
│                                             │
│ 3. App tracks everything automatically      │
│    [screenshot: overlay bar + popup]        │
└─────────────────────────────────────────────┘

┌─────────────────────────────────────────────┐
│ FEATURES                                    │
│                                             │
│ 🎯 Real-time overlay bar (top of screen)    │
│ 📊 Daily statistics & KPIs                  │
│ 🏆 Gamification — score, streaks, coaching  │
│ 🔔 Smart notifications (not annoying)       │
│ 🔓 Open source — customize everything       │
│ 🔌 Plugin system — community skins          │
└─────────────────────────────────────────────┘

┌─────────────────────────────────────────────┐
│ APP SCREENSHOTS                             │
│                                             │
│ [popup main view] [overlay bar] [settings]  │
│ [KPI dashboard]   [timeline]    [debug]     │
└─────────────────────────────────────────────┘

┌─────────────────────────────────────────────┐
│ WHAT YOU GET (Dev Kit)                      │
│                                             │
│ ✅ VL53L1X ToF sensor (assembled)           │
│ ✅ Microcontroller (pre-flashed firmware)   │
│ ✅ Mount (plexi or PCB carrier)             │
│ ✅ USB cable                                │
│ ✅ Desktop app (Windows, open source)       │
│ ✅ Access to Discord community              │
│ ✅ Your feedback shapes the product         │
│                                             │
│ 49 EUR — ships in ~3 months                 │
│ [Pre-order Now]                             │
└─────────────────────────────────────────────┘

┌─────────────────────────────────────────────┐
│ DIY OPTION                                  │
│                                             │
│ "Prefer to build it yourself?"              │
│ Full BOM, firmware, assembly guide — free.  │
│ [GitHub] [Assembly Guide]                   │
└─────────────────────────────────────────────┘

┌─────────────────────────────────────────────┐
│ OPEN SOURCE                                 │
│                                             │
│ App is fully open source.                   │
│ Skins, plugins, integrations — community.   │
│ Your data stays on YOUR computer.           │
│                                             │
│ [GitHub ⭐] [Discord 💬]                    │
└─────────────────────────────────────────────┘

┌─────────────────────────────────────────────┐
│ ROADMAP                                     │
│                                             │
│ ✅ Desktop app (Windows)                    │
│ ✅ Real-time tracking + overlay bar         │
│ ✅ Gamification + KPIs                      │
│ 🔜 Phone as desk display                   │
│ 🔜 Cloud sync + history                    │
│ 🔜 Smartwatch integration                  │
│ 🔜 Linux / macOS support                   │
└─────────────────────────────────────────────┘

┌─────────────────────────────────────────────┐
│ FAQ                                         │
│                                             │
│ "Does it work with any desk?"              │
│ → Yes, any sit/stand desk. Mount under top. │
│                                             │
│ "Do I need WiFi?"                          │
│ → No. USB only. No cloud, no account.       │
│                                             │
│ "Is my data private?"                      │
│ → 100%. Everything stays on your computer.  │
│                                             │
│ "When does it ship?"                       │
│ → ~3 months after pre-order opens.          │
│                                             │
│ "What if I want a refund?"                 │
│ → Full refund before shipping.              │
└─────────────────────────────────────────────┘

┌─────────────────────────────────────────────┐
│ FOOTER CTA                                  │
│                                             │
│ [Pre-order Dev Kit — 49 EUR]                │
│ or [Join Waitlist — free]                   │
│                                             │
│ "No spam. One email when it ships."         │
└─────────────────────────────────────────────┘
```

### Pre-order Mechanics

**Model: Threshold-based pre-order (crowdfunding-like)**

The deal:
- You can join the **free waitlist** (email only) — notified when production starts
- You can **pre-order and pay now** (49 EUR) — reserved spot, ships when threshold met
- **Production starts when 100 pre-orders are collected**
- If threshold not met within [TBD] months → full refund to all pre-order customers
- Delivery: ~3 months after threshold is met

Why 100:
- Covers BOM bulk order from China (better prices at 100+ units)
- Covers assembly time investment
- Proves genuine demand (not just "cool idea" clicks)
- zentala has an active company — can process payments legally now

**Payment:** Stripe Checkout (simplest, handles EU VAT)
- Pre-order button → Stripe payment page → confirmation email
- Full refund available at any time before shipping
- No Stripe subscription needed — one-time payment

**Waitlist (free):** For people not ready to pay
- Email collection via: Mailchimp free tier, or simple Google Form
- Purpose: measure interest without payment commitment
- Lower commitment = more signups = broader signal
- Follow-up emails: progress updates, threshold counter, launch notification

**Landing page shows live counter:**
- "23 / 100 pre-orders — help us reach the goal!"
- Creates urgency + social proof
- Waitlist count shown separately: "147 people waiting"

**Key metrics to track:**
- Landing page visits (Google Analytics — already set up on desk.zentala.io)
- Pre-order conversion rate (visits → payments)
- Waitlist signups
- Traffic sources (which forum/post drove visits)
- Time to reach threshold (velocity matters)

### Validation Thresholds

| Metric | Signal |
|--------|--------|
| <10 pre-orders after 1000 visits | STOP — no demand at this price/messaging |
| 10-30 pre-orders, slow growth | ITERATE — change messaging, try new channels |
| 50+ pre-orders, steady growth | ON TRACK — keep marketing, prepare production |
| 100 pre-orders reached | GO — start production, order components |
| 200+ waitlist signups | Strong interest signal even if pre-orders lag |

---

## 4. Marketing Strategy (Zero Budget)

### Launch Posts (Week 1)

**Post 1: Reddit r/standingdesks**
- Title: "I built an automatic sitting/standing tracker with a $12 sensor"
- Content: problem statement, photos, demo video (30s screen recording)
- CTA: link to landing page
- Tone: maker/builder, not salesy

**Post 2: Reddit r/quantifiedself**
- Title: "Tracking my sitting habits automatically — open source desk sensor"
- Content: data/stats angle, KPI screenshots, what I learned about my habits
- CTA: link to GitHub + landing page

**Post 3: Hacker News (Show HN)**
- Title: "Show HN: Open-source sitting/standing tracker for height-adjustable desks"
- Content: short technical description, link to GitHub, link to landing page
- Best time: Tuesday-Thursday, 9-11 AM EST

**Post 4: Reddit r/homeautomation or r/selfhosted**
- Title: "DIY desk height sensor — tracks sitting/standing, open source"
- Angle: self-hosted, privacy-first, no cloud

**Post 5: Polish communities**
- elektroda.pl (hardware angle)
- wykop.pl (Polish maker community)
- 4programmers.net (developer angle)

### Content Sequence (Weeks 2-4)

| Week | Content | Platform |
|------|---------|----------|
| 2 | "What I learned tracking my sitting for 2 weeks" | Blog + Reddit |
| 2 | Assembly video (DIY guide) | YouTube + Reddit |
| 3 | "Why I made the app open source" | Blog + HN |
| 3 | App demo video (features walkthrough) | YouTube |
| 4 | "10 users later — what they asked for" | Blog + Reddit |

### Key Rules for Posts

- **Lead with the PROBLEM, not the product** — "I sit 8h/day even though I have a standing desk"
- **Show real data** — screenshots of YOUR actual stats (standing %, position changes)
- **Be honest about limitations** — "Windows only, USB only, ugly plexi mount"
- **Never be salesy** — the landing page does the selling; posts do the storytelling
- **Respond to EVERY comment** — early community building is 1:1

---

## 5. Pre-Sale Fulfillment Plan

### Timeline: Pre-order to Delivery (~3 months)

```
Month 1: Collect pre-orders + build landing page
Month 2: Source components, assemble kits, test each unit
Month 3: Ship + post-sale support
```

### Per-Unit Assembly Checklist

1. Order components from AliExpress/LCSC (bulk, ~2 weeks shipping)
2. Order plexi/PCB carriers (local laser cutting, ~1 week)
3. Flash firmware onto each MCU (batch process)
4. Solder/connect sensor to MCU (if not plug-and-play breakout)
5. Mount on plexi carrier
6. Test: plug into PC, verify distance reading
7. Package: bubble wrap + cardboard box
8. Ship (Poczta Polska / InPost for Poland, DHL/DPD for EU)

### Shipping Costs (estimate)

| Destination | Cost |
|-------------|------|
| Poland (InPost paczkomat) | ~12-15 PLN |
| EU (DHL/DPD standard) | ~30-50 PLN |
| Worldwide | ~50-80 PLN |

**Decision:** Include shipping in price for Poland. EU shipping: +10 EUR.

---

## 6. Success Criteria (3-month checkpoint)

| Question | Answer needed |
|----------|--------------|
| Do people buy it? | ≥10 pre-orders |
| Do they use it daily? | ≥50% daily active after 2 weeks |
| What do they ask for? | Collect feature requests |
| Do they recommend it? | ≥2 organic referrals |
| Would they pay for premium? | Survey after 1 month of use |
