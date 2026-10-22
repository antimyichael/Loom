pub mod rbx_xml;
pub mod rojo;

use rbx_xml::{parse_rbx_xml, RbxInstance, RobloxModelFile};
use rojo::find_and_parse_rojo_project;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RobloxAttachmentUsage {
    pub model_path: String,
    pub model_name: String,
    pub parent_instance_name: String,
    pub script_type: String,
}

#[derive(Debug, Clone)]
pub struct RobloxProjectAnalysis {
    pub file_to_datamodel: HashMap<String, String>,
    pub models: Vec<RobloxModelFile>,
    pub script_attachments: HashMap<String, Vec<RobloxAttachmentUsage>>,
}

impl RobloxProjectAnalysis {
    pub fn build(
        file_to_datamodel: HashMap<String, String>,
        models: Vec<RobloxModelFile>,
    ) -> Self {
        let mut script_attachments: HashMap<String, Vec<RobloxAttachmentUsage>> = HashMap::new();

        for model in &models {
            let model_name = Path::new(&model.file_path)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| model.file_path.clone());

            for script in &model.scripts {
                let usage = RobloxAttachmentUsage {
                    model_path: model.file_path.clone(),
                    model_name: model_name.clone(),
                    parent_instance_name: script.parent_name.clone(),
                    script_type: script.script_type.clone(),
                };
                script_attachments
                    .entry(script.script_name.clone())
                    .or_default()
                    .push(usage);
            }
        }

        Self {
            file_to_datamodel,
            models,
            script_attachments,
        }
    }
}

pub fn is_roblox_project(project_root: &Path) -> bool {
    if project_root.join("default.project.json").is_file()
        || project_root.join("wally.toml").is_file()
        || project_root.join("aftman.toml").is_file()
    {
        return true;
    }

    if let Ok(entries) = fs::read_dir(project_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(".project.json") || name.ends_with(".luau") || name.ends_with(".rbxmx") || name.ends_with(".rbxlx") {
                        return true;
                    }
                }
            }
        }
    }

    let walker = WalkDir::new(project_root).max_depth(3).into_iter();
    for entry in walker.flatten() {
        if entry.file_type().is_file() {
            let ext = entry.path().extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext == "luau" || ext == "rbxmx" || ext == "rbxlx" {
                return true;
            }
        }
    }

    false
}

pub fn scan_and_analyze_roblox(project_root: &Path) -> RobloxProjectAnalysis {
    let file_to_datamodel = find_and_parse_rojo_project(project_root).unwrap_or_default();

    let mut models = Vec::new();
    let walker = WalkDir::new(project_root).follow_links(false).into_iter();

    for entry in walker.filter_entry(|e| {
        let name = e.file_name().to_string_lossy();
        name != ".git" && name != ".obsidian-index" && name != "LoomVault" && name != "node_modules"
    }).flatten() {
        if entry.file_type().is_file() {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext == "rbxmx" || ext == "rbxlx" {
                if let Ok(content) = fs::read_to_string(path) {
                    if let Ok(rel) = path.strip_prefix(project_root) {
                        let rel_str = rel.to_string_lossy().replace('\\', "/");
                        if let Some(model) = parse_rbx_xml(&rel_str, &content) {
                            models.push(model);
                        }
                    }
                }
            }
        }
    }

    RobloxProjectAnalysis::build(file_to_datamodel, models)
}

fn class_icon(class_name: &str) -> &'static str {
    match class_name {
        "Model" => "📦",
        "Part" | "MeshPart" | "WedgePart" | "TrussPart" => "🧱",
        "Script" => "📜",
        "LocalScript" => "💻",
        "ModuleScript" => "🧩",
        "Folder" => "📁",
        "RemoteEvent" | "RemoteFunction" | "BindableEvent" | "BindableFunction" => "⚡",
        "ScreenGui" | "Frame" | "TextLabel" | "TextButton" | "ImageLabel" => "🖥️",
        "Sound" => "🔊",
        "ParticleEmitter" | "Beam" | "Trail" => "✨",
        "ProximityPrompt" | "ClickDetector" => "🔘",
        "Tool" => "🗡️",
        _ => "🔹",
    }
}

fn render_instance_tree(inst: &RbxInstance, indent: usize, md: &mut String) {
    let pad = "  ".repeat(indent);
    let icon = class_icon(&inst.class_name);

    if inst.class_name == "Script" || inst.class_name == "LocalScript" || inst.class_name == "ModuleScript" {
        md.push_str(&format!(
            "{}- {} **{}** (`{}`) [[{}]]\n",
            pad, icon, inst.name, inst.class_name, inst.name
        ));
    } else {
        md.push_str(&format!(
            "{}- {} **{}** (`{}`)\n",
            pad, icon, inst.name, inst.class_name
        ));
    }

    for child in &inst.children {
        render_instance_tree(child, indent + 1, md);
    }
}

pub fn build_roblox_model_note(model: &RobloxModelFile) -> String {
    let mut md = String::new();

    let filename = Path::new(&model.file_path)
        .file_name()
        .map(|s| s.to_string_lossy())
        .unwrap_or_else(|| std::borrow::Cow::Borrowed(&model.file_path));

    md.push_str("---\n");
    md.push_str("tags: [loom, roblox, model]\n");
    md.push_str(&format!("file: {}\n", model.file_path));
    md.push_str(&format!("instances: {}\n", model.total_instances));
    md.push_str("---\n\n");

    md.push_str(&format!("# {}\n\n", filename));
    md.push_str(&format!("**Roblox Model File:** `{}`\n", model.file_path));
    md.push_str(&format!("**Total Instances:** {}\n\n", model.total_instances));

    if !model.scripts.is_empty() {
        md.push_str("## Embedded Scripts\n");
        for script in &model.scripts {
            let icon = class_icon(&script.script_type);
            md.push_str(&format!(
                "- {} [[{}]] (`{}`) — inside **{}**\n",
                icon, script.script_name, script.script_type, script.parent_name
            ));
        }
        md.push('\n');
    }

    md.push_str("## Instance Hierarchy\n\n");
    for root in &model.root_instances {
        render_instance_tree(root, 0, &mut md);
    }

    format!("{}\n", md.trim_end())
}
