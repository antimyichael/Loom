#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import https from 'node:https';
import { execSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const binDir = __dirname;
const REPO = 'loom-code/loom';
const VERSION = 'v0.2.0';

export function getPlatformBinaryName() {
  const platform = process.platform;
  const arch = process.arch;

  let osStr = '';
  if (platform === 'darwin') osStr = 'macos';
  else if (platform === 'linux') osStr = 'linux';
  else if (platform === 'win32') osStr = 'windows';
  else throw new Error(`Unsupported OS platform: ${platform}`);

  let archStr = '';
  if (arch === 'x64') archStr = 'x86_64';
  else if (arch === 'arm64') archStr = 'aarch64';
  else throw new Error(`Unsupported CPU architecture: ${arch}`);

  return `loom-${osStr}-${archStr}`;
}

export function getBinaryPath() {
  const isWindows = process.platform === 'win32';
  return path.join(binDir, isWindows ? 'loom-bin.exe' : 'loom-bin');
}

export async function downloadBinary() {
  const binaryPath = getBinaryPath();
  if (fs.existsSync(binaryPath)) {
    return binaryPath;
  }

  const targetName = getPlatformBinaryName();
  const ext = process.platform === 'win32' ? 'zip' : 'tar.gz';
  const url = `https://github.com/${REPO}/releases/download/${VERSION}/${targetName}.${ext}`;

  console.log(`Downloading Loom binary for ${targetName}...`);

  const tmpArchive = path.join(binDir, `loom-archive.${ext}`);

  await new Promise((resolve, reject) => {
    function getWithRedirect(currUrl) {
      https.get(currUrl, res => {
        if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          getWithRedirect(res.headers.location);
          return;
        }
        if (res.statusCode !== 200) {
          reject(new Error(`Failed to download binary: HTTP ${res.statusCode} from ${currUrl}`));
          return;
        }
        const fileStream = fs.createWriteStream(tmpArchive);
        res.pipe(fileStream);
        fileStream.on('finish', () => {
          fileStream.close();
          resolve();
        });
      }).on('error', reject);
    }
    getWithRedirect(url);
  });

  if (ext === 'tar.gz') {
    execSync(`tar -xzf "${tmpArchive}" -C "${binDir}"`);
    fs.renameSync(path.join(binDir, 'loom'), binaryPath);
  } else {
    execSync(`powershell -command "Expand-Archive -Path '${tmpArchive}' -DestinationPath '${binDir}' -Force"`);
    fs.renameSync(path.join(binDir, 'loom.exe'), binaryPath);
  }

  try {
    fs.unlinkSync(tmpArchive);
  } catch (_) {}

  fs.chmodSync(binaryPath, 0o755);
  console.log('✓ Loom binary installed successfully.');
  return binaryPath;
}

if (process.argv[1] && process.argv[1].endsWith('install.js')) {
  downloadBinary().catch(err => {
    console.warn(`Note: Could not pre-download Loom binary (${err.message}). It will be downloaded on first run.`);
  });
}
