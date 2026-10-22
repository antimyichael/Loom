use super::common::{find_enclosing_class, line_number, node_text};
use crate::types::{CodeSymbol, SymbolKind};
use std::collections::HashSet;
use tree_sitter::{Node, Parser};

const CLASS_TYPES: &[&str] = &["class_declaration", "interface_declaration", "enum_declaration"];

fn extract_calls(node: Node, source: &[u8]) -> Vec<String> {
    let mut callees = HashSet::new();
    let mut stack = vec![node];

    while let Some(current) = stack.pop() {
        if current.kind() == "method_invocation" {
            if let Some(name_node) = current.child_by_field_name("name") {
                let name = node_text(name_node, source).trim();
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

pub fn extract_symbols_java(source_code: &str, file_path: &str) -> Vec<CodeSymbol> {
    let mut parser = Parser::new();
    if parser.set_language(&tree_sitter_java::language()).is_err() {
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
            "class_declaration" | "interface_declaration" | "enum_declaration" => {
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

            "method_declaration" | "constructor_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let parent_class = find_enclosing_class(node, source, CLASS_TYPES, "name");
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

            "field_declaration" => {
                let parent_class = find_enclosing_class(node, source, CLASS_TYPES, "name");
                for i in 0..node.named_child_count() {
                    if let Some(child) = node.named_child(i) {
                        if child.kind() == "variable_declarator" {
                            if let Some(name_node) = child.child_by_field_name("name") {
                                symbols.push(CodeSymbol {
                                    name: node_text(name_node, source).to_string(),
                                    kind: SymbolKind::Variable,
                                    file_path: file_path.to_string(),
                                    body: None,
                                    calls: None,
                                    line: line_number(child),
                                    references: Vec::new(),
                                    referenced_by: Vec::new(),
                                    parent_class: parent_class.clone(),
                                });
                            }
                        }
                    }
                }
            }

            "import_declaration" => {

                for i in 0..node.named_child_count() {
                    if let Some(child) = node.named_child(i) {
                        if child.kind() == "scoped_identifier" || child.kind() == "identifier" {
                            symbols.push(CodeSymbol {
                                name: node_text(child, source).to_string(),
                                kind: SymbolKind::Import,
                                file_path: file_path.to_string(),
                                body: None,
                                calls: None,
                                line: line_number(node),
                                references: Vec::new(),
                                referenced_by: Vec::new(),
                                parent_class: None,
                            });
                            break;
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
