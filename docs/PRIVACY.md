# Privacy Policy

MoveUp keeps your data on your computer. Two optional features can send
something off the machine — the Google Fit step-count integration and the relay
that lets a paired phone reach your desk from anywhere. Both are off until you
turn them on, and this document describes each in full, along with every other
place data can leave the app.

**Applies to:** MoveUp 0.6.0.

## Summary

| Data | Where it goes |
|---|---|
| Session history, desk heights, settings, logs | Your computer only |
| Step counts (Google Fit) | Requested from Google over the internet — **only if you set it up** |
| Remote display (phone dashboard) | Served to devices on your local network |
| Paired phone over the relay | **Only if you turn it on**: the current state passes through our relay, which keeps no history |
| Telemetry | Off by default; nothing is transmitted in 0.6.0 (see below) |
| Update checks | None — the app never contacts an update server |

## What is stored locally

MoveUp stores the following on your computer:

- **Session history** — times you were sitting, standing, walking, or away
- **Desk height readings** — distances reported by your sensor
- **User preferences** — session limits, notification and profile settings
- **Logs** — application events, state transitions, and minute snapshots

Locations on Windows:

- **Application data:** `%APPDATA%\io.zntl.desk\`
  (`C:\Users\<you>\AppData\Roaming\io.zntl.desk\`)
  - `desk.db` — SQLite database with session history
  - `logs\YYYY-MM-DD\` — event log and per-minute snapshots, deleted after 7 days
  - `backups\` — automatic copies of `desk.db` made at startup, last 30 kept
  - `profiles\` — ergonomic and communication profiles
- **Installed program files:** `%LOCALAPPDATA%\MoveUp\`

## What MoveUp does not do

MoveUp does **not**:

- Log what you type or which windows you open
- Read your files, work content, or browsing history
- Show ads or embed third-party trackers
- Create an account, ask for your e-mail address, or ask you to sign in
- Contact any MoveUp-operated server unless you turn the relay on yourself
  (see [the relay](#pairing-a-phone-through-the-relay) below — off by default)

Keyboard and mouse hooks are used for one thing: deciding whether you are at
the computer. The app records "active" or "idle for N seconds" — never which
keys were pressed.

## Google Fit — the one integration that sends data off your machine

MoveUp can show your daily step count next to your desk stats. This feature
talks to Google's servers. It is **opt-in and off by default**: with no
credentials configured the code never makes a network call and the widget only
shows a "connect Google Fit" hint.

**How you turn it on.** You create your own Google Cloud OAuth client, run
`node scripts/google-fit-auth.cjs`, and set three values — `GOOGLE_CLIENT_ID`,
`GOOGLE_CLIENT_SECRET`, `GOOGLE_REFRESH_TOKEN` — in the `.env` file at the
repository root, or as environment variables for the running app. Nothing
happens until all three exist. The credentials stay on your machine; MoveUp
never uploads them anywhere except to Google's own token endpoint below.

**What is sent to Google**, once configured:

- Your OAuth client ID, client secret, and refresh token, to
  `https://oauth2.googleapis.com/token`, to obtain a short-lived access token
- The access token plus the start and end timestamps of the current day, to
  `https://www.googleapis.com/fitness/v1/users/me/dataset:aggregate`
- A request for your available step data sources, to
  `https://www.googleapis.com/fitness/v1/users/me/dataSources` (once per
  session, to pick which source to read; skipped if you pin one with
  `GOOGLE_FIT_STEPS_SOURCE`)

**What comes back:** a step count for today. Nothing else is stored or
displayed.

**What is never sent:** your desk height, session history, sitting or standing
times, scores, logs, or any MoveUp data at all. The traffic is one-way — the
app reads from Google and writes nothing back.

**Scope:** `https://www.googleapis.com/auth/fitness.activity.read` — read-only
access to activity data. MoveUp cannot modify or delete anything in your Google
account.

**How often:** while the steps widget is on screen, roughly every 5 minutes,
with backoff after failures. Requests stop entirely when the token is revoked.

