/**
 * Manages the project-wide symbol index stored in symbol-index.json
 */

import { readFile, writeFile } from 'node:fs/promises';
import { join, dirname, resolve, normalize } from 'path';
import type { ProjectIndex, FileIndex } from '../types.js';

/**
 * Cross-reference maps for tracking file relationships
 */
export interface CrossReferenceMap {
  // Maps a filePath to the list of filePaths that import it
  importedBy: Map<string, string[]>;
  // Maps a symbol name to the list of filePaths that call it
  calledBy: Map<string, string[]>;
}

/**
 * Loads the symbol index from disk
 * @param vaultPath - Path to the .obsidian-index vault root
 * @param projectRoot - Path to the project being indexed
 * @returns The loaded index, or a blank index if file doesn't exist
 */
export async function loadIndex(
  vaultPath: string,
  projectRoot: string
): Promise<ProjectIndex> {
  const indexPath = join(vaultPath, 'symbol-index.json');

  try {
    const content = await readFile(indexPath, 'utf8');
    return JSON.parse(content) as ProjectIndex;
  } catch (error) {
    // File doesn't exist - return blank index
    if ((error as NodeJS.ErrnoException).code === 'ENOENT') {
      return {
        projectRoot,
        lastParsed: Date.now(),
        files: []
      };
    }
    // Re-throw unexpected errors
    throw error;
  }
}

/**
 * Saves the symbol index to disk
 * @param vaultPath - Path to the .obsidian-index vault root
 * @param index - The index to save
 */
export async function saveIndex(
  vaultPath: string,
  index: ProjectIndex
): Promise<void> {
  const indexPath = join(vaultPath, 'symbol-index.json');

  // Update timestamp before writing
  index.lastParsed = Date.now();

  // Pretty-print with 2-space indentation
  const content = JSON.stringify(index, null, 2);
  await writeFile(indexPath, content, 'utf8');
}

/**
 * Updates or inserts a file entry in the index
 * Mutates the index object in place
 * @param index - The project index to modify
 * @param fileIndex - The file index to upsert
 */
export function upsertFile(index: ProjectIndex, fileIndex: FileIndex): void {
  const existingIndex = index.files.findIndex(f => f.filePath === fileIndex.filePath);

  if (existingIndex !== -1) {
    // Replace existing entry
    index.files[existingIndex] = fileIndex;
  } else {
    // Append new entry
    index.files.push(fileIndex);
  }
}

/**
 * Removes a file entry from the index by its path
 * Mutates the index object in place. No-op if not found.
 * @param index - The project index to modify
 * @param filePath - The file path to remove
 */
export function removeFile(index: ProjectIndex, filePath: string): void {
  const existingIndex = index.files.findIndex(f => f.filePath === filePath);

  if (existingIndex !== -1) {
    index.files.splice(existingIndex, 1);
  }
}

/**
 * Builds cross-reference maps from the project index
 * @param index - The project index to analyze
 * @returns Maps tracking which files import/call which symbols
 */
export function buildCrossReferenceMap(index: ProjectIndex): CrossReferenceMap {
  const importedBy = new Map<string, string[]>();
  const calledBy = new Map<string, string[]>();

  // Process each file in the index
  for (const file of index.files) {
    // Process imports
    const imports = file.symbols.filter(s => s.kind === 'import');
    for (const importSymbol of imports) {
      const importingDir = dirname(file.filePath);
      const resolvedImport = normalize(

        join(importingDir, importSymbol.name)
      ).replace(/\.(ts|tsx|js|jsx)$/, '');

      const matchedFile = index.files.find(f => {
        const normalizedFilePath = normalize(f.filePath)
          .replace(/\.(ts|tsx)$/, '');
        return normalizedFilePath === resolvedImport;
      });

      if (matchedFile) {
        const targetPath = matchedFile.filePath;
        if (!importedBy.has(targetPath)) {
          importedBy.set(targetPath, []);
        }
        const importers = importedBy.get(targetPath);
        if (importers && !importers.includes(file.filePath)) {
          importers.push(file.filePath);
        }
      }
    }

    // Process function and method calls
    const callableSymbols = file.symbols.filter(
      s => s.kind === 'function' || s.kind === 'method'
    );

    for (const symbol of callableSymbols) {
      if (symbol.calls && symbol.calls.length > 0) {
        for (const calledName of symbol.calls) {
          if (!calledBy.has(calledName)) {
            calledBy.set(calledName, []);
          }
          const callers = calledBy.get(calledName);
          if (callers && !callers.includes(file.filePath)) {
            callers.push(file.filePath);
          }
        }
      }
    }
  }

  return { importedBy, calledBy };
}
