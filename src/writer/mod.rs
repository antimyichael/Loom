pub mod obsidian_config;
pub mod template;

use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn write_note_if_changed(output_path: &Path, content: &str) -> std::io::Result<bool> {
    let new_hash = hash_content(content);

    if output_path.exists() {
        if let Ok(existing_content) = fs::read_to_string(output_path) {
            let existing_hash = hash_content(&existing_content);
            if new_hash == existing_hash {
                return Ok(false);
            }
        }
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(output_path, content)?;
    Ok(true)
}

pub fn ensure_vault_structure(vault_path: &Path) -> std::io::Result<()> {
    fs::create_dir_all(vault_path.join("notes"))?;
    fs::create_dir_all(vault_path.join(".obsidian"))?;
    obsidian_config::write_obsidian_graph_config(vault_path)?;
    Ok(())
}
