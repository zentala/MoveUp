# Remote Display — Phone Dashboard Setup

Turn an old phone or tablet into a dedicated desk dashboard. The app runs an embedded HTTP + WebSocket server that streams live session data to any browser on your local network.

## Requirements

- Android phone or tablet (any age — even old devices work)
- Same WiFi network as the PC running MoveUp
- A kiosk browser app (recommended: Fully Kiosk Browser)

## Quick Setup

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

## How It Works

1. MoveUp starts an embedded HTTP server alongside the Tauri app
2. `GET /display` serves the same React UI used by the desktop popup
3. The browser connects via WebSocket (`/display/ws`) for real-time updates
4. Session state, metrics, today's summary, and health stream at ~1 event/sec
5. If the connection drops, the phone auto-reconnects with exponential backoff

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
