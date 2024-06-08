import { BrowserWindow, nativeTheme } from 'electron';
import path from 'path';
import { getIcon } from './icons';

export function createMainWindow(): BrowserWindow {
  const isDarkMode = nativeTheme.shouldUseDarkColors;
  console.log('Selected Icon:', getIcon(isDarkMode));

  const mainWindow = new BrowserWindow({
    width: 800,
    height: 600,
    icon: getIcon(isDarkMode),
    webPreferences: {
      preload: path.join(__dirname, '../renderer/preload.ts'),
    },
  });

  mainWindow.loadFile(path.join(__dirname, '../renderer/index.html'));

  return mainWindow;
}
