/**
 * Parser router - determines language and delegates to extractSymbols
 */

import { extractSymbols } from './extractSymbols.js';
import type { CodeSymbol } from '../types.js';
import { extname } from 'path';
import { extractSymbolsPython } from './extractSymbolsPython.js';
import { extractSymbolsCSharp } from './extractSymbolsCSharp.js';
import { extractSymbolsJava } from './extractSymbolsJava.js';
import { extractSymbolsKotlin } from './extractSymbolsKotlin.js';
import { extractSymbolsCpp } from './extractSymbolsCpp.js';
import { extractSymbolsBash } from './extractSymbolsBash.js';

/**
 * Returns the list of file extensions supported by the parser
 */
export function getSupportedExtensions(): string[] {
  return ['.ts', '.tsx', '.py', '.cs', '.java', '.kt', '.kts', '.cpp', '.cc', '.cxx', '.h', '.hpp', '.sh', '.bash'];
}

/**
 * Parses a file and extracts all symbols
 * @param filePath - Relative path to the file
 * @param contents - The file contents to parse
 * @returns Array of extracted symbols, or empty array if extension not supported
 */
export function parseFile(filePath: string, contents: string): CodeSymbol[] {
  const ext = extname(filePath);
  
  // Determine language based on file extension
  let language: 'typescript' | 'tsx';
  
  if (ext === '.tsx') {
    language = 'tsx';
  } else if (ext === '.ts') {
    language = 'typescript';
  } else if (ext === '.py') {
    return extractSymbolsPython(contents, filePath);
  } else if (ext === '.cs') {
    return extractSymbolsCSharp(contents, filePath);
  } else if (ext === '.java') {
    return extractSymbolsJava(contents, filePath);
  } else if (ext === '.kt' || ext === '.kts') {
    return extractSymbolsKotlin(contents, filePath);
  } else if (['.cpp', '.cc', '.cxx', '.h', '.hpp'].includes(ext)) {
    return extractSymbolsCpp(contents, filePath);
  } else if (ext === '.sh' || ext === '.bash') {
    return extractSymbolsBash(contents, filePath);
  } else {
    // Unsupported extension - return empty array
    return [];
  }
  
  return extractSymbols(contents, filePath, language);
}
