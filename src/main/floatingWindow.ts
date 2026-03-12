import { BrowserWindow, screen } from 'electron';
import path from 'path';

let floatingWindowInstance: BrowserWindow | null = null;

export function createFloatingWindow(): BrowserWindow {
  if (floatingWindowInstance && !floatingWindowInstance.isDestroyed()) {
    floatingWindowInstance.focus();
    return floatingWindowInstance;
  }

  // Get the display dimensions
  const display = screen.getPrimaryDisplay();
  const { width, height } = display.workAreaSize;
  const windowBounds = { width: 300, height: 200 };

  const x = width - windowBounds.width;
  const y = height - windowBounds.height;

  floatingWindowInstance = new BrowserWindow({
    width: windowBounds.width,
    height: windowBounds.height,
    x: x,
    y: y,
    frame: false,
    alwaysOnTop: true,
    skipTaskbar: true,
    webPreferences: {
      preload: path.join(__dirname, '../renderer/preload.js')
    }
  });

  floatingWindowInstance.loadFile(path.join(__dirname, '../renderer/floating.html'));

  floatingWindowInstance.on('blur', () => {
    floatingWindowInstance?.close();
  });

  floatingWindowInstance.on('closed', () => {
    floatingWindowInstance = null;
  });

  return floatingWindowInstance;
}
