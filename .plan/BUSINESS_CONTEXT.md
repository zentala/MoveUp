# Business Context — SmartDesk

**Every agent planning features, epics, or strategy MUST read this file first.**
This is the accumulated business knowledge — research, strategy docs, and reports
that inform all product decisions.

## Product Positioning

"Developer Platform + Reference Hardware" — software is the business, hardware is the entry point.
Open core model: app is open source (MIT/Apache), cloud/AI/smartwatch are closed/paid.

## Vision & Strategy Documents

| Document | What it covers | When to read |
|----------|---------------|-------------|
| [Product Vision](.plan/vision/2026-03-15-desk-app-vision.md) | Hardware, states, UI, session logic, remote display phases | Any feature planning |
| [Business Vision & GTM](vision/2026-03-24-business-vision.md) | Dev kit → SaaS → consumer product; pricing; open core; certification CE; funding | Business decisions, pricing |
| [Validation & Marketing](vision/2026-03-25-validation-and-gtm.md) | Landing page; pre-order + waitlist; marketing posts; validation thresholds | Marketing, launch planning |
| [Premium Tier Definition](vision/2026-03-25-premium-tier-definition.md) | Free vs Pro features; Pro = cloud sync, history, AI coaching, smartwatch | Feature scoping, monetization |
| [Distribution & Tiers](vision/2026-03-25-distribution-and-tiers.md) | 3 tiers (DIY/Dev Kit/Founder's); EU-first; pre-order thresholds | Pricing, distribution |
| [Marketing Launch Plan](vision/2026-03-25-marketing-launch-plan.md) | Reddit/HN posts; Google/FB ads; KPI dashboard; content calendar; SEO | Marketing execution |
| [Story-Driven Launch](vision/2026-03-26-story-driven-launch.md) | Founder story landing page; 3 product tiers; viral video; influencer outreach | Landing page, content |

## Research Reports

| Report | What it covers | When to read |
|--------|---------------|-------------|
| [Sedentary Lifestyle Research](reports/sedentary-lifestyle-research.md) | 19 cited sources on health risks of sitting; used in landing page copy and marketing posts | Landing page, content, health claims |
| [Launch Posts](reports/launch-posts.md) | 5 ready-to-post Reddit/HN posts with prepared responses to common objections | Community launch, marketing |
| [Email Drip Sequence](reports/email-drip-sequence.md) | 5-email waitlist conversion funnel (welcome → story → social proof → urgency → last chance) | Email marketing, waitlist |
| [Ads Strategy](reports/ads-strategy.md) | Google + Facebook ads plan (500 PLN/mo budget); target keywords; audience segments | Paid marketing |
| [Influencer Outreach](reports/influencer-outreach.md) | 18 target YouTube channels with subscriber counts, contact info, pitch angles | Influencer marketing |
| [Product Hunt Launch](reports/product-hunt-launch.md) | PH listing draft (tagline, description, maker comment); launch day strategy | Product Hunt launch |
| [Waitlist Worker Spec](reports/waitlist-worker-spec.md) | Cloudflare Worker for waitlist endpoint — API design, D1 schema, rate limiting | Backend implementation |

## Pricing (canonical source of truth)

**File: [`.plan/vision/config/pricing.json`](vision/config/pricing.json)**

All landing pages, docs, and tasks MUST read pricing from this file.
Do NOT hardcode prices — they may change or be A/B tested.

## Architecture Decision Records

| ADR | Decision | When relevant |
|-----|----------|--------------|
| [001](.arch/ADR/001-remote-display-web-kiosk.md) | Remote display via embedded HTTP+WS server | Remote display features |
| [002](.arch/ADR/002-tof-sensor-over-laser.md) | VL53L1X ToF module — Class 1 eye-safe | Hardware decisions |
| [003](.arch/ADR/003-usb-only-no-radio-phase1.md) | USB-only in Phase 1 — avoids RED directive | Hardware, certification |
| [004](.arch/ADR/004-dev-kit-before-consumer-product.md) | Dev Kit before consumer product | Product strategy |
| [005](.arch/ADR/005-open-core-software-model.md) | Open core — app open source, cloud/AI closed | Monetization |
| [006](.arch/ADR/006-self-declaration-ce-not-notified-body.md) | CE self-declaration (not notified body) | Certification |
| [007](.arch/ADR/007-plexi-mount-dev-kit-enclosure.md) | Plexi/PCB carrier mount for dev kit | Hardware design |
| [008](.arch/ADR/008-proportional-break-credit.md) | Proportional break credit (configurable multiplier) | Session logic, scoring |

## Hardware Design

See [`.arch/hardware/HARDWARE-OPTIONS.md`](.arch/hardware/HARDWARE-OPTIONS.md) for:
- MCU comparison, carrier PCB design, optional components, BOM, production timeline

## Design Specs

| Spec | What it covers |
|------|---------------|
| [Communication Architecture](../docs/superpowers/specs/2026-03-30-communication-architecture-design.md) | Profile system, color dictionary, signal matrix, CommunicationPolicy |
