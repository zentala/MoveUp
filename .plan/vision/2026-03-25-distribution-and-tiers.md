# Distribution Strategy & Product Tiers

**Date**: 2026-03-25
**Status**: PARTIALLY SUPERSEDED — tier PRICING moved to `config/pricing.json` (canonical).
Tier descriptions and Founder's Edition here are outdated (was 2 tiers, now 3).
**Still valid**: Shipping costs, distribution phases, regional strategy (deferred), fulfillment.
**See**: `story-driven-launch.md` for current 3-tier structure, `config/pricing.json` for prices.

---

## 1. Product Tiers

### Tier A: DIY (Free)

- BOM list + assembly guide + firmware + app — all on GitHub
- User sources components themselves (~10-15 EUR from AliExpress)
- User flashes firmware, assembles, installs app
- Effort: 5-10 hours for someone comfortable with Arduino
- **Purpose:** Proves open source commitment, creates content, builds community
- **Revenue:** Zero (but converts to app users → potential future Pro subscribers)

### Tier B: Dev Kit (49 EUR)

- Assembled sensor + MCU + mount (plexi or PCB carrier)
- Pre-flashed firmware, ready to plug in
- USB cable included
- Desktop app (open source, free)
- Discord community access
- **Purpose:** Primary validation product. Low-commitment entry point.
- **Revenue:** ~20-30 EUR margin per unit after BOM + shipping (EU)

### Tier C: Founder's Edition (99 EUR)

Everything in Dev Kit, PLUS:
- **Priority feature requests** — your needs influence the roadmap
- **Direct support channel** — private Discord channel or email thread with zentala
- **6 months Pro subscription** (when Pro launches) — cloud sync, history, AI coaching
- **Name in credits** — listed as founding supporter in app About page and website
- **Early access** — beta builds of new features before public release

**Why 99 EUR:**
- ~50 EUR margin after BOM + shipping
- Attracts people who WANT to invest in the project, not just buy hardware
- Self-selects for engaged users who will give quality feedback
- 20 Founder's Edition × 99 EUR = 1,980 EUR — meaningful for a solo dev
- Psychological: "I'm backing a project" not "I'm buying a gadget"

**No limit** — we WANT as many Founder's Edition buyers as possible. They are the
most engaged users, the best source of feedback, and the highest margin tier.

**Pro subscription:** 5 years included (effectively lifetime for early product stage).
Cost to us: near zero (cloud infrastructure cost per user is negligible — Cloudflare
Workers free tier handles thousands of requests, D1 free tier is 5GB).

### Tier Comparison on Landing Page

```
┌──────────────┬──────────────┬──────────────────────┐
│   DIY        │   Dev Kit    │   Founder's Edition  │
│   Free       │   49 EUR     │   99 EUR (50 only)   │
├──────────────┼──────────────┼──────────────────────┤
│ BOM list     │ ✅ Assembled │ ✅ Assembled          │
│ Firmware     │ ✅ Flashed   │ ✅ Flashed            │
│ Assembly     │ ✅ Mount     │ ✅ Mount               │
│ guide        │ ✅ USB cable │ ✅ USB cable           │
│              │ ✅ App       │ ✅ App                 │
│              │ ✅ Discord   │ ✅ Discord             │
│              │              │ ✅ Priority features   │
│              │              │ ✅ Direct support      │
│              │              │ ✅ 5 years Pro free    │
│              │              │ ✅ Name in credits     │
│              │              │ ✅ Beta access         │
└──────────────┴──────────────┴──────────────────────┘
```

---

## 2. Distribution: Phase 1 (NOW — EU Only)

### Why EU only at start:
- zentala lives in Poland — assembly + shipping is local
- EU single market — no customs between countries
- InPost/DPD/DHL — cheap, reliable, tracked
- Active company in Poland — legal invoicing ready
- CE dev kit positioning works within EU regulatory framework

### Shipping costs (from Poland):

| Destination | Carrier | Cost | Delivery |
|-------------|---------|------|----------|
| Poland | InPost Paczkomat | ~12-15 PLN | 1-2 days |
| EU (nearby: DE, CZ, SK) | DPD/DHL | ~30-40 PLN | 2-4 days |
| EU (far: ES, PT, IT, SE) | DPD/DHL | ~40-60 PLN | 3-6 days |
| UK (post-Brexit) | DHL Express | ~60-80 PLN | 5-7 days + customs |

### Pricing with shipping:

| Market | Dev Kit | Founder's | Shipping | Total (Dev Kit) |
|--------|---------|-----------|----------|-----------------|
| Poland | 49 EUR | 99 EUR | Free | 49 EUR |
| EU | 49 EUR | 99 EUR | +8 EUR | 57 EUR |
| UK | 49 EUR | 99 EUR | +15 EUR | 64 EUR |

**Decision:** Free shipping for Poland. Flat 8 EUR for EU. UK: +15 EUR.
Include shipping in checkout (no surprise costs at payment).

### Assembly (Phase 1):
- zentala assembles manually in Poland
- Capacity: ~5-10 units per evening
- Batch: order 100+ components from AliExpress, assemble over 2-3 weeks
- Quality check: plug each unit into PC, verify distance reading

