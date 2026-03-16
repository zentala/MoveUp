# zntlDesk Documentation

Welcome to zntlDesk! Here you'll find guides for installation, updates, support, and privacy.

## For Users

**Getting started?**
- **[Installation Guide](./USER_INSTALL.md)** — Download, install, and set up zntlDesk
- **[Updates](./USER_UPDATES.md)** — Automatic updates, manual checks, and rollback
- **[Support & Troubleshooting](./USER_SUPPORT.md)** — Common issues, logs, and settings
- **[Privacy](./PRIVACY.md)** — How your data is handled (spoiler: it stays local)

## For Developers

**Working on zntlDesk?**
- See **`../CLAUDE.md`** for:
  - Architecture and testing strategy
  - Tauri plugins and configuration
  - Memory profiling and optimization guidelines
  - Pre-release checklist

- **[Optimization Guide](./OPTIMIZATION_GUIDE.md)** — Performance baselines and strategies

## Quick Links

| Topic | Guide |
|-------|-------|
| How do I install? | [Installation Guide](./USER_INSTALL.md) |
| How do I update? | [Updates](./USER_UPDATES.md) |
| Where are my logs? | [Support Guide](./USER_SUPPORT.md#accessing-logs) |
| Is my data private? | [Privacy Policy](./PRIVACY.md) |
| How can I help? | [GitHub Issues](https://github.com/zentala/zntl-tray/issues) |

## Frequently Asked Questions

**Q: Does zntlDesk collect my data?**
A: No. All data stays on your computer. See [Privacy Policy](./PRIVACY.md).

**Q: How do updates work?**
A: Automatic every 24 hours, with an option to check manually. See [Updates Guide](./USER_UPDATES.md).

**Q: Where is my session data stored?**
A: In `C:\Users\[YourUsername]\AppData\Local\zntlDesk\`. See [Support Guide](./USER_SUPPORT.md#accessing-application-data).

**Q: Can I export my data?**
A: Yes! Open Settings → Data Management → Export All Data. See [Privacy Policy](./PRIVACY.md#exporting-your-data).

**Q: My sensor isn't detected. What do I do?**
A: See [Support Guide - Sensor not detected](./USER_SUPPORT.md#issue-sensor-not-detected).

**Q: How do I uninstall?**
A: See [Installation Guide - Uninstall](./USER_INSTALL.md#uninstall).

## Getting Help

Stuck on something? Here's where to look:

1. **Search this documentation** — Most questions are answered here
2. **Check [Support Guide](./USER_SUPPORT.md)** — Troubleshooting steps
3. **Report on GitHub** — [Create an issue](https://github.com/zentala/zntl-tray/issues)

When reporting a problem:
- Describe what you were doing
- What happened (include error messages)
- Include your Windows version
- Attach relevant logs from `%APPDATA%\Local\zntlDesk\logs\`

## What is zntlDesk?

zntlDesk is an app that helps you maintain healthy posture and take regular breaks. It:

- **Detects desk position** via a height sensor (sitting vs standing)
- **Tracks session duration** and alerts you after 40 minutes of sitting
- **Records break history** so you can see your patterns
- **Keeps all data local** — nothing leaves your computer

## System Requirements

- **Windows 10 or later** (Windows 11 recommended)
- **~150 MB disk space**
- **USB connection** for sensor (optional, but recommended)

## Get Started

1. **[Install zntlDesk](./USER_INSTALL.md)**
2. **Connect your sensor** (if you have one)
3. **Configure your preferences** (see Support Guide → Settings)
4. **Start tracking** and enjoy healthier work habits!

---

**Questions or issues?**
→ Check the relevant guide above
→ Browse [GitHub Issues](https://github.com/zentala/zntl-tray/issues)
→ Reach out on GitHub with your question
