# Privacy Policy

Your privacy is important. zntlDesk is designed to keep all your data local and private.

## Data We Collect

### What is Stored Locally

zntlDesk stores the following data on **your computer only**:

- **Session history** — Times you were sitting, standing, or away
- **Desk height readings** — Data from your sensor
- **User preferences** — Your settings (break interval, notification preferences)
- **Activity logs** — Application events and errors for troubleshooting

**All data is stored in:** `C:\Users\[YourUsername]\AppData\Local\zntlDesk\`

### What We Don't Collect

zntlDesk **does NOT:**
- Send any data to servers (cloud, analytics, telemetry)
- Track your activity outside the app
- Log your work content or browsing
- Collect personal information
- Use ads or third-party trackers
- Create user profiles or accounts
- Monitor your keyboard or mouse input beyond idle detection

## Data Security

### Local-Only

All your data stays on your computer. No information leaves your machine unless **you** explicitly export it.

### Database Protection

Your session data is stored in an SQLite database (`zntlDesk.db`). This file is:
- Created only on your computer
- Protected by Windows file permissions
- Not sent anywhere

### Logs

Application logs are stored in `AppData\Local\zntlDesk\logs\`:
- Contains app events, errors, and sensor readings
- Does **not** contain personal files, browsing history, or work content
- Only used for troubleshooting

## Automatic Updates

When checking for updates, zntlDesk:
- Connects to GitHub to check version numbers
- Checks your current version
- GitHub may log the IP address (standard web server practice)
- **No personal data is sent**

To disable this:
1. Open zntlDesk
2. Go to **Settings** → **General**
3. Turn off **Check for updates automatically**

## Exporting Your Data

You own your data. You can export it anytime:

1. Open zntlDesk
2. Go to **Settings** → **Data Management**
3. Click **Export All Data**
4. Choose a location to save the file

**Exported file format:** JSON (readable in any text editor)
**Includes:** All session history, sensor readings, preferences

## Deleting Your Data

To permanently delete all zntlDesk data:

1. **Close zntlDesk completely**
2. **Open File Explorer**
3. **Type in the address bar:** `%APPDATA%\Local\zntlDesk\`
4. **Delete the entire zntlDesk folder**
5. **Empty Recycle Bin**

**Warning:** This cannot be undone. All session history will be lost.

To reset the app but keep your install:
1. Open zntlDesk
2. Go to **Settings** → **Data Management**
3. Click **Reset App Data**
4. Confirm the warning

## Sensor Data Privacy

Your desk sensor (VL53L1X) communicates directly with zntlDesk via USB:
- **No network connection** from the sensor
- **No cloud storage** of readings
- **All readings stay on your computer**

## Windows Activity Access

zntlDesk detects keyboard/mouse activity to determine if you're away. This:
- Checks if you've moved your mouse or pressed keys in the last few minutes
- **Does NOT log what keys you pressed**
- **Does NOT record what windows are open**
- **Does NOT access file contents**
- Is only used to tag sessions as "away" if idle for 10+ minutes

## Third-Party Libraries

zntlDesk uses open-source libraries. All are vetted for:
- No telemetry or tracking
- No external network calls (except GitHub updates)
- No personal data collection

**Key libraries:**
- Tauri — Application framework (local execution only)
- React — UI library (no tracking)
- Rust — Backend (no external calls)

See `package.json` and `Cargo.toml` for full dependency list.

## Changes to This Policy

We may update this privacy policy. You will be notified:
- In-app notification of changes
- Release notes on [GitHub Releases](https://github.com/zentala/zntl-tray/releases)

Your use of zntlDesk after changes constitutes acceptance.

## Questions?

If you have concerns about privacy:
1. Review your data export (see above)
2. Check the logs for unexpected activity
3. Open an issue on [GitHub](https://github.com/zentala/zntl-tray/issues)

## Compliance

zntlDesk is designed with privacy-by-default:
- GDPR compliant (no personal data shared)
- CCPA compliant (you control your data)
- No tracking pixels or beacons
- Open source (verify the code yourself)

**Source code:** [github.com/zentala/zntl-tray](https://github.com/zentala/zntl-tray)

---

**Last updated:** March 2026

zntlDesk respects your privacy. All your data belongs to you.