---

## 3. Distribution: Phase 2 (FUTURE — When Demand Exists Outside EU)

> ⚠️ DEFERRED. Do not plan this until: (a) EU demand is proven, (b) international
> pre-orders accumulate organically, (c) at least 50 units shipped in EU.

### Concept: Regional Assembly Partners

If significant demand appears outside EU, the model would be:
- Ship components in bulk to a regional partner
- Partner assembles, tests, ships locally
- Partner earns per-unit fee (~10-15 EUR/unit)
- Avoids international shipping costs for end customers

### Potential regions (ordered by likely demand):

| Region | Assembly location | Ships to | Threshold to activate |
|--------|-------------------|----------|----------------------|
| **EU** | Poland (zentala) | EU + UK | Active now |
| **North America** | US (partner TBD) | US + Canada | 50 pre-orders from NA |
| **East Asia** | South Korea or Taiwan | JP, KR, TW, SG, HK | 30 pre-orders from Asia |
| **Oceania** | Australia (partner TBD) | AU + NZ | 20 pre-orders from AU/NZ |
| **South America** | Skip for now | — | Not worth the complexity |

### Why NOT China for assembly:
- IP risk (copying the product, selling direct)
- For simple assembly (2 modules + mount), a hobbyist partner is better than a factory
- Component sourcing FROM China is fine; assembly IN China adds risk without benefit

### Partner profile:
- Maker/hobbyist who can solder and test
- Reliable (ships on time)
- Pays for local shipping, reimbursed per-unit
- Found via: maker communities, Reddit, Discord

### Regulatory notes per region:
- **US:** FCC (similar to CE EMC). Dev kit may have same exemption as EU.
- **Australia:** RCM mark. Dev kit positioning likely helps.
- **Japan:** TELEC/PSE. More strict. Research needed before entering.
- **Canada:** ISED. Similar to FCC.

> All of this is FUTURE planning. Do NOT invest time in this until EU is working.

---

## 4. Pre-Order Threshold Model

### Phase 1: Single global threshold

**100 pre-orders total → start production and shipping**

NOT per-market. Reasons:
- At this stage, 100 total is ambitious enough
- International orders can be shipped from Poland (expensive but feasible)
- Regional hubs only make sense at 200+ orders per region
- Simplicity: one counter on the landing page, one goal

### Phase 2 (future): Per-market thresholds

When total orders exceed 200-300 and non-EU orders are >30%:
- Split into EU + NA + Asia thresholds
- Each market activates independently
- Counter on landing page shows per-region progress

### Landing page counter:

```
╔══════════════════════════════════════════╗
║  🎯 37 / 100 pre-orders                 ║
║  ████████████░░░░░░░░░░░░░░░░░░░  37%   ║
║                                          ║
║  When we hit 100, production starts.     ║
║  Estimated delivery: ~3 months after.    ║
║                                          ║
║  [Pre-order Dev Kit — 49 EUR]            ║
║  [Pre-order Founder's Edition — 99 EUR]  ║
║                                          ║
║  Not ready to buy? [Join waitlist →]     ║
║  (142 people watching)                   ║
╚══════════════════════════════════════════╝
```

---

## 5. International Shipping (Phase 1 — from Poland)

For non-EU orders that come in organically (before regional hubs):

| Destination | Carrier | Cost | Delivery |
|-------------|---------|------|----------|
| US/Canada | Poczta Polska priority | ~60-80 PLN (~15-20 EUR) | 7-14 days |
| Australia | Poczta Polska priority | ~70-90 PLN (~17-22 EUR) | 10-21 days |
| Asia (JP/KR/SG) | Poczta Polska priority | ~60-80 PLN (~15-20 EUR) | 7-14 days |

**Decision for international:** Customer pays actual shipping cost.
Show on checkout: "Shipping to US: +18 EUR"

### "Suitcase shipping" (zentala's idea)

When traveling or when friends/contacts travel:
- Bring batch of packaged units in checked luggage
- Ship locally from destination country (domestic rates)
- Saves ~10-15 EUR per package vs international post
- Informal, doesn't scale, but works for 10-20 units

**Note:** This is a clever hack for Phase 1 but NOT a distribution strategy.
Document it as a cost optimization, not a process.

---

## 6. Decisions NOT Made Yet

These need answers before implementation but can wait until validation:

1. **VAT handling for EU cross-border sales** — OSS (One-Stop Shop) simplifies VAT
   for EU sales. Research needed. zentala has active Polish company.
2. **Customs declarations for non-EU** — small electronics, low value (<150 EUR).
   Usually simple. Check per-destination.
3. **Return policy** — full refund before shipping; after shipping, replacement
   for defective units. Define in T&C.
4. **Warranty** — EU consumer law requires 2 years. Dev kit positioning may
   reduce this obligation. Legal review needed before consumer sales.
5. **Payment processor** — Stripe is recommended (handles EU VAT, supports
   pre-orders, good for small volumes). Alternative: PayU (popular in Poland).
