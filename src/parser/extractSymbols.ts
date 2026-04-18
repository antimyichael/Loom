/**
 * Symbol extraction using tree-sitter for TypeScript/TSX files
 */

import Parser from 'tree-sitter';
import TypeScript from 'tree-sitter-typescript';
import type { CodeSymbol, SymbolKind } from '../types.js';

// Module-level parser instances
const parser = new Parser();
const tsLanguage = TypeScript.typescript;
const tsxLanguage = TypeScript.tsx;

/**
 * Extracts all symbols from TypeScript/TSX source code using tree-sitter
 * @param sourceCode - The source code to parse
 * @param filePath - Relative path to the file being parsed
 * @param language - 'typescript' or 'tsx'
 * @returns Array of extracted symbols
 */
export function extractSymbols(
  sourceCode: string,
  filePath: string,
  language: 'typescript' | 'tsx'
): CodeSymbol[] {
  // Set the appropriate language grammar
  parser.setLanguage(language === 'tsx' ? tsxLanguage : tsLanguage);

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

  // Helper to extract all function/method calls from a node
  const extractCalls = (node: Parser.SyntaxNode): string[] => {
    const callees = new Set<string>();

    const callStack: Parser.SyntaxNode[] = [node];
    while (callStack.length > 0) {
      const current = callStack.pop()!;

      if (current.type === 'call_expression') {
        const functionNode = current.childForFieldName('function');
        if (functionNode) {
          let calleeName: string | undefined;

          if (functionNode.type === 'identifier') {
            calleeName = getText(functionNode);
          } else if (functionNode.type === 'member_expression') {
            const propertyNode = functionNode.childForFieldName('property');
            if (propertyNode) {
              calleeName = getText(propertyNode);
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

      // Extract class declarations
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

      // Extract function declarations
      else if (node.type === 'function_declaration') {
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

      // Extract method definitions (inside classes)
      else if (node.type === 'method_definition') {
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

      // Extract method signatures (in interfaces/types)
      else if (node.type === 'method_signature') {
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

      // Extract abstract method signatures
      else if (node.type === 'abstract_method_signature') {
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

      // Extract public field definitions (class properties)
      else if (node.type === 'public_field_definition') {
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

      // Extract variable declarations (const, let, var)
      else if (node.type === 'variable_declarator') {
        const nameNode = node.childForFieldName('name');
        const valueNode = node.childForFieldName('value');

        // Skip — lexical_declaration handler already captured this as a function
        if (valueNode?.type === 'arrow_function' || valueNode?.type === 'function_expression') {
          // fall through to child traversal only
        } else if (nameNode) {
          if (nameNode.type === 'identifier') {
            symbols.push({
              name: getText(nameNode),
              kind: 'variable',
              filePath,
              line: getLineNumber(node),
              references: [],
              referencedBy: []
            });
          } else if (nameNode.type === 'object_pattern' || nameNode.type === 'array_pattern') {
            extractDestructuredVariables(nameNode, node, filePath, symbols);
          }
        }
      }

      // Extract import statements
      else if (node.type === 'import_statement') {
        const sourceNode = node.childForFieldName('source');
        if (sourceNode) {
          const importPath = getText(sourceNode).replace(/['"]/g, '');
          symbols.push({
            name: importPath,
            kind: 'import',
            filePath,
            line: getLineNumber(node),
            references: [],
            referencedBy: []
          });
        }
      }

      // Extract arrow function assignments (e.g., const myFunc = () => {})
      else if (node.type === 'lexical_declaration') {
        for (let i = 0; i < node.namedChildCount; i++) {
          const child = node.namedChild(i);
          if (child && child.type === 'variable_declarator') {
            const nameNode = child.childForFieldName('name');
            const valueNode = child.childForFieldName('value');

            if (nameNode && valueNode &&
              (valueNode.type === 'arrow_function' || valueNode.type === 'function_expression')) {
              symbols.push({
                name: getText(nameNode),
                kind: 'function',
                filePath,
                line: getLineNumber(child),
                body: getText(valueNode),
                calls: extractCalls(valueNode),
                references: [],
                referencedBy: []
              });
            }
          }
        }
      }

      // Push children in reverse so processing order remains left-to-right
      for (let i = node.namedChildCount - 1; i >= 0; i--) {
        const child = node.namedChild(i);
        if (child) {
          stack.push(child);
        }
      }
    }
  };

  // Helper to extract variables from destructuring patterns
  const extractDestructuredVariables = (
    patternNode: Parser.SyntaxNode,
    declaratorNode: Parser.SyntaxNode,
    filePath: string,
    symbols: CodeSymbol[]
  ): void => {
    const findIdentifiers = (node: Parser.SyntaxNode): void => {
      if (node.type === 'identifier') {
        symbols.push({
          name: getText(node),
          kind: 'variable',
          filePath,
          line: getLineNumber(declaratorNode),
          references: [],
          referencedBy: []
        });
      }

      for (let i = 0; i < node.namedChildCount; i++) {
        const child = node.namedChild(i);
        if (child) {
          findIdentifiers(child);
        }
      }
    };

    findIdentifiers(patternNode);
  };

  // Start traversal from root
  traverse();

  return symbols;
}
