// core type definitions for Loom - a tool for generating Obsidian vaults from codebases

export type SymbolKind = 'class' | 'method' | 'function' | 'variable' | 'import';

export interface SymbolRef {
  name: string;
  filePath: string;
  line: number;
}

export interface CodeSymbol {
  name: string;
  kind: SymbolKind;
  filePath: string;
  body?: string;
  calls?: string[];
  line: number;
  references: SymbolRef[];
  referencedBy: SymbolRef[];
}

export interface FileIndex {
  filePath: string;
  language: string;
  symbols: CodeSymbol[];
  lastParsed: number;
}

export interface SymbolNote {
  symbolName: string;
  filePath: string;
  noteFilename: string;
  kind: SymbolKind;
  body?: string;
  calls: string[];
}

export interface ProjectIndex {
  projectRoot: string;
  lastParsed: number;
  files: FileIndex[];
}