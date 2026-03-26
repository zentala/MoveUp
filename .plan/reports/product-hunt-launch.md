# Product Hunt Launch Plan

**Date**: 2026-03-26
**Epic**: E010
**Task**: E010-T20

---

## Product Hunt Listing Draft

### Tagline Options (max 60 chars)

1. "Know how much you actually use your standing desk" (51 chars)
2. "A tiny sensor that tracks your real sit/stand time" (51 chars)
3. "Open source desk sensor — stop guessing, start measuring" (57 chars)

**Recommended**: Option 1 — speaks to the gap between perception and reality.

### Description Options (max 260 chars)

1. "Mount a EUR 12 sensor under your desk. See a real-time progress bar that turns red when you've sat too long. Open source app, local data, no cloud. Most standing desk owners sit 85% of the day. Now you'll know — and change it." (226 chars)

2. "A ToF sensor + open source desktop app that tracks your actual sit/stand time. Real-time overlay bar, session history, break nudges. No cloud, no subscription. Built with Rust + Tauri. Pre-order the dev kit from EUR 49." (220 chars)

**Recommended**: Option 1 — leads with the problem, ends with the promise.

### Topics

- Productivity
- Health
- Open Source
- Hardware
- Developer Tools

### First Comment (by zentala — maker)

```
Hey Product Hunt!

I'm Pawel, a developer from Poland. I bought a motorized standing desk
two years ago, convinced it would fix my sedentary habits.

Spoiler: it didn't.

I realized I had no idea how much I was actually standing. My gut said
"plenty." So I stuck a cheap time-of-flight sensor under my desk and
wrote a quick script to log the data. First day's result: 87% sitting.
On a standing desk. Embarrassing.

That script turned into a full desktop app. A tiny overlay bar at the
top of your screen shows your sitting session progress — green fading
to red over 40 minutes. When you stand up, the bar disappears. When
you sit back down, it starts fresh. Simple, visual, impossible to ignore.

What makes this different from Pomodoro apps or generic timers:
- It measures REALITY, not intentions. The sensor knows if you're
  sitting or standing — you can't cheat.
- It's ambient. No manual tracking, no buttons. Just a bar that's
  always there.
- It's completely local. No cloud, no account, no data leaving your
  machine. The app is open source (MIT).
- The hardware is dirt cheap — EUR 12 in parts if you DIY, or
  pre-order an assembled kit from EUR 49.

The tech: VL53L1X time-of-flight sensor on an ESP32-C3 microcontroller,
connected via USB. Desktop app built with Rust (Tauri) + React. Works
on Windows today, macOS/Linux planned.

I built this for myself, but after sharing it on Reddit and HN, turns
out a lot of standing desk owners have the same blind spot.

Would love your feedback. Happy to answer any questions about the
hardware, the app architecture, or the embarrassing data from my
first week.

— Pawel (@zentala)
```

### Gallery Images (5 required)

| # | Image | Description | What to create |
|---|-------|-------------|---------------|
| 1 | Hero image | Sensor mounted under desk + laptop showing app | Photo: sensor on PCB with mounting tape under desk edge, laptop in background with overlay bar visible. Clean, well-lit desk setup. |
| 2 | Overlay bar screenshot | Progress bar across top of screen | Screenshot: Windows desktop with overlay bar at ~70% (orange/red gradient). Show it's subtle but visible during normal work. |
| 3 | KPI dashboard | Popup window with session stats | Screenshot: floating window showing today's sitting/standing time, session history timeline, current state. Dark theme. |
| 4 | Sensor closeup | Hardware mounted under desk | Photo: close-up of VL53L1X + ESP32-C3 carrier PCB mounted with tape under desk. Show how small and clean the install is. Include a coin for scale. |
| 5 | Before/after comparison | Data improvement over time | Mockup: side-by-side daily timeline. Left (Week 1): mostly red blocks (sitting). Right (Week 3): mixed green and red. Caption: "Same desk. Same person. Just add data." |

**Image specs**: 1270x760px recommended. PNG or JPG. No text-heavy images (PH guidelines).

---

## Launch Strategy

### Best Day and Time

- **Day**: Wednesday (highest traffic) or Tuesday (less competition)
- **Time**: 12:01 AM PT (00:01 Pacific) — products posted at midnight get a full 24h cycle
- **Why not Thursday**: too much competition from established products
- **Why not Monday/Friday**: lower traffic, weekend effect

**Recommended**: Tuesday at 00:01 PT — full cycle with moderate competition.

### Getting Initial Upvotes (First 2 Hours Are Critical)

**Organic sources** (no vote-buying — PH detects and penalizes this):

