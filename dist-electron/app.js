"use strict";
const electron = require("electron");
const path = require("path");
const iconSVG = '<?xml version="1.0" encoding="utf-8" ?>\r\n<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="1024" height="1024">\r\n	<path transform="scale(1.6 1.6)" d="M303.23 146.811C297.93 151.377 290.665 161.93 284.625 164.349C279.859 166.259 271.813 164.29 269.862 159.042C267.181 151.832 273.459 146.805 277.504 141.768C287.924 128.793 298.515 115.961 308.888 102.946C311.896 99.1722 313.777 95.4293 318.561 93.7936L320.846 93.7031C327.095 93.6851 330.768 100.76 334.163 105.059C343.026 116.279 352.002 127.488 361.109 138.508C364.247 142.305 367.959 146.012 370.672 150.103C375.842 157.896 369.511 165.866 360.938 165.299C353.137 164.783 343.889 151.504 338.415 145.455C338.526 169.67 338.384 193.889 338.405 218.105C338.417 231.245 340.486 241.017 324.795 246.093L321.46 246.507C307.089 247.03 303.427 236.659 303.424 225.123C303.417 199.42 304.256 172.416 303.23 146.811Z"/>\r\n	<path transform="scale(1.6 1.6)" d="M138.855 317.225C114.879 317.295 91.972 318.438 91.1192 287.69C90.5147 265.893 91.1985 244.062 91.0572 222.255C90.9567 206.754 93.8034 192.605 112.443 189.093C116.644 188.301 121.475 188.732 125.764 188.731C166.49 188.724 208.273 189.69 248.917 188.589L284.976 189.049C285.817 202.626 283.553 225.588 286.603 237.656C295.257 271.896 347.525 272.885 355.675 235.388C358.203 223.758 355.665 201.961 356.487 188.854L359.777 188.766C410.204 187.713 461.559 188.751 512.064 188.722C517.589 188.719 523.827 188.143 529.27 188.954C540.913 190.69 550.163 202.245 550.282 213.772C550.528 237.72 550.404 261.69 550.294 285.641C550.145 318.342 529.021 317.338 502.799 317.194C502.558 387.087 503.198 456.987 502.773 526.877C502.754 529.928 502.861 533.24 502.058 536.205L501.91 536.719C494.784 562.326 455.291 561.74 450.981 534.823C449.746 527.111 450.311 518.874 450.308 511.077C450.287 452.332 450.301 393.588 450.293 334.844L450.281 317.526L191.261 317.228C191.56 386.112 191.428 455.012 191.271 523.897C191.265 526.759 191.393 529.698 191.045 532.539L190.97 533.125C187.302 563.001 143.868 562.879 139.429 533.827C138.342 526.708 138.953 518.885 138.951 511.686C138.934 447.128 140.066 381.711 138.855 317.225Z"/>\r\n</svg>\r\n';
let trayIconWhite;
let trayIconBlack;
async function generateIcons() {
  const whiteIconSVG = iconSVG.replace(/<path/g, '<path fill="#FFFFFF" stroke="#FFFFFF"');
  const blackIconSVG = iconSVG.replace(/<path/g, '<path fill="#000000" stroke="#000000"');
  const asImage = (svg) => electron.nativeImage.createFromDataURL(`data:image/svg+xml;base64,${Buffer.from(svg).toString("base64")}`);
  trayIconWhite = asImage(whiteIconSVG);
  trayIconBlack = asImage(blackIconSVG);
}
function getIcon(isDarkMode) {
  return isDarkMode ? trayIconWhite : trayIconBlack;
}
function createMainWindow() {
  const isDarkMode = electron.nativeTheme.shouldUseDarkColors;
  console.log("Selected Icon:", getIcon(isDarkMode));
  const mainWindow2 = new electron.BrowserWindow({
    width: 800,
    height: 600,
    icon: getIcon(isDarkMode),
    webPreferences: {
      preload: path.join(__dirname, "preload.js")
    }
  });
  mainWindow2.loadFile(path.join(__dirname, "../renderer/index.html"));
  return mainWindow2;
}
let floatingWindowInstance = null;
function createFloatingWindow() {
  if (floatingWindowInstance && !floatingWindowInstance.isDestroyed()) {
    floatingWindowInstance.focus();
    return floatingWindowInstance;
  }
  const display = electron.screen.getPrimaryDisplay();
  const { width, height } = display.workAreaSize;
  const windowBounds = { width: 300, height: 200 };
  const x = width - windowBounds.width;
  const y = height - windowBounds.height;
  floatingWindowInstance = new electron.BrowserWindow({
    width: windowBounds.width,
    height: windowBounds.height,
    x,
    y,
    frame: false,
    alwaysOnTop: true,
    skipTaskbar: true,
    webPreferences: {
      preload: path.join(__dirname, "../renderer/preload.js")
    }
  });
  floatingWindowInstance.loadFile(path.join(__dirname, "../renderer/floating.html"));
  floatingWindowInstance.on("blur", () => {
    floatingWindowInstance == null ? void 0 : floatingWindowInstance.close();
  });
  floatingWindowInstance.on("closed", () => {
    floatingWindowInstance = null;
  });
  return floatingWindowInstance;
}
let tray;
function createTray(mainWindow2) {
  const isDarkMode = electron.nativeTheme.shouldUseDarkColors;
  tray = new electron.Tray(getIcon(isDarkMode));
  const contextMenu = electron.Menu.buildFromTemplate([
    {
      label: "Dashboard",
      click: () => {
        if (mainWindow2) {
          mainWindow2.show();
        }
      }
    },
    {
      label: "Quit",
      click: () => {
        electron.app.quit();
      }
    }
  ]);
  tray.setToolTip("MoveUp");
  tray.setContextMenu(contextMenu);
  tray.on("click", () => {
    createFloatingWindow();
  });
  electron.nativeTheme.on("updated", () => {
    const newIsDarkMode = electron.nativeTheme.shouldUseDarkColors;
    tray.setImage(getIcon(newIsDarkMode));
    if (mainWindow2) {
      mainWindow2.webContents.send("update-icon", newIsDarkMode ? "white" : "black");
    }
  });
  return tray;
}
let mainWindow;
const managedByPm3 = process.env.MOVEUP_PM3 === "true";
const launchedInBackground = process.argv.includes("--hidden") || process.argv.includes("--autostart");
function configureStartup() {
  if (managedByPm3 || !electron.app.isPackaged) {
    return;
  }
  electron.app.setLoginItemSettings({
    openAtLogin: true,
    openAsHidden: true,
    args: ["--autostart"]
  });
}
if (!electron.app.requestSingleInstanceLock()) {
  electron.app.quit();
} else {
  electron.app.on("second-instance", () => {
    if (mainWindow) {
      if (mainWindow.isMinimized())
        mainWindow.restore();
      mainWindow.show();
      mainWindow.focus();
    }
  });
}
electron.app.on("ready", async () => {
  configureStartup();
  await generateIcons();
  mainWindow = createMainWindow();
  createTray(mainWindow);
  if (launchedInBackground) {
    mainWindow.hide();
  }
});
electron.app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    electron.app.quit();
  }
});
electron.app.on("activate", () => {
  if (electron.BrowserWindow.getAllWindows().length === 0) {
    mainWindow = createMainWindow();
  }
});
electron.ipcMain.on("show-floating-window", () => {
  createFloatingWindow();
});
