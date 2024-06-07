import { app, ipcMain, BrowserWindow } from 'electron';
import { createMainWindow } from './mainWindow';
import { createTray } from './tray';
import { generateIcons } from './icons';
import { createFloatingWindow } from './floatingWindow';

let mainWindow: BrowserWindow | null;

app.on('ready', async () => {
  await generateIcons();
  mainWindow = createMainWindow();
  createTray(mainWindow);
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
