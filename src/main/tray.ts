import { app, Tray, Menu, nativeTheme, BrowserWindow } from 'electron';
import { createFloatingWindow } from './floatingWindow';
import { getIcon } from './icons';

let tray: Tray;

export function createTray(mainWindow: BrowserWindow): Tray {
  const isDarkMode = nativeTheme.shouldUseDarkColors;
  tray = new Tray(getIcon(isDarkMode));

  const contextMenu = Menu.buildFromTemplate([
    {
      label: 'Dashboard',
      click: () => {
        if (mainWindow) {
          mainWindow.show();
        }
      }
    },
    {
      label: 'Quit',
      click: () => {
        app.quit();
      }
    }
  ]);

  tray.setToolTip('MoveUp');
  tray.setContextMenu(contextMenu);

  tray.on('click', () => {
    createFloatingWindow();
  });

  nativeTheme.on('updated', () => {
    const newIsDarkMode = nativeTheme.shouldUseDarkColors;
    tray.setImage(getIcon(newIsDarkMode));
    if (mainWindow) {
      mainWindow.webContents.send('update-icon', newIsDarkMode ? 'white' : 'black');
    }
  });

  return tray;
}
