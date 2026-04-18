#!/usr/bin/env node
/**
 * cli.ts - Loom CLI - Parse programming projects and generate Obsidian vaults
 */

import { resolve, relative, join, extname } from 'path';
import { readdir, readFile } from 'node:fs/promises';
import { parseFile, getSupportedExtensions } from './parser/index.js';
import { buildNote, buildSymbolNote } from './writer/noteTemplate.js';
import { writeNoteIfChanged, ensureVaultStructure } from './writer/diffWriter.js';
import { loadIndex, saveIndex, upsertFile, buildCrossReferenceMap } from './index/symbolIndex.js';
import { filePathToNoteId } from './utils.js';
import type { FileIndex } from './types.js';

/**
 * Maps a file extension to its Obsidian/code-fence language identifier.
 */
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
  // Parse command line arguments
  const projectPath = process.argv[2];

  if (!projectPath) {
    console.error('Usage: loom <project-path>');
    process.exit(1);
  }

  // Resolve to absolute path
  const projectRoot = resolve(projectPath);
  const vaultPath = join(projectRoot, '.obsidian-index');

  console.log(`Parsing project: ${projectRoot}`);
  console.log(`Vault location: ${vaultPath}\n`);

  // Ensure vault structure exists
  await ensureVaultStructure(vaultPath);

  // Load existing index
  const index = await loadIndex(vaultPath, projectRoot);

  // Get supported extensions
  const supportedExtensions = getSupportedExtensions();

  // Walk the project directory recursively
  const entries = await readdir(projectRoot, {
    recursive: true,
    withFileTypes: true
  });

  // Filter to files only and check extensions
  const files = entries
    .filter(entry => entry.isFile())
    .map(entry => join((entry as { parentPath?: string }).parentPath ?? projectRoot, entry.name))
    .filter(filePath => {
      // Skip node_modules, .git, and .obsidian-index
      const relativePath = relative(projectRoot, filePath);
      if (relativePath.includes('node_modules') ||
        relativePath.includes('.git') ||
        relativePath.includes('.obsidian-index')) {
        return false;
      }

      // Check if extension is supported
      const ext = extname(filePath);
      return supportedExtensions.includes(ext);
    });

  console.log(`Found ${files.length} files to parse\n`);

  let filesProcessed = 0;
  let fileNotesWritten = 0;
  let fileNotesSkipped = 0;
  let symbolNotesWritten = 0;
  let symbolNotesSkipped = 0;

  // ========================================
  // PASS 1: Parse all files and build index
  // ========================================
  console.log('=== Pass 1: Parsing files ===\n');

  // Process each file
  for (const absolutePath of files) {
    const relativePath = relative(projectRoot, absolutePath);

    try {
      // Read file contents
      const contents = await readFile(absolutePath, 'utf8');

      // Parse symbols
      const symbols = parseFile(relativePath, contents);

      // Determine language from extension
      const ext = extname(relativePath);
      const language = getLanguageForExtension(ext);

      // Build FileIndex
      const fileIndex: FileIndex = {
        filePath: relativePath,
        language,
        symbols,
        lastParsed: Date.now()
      };

      // Update index
      upsertFile(index, fileIndex);

      filesProcessed++;
      console.log(`[1/2] Parsing: ${relativePath}`);
    } catch (error) {
      console.error(`✗ Error parsing ${relativePath}:`, (error as Error).message);
      // continue to next file
    }
  }

  // ========================================
  // Build cross-reference map
  // ========================================
  console.log('\nBuilding cross-reference map...');
  const crossRefs = buildCrossReferenceMap(index);
  console.log('Cross-reference map built.\n');

  // ========================================
  // PASS 2: Write all notes with cross-refs
  // ========================================
  console.log('=== Pass 2: Writing notes ===\n');

  for (const fileIndex of index.files) {
    const relativePath = fileIndex.filePath;

    try {
      // Get cross-reference data for this file
      const importedBy = crossRefs.importedBy.get(relativePath);

      // Generate note content with cross-refs
      const noteContent = buildNote(fileIndex, importedBy);

      // Derive output path: replace path separators with __
      const noteFilename = filePathToNoteId(relativePath) + '.md';
      const notePath = join(vaultPath, 'notes', noteFilename);

      // Write file note if changed
      const wasWritten = await writeNoteIfChanged(notePath, noteContent);

      if (wasWritten) {
        fileNotesWritten++;
        console.log(`[2/2] Writing: ${relativePath}`);
      } else {
        fileNotesSkipped++;
        console.log(`[2/2] Writing: ${relativePath} (unchanged)`);
      }

      // Write individual symbol notes for functions, methods, and classes
      const className = fileIndex.symbols.find(s => s.kind === 'class')?.name;
      for (const symbol of fileIndex.symbols) {
        if (symbol.kind === 'function' || symbol.kind === 'method' || symbol.kind === 'class') {
          // Get cross-reference data for this symbol
          const usedBy = crossRefs.calledBy.get(symbol.name);

          const symbolNoteContent = buildSymbolNote(symbol, fileIndex.language, usedBy);
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
      // continue to next file
    }
  }

  // Save updated index
  await saveIndex(vaultPath, index);

  // Print summary
  console.log('\n' + '='.repeat(50));
  console.log('Summary:');
  console.log(`  Files processed:      ${filesProcessed}`);
  console.log(`  File notes written:   ${fileNotesWritten}`);
  console.log(`  File notes skipped:   ${fileNotesSkipped}`);
  console.log(`  Symbol notes written: ${symbolNotesWritten}`);
  console.log(`  Symbol notes skipped: ${symbolNotesSkipped}`);
  console.log('='.repeat(50));
}

// Execute with error handling
main().catch(error => {
  console.error('\nError:', error.message);
  process.exit(1);
});