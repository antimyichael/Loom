/**
 * Core type definitions for Loom - a tool for generating Obsidian vaults from codebases
 */

/**
 * The kind of symbol extracted from source code
 */
export type SymbolKind = 'class' | 'method' | 'function' | 'variable' | 'import';

/**
 * A reference to a symbol at a specific location in a file
 */
export interface SymbolRef {
  name: string;
  filePath: string;
  line: number;
}

/**
 * A symbol extracted from source code with its metadata and relationships
 */
export interface CodeSymbol {
  name: string;
  kind: SymbolKind;
  filePath: string;
  body?: string;
  calls?: string[];
  line: number;
  // Phase 2: symbols this symbol references
  references: SymbolRef[];
  // Phase 2: symbols that reference this symbol
  referencedBy: SymbolRef[];
}

/**
 * Index of all symbols found in a single file
 */
export interface FileIndex {
  filePath: string;
  language: string;
  symbols: CodeSymbol[];
  lastParsed: number;
}

/**
 * Represents a single generated Obsidian note for an individual symbol
 */
export interface SymbolNote {
  symbolName: string;       // e.g. "MethodA"
  filePath: string;         // relative path to the source file
  noteFilename: string;     // e.g. "MethodA.md"
  kind: SymbolKind;
  body?: string;            // raw source code
  calls: string[];          // names of symbols this one calls
}

/**
 * The full project-level index written to symbol-index.json
 */
export interface ProjectIndex {
  projectRoot: string;
  lastParsed: number;
  files: FileIndex[];
}
