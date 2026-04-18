/**
 * Symbol extraction using tree-sitter for C# files
 */

import Parser from 'tree-sitter';
import CSharp from 'tree-sitter-c-sharp';
import type { CodeSymbol } from '../types.js';

const parser = new Parser();
parser.setLanguage(CSharp);

/**
 * Extracts all symbols from C# source code using tree-sitter
 * @param sourceCode - The source code to parse
 * @param filePath - Relative path to the file being parsed
 * @returns Array of extracted symbols
 */
export function extractSymbolsCSharp(sourceCode: string, filePath: string): CodeSymbol[] {
  const tree = parser.parse(sourceCode);
  const symbols: CodeSymbol[] = [];
  const visitedNodes = new Set<number>();

  // Helper to get line number from a node (1-indexed)
  const getLineNumber = (node: Parser.SyntaxNode): number => {
    return node.startPosition.row + 1;
  };

  // Helper to extract text for a node
  const getText = (node: Parser.SyntaxNode): string => {
    return sourceCode.slice(node.startIndex, node.endIndex);
  };

  // Helper to extract all function calls from a node
  const extractCalls = (node: Parser.SyntaxNode): string[] => {
    const callees = new Set<string>();
    const callStack: Parser.SyntaxNode[] = [node];

    while (callStack.length > 0) {
      const current = callStack.pop()!;

      if (current.type === 'invocation_expression') {
        const functionNode = current.childForFieldName('function');
        if (functionNode) {
          let calleeName: string | undefined;

          if (functionNode.type === 'identifier') {
            calleeName = getText(functionNode);
          } else if (functionNode.type === 'member_access_expression') {
            const nameNode = functionNode.childForFieldName('name');
            if (nameNode) {
              calleeName = getText(nameNode);
            }
          }

          if (calleeName?.trim()) {
            callees.add(calleeName);
          }
        }
      }

      for (let i = current.namedChildCount - 1; i >= 0; i--) {
        const child = current.namedChild(i);
        if (child) {
          callStack.push(child);
        }
      }
    }

    return Array.from(callees);
  };

  const traverse = (): void => {
    const stack: Parser.SyntaxNode[] = [tree.rootNode];

    while (stack.length > 0) {
      const node = stack.pop()!;
      if (visitedNodes.has(node.id)) {
        continue;
      }
      visitedNodes.add(node.id);

      if (node.type === 'class_declaration') {
        const nameNode = node.childForFieldName('name');
        if (nameNode) {
          symbols.push({
            name: getText(nameNode),
            kind: 'class',
            filePath,
            line: getLineNumber(node),
            body: getText(node),
            references: [],
            referencedBy: []
          });
        }
      }
      else if (node.type === 'interface_declaration') {
        const nameNode = node.childForFieldName('name');
        if (nameNode) {
          symbols.push({
            name: getText(nameNode),
            kind: 'class',
            filePath,
            line: getLineNumber(node),
            body: getText(node),
            references: [],
            referencedBy: []
          });
        }
      }
      else if (node.type === 'method_declaration') {
        const nameNode = node.childForFieldName('name');
        if (nameNode) {
          symbols.push({
            name: getText(nameNode),
            kind: 'method',
            filePath,
            line: getLineNumber(node),
            body: getText(node),
            calls: extractCalls(node),
            references: [],
            referencedBy: []
          });
        }
      }
      else if (node.type === 'constructor_declaration') {
        const nameNode = node.childForFieldName('name');
        if (nameNode) {
          symbols.push({
            name: getText(nameNode),
            kind: 'method',
            filePath,
            line: getLineNumber(node),
            body: getText(node),
            calls: extractCalls(node),
            references: [],
            referencedBy: []
          });
        }
      }
      else if (node.type === 'field_declaration') {
        // In tree-sitter-c-sharp the hierarchy is:
        //   field_declaration → variable_declaration → variable_declarator
        // We must descend through variable_declaration first.
        for (let i = 0; i < node.namedChildCount; i++) {
          const varDecl = node.namedChild(i);
          if (varDecl && varDecl.type === 'variable_declaration') {
            for (let j = 0; j < varDecl.namedChildCount; j++) {
              const declarator = varDecl.namedChild(j);
              if (declarator && declarator.type === 'variable_declarator') {
                const nameNode = declarator.childForFieldName('name');
                if (nameNode) {
                  symbols.push({
                    name: getText(nameNode),
                    kind: 'variable',
                    filePath,
                    line: getLineNumber(declarator),
                    references: [],
                    referencedBy: []
                  });
                }
              }
            }
          }
        }
      }
      else if (node.type === 'property_declaration') {
        const nameNode = node.childForFieldName('name');
        if (nameNode) {
          symbols.push({
            name: getText(nameNode),
            kind: 'variable',
            filePath,
            line: getLineNumber(node),
            references: [],
            referencedBy: []
          });
        }
      }
      else if (node.type === 'using_directive') {
        const importText = getText(node)
          .replace(/^using\s+/, '')
          .replace(/;$/, '')
          .trim();

        if (importText) {
          symbols.push({
            name: importText,
            kind: 'import',
            filePath,
            line: getLineNumber(node),
            references: [],
            referencedBy: []
          });
        }
      }
      else if (node.type === 'local_function_statement') {
        const nameNode = node.childForFieldName('name');
        if (nameNode) {
          symbols.push({
            name: getText(nameNode),
            kind: 'function',
            filePath,
            line: getLineNumber(node),
            body: getText(node),
            calls: extractCalls(node),
            references: [],
            referencedBy: []
          });
        }
      }

      for (let i = node.namedChildCount - 1; i >= 0; i--) {
        const child = node.namedChild(i);
        if (child) {
          stack.push(child);
        }
      }
    }
  };

  traverse();

  return symbols;
}