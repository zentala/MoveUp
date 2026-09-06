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

## Data Storage

Your sessions, settings and logs live in:

```
%APPDATA%\io.zntl.desk\
```

That is `C:\Users\<you>\AppData\Roaming\io.zntl.desk\`. It contains:

- `desk.db` — SQLite database with your sitting/standing sessions
- `logs\YYYY-MM-DD\` — per-minute state snapshots and `events.log`
- `profiles\` — ergonomic and communication profiles
- `backups\` — timestamped copies of `desk.db`
- the settings store written by `tauri-plugin-store`

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

If a COM port does appear and the sensor is still not found, see
[Support — Sensor not detected](./USER_SUPPORT.md#issue-sensor-not-detected).

### "Installer won't run"

- Windows SmartScreen blocks unsigned installers. Click **More info** →
  **Run anyway**.
- Your antivirus may quarantine the file — allow it, or download again from
  the [releases page](https://github.com/zentala/MoveUp/releases).

## Next Steps

- **[Settings](./USER_SUPPORT.md#settings)** — desk calibration, break interval
- **[Updates](./USER_UPDATES.md)** — how to install a new version
- **[Privacy](./PRIVACY.md)** — what MoveUp stores and what leaves your machine
- **[Remote Display](./REMOTE_DISPLAY.md)** — optional phone dashboard

## Getting Help

1. Check **[Support](./USER_SUPPORT.md)** for common problems.
2. Open an issue: <https://github.com/zentala/MoveUp/issues>
