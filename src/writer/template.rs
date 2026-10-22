use crate::index::cross_ref::SymbolNoteRef;
use crate::types::{CodeSymbol, FileIndex, ProjectIndex, SymbolKind};
use crate::unity::hierarchy::UnityAttachmentUsage;
use crate::utils::{file_path_to_note_id, symbol_note_id};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::Path;

pub fn build_symbol_note(
    symbol: &CodeSymbol,
    language: &str,
    called_by_refs: Option<&[SymbolNoteRef]>,
    callee_note_id: Option<&HashMap<String, String>>,
    used_by_files: Option<&[String]>,
    referenced_by: Option<&[SymbolNoteRef]>,
    unity_attachments: Option<&[UnityAttachmentUsage]>,
) -> String {
    let mut md = String::new();

    md.push_str("---\n");
    md.push_str(&format!("tags: [loom, {}, {}]\n", language, symbol.kind));
    md.push_str(&format!("symbol: {}\n", symbol.name));
    md.push_str(&format!("file: {}\n", symbol.file_path));
    md.push_str(&format!("line: {}\n", symbol.line));
    md.push_str("---\n\n");

    md.push_str(&format!("# {}\n\n", symbol_note_id(symbol)));
    md.push_str(&format!(
        "**Defined in:** [[{}]]\n\n",
        file_path_to_note_id(&symbol.file_path)
    ));

    if let Some(ref body) = symbol.body {
        md.push_str("## Code\n\n");
        md.push_str(&format!("```{}\n", language));
        md.push_str(body);
        md.push_str("\n```\n\n");
    }

    if let Some(ref calls) = symbol.calls {
        if !calls.is_empty() {
            md.push_str("## Calls\n");
            let mut sorted_calls = calls.clone();
            sorted_calls.sort();
            sorted_calls.dedup();
            for callee in sorted_calls {
                let qualified_id = callee_note_id
                    .and_then(|map| map.get(&callee))
                    .map(|s| s.as_str())
                    .unwrap_or(&callee);
                md.push_str(&format!("- [[{}]]\n", qualified_id));
            }
            md.push('\n');
        }
    }

    if let Some(called_by) = called_by_refs {
        if !called_by.is_empty() {
            md.push_str("## Called by\n");
            let mut sorted: Vec<_> = called_by.to_vec();
            sorted.sort_by(|a, b| a.note_id.cmp(&b.note_id));
            sorted.dedup_by(|a, b| a.note_id == b.note_id);
            for ref_item in sorted {
                md.push_str(&format!("- [[{}]]\n", ref_item.note_id));
            }
            md.push('\n');
        }
    }

    if let Some(used_by) = used_by_files {
        if !used_by.is_empty() {
            md.push_str("## Used by\n");
            let mut sorted: Vec<_> = used_by.iter().map(|fp| file_path_to_note_id(fp)).collect();
            sorted.sort();
            sorted.dedup();
            for note_id in sorted {
                md.push_str(&format!("- [[{}]]\n", note_id));
            }
            md.push('\n');
        }
    }

    if let Some(referenced) = referenced_by {
        if !referenced.is_empty() {
            md.push_str("## Referenced by\n");
            let mut sorted: Vec<_> = referenced.to_vec();
            sorted.sort_by(|a, b| a.note_id.cmp(&b.note_id));
            sorted.dedup_by(|a, b| a.note_id == b.note_id);
            for ref_item in sorted {
                md.push_str(&format!("- [[{}]]\n", ref_item.note_id));
            }
            md.push('\n');
        }
    }

    if let Some(attachments) = unity_attachments {
        if !attachments.is_empty() {
            md.push_str("## Unity Attachments\n");
            for att in attachments {
                let note_id = file_path_to_note_id(&att.asset_path);
                md.push_str(&format!(
                    "- Attached to **{}** in [[{}]] (`{}`)\n",
                    att.game_object_name, note_id, att.asset_kind
                ));
            }
            md.push('\n');
        }
    }

    format!("{}\n", md.trim_end())
}

