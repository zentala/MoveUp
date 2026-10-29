# Installation Guide

This guide walks you through installing MoveUp on a Windows computer.

## System Requirements

- **Windows 10 or later** (Windows 11 recommended)
- **Disk space**: ~150 MB for the application and its data
- **RAM**: 512 MB minimum (1 GB recommended)
- **USB port**: for the desk height sensor (optional)

## Download

Download the latest installer from
[GitHub Releases](https://github.com/zentala/MoveUp/releases) — look for the
Windows setup asset, `MoveUp_<version>_x64-setup.exe`.

## Step-by-Step Installation

### 1. Run the installer

Double-click `MoveUp_<version>_x64-setup.exe` to start the wizard.

MoveUp installs for the current user into `%LOCALAPPDATA%\MoveUp\`, so it does
not ask for administrator rights. Windows SmartScreen may still warn you — the
installer is not code-signed yet (see [Troubleshooting](#installer-wont-run)).

### 2. Choose the installation location

The installer suggests `%LOCALAPPDATA%\MoveUp\`. Accept it, or click **Browse**
to pick another folder.

### 3. Installation progress

The installer copies files to your computer. This takes a few seconds.

### 4. Shortcuts

You can add MoveUp to the Start menu and, optionally, to the desktop.

### 5. Launch

When the wizard finishes, start MoveUp from the Start menu. It runs in the
system tray — look for the tray icon showing the current desk height.

## First Run

On first launch MoveUp:

1. looks for the sensor on the USB serial ports and connects if it finds one
2. creates its database and log folders under `%APPDATA%\io.zntl.desk\`
3. shows the welcome window and asks you to calibrate your sitting and standing
   desk heights

## Optional PM3 supervision on development machines

Where PM3 is installed, prefer it as the sole owner of the running application.
The repository can prepare an ordinary PM3 configuration without launching
MoveUp or registering a service. Select the actual application executable;
preparation does not select or promote release candidates.

```powershell
node scripts/prepare-pm3.mjs --install-root "C:/path/to/MoveUp" --executable "C:/path/to/MoveUp/desk.exe" --output-dir "C:/path/to/new-config-directory"
```

The output directory must already exist and must not contain `pm3.yaml`.
All three paths must be absolute. Optional `--port` changes the local readiness
port in both the application environment and PM3's HTTP probe. Optional
`--pm3-node ABS --pm3-home ABS` runs the installed public `lint` command with
an isolated PM3 home. Lint warnings are non-blocking and a successful lint is
not a running-app check. No candidate file or callback is required.

The configuration uses the exact executable, `--minimized`, automatic restart
with capped delay, and boot restoration for services PM3 still desires to run.
An unexpected exit, including exit code zero, is eligible for restart.
Ordinary `stop` clears that restore intent. A separate per-user login activation
must be verified if deliberate Quit should last only until the next login.

Installation also requires verifying cooperative Quit and development-replacement
behavior, plus exactly one app process and actual recovery evidence. A successful
checkpoint protects the last saved state, but it does not establish that Windows
PM3 stop performs a cooperative final flush or completes its final diagnostics.
This preparation command does not implement or verify those lifecycle steps. Keep
the existing direct Windows login entry until the replacement is proven; then
record its rollback value before disabling it. Never run both launch paths as
competing owners. See [Support](./USER_SUPPORT.md#pm3-startup-diagnostics).

Current verification distinguishes three Windows termination paths. Native tray
Quit saved Store state before PM3 stopped the app and it stayed stopped, although
the final lifecycle summary did not complete. Native main-window close separately
saved state and PM3 recovered the process. Ordinary Windows PM3 stop uses taskkill
and can force termination without a cooperative flush. Cooperative flush-before-stop
on that PM3 stop path and completion of final lifecycle diagnostics remain unresolved;
a successful command is not proof of either.

Settings and session state now use one application-owned writer. It writes a
complete JSON snapshot through a same-directory temporary file, flushes that
file, and replaces the settings file atomically. A periodic session checkpoint
runs every 60 seconds independently of sensor readings. After a successful
checkpoint, recovery retains the saved known state. When saves succeed and the
schedule runs normally, the unsaved changes at risk are usually those since the
last successful save plus scheduling and save latency—not zero loss. Failed saves
are reported and can widen that window. Forced termination cannot run an immediate
final flush, and time without observed sensor data remains Unknown after restart.
This is crash protection, not proof of hardware or power-loss durability, or of the
new runtime installed on Windows.
The reviewed PM3 login activation is now registered in the existing Windows Run
entry, with its REG_SZ type and exact value verified by readback after the reviewed
build and native readiness checks. The previous value and type are retained for
rollback. The earlier operator-triggered manual activation is separate evidence;
no reboot was performed and an actual subsequent login remains NOT_CHECKED.

## Data Storage

Your sessions, settings and logs live in:

```
%APPDATA%\io.zntl.desk\
```

That is `C:\Users\<you>\AppData\Roaming\io.zntl.desk\`. It contains:

- `desk.db` — SQLite database with your sitting/standing sessions
- `logs\YYYY-MM-DD\` — per-minute state snapshots and `events.log`
- `profiles\` — ergonomic and communication profiles
- `backups\` — up to 30 validated timestamped SQLite-consistent copies of an
  existing `desk.db`; the first launch has no database to copy. Backup failures
  are recorded in `logs\...\events.log` and do not prevent startup.
- the settings JSON store, written by MoveUp's shared persistence owner

The program folder (`%LOCALAPPDATA%\MoveUp\`) holds only the application
itself, so reinstalling never touches your data.

## Uninstall

1. Open **Settings** → **Apps** → **Installed apps**.
2. Find **MoveUp** in the list.
3. Click it and select **Uninstall**.

Your session data is **not** deleted. To remove it too:

1. Quit MoveUp from the tray icon.
2. Delete the folder `%APPDATA%\io.zntl.desk\`.

## Troubleshooting Installation

### The app reports no sensor

Check the USB cable before anything else — on this board the cable is the most
common cause by a wide margin.

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

A COM port confirms that the USB controller enumerated; it does not prove that
the assembled sensor is working or sending readings. Before calibrating, move
the desk and confirm that the height reading changes in MoveUp. A missing
reading is unknown data, not evidence that the person is away.

If a COM port does appear and the sensor is still not found, see
[Support — Sensor not detected](./USER_SUPPORT.md#issue-sensor-not-detected).

### "Installer won't run"

- Windows SmartScreen blocks unsigned installers. Click **More info** →
  **Run anyway**.
- Your antivirus may quarantine the file — allow it, or download again from
  the [releases page](https://github.com/zentala/MoveUp/releases).

## Next Steps

- **[First sensor reading and calibration help](./USER_SUPPORT.md#issue-sensor-not-detected)** — safe cable checks and what a real reading confirms
- **[Settings](./USER_SUPPORT.md#settings)** — desk calibration, break interval
- **[Updates](./USER_UPDATES.md)** — how to install a new version
- **[Privacy](./PRIVACY.md)** — what MoveUp stores and what leaves your machine
- **[Remote Display](./REMOTE_DISPLAY.md)** — optional phone dashboard

## Getting Help

1. Check **[Support](./USER_SUPPORT.md)** for common problems.
2. Open an issue: <https://github.com/zentala/MoveUp/issues>
