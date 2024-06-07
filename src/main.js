const { app, BrowserWindow, Tray, Menu, ipcMain, nativeImage } = require('electron');
const path = require('path');
const fs = require('fs');
const sharp = require('sharp');


let mainWindow;
let tray;

function createMainWindow() {
  mainWindow = new BrowserWindow({
    width: 800,
    height: 600,
    webPreferences: {
      preload: path.join(__dirname, 'renderer.js')
    }
  });

  mainWindow.loadFile(path.join(__dirname, 'index.html'));
}

async function createTray() {
  const trayIconPath = path.join(__dirname, 'icon.svg');
  let trayIconSVG = fs.readFileSync(trayIconPath, 'utf8');
  
  // Add fill and stroke attributes to all path elements
  trayIconSVG = trayIconSVG.replace(/<path/g, '<path fill="#FFFFFF" stroke="#FFFFFF"');

  const trayIconPNG = await sharp(Buffer.from(trayIconSVG))
    .png()
    .resize(16, 16)
    .toBuffer();

  const trayIcon = nativeImage.createFromBuffer(trayIconPNG);

  tray = new Tray(trayIcon);
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
}


function createFloatingWindow() {
  const { screen } = require('electron');

  // Get the display dimensions
  const display = screen.getPrimaryDisplay();
  const { width, height } = display.workAreaSize;
  const windowBounds = { width: 300, height: 200 }; // Define the size of the floating window

  // Calculate the position of the floating window in the bottom right corner
  const x = width - windowBounds.width - 10; // 10 pixels from the right edge
  const y = height - windowBounds.height - 10; // 10 pixels from the bottom edge

  const floatingWindow = new BrowserWindow({
    width: windowBounds.width,
    height: windowBounds.height,
    x: x,
    y: y,
    frame: false,
    alwaysOnTop: true,
    skipTaskbar: true,
    webPreferences: {
      preload: path.join(__dirname, 'renderer.js')
    }
  });

  floatingWindow.loadFile(path.join(__dirname, 'floating.html'));

  floatingWindow.on('blur', () => {
    floatingWindow.close();
  });
}




app.on('ready', () => {
  createMainWindow();
  createTray();
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.on('activate', () => {
  if (BrowserWindow.getAllWindows().length === 0) {
    createMainWindow();
  }
});



ipcMain.on('show-floating-window', () => {
  createFloatingWindow();
});
