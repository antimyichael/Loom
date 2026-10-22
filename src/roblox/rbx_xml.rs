use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref RE_ITEM_OPEN: Regex = Regex::new(r#"<Item\s+class="([^"]+)""#).unwrap();
    static ref RE_NAME_PROP: Regex = Regex::new(r#"<string\s+name="Name">([^<]*)</string>"#).unwrap();
}

#[derive(Debug, Clone)]
pub struct RbxInstance {
    pub class_name: String,
    pub name: String,
    pub children: Vec<RbxInstance>,
}

#[derive(Debug, Clone)]
pub struct RobloxEmbeddedScript {
    pub script_name: String,
    pub script_type: String,
    pub parent_name: String,
}

#[derive(Debug, Clone)]
pub struct RobloxModelFile {
    pub file_path: String,
    pub root_instances: Vec<RbxInstance>,
    pub total_instances: usize,
    pub scripts: Vec<RobloxEmbeddedScript>,
}

pub fn parse_rbx_xml(file_path: &str, content: &str) -> Option<RobloxModelFile> {
    let mut root_instances = Vec::new();
    let mut stack: Vec<RbxInstance> = Vec::new();
    let mut total_count = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some(cap) = RE_ITEM_OPEN.captures(trimmed) {
            let class_name = cap[1].to_string();
            let inst = RbxInstance {
                class_name,
                name: "Instance".to_string(),
                children: Vec::new(),
            };
            stack.push(inst);
            total_count += 1;
            continue;
        }

        if let Some(cap) = RE_NAME_PROP.captures(trimmed) {
            if let Some(top) = stack.last_mut() {
                top.name = cap[1].to_string();
            }
            continue;
        }

        if trimmed == "</Item>" || trimmed.starts_with("</Item>") {
            if stack.len() > 1 {
                let child = stack.pop().unwrap();
                stack.last_mut().unwrap().children.push(child);
            } else if stack.len() == 1 {
                let root = stack.pop().unwrap();
                root_instances.push(root);
            }
        }
    }

    while let Some(rem) = stack.pop() {
        if stack.is_empty() {
            root_instances.push(rem);
        } else {
            stack.last_mut().unwrap().children.push(rem);
        }
    }

    if root_instances.is_empty() {
        return None;
    }

    let mut scripts = Vec::new();
    for root in &root_instances {
        collect_scripts(root, "Root", &mut scripts);
    }

    Some(RobloxModelFile {
        file_path: file_path.to_string(),
        root_instances,
        total_instances: total_count,
        scripts,
    })
}

fn collect_scripts(inst: &RbxInstance, parent_name: &str, scripts: &mut Vec<RobloxEmbeddedScript>) {
    if inst.class_name == "Script" || inst.class_name == "LocalScript" || inst.class_name == "ModuleScript" {
        scripts.push(RobloxEmbeddedScript {
            script_name: inst.name.clone(),
            script_type: inst.class_name.clone(),
            parent_name: parent_name.to_string(),
        });
    }

    for child in &inst.children {
        collect_scripts(child, &inst.name, scripts);
    }
}
