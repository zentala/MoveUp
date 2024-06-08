import { app, BrowserWindow, screen } from 'electron';
import * as path from 'path';
import * as url from 'url';

let mainWindow: BrowserWindow | null;
let floatingWindow: BrowserWindow | null;

const createWindow = () => {
  const display = screen.getPrimaryDisplay();
  const width = display.bounds.width;
  const height = display.bounds.height;

  mainWindow = new BrowserWindow({
    width: 800,
    height: 600,
    webPreferences: {
      preload: path.join(__dirname, '../renderer/preload.js'), // zmieniona ścieżka
    },
  });

  floatingWindow = new BrowserWindow({
    width: 400,
    height: 300,
    x: width - 400,
    y: height - 300,
    frame: false,
    transparent: true,
    alwaysOnTop: true,
    webPreferences: {
      preload: path.join(__dirname, '../renderer/preload.js'), // zmieniona ścieżka
    },
  });

  if (process.env.VITE_DEV_SERVER_URL) {
    mainWindow.loadURL(process.env.VITE_DEV_SERVER_URL);
    floatingWindow.loadURL(process.env.VITE_DEV_SERVER_URL);
  } else {
    mainWindow.loadFile(path.join(__dirname, '../renderer/index.html'));
    floatingWindow.loadFile(path.join(__dirname, '../renderer/index.html'));
  }

  mainWindow.on('closed', () => {
    mainWindow = null;
  });

  floatingWindow.on('closed', () => {
    floatingWindow = null;
  });
};

app.on('ready', createWindow);

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.on('activate', () => {
  if (mainWindow === null) {
    createWindow();
  }
});

if (module.hot) {
  module.hot.accept();
}
