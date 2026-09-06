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

## How It Works

1. MoveUp starts an embedded HTTP server alongside the Tauri app
2. `GET /display` serves the same React UI used by the desktop popup
3. The browser connects via WebSocket (`/display/ws`) for real-time updates
4. Session state, metrics, and today's summary stream at ~1 event/sec
5. If the connection drops, the phone auto-reconnects with exponential backoff

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
