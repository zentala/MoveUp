# Support and Troubleshooting

## Save a UI diagnostics report

When a window is blank or unresponsive, use the native tray menu **Save UI
diagnostics** before restarting anything. Collection does not show or create
Analyst, reload a page, change settings or restart MoveUp. The native confirmation
shows the actual report path under the existing daily logs directory. An
incomplete report still has a saved path and an explicit collection warning;
unconfirmed publication and write failures are reported separately.

The bounded JSON includes run identity, window/navigation generations, native
process/navigation/power events, probe outcomes and safe boot/view/query/error
categories. It excludes page text, HTML, inputs, raw errors/stacks/URLs, settings
and sensor history. Review the report before sharing: run identity and timestamps
remain local support information. Nearby `events.log` retains bounded diagnostic
receipts and write/loss counters expose gaps.

A timeout is an observed lack of response, not proof of a crash. An absent
Analyst is expected when it was never opened. DOM or a mounted React stage cannot
prove that pixels were painted. Unsupported registrations, interrupted probes,
collection failures and dropped events remain separate. Backend readiness and
checkpoint availability are independent observations; neither proves UI health.
Real sleep/resume and behavior of an installed version require their own checks.

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

### Issue: Break reminder won't appear

**Problem:** no reminder is visible when expected.

**Solutions:**

1. MoveUp presents policy-approved reminders in its existing main window, not
   Windows notification center. Automatic presentation does not take focus;
   click the MoveUp tray icon to open or focus the current message.
