use crate::types::{ProjectIndex, SymbolKind};
use crate::utils::{escape_regex, symbol_note_id_disambiguated};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SymbolNoteRef {
    pub note_id: String,
    pub file_path: String,
}

#[derive(Debug, Clone, Default)]
pub struct CrossReferenceMap {
    pub imported_by: HashMap<String, Vec<String>>,
    pub called_by: HashMap<String, Vec<SymbolNoteRef>>,
    pub callee_note_id: HashMap<String, String>,
    pub referenced_by: HashMap<String, Vec<SymbolNoteRef>>,
    pub colliding_names: HashSet<String>,
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            c => normalized.push(c.as_os_str()),
        }
    }
    normalized
}

fn strip_extension(path: &str) -> &str {
    let extensions = [
        ".ts", ".tsx", ".js", ".jsx", ".py", ".cs", ".java", ".kt", ".kts", ".cpp", ".cc", ".cxx", ".h", ".hpp", ".sh", ".bash",
    ];
    for ext in extensions {
        if let Some(stripped) = path.strip_suffix(ext) {
            return stripped;
        }
    }
    path
}

pub fn build_cross_reference_map(index: &ProjectIndex) -> CrossReferenceMap {
    let mut imported_by: HashMap<String, Vec<String>> = HashMap::new();
    let mut called_by: HashMap<String, Vec<SymbolNoteRef>> = HashMap::new();
    let mut callee_note_id: HashMap<String, String> = HashMap::new();
    let mut referenced_by: HashMap<String, Vec<SymbolNoteRef>> = HashMap::new();

    let mut name_counts: HashMap<String, usize> = HashMap::new();
    for file in &index.files {
        for symbol in &file.symbols {
            if symbol.kind != SymbolKind::Import && symbol.parent_class.is_none() {
                *name_counts.entry(symbol.name.clone()).or_insert(0) += 1;
            }
        }
    }

    let mut colliding_names = HashSet::new();
    for (name, count) in name_counts {
        if count > 1 {
            colliding_names.insert(name);
        }
    }

    for file in &index.files {
        for symbol in &file.symbols {
            if symbol.kind == SymbolKind::Function
                || symbol.kind == SymbolKind::Method
                || symbol.kind == SymbolKind::Class
            {
                let is_colliding = colliding_names.contains(&symbol.name);
                let note_id = symbol_note_id_disambiguated(symbol, is_colliding);
                callee_note_id.entry(symbol.name.clone()).or_insert(note_id);
            }
        }
    }

    let mut var_regexes: Vec<(String, Regex)> = Vec::new();
    for file in &index.files {
        for symbol in &file.symbols {
            if symbol.kind == SymbolKind::Variable {
                let pattern = format!(r"\b{}\b", escape_regex(&symbol.name));
                if let Ok(re) = Regex::new(&pattern) {
                    var_regexes.push((symbol.name.clone(), re));
                }
            }
        }
    }

    for file in &index.files {
        let importing_path = Path::new(&file.file_path);
        let importing_dir = importing_path.parent().unwrap_or_else(|| Path::new(""));

        for symbol in file.symbols.iter().filter(|s| s.kind == SymbolKind::Import) {
            let import_target = importing_dir.join(&symbol.name);
            let normalized_import = normalize_path(&import_target);
            let normalized_import_str = normalized_import.to_string_lossy().replace('\\', "/");
            let stripped_import = strip_extension(&normalized_import_str);

            for candidate in &index.files {
                let candidate_normalized = normalize_path(Path::new(&candidate.file_path));
                let candidate_str = candidate_normalized.to_string_lossy().replace('\\', "/");
                let stripped_candidate = strip_extension(&candidate_str);

                if stripped_candidate == stripped_import || candidate.file_path.ends_with(&symbol.name) {
                    let importers = imported_by.entry(candidate.file_path.clone()).or_default();
                    if !importers.contains(&file.file_path) {
                        importers.push(file.file_path.clone());
                    }
                }
            }
        }

        let callables: Vec<_> = file
            .symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Function || s.kind == SymbolKind::Method)
            .collect();

        for symbol in callables {
            let is_colliding = colliding_names.contains(&symbol.name);
            let caller_note_id = symbol_note_id_disambiguated(symbol, is_colliding);
            let caller_ref = SymbolNoteRef {
                note_id: caller_note_id.clone(),
                file_path: file.file_path.clone(),
            };

            if let Some(ref calls) = symbol.calls {
                for callee_name in calls {
                    let callers = called_by.entry(callee_name.clone()).or_default();
                    if !callers.iter().any(|r| r.note_id == caller_note_id) {
                        callers.push(caller_ref.clone());
                    }
                }
            }

            if let Some(ref body) = symbol.body {
                for (var_name, regex) in &var_regexes {
                    if regex.is_match(body) {
                        let refs = referenced_by.entry(var_name.clone()).or_default();
                        if !refs.iter().any(|r| r.note_id == caller_note_id) {
                            refs.push(caller_ref.clone());
                        }
                    }
                }
            }
        }
    }

    CrossReferenceMap {
        imported_by,
        called_by,
        callee_note_id,
        referenced_by,
        colliding_names,
    }
}
