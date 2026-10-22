use super::common::{line_number, node_text};
use crate::types::{CodeSymbol, SymbolKind};
use std::collections::HashSet;
use tree_sitter::{Node, Parser};

fn extract_calls(node: Node, source: &[u8]) -> Vec<String> {
    let mut callees = HashSet::new();
    let mut stack = vec![node];

    while let Some(current) = stack.pop() {
        if current.kind() == "call_expression" {
            if let Some(func_node) = current.child_by_field_name("function") {
                let name = node_text(func_node, source).trim();
                if !name.is_empty() {
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

pub fn extract_symbols_c(source_code: &str, file_path: &str) -> Vec<CodeSymbol> {
    let mut parser = Parser::new();
    if parser.set_language(&tree_sitter_c::language()).is_err() {
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
            "struct_specifier" | "union_specifier" | "enum_specifier" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    symbols.push(CodeSymbol {
                        name: node_text(name_node, source).to_string(),
                        kind: SymbolKind::Class,
                        file_path: file_path.to_string(),
                        body: Some(node_text(node, source).to_string()),
                        calls: None,
                        line: line_number(node),
                        references: Vec::new(),
                        referenced_by: Vec::new(),
                        parent_class: None,
                    });
                }
            }

            "function_definition" => {
                if let Some(declarator_node) = node.child_by_field_name("declarator") {
                    let mut curr = declarator_node;
                    while let Some(inner) = curr.child_by_field_name("declarator") {
                        curr = inner;
                    }
                    let func_name = node_text(curr, source).trim();
                    let calls = extract_calls(node, source);

                    symbols.push(CodeSymbol {
                        name: func_name.to_string(),
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

            "preproc_include" => {
                if let Some(path_node) = node.child_by_field_name("path") {
                    let raw = node_text(path_node, source);
                    let clean = raw.trim_matches(['<', '>', '"', '\'']);
                    symbols.push(CodeSymbol {
                        name: clean.to_string(),
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
