// symbol extraction using tree-sitter for Kotlin files

import Parser from 'tree-sitter';
import Kotlin from 'tree-sitter-kotlin';
import type { CodeSymbol } from '../types.js';

const parser = new Parser();
parser.setLanguage(Kotlin);

export function extractSymbolsKotlin(sourceCode: string, filePath: string): CodeSymbol[] {
  const tree = parser.parse(sourceCode);
  const symbols: CodeSymbol[] = [];
  const visitedNodes = new Set<number>();

  const getLineNumber = (node: Parser.SyntaxNode): number => {
    return node.startPosition.row + 1;
  };

  const getText = (node: Parser.SyntaxNode): string => {
    return sourceCode.slice(node.startIndex, node.endIndex);
  };

  const extractCalls = (node: Parser.SyntaxNode): string[] => {
    const callees = new Set<string>();
    const callStack: Parser.SyntaxNode[] = [node];

    while (callStack.length > 0) {
      const current = callStack.pop()!;

      if (current.type === 'call_expression') {
        let calleeNode = current.childForFieldName('calleeExpression');
        if (!calleeNode) {
          calleeNode = current.namedChild(0);
        }

        if (calleeNode) {
          let calleeName: string | undefined;

          if (calleeNode.type === 'simple_identifier') {
            calleeName = getText(calleeNode);
          } else if (calleeNode.type === 'navigation_expression') {
            for (let i = calleeNode.namedChildCount - 1; i >= 0; i--) {
              const child = calleeNode.namedChild(i);
              if (child && child.type === 'simple_identifier') {
                calleeName = getText(child);
                break;
              }
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

  const isInsideClassOrObject = (node: Parser.SyntaxNode): boolean => {
    const parent = node.parent;
    if (!parent) return false;
    const grandparent = parent.parent;
    return (
      grandparent?.type === 'class_declaration' ||
      grandparent?.type === 'object_declaration'
    );
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
      else if (node.type === 'object_declaration') {
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
      else if (node.type === 'function_declaration') {
        const nameNode = node.childForFieldName('name');
        if (nameNode) {
          const kind = isInsideClassOrObject(node) ? 'method' : 'function';
          symbols.push({
            name: getText(nameNode),
            kind,
            filePath,
            line: getLineNumber(node),
            body: getText(node),
            calls: extractCalls(node),
            references: [],
            referencedBy: []
          });
        }
      }
      else if (node.type === 'secondary_constructor') {
        symbols.push({
          name: 'constructor',
          kind: 'method',
          filePath,
          line: getLineNumber(node),
          body: getText(node),
          calls: extractCalls(node),
          references: [],
          referencedBy: []
        });
      }
      else if (node.type === 'property_declaration') {
        let nameNode: Parser.SyntaxNode | null = null;
        for (let i = 0; i < node.namedChildCount; i++) {
          const child = node.namedChild(i);
          if (child && child.type === 'simple_identifier') {
            nameNode = child;
            break;
          }
        }

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
      else if (node.type === 'import_header') {
        const importText = getText(node)
          .replace(/^import\s+/, '')
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