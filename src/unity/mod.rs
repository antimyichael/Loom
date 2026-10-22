pub mod hierarchy;
pub mod meta;
pub mod yaml_parser;

use crate::utils::file_path_to_note_id;
use hierarchy::{analyze_unity_file, UnityAssetFile, UnityProjectAnalysis};
use meta::UnityGuidIndex;
use std::fs;
use std::path::Path;

pub fn is_unity_project(project_root: &Path) -> bool {
    project_root.join("Assets").is_dir() || project_root.join("ProjectSettings").is_dir()
}

pub fn scan_and_analyze_unity(project_root: &Path) -> UnityProjectAnalysis {
    let guid_index = UnityGuidIndex::scan_project(project_root);

    let mut asset_files = Vec::new();
    let walker = walkdir::WalkDir::new(project_root)
        .follow_links(false)
        .into_iter();

    for entry in walker.filter_entry(|e| {
        let name = e.file_name().to_string_lossy();
        name != ".git" && name != ".obsidian-index" && name != "Library" && name != "Temp"
    }) {
        if let Ok(entry) = entry {
            if entry.file_type().is_file() {
                let path = entry.path();
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if ext == "prefab" || ext == "unity" || ext == "asset" {
                    if let Ok(content) = fs::read_to_string(path) {
                        if let Ok(rel) = path.strip_prefix(project_root) {
                            let rel_str = rel.to_string_lossy().replace('\\', "/");
                            if let Some(asset) =
                                analyze_unity_file(&rel_str, &content, &guid_index)
                            {
                                asset_files.push(asset);
                            }
                        }
                    }
                }
            }
        }
    }

    UnityProjectAnalysis::build(asset_files)
}


pub fn build_unity_asset_note(asset: &UnityAssetFile) -> String {
    let mut md = String::new();

    let filename = Path::new(&asset.file_path)
        .file_name()
        .map(|s| s.to_string_lossy())
        .unwrap_or_else(|| std::borrow::Cow::Borrowed(&asset.file_path));

    md.push_str("---\n");
    md.push_str(&format!("tags: [loom, unity, {}]\n", asset.kind));
    md.push_str(&format!("file: {}\n", asset.file_path));
    md.push_str(&format!("unity_type: {}\n", asset.kind));
    md.push_str(&format!("game_objects: {}\n", asset.game_objects.len()));
    md.push_str("---\n\n");

    md.push_str(&format!("# {}\n\n", filename));
    md.push_str(&format!("**Asset Path:** `{}`\n\n", asset.file_path));

    if asset.game_objects.is_empty() {
        md.push_str("*No GameObjects parsed in asset.*\n");
        return md;
    }

    md.push_str("## Hierarchy & Components\n\n");

    for go in &asset.game_objects {
        md.push_str(&format!("### {}\n", go.name));

        if let Some(ref tag) = go.tag {
            md.push_str(&format!("- **Tag:** `{}`\n", tag));
        }

        if let Some(ref parent) = go.parent_name {
            md.push_str(&format!("- **Parent:** [[{}]]\n", parent));
        }

        if !go.children_names.is_empty() {
            let children_links: Vec<String> = go
                .children_names
                .iter()
                .map(|c| format!("[[{}]]", c))
                .collect();
            md.push_str(&format!("- **Children:** {}\n", children_links.join(", ")));
        }

        if !go.attached_scripts.is_empty() {
            md.push_str("- **Attached Scripts:**\n");
            for script in &go.attached_scripts {
                let note_link = format!("[[{}]]", script.script_name);
                let file_link = format!("[[{}]]", file_path_to_note_id(&script.script_path));
                md.push_str(&format!("  - {} (defined in {})\n", note_link, file_link));
            }
        }

        if !go.attached_components.is_empty() {
            md.push_str(&format!(
                "- **Components:** {}\n",
                go.attached_components.join(", ")
            ));
        }

        md.push('\n');
    }

    format!("{}\n", md.trim_end())
}
