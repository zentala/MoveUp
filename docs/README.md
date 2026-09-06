# MoveUp Documentation

MoveUp is an ergonomics tracker for a sit/stand desk. Here you'll find guides
for installation, updates, support and privacy.

## For Users

- **[Installation Guide](./USER_INSTALL.md)** — download, install and set up MoveUp
- **[Updates](./USER_UPDATES.md)** — how to move to a new version (manual, no auto-updater)
- **[Support & Troubleshooting](./USER_SUPPORT.md)** — common problems, logs, settings
- **[Privacy](./PRIVACY.md)** — what MoveUp stores and what leaves your machine
- **[Remote Display](./REMOTE_DISPLAY.md)** — turn an old phone into a desk dashboard

## For Developers

- **`../CLAUDE.md`** — architecture, session logic, testing strategy, Tauri plugins
- **[Optimization Guide](./OPTIMIZATION_GUIDE.md)** — performance baselines and strategies
- Source: <https://github.com/zentala/MoveUp>

## Quick Links

| Topic | Guide |
|-------|-------|
| How do I install? | [Installation Guide](./USER_INSTALL.md) |
| How do I update? | [Updates](./USER_UPDATES.md) |
| Where are my logs? | [Support Guide](./USER_SUPPORT.md#accessing-logs) |
| What data leaves my machine? | [Privacy](./PRIVACY.md) |
| My sensor isn't detected | [Support Guide](./USER_SUPPORT.md#issue-sensor-not-detected) |
| How can I help? | [GitHub Issues](https://github.com/zentala/MoveUp/issues) |

## Frequently Asked Questions

**Q: Does MoveUp send my data anywhere?**
A: Session tracking is local. The optional Google Fit step import and the
optional remote display are the only network features. See
[Privacy](./PRIVACY.md).

**Q: How do updates work?**
A: You download and run the new installer yourself. There is no auto-updater.
See [Updates](./USER_UPDATES.md).

**Q: Where is my session data stored?**
A: In `%APPDATA%\io.zntl.desk\` (that is
`C:\Users\<you>\AppData\Roaming\io.zntl.desk\`). See
[Support Guide](./USER_SUPPORT.md#accessing-application-data).

**Q: My sensor isn't detected. What do I do?**
A: Most often it is the USB cable. See
[Support Guide — Sensor not detected](./USER_SUPPORT.md#issue-sensor-not-detected).

**Q: How do I uninstall?**
A: See [Installation Guide — Uninstall](./USER_INSTALL.md#uninstall).

## Getting Help

1. Search this documentation — most questions are answered here.
2. Check the [Support Guide](./USER_SUPPORT.md) for troubleshooting steps.
3. Open an issue: <https://github.com/zentala/MoveUp/issues>

When reporting a problem, include:

- what you were doing and what happened (with error messages)
- your Windows version and your MoveUp version
- the relevant log files from `%APPDATA%\io.zntl.desk\logs\`

## What is MoveUp?

MoveUp helps you keep a healthy posture and take regular breaks. It:

- **detects desk position** with a laser distance sensor (sitting vs standing)
- **tracks how long you have been sitting** and warns you as you approach the
  session limit (40 minutes by default)
- **credits breaks** — standing and walking cancel sitting time
- **records your history** so you can see your patterns in the Analyst window

## System Requirements

- **Windows 10 or later** (Windows 11 recommended)
- **~150 MB disk space**
- **USB port** for the desk sensor (optional — the app runs without it, but
  cannot detect desk position)

## Get Started

1. **[Install MoveUp](./USER_INSTALL.md)**
2. Connect your sensor, if you have one.
3. Calibrate your sitting and standing heights (Settings → Sensor Calibration).
4. Start tracking.
