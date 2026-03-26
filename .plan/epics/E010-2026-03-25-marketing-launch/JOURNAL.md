# E010 Journal — Marketing Launch

## Session 2026-03-25 / 2026-03-26

- **Goal**: Define full marketing/launch strategy for desk.zentala.io
- **Done**:
  - Business vision doc (certification, GTM, competitive moat)
  - 7 ADRs (ToF, USB-only, dev kit, open core, CE self-declaration, plexi mount, web kiosk)
  - Hardware options doc (MCU comparison, carrier PCB, optional sensors, BOM)
  - Validation strategy (landing page, pre-orders, marketing posts)
  - Premium tier definition (free vs Pro, 4 implementation epics)
  - Distribution & tiers (shipping, regional strategy)
  - Full marketing plan (landing page copy, 5 Reddit/HN posts, ads, SEO, social proof, conversion)
  - Story-driven launch (founder narrative, 3 tiers, mission, video/influencer strategy)
  - Growth tactics (share stats, PH, drip emails, referral, social wall)
  - Canonical pricing config (`config/pricing.json`)
  - CLAUDE.md refactored (520→220 lines, overlay+installer to rules/)
- **Decisions**:
  - 3 tiers: Basic €49 / Pro €79 / Founder's €149 (evolved from 2 tiers)
  - Per-tier thresholds: 200 / 500 / ships-with-Pro
  - Privacy-first analytics (Plausible, NOT Google Analytics)
  - Story-driven landing page (not product grid)
  - desk.zentala.io stays (no ergodesk.io domain purchase)
  - No MCU change yet (ESP32-C3 stays, RP2040 option for production)
- **Findings**:
  - Pricing was defined in 4 different places → created canonical config/pricing.json
  - marketing-launch-plan.md grew to 1059 lines → needs splitting (tracked as impro)
  - Memory files were stale (old 2-tier pricing) → needs update
- **Improvements logged**: 2 (pricing inconsistency → config/pricing.json, marketing plan too long → split needed)
- **Next**:
  - Execute Wave 1: landing page rebuild + collect real usage data
  - Split marketing-launch-plan.md (1059 lines) into 2-3 focused documents
  - Record 15-sec hero GIF/video
  - 22 tasks defined in ORCHESTRATOR, 0 completed yet
