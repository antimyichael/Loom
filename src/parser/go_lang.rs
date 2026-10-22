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
                let callee_name = match func_node.kind() {
                    "identifier" => Some(node_text(func_node, source)),
                    "selector_expression" => func_node
                        .child_by_field_name("field")
                        .map(|f| node_text(f, source)),
                    _ => None,
                };

                if let Some(name) = callee_name {
                    let trimmed = name.trim();
                    if !trimmed.is_empty() {
                        callees.insert(trimmed.to_string());
                    }
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

pub fn extract_symbols_go(source_code: &str, file_path: &str) -> Vec<CodeSymbol> {
    let mut parser = Parser::new();
    if parser.set_language(&tree_sitter_go::language()).is_err() {
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
            "type_spec" => {
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

            "function_declaration" => {
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

            "method_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {

                    let mut parent_class = None;
                    if let Some(receiver_node) = node.child_by_field_name("receiver") {
                        let rec_text = node_text(receiver_node, source);
                        let clean_rec = rec_text.trim_matches(['(', ')', '*', ' ']);
                        let parts: Vec<&str> = clean_rec.split_whitespace().collect();
                        if let Some(type_name) = parts.last() {
                            parent_class = Some(type_name.trim_start_matches('*').to_string());
                        }
                    }

                    let calls = extract_calls(node, source);
                    symbols.push(CodeSymbol {
                        name: node_text(name_node, source).to_string(),
                        kind: SymbolKind::Method,
                        file_path: file_path.to_string(),
                        body: Some(node_text(node, source).to_string()),
                        calls: Some(calls),
                        line: line_number(node),
                        references: Vec::new(),
                        referenced_by: Vec::new(),
                        parent_class,
                    });
                }
            }

            "var_spec" | "const_spec" => {
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

            "import_spec" => {
                if let Some(path_node) = node.child_by_field_name("path") {
                    let raw_path = node_text(path_node, source);
                    let clean = raw_path.trim_matches(['"', '\'']);
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
