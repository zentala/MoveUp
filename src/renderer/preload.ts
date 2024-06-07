const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('api', {
  send: (channel: string, data: any) => ipcRenderer.send(channel, data),
  receive: (channel: string, func: any) => ipcRenderer.on(channel, (event, ...args) => func(...args))
});
