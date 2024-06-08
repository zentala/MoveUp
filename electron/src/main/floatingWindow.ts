import { BrowserWindow, screen } from 'electron';
import path from 'path';

export function createFloatingWindow(): BrowserWindow {
  // Get the display dimensions
  const display = screen.getPrimaryDisplay();
  const { width, height } = display.workAreaSize;
  const windowBounds = { width: 300, height: 200 }; // Define the size of the floating window

  // Calculate the position of the floating window in the bottom right corner
  // const x = width - windowBounds.width - 10; // 10 pixels from the right edge
  // const y = height - windowBounds.height - 10; // 10 pixels from the bottom edge
  const x = width - windowBounds.width;
  const y = height - windowBounds.height;

  const floatingWindow = new BrowserWindow({
    width: windowBounds.width,
    height: windowBounds.height,
    x: x,
    y: y,
    frame: false,
    alwaysOnTop: true,
    skipTaskbar: true,
    webPreferences: {
      preload: path.join(__dirname, '../renderer/preload.ts'),
    },
  });

  floatingWindow.loadFile(path.join(__dirname, '../renderer/floating.html'));

  floatingWindow.on('blur', () => {
    floatingWindow.close();
  });

  return floatingWindow;
}
