# Support and Troubleshooting

## Common Issues and Solutions

### Issue: App won't start

**Problem:** the MoveUp tray icon appears but no window opens.

**Solutions:**

1. Click the tray icon (bottom-right) to open the floating window.
2. Check whether another copy is already running — MoveUp allows only one
   instance, and the second one exits silently.
3. Restart your computer.
4. Reinstall MoveUp (see [Installation Guide](./USER_INSTALL.md#uninstall)).

### Issue: Sensor not detected

**Problem:** MoveUp reports no sensor connected.

**Check the cable first.** On this board the cable is by far the most common
cause.

**Cable sensitivity — most USB-C cables do NOT work with this board.**
The XIAO ESP32-C3 uses *native* USB (no CH340/CP2102 bridge), so it is far
pickier than a classic Arduino. Three overlapping causes, all observed
2026-09-06:

1. Charge-only cables (VBUS+GND, no D+/D-) — board lights up, host sees
   nothing, and Windows enumerates **zero** COM ports.
2. C-to-C links depend on the board's 5.1k CC resistors; flipping the plug 180°
   or using an A-to-C cable often fixes a link that refuses to come up.
3. Voltage drop on thin (28 AWG) or long cables — the ESP32-C3 plus VL53L1X
   browns out mid-enumeration, producing a `DEVICE connected` / `DEVICE lost`
   loop within the same second, audible as repeated Windows plug/unplug chimes.

Diagnosis order when the app reports no sensor: check for a COM port at all
(`HKLM\HARDWARE\DEVICEMAP\SERIALCOMM`; empty = cable or power, not software),
then check `events.log` for connect/lost churn. Prefer a short (<=1 m) A-to-C
cable straight into the motherboard, bypassing USB hubs.

**If a COM port does exist and MoveUp still finds nothing:**

1. Try a different USB port, straight into the motherboard, not a hub.
2. Open Device Manager (right-click Start → Device Manager) and look for
   "XIAO ESP32" or a `USB Serial Device (COMn)` entry under **Ports (COM & LPT)**.
   The board's USB identity is `VID_303A&PID_1001`.
3. If the entry carries a warning icon, right-click → **Update driver**, or
   install the drivers from
   [Seeed Studio](https://wiki.seeedstudio.com/XIAO_ESP32C3_Getting_Started/).
4. Restart MoveUp after the sensor appears in Device Manager.
5. If the board is there but sends nothing, reflash the firmware — see
   [`firmware/README.md`](../firmware/README.md).

### Issue: Session timer doesn't update

**Problem:** sitting time does not increase, or seems to reset.

**Solutions:**

1. Calibrate the sensor: **Settings** → **Calibr.**, then follow the
   on-screen steps for the sitting and standing positions.
2. Check that readings arrive — the height shown in the floating window should
   change when you move the desk.
3. Move the desk fully up and down once; MoveUp needs a stable reading for
   about 5 seconds before it accepts a state change (debounce).

Note that sitting time is **not** reset when you sit down again after a break.
A break reduces it proportionally: in the default `standard` profile each
second of standing or walking cancels 3 seconds of sitting, and breaks shorter
than 60 seconds earn no credit at all.

### Issue: Break notification won't appear

**Problem:** no reminder after 40 minutes of sitting.

**Solutions:**

1. Check Windows notification settings: **Settings** → **System** →
   **Notifications** — MoveUp must be allowed to send notifications.
2. Check your profile: **Settings** → **Profiles**. The `silent` communication
   profile deliberately sends nothing, and every profile escalates visually
   (yellow → red → blinking tray) before it sends a single toast.
3. After each reminder the cooldown grows (0 → 5 min → 15 min → 30 min →
   silence). Once the reminders are exhausted MoveUp stays quiet until your
   position changes. That is by design, not a bug.
4. Make sure MoveUp actually sees you sitting — check the state shown in the
   floating window.

### Issue: Memory usage is high

**Problem:** MoveUp uses a lot of RAM.

**Solutions:**

1. High usage in the first 10-15 seconds after launch is normal.
2. If it stays above 300 MB after 30 seconds, restart MoveUp and check what
   else is running.
3. See the **[Optimization Guide](./OPTIMIZATION_GUIDE.md)** for the current
   memory baselines.

### Issue: App crashes on startup

**Problem:** MoveUp closes immediately after starting.

**Solutions:**

1. Read the logs for the error (see below).
2. Reinstall (see [Installation Guide](./USER_INSTALL.md#uninstall)); if that
   does not help, quit MoveUp and move `%APPDATA%\io.zntl.desk\` aside so the
   app starts from a clean state. Keep the folder — it holds your history.
3. [Report the crash](https://github.com/zentala/MoveUp/issues) with the logs
   attached.

## Accessing Logs

Logs help diagnose problems. To find them:

1. Open File Explorer.
2. Type `%APPDATA%\io.zntl.desk\logs\` in the address bar.
3. Open the folder for the day in question (`YYYY-MM-DD`).

Each day folder holds:

- `events.log` — one line per event: state changes, break credit, notifications,
  sensor connect/lost, alerts, daily resets, app starts
- `HH-MM.json` — a full snapshot of the session state, one file per minute

Logs older than 7 days are deleted automatically when the app starts.

**Sharing logs:** attach the relevant day's `events.log` to your GitHub issue.
Remove anything personal (paths, usernames) first.

## Accessing Application Data

Your data is stored in:

```
%APPDATA%\io.zntl.desk\
```

**Folder contents:**

- `desk.db` — SQLite database with your session history
- `logs\` — per-day event log and minute snapshots
- `profiles\` — ergonomic and communication profiles (editable JSON)
- `backups\` — timestamped copies of `desk.db`
- the settings store written by `tauri-plugin-store`

Do not edit these files while MoveUp is running.

## Settings

Open the floating window and click the settings icon. The tabs are
**Profiles**, **Calibr.**, **Notif.**, **More** and **Debug**.

### Profiles

Pick an ergonomic profile (limits, scoring, break credit) and a communication
profile (how loudly MoveUp nudges you). Built-in ergonomic profiles: `standard`
(default), `strict`, `relaxed`, `demo`. Built-in communication profiles:
`default`, `aggressive`, `gentle`, `silent`, `demo`. Profiles are plain JSON in
`%APPDATA%\io.zntl.desk\profiles\` and are reloaded when you edit them.

### Sensor Calibration

If the desk height readings look wrong:

1. Go to **Settings** → **Calibr.**
2. Lower the desk to sitting position, then click **Calibrate Sitting Height**.
3. Raise the desk to standing position, then click **Calibrate Standing Height**.
4. Save.

### Notifications

**Settings** → **Notif.** controls how MoveUp reaches you. The detailed
escalation timing and channels come from the selected communication profile.

### More

Screen-time tracking, the activity status readout ("Active" / "Idle Xm Ys"),
the timeline colour theme and the telemetry opt-in live here. Telemetry is off
unless you turn it on — see [Privacy](./PRIVACY.md).

### Updates

There is no update setting, because there is no auto-updater. See
[Updates](./USER_UPDATES.md) for the manual procedure.

## Getting Help

### Check these first

1. **[Installation Guide](./USER_INSTALL.md)** — setup problems
2. **[Updates](./USER_UPDATES.md)** — moving to a new version
3. **[Privacy](./PRIVACY.md)** — data questions
4. **[Remote Display](./REMOTE_DISPLAY.md)** — phone dashboard problems

### Report a bug

Open an issue at
[github.com/zentala/MoveUp/issues](https://github.com/zentala/MoveUp/issues)
and include:

1. steps to reproduce
2. what you expected and what happened instead
3. your Windows version and hardware
4. your MoveUp version
5. the relevant lines from `events.log`

## Performance and Optimization

See the **[Optimization Guide](./OPTIMIZATION_GUIDE.md)** for memory baselines
and the regression checklist.
