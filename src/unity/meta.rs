use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct UnityGuidIndex {

    pub guid_to_path: HashMap<String, String>,

    pub path_to_guid: HashMap<String, String>,
}

impl UnityGuidIndex {
    pub fn scan_project(project_root: &Path) -> Self {
        let mut index = Self::default();

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
                    if path.extension().and_then(|e| e.to_str()) == Some("meta") {
                        if let Some(guid) = extract_guid_from_meta_file(path) {

                            if let Ok(rel_meta) = path.strip_prefix(project_root) {
                                let meta_str = rel_meta.to_string_lossy().replace('\\', "/");
                                if let Some(asset_rel) = meta_str.strip_suffix(".meta") {
                                    index.guid_to_path.insert(guid.clone(), asset_rel.to_string());
                                    index.path_to_guid.insert(asset_rel.to_string(), guid);
                                }
                            }
                        }
                    }
                }
            }
        }

        index
    }
}

pub fn extract_guid_from_meta_file(meta_path: &Path) -> Option<String> {
    let content = fs::read_to_string(meta_path).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("guid:") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                return Some(parts[1].trim().to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_guid() {
        let sample_meta = r#"fileFormatVersion: 2
guid: 82542a170a0494fbca71e7db79f69165
MonoImporter:
  externalObjects: {}
"#;
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), sample_meta).unwrap();
        let guid = extract_guid_from_meta_file(temp_file.path());
        assert_eq!(guid.as_deref(), Some("82542a170a0494fbca71e7db79f69165"));
    }
}
