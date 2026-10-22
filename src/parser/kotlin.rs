use super::common::{find_enclosing_class, line_number, node_text};
use crate::types::{CodeSymbol, SymbolKind};
use std::collections::HashSet;
use tree_sitter::{Node, Parser};

const CLASS_TYPES: &[&str] = &["class_declaration", "object_declaration", "interface_declaration"];

fn extract_calls(node: Node, source: &[u8]) -> Vec<String> {
    let mut callees = HashSet::new();
    let mut stack = vec![node];

    while let Some(current) = stack.pop() {
        if current.kind() == "call_expression" {

            if let Some(callee_node) = current.child(0) {
                let callee_name = match callee_node.kind() {
                    "simple_identifier" => Some(node_text(callee_node, source)),
                    "navigation_expression" => {

                        callee_node
                            .named_child(callee_node.named_child_count().saturating_sub(1))
                            .map(|c| node_text(c, source))
                    }
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

pub fn extract_symbols_kotlin(source_code: &str, file_path: &str) -> Vec<CodeSymbol> {
    let mut parser = Parser::new();
    if parser.set_language(&tree_sitter_kotlin::language()).is_err() {
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
            "class_declaration" | "object_declaration" => {
                let name_node = node
                    .child_by_field_name("name")
                    .or_else(|| node.child_by_field_name("identifier"))
                    .or_else(|| {

                        (0..node.named_child_count())
                            .find_map(|i| node.named_child(i))
                            .filter(|c| c.kind() == "simple_identifier")
                    });

                if let Some(name_node) = name_node {
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

            "function_declaration" | "secondary_constructor" => {
                let name_node = node
                    .child_by_field_name("name")
                    .or_else(|| node.child_by_field_name("identifier"))
                    .or_else(|| {
                        (0..node.named_child_count())
                            .find_map(|i| node.named_child(i))
                            .filter(|c| c.kind() == "simple_identifier")
                    });

                if let Some(name_node) = name_node {
                    let parent_class = find_enclosing_class(node, source, CLASS_TYPES, "name")
                        .or_else(|| find_enclosing_class(node, source, CLASS_TYPES, "identifier"));
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

            "property_declaration" => {

                let parent_class = find_enclosing_class(node, source, CLASS_TYPES, "name");
                for i in 0..node.named_child_count() {
                    if let Some(child) = node.named_child(i) {
                        if child.kind() == "variable_declaration" {
                            for j in 0..child.named_child_count() {
                                if let Some(sub) = child.named_child(j) {
                                    if sub.kind() == "simple_identifier" {
                                        symbols.push(CodeSymbol {
                                            name: node_text(sub, source).to_string(),
                                            kind: SymbolKind::Variable,
                                            file_path: file_path.to_string(),
                                            body: None,
                                            calls: None,
                                            line: line_number(sub),
                                            references: Vec::new(),
                                            referenced_by: Vec::new(),
                                            parent_class: parent_class.clone(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }

            "import_header" => {
                for i in 0..node.named_child_count() {
                    if let Some(child) = node.named_child(i) {
                        if child.kind() == "identifier" {
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
