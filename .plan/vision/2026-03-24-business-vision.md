# zntlDesk — Business Vision & Go-to-Market Strategy

**Date**: 2026-03-24
**Status**: Defined
**Author**: zentala (based on strategic analysis session)

---

## Executive Summary

zntlDesk is a **behavioral change platform** — not a hardware product. The sensor hardware
is the trigger; the real value is in software that changes how people work at their desks.

**Core positioning:** "Developer Platform + Reference Hardware"

**Revenue model:** Hardware as entry point → SaaS subscription as growth engine.

---

## 1. Product Definition

### What We Sell

We do NOT sell "a sensor for your desk."
We sell **behavior change** — awareness of sitting patterns, motivation to move, and
measurable improvement in work ergonomics.

**The sentence:** "Track your sitting and standing automatically. Build smarter work habits."

### Value Proposition

- Zero manual tracking — sensor detects position automatically
- Real-time feedback — overlay bar, tray icon, notifications
- Behavior change — gamification, coaching, streaks, recovery mechanics
- Data ownership — all data stays local (privacy-first)
- Extensible — open plugin system, community skins, developer SDK

---

## 2. Hardware Strategy

### Current Prototype

| Component | Part | Cost |
|-----------|------|------|
| Sensor | VL53L1X ToF (Grove breakout) | ~40-70 PLN |
| MCU | Seeed XIAO ESP32-C3 | ~30-50 PLN |
| Cables/connectors | USB-C | ~10-20 PLN |
| Mount | Laser-cut plexi + screws | ~10-30 PLN |
| **Total BOM** | | **~80-180 PLN** |

### Sensor Choice: ToF, Not "Laser"

VL53L1X uses a VCSEL (Vertical-Cavity Surface-Emitting Laser) at 940nm IR.
It IS technically a laser, but:
- Already classified **Class 1 (eye-safe)** by the module manufacturer (ST)
- Invisible IR beam, very low power, wide divergence
- Meets IEC 60825-1 at module level
- No optical modifications needed → no laser re-certification

**This is a solved problem.** The sensor module handles laser safety. We don't touch optics.

### MCU Decision: USB-Only vs WiFi/BLE

**Phase 1: USB-only (no radio)**
- ESP32-C3 used ONLY as USB serial bridge + I2C host
- WiFi and BLE radios are NOT activated
- This avoids RED (Radio Equipment Directive) entirely
- Saves 15-50k PLN in certification costs

**Future: WiFi/BLE for standalone mode**
- Phase 3 (see Roadmap) adds wireless communication
- Will require RED certification when we activate radio
- Deferred until consumer product stage (funded by Kickstarter/grants)

**Alternative MCU (under consideration):** A simpler MCU without WiFi/BLE
(e.g., ATmega328, RP2040, STM32) could further simplify compliance — no radio
hardware present at all means zero ambiguity about RED applicability.
Trade-off: ESP32-C3 is already working, switching MCU costs development time.

### Mount Design: Laser-Cut Plexi

- Semi-transparent acrylic sheet, laser-cut to fit sensor + MCU
- Components screwed onto plexi with standoffs
- **Intentionally looks like a prototype** — reinforces "dev kit" positioning
- No injection molding costs (those come at consumer product stage)
- Cost per unit: ~10-30 PLN
- Can be produced in batches of 10-50 with a local laser cutting service

---

## 3. Certification Strategy (EU CE Marking)

### What CE Requires

CE marking is mandatory for selling electronic products in the EU.
Even assembled from certified modules, the FINAL PRODUCT needs its own CE.

Applicable directives for our device:
- **EMC Directive (2014/30/EU)** — electromagnetic compatibility (ALWAYS required)
- **RoHS Directive (2011/65/EU)** — restricted substances (ALWAYS required)
- **Low Voltage Directive (2014/35/EU)** — only if >50V AC / 75V DC (NOT applicable — USB 5V)
- **RED (2014/53/EU)** — only if radio is active (NOT applicable in Phase 1 — USB only)

### Self-Declaration (Declaration of Conformity — DoC)

We do NOT need a "notified body" to issue a certificate.
We write our own Declaration of Conformity (DoC), stating:
- Product description and intended use
- Applicable directives and harmonized standards
- Our identity as manufacturer
- Test results demonstrating compliance