2. Check that the main window shows a current Sitting state. If readings are
   missing, follow [Sensor not detected](#issue-sensor-not-detected); missing
   readings do not mean Away.
3. Check the communication profile in **Settings** → **Notifications**. The
   `silent` profile deliberately stays silent. Reminder timing and channels are
   controlled by the selected profile.
4. To inspect the reminder surface, use **Test appearance** and **Test
   reminder** in **Settings** → **Notifications**. A preview is labelled as a
   test, shares the 60-second test limit, and does not change your session. If
   it is queued while another view is open, return to the main status window.
   See [Application reminders](#application-reminders) for presentation and
   dismissal behavior.

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

### Issue: View unavailable

**Problem:** the floating window displays only `View unavailable.`

This means a React view failed while rendering. The application may write a
best-effort `UI_FAILURE` line to `%LOCALAPPDATA%\io.zntl.desk\logs\MoveUp.log`.
It contains an allowlisted error kind and, when a recognized stack frame
points under the existing `/src/` or `/assets/` route, a basename with line
and column. These route-based locations are hints only; they do not prove
module ownership or authenticity. The entry does not include the raw exception
message, stack, full path, URL or query. It may be absent if logging fails.

This fallback covers React render errors. A native WebView or GPU hang/crash is
outside React and is not caught by this boundary. This diagnostic does not
establish the cause of an earlier blank view.

## Accessing Logs

### PM3 startup diagnostics

If MoveUp is registered with PM3, inspect `pm3 describe moveup/desk --json`,
`pm3 health moveup/desk`, `pm3 events --json`, and `pm3 logs moveup/desk`.
Use PM3 for process operations rather than starting another executable.
An empty application stdout tail is possible for Windows GUI builds; it does
not prove that the application is silent or stopped. Use the persistent
application `events.log` and PM3's disk lifecycle records to correlate starts,
exits and restarts. Early startup failures also have local
`moveup-startup-*.log` files in the Windows temporary directory.

The local readiness response returns HTTP 503 when database initialization,
tray initialization or setup is incomplete, after an observed failure, or
while quitting. Its JSON includes the reason and process identity. HTTP 200
with `state: degraded` and `reason: sensor_missing` means the core application
is ready without the sensor. `session_store_available: false` separately means
the session settings Store is unavailable; HTTP 200 does not certify that
session persistence works. Preserve this limitation in diagnostic reports.
Graceful Quit and native close currently require a successful session Store
save as well as the database flush. If the Store is unavailable or its save
fails, MoveUp reports the failure and remains running; core HTTP readiness
alone does not establish that Quit works. The application-owned settings writer
uses a same-directory temporary file and a flushed replacement, and the plugin's
independent exit-time writer is deregistered while Rust readers retain the shared
Store. A sensor-independent 60-second checkpoint records its last success or
current error in lifecycle state and `events.log`; failures produce a native
notification with rate limiting. After a successful checkpoint, recovery retains
that saved known state. With healthy saves and normal scheduling, changes since
the last successful save plus scheduling and save latency are usually the loss
window; this is not a zero-loss guarantee. Failed saves are reported and may widen
the window. Forced termination cannot run an immediate final flush, and unobserved
offline time remains Unknown. This gives routine crash protection, but does not
prove hardware or power-loss durability or the newly integrated runtime running
on an installed Windows system.
The installed ordinary PM3 probe checks HTTP status rather than JSON identity;
compare the returned PID and version with the actual supervised process during
installation and recovery checks.

PM3 restarts unexpected clean exits too. Deliberate Quit must flush state before
ordinary PM3 stop; stop removes boot restoration intent, so `boot: true` is
insufficient to promise a return at next login. Verify the separate per-user
login activation and development replacement before enabling this installation.
A manually triggered activation demonstrates its action, not an actual login.

Observed native tray Quit saved Store state before PM3 stopped the app, and the
app stayed stopped. The supervisor terminated it before the final lifecycle
summary completed, so that summary can still show ready/not_attempted rather
than the observed pre-stop save. Final diagnostic completeness is unresolved.
Native main-window close separately saved Store state, needed no database row,
recorded clean exit and was automatically recovered by PM3. That recovery is
expected: an exit without explicit user Quit intent remains restartable.

Ordinary Windows PM3 stop is a different path: taskkill followed by force can
terminate the process without running application cleanup. An observed successful
stop left flush=not_attempted and no clean_exit. A cooperative flush before that
stop and completion of the final lifecycle diagnostics remain unresolved; command
exit zero is not persistence evidence. Do not infer graceful save from the label
SIGTERM in supervisor records.

The reviewed PM3 login activation is now registered in the existing Windows Run
entry. After the reviewed build and native readiness checks, readback verified
the exact value and REG_SZ type; the previous value and type are retained for
rollback. Registration did not execute the action again. The earlier manual
activation remains separate evidence. No reboot was performed; an actual
subsequent login remains NOT_CHECKED, independently of process recovery.

The supervisor also depends on its Windows Scheduled Task lifetime. Battery
conditions, interactive-user requirements, missed-trigger behavior and a task
execution time limit may interrupt supervision. Check the installed task;
application configuration does not override those settings.

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
- `backups\` — up to 30 validated SQLite-consistent copies of an existing
  `desk.db`; failures are recorded in `events.log` and do not block startup
- the settings JSON store written by MoveUp's shared persistence owner

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

To record your calibration heights:

1. Confirm that the height in the main MoveUp window changes as the desk moves.
2. Open **Settings** → **Calibr.**
3. Enter the measured sitting height in **Sitting height (mm)**.
4. Move the desk to standing position and enter the measured height in **Standing height (mm)**.
5. Select **Save**.

### Notifications

**Settings** → **Notif.** controls how MoveUp reaches you. The detailed
escalation timing and channels come from the selected communication profile.

### More

Screen-time tracking, the activity status readout ("Recent input" / "No recent input"),
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
## Application reminders

MoveUp shows policy-approved reminders in its existing main window. A reminder
stays until you dismiss or snooze it, or its reason is no longer current. Automatic
presentation does not request keyboard focus. Click the tray icon to focus the
current reminder; the Analyst and phone display remain separate.

Settings edits and an open Share dialog take priority. Messages wait until you
return to status, without closing the current editor or dialog. In Notifications,
choose a **Test appearance** and **Test reminder** to preview information, warning,
urgent or success styling. A queued confirmation means you can return to status
to view it. These are labelled tests, share one 60-second rate limit and leave
your ergonomic session unchanged.

If saving a response fails, keep the reminder open and choose **Retry saving**.
This completes the pending save without applying your response twice, including
after the view remounts. If the reminder connection fails, use **Retry connection**.
Messages never supply missing observations or change your recorded posture history.
