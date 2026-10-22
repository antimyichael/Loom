use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub fn find_and_parse_rojo_project(project_root: &Path) -> Option<HashMap<String, String>> {
    let default_project = project_root.join("default.project.json");
    if default_project.is_file() {
        if let Ok(content) = fs::read_to_string(&default_project) {
            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                return Some(parse_rojo_tree(&json, project_root));
            }
        }
    }

    if let Ok(entries) = fs::read_dir(project_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(".project.json") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                                return Some(parse_rojo_tree(&json, project_root));
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

fn parse_rojo_tree(json: &Value, project_root: &Path) -> HashMap<String, String> {
    let mut file_to_dm = HashMap::new();

    if let Some(tree) = json.get("tree") {
        walk_rojo_node(tree, "game", project_root, &mut file_to_dm);
    }

    file_to_dm
}

fn clean_script_stem(filename: &str) -> String {
    let mut s = filename;
    for suffix in &[
        ".server.luau",
        ".server.lua",
        ".client.luau",
        ".client.lua",
        ".luau",
        ".lua",
        ".rbxmx",
        ".rbxm",
    ] {
        if let Some(stripped) = s.strip_suffix(suffix) {
            s = stripped;
            break;
        }
    }
    s.to_string()
}

fn walk_rojo_node(
    node: &Value,
    current_dm_path: &str,
    project_root: &Path,
    file_to_dm: &mut HashMap<String, String>,
) {
    let obj = match node.as_object() {
        Some(o) => o,
        None => return,
    };

    if let Some(path_val) = obj.get("$path").and_then(|v| v.as_str()) {
        let mapped_fs_path = project_root.join(path_val);

        if mapped_fs_path.is_file() {
            if let Ok(rel) = mapped_fs_path.strip_prefix(project_root) {
                let rel_str = rel.to_string_lossy().replace('\\', "/");
                file_to_dm.insert(rel_str, current_dm_path.to_string());
            }
        } else if mapped_fs_path.is_dir() {
            for entry in WalkDir::new(&mapped_fs_path).into_iter().flatten() {
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    if ext == "luau" || ext == "lua" || ext == "rbxmx" || ext == "rbxm" {
                        if let Ok(rel_to_base) = file_path.strip_prefix(&mapped_fs_path) {
                            if let Ok(rel_to_root) = file_path.strip_prefix(project_root) {
                                let rel_to_root_str = rel_to_root.to_string_lossy().replace('\\', "/");

                                let parts: Vec<String> = rel_to_base
                                    .iter()
                                    .map(|p| p.to_string_lossy().to_string())
                                    .collect();

                                if !parts.is_empty() {
                                    let last_idx = parts.len() - 1;
                                    let filename = &parts[last_idx];
                                    let stem = clean_script_stem(filename);

                                    let target_dm = if stem == "init" {
                                        if parts.len() == 1 {
                                            current_dm_path.to_string()
                                        } else {
                                            let mut dm = current_dm_path.to_string();
                                            for part in &parts[..last_idx] {
                                                dm.push('.');
                                                dm.push_str(part);
                                            }
                                            dm
                                        }
                                    } else {
                                        let mut dm = current_dm_path.to_string();
                                        for part in &parts[..last_idx] {
                                            dm.push('.');
                                            dm.push_str(part);
                                        }
                                        dm.push('.');
                                        dm.push_str(&stem);
                                        dm
                                    };

                                    file_to_dm.insert(rel_to_root_str, target_dm);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    for (key, val) in obj {
        if key.starts_with('$') {
            continue;
        }
        let next_dm_path = if current_dm_path.is_empty() {
            format!("game.{}", key)
        } else {
            format!("{}.{}", current_dm_path, key)
        };
        walk_rojo_node(val, &next_dm_path, project_root, file_to_dm);
    }
}
