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

## Session 2026-03-27 01:00

- **Goal**: Execute E010 Marketing Launch — all 5 waves
- **Done** (19/22 tasks completed):
  - **Wave 1**: Landing page rebuilt as founder story (desk.zentala.io), sedentary research report (19 sources)
  - **Wave 2**: Stripe checkout UI, waitlist forms, Plausible analytics, SEO (schema.org, OG), conversion optimization (sticky CTA, exit popup), 5-email drip sequence
  - **Wave 3**: 5 Reddit/HN post drafts, social proof + comparison table, blog infra + 2 SEO articles
  - **Wave 4**: zentala.agency case study, ads strategy doc, influencer outreach (18 channels), Product Hunt launch plan, referral program, social mention wall
  - **Wave 5**: Telemetry opt-in (Rust + React), Share My Stats (clipboard + Twitter/Reddit)
  - **Code review**: 15 findings across 2 repos, all fixed (error handling, DRY pricing, unwrap→Result, telemetry race condition, SEO, a11y)
  - **Bug fixes**: Overlay Live default (3rd time — fixed script + fallback + docs), timezone DB query bug (date_local column)
- **Commits** (desk app): 0602535, 451cf0a, cc3a18e, 2b69254, 87b1c58, c50e29b, 7ace0d1, a9b0ac6, 94e6771, d7dad76
- **Commits** (desk.zentala.io): a1f516c, c201860, 58a5252, 48b2fd3, ceca044, 8a76020, 66ce6a7, 9b06dd6, 077d86f, 5bef278, 29a7595
- **Commits** (zentala.agency): 05f8116
- **Decisions**: Plausible for analytics (not GA), date_local column for timezone-safe DB queries
- **Findings this session**: 3
  1. Overlay Demo default embedded in 3 places (script, fallback, docs) — all fixed
  2. Timezone bug: UTC timestamps vs local date queries → data loss on restart near midnight
  3. Telemetry sent after daily reset → reports zeros (fixed: send before reset)
- **Improvements logged**: 15 (from code review — all addressed)
- **Next**:
  1. User tasks: T02 (photos/screenshots), T14 (marketing videos), T18 (hero GIF)
  2. Replace placeholder Stripe URLs with real Payment Links
  3. Deploy desk.zentala.io (push to main)
  4. Deploy zentala.agency case study
  5. Build Cloudflare Worker for waitlist endpoint
  6. Create OG image (1200×630px)
  7. Replace all [PLACEHOLDER] data in posts/emails with real usage stats
