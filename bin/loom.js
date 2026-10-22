#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { getBinaryPath, downloadBinary } from './install.js';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

async function main() {
  let binaryPath = getBinaryPath();

  const localTarget = path.join(__dirname, '..', 'target', 'release', process.platform === 'win32' ? 'loom.exe' : 'loom');
  if (fs.existsSync(localTarget)) {
    binaryPath = localTarget;
  } else if (!fs.existsSync(binaryPath)) {
    try {
      binaryPath = await downloadBinary();
    } catch (err) {
      console.error(`✗ Error downloading Loom precompiled binary: ${err.message}`);
      console.error('  Please install Loom via Cargo or the shell installer:');
      console.error('    cargo install --path .');
      process.exit(1);
    }
  }

  const result = spawnSync(binaryPath, process.argv.slice(2), {
    stdio: 'inherit',
    env: process.env,
  });

  process.exit(result.status ?? 0);
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
