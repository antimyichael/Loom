use super::common::{find_enclosing_class, line_number, node_text};
use crate::types::{CodeSymbol, SymbolKind};
use std::collections::HashSet;
use tree_sitter::{Node, Parser};

const CLASS_TYPES: &[&str] = &["class_declaration", "interface_declaration", "struct_declaration"];

fn extract_calls(node: Node, source: &[u8]) -> Vec<String> {
    let mut callees = HashSet::new();
    let mut stack = vec![node];

    while let Some(current) = stack.pop() {
        if current.kind() == "invocation_expression" {
            if let Some(func_node) = current.child_by_field_name("function") {
                let callee_name = match func_node.kind() {
                    "identifier" => Some(node_text(func_node, source)),
                    "member_access_expression" => func_node
                        .child_by_field_name("name")
                        .map(|n| node_text(n, source)),
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

pub fn extract_symbols_csharp(source_code: &str, file_path: &str) -> Vec<CodeSymbol> {
    let mut parser = Parser::new();
    if parser.set_language(&tree_sitter_c_sharp::language()).is_err() {
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
            "class_declaration" | "interface_declaration" | "struct_declaration" => {
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

            "local_function_statement" => {
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

            "property_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let parent_class = find_enclosing_class(node, source, CLASS_TYPES, "name");
                    symbols.push(CodeSymbol {
                        name: node_text(name_node, source).to_string(),
                        kind: SymbolKind::Variable,
                        file_path: file_path.to_string(),
                        body: None,
                        calls: None,
                        line: line_number(node),
                        references: Vec::new(),
                        referenced_by: Vec::new(),
                        parent_class,
                    });
                }
            }

            "field_declaration" => {

                for i in 0..node.named_child_count() {
                    if let Some(child) = node.named_child(i) {
                        if child.kind() == "variable_declaration" {
                            for j in 0..child.named_child_count() {
                                if let Some(decl) = child.named_child(j) {
                                    if decl.kind() == "variable_declarator" {
                                        if let Some(name_node) = decl.child_by_field_name("name") {
                                            let parent_class = find_enclosing_class(
                                                node,
                                                source,
                                                CLASS_TYPES,
                                                "name",
                                            );
                                            symbols.push(CodeSymbol {
                                                name: node_text(name_node, source).to_string(),
                                                kind: SymbolKind::Variable,
                                                file_path: file_path.to_string(),
                                                body: None,
                                                calls: None,
                                                line: line_number(decl),
                                                references: Vec::new(),
                                                referenced_by: Vec::new(),
                                                parent_class,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            "using_directive" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    symbols.push(CodeSymbol {
                        name: node_text(name_node, source).to_string(),
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
