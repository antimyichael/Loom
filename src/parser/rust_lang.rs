use super::common::{find_enclosing_class, line_number, node_text};
use crate::types::{CodeSymbol, SymbolKind};
use std::collections::HashSet;
use tree_sitter::{Node, Parser};

const CLASS_TYPES: &[&str] = &["impl_item", "struct_item", "enum_item", "trait_item"];

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
                    "scoped_identifier" => func_node
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

pub fn extract_symbols_rust(source_code: &str, file_path: &str) -> Vec<CodeSymbol> {
    let mut parser = Parser::new();
    if parser.set_language(&tree_sitter_rust::language()).is_err() {
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
            "struct_item" | "enum_item" | "trait_item" => {
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

            "function_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {

                    let mut parent_class = None;
                    let mut curr = node.parent();
                    while let Some(p) = curr {
                        if p.kind() == "impl_item" {
                            if let Some(type_node) = p.child_by_field_name("type") {
                                parent_class = Some(node_text(type_node, source).to_string());
                            }
                            break;
                        }
                        curr = p.parent();
                    }

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

            "const_item" | "static_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let parent_class = find_enclosing_class(node, source, CLASS_TYPES, "type");
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

            "use_declaration" => {
                if let Some(arg) = node.child_by_field_name("argument") {
                    let import_text = node_text(arg, source);
                    symbols.push(CodeSymbol {
                        name: import_text.to_string(),
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