**We sign it. We take legal responsibility. We affix the CE mark.**

### Cost Estimate (Phase 1: USB-only, ToF sensor)

| Item | Cost (PLN) |
|------|-----------|
| EMC lab testing | 5,000 - 15,000 |
| Laser documentation (paperwork only, module is Class 1) | 0 - 3,000 |
| Documentation preparation | 0 - 3,000 |
| **Re-test budget** (EMC rarely passes first time) | 2,000 - 10,000 |
| **Total** | **~7,000 - 25,000 PLN** |

### EMC: The Real Challenge

EMC (Electromagnetic Compatibility) is the #1 cost and risk:
- Tests are done in anechoic chambers at certified labs
- USB cable acts as an antenna — biggest EMC failure cause
- PCB layout, cable routing, lack of shielding cause failures
- **First-time pass is rare** — budget for 2-3 iterations

Mitigation strategies:
- Use short, shielded USB cables (sell cable in kit or specify requirements)
- Keep wires between modules as short as possible
- Use ground planes on any custom PCB
- Pre-screen with near-field EMC probe before lab visit (~500 PLN investment)

### Dev Kit Exemption Strategy

Selling as a "Development Kit" / "Evaluation Board" provides partial regulatory relief:
- Target audience: developers and early adopters (not general consumers)
- No retail packaging, no "plug and play" UX
- Visible electronics (plexi mount helps)
- Technical documentation included
- Disclaimer: "For development and evaluation purposes"
- Label: "Development Kit" / "Evaluation Board" prominently displayed

**Important:** This is a gray area. If regulators determine the product is functionally
a consumer device, dev kit labeling won't protect you. The plexi mount, visible PCB,
and developer-focused marketing are all important signals.

**Evaluation Board** = a board meant for people evaluating whether to invest in /
develop with a technology. It implies: "this is a tool for building something,
not a finished product."

---

## 4. Software Strategy

### Open Core Model

**Open Source (MIT or Apache 2.0):**
- Desktop application (Tauri + React)
- UI components and layout system
- Plugin/skin system — community can create custom themes
- Device communication protocol (serial/USB)
- SDK for third-party integrations
- Basic tracking: sitting/standing detection, session timing, daily stats

**Why open source the app:**
- Removes incentive for competitors to write a new app — invest in our ecosystem instead
- Community contributions improve the product (skins, integrations, translations)
- Builds trust (privacy-first claim is verifiable)
- Dev kit buyers become contributors
- Hardware is simple to copy — software ecosystem is the moat

**Closed Source (proprietary):**
- Cloud backend (sync, history, analytics)
- AI/ML algorithms (behavior prediction, personalized coaching)
- Scoring/gamification engine (advanced version)
- Smartwatch integrations (Apple Watch, Garmin, Fitbit)
- Cross-device sync protocol
- Team/enterprise features

### Freemium Model

**Free tier (local, no account needed):**
- Full sitting/standing tracking
- Real-time overlay bar, tray icon, notifications
- Daily statistics and session history
- Basic gamification (daily score, streaks)
- Configurable limits and thresholds
- Plugin/skin support

**Pro tier (10-30 PLN/month):**
- Cloud sync — access history from any device
- Long-term trends (weekly/monthly/yearly analytics)
- AI coaching — personalized recommendations based on patterns
- Smartwatch integration (detect walking, HRV data)
- Calendar integration (auto-adjust goals based on meeting schedule)
- Advanced gamification (leagues, challenges, achievements)
- Priority support

**Enterprise tier (future, per-seat):**
- Team dashboards — manager sees aggregate team ergonomics
- Compliance reporting (workplace health regulations)
- Bulk hardware procurement
- Custom integration API

---

## 5. Go-to-Market Roadmap

### Phase 1: Dev Kit (NOW → 3-6 months)

**Goal:** Validate product-market fit. Find 20-50 paying users.

**Product:**
- Hardware: VL53L1X + ESP32-C3 (USB) + plexi mount
- Software: Open source desktop app (current Tauri app)
- Price: **299-399 PLN** per dev kit
- No CE yet (dev kit positioning)
- Sold without USB cable (user provides own)

**Channel:**
- Direct sales (landing page + Stripe/PayU)
- Developer communities (Reddit r/standingdesks, Hacker News, Polish dev forums)
- Open source community (GitHub stars → awareness → dev kit sales)

