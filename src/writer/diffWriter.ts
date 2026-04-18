/**
 * Incremental writer - only writes files when content has changed
 */

import { createHash } from 'node:crypto';
import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { join } from 'path';

/**
 * Computes SHA-256 hash of a string
 */
function hashContent(content: string): string {
  return createHash('sha256').update(content, 'utf8').digest('hex');
}

/**
 * Writes a note to disk only if its content has changed
 * @param outputPath - Full path where the note should be written
 * @param content - The note content to write
 * @returns true if file was written, false if skipped (content identical)
 */
export async function writeNoteIfChanged(
  outputPath: string,
  content: string
): Promise<boolean> {
  const newHash = hashContent(content);
  
  // Try to read existing file and compare hashes
  try {
    const existingContent = await readFile(outputPath, 'utf8');
    const existingHash = hashContent(existingContent);
    
    // Content unchanged - skip write
    if (newHash === existingHash) {
      return false;
    }
  } catch (error) {
    // File doesn't exist or can't be read - proceed with write
    // This is expected on first run, so handle gracefully
    if ((error as NodeJS.ErrnoException).code !== 'ENOENT') {
      // Re-throw unexpected errors
      throw error;
    }
  }
  
  // Content has changed or file doesn't exist - write it
  await writeFile(outputPath, content, 'utf8');
  return true;
}

/**
 * Ensures the vault folder structure exists
 * @param vaultPath - Path to the .obsidian-index vault root
 */
export async function ensureVaultStructure(vaultPath: string): Promise<void> {
  await mkdir(join(vaultPath, 'notes'), { recursive: true });
  await mkdir(join(vaultPath, '.obsidian'), { recursive: true });
}