pub fn build_note(
    file_index: &FileIndex,
    imported_by: Option<&[String]>,
    unity_attachments: Option<&[UnityAttachmentUsage]>,
) -> String {
    let file_path = &file_index.file_path;
    let language = &file_index.language;
    let symbols = &file_index.symbols;

    let filename = Path::new(file_path)
        .file_name()
        .map(|s| s.to_string_lossy())
        .unwrap_or_else(|| std::borrow::Cow::Borrowed(file_path));

    let last_parsed_dt = DateTime::<Utc>::from_timestamp_millis(file_index.last_parsed as i64)
        .unwrap_or_else(Utc::now);
    let last_parsed_iso = last_parsed_dt.to_rfc3339();

    let mut classes: Vec<_> = symbols.iter().filter(|s| s.kind == SymbolKind::Class).collect();
    classes.sort_by(|a, b| a.name.cmp(&b.name));

    let mut functions: Vec<_> = symbols
        .iter()
        .filter(|s| s.kind == SymbolKind::Function)
        .collect();
    functions.sort_by(|a, b| a.name.cmp(&b.name));

    let mut methods: Vec<_> = symbols.iter().filter(|s| s.kind == SymbolKind::Method).collect();
    methods.sort_by(|a, b| a.name.cmp(&b.name));

    let mut variables: Vec<_> = symbols
        .iter()
        .filter(|s| s.kind == SymbolKind::Variable)
        .collect();
    variables.sort_by(|a, b| a.name.cmp(&b.name));

    let mut imports: Vec<_> = symbols.iter().filter(|s| s.kind == SymbolKind::Import).collect();
    imports.sort_by(|a, b| a.name.cmp(&b.name));

    let mut md = String::new();
    md.push_str("---\n");
    md.push_str(&format!("tags: [loom, {}]\n", language));
    md.push_str(&format!("file: {}\n", file_path));
    md.push_str(&format!("last_parsed: {}\n", last_parsed_iso));
    md.push_str("---\n\n");
    md.push_str(&format!("# {}\n\n", filename));

    if !classes.is_empty() {
        md.push_str("## Classes\n");
        for cls in &classes {
            md.push_str(&format!("- [[{}]]\n", cls.name));
            let class_methods: Vec<_> = methods
                .iter()
                .filter(|m| m.parent_class.as_deref() == Some(&cls.name))
                .collect();
            for method in class_methods {
                md.push_str(&format!("  - [[{}.{}]]\n", cls.name, method.name));
            }
        }
        md.push('\n');
    }

    let standalone_functions: Vec<_> = if classes.is_empty() && !methods.is_empty() {
        let mut all = functions.clone();
        all.extend(methods.iter());
        all.sort_by(|a, b| a.name.cmp(&b.name));
        all
    } else {
        let mut non_class_methods: Vec<_> = methods
            .iter()
            .filter(|m| m.parent_class.is_none())
            .copied()
            .collect();
        let mut all = functions.clone();
        all.append(&mut non_class_methods);
        all.sort_by(|a, b| a.name.cmp(&b.name));
        all
    };

    if !standalone_functions.is_empty() {
        md.push_str("## Functions\n");
        for func in standalone_functions {
            md.push_str(&format!("- [[{}]]\n", func.name));
        }
        md.push('\n');
    }

    if !variables.is_empty() {
        md.push_str("## Variables\n");
        for var in variables {
            md.push_str(&format!("- [[{}]]\n", var.name));
        }
        md.push('\n');
    }

    if !imports.is_empty() {
        md.push_str("## Imports\n");
        for imp in imports {
            md.push_str(&format!("- [[{}]]\n", imp.name));
        }
        md.push('\n');
    }

    if let Some(imported_by) = imported_by {
        if !imported_by.is_empty() {
            md.push_str("## Imported by\n");
            let mut sorted: Vec<_> = imported_by.iter().map(|fp| file_path_to_note_id(fp)).collect();
            sorted.sort();
            sorted.dedup();
            for note_id in sorted {
                md.push_str(&format!("- [[{}]]\n", note_id));
            }
            md.push('\n');
        }
    }

    if let Some(attachments) = unity_attachments {
        if !attachments.is_empty() {
            md.push_str("## Unity Attachments\n");
            for att in attachments {
                let note_id = file_path_to_note_id(&att.asset_path);
                md.push_str(&format!(
                    "- Attached to **{}** in [[{}]] (`{}`)\n",
                    att.game_object_name, note_id, att.asset_kind
                ));
            }
            md.push('\n');
        }
    }

    format!("{}\n", md.trim_end())
}

pub fn build_index_note(project_index: &ProjectIndex) -> String {
    let mut md = String::new();
    md.push_str("---\n");
    md.push_str("tags: [loom, overview]\n");
    md.push_str("---\n\n");

    md.push_str("# Codebase Overview\n\n");
    md.push_str("> Auto-generated by **Loom** — an interconnected Obsidian graph of your codebase.\n\n");

    md.push_str("## Files Indexed\n\n");
    let mut sorted_files = project_index.files.clone();
    sorted_files.sort_by(|a, b| a.file_path.cmp(&b.file_path));

    for file in sorted_files {
        let note_id = file_path_to_note_id(&file.file_path);
        let symbol_count = file.symbols.len();
        md.push_str(&format!(
            "- [[{}]] (`{}`) — *{} symbols*\n",
            note_id, file.language, symbol_count
        ));
    }

    format!("{}\n", md.trim_end())
}
