pub mod cross_ref;

use crate::types::{FileIndex, ProjectIndex};
use std::fs;
use std::path::Path;

pub fn load_index(vault_path: &Path, project_root: &Path) -> ProjectIndex {
    let index_path = vault_path.join("symbol-index.json");
    if index_path.exists() {
        if let Ok(content) = fs::read_to_string(&index_path) {
            if let Ok(index) = serde_json::from_str::<ProjectIndex>(&content) {
                return index;
            }
        }
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    ProjectIndex {
        project_root: project_root.to_string_lossy().to_string(),
        last_parsed: now,
        files: Vec::new(),
    }
}

pub fn save_index(vault_path: &Path, index: &mut ProjectIndex) -> Result<(), std::io::Error> {
    let index_path = vault_path.join("symbol-index.json");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    index.last_parsed = now;

    let content = serde_json::to_string_pretty(index)?;
    fs::write(index_path, content)?;
    Ok(())
}

pub fn upsert_file(index: &mut ProjectIndex, file_index: FileIndex) {
    if let Some(pos) = index.files.iter().position(|f| f.file_path == file_index.file_path) {
        index.files[pos] = file_index;
    } else {
        index.files.push(file_index);
    }
}

#[allow(dead_code)]
pub fn remove_file(index: &mut ProjectIndex, file_path: &str) {
    index.files.retain(|f| f.file_path != file_path);
}
