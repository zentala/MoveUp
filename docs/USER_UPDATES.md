# Updates and Upgrades

zntlDesk keeps itself up-to-date automatically. This guide explains how the update system works and how to manage it.

## Automatic Updates

zntlDesk checks for updates **every 24 hours** while the application is running.

### When an Update is Available

1. **Notification appears** in the system tray (bottom-right of screen)
2. **Message says:** "A new version of zntlDesk is available"
3. **Your choices:**
   - **Install Later** — Update will be installed next time you restart the app
   - **Install Now** — Update starts immediately and restarts the application

### During Update

- Your session data is **never lost** during updates
- The update is downloaded in the background
- Old data remains in `C:\Users\[YourUsername]\AppData\Local\zntlDesk\`

### After Update

- The application restarts automatically
- Your previous session state (sitting time, break credits) is preserved
- Release notes are shown in the app

## Manual Update Check

To check for updates without waiting 24 hours:

1. Click the zntlDesk system tray icon
2. Select **Check for Updates**
3. The app will check GitHub Releases immediately

## Release Notes

To see what's new in the latest version:

1. Open zntlDesk
2. Go to **Help** → **About**
3. Click **View Release Notes**

Or visit: [GitHub Releases](https://github.com/zentala/zntl-tray/releases)

## Disabling Auto-Updates

If you prefer to manually control updates:

1. Open zntlDesk
2. Go to **Settings** → **General**
3. Toggle off **Check for updates automatically**
4. You can still use **Check for Updates** manually anytime

## Rollback (Going Back to Previous Version)

If an update causes problems:

1. **Uninstall zntlDesk** (see [Installation Guide](./USER_INSTALL.md#uninstall))
2. **Download the previous version** from [GitHub Releases](https://github.com/zentala/zntl-tray/releases)
3. **Install the older version** using the downloaded `.exe`

Your session data remains in `AppData\Local\zntlDesk\` and will be accessible by the older version.

## Update History

Your update history is available in the application logs:
- Location: `C:\Users\[YourUsername]\AppData\Local\zntlDesk\logs\`
- File: `zntlDesk.log`

To view logs:
1. Open File Explorer
2. Type in the address bar: `%APPDATA%\Local\zntlDesk\logs\`
3. Open `zntlDesk.log` with Notepad

## Troubleshooting Updates

### "Update fails to download"

- Check your internet connection
- Try checking updates again later
- If the problem persists, [report an issue](https://github.com/zentala/zntl-tray/issues)

### "App crashes after update"

- Try restarting your computer
- If the crash persists, rollback to the previous version (see above)
- Report the issue on [GitHub Issues](https://github.com/zentala/zntl-tray/issues) with logs attached

### "Update notification keeps appearing"

- Click **Install Now** to apply the update
- Or disable auto-updates in Settings (see above)
- Restart zntlDesk

## Data Security During Updates

- Your session history is **never sent** to any server
- Updates only check version numbers from GitHub
- No personal data is accessed or transmitted

See **[Privacy](./PRIVACY.md)** for full details.
