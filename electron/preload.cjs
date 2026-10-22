const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('loomAPI', {
  selectDirectory: (title) => ipcRenderer.invoke('select-directory', title),
  runLoom: (options) => ipcRenderer.invoke('run-loom', options),
  stopLoom: () => ipcRenderer.invoke('stop-loom'),
  openVault: (vaultPath) => ipcRenderer.invoke('open-vault', vaultPath),
  checkUnityProject: (dirPath) => ipcRenderer.invoke('check-unity-project', dirPath),
  checkRobloxProject: (dirPath) => ipcRenderer.invoke('check-roblox-project', dirPath),
  onLog: (callback) => {
    const handler = (_event, data) => callback(data);
    ipcRenderer.on('loom-log', handler);
    return () => ipcRenderer.removeListener('loom-log', handler);
  },
  onDone: (callback) => {
    const handler = (_event, result) => callback(result);
    ipcRenderer.on('loom-done', handler);
    return () => ipcRenderer.removeListener('loom-done', handler);
  },
});
