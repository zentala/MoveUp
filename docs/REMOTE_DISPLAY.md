# Remote Display — Phone Dashboard Setup

Turn a phone or tablet into a desk dashboard. There are two ways to reach the
desk, and they show the same screen:

| | **Pair a phone** (Pro) | **Your own Wi-Fi** (free) |
|---|---|---|
| Works from | anywhere, mobile data included | the same network as the PC |
| Setup | a code on the phone, once | the PC's IP address and a firewall rule |
| Sign-in | pairing code, revocable per device | none — anyone on the network can look |
| Controls | dismiss an alert, change limits, switch profile | view only |
| PC turned off | last known state, with the time it was last seen | nothing |

Pairing is the shorter road and the private one. Start there; the LAN section
below is still the whole story if you never want the desk to talk to a server.

## Pair a phone — works from anywhere

Your PC connects **out** to a relay at `relay.desk.zentala.io`; the phone
connects out to the same place. Nothing listens for incoming connections, so
there is no firewall rule, no port forward, and no need for the phone and the PC
to share a network.

Requires a licence key (Pro). The relay never stores your ergonomics history —
see [what the relay holds](#what-the-relay-holds) below.

### 1. Turn on remote access on the PC

Open **Settings → Remote access**, type your licence key into **Licence key**
and press **Enable remote access**. The line above the field is the status; it
should settle on *"Online — your phone can reach this desk"*.

### 2. Show a pairing code

Press **Pair a phone**. An 8-character code appears together with a QR image.
The code is valid for **5 minutes** and can be used **once**. Ten wrong guesses
lock pairing on this desk for 15 minutes.

### 3. Enter it on the phone

Scan the QR — it opens the dashboard already filled in — or open
`https://relay.desk.zentala.io/app/#/pair` on the phone and type the desk id and
the code shown next to it.

That is the whole setup. The phone stores a token of its own; it never asks
again.

### 4. What you can do from the phone

| Control | What happens on the PC |
|---|---|
| Dismiss | same as clicking the alert popup, snooze rules included |
| Sit / stand limit | the same change as Settings, with the same clamps (sit 5–240 min, stand 1–120 min) |
| Profile | switches the ergonomic or communication profile by name |

Everything else — calibration, the rest of Settings — stays on the PC on
purpose. A phone cannot measure your desk, and a general "write any setting"
path from the internet is exactly what this design refuses to have.

Each command is written to `logs/YYYY-MM-DD/events.log` as a `REMOTE` line, so
there is a record of what was changed from where. Anything not on the list above
is refused and logged as `REMOTE DENIED`.

### 5. Removing a phone

**Settings → Remote access → Paired phones** lists every paired device. Remove
one and its connection drops within a second and cannot come back — that is the
answer to a lost phone.

**Turn off and forget this desk** unregisters the desk entirely. It clears the
local credential even when the relay cannot be reached: turning sharing off must
never depend on a working network.

On the phone, **Forget this desk** clears its own stored token. The desk owns
the list, so a phone cannot un-pair itself server-side.

### What the relay holds

- The **latest snapshot only**, in memory, so a phone opened while the PC is off
  shows the last known state with a "desk offline" banner instead of a spinner.
  It is gone when the connection is.
- **Hashed tokens and device names** — never a plaintext token, never a reading,
  never a history row. Your ergonomics data stays on the PC.
- Pairing codes live in memory and expire in five minutes.

Details: [`PRIVACY.md`](PRIVACY.md) and
[ADR 022](../.arch/ADR/022-relay-on-cloudflare-durable-objects.md) /
[ADR 023](../.arch/ADR/023-pairing-code-device-token-auth.md).

## On your own Wi-Fi — the LAN dashboard

The app also runs an embedded HTTP + WebSocket server that streams live session
data to any browser on your local network. No account, no relay, nothing leaves
the machine — and no password either, so treat it as "anyone on this network may
look".

### Requirements

- Android phone or tablet (any age — even old devices work)
- Same WiFi network as the PC running MoveUp
- A kiosk browser app (recommended: Fully Kiosk Browser)

### 1. Find your PC's local IP

On the PC running MoveUp, open a terminal:

```
ipconfig
```

Look for your WiFi adapter's **IPv4 Address** (e.g. `192.168.0.105`).

### 2. Install Fully Kiosk Browser

Download **Fully Kiosk Browser** from Google Play Store (free version works).

### 3. Configure the URL

In Fully Kiosk Browser, set the start URL to:

```
http://<PC-IP>:3390/display
```

Example: `http://192.168.0.105:3390/display`

### 4. Recommended Fully Kiosk settings

| Setting | Value |
|---------|-------|
| Start URL | `http://<PC-IP>:3390/display` |
| Orientation | Landscape |
| Screen always on | Yes |
| Fullscreen mode | Yes |
| Autostart on boot | Yes (optional) |
| Status bar | Hidden |
| Navigation bar | Hidden |

Fully Kiosk works the same way with a paired phone: set the start URL to
`https://relay.desk.zentala.io/app/` once the tablet has been paired, and the
same kiosk settings apply. Pair first in a normal browser tab — the code entry
is easier there.

### 5. Allow through Windows Firewall

The remote display server listens on port **3390**. You may need to allow inbound connections:

1. Open **Windows Defender Firewall** > **Advanced Settings**
2. Click **Inbound Rules** > **New Rule...**
3. Select **Port** > **TCP** > Specific port: `3390`
4. **Allow the connection**
5. Name it: `MoveUp Remote Display`

## Configuration

| Env variable | Default | Description |
|--------------|---------|-------------|
| `DESK_REMOTE_PORT` | `3390` | Port for the HTTP + WebSocket server |
| `DESK_REMOTE_TOKEN` | *(unset)* | Shared secret for the write endpoints. **Unset means closed** — `POST /display/health` and `POST /display/voice` both answer `503` until you set it. |
| `OPENROUTER_API_KEY` | *(unset)* | Your own OpenRouter key for the optional AI reply to a dictated note. Unset = no reply, not an error. |
| `DESK_NOTIFY_WEBHOOK_URL` | *(unset)* | Fallback for the phone-notification URL when Settings → More leaves it blank. |

## Turning the LAN display off

The LAN display is on by default and has no login: anyone already on your
network can open it. If that is not what you want, switch it off — the toggle
is `remote_lan_enabled` in the app config (Settings → Remote), default `true`.

Set to `false`, the app does not open port 3390 at all: no dashboard, no
WebSocket, no health or voice inlet. Nothing else changes — the desk keeps
tracking, and a paired phone reached through the relay keeps working, because
that path does not go through this server.

A config written by an older version carries no such key, and a missing key
counts as **on**. Only an explicit `false` closes the port.

## How It Works

1. MoveUp starts an embedded HTTP server alongside the Tauri app
2. `GET /display` serves the same React UI used by the desktop popup
3. The browser connects via WebSocket (`/display/ws`) for real-time updates
4. Session state, metrics, today's summary, and health stream at ~1 event/sec
5. If the connection drops, the phone auto-reconnects with exponential backoff

Every message the socket sends is wrapped in the shared v1 envelope —
`{v, type: "event", id, ts, payload}` — with the event itself untouched inside
`payload`. The relay sends the same shape, so one browser client reads both.

The socket is **read-only**: it never acts on anything a client sends. A frame
that arrives is logged at debug level and dropped. Remote controls (dismiss an
alert, change a limit, switch a profile) exist only over the relay, which
authenticates each device.

## Health Push Inlet — `POST /display/health`

The phone (or anything else on the LAN) can push today's health metrics into
the app. They join whatever the app already reads — Google Fit today — through
one aggregator, so the widget and the phone snapshot never learn which source
produced a number.

### 1. Set a token

Reads are open on the LAN; the one endpoint that *writes* app state requires a
shared secret. Put it in `apps/desk/.env` and restart the app:

```
DESK_REMOTE_TOKEN=<a-long-random-string>
```

Generate one with `openssl rand -hex 24`, or any password manager. While the
variable is unset the inlet is **closed**, not open: every push gets `503`.

### 2. Push a reading

```bash
curl -X POST http://<PC-IP>:3390/display/health \
  -H "Content-Type: application/json" \
  -H "X-Desk-Token: $DESK_REMOTE_TOKEN" \
  -d '{"steps_today":1234,"heart_rate_bpm":61,"source_id":"curl","measured_at_ms":'"$(date +%s000)"'}'
```

The response is the merged health view the app now holds, so a bare `curl` is
also the diagnostic for "did it land".

### Request schema

| Field | Type | Required | Notes |
|---|---|---|---|
| `steps_today` | integer ≥ 0 | yes | Steps so far today, from the pusher's own midnight |
| `heart_rate_bpm` | integer 0–65535 | no | Omit when you have no reading — do not send `0` |
| `hrv_rmssd_ms` | number ≥ 0 | no | RMSSD in milliseconds |
| `source_id` | string, 1–64 chars | yes | Names the pusher (`phone`, `curl`, `health-connect`); shown as the widget's source label |
| `measured_at_ms` | integer > 0 | yes | Unix milliseconds the reading was taken |

Bodies are capped at **1 KiB**. A reading older than **1 hour** stops being
reported: the source stays "configured" but withdraws its snapshot, so the app
falls back to another source rather than showing an hour-old step count as
current.

### Responses

| Status | Meaning |
|---|---|
| `200` | Accepted; body is the merged `HealthView` |
| `401` | `X-Desk-Token` missing or wrong |
| `413` | Body over 1 KiB |
| `422` | Parsed, but a value is unusable (blank/over-long `source_id`, non-positive `measured_at_ms`, negative or `NaN` HRV) |
| `400` | Body is not JSON matching the schema |
| `503` | `DESK_REMOTE_TOKEN` is not set on the PC |

## Voice Dictation — `POST /display/voice`

The phone dashboard has a dictation panel. Talk to the desk app instead of
walking back to the keyboard: ask for five more minutes, say you are going for
a walk, or leave a note. The reply comes back as a phone notification, which
Android mirrors to a paired watch.

The panel appears on `/display` only — the desktop popup already has a
keyboard.

### 1. Unlock the panel with the token

The first time you open `/display`, the panel asks for the same
`DESK_REMOTE_TOKEN` you set for the health inlet above. It is stored in the
browser's `localStorage` on that phone and sent as `X-Desk-Token` on every
note. Without it the app answers `401`; with no token set on the PC at all,
`503`.

### 2. Dictate with the keyboard's mic

Tap the text field, then tap **the microphone on your phone's own keyboard**
(Gboard, Samsung Keyboard) and speak. Tap **Send**.

This is the primary path on purpose. It is your phone's normal dictation, so
it works in any browser, in Fully Kiosk, and over plain `http://`.

A second mic button — the browser's own speech recognition — appears **only**
when your browser supports it and the microphone permission is not blocked.
On most phones opening `http://<PC-IP>:3390` it will not appear, because
Chrome reserves that API for secure (HTTPS) pages. Nothing is broken when the
button is missing; use the keyboard mic.

### 3. What it understands

Polish and English, matched offline on the PC — no cloud, no API key:

| Say | What happens |
|---|---|
| "drzemka 5" · "odłóż o 10 minut" · "snooze" · "remind me in 20" | Snoozes the current alert for that many minutes (1–180, default 5) |
| "idę na spacer" · "wychodzę" · "going for a walk" | Recorded as a walk note. It does **not** change what the desk sensor reports |
| "wracam" · "koniec spaceru" · "I'm back" | Recorded as the end of that walk |
| anything else | Kept as a plain note — nothing you say is ever thrown away |

Every note is saved locally: a `VOICE` line in `logs/YYYY-MM-DD/events.log`
and a row in the app's database, visible in the Analyst window under
Catalog → `voice_notes`.

### 4. Optional — an AI reply (bring your own key)

Add your own [OpenRouter](https://openrouter.ai) key to `apps/desk/.env` and
each note comes back with a short (≤60-word) ergonomics reply in the language
you spoke:

```
OPENROUTER_API_KEY=<your-key>
```

The model defaults to a cheap one and can be changed in Settings → More.
Without a key you still get the acknowledgement — the reply is simply absent.
Your transcripts go to OpenRouter only while this key is set; nothing is sent
anywhere by default.

### 5. Optional — replies on your phone and watch

Turn on **Settings → More → Phone notifications** and give it an
[ntfy](https://ntfy.sh)-style URL. MoveUp then pushes the sit-limit alert and
every voice acknowledgement to that endpoint:

```
POST <your-url>
Content-Type: application/json

{"title": "...", "message": "...", "priority": 3, "tags": []}
```

Install the ntfy app on the phone, subscribe to the topic, and Android mirrors
those notifications to a paired smartwatch. That is the whole watch setup —
there is nothing to install on the watch.

The push is fire-and-forget (5 s timeout, one retry on a server error), so a
dead endpoint never slows the app down. The default is off; nothing leaves
your machine until you set this.

### Request schema

| Field | Type | Required | Notes |
|---|---|---|---|
| `transcript` | string, 1–2000 chars | yes | What was said, already turned into text |
| `lang` | string | no | BCP-47 tag, e.g. `pl-PL`. Stored with the note |
| `captured_at_ms` | integer > 0 | yes | Unix milliseconds; decides which day the note belongs to |

Bodies are capped at **4 KiB**.

### Responses

| Status | Meaning |
|---|---|
| `200` | Accepted; body is the acknowledgement — transcript, intent, snooze minutes, reply, note id |
| `401` | `X-Desk-Token` missing or wrong |
| `413` | Body over 4 KiB |
| `422` | Parsed, but `transcript` is empty/too long or `captured_at_ms` is not positive |
| `400` | Body is not JSON matching the schema |
| `503` | `DESK_REMOTE_TOKEN` is not set on the PC |

Test it without a phone:

```bash
curl -X POST http://<PC-IP>:3390/display/voice \
  -H "Content-Type: application/json" \
  -H "X-Desk-Token: $DESK_REMOTE_TOKEN" \
  -d '{"transcript":"drzemka 5","captured_at_ms":'"$(date +%s000)"'}'
```

## Troubleshooting

### Pairing: "that code is not valid"

- Codes expire after **5 minutes** and work **once** — press **Pair a phone**
  again for a fresh one.
- The alphabet has no `I`, `O`, `0` or `1`, so a character that looks like one
  of those is the other one.
- After ten wrong tries pairing is locked on that desk for 15 minutes. The
  message says when it reopens.

### The phone says "Desk offline — showing last known state"

The relay is reachable and the PC is not: it is asleep, powered off, or MoveUp
is not running. The data on screen is the last snapshot, with the time it was
taken. This is different from "Reconnecting…", which means the **phone** lost
the relay.

### Settings shows something other than "Online"

| Line | Means |
|---|---|
| *Licence expired or not valid for this desk* | the licence ended or was revoked; the app stops retrying on purpose |
| *This desk was removed from the licence* | the desk registration was deleted — register again |
| *Another desk took over this registration* | the same registration is in use elsewhere; only one desk may hold it |
| *Cannot reach the relay* | network or relay outage; the app keeps retrying with a growing delay |

### "Cannot connect" / page won't load

- Verify phone and PC are on the **same WiFi network**
- Check the PC's IP hasn't changed (use `ipconfig` again)
- Ensure Windows Firewall allows port 3390 (see step 5 above)
- Try opening the URL in the phone's regular browser first

### "Reconnecting..." shown on phone

- The desktop app is not running — start MoveUp on the PC (from source:
  `pnpm tauri:dev`)
- WiFi dropped briefly — the phone will auto-reconnect within seconds
- PC went to sleep — wake it up, the phone reconnects automatically

### "Sensor disconnected" banner

- The USB cable to the desk sensor may be loose — check the connection
- The sensor board may need a reset — unplug and replug the USB cable
- The sensor is optional for the display — dashboard still shows last known data

### The mic button on the dictation panel is missing

Expected on most phones. The browser's speech API needs an HTTPS page and
`/display` is served over plain HTTP. Use your keyboard's own microphone
instead — tap the text field, then the mic on the keyboard.

### Dictation returns 401, or nothing happens on Send

- `401` — the token in the panel does not match `DESK_REMOTE_TOKEN` on the PC.
  Reopen the token sheet and paste it again.
- `503` — the PC has no `DESK_REMOTE_TOKEN` set at all. Set it in
  `apps/desk/.env` and restart the app.

### Display looks wrong / text too small

- Ensure **landscape** orientation in Fully Kiosk settings
- The UI auto-scales based on viewport width
- If portrait is shown, you'll see a "Please rotate to landscape" message
