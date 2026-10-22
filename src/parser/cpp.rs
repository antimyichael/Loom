use super::common::{find_enclosing_class, is_inside_callable, line_number, node_text};
use crate::types::{CodeSymbol, SymbolKind};
use std::collections::HashSet;
use tree_sitter::{Node, Parser};

const CALLABLE_TYPES: &[&str] = &["function_definition"];
const CLASS_TYPES: &[&str] = &["class_specifier", "struct_specifier"];

fn extract_calls(node: Node, source: &[u8]) -> Vec<String> {
    let mut callees = HashSet::new();
    let mut stack = vec![node];

    while let Some(current) = stack.pop() {
        if current.kind() == "call_expression" {
            if let Some(func_node) = current.child_by_field_name("function") {
                let callee_name = match func_node.kind() {
                    "identifier" => Some(node_text(func_node, source)),
                    "field_expression" => func_node
                        .child_by_field_name("field")
                        .map(|f| node_text(f, source)),
                    "qualified_identifier" => func_node
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

fn get_innermost_identifier<'a>(node: Node<'a>, source: &'a [u8]) -> Option<&'a str> {
    let mut current = node;
    loop {
        match current.kind() {
            "identifier" | "field_identifier" => return Some(node_text(current, source)),
            "function_declarator" | "pointer_declarator" | "reference_declarator" => {
                if let Some(declarator) = current.child_by_field_name("declarator") {
                    current = declarator;
                } else if let Some(child) = current.named_child(0) {
                    current = child;
                } else {
                    return None;
                }
            }
            "qualified_identifier" => {
                if let Some(name_node) = current.child_by_field_name("name") {
                    return Some(node_text(name_node, source));
                }
                return None;
            }
            _ => {
                if let Some(child) = current.named_child(0) {
                    current = child;
                } else {
                    return None;
                }
            }
        }
    }
}

pub fn extract_symbols_cpp(source_code: &str, file_path: &str) -> Vec<CodeSymbol> {
    let mut parser = Parser::new();
    if parser.set_language(&tree_sitter_cpp::language()).is_err() {
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
            "class_specifier" | "struct_specifier" => {
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
                    if let Some(name) = get_innermost_identifier(declarator_node, source) {
                        let parent_class = find_enclosing_class(node, source, CLASS_TYPES, "name");
                        let kind = if parent_class.is_some() {
                            SymbolKind::Method
                        } else {
                            SymbolKind::Function
                        };
                        let calls = extract_calls(node, source);

                        symbols.push(CodeSymbol {
                            name: name.to_string(),
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
            }

            "declaration" => {
                let is_inside_fn = is_inside_callable(node, CALLABLE_TYPES);
                if !is_inside_fn {
                    if let Some(declarator_node) = node.child_by_field_name("declarator") {
                        if let Some(name) = get_innermost_identifier(declarator_node, source) {
                            let parent_class =
                                find_enclosing_class(node, source, CLASS_TYPES, "name");
                            symbols.push(CodeSymbol {
                                name: name.to_string(),
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
