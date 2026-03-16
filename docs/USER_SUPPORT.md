# Support and Troubleshooting

## Common Issues and Solutions

### Issue: App won't start

**Problem:** zntlDesk icon appears but window doesn't open.

**Solutions:**
1. Click the system tray icon (bottom-right) to open the floating window
2. Check if another instance is already running (only one allowed)
3. Restart your computer
4. Uninstall and reinstall zntlDesk (see [Installation Guide](./USER_INSTALL.md#uninstall))

### Issue: Sensor not detected

**Problem:** "No sensor connected" message appears.

**Solutions:**
1. Check USB cable connection to your desk sensor
2. Try a different USB port
3. Open Device Manager:
   - Right-click Start → Device Manager
   - Look for "XIAO ESP32" under Ports (COM & LPT)
   - If found but labeled with warning: right-click → Update driver
4. Try installing USB drivers from [Seeed Studio](https://wiki.seeedstudio.com/XIAO_ESP32C3_Getting_Started/)
5. Restart zntlDesk after connecting the sensor

### Issue: Session timer doesn't update

**Problem:** Sitting time doesn't increase or resets unexpectedly.

**Solutions:**
1. Make sure the sensor is properly calibrated:
   - Go to **Settings** → **Sensor Calibration**
   - Follow the on-screen instructions
2. Check that sensor is receiving readings:
   - Click the height display in the floating window
   - You should see numbers changing (in cm)
3. Verify desk is not stuck in a position:
   - Manually move desk up and down a few times
   - Make sure sensor detects the changes

### Issue: Break notification won't appear

**Problem:** You don't see the "take a break" popup after 40 minutes of sitting.

**Solutions:**
1. Check Windows notification settings:
   - Go to **Settings** → **System** → **Notifications**
   - Make sure zntlDesk has notification permission
   - Turn on notifications if they're disabled
2. Verify break interval setting:
   - Open zntlDesk
   - Go to **Settings** → **General**
   - Check the "Break interval" value (default is 40 minutes)
3. Make sure you're actually sitting (sensor detecting low position)
4. Try standing for at least 5 minutes (this counts as a break)

### Issue: Memory usage is high

**Problem:** zntlDesk uses a lot of RAM.

**Solutions:**
1. This is normal for the first 10-15 seconds after launching
2. If memory stays above 300 MB after 30 seconds:
   - Restart zntlDesk
   - Check Windows Task Manager for other apps using RAM
   - Free up RAM by closing unused applications
3. See **[Optimization Guide](./OPTIMIZATION_GUIDE.md)** for details

### Issue: App crashes on startup

**Problem:** zntlDesk closes immediately after starting.

**Solutions:**
1. Check logs for error details (see below)
2. Try safe mode:
   - Uninstall (see [Installation Guide](./USER_INSTALL.md#uninstall))
   - Delete: `C:\Users\[YourUsername]\AppData\Local\zntlDesk\`
   - Reinstall
3. [Report the crash](https://github.com/zentala/zntl-tray/issues) with logs attached

## Accessing Logs

Logs help us diagnose problems. To access them:

1. Open File Explorer
2. Type in the address bar: `%APPDATA%\Local\zntlDesk\`
3. Open the **logs** folder
4. Find **zntlDesk.log** (most recent)

**Contents:** Application events, errors, sensor readings, state changes

**Sharing logs:**
- When reporting an issue on GitHub, attach the latest log file
- Remove any personal information first (file paths, usernames)

## Accessing Application Data

Your data is stored in:
```
C:\Users\[YourUsername]\AppData\Local\zntlDesk\
```

**Folder structure:**
- **zntlDesk.db** — SQLite database (your session history)
- **config.json** — Your settings and preferences
- **logs/** — Application logs
- **cache/** — Temporary data

**Note:** Do not edit these files directly while zntlDesk is running.

## Settings

### General Settings

**Break interval:** How many minutes of sitting before the break reminder (default: 40 minutes)

**Check for updates automatically:** Enable/disable auto-update checks (default: enabled)

### Sensor Calibration

If your desk height readings seem wrong:

1. Open zntlDesk
2. Go to **Settings** → **Sensor Calibration**
3. Lower your desk to sitting position
4. Click **Calibrate Sitting Height**
5. Raise your desk to standing position
6. Click **Calibrate Standing Height**
7. Click **Save**

### Notifications

To control notification settings:

1. Open zntlDesk
2. Go to **Settings** → **Notifications**
3. Toggle options:
   - **Desktop notifications** — Show popup alerts
   - **Sound** — Play sound with notifications
   - **Vibration** — Haptic feedback (if supported)

## Getting Help

### Check These First

1. **[Installation Guide](./USER_INSTALL.md)** — Setup issues
2. **[Updates Guide](./USER_UPDATES.md)** — Update problems
3. **[Privacy Guide](./PRIVACY.md)** — Data questions

### Ask the Community

Report bugs or ask questions:
- **GitHub Issues:** [github.com/zentala/zntl-tray/issues](https://github.com/zentala/zntl-tray/issues)
- **Include:**
  - What you were trying to do
  - What happened instead
  - Your Windows version (check Settings → System → About)
  - zntlDesk version (click Help → About)
  - Relevant log entries (see above)

### Report a Bug

When reporting a bug, please include:
1. Steps to reproduce the issue
2. What you expected to happen
3. What actually happened
4. Your system info (Windows version, hardware)
5. zntlDesk version number
6. Any error messages (copy from logs)

The more details you provide, the faster we can help!

## Performance and Optimization

For information about optimizing performance:
- See **[Optimization Guide](./OPTIMIZATION_GUIDE.md)**
- Includes memory reduction strategies
- Checklist for detecting performance regressions
