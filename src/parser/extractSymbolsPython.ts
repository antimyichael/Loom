// symbol extraction using tree-sitter for Python files

import Parser from 'tree-sitter';
import Python from 'tree-sitter-python';
import type { CodeSymbol } from '../types.js';

const parser = new Parser();
parser.setLanguage(Python);

export function extractSymbolsPython(sourceCode: string, filePath: string): CodeSymbol[] {
  const tree = parser.parse(sourceCode);
  const symbols: CodeSymbol[] = [];;
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

      if (current.type === 'call') {
        const functionNode = current.childForFieldName('function');
        if (functionNode) {
          let calleeName: string | undefined;

          if (functionNode.type === 'identifier') {
            calleeName = getText(functionNode);
          } else if (functionNode.type === 'attribute') {
            const attributeNode = functionNode.childForFieldName('attribute');
            if (attributeNode) {
              calleeName = getText(attributeNode);
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

  const isInsideClass = (node: Parser.SyntaxNode): boolean => {
    const parent = node.parent;
    if (!parent) return false;
    const grandparent = parent.parent;
    return grandparent?.type === 'class_definition';
  };

  const traverse = (): void => {
    const stack: Parser.SyntaxNode[] = [tree.rootNode];

    while (stack.length > 0) {
      const node = stack.pop()!;
      if (visitedNodes.has(node.id)) {
        continue;
      }
      visitedNodes.add(node.id);

      if (node.type === 'class_definition') {
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
        const nameNode = node.childForFieldName('name');
        if (nameNode) {
          const kind = isInsideClass(node) ? 'method' : 'function';
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
      else if (node.type === 'import_statement') {
        const sourceNode = node.namedChild(0);
        if (sourceNode) {
          symbols.push({
            name: getText(sourceNode),
            kind: 'import',
            filePath,
            line: getLineNumber(node),
            references: [],
            referencedBy: []
          });
        }
      }
      else if (node.type === 'import_from_statement') {
        const moduleNode = node.childForFieldName('module_name');
        if (moduleNode) {
          symbols.push({
            name: getText(moduleNode),
            kind: 'import',
            filePath,
            line: getLineNumber(node),
            references: [],
            referencedBy: []
          });
        }
      }
      else if (node.type === 'assignment') {
        if (node.parent?.type === 'module') {
          const nameNode = node.namedChild(0);
          if (nameNode && nameNode.type === 'identifier') {
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