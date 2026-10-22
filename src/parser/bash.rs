use super::common::{line_number, node_text};
use crate::types::{CodeSymbol, SymbolKind};
use std::collections::HashSet;
use tree_sitter::{Node, Parser};

const BUILTIN_CALLS: &[&str] = &[
    "echo", "printf", "cd", "pwd", "ls", "cp", "mv", "rm", "mkdir", "rmdir", "cat", "grep",
    "sed", "awk", "cut", "sort", "uniq", "tr", "head", "tail", "wc", "find", "xargs", "chmod",
    "chown", "source", ".", "export", "local", "read", "return", "exit", "set", "unset", "shift",
    "true", "false", "test", "[", "[[",
];

fn extract_calls(node: Node, source: &[u8]) -> Vec<String> {
    let mut callees = HashSet::new();
    let mut stack = vec![node];

    while let Some(current) = stack.pop() {
        if current.kind() == "command" {
            if let Some(cmd_name_node) = current.child_by_field_name("name") {
                let name = node_text(cmd_name_node, source).trim();
                if !name.is_empty() && !BUILTIN_CALLS.contains(&name) {
                    callees.insert(name.to_string());
                }
            }
        }

        for i in (0..current.named_child_count()).rev() {
            if let Some(child) = current.named_child(i) {
                stack.push(child);
            }
        }
    }

    callees.into_iter().collect()
}

pub fn extract_symbols_bash(source_code: &str, file_path: &str) -> Vec<CodeSymbol> {
    let mut parser = Parser::new();
    if parser.set_language(&tree_sitter_bash::language()).is_err() {
        return Vec::new();
    }

    let Some(tree) = parser.parse(source_code, None) else {
        return Vec::new();
    };

    let source = source_code.as_bytes();
    let mut symbols = Vec::new();
    let mut stack = vec![tree.root_node()];
    let mut visited = HashSet::new();

    while let Some(node) = stack.pop() {
        let node_id = node.id();
        if !visited.insert(node_id) {
            continue;
        }

        match node.kind() {
            "function_definition" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let calls = extract_calls(node, source);
                    symbols.push(CodeSymbol {
                        name: node_text(name_node, source).to_string(),
                        kind: SymbolKind::Function,
                        file_path: file_path.to_string(),
                        body: Some(node_text(node, source).to_string()),
                        calls: Some(calls),
                        line: line_number(node),
                        references: Vec::new(),
                        referenced_by: Vec::new(),
                        parent_class: None,
                    });
                }
            }

            "variable_assignment" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    symbols.push(CodeSymbol {
                        name: node_text(name_node, source).to_string(),
                        kind: SymbolKind::Variable,
                        file_path: file_path.to_string(),
                        body: None,
                        calls: None,
                        line: line_number(node),
                        references: Vec::new(),
                        referenced_by: Vec::new(),
                        parent_class: None,
                    });
                }
            }

            "command" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let cmd_name = node_text(name_node, source).trim();
                    if cmd_name == "source" || cmd_name == "." {

                        if let Some(arg) = node.named_child(1) {
                            let import_file = node_text(arg, source).trim_matches(['\'', '"']);
                            symbols.push(CodeSymbol {
                                name: import_file.to_string(),
                                kind: SymbolKind::Import,
                                file_path: file_path.to_string(),
                                body: None,
                                calls: None,
                                line: line_number(node),
                                references: Vec::new(),
                                referenced_by: Vec::new(),
                                parent_class: None,
                            });
                        }
                    }
                }
            }

            _ => {}
        }

        for i in (0..node.named_child_count()).rev() {
            if let Some(child) = node.named_child(i) {
                stack.push(child);
            }
        }
    }

    symbols
}
