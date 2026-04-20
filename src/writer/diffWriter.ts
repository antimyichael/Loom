// incremental writer - only writes files when content has changed

import { createHash } from 'node:crypto';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { join } from 'path';

function hashContent(content: string): string {
  return createHash('sha256').update(content, 'utf8').digest('hex');
}

export async function writeNoteIfChanged(
  outputPath: string,
  content: string
): Promise<boolean> {
  const newHash = hashContent(content);

  try {
    const existingContent = await readFile(outputPath, 'utf8');
    const existingHash = hashContent(existingContent);

    if (newHash === existingHash) {
      return false;
    }
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== 'ENOENT') {
      throw error;
    }
  }

  await writeFile(outputPath, content, 'utf8');
  return true;
}

export async function ensureVaultStructure(vaultPath: string): Promise<void> {
  await mkdir(join(vaultPath, 'notes'), { recursive: true });
  await mkdir(join(vaultPath, '.obsidian'), { recursive: true });
}