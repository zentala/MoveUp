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
| `DESK_REMOTE_TOKEN` | *(unset)* | Shared secret for the write endpoints. **Unset means closed** — `POST /display/health` answers `503` until you set it. |

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

### Display looks wrong / text too small

- Ensure **landscape** orientation in Fully Kiosk settings
- The UI auto-scales based on viewport width
- If portrait is shown, you'll see a "Please rotate to landscape" message
