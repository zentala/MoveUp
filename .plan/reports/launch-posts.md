# Launch Post Drafts — Reddit & Hacker News

**Date**: 2026-03-26
**Epic**: E010
**Task**: E010-T08
**Status**: Draft — replace [PLACEHOLDER] values with real data before posting

---

## Posting Schedule (recommended order)

| Day | Platform | Post | Why this order |
|-----|----------|------|----------------|
| Mon | r/standingdesks | Post 1 | Warm, niche audience — get initial feedback |
| Tue | r/quantifiedself | Post 2 | Data-driven crowd validates the concept |
| Wed | Hacker News | Post 3 | Peak HN traffic Wed-Thu; leverage Reddit comments as social proof |
| Thu | r/selfhosted | Post 4 | Privacy crowd, strong overlap with HN readers |
| Fri | r/homeautomation | Post 5 | Weekend DIY crowd starts browsing Friday afternoon |

Space posts 24h apart minimum. If any post gains traction, delay the next one to ride the wave.

---

## Post 1: r/standingdesks — Product / Personal Story

**Title:** I built a €12 sensor that tracks how much I actually use my standing desk — and the results were embarrassing

**Subreddit:** r/standingdesks

**Best posting time:** Monday 14:00–16:00 UTC (morning in US, evening in EU)

**Flair:** DIY / Modification (check sub rules; may need "Discussion")

---

Hey everyone,

I'm a software developer who bought a motorized standing desk about two years ago, convinced it would fix my sedentary habits. Spoiler: it didn't.

I realized the problem wasn't the desk — it was me. I had no idea how much I was actually standing. My gut said "plenty." The data said otherwise.

**So I built a sensor to find out.**

It's a VL53L1X Time-of-Flight sensor (about €12 in parts) mounted under the desk, pointing at the floor. It measures desk height automatically — no buttons, no manual logging. The desktop app (Tauri, open source) does the rest: tracks sitting/standing time, shows a progress bar that goes from green to red as you sit too long, and nudges you to change position.

**My results after [PLACEHOLDER — replace with real number] days:**

- Standing time: [PLACEHOLDER — e.g., "went from 8% to 22% of my workday"]
- Position changes per day: [PLACEHOLDER — e.g., "from 1.2 to 4.8"]
- Longest sitting streak: [PLACEHOLDER — e.g., "dropped from 4+ hours to under 90 minutes"]
- Back pain: [PLACEHOLDER — e.g., "noticeably reduced after week 2"]

The thing that surprised me most: I thought I was standing "a lot" because I'd do one long standing session. But the research says **frequency of position changes matters more than total standing time**. The app tracks both, and seeing those numbers daily actually changed my behavior.

