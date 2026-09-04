import { app, ipcMain, BrowserWindow } from 'electron';
import { createMainWindow } from './mainWindow';
import { createTray } from './tray';
import { generateIcons } from './icons';
import { createFloatingWindow } from './floatingWindow';

let mainWindow: BrowserWindow | null;
const managedByPm3 = process.env.MOVEUP_PM3 === 'true';
const launchedInBackground = process.argv.includes('--hidden') || process.argv.includes('--autostart');

// Keep the desktop app available for users who do not run it under PM3.
// PM3 sets MOVEUP_PM3=true so its supervised process does not register a
// second Windows logon entry and start a duplicate Electron instance.
function configureStartup(): void {
  if (managedByPm3 || !app.isPackaged) {
    return;
  }

  app.setLoginItemSettings({
    openAtLogin: true,
    openAsHidden: true,
    args: ['--autostart'],
  });
}

if (!app.requestSingleInstanceLock()) {
  app.quit();
} else {
  app.on('second-instance', () => {
    if (mainWindow) {
      if (mainWindow.isMinimized()) mainWindow.restore();
      mainWindow.show();
      mainWindow.focus();
    }
  });
}

app.on('ready', async () => {
  configureStartup();
  await generateIcons();
  mainWindow = createMainWindow();
  createTray(mainWindow);

  if (launchedInBackground) {
    mainWindow.hide();
  }
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.on('activate', () => {
  if (BrowserWindow.getAllWindows().length === 0) {
    mainWindow = createMainWindow();
  }
});

ipcMain.on('show-floating-window', () => {
  createFloatingWindow();
});