| Source | Action | Expected upvotes |
|--------|--------|-----------------|
| Reddit communities | Post "We just launched on PH" in r/standingdesks, r/quantifiedself, r/selfhosted (where we've already been active) | 10-20 |
| Hacker News | "Show HN" post linking to PH page | 5-15 |
| Email list | Send launch email to pre-order waitlist | 5-10 |
| Twitter/X | Post launch thread, tag relevant accounts | 3-8 |
| Personal network | DM friends/colleagues who use PH | 10-15 |
| Influencer kit recipients | Ask creators who received kits to check it out | 2-5 |
| Maker communities | Post in Maker/Indie Hacker communities (IH, WIP) | 3-5 |

**Target**: 40-80 upvotes in first 4 hours to hit the front page.

### Pre-Launch Teaser Strategy (1 week before)

| Day | Action |
|-----|--------|
| D-7 | Create PH "upcoming" page. Collect followers. |
| D-5 | Twitter thread: "Next week I'm launching the stupidest simple gadget that changed my desk habits" |
| D-3 | Reddit post in r/standingdesks: "Teaser — launching something next week for standing desk owners" |
| D-2 | Email waitlist: "We're launching on Product Hunt Tuesday — here's how you can help" |
| D-1 | Final Twitter/social reminder with PH upcoming page link |
| D-0 | Launch at 00:01 PT. Immediately notify all channels. |

### Comment Engagement Strategy

**Respond to every comment within 30 minutes during the first 12 hours.** PH rewards active makers.

#### Templates for Common Questions

**"Does it work on Mac/Linux?"**
> "Windows only right now — macOS is the next priority. The app is built with Tauri (Rust + React), which is cross-platform by design, so the port should be straightforward. Linux after that. If you're interested in a specific platform, let me know — it helps me prioritize."

**"Why not just use a Pomodoro timer?"**
> "Great question — I tried that! The difference is measurement vs intention. A Pomodoro timer reminds you to stand, but doesn't know if you actually did. This sensor measures reality. Turns out, knowing the truth is way more motivating than setting reminders."

**"Why not Bluetooth?"**
> "USB-only for now, intentionally. Bluetooth adds pairing complexity, battery management, and would require radio certification (RED directive in EU — EUR 15-50k). USB is plug-and-play: connect the cable, app auto-detects the sensor. Wireless is on the roadmap once we validate demand."

**"Can I build my own?"**
> "Absolutely! That's the point. The app is MIT licensed, firmware is open source, and the BOM is about EUR 12. I wrote a build guide: [LINK]. The pre-order kit is for people who'd rather just plug it in."

**"What about privacy / data collection?"**
> "Zero data leaves your machine. No cloud, no account, no telemetry (unless you explicitly opt in). All data is stored in a local SQLite database. The app doesn't even have network access by default — except the optional remote display feature (local network only). Full privacy policy: [LINK]."

**"Price seems high for a sensor"**
> "Fair point! The EUR 12 in parts is for DIY builders. The EUR 49 kit price covers assembly, testing, quality control, packaging, and shipping from Poland. You're also supporting an indie open source project. That said — the DIY route is always available."

---

## Hunter Strategy

### Should zentala hunt himself?

**Recommendation: self-hunt for now.**

Reasons:
- As a solo maker, self-hunting is normal and expected on PH
- No established PH reputation yet — hard to attract a top hunter
- Hunters add value mainly through their follower notifications — irrelevant if followers don't match our niche
- Self-hunting lets you control timing precisely

**Revisit for v2 launch**: if v1 gets traction and the product matures, approach a hunter for the "major update" launch.

### If Pursuing a Hunter (for future launches)

**How to find relevant hunters**:
1. Browse producthunt.com/topics/hardware — look at who hunted top products
2. Browse producthunt.com/topics/productivity — same approach
3. Check profiles: ideal hunter has 1k+ followers AND hunts hardware/health/productivity products
4. Look for hunters who hunted similar products: Oura Ring, Whoop, Framework Laptop, Pine64

**Approach template**:

Subject: "Would you hunt our open source desk sensor on Product Hunt?"

```
Hi [HUNTER NAME],

I noticed you hunted [PRODUCT THEY HUNTED] — we're building something
in a similar space.

[PRODUCT NAME] is an open source desk sensor (EUR 49) that tracks how
much you actually sit vs stand at your standing desk. Real-time overlay
bar, local data, no cloud. Built with Rust + Tauri.

We have [X] pre-orders and were featured on [HN/Reddit/etc.].

Would you be interested in hunting it? Happy to send you a kit to try
first.

Best,
Pawel
```

**When to send**: 2-3 weeks before target launch date. Follow up once after 5 days.

### Top Hunter Research Approach

Since Product Hunt hunter rankings change frequently, use this research method at launch time:

1. Go to producthunt.com/topics/hardware — note hunters of top 20 products from last 6 months
2. Go to producthunt.com/topics/health — same process
3. Cross-reference: hunters who appear in both lists are ideal
4. Check their Twitter/X — do they post about ergonomics, health tech, open source?
5. Prioritize hunters with 2k-10k followers (mega-hunters rarely accept cold pitches)
6. Build a shortlist of 5 hunters, approach in order of relevance

---

## Success Metrics

| Metric | Minimum | Good | Great |
|--------|---------|------|-------|
| Upvotes (24h) | 50 | 150 | 300+ |
| Comments | 10 | 30 | 50+ |
| PH ranking (daily) | Top 20 | Top 10 | Top 5 |
| Website visits from PH | 200 | 500 | 1000+ |
| Pre-orders from PH | 5 | 15 | 30+ |
| New GitHub stars | 20 | 50 | 100+ |
| Email signups | 30 | 80 | 150+ |

**Post-launch actions**:
- Update PH listing with "Featured" badge screenshot (if achieved)
- Add "As seen on Product Hunt" to landing page
- Share results on Twitter, Reddit, and with influencer contacts
- Write a "lessons learned" post for Indie Hackers
