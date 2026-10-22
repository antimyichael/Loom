use tree_sitter::Node;

pub fn node_text<'a>(node: Node<'a>, source: &'a [u8]) -> &'a str {
    std::str::from_utf8(&source[node.start_byte()..node.end_byte()]).unwrap_or("")
}

pub fn line_number(node: Node) -> usize {
    node.start_position().row + 1
}

pub fn is_inside_callable(node: Node, callable_types: &[&str]) -> bool {
    let mut current = node.parent();
    while let Some(parent) = current {
        if callable_types.contains(&parent.kind()) {
            return true;
        }
        current = parent.parent();
    }
    false
}

pub fn find_enclosing_class(
    node: Node,
    source: &[u8],
    class_types: &[&str],
    name_field: &str,
) -> Option<String> {
    let mut current = node.parent();
    while let Some(parent) = current {
        if class_types.contains(&parent.kind()) {
            if let Some(name_node) = parent.child_by_field_name(name_field) {
                let name = node_text(name_node, source).trim();
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }
        current = parent.parent();
    }
    None
}