**Revenue target:** 50 units × 350 PLN = ~17,500 PLN
**Purpose:** Cover BOM costs, validate demand, collect user feedback

**What we learn:**
- Do people actually buy it?
- Do they use it daily?
- What features do they ask for?
- What breaks?

### Phase 2: SaaS Launch (6-12 months)

**Goal:** Recurring revenue from software subscriptions.

**Product:**
- Software Pro tier launched (cloud sync, AI coaching, smartwatch)
- Free tier remains fully functional (important — no bait-and-switch)
- Open source community growing (skins, plugins)

**Revenue model:**
- Pro: 20 PLN/month × 100 subscribers = 2,000 PLN/month
- Dev kit sales continue (improved version)

**Key milestones:**
- 100+ active daily users
- 30%+ free-to-pro conversion
- 3+ community-contributed skins/plugins

### Phase 3: Consumer Product (12-24 months)

**Goal:** Mass market product with full CE certification.

**Prerequisites:**
- Proven demand (Phase 1-2 data)
- Funding secured (see below)
- CE certification completed

**Product:**
- Designed enclosure (injection molded or CNC'd aluminum)
- Certified (CE: EMC + potentially RED if WiFi added)
- Retail packaging
- Plug-and-play UX (auto-detect, zero configuration)

**Pricing:**
- Consumer device: **300-600 PLN**
- Margin on hardware: thin (covers production + support)
- Real revenue: SaaS subscriptions from consumer user base

**Channel:**
- Own website
- Amazon EU
- Allegro (Poland)
- Retail partnerships (standing desk manufacturers — upsell bundle)

---

## 6. Funding Strategy

### Revenue Sequence (each stage funds the next)

```
Dev Kit Sales (17-35k PLN)
    → funds SaaS development
        → SaaS Revenue (24k+ PLN/year)
            → funds CE certification (7-25k PLN)
                → Kickstarter + Grants
                    → funds mass production
```

### Kickstarter Campaign (Month 12-18)

**When:** After Phase 2 proves demand (100+ active users, positive reviews)

**What we show:**
- Working product (demo videos from real users)
- User testimonials and data
- Professional render of consumer product
- Software demo (free + pro features)

**Target:** 50,000-150,000 PLN

**Use of funds:**
- Injection mold tooling (~15-30k PLN)
- CE certification (~7-25k PLN)
- First production batch (500-1000 units)
- Marketing

### EU Grants (Month 12-24)

**Applicable programs (Poland):**

| Program | Amount | What it funds |
|---------|--------|---------------|
| **PARP — Ścieżka SMART** | 50k-500k+ PLN | R&D, prototyping, market entry |
| **NCBR** | varies | B+R projects |
| **Bon na innowacje** | up to 50k PLN | Smaller innovation projects |
| **Regional funds (Mazowieckie)** | varies | Depends on voivodeship |

**Important:** Grants fund the PROJECT, not specifically "CE certification."
CE is a qualified cost within a broader product development / market entry project.

**Timeline reality:** Grant applications take months. Don't wait for grants to start.
Dev kit sales and Kickstarter are faster validation.

### Accelerator Option (Eastern Europe)

- Typical deal: **5-10% equity** for **20,000-100,000 EUR**
- Worth it IF they provide: network, distribution, hardware manufacturing contacts
- NOT worth it if they only offer: "logo and mentoring"
- Consider after Phase 1 proves demand — negotiate from strength

---

## 7. Data & Privacy Strategy

### Privacy-First Architecture

**All data stays local by default.** This is a core product principle, not just a feature.

- Desktop app stores everything in local SQLite (already implemented)
- No account required for free tier
- No telemetry, no tracking, no ads
- GDPR/RODO compliant by design — no personal data leaves the device

### Cloud Sync (Pro feature, opt-in)

- User explicitly chooses to sync to cloud
- Data encrypted in transit and at rest
- User can delete all cloud data at any time
- Data portability: export everything as JSON/CSV

### Aggregate Data (future, with explicit consent)

**NOT a business model.** Do not plan to "monetize the data."

However, anonymized aggregate data could enable:
- "Average standing % for developers in Poland: 14%"
- Research partnerships with ergonomics researchers
- Benchmark: "You stand more than 73% of users"

This requires:
- Explicit opt-in (not buried in ToS)
- True anonymization (not just pseudonymization)
- RODO compliance review
- Transparency about what data is collected and why

**Bottom line:** Data is a nice-to-have for product features (benchmarks),
NOT a revenue stream. Revenue comes from subscriptions.

---

## 8. Competitive Moat

### Why Not Just Copy This?

The hardware IS easy to copy. A VL53L1X + any MCU = working sensor.
The software app is open source. Anyone can fork it.

**Our moat is the ecosystem:**

1. **Community** — skins, plugins, integrations built by developers
2. **Cloud features** — history, AI coaching, smartwatch sync (closed source)
3. **Brand** — first mover in "desk height tracking for developers"
4. **Data network effect** — more users = better benchmarks = more value per user
5. **Hardware iteration** — we ship consumer product while clones are still DIY
6. **Partnerships** — standing desk manufacturers bundle our sensor (exclusive deals)

### Affiliate Revenue (Supplementary)

- Standing desk mats, anti-fatigue mats
- Balance boards, kneeling chairs (klęcznik)
- Walking pads
- Standing desk recommendations

This is supplementary income, not the core business. Works best once we have
a large user base who trusts our recommendations.

---

## 9. Long-Term Vision (2-5 years)

### Beyond the Desk

The desk sensor is the wedge. The platform expands to:

1. **Smartwatch integration** — detect walking, stairs, HRV, sleep quality
2. **Multiple positions** — sitting, standing, leaning, perching, kneeling, walking pad
3. **Meeting awareness** — auto-detect meeting times, adjust goals
4. **Team health** — enterprise dashboard for workplace wellness
5. **Health API** — integrate with Apple Health, Google Fit, Garmin Connect
6. **Research platform** — partner with universities studying workplace ergonomics

### Hardware Evolution

1. **Dev Kit** (now) — plexi + modules + USB
2. **Consumer v1** (12-24mo) — designed enclosure + USB
3. **Consumer v2** (24-36mo) — wireless (BLE/WiFi) + battery/USB-C power
4. **Consumer v3** (36-48mo) — multi-sensor (height + presence + ambient light)

### Business Model Evolution

```
Phase 1: Dev Kit sales (hardware margin)
Phase 2: SaaS subscriptions (recurring revenue)
Phase 3: Consumer hardware + SaaS (volume)
Phase 4: Enterprise + API licensing (high margin)
Phase 5: Affiliate + partnerships (passive income)
```

---

## 10. Key Metrics to Track

| Metric | Phase 1 Target | Phase 2 Target |
|--------|---------------|---------------|
| Dev kits sold | 50 | 200 |
| Daily active users | 20 | 100+ |
| Free-to-Pro conversion | n/a | 30%+ |
| Community plugins/skins | 0 | 3+ |
| GitHub stars | 50 | 500 |
| Monthly recurring revenue | 0 | 2,000+ PLN |
| NPS | n/a | 40+ |

---

## 11. Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|-----------|
| No one buys dev kit | Fatal | Validate with landing page + waitlist before production |
| EMC fails repeatedly | Delays + cost | Pre-screen with near-field probe, use shielded cables |
| Regulator classifies dev kit as consumer product | Legal | Maintain clear dev kit positioning, technical docs, disclaimer |
| Competitor launches similar product | Medium | Speed to market, community ecosystem, SaaS features |
| Open source fork gains traction | Low | Closed cloud features, brand, community relationships |
| Standing desks fall out of trend | Low | Platform is about "movement" not "standing" — position agnostic |

---

## Appendix: Key Terminology

- **ToF** — Time of Flight (sensor technology, measures distance via light pulse timing)
- **VCSEL** — Vertical-Cavity Surface-Emitting Laser (the laser type in VL53L1X)
- **CE** — Conformité Européenne (EU product safety marking)
- **EMC** — Electromagnetic Compatibility (device doesn't interfere with others)
- **RED** — Radio Equipment Directive (EU directive for radio-transmitting devices)
- **DoC** — Declaration of Conformity (self-declaration document for CE)
- **RoHS** — Restriction of Hazardous Substances (EU directive)
- **BOM** — Bill of Materials (list of all components and their costs)
- **PARP** — Polska Agencja Rozwoju Przedsiębiorczości
- **NCBR** — Narodowe Centrum Badań i Rozwoju
