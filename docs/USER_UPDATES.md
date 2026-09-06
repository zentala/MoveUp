# Updates

MoveUp has **no automatic updater**. It never phones home to check for a new
version, and it will never install one on its own. Updating is a manual step:
you download the new installer and run it over the current install.

If you read an older copy of this page describing a 24-hour update check, a
tray "Check for Updates" entry, or an auto-update toggle in Settings — none of
that exists. The app has no updater plugin and no version-check code.

## How to update

1. Open the releases page: <https://github.com/zentala/MoveUp/releases>
2. Compare the newest tag with the version you run. To see your version, open
   the MoveUp window — the version is shown in the app; you can also read
   `version` in `src-tauri/tauri.conf.json` if you build from source.
3. Download the installer asset for the newest release (`.exe`, Windows).
4. Close MoveUp — right-click the tray icon and quit. An update cannot replace
   a running `desk.exe`.
5. Run the downloaded installer. It installs over the existing copy in
   `%LOCALAPPDATA%\MoveUp\`; you do not need to uninstall first.
6. Start MoveUp again from the Start menu.

## What survives an update

Your data lives outside the program folder, so installing a new version does
not touch it:

- session history and settings — `%APPDATA%\io.zntl.desk\`
- logs — `%APPDATA%\io.zntl.desk\logs\YYYY-MM-DD\`

Sitting/standing counters, break credit and notification state for the current
day are stored there too and are read back when the app restarts.

## Going back to an older version

1. Close MoveUp.
2. Download the older release's installer from the same releases page —
   every past release stays available.
3. Run it. It overwrites the newer build.

Your data in `%APPDATA%\io.zntl.desk\` stays where it is and the older build
reads it.

If you would rather start from a clean install, uninstall first (see
[Installation Guide](./USER_INSTALL.md#uninstall)) and then run the older
installer.

## Being told about new versions

Since the app does not check for updates, use GitHub instead:

- Open <https://github.com/zentala/MoveUp>, click **Watch** →
  **Custom** → **Releases**. GitHub then emails you on every new release.
- Or subscribe to the releases feed:
  <https://github.com/zentala/MoveUp/releases.atom>

## Release notes

Release notes are written on each GitHub release. There is no in-app "What's
new" screen.

## Troubleshooting

### The installer says the file is in use

MoveUp is still running. Quit it from the tray icon (and check Task Manager
for `desk.exe`), then run the installer again.

### Windows SmartScreen warns about an unknown publisher

Release builds are not code-signed yet. SmartScreen shows "Windows protected
your PC" for unsigned installers. Choose **More info** → **Run anyway** if you
trust the download, or build from source instead.

### The new version behaves worse than the old one

Go back to the previous release as described above, and please
[report the issue](https://github.com/zentala/MoveUp/issues) with the log files
from `%APPDATA%\io.zntl.desk\logs\`.

## What is sent during an update

Nothing by the app. You download the installer from GitHub yourself; MoveUp
itself makes no version-check request. See **[Privacy](./PRIVACY.md)** for what
the app does and does not send while it runs.

## Will there be an automatic updater?

It is on the backlog, not in the product. It needs code signing first — an
unsigned auto-update would install unverified binaries. Tracked in
[`.plan/BACKLOG.md`](../.plan/BACKLOG.md).
