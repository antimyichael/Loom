use crate::types::{CodeSymbol, SymbolKind};
use std::path::Path;

pub fn file_path_to_note_id(file_path: &str) -> String {
    file_path.replace(['/', '\\'], "__")
}

pub fn symbol_note_id(symbol: &CodeSymbol) -> String {
    symbol_note_id_disambiguated(symbol, false)
}

pub fn symbol_note_id_disambiguated(symbol: &CodeSymbol, is_colliding: bool) -> String {
    if symbol.kind == SymbolKind::Method {
        if let Some(ref class_name) = symbol.parent_class {
            return format!("{}.{}", class_name, symbol.name);
        }
    }

    if is_colliding {
        let stem = Path::new(&symbol.file_path)
            .file_stem()
            .map(|s| s.to_string_lossy())
            .unwrap_or_default();
        if !stem.is_empty() {
            return format!("{}.{}", stem, symbol.name);
        }
    }

    symbol.name.clone()
}

pub fn escape_regex(s: &str) -> String {
    regex::escape(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_path_to_note_id() {
        assert_eq!(file_path_to_note_id("src/parser/index.ts"), "src__parser__index.ts");
        assert_eq!(file_path_to_note_id("src\\utils.ts"), "src__utils.ts");
    }
}
