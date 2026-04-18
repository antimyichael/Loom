/**
 * Symbol extraction using tree-sitter for C++ files
 */

import Parser from 'tree-sitter';
import Cpp from 'tree-sitter-cpp';
import type { CodeSymbol } from '../types.js';

const parser = new Parser();
parser.setLanguage(Cpp);

/**
 * Extracts all symbols from C++ source code using tree-sitter
 * @param sourceCode - The source code to parse
 * @param filePath - Relative path to the file being parsed
 * @returns Array of extracted symbols
 */
export function extractSymbolsCpp(sourceCode: string, filePath: string): CodeSymbol[] {
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
          } else if (functionNode.type === 'field_expression') {
            const fieldNode = functionNode.childForFieldName('field');
            if (fieldNode) {
              calleeName = getText(fieldNode);
            }
          } else if (functionNode.type === 'scoped_identifier') {
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

  /**
   * Walks a declarator subtree iteratively and returns the innermost
   * identifier or field_identifier node. This handles arbitrarily nested
   * declarators such as pointer_declarator → function_declarator → identifier.
   */
  const getInnermostIdentifier = (node: Parser.SyntaxNode): Parser.SyntaxNode | null => {
    let result: Parser.SyntaxNode | null = null;
    const stack: Parser.SyntaxNode[] = [node];

    while (stack.length > 0) {
      const current = stack.pop()!;
      if (current.type === 'identifier' || current.type === 'field_identifier') {
        result = current;
      }

      for (let i = current.namedChildCount - 1; i >= 0; i--) {
        const child = current.namedChild(i);
        if (child) {
          stack.push(child);
        }
      }
    }

    return result;
  };

  const traverse = (): void => {
    const stack: Parser.SyntaxNode[] = [tree.rootNode];

    while (stack.length > 0) {
      const node = stack.pop()!;
      if (visitedNodes.has(node.id)) {
        continue;
      }
      visitedNodes.add(node.id);

      if (node.type === 'class_specifier') {
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
      else if (node.type === 'struct_specifier') {
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
      else if (node.type === 'function_definition') {
        const declaratorNode = node.childForFieldName('declarator');
        const nameNode = declaratorNode ? getInnermostIdentifier(declaratorNode) : null;
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
      else if (node.type === 'declaration') {
        const hasFunctionDeclarator = node.descendantsOfType('function_declarator').length > 0;
        if (!hasFunctionDeclarator) {
          // In C++ ASTs, a declaration's variable name is typically nested inside
          // an init_declarator node, not a direct identifier child. Using
          // getInnermostIdentifier on the declarator field handles all cases:
          //   int x;            → declarator: identifier
          //   int x = 5;        → declarator: init_declarator → identifier
          //   int* x;           → declarator: pointer_declarator → identifier
          const declaratorNode = node.childForFieldName('declarator');
          const nameNode = declaratorNode
            ? getInnermostIdentifier(declaratorNode)
            : null;

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
      }
      else if (node.type === 'preproc_include') {
        const pathNode = node.childForFieldName('path');
        if (pathNode) {
          const importText = getText(pathNode)
            .replace(/^<|>$/g, '')
            .replace(/^\"|\"$/g, '')
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