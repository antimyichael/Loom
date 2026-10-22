use loom_code::index::cross_ref::build_cross_reference_map;
use loom_code::index::{load_index, save_index, upsert_file};
use loom_code::parser::{get_language_for_extension, parse_file};
use loom_code::types::FileIndex;
use loom_code::utils::{file_path_to_note_id, symbol_note_id};
use loom_code::writer::template::{build_index_note, build_note, build_symbol_note};
use loom_code::writer::{ensure_vault_structure, write_note_if_changed};
use std::fs;

#[test]
fn test_end_to_end_indexing_and_vault_generation() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let project_root = temp_dir.path().join("my_project");
    let vault_path = project_root.join(".obsidian-index");

    fs::create_dir_all(&project_root).unwrap();

    let ts_file = project_root.join("auth.ts");
    fs::write(
        &ts_file,
        r#"
import { HashService } from './hasher';

export class AuthService {
    public login(user: string): boolean {
        return HashService.verify(user);
    }
}
"#,
    )
    .unwrap();

    let hasher_file = project_root.join("hasher.ts");
    fs::write(
        &hasher_file,
        r#"
export class HashService {
    public static verify(token: string): boolean {
        return token.length > 0;
    }
}
"#,
    )
    .unwrap();

    ensure_vault_structure(&vault_path).unwrap();
    assert!(vault_path.join("notes").exists());
    assert!(vault_path.join(".obsidian").join("graph.json").exists());

    let mut index = load_index(&vault_path, &project_root);

    let files = vec!["auth.ts", "hasher.ts"];
    for rel_path in &files {
        let abs_path = project_root.join(rel_path);
        let content = fs::read_to_string(&abs_path).unwrap();
        let symbols = parse_file(rel_path, &content);
        let language = get_language_for_extension(".ts").to_string();

        let file_index = FileIndex {
            file_path: rel_path.to_string(),
            language,
            symbols,
            last_parsed: 1000,
        };
        upsert_file(&mut index, file_index);
    }

    assert_eq!(index.files.len(), 2);

    let cross_refs = build_cross_reference_map(&index);

    let hasher_importers = cross_refs.imported_by.get("hasher.ts");
    assert!(
        hasher_importers.is_some(),
        "hasher.ts should be marked as imported"
    );
    assert!(hasher_importers.unwrap().contains(&"auth.ts".to_string()));

    let verify_callers = cross_refs.called_by.get("verify");
    assert!(verify_callers.is_some(), "verify should have callers recorded");
    assert_eq!(verify_callers.unwrap()[0].note_id, "AuthService.login");

    let mut written_first_pass = 0;
    for file_index in &index.files {
        let rel_path = &file_index.file_path;
        let imported_by = cross_refs.imported_by.get(rel_path).map(|v| v.as_slice());
        let note_content = build_note(file_index, imported_by, None);
        let note_path = vault_path
            .join("notes")
            .join(format!("{}.md", file_path_to_note_id(rel_path)));

        if write_note_if_changed(&note_path, &note_content).unwrap() {
            written_first_pass += 1;
        }

        for symbol in &file_index.symbols {
            let called_by_refs = cross_refs.called_by.get(&symbol.name).map(|v| v.as_slice());
            let used_by_files = cross_refs.imported_by.get(rel_path).map(|v| v.as_slice());
            let referenced_by = cross_refs.referenced_by.get(&symbol.name).map(|v| v.as_slice());

            let sym_content = build_symbol_note(
                symbol,
                &file_index.language,
                called_by_refs,
                Some(&cross_refs.callee_note_id),
                used_by_files,
                referenced_by,
                None,
            );
            let sym_path = vault_path
                .join("notes")
                .join(format!("{}.md", symbol_note_id(symbol)));
            if write_note_if_changed(&sym_path, &sym_content).unwrap() {
                written_first_pass += 1;
            }
        }
    }

    assert!(written_first_pass > 0);

    save_index(&vault_path, &mut index).unwrap();
    assert!(vault_path.join("symbol-index.json").exists());

    let overview_content = build_index_note(&index);
    let overview_path = vault_path.join("notes").join("_Overview.md");
    write_note_if_changed(&overview_path, &overview_content).unwrap();
    assert!(overview_path.exists());

    let mut written_second_pass = 0;
    for file_index in &index.files {
        let rel_path = &file_index.file_path;
        let imported_by = cross_refs.imported_by.get(rel_path).map(|v| v.as_slice());
        let note_content = build_note(file_index, imported_by, None);
        let note_path = vault_path
            .join("notes")
            .join(format!("{}.md", file_path_to_note_id(rel_path)));

        if write_note_if_changed(&note_path, &note_content).unwrap() {
            written_second_pass += 1;
        }
    }
    assert_eq!(
        written_second_pass, 0,
        "Incremental diff writer should skip all unchanged files"
    );
}