**Google's own handling** of these requests is governed by
[Google's Privacy Policy](https://policies.google.com/privacy), not this one.

**How to turn it off:** delete the `GOOGLE_*` keys from `.env` and restart the
app, and/or revoke MoveUp's access in your
[Google account permissions](https://myaccount.google.com/permissions). The
feature then no-ops; nothing else in the app changes.

## Remote display (phone dashboard)

MoveUp runs a small HTTP and WebSocket server so you can open the same dashboard
on a phone or tablet. It listens on **port 3390 on every network interface**
(`0.0.0.0:3390`) and starts with the app.

- It serves your current desk state and today's session totals.
- There is **no password**. Anyone who can reach your computer on port 3390 —
  that is, anyone on the same local network — can view that dashboard.
- At most 10 clients may connect at once.
- Nothing is sent out to the internet; the server only answers requests that
  reach it.

If your machine sits on a network you do not trust, block port 3390 in Windows
Firewall — or switch the whole thing off in **Settings → Remote access →
"Serve the display on this network too"**, which stops the app from opening the
port at all. See [`REMOTE_DISPLAY.md`](REMOTE_DISPLAY.md) for setup details.

## Pairing a phone through the relay

MoveUp can also show the dashboard on a phone that is **not** on your network.
That path goes through a relay we operate at `relay.desk.zentala.io`. It is
**off by default**: with no licence key entered, the app never contacts it.

While it is on:

- **What passes through:** the same live state the local dashboard shows —
  current position, today's totals, sensor status — and the three commands a
  paired phone may send back (dismiss an alert, change a limit, switch a
  profile). The connection is TLS the whole way.
- **What the relay keeps:** the **latest snapshot only, in memory**, so a phone
  opened while your PC is off can show the last known state instead of a
  spinner. It disappears when the connection does. There is no history table,
  no database of readings, and no log of message contents in the cloud. Your
  session history never leaves your computer.
- **What is stored in the relay's database:** a hash of each device's token,
  the desk and device names you chose, and timestamps. **No plaintext token, no
  e-mail address, no password, no readings.** Your PC keeps its own token in the
  Windows Credential Manager, not in a settings file.
- **Who can connect:** only a device you paired with a code shown on your PC.
  The code lasts five minutes and works once. Every paired phone is listed in
  Settings, and removing one drops its connection within a second.
- **Turning it off:** **Settings → Remote access → Turn off and forget this
  desk** unregisters the desk and deletes the local credential — including when
  the relay cannot be reached.

Design and reasoning:
[ADR 022](../.arch/ADR/022-relay-on-cloudflare-durable-objects.md) and
[ADR 023](../.arch/ADR/023-pairing-code-device-token-auth.md).

## Telemetry

Settings → Telemetry has a "Share anonymized usage data" toggle. It is **off by
default**.

In version 0.6.0 the toggle transmits nothing. The receiving service is not
deployed, so when the switch is on, the daily summary (a random device ID, app
version, OS name, date, standing percentage, position changes, longest session,
daily score, active minutes) is written to the local debug log and goes no
further. No name, no email, no raw sensor data, no session detail is included
in that summary.

If a telemetry endpoint is ever activated, it will remain opt-in and this
document will be updated before the code ships.

## Updates

MoveUp has **no auto-updater**. The app never checks a server for a new version,
so no update request — and no IP address — is sent anywhere. You update by
downloading a new installer yourself; see [`USER_UPDATES.md`](USER_UPDATES.md).

## Sensor data

The VL53L1X sensor talks to MoveUp over USB only. It has no network connection
and no cloud component. Readings go straight into the local database.

## Getting your data out

There is no in-app export button in 0.6.0. Your data is in open formats and you
can take it directly:

- **Sessions:** `%APPDATA%\io.zntl.desk\desk.db` — SQLite, readable with any
  SQLite browser or `sqlite3`
- **Snapshots and events:** `%APPDATA%\io.zntl.desk\logs\` — JSON and plain text

Copy the folder anywhere you like; nothing in it is encrypted or locked to this
machine.

## Deleting your data

1. Close MoveUp (right-click the tray icon → Quit).
2. Open File Explorer and go to `%APPDATA%\io.zntl.desk\`.
3. Delete the folder. This removes the database, logs, backups, and settings.
4. Empty the Recycle Bin.

To remove the program as well, uninstall MoveUp from Windows Settings → Apps.

**This cannot be undone.** Note that step 3 also deletes `backups\`, so make a
copy of `desk.db` first if you want to keep your history.

## Third-party libraries

MoveUp is built on Tauri, React, and Rust crates. None of them is configured to
report usage or send analytics. The full list is in `package.json` and
`src-tauri/Cargo.toml`; the source is public, so you can check any claim here
against the code.

## Changes to this policy

Changes are recorded in the repository's git history and mentioned in release
notes on [GitHub Releases](https://github.com/zentala/MoveUp/releases). There is
no in-app notification of policy changes.

## Questions

Open an issue on [GitHub](https://github.com/zentala/MoveUp/issues).

**Source code:** [github.com/zentala/MoveUp](https://github.com/zentala/MoveUp)

---

**Last updated:** September 2026 (MoveUp 0.6.0)
