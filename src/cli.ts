#!/usr/bin/env node
// Loom CLI - parse programming projects and generate Obsidian vaults

import { resolve, relative, join, extname } from 'path';
import { readdir, readFile } from 'node:fs/promises';
import { parseFile, getSupportedExtensions } from './parser/index.js';
import { buildNote, buildSymbolNote } from './writer/noteTemplate.js';
import { writeNoteIfChanged, ensureVaultStructure } from './writer/diffWriter.js';
import { loadIndex, saveIndex, upsertFile, buildCrossReferenceMap } from './index/symbolIndex.js';
import { filePathToNoteId } from './utils.js';
import type { FileIndex } from './types.js';

function getLanguageForExtension(ext: string): string {
  switch (ext) {
    case '.ts':    return 'typescript';
    case '.tsx':   return 'tsx';
    case '.py':    return 'python';
    case '.cs':    return 'csharp';
    case '.java':  return 'java';
    case '.kt':
    case '.kts':   return 'kotlin';
    case '.cpp':
    case '.cc':
    case '.cxx':
    case '.h':
    case '.hpp':   return 'cpp';
    case '.sh':
    case '.bash':  return 'bash';
    default:       return 'text';
  }
}

async function main(): Promise<void> {
  const [major] = process.versions.node.split('.').map(Number);
  if (major !== 22) {
    console.error(
      `✗ Loom requires Node.js 22 LTS. You are running v${process.versions.node}.\n` +
      `  Install nvm (https://github.com/nvm-sh/nvm) or nvm-windows\n` +
      `  (https://github.com/coreybutler/nvm-windows), then run:\n\n` +
      `    nvm install 22\n` +
      `    nvm use 22\n` +
      `    npm install --legacy-peer-deps\n`
    );
    process.exit(1);
  }

  const projectPath = process.argv[2];
  if (!projectPath) {
    console.error('Usage: loom <project-path>');
    process.exit(1);
  }

  const projectRoot = resolve(projectPath);
  const vaultPath   = join(projectRoot, '.obsidian-index');

  console.log(`Parsing project: ${projectRoot}`);
  console.log(`Vault location: ${vaultPath}\n`);

  await ensureVaultStructure(vaultPath);

  const index = await loadIndex(vaultPath, projectRoot);
  const supportedExtensions = getSupportedExtensions();

  const entries = await readdir(projectRoot, { recursive: true, withFileTypes: true });

  const files = entries
    .filter(entry => entry.isFile())
    .map(entry => join((entry as { parentPath?: string }).parentPath ?? projectRoot, entry.name))
    .filter(filePath => {
      const relativePath = relative(projectRoot, filePath);
      if (
        relativePath.includes('node_modules') ||
        relativePath.includes('.git') ||
        relativePath.includes('.obsidian-index')
      ) return false;
      return supportedExtensions.includes(extname(filePath));
    });

  console.log(`Found ${files.length} files to parse\n`);

  let filesProcessed    = 0;
  let fileNotesWritten  = 0;
  let fileNotesSkipped  = 0;
  let symbolNotesWritten = 0;
  let symbolNotesSkipped = 0;

  console.log('=== Pass 1: Parsing files ===\n');

  for (const absolutePath of files) {
    const relativePath = relative(projectRoot, absolutePath);
    try {
      const contents = await readFile(absolutePath, 'utf8');
      const symbols  = parseFile(relativePath, contents);
      const ext      = extname(relativePath);
      const language = getLanguageForExtension(ext);

      const fileIndex: FileIndex = {
        filePath: relativePath,
        language,
        symbols,
        lastParsed: Date.now()
      };

      upsertFile(index, fileIndex);
      filesProcessed++;
      console.log(`[1/2] Parsing: ${relativePath}`);
    } catch (error) {
      console.error(`✗ Error parsing ${relativePath}:`, (error as Error).message);
    }
  }

  console.log('\nBuilding cross-reference map...');
  const crossRefs = buildCrossReferenceMap(index);
  console.log('Cross-reference map built.\n');

  console.log('=== Pass 2: Writing notes ===\n');

  for (const fileIndex of index.files) {
    const relativePath = fileIndex.filePath;

    try {
      const importedBy  = crossRefs.importedBy.get(relativePath);
      const noteContent = buildNote(fileIndex, importedBy);
      const noteFilename = filePathToNoteId(relativePath) + '.md';
      const notePath    = join(vaultPath, 'notes', noteFilename);

      const wasWritten = await writeNoteIfChanged(notePath, noteContent);
      if (wasWritten) {
        fileNotesWritten++;
        console.log(`[2/2] Writing: ${relativePath}`);
      } else {
        fileNotesSkipped++;
        console.log(`[2/2] Writing: ${relativePath} (unchanged)`);
      }

      const className = fileIndex.symbols.find(s => s.kind === 'class')?.name;

      for (const symbol of fileIndex.symbols) {
        if (
          symbol.kind === 'function' ||
          symbol.kind === 'method'   ||
          symbol.kind === 'class'    ||
          symbol.kind === 'variable'
        ) {
          const calledByRefs = symbol.kind !== 'variable'
            ? crossRefs.calledBy.get(symbol.name)
            : undefined;
          const calleeNoteId = crossRefs.calleeNoteId;
          const usedByFiles = importedBy;
          const referencedBy = symbol.kind === 'variable'
            ? crossRefs.referencedBy.get(symbol.name)
            : undefined;
          const symbolNoteContent = buildSymbolNote(
            symbol,
            fileIndex.language,
            calledByRefs,
            calleeNoteId,
            usedByFiles,
            referencedBy
          );
          const symbolFilename = className && symbol.kind === 'method'
            ? `${className}.${symbol.name}.md`
            : `${symbol.name}.md`;
          const symbolNotePath = join(vaultPath, 'notes', symbolFilename);
          const symbolWasWritten = await writeNoteIfChanged(symbolNotePath, symbolNoteContent);
          if (symbolWasWritten) {
            symbolNotesWritten++;
            console.log(`  ✓ Wrote symbol: ${symbol.name}`);
          } else {
            symbolNotesSkipped++;
            console.log(`  - Skipped symbol: ${symbol.name} (unchanged)`);
          }
        }
      }
    } catch (error) {
      console.error(`✗ Error writing notes for ${relativePath}:`, (error as Error).message);
    }
  }

  await saveIndex(vaultPath, index);

  console.log('\n' + '='.repeat(50));
  console.log('Summary:');
  console.log(`  Files processed:      ${filesProcessed}`);
  console.log(`  File notes written:   ${fileNotesWritten}`);
  console.log(`  File notes skipped:   ${fileNotesSkipped}`);
  console.log(`  Symbol notes written: ${symbolNotesWritten}`);
  console.log(`  Symbol notes skipped: ${symbolNotesSkipped}`);
  console.log('='.repeat(50));
}

main().catch(error => {
  console.error('\nError:', error.message);
  process.exit(1);
});