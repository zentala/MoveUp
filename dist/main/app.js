"use strict";
const electron = require("electron");
const path = require("path");
function _interopNamespaceDefault(e) {
  const n = Object.create(null, { [Symbol.toStringTag]: { value: "Module" } });
  if (e) {
    for (const k in e) {
      if (k !== "default") {
        const d = Object.getOwnPropertyDescriptor(e, k);
        Object.defineProperty(n, k, d.get ? d : {
          enumerable: true,
          get: () => e[k]
        });
      }
    }
  }
  n.default = e;
  return Object.freeze(n);
}
const path__namespace = /* @__PURE__ */ _interopNamespaceDefault(path);
let mainWindow;
let floatingWindow;
const createWindow = () => {
  const display = electron.screen.getPrimaryDisplay();
  const width = display.bounds.width;
  const height = display.bounds.height;
  mainWindow = new electron.BrowserWindow({
    width: 800,
    height: 600,
    webPreferences: {
      preload: path__namespace.join(__dirname, "../renderer/preload.js")
      // zmieniona ścieżka
    }
  });
  floatingWindow = new electron.BrowserWindow({
    width: 400,
    height: 300,
    x: width - 400,
    y: height - 300,
    frame: false,
    transparent: true,
    alwaysOnTop: true,
    webPreferences: {
      preload: path__namespace.join(__dirname, "../renderer/preload.js")
      // zmieniona ścieżka
    }
  });
  if (process.env.VITE_DEV_SERVER_URL) {
    mainWindow.loadURL(process.env.VITE_DEV_SERVER_URL);
    floatingWindow.loadURL(process.env.VITE_DEV_SERVER_URL);
  } else {
    mainWindow.loadFile(path__namespace.join(__dirname, "../renderer/index.html"));
    floatingWindow.loadFile(path__namespace.join(__dirname, "../renderer/index.html"));
  }
  mainWindow.on("closed", () => {
    mainWindow = null;
  });
  floatingWindow.on("closed", () => {
    floatingWindow = null;
  });
};
electron.app.on("ready", createWindow);
electron.app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    electron.app.quit();
  }
});
electron.app.on("activate", () => {
  if (mainWindow === null) {
    createWindow();
  }
});
if (module.hot) {
  module.hot.accept();
}
