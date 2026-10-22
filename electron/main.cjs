const { app, BrowserWindow, ipcMain, dialog, shell } = require('electron');
const path = require('node:path');
const fs = require('node:fs');
const { spawn } = require('node:child_process');

let mainWindow = null;
let activeLoomProcess = null;

function resolveLoomBinary() {
  const isWindows = process.platform === 'win32';
  const binaryName = isWindows ? 'loom.exe' : 'loom';

  if (process.resourcesPath) {
    const candidatePaths = [
      path.join(process.resourcesPath, 'bin', binaryName),
      path.join(process.resourcesPath, binaryName),
      path.join(process.resourcesPath, 'app.asar.unpacked', 'bin', binaryName),
    ];
    for (const p of candidatePaths) {
      if (fs.existsSync(p)) {
        if (!isWindows) {
          try {
            fs.chmodSync(p, 0o755);
          } catch {}
        }
        return p;
      }
    }
  }

  const localBuild = path.join(__dirname, '..', 'target', 'release', binaryName);
  if (fs.existsSync(localBuild)) {
    return localBuild;
  }

  const binBuild = path.join(__dirname, '..', 'bin', isWindows ? 'loom-bin.exe' : 'loom-bin');
  if (fs.existsSync(binBuild)) {
    return binBuild;
  }

  return 'loom';
}

function createWindow() {
  const iconPath = path.join(__dirname, '..', 'build', process.platform === 'win32' ? 'icon.ico' : 'icon.png');
  mainWindow = new BrowserWindow({
    width: 1040,
    height: 820,
    minWidth: 840,
    minHeight: 640,
    backgroundColor: '#0f1117',
    icon: fs.existsSync(iconPath) ? iconPath : undefined,
    title: 'Loom — Codebase to Obsidian Vault',
    titleBarStyle: process.platform === 'darwin' ? 'hiddenInset' : 'default',
    webPreferences: {
      preload: path.join(__dirname, 'preload.cjs'),
      nodeIntegration: false,
      contextIsolation: true,
      sandbox: false,
    },
  });

  mainWindow.loadFile(path.join(__dirname, 'renderer', 'index.html'));

  mainWindow.on('closed', () => {
    mainWindow = null;
    if (activeLoomProcess) {
      activeLoomProcess.kill('SIGINT');
      activeLoomProcess = null;
    }
  });
}

app.whenReady().then(() => {
  createWindow();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

app.on('window-all-closed', () => {
  if (activeLoomProcess) {
    activeLoomProcess.kill('SIGINT');
    activeLoomProcess = null;
  }
  if (process.platform !== 'darwin') app.quit();
});

ipcMain.handle('select-directory', async (_event, title) => {
  if (!mainWindow) return null;
  const result = await dialog.showOpenDialog(mainWindow, {
    title: title || 'Select Directory',
    properties: ['openDirectory', 'createDirectory'],
  });

  if (result.canceled || result.filePaths.length === 0) {
    return null;
  }
  return result.filePaths[0];
});

ipcMain.handle('check-unity-project', async (_event, dirPath) => {
  if (!dirPath || typeof dirPath !== 'string') return false;
  try {
    const assetsPath = path.join(dirPath, 'Assets');
    const projectSettingsPath = path.join(dirPath, 'ProjectSettings');
    return fs.existsSync(assetsPath) || fs.existsSync(projectSettingsPath);
  } catch {
    return false;
  }
});

ipcMain.handle('check-roblox-project', async (_event, dirPath) => {
  if (!dirPath || typeof dirPath !== 'string') return false;
  try {
    const defaultProject = path.join(dirPath, 'default.project.json');
    const wallyToml = path.join(dirPath, 'wally.toml');
    if (fs.existsSync(defaultProject) || fs.existsSync(wallyToml)) return true;

    const entries = fs.readdirSync(dirPath);
    return entries.some(file => file.endsWith('.project.json') || file.endsWith('.luau') || file.endsWith('.rbxmx') || file.endsWith('.rbxlx'));
  } catch {
    return false;
  }
});

ipcMain.handle('run-loom', async (_event, options) => {
  if (activeLoomProcess) {
    return { success: false, error: 'A Loom indexing run is already in progress.' };
  }

  const { projectPath, outputPath, name, unity, roblox, watch, verbose } = options;

  if (!projectPath) {
    return { success: false, error: 'Please choose a project directory.' };
  }

  const binary = resolveLoomBinary();
  const args = [projectPath];

  if (outputPath && outputPath.trim().length > 0) {
    args.push('-o', outputPath.trim());
  } else if (name && name.trim().length > 0) {
    args.push('-n', name.trim());
  }

  if (unity) {
    args.push('--unity');
  }

  if (roblox) {
    args.push('--roblox');
  }

  if (watch) {
    args.push('--watch');
  }

  if (verbose) {
    args.push('--verbose');
  }

  try {
    const child = spawn(binary, args, {
      cwd: projectPath,
      env: { ...process.env, CLICOLOR_FORCE: '1' },
    });

    activeLoomProcess = child;

    child.stdout.on('data', (chunk) => {
      if (mainWindow) {
        mainWindow.webContents.send('loom-log', { type: 'stdout', text: chunk.toString('utf8') });
      }
    });

    child.stderr.on('data', (chunk) => {
      if (mainWindow) {
        mainWindow.webContents.send('loom-log', { type: 'stderr', text: chunk.toString('utf8') });
      }
    });

    child.on('close', (code) => {
      activeLoomProcess = null;
      if (mainWindow) {
        mainWindow.webContents.send('loom-done', { code });
      }
    });

    child.on('error', (err) => {
      activeLoomProcess = null;
      if (mainWindow) {
        mainWindow.webContents.send('loom-log', {
          type: 'stderr',
          text: `Failed to launch Loom binary (${binary}): ${err.message}\n`,
        });
        mainWindow.webContents.send('loom-done', { code: 1, error: err.message });
      }
    });

    return { success: true };
  } catch (err) {
    activeLoomProcess = null;
    return { success: false, error: err.message };
  }
});

ipcMain.handle('stop-loom', async () => {
  if (activeLoomProcess) {
    activeLoomProcess.kill('SIGINT');
    setTimeout(() => {
      if (activeLoomProcess) {
        activeLoomProcess.kill('SIGKILL');
        activeLoomProcess = null;
      }
    }, 1500);
    return true;
  }
  return false;
});

ipcMain.handle('open-vault', async (_event, vaultPath) => {
  if (!vaultPath) return false;

  try {
    const obsidianUri = `obsidian://open?path=${encodeURIComponent(vaultPath)}`;
    await shell.openExternal(obsidianUri);
    return true;
  } catch {
    await shell.openPath(vaultPath);
    return true;
  }
});
