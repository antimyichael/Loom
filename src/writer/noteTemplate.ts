/**
 * noteTemplate.ts - Converts a FileIndex into Obsidian-formatted Markdown
 */

import { basename } from 'path';
import { filePathToNoteId } from '../utils.js';
import type { FileIndex, CodeSymbol } from '../types.js';

/**
 * Builds an Obsidian note for a single symbol (function, method, or class)
 * @param symbol - The symbol to generate a note for
 * @param language - The language of the source file
 * @param usedBy - Optional list of file paths that use this symbol
 * @returns Markdown string formatted for Obsidian
 */
export function buildSymbolNote(symbol: CodeSymbol, language: string, usedBy?: string[]): string {
  // Build frontmatter
  let markdown = '---\n';
  markdown += `tags: [loom, ${language}, ${symbol.kind}]\n`;
  markdown += `symbol: ${symbol.name}\n`;
  markdown += `file: ${symbol.filePath}\n`;
  markdown += `line: ${symbol.line}\n`;
  markdown += '---\n\n';

  // Add heading
  markdown += `# ${symbol.name}\n\n`;

  // Add "Defined in" section
  markdown += `**Defined in:** [[${filePathToNoteId(symbol.filePath)}]]\n\n`;

  // Add Code section if body exists
  if (symbol.body) {
    markdown += '## Code\n\n';
    markdown += `\`\`\`${language}\n`;
    markdown += symbol.body;
    markdown += '\n```\n\n';
  }

  // Add Calls section if calls exist
  if (symbol.calls && symbol.calls.length > 0) {
    markdown += '## Calls\n';
    const sortedCalls = [...symbol.calls].sort((a, b) => a.localeCompare(b));
    for (const callee of sortedCalls) {
      markdown += `- [[${callee}]]\n`;
    }
    markdown += '\n';
  }

  // Add Used by section if usedBy is provided.
  // Use filePathToNoteId() — not basename() — so the wikilink matches the actual note filename.
  if (usedBy && usedBy.length > 0) {
    markdown += '## Used by\n';
    const sortedUsedBy = usedBy
      .map(filePath => filePathToNoteId(filePath))
      .sort((a, b) => a.localeCompare(b));
    for (const noteId of sortedUsedBy) {
      markdown += `- [[${noteId}]]\n`;
    }
    markdown += '\n';
  }

  return markdown.trimEnd() + '\n';
}

/**
 * Builds an Obsidian note from a file index
 * @param fileIndex - The file index containing symbols
 * @param importedBy - Optional list of file paths that import this file
 * @returns Markdown string formatted for Obsidian
 */
export function buildNote(fileIndex: FileIndex, importedBy?: string[]): string {
  const { filePath, language, symbols, lastParsed } = fileIndex;

  // Convert timestamp to ISO 8601
  const lastParsedISO = new Date(lastParsed).toISOString();

  // Get filename only
  const filename = basename(filePath);

  // Group symbols by kind
  const classes = symbols.filter(s => s.kind === 'class').sort((a, b) => a.name.localeCompare(b.name));
  const functions = symbols.filter(s => s.kind === 'function').sort((a, b) => a.name.localeCompare(b.name));
  const methods = symbols.filter(s => s.kind === 'method').sort((a, b) => a.name.localeCompare(b.name));
  const variables = symbols.filter(s => s.kind === 'variable').sort((a, b) => a.name.localeCompare(b.name));
  const imports = symbols.filter(s => s.kind === 'import').sort((a, b) => a.name.localeCompare(b.name));

  // Build frontmatter
  let markdown = '---\n';
  markdown += `tags: [loom, ${language}]\n`;
  markdown += `file: ${filePath}\n`;
  markdown += `last_parsed: ${lastParsedISO}\n`;
  markdown += '---\n\n';

  // Add heading
  markdown += `# ${filename}\n\n`;

  // Add Classes section with methods indented underneath
  if (classes.length > 0) {
    markdown += '## Classes\n';
    for (const cls of classes) {
      markdown += `- [[${cls.name}]]\n`;

      // Add methods indented under the first class
      if (methods.length > 0) {
        for (const method of methods) {
          markdown += `  - [[${cls.name}.${method.name}]]\n`;
        }
      }
    }
    markdown += '\n';
  }

  // Add Functions section (include methods if no classes exist)
  const functionsToShow = classes.length === 0 && methods.length > 0
    ? [...functions, ...methods]
    : functions;

  if (functionsToShow.length > 0) {
    markdown += '## Functions\n';
    for (const func of functionsToShow.sort((a, b) => a.name.localeCompare(b.name))) {
      markdown += `- [[${func.name}]]\n`;
    }
    markdown += '\n';
  }

  // Add Variables section
  if (variables.length > 0) {
    markdown += '## Variables\n';
    for (const variable of variables) {
      markdown += `- [[${variable.name}]]\n`;
    }
    markdown += '\n';
  }

  // Add Imports section
  if (imports.length > 0) {
    markdown += '## Imports\n';
    for (const imp of imports) {
      markdown += `- [[${imp.name}]]\n`;
    }
    markdown += '\n';
  }

  // Add Imported by section if importedBy is provided
  if (importedBy && importedBy.length > 0) {
    markdown += '## Imported by\n';
    const sortedImportedBy = importedBy
      .map(filePath => filePathToNoteId(filePath))
      .sort((a, b) => a.localeCompare(b));
    for (const filename of sortedImportedBy) {
      markdown += `- [[${filename}]]\n`;
    }
    markdown += '\n';
  }

  return markdown.trimEnd() + '\n';
}