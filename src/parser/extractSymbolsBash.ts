// symbol extraction using tree-sitter for Bash/Shell files

import Parser from 'tree-sitter';
import Bash from 'tree-sitter-bash';
import type { CodeSymbol } from '../types.js';

const parser = new Parser();
parser.setLanguage(Bash);

const BUILTIN_CALLS = new Set([
  'echo',
  'cd',
  'export',
  'set',
  'unset',
  'return',
  'exit',
  'local',
  'declare',
  'printf',
  'read',
  'test',
  'true',
  'false',
  '[',
  '[[',
]);

export function extractSymbolsBash(sourceCode: string, filePath: string): CodeSymbol[] {
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

      if (current.type === 'command') {
        const commandNameNode = current.namedChild(0);
        if (commandNameNode && commandNameNode.type === 'word') {
          const calleeName = getText(commandNameNode);
          if (calleeName?.trim() && !BUILTIN_CALLS.has(calleeName)) {
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

      if (node.type === 'function_definition') {
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
      else if (node.type === 'variable_assignment') {
        if (node.parent?.type === 'program') {
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
      }
      else if (node.type === 'source_command') {
        const sourceNode = node.namedChild(0);
        if (sourceNode) {
          const importText = getText(sourceNode)
            .replace(/^['\"]|['\"]$/g, '')
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