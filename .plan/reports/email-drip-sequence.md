# Email Drip Sequence: Waitlist to Pre-Order Conversion

*Created: 2026-03-26 | Epic: E010 | Task: T19*
*Sender: zentala (personal, not "team")*
*Trigger: user joins waitlist but has not pre-ordered*
*Sequence: 5 emails over 21 days*

---

## Email 1: Welcome (Immediately after signup)

**Subject A:** You're on the list. Here's what I'm building.
**Subject B:** Welcome to zntlDesk — let me show you why I built this
**Preview text:** A sensor, an app, and a mission to fix how we sit.

---

Hey,

I'm zentala — a programmer who sits 10+ hours a day. Two years ago I bought a standing desk thinking it would fix everything. It didn't.

So I built zntlDesk: a tiny sensor that mounts under your desk and an open-source app that tracks your sit/stand patterns automatically. No buttons, no manual logging. It just knows.

Right now we're at [PLACEHOLDER — XX/200] pre-orders for the first production batch. When we hit 200, manufacturing starts.

The whole thing is open source — you can check out the code on [GitHub](https://github.com/zentala/zntl-tray) and see exactly what you're getting.

If you already know this is for you, skip the wait.

---

**CTA button text:** Pre-order now — from EUR 49
**CTA link:** https://desk.zentala.io/#pricing
**Unsubscribe:** You signed up for the zntlDesk waitlist. [Unsubscribe]({{unsubscribe_url}}) anytime — no hard feelings.

---


## Email 2: The Problem (Day 3)

**Subject A:** You probably stand less than you think
**Subject B:** 50% of standing desk owners barely use them
**Preview text:** The data on sitting is worse than you'd expect.

---

Hey,

Here's an uncomfortable fact: only 50% of people who own standing desks use them regularly. Average standing time? Just 36%.

The reason is simple — we forget. When you're deep in code or a doc, you don't notice time passing. Hours go by. Your desk stays down.

Meanwhile, the health data keeps piling up. Sitting 8+ hours a day increases cardiovascular mortality risk comparable to smoking (Lancet, 2016). Diabetes risk jumps 112%. And 80% of office workers report musculoskeletal disorders — neck, back, shoulders.

Standing desks were supposed to fix this. But a desk that can go up doesn't mean it will go up.

That's why I built zntlDesk. Not another standing desk accessory — a behavior change tool that makes the invisible visible and nudges you when it matters.

---

**CTA button text:** See how it works
**CTA link:** https://desk.zentala.io/#how-it-works
**Unsubscribe:** [Unsubscribe]({{unsubscribe_url}}) — no questions asked.

---


## Email 3: The Data (Day 7)

**Subject A:** 8% to 22% — what happened when I tracked everything
**Subject B:** My standing desk data after 3 months of tracking
**Preview text:** The sensor made invisible behavior visible.

---

Hey,

Before I built zntlDesk, I had no idea how bad my habits were. I thought I stood "a decent amount." The sensor told a different story.

**Before tracking:**
- Standing time: [PLACEHOLDER — 8%] of my workday
- Position changes: [PLACEHOLDER — 1.2 per day]
- Longest sitting streak: [PLACEHOLDER — 4+ hours without noticing]

**After 3 months with zntlDesk:**
- Standing time: [PLACEHOLDER — 22%]
- Position changes: [PLACEHOLDER — 4.8 per day]
- Back pain: [PLACEHOLDER — gone]

The app didn't force me to do anything. It just showed me reality — a progress bar at the top of my screen, gentle nudges, a simple score. Knowing the number changed the number.

Research backs this up: gentle reminders increase standing time by 117% (iMovR). zntlDesk automates those reminders based on real data, not timers.

---

**CTA button text:** Pre-order the kit — EUR 49
**CTA link:** https://desk.zentala.io/#pricing
**Unsubscribe:** [Unsubscribe]({{unsubscribe_url}})

---


## Email 4: Social Proof + Urgency (Day 14)

**Subject A:** [PLACEHOLDER — XX] people are in. Are you?
**Subject B:** Production starts at 200 pre-orders — here's where we are
**Preview text:** The first batch is filling up.

---

Hey,

Quick update: we're at [PLACEHOLDER — XX/200] pre-orders for the first batch.

The community is growing too — [PLACEHOLDER — XX] members on Discord, [PLACEHOLDER — XX] GitHub stars. People are building on it, suggesting features, sharing their tracking data.

Here's what one early tester said:

> "[PLACEHOLDER — real testimonial from early user about their experience with the sensor and app]"
> — [PLACEHOLDER — Name, role]

Production starts the moment we hit 200. First batch ships from Poland, EU-wide delivery in 2-6 days. Every kit ships with a pre-flashed sensor, USB cable, mount, and full access to the open-source app.

If you've been thinking about it — now is the time. The first batch always ships fastest.

---

**CTA button text:** Secure your kit
**CTA link:** https://desk.zentala.io/#pricing
**Unsubscribe:** [Unsubscribe]({{unsubscribe_url}})

---


## Email 5: Last Chance / Founder's Edition (Day 21)

**Subject A:** The Founder's Edition — for those who want to build this with me
**Subject B:** Last call: 3 ways to join zntlDesk
**Preview text:** Basic, Pro, or Founder's — pick your level.

---

Hey,

Three weeks ago you signed up because something about this resonated. Let me recap what zntlDesk is about:

**The problem:** Standing desks don't change behavior. 50% of owners barely use them.

**The solution:** A tiny sensor under your desk + an open-source app that tracks sit/stand automatically and nudges you to move.

**The results:** [PLACEHOLDER — XX%] more standing time, [PLACEHOLDER — XX]x more position changes. No willpower needed — just awareness.

Today I want to tell you about the **Founder's Edition** (EUR 149). This is for people who don't just want the product — they want to shape it:

- Everything in the Pro Kit (sensor + haptic alerts + presence detection)
- 5 years of Pro cloud subscription (sync, history, AI coaching)
- Smartwatch integration when available
- Priority feature requests and direct support
- Your name in the credits

There are three tiers. Pick whatever fits:

| | Basic | Pro | Founder's |
|---|---|---|---|
| **Price** | EUR 49 | EUR 79 | EUR 149 |
| **Sensor** | Yes | Yes | Yes |
| **Haptic alerts** | — | Yes | Yes |
| **5yr Pro cloud** | — | — | Yes |
| **Name in credits** | — | — | Yes |

This is your chance to be part of something from the very beginning.

---

**CTA button text:** Pre-order now
**CTA link:** https://desk.zentala.io/#pricing
**Unsubscribe:** [Unsubscribe]({{unsubscribe_url}})

---


## Implementation Notes

### Sending platform
TBD — candidates: Buttondown (indie-friendly), Resend + React Email, Loops.

### Metrics to track per email
- Open rate (Subject A vs B)
- Click-through rate (CTA clicks)
- Conversion rate (pre-orders within 48h of open)
- Unsubscribe rate

### Placeholder checklist
Before going live, replace all `[PLACEHOLDER]` values:
- [ ] Pre-order counter (Emails 1, 4)
- [ ] Personal tracking data — before/after (Email 3)
- [ ] Community stats — Discord members, GitHub stars (Email 4)
- [ ] Early user testimonial + name (Email 4)
- [ ] Results summary numbers (Email 5)

### A/B testing plan
- Test subject lines (A vs B) on first 100 recipients, send winner to rest
- Track which email in the sequence drives most conversions
- Consider testing send timing (morning vs evening) in second cohort
