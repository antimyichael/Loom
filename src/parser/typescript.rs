use super::common::{find_enclosing_class, is_inside_callable, line_number, node_text};
use crate::types::{CodeSymbol, SymbolKind};
use std::collections::HashSet;
use tree_sitter::{Node, Parser};

const CALLABLE_TYPES: &[&str] = &[
    "function_declaration",
    "method_definition",
    "arrow_function",
    "function_expression",
];

const CLASS_TYPES: &[&str] = &["class_declaration", "interface_declaration"];

fn extract_calls(node: Node, source: &[u8]) -> Vec<String> {
    let mut callees = HashSet::new();
    let mut stack = vec![node];

    while let Some(current) = stack.pop() {
        if current.kind() == "call_expression" {
            if let Some(func_node) = current.child_by_field_name("function") {
                let callee_name = match func_node.kind() {
                    "identifier" => Some(node_text(func_node, source)),
                    "member_expression" => func_node
                        .child_by_field_name("property")
                        .map(|p| node_text(p, source)),
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

pub fn extract_symbols_typescript(
    source_code: &str,
    file_path: &str,
    is_tsx: bool,
) -> Vec<CodeSymbol> {
    let mut parser = Parser::new();
    let language = if is_tsx {
        tree_sitter_typescript::language_tsx()
    } else {
        tree_sitter_typescript::language_typescript()
    };

    if parser.set_language(&language).is_err() {
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
            "class_declaration" => {
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

            "method_definition" | "method_signature" | "abstract_method_signature" => {
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

            "public_field_definition" => {
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

            "variable_declarator" => {
                let is_inside_fn = is_inside_callable(node, CALLABLE_TYPES);
                if let Some(name_node) = node.child_by_field_name("name") {
                    let value_node = node.child_by_field_name("value");
                    let is_function_val = value_node
                        .map(|v| v.kind() == "arrow_function" || v.kind() == "function_expression")
                        .unwrap_or(false);

                    if is_function_val {
                        if !is_inside_fn {
                            let calls = value_node
                                .map(|v| extract_calls(v, source))
                                .unwrap_or_default();
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
                    } else if !is_inside_fn && name_node.kind() == "identifier" {

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
            }

            "import_statement" => {
                if let Some(source_node) = node.child_by_field_name("source") {
                    let raw_import = node_text(source_node, source);
                    let import_path = raw_import.trim_matches(['\'', '"']);
                    symbols.push(CodeSymbol {
                        name: import_path.to_string(),
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
