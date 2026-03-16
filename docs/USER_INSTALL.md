# Installation Guide

Welcome to zntlDesk! This guide will walk you through installing the application on your Windows computer.

## System Requirements

- **Windows 10 or later** (Windows 11 recommended)
- **Disk space**: ~150 MB for application and data
- **RAM**: 512 MB minimum (1 GB recommended)
- **USB connection**: For sensor hardware (if using a desk height sensor)

## Download

Download the latest installer from:
[GitHub Releases](https://github.com/zentala/zntl-tray/releases) → Look for `zntlDesk-Setup-*.exe`

## Step-by-Step Installation

### 1. Run the Installer

Double-click `zntlDesk-Setup-*.exe` to start the installation wizard.

### 2. Accept License (if shown)

Read and accept the license agreement to proceed.

### 3. Choose Installation Location

The installer will suggest a default location. You can:
- Accept the default (recommended)
- Click "Browse" to choose a custom folder

**Note:** The application requires administrator privileges to:
- Access keyboard/mouse activity
- Create system tray icon
- Enable auto-start on login

### 4. Installation Progress

The installer will copy files to your computer. This takes 30-60 seconds.

### 5. Create Start Menu Shortcut

You can choose to:
- Add a shortcut to your Start Menu (recommended)
- Create a desktop shortcut

### 6. Launch Application

After installation completes:
- Check "Launch zntlDesk" to start immediately
- Or find "zntlDesk" in your Start Menu

## First Run

On first launch, zntlDesk will:
1. Detect your sensor hardware (if connected via USB)
2. Create a database to store your data
3. Ask you to calibrate desk height (if sensor detected)

## Data Storage

Your session data and preferences are stored in:
```
C:\Users\[YourUsername]\AppData\Local\zntlDesk\
```

This includes:
- SQLite database with sitting/standing sessions
- User preferences and settings
- Application logs

## Uninstall

To remove zntlDesk:

1. Go to **Settings** → **Apps** → **Apps & Features**
2. Find **zntlDesk** in the list
3. Click it and select **Uninstall**
4. Follow the uninstall wizard

**Note:** Your session data will NOT be deleted. To remove it manually:
1. Close zntlDesk
2. Delete the folder: `C:\Users\[YourUsername]\AppData\Local\zntlDesk\`

## Troubleshooting Installation

### "Administrator privileges required"

Right-click the installer and select **Run as administrator**.

### "Cannot find sensor hardware"

If you have a desk height sensor but zntlDesk doesn't detect it:
1. Check the USB cable connection
2. Open Device Manager (search in Start Menu)
3. Look for "XIAO ESP32" or "COM3" in ports
4. If missing, install USB drivers from Seeed Studio

### "Installer won't run"

- Windows Defender may block it. Click "More info" → "Run anyway"
- Disable antivirus temporarily
- Try downloading again from GitHub

## Next Steps

After installation, see:
- **[Settings](./USER_SUPPORT.md#settings)** — Configure desk height, break intervals
- **[Updates](./USER_UPDATES.md)** — How auto-updates work
- **[Privacy](./PRIVACY.md)** — Your data stays local

## Getting Help

If installation fails or you need help:
1. Check **[Support](./USER_SUPPORT.md)** for common issues
2. Report a problem on [GitHub Issues](https://github.com/zentala/zntl-tray/issues)
