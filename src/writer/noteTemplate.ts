// converts a FileIndex into Obsidian-formatted Markdown

import { basename } from 'path';
import { filePathToNoteId } from '../utils.js';
import type { FileIndex, CodeSymbol } from '../types.js';
import type { SymbolNoteRef } from '../index/symbolIndex.js';

export function buildSymbolNote(
  symbol: CodeSymbol,
  language: string,
  calledByRefs?: SymbolNoteRef[],
  calleeNoteId?: Map<string, string>,
  usedByFiles?: string[],
  referencedBy?: SymbolNoteRef[]
): string {
  let markdown = '---\n';
  markdown += `tags: [loom, ${language}, ${symbol.kind}]\n`;
  markdown += `symbol: ${symbol.name}\n`;
  markdown += `file: ${symbol.filePath}\n`;
  markdown += `line: ${symbol.line}\n`;
  markdown += '---\n\n';

  markdown += `# ${symbol.name}\n\n`;
  markdown += `**Defined in:** [[${filePathToNoteId(symbol.filePath)}]]\n\n`;

  if (symbol.body) {
    markdown += '## Code\n\n';
    markdown += `\`\`\`${language}\n`;
    markdown += symbol.body;
    markdown += '\n```\n\n';
  }

  if (symbol.calls && symbol.calls.length > 0) {
    markdown += '## Calls\n';
    const sortedCalls = [...symbol.calls].sort((a, b) => a.localeCompare(b));
    for (const callee of sortedCalls) {
      const qualifiedId = calleeNoteId?.get(callee) ?? callee;
      markdown += `- [[${qualifiedId}]]\n`;
    }
    markdown += '\n';
  }

  if (calledByRefs && calledByRefs.length > 0) {
    markdown += '## Called by\n';
    const sorted = [...calledByRefs].sort((a, b) => a.noteId.localeCompare(b.noteId));
    for (const ref of sorted) {
      markdown += `- [[${ref.noteId}]]\n`;
    }
    markdown += '\n';
  }

  if (usedByFiles && usedByFiles.length > 0) {
    markdown += '## Used by\n';
    const sorted = usedByFiles
      .map(fp => filePathToNoteId(fp))
      .sort((a, b) => a.localeCompare(b));
    for (const noteId of sorted) {
      markdown += `- [[${noteId}]]\n`;
    }
    markdown += '\n';
  }

  if (referencedBy && referencedBy.length > 0) {
    markdown += '## Referenced by\n';
    const sorted = [...referencedBy].sort((a, b) => a.noteId.localeCompare(b.noteId));
    for (const ref of sorted) {
      markdown += `- [[${ref.noteId}]]\n`;
    }
    markdown += '\n';
  }

  return markdown.trimEnd() + '\n';
}

export function buildNote(fileIndex: FileIndex, importedBy?: string[]): string {
  const { filePath, language, symbols, lastParsed } = fileIndex;

  const lastParsedISO = new Date(lastParsed).toISOString();
  const filename = basename(filePath);

  const classes   = symbols.filter(s => s.kind === 'class').sort((a, b) => a.name.localeCompare(b.name));
  const functions = symbols.filter(s => s.kind === 'function').sort((a, b) => a.name.localeCompare(b.name));
  const methods   = symbols.filter(s => s.kind === 'method').sort((a, b) => a.name.localeCompare(b.name));
  const variables = symbols.filter(s => s.kind === 'variable').sort((a, b) => a.name.localeCompare(b.name));
  const imports   = symbols.filter(s => s.kind === 'import').sort((a, b) => a.name.localeCompare(b.name));

  let markdown = '---\n';
  markdown += `tags: [loom, ${language}]\n`;
  markdown += `file: ${filePath}\n`;
  markdown += `last_parsed: ${lastParsedISO}\n`;
  markdown += '---\n\n';
  markdown += `# ${filename}\n\n`;

  if (classes.length > 0) {
    markdown += '## Classes\n';
    for (const cls of classes) {
      markdown += `- [[${cls.name}]]\n`;
      if (methods.length > 0) {
        for (const method of methods) {
          markdown += `  - [[${cls.name}.${method.name}]]\n`;
        }
      }
    }
    markdown += '\n';
  }

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

  if (variables.length > 0) {
    markdown += '## Variables\n';
    for (const variable of variables) {
      markdown += `- [[${variable.name}]]\n`;
    }
    markdown += '\n';
  }

  if (imports.length > 0) {
    markdown += '## Imports\n';
    for (const imp of imports) {
      markdown += `- [[${imp.name}]]\n`;
    }
    markdown += '\n';
  }

  if (importedBy && importedBy.length > 0) {
    markdown += '## Imported by\n';
    const sorted = importedBy
      .map(fp => filePathToNoteId(fp))
      .sort((a, b) => a.localeCompare(b));
    for (const noteId of sorted) {
      markdown += `- [[${noteId}]]\n`;
    }
    markdown += '\n';
  }

  return markdown.trimEnd() + '\n';
}