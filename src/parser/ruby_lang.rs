use super::common::{find_enclosing_class, line_number, node_text};
use crate::types::{CodeSymbol, SymbolKind};
use std::collections::HashSet;
use tree_sitter::{Node, Parser};

const CLASS_TYPES: &[&str] = &["class", "module"];

fn extract_calls(node: Node, source: &[u8]) -> Vec<String> {
    let mut callees = HashSet::new();
    let mut stack = vec![node];

    while let Some(current) = stack.pop() {
        if current.kind() == "call" {
            if let Some(func_node) = current.child_by_field_name("method") {
                let name = node_text(func_node, source).trim();
                if !name.is_empty() && name != "require" && name != "require_relative" {
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

pub fn extract_symbols_ruby(source_code: &str, file_path: &str) -> Vec<CodeSymbol> {
    let mut parser = Parser::new();
    if parser.set_language(&tree_sitter_ruby::language()).is_err() {
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
            "class" | "module" => {
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

            "method" | "singleton_method" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let parent_class = find_enclosing_class(node, source, CLASS_TYPES, "name");
                    let kind = if parent_class.is_some() {
                        SymbolKind::Method
                    } else {
                        SymbolKind::Function
                    };
                    let calls = extract_calls(node, source);

                    symbols.push(CodeSymbol {
                        name: node_text(name_node, source).to_string(),
                        kind,
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

            "call" => {

                if let Some(method_node) = node.child_by_field_name("method") {
                    let method_name = node_text(method_node, source);
                    if method_name == "require" || method_name == "require_relative" {
                        if let Some(args_node) = node.child_by_field_name("arguments") {
                            let arg_text = node_text(args_node, source).trim_matches(['"', '\'', '(', ')', ' ']);
                            symbols.push(CodeSymbol {
                                name: arg_text.to_string(),
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
