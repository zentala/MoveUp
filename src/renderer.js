const { ipcRenderer } = require('electron');

document.addEventListener('DOMContentLoaded', () => {
  const triggerButton = document.getElementById('triggerFloatingWindow');
  if (triggerButton) {
    triggerButton.addEventListener('click', () => {
      ipcRenderer.send('show-floating-window');
    });
  }
});
