// manages the project-wide symbol index stored in symbol-index.json

import { readFile, writeFile } from 'node:fs/promises';
import { join, dirname, normalize } from 'path';
import type { ProjectIndex, FileIndex } from '../types.js';

export interface SymbolNoteRef {
  noteId: string;
  filePath: string;
}

export interface CrossReferenceMap {
  importedBy: Map<string, string[]>;
  calledBy: Map<string, SymbolNoteRef[]>;
  calleeNoteId: Map<string, string>;
  referencedBy: Map<string, SymbolNoteRef[]>; 
}

function symbolNoteId(
  symbolName: string,
  symbolKind: string,
  className: string | undefined
): string {
  return className && symbolKind === 'method'
    ? `${className}.${symbolName}`
    : symbolName;
}

export async function loadIndex(
  vaultPath: string,
  projectRoot: string
): Promise<ProjectIndex> {
  const indexPath = join(vaultPath, 'symbol-index.json');

  try {
    const content = await readFile(indexPath, 'utf8');
    return JSON.parse(content) as ProjectIndex;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === 'ENOENT') {
      return { projectRoot, lastParsed: Date.now(), files: [] };
    }
    throw error;
  }
}

export async function saveIndex(
  vaultPath: string,
  index: ProjectIndex
): Promise<void> {
  const indexPath = join(vaultPath, 'symbol-index.json');
  index.lastParsed = Date.now();
  await writeFile(indexPath, JSON.stringify(index, null, 2), 'utf8');
}

export function upsertFile(index: ProjectIndex, fileIndex: FileIndex): void {
  const existingIndex = index.files.findIndex(f => f.filePath === fileIndex.filePath);
  if (existingIndex !== -1) {
    index.files[existingIndex] = fileIndex;
  } else {
    index.files.push(fileIndex);
  }
}

export function removeFile(index: ProjectIndex, filePath: string): void {
  const existingIndex = index.files.findIndex(f => f.filePath === filePath);
  if (existingIndex !== -1) {
    index.files.splice(existingIndex, 1);
  }
}

export function buildCrossReferenceMap(index: ProjectIndex): CrossReferenceMap {
  const importedBy    = new Map<string, string[]>();
  const calledBy      = new Map<string, SymbolNoteRef[]>();
  const calleeNoteId  = new Map<string, string>();
  const referencedBy  = new Map<string, SymbolNoteRef[]>();

  for (const file of index.files) {
    const className = file.symbols.find(s => s.kind === 'class')?.name;

    for (const symbol of file.symbols) {
      if (
        symbol.kind === 'function' ||
        symbol.kind === 'method' ||
        symbol.kind === 'class'
      ) {
        const noteId = symbolNoteId(symbol.name, symbol.kind, className);
        if (!calleeNoteId.has(symbol.name)) {
          calleeNoteId.set(symbol.name, noteId);
        }
      }
    }
  }

  for (const file of index.files) {
    const className = file.symbols.find(s => s.kind === 'class')?.name;
    const imports = file.symbols.filter(s => s.kind === 'import');
    for (const importSymbol of imports) {
      const importingDir = dirname(file.filePath);
      const resolvedImport = normalize(
        join(importingDir, importSymbol.name)
      ).replace(/\.(ts|tsx|js|jsx)$/, '');

      const matchedFile = index.files.find(f => {
        const normalizedFilePath = normalize(f.filePath).replace(/\.(ts|tsx)$/, '');
        return normalizedFilePath === resolvedImport;
      });

      if (matchedFile) {
        const targetPath = matchedFile.filePath;
        if (!importedBy.has(targetPath)) importedBy.set(targetPath, []);
        const importers = importedBy.get(targetPath)!;
        if (!importers.includes(file.filePath)) importers.push(file.filePath);
      }
    }

    const callableSymbols = file.symbols.filter(
      s => s.kind === 'function' || s.kind === 'method'
    );

    for (const symbol of callableSymbols) {
      const callerNoteId = symbolNoteId(symbol.name, symbol.kind, className);
      const callerRef: SymbolNoteRef = { noteId: callerNoteId, filePath: file.filePath };

      if (symbol.calls && symbol.calls.length > 0) {
        for (const calleeName of symbol.calls) {
          if (!calledBy.has(calleeName)) calledBy.set(calleeName, []);
          const callers = calledBy.get(calleeName)!;
          if (!callers.some(r => r.noteId === callerNoteId)) {
            callers.push(callerRef);
          }
        }
      }

      if (symbol.body) {
        for (const otherFile of index.files) {
          for (const otherSymbol of otherFile.symbols) {
            if (otherSymbol.kind === 'variable') {
              const pattern = new RegExp(`\\b${escapeRegExp(otherSymbol.name)}\\b`);
              if (pattern.test(symbol.body)) {
                if (!referencedBy.has(otherSymbol.name)) {
                  referencedBy.set(otherSymbol.name, []);
                }
                const refs = referencedBy.get(otherSymbol.name)!;
                if (!refs.some(r => r.noteId === callerNoteId)) {
                  refs.push(callerRef);
                }
              }
            }
          }
        }
      }
    }
  }

  return { importedBy, calledBy, calleeNoteId, referencedBy };
}

function escapeRegExp(str: string): string {
  return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}