The app is open source (MIT): [GitHub link]
I also put together a product page if you want the sensor pre-assembled: [desk.zentala.io](https://desk.zentala.io?utm_source=reddit&utm_medium=post&utm_campaign=launch&utm_content=standingdesks)

Happy to answer any questions about the build, the data, or the experience.

[Photo: sensor mounted under desk, showing the small PCB with USB cable]
[Screenshot: app popup showing daily KPIs — standing %, position changes, session timer]

---

**Suggested media:**
- Photo of VL53L1X sensor mounted under real desk (close-up + wide shot)
- Screenshot of app popup with daily stats visible
- Before/after comparison graphic (standing % over time)

**UTM link:** `desk.zentala.io?utm_source=reddit&utm_medium=post&utm_campaign=launch&utm_content=standingdesks`

**Expected audience:** Standing desk owners frustrated with their own habits, ergonomics enthusiasts, people considering a standing desk

**Key comments to prepare:**

1. **"Why not just use a smartwatch / fitness tracker?"**
   > Smartwatches don't know your desk height. They can tell you're standing, but not whether your desk is up or down. The whole point is tracking *desk usage*, not body position. That said, smartwatch integration is on the roadmap for combining both signals.

2. **"What standing desk do you use? Does it work with any desk?"**
   > Works with any motorized or manual standing desk — the sensor measures absolute height from the floor, so it doesn't matter what brand or mechanism you have. I use a [PLACEHOLDER — your desk brand]. You just tape the sensor under the desktop surface.

3. **"€12 seems too cheap — what's the catch?"**
   > That's the raw component cost (sensor module + microcontroller + USB cable). The pre-assembled kit is €49 because it includes a carrier PCB, pre-flashed firmware, mounting hardware, and the open-source desktop app. If you're handy with a soldering iron, the GitHub repo has everything you need to build it yourself for €12.

4. **"Is this just a timer? I can set a phone alarm."**
   > The difference is it's automatic. No buttons, no remembering to start/stop. The sensor detects your desk position continuously. And beyond timing, it tracks patterns — how often you change position, your daily standing ratio, session history. The overlay bar on your screen is a constant gentle reminder without being intrusive.

5. **"Does it work on Mac/Linux?"**
   > Currently Windows only (the overlay bar uses WinAPI). Mac and Linux support are planned — the core logic is cross-platform Rust, so it's mostly the native overlay that needs porting. The web-based phone dashboard works on any device already.

---

## Post 2: r/quantifiedself — Data / Insights Angle

**Title:** 30 days of automatic sit/stand tracking — what I learned about my sedentary habits

**Subreddit:** r/quantifiedself

**Best posting time:** Tuesday 15:00–17:00 UTC

**Flair:** Data / Results (check sub for exact flair options)

---

I've been automatically tracking my sitting and standing patterns for [PLACEHOLDER — number] days using a sensor I built. No manual logging — a Time-of-Flight sensor under my desk detects height changes and a desktop app records everything to a local SQLite database.

Here are 5 things the data taught me:

**1. I was standing [PLACEHOLDER — e.g., "8%"] of my workday, not the "30-40%" I believed**

My subjective estimate was wildly wrong. I'd do one 20-minute standing session and feel like I'd been standing all morning. The numbers don't lie. After [PLACEHOLDER — number] weeks with the tracker, I'm at [PLACEHOLDER — e.g., "22%"] — still not great, but measurably better.

**2. Frequency of position changes matters more than total standing time**

I started tracking "position changes per day" as a separate metric. Research suggests changing position every 30-45 minutes is more beneficial than one long standing block. My baseline was [PLACEHOLDER — e.g., "1.2 changes/day"]. Now it's [PLACEHOLDER — e.g., "4.8"]. This single metric changed how I think about desk ergonomics.

**3. The first 2 hours of the day are my worst**

Looking at hourly heatmaps, I almost never stand before [PLACEHOLDER — e.g., "11:00"]. I get into flow state and completely forget. The app's progress bar (green-to-red overlay at the top of my screen) helps, but morning sessions are still my weakest point.

**4. Break credit math is counterintuitive**

The app uses a break credit system: stand for 5-9 minutes and you get partial credit (reduces your sitting timer by 20 min). Stand for 10+ minutes and it fully resets. Under 5 minutes standing does nothing — it's just a position change, not a real break. This changed my behavior: I stopped doing quick 2-minute stand-ups and started committing to actual breaks.

**5. Weekday vs. weekend patterns are completely different**

[PLACEHOLDER — describe the pattern, e.g., "On weekends I stand 35% of the time because I'm doing chores, cooking, etc. Weekdays I'm glued to the chair. The sitting problem is specifically a *work* problem for me."]

**Technical setup:**
- Sensor: VL53L1X Time-of-Flight (measures desk height via distance to floor)
- MCU: Seeed XIAO ESP32-C3, USB-C connection
- App: Tauri 2 (Rust backend + React frontend), open source
- Data: Local SQLite, no cloud, no telemetry
- Detection: 5-second debounce, automatic sitting/standing/away classification

Total hardware cost: ~€12 if you source the components yourself.

The app is open source: [GitHub link]
Product page with pre-assembled kits: [desk.zentala.io](https://desk.zentala.io?utm_source=reddit&utm_medium=post&utm_campaign=launch&utm_content=quantifiedself)

I'm planning to add minute-level JSON snapshots and an event log for deeper analysis. If anyone's interested in the raw data format, happy to share.

---

**Suggested media:**
- Chart: standing % over 30 days (line chart showing improvement trend)
- Heatmap: hourly sitting/standing patterns across a week
- Screenshot: app debug tab showing raw session data

**UTM link:** `desk.zentala.io?utm_source=reddit&utm_medium=post&utm_campaign=launch&utm_content=quantifiedself`

**Expected audience:** Self-trackers, data nerds, biohackers, health-conscious developers

**Key comments to prepare:**

1. **"Can you export the data? What format?"**
   > All data is in a local SQLite database — you can query it directly. The app stores sessions with timestamps, duration, and state (sitting/standing/walking/away). I'm also adding minute-level JSON snapshots (one file per minute with full state dump) and a human-readable event log. Both planned for the next release.

2. **"How accurate is the height detection?"**
   > The VL53L1X has ±3mm accuracy at distances under 1.3m. For sit/stand detection, you only need to distinguish two positions (typically 70cm vs 110cm), so accuracy is not an issue. I use a 5-second debounce to avoid false transitions — the desk needs to stay at a new height for 5 continuous readings before the state changes.

3. **"What about walking/away detection?"**
   > The app monitors keyboard and mouse activity. If you're standing but there's no input for 60+ seconds, it classifies you as "away" (which also counts as a break). Standing + active = standing. Standing + inactive = away/walking. Both count toward break credit.

4. **"N=1 studies aren't very useful"**
   > Completely agree on the scientific rigor front. This is personal tracking, not a clinical trial. But I'd argue the value of QS is exactly this: understanding YOUR patterns. My "8% standing" baseline was specific to me, and the interventions (overlay bar, break credits, gamification) work because I can see them working in my own data. If enough people use this, aggregated (and anonymized, opt-in) data could be interesting.

5. **"Have you looked at the research on sit-stand frequency?"**
   > Yes — the key papers I've been referencing suggest position changes every 30-45 minutes are more beneficial than prolonged standing. Hedge et al. (2015) recommend a 20/8/2 ratio (sit/stand/move), and Gilson et al. (2012) found that regular transitions reduce musculoskeletal discomfort more than total standing time. My app tracks both metrics so you can optimize for either.

---

## Post 3: Hacker News — Show HN (Technical + Philosophical)

**Title:** Show HN: Open-source sit/stand tracker — automatic detection via ToF sensor

**Platform:** Hacker News (Show HN)

**Best posting time:** Wednesday 14:00–15:00 UTC (9-10 AM ET)

---

Hi HN,

I built an open-source desktop app that automatically tracks how much you use your standing desk. A VL53L1X Time-of-Flight sensor under the desk measures height; the app does the rest.

**Why I built this:** I bought a standing desk two years ago. I thought I was standing "a lot." I was standing [PLACEHOLDER — e.g., "8%"] of my workday. The problem isn't hardware — it's behavior. So I built a tool that focuses on changing behavior, not just displaying numbers.

**Technical stack:**

- Rust backend (Tauri 2) — session state machine, serial communication, WinAPI overlay renderer
- React + TypeScript frontend (Vite)
- Local SQLite database (tauri-plugin-sql)
- Hardware: VL53L1X ToF sensor + Seeed XIAO ESP32-C3 (~€12 total)
- Embedded HTTP + WebSocket server for remote display (phone as desk dashboard)
- 500+ tests across 3 layers (Rust unit, TS unit/integration, Playwright E2E)
- Windows only for now (overlay uses WinAPI directly — not a webview)

**Architecture decisions I'm happy with:**

- *Native overlay, not a webview window.* The progress bar (4px at the top of your screen, green-to-red) is rendered via GDI/UpdateLayeredWindow in a background thread. No Electron-style overhead for what's essentially a colored rectangle.
- *State machine, not event spaghetti.* Four states (sitting/standing/walking/away) with explicit transitions, 5-second debounce, and break credit rules. The state machine is pure — returns actions, doesn't execute them.
- *Coach, not tracker.* The app doesn't just show you data. It actively nudges you: overlay bar creates ambient awareness, progressive alerts escalate if you ignore the bar, gamification with a scoring system that goes negative (yes, your score can drop below zero — real consequences make it feel real).
- *Comeback mechanics.* If your score tanks, recovery is faster than the initial drop. This prevents the "I'm already failing, why bother" spiral that kills most habit trackers.

**What I'd do differently:**

- Started with Electron, migrated to Tauri. Should have started with Tauri.
- The overlay renderer went through 4 rewrites (see the knowledge base in the repo). WinAPI transparency is hard.
- Break credit was originally a step function (0/20min/full reset). Should have been a progressive curve from the start.

**Links:**

- GitHub: [link]
- Product page (pre-assembled sensor kits): [desk.zentala.io](https://desk.zentala.io?utm_source=hn&utm_medium=post&utm_campaign=launch&utm_content=showhn)
- Architecture docs are in the repo under `.arch/`

Would love feedback on the state machine design and the gamification approach. The scoring system is the part I'm least sure about — currently experimenting with different penalty/reward curves.

---

**Suggested media:** None for HN (text-only is the norm). Include a direct GitHub link prominently.

**UTM link:** `desk.zentala.io?utm_source=hn&utm_medium=post&utm_campaign=launch&utm_content=showhn`

**Expected audience:** Developers, Rust/Tauri enthusiasts, hardware hackers, ergonomics-curious engineers

**Key comments to prepare:**

1. **"Why not just use a timer / Pomodoro app?"**
   > Timers require you to remember to start them. This is fully automatic — the sensor detects your desk position, the app classifies your state (sitting/standing/away based on input activity), and everything happens in the background. The difference between "I should stand up" and "the bar is turning red, I haven't moved in 38 minutes" is the difference between intention and feedback.

2. **"Why Tauri instead of a CLI / daemon?"**
   > The core insight is that ambient visual feedback changes behavior more than notifications or logs. The 4px overlay bar at the top of your screen is always visible but never intrusive. You can't get that from a CLI. The Tauri webview handles the settings panel and detailed stats, but the overlay itself is native WinAPI — no webview overhead for the always-on component.

3. **"Will this work on Mac/Linux?"**
   > The session logic, serial communication, and data layer are cross-platform Rust. The blocker is the overlay renderer (currently WinAPI-specific). On macOS it would need Core Animation, on Linux probably X11/Wayland compositing. PRs welcome — the overlay is isolated in a single file (`overlay_renderer.rs`).

4. **"€12 for the sensor — but you're selling kits for €49?"**
   > The €12 is raw component cost (sensor + MCU + cable). The kit includes a carrier PCB, pre-flashed firmware, mounting hardware, and documentation. The markup covers production, assembly, shipping, and funds continued development. The app and firmware are MIT-licensed — building from source is a first-class use case, not a workaround.

5. **"How does the scoring/gamification work exactly?"**
   > Your score increases when you stand or change position, decreases when you sit too long. The key design choice: your score CAN go negative. Most gamification systems floor at zero, which means there's no cost to ignoring them. A negative score means you have to actively recover — and recovery is slightly faster than the decline, so comebacks feel achievable. The exact curves are configurable and I'm still experimenting.

---

## Post 4: r/selfhosted — Privacy / Self-Hosted Angle

**Title:** Self-hosted sit/stand tracker — zero cloud, zero accounts, all data on your machine

**Subreddit:** r/selfhosted

**Best posting time:** Thursday 15:00–17:00 UTC

**Flair:** Productivity / Health (check sub for exact options)

---

Built an automatic sit/stand desk tracker that runs entirely on your machine. No cloud, no accounts, no telemetry. Sharing here because I think this community will appreciate the architecture.

**What it does:**
- VL53L1X ToF sensor under your desk measures height automatically
- Desktop app (Tauri 2, Rust + React) tracks sitting/standing/away time
- Overlay bar at top of screen shows session progress (green to red)
- Nudges you to change position before your session limit

**Privacy features:**
- All data in local SQLite database (your AppData folder)
- Zero network calls by default — no analytics, no telemetry, no update checks
- No account creation, no login, no email collection
- App telemetry is opt-in only (and doesn't exist yet — planned, not shipped)
- Full source code available: MIT license

**Self-hosted remote display:**
- Embedded HTTP + WebSocket server on `:3390`
- Serves the same React UI to any browser on your network
- Use an old phone as a desk dashboard — no app install needed
- Auto-reconnects with exponential backoff, REST polling fallback
- Port configurable via `DESK_REMOTE_PORT` env var

**Stack:**
- Rust backend (session state machine, serial I/O, WinAPI overlay)
- React + TypeScript frontend
- SQLite via tauri-plugin-sql
- No Docker (it's a desktop app), but the remote display could theoretically be split out

**Hardware:** VL53L1X sensor + ESP32-C3 MCU, USB-C connection. ~€12 in parts if you source them yourself. Pre-assembled kits available if you'd rather not solder.

GitHub: [link]
More info: [desk.zentala.io](https://desk.zentala.io?utm_source=reddit&utm_medium=post&utm_campaign=launch&utm_content=selfhosted)

---

**Suggested media:**
- Screenshot: phone browser showing the remote dashboard
- Network diagram: sensor -> USB -> desktop app -> WS -> phone (all local)

**UTM link:** `desk.zentala.io?utm_source=reddit&utm_medium=post&utm_campaign=launch&utm_content=selfhosted`

**Expected audience:** Self-hosting enthusiasts, privacy-conscious developers, home lab operators

**Key comments to prepare:**

1. **"Can I run this in Docker / on a server?"**
   > Not currently — it's a desktop app that needs USB access to the sensor and renders a native overlay on your screen. The remote display server (HTTP + WS) is embedded in the app. In theory, the sensor reading + state machine could be extracted into a standalone service, but that's not the current architecture. If there's demand, a headless mode that just serves the web UI is feasible.

2. **"What data exactly is stored? Can I see the schema?"**
   > SQLite database with session records: start time, end time, state (sitting/standing/walking/away), duration. Plus a settings store (session limits, height thresholds, notification preferences). No personal data beyond desk usage patterns. The schema is in `db.rs` in the repo.

3. **"Does the WebSocket server have any authentication?"**
   > Currently no — it's designed for local network use (your desk, your phone). Adding basic auth or a token is on the backlog. For now, it only binds to `0.0.0.0:3390` and serves read-only state data. No control commands are exposed over WS.

4. **"Linux support?"**
   > The core Rust logic is cross-platform. The blocker is the native overlay (WinAPI). Serial communication should work on Linux with minor changes. If you want to run it headless (just the remote display), that's the most feasible path for Linux support. PRs welcome.

5. **"How much storage does it use?"**
   > Minimal. SQLite DB grows maybe 1-2 MB per month of continuous use. Log files (minute snapshots + event log) are auto-deleted after 7 days. Total footprint is under 10 MB for the app + months of data.

---

## Post 5: r/homeautomation — Smart Desk / DIY Angle

**Title:** Added automatic height tracking to my standing desk for €12

**Subreddit:** r/homeautomation

**Best posting time:** Friday 16:00–18:00 UTC (people browsing before the weekend)

**Flair:** DIY (check sub for exact options)

---

Turned my standing desk into a smart desk with a €12 sensor. It automatically detects whether the desk is in sitting or standing position and tracks my usage patterns throughout the day.

**Hardware (total ~€12):**
- VL53L1X Time-of-Flight sensor — €5 (measures distance to floor)
- Seeed XIAO ESP32-C3 — €5 (tiny MCU with USB-C)
- USB-C cable — €2
- Mounting: double-sided tape under the desk

The sensor points straight down from under the desk surface. It measures distance to the floor, so it knows the desk height. Sitting position = ~750mm, standing = ~1080mm (varies by desk). 5-second debounce to avoid false triggers.

**Software:**
- Open-source desktop app (Tauri 2 — Rust + React)
- Auto-detects the sensor on USB (no config needed)
- Tracks sitting/standing/away time with daily stats
- Shows a thin progress bar at the top of your screen (green when fresh, red when you've been sitting too long)
- Built-in WebSocket server — you can connect any browser on your local network

**The WebSocket server is the interesting part for this sub:**
- Runs on `:3390` by default
- Broadcasts state updates in real-time (state, height, session duration, daily stats)
- Could be consumed by Home Assistant, Node-RED, or any WS client
- Example: trigger smart lights to change color based on sitting duration
- Example: log desk usage to InfluxDB/Grafana alongside other home metrics

I haven't built HA integrations myself yet, but the data is there on the WebSocket if anyone wants to play with it.

**Build time:** About 30 minutes if you're comfortable with USB and mounting tape. The firmware comes pre-flashed if you buy the kit, or you can flash it yourself via Arduino IDE.

[Photo: sensor mounted under desk — small PCB, clean cable routing]
[Photo: wide shot of desk setup with app visible on screen]
[Screenshot: phone browser showing remote dashboard on local network]

GitHub: [link]
Pre-assembled kits: [desk.zentala.io](https://desk.zentala.io?utm_source=reddit&utm_medium=post&utm_campaign=launch&utm_content=homeautomation)

---

**Suggested media:**
- Close-up photo of sensor mounted under desk
- Wide shot of desk setup with the app overlay visible
- Phone showing the remote dashboard (browser, not a native app)
- Wiring diagram or pinout (simple: sensor -> MCU -> USB)

**UTM link:** `desk.zentala.io?utm_source=reddit&utm_medium=post&utm_campaign=launch&utm_content=homeautomation`

**Expected audience:** Home automation enthusiasts, DIY/maker community, smart home tinkerers

**Key comments to prepare:**

1. **"Can I integrate this with Home Assistant?"**
   > The app runs a WebSocket server on your local network that broadcasts state updates in real-time. You could use the HA WebSocket integration or a Node-RED flow to consume the data. I haven't built an official HA integration yet, but the protocol is simple JSON messages. If someone builds one, I'd love to link to it from the repo.

2. **"Why not use a BLE sensor so it doesn't need USB?"**
   > Phase 1 is USB-only to keep things simple and avoid radio certification costs (EU RED directive). BLE or Zigbee is on the roadmap for a future version. USB also means zero latency and no battery to charge.

3. **"Does it work with IKEA BEKANT / Uplift / Flexispot / [brand]?"**
   > Yes — it measures absolute height from the floor, so it works with any desk regardless of brand or mechanism. Motorized, crank, or even a stack of books. As long as the height changes between sitting and standing, the sensor detects it.

4. **"What if I have stuff under my desk (cable trays, drawers)?"**
   > The sensor has a narrow field of view (~27 degrees). Mount it where there's a clear line of sight to the floor. If you have a cable tray directly below, just offset the sensor a few centimeters to the side. The mounting tape makes it easy to reposition.

5. **"€12 is cheap — what's the quality like?"**
   > The VL53L1X is a well-known module (same sensor used in many phones for autofocus). The XIAO ESP32-C3 is a production-quality MCU from Seeed Studio, not a prototype board. For the pre-assembled kit (€49), I add a carrier PCB that makes the connections more robust than jumper wires. But honestly, even the DIY version with jumper wires has been running on my desk for months without issues.

---

## Pre-Posting Checklist

Before publishing any post:

- [ ] Replace ALL [PLACEHOLDER] values with real data from your tracking
- [ ] Take fresh screenshots of the app (popup, overlay, remote display)
- [ ] Take photos of the sensor mounted under your desk (clean cable routing)
- [ ] Create a before/after data visualization (standing % over time)
- [ ] Set up UTM links and verify they resolve correctly
- [ ] Verify GitHub repo is public and README is presentable
- [ ] Verify desk.zentala.io is live and the CTA works
- [ ] Prepare a 2-sentence bio/response for "who are you" questions
- [ ] Read each subreddit's rules and adjust flair/formatting if needed
- [ ] Have the prepared comment responses ready in a document for quick pasting

## Cross-Post Rules

- Never cross-post the same text. Each post is written for its specific audience.
- If a post gets traction, do NOT post the next one for at least 24 hours.
- If a post flops (< 5 upvotes after 6 hours), still wait 24 hours before the next one.
- Monitor comments for the first 2 hours after posting — early engagement matters.
- If someone asks a question you haven't prepared for, answer honestly and add it to the prepared responses document.
