use super::meta::UnityGuidIndex;
use super::yaml_parser::parse_unity_yaml;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnityAssetKind {
    Prefab,
    Scene,
    ScriptableObject,
}

impl std::fmt::Display for UnityAssetKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnityAssetKind::Prefab => write!(f, "prefab"),
            UnityAssetKind::Scene => write!(f, "scene"),
            UnityAssetKind::ScriptableObject => write!(f, "asset"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnityScriptRef {
    pub script_name: String,
    pub script_path: String,
    #[allow(dead_code)]
    pub script_guid: String,
}

#[derive(Debug, Clone)]
pub struct UnityGameObjectNode {
    pub name: String,
    pub tag: Option<String>,
    pub parent_name: Option<String>,
    pub children_names: Vec<String>,
    pub attached_scripts: Vec<UnityScriptRef>,
    pub attached_components: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct UnityAssetFile {
    pub file_path: String,
    pub kind: UnityAssetKind,
    pub game_objects: Vec<UnityGameObjectNode>,
}

#[derive(Debug, Clone)]
pub struct UnityAttachmentUsage {
    pub asset_path: String,
    pub game_object_name: String,
    pub asset_kind: UnityAssetKind,
}

#[derive(Debug, Clone, Default)]
pub struct UnityProjectAnalysis {
    pub assets: Vec<UnityAssetFile>,

    pub script_attachments: HashMap<String, Vec<UnityAttachmentUsage>>,
}

pub fn analyze_unity_file(
    file_path: &str,
    content: &str,
    guid_index: &UnityGuidIndex,
) -> Option<UnityAssetFile> {
    let ext = Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let kind = match ext {
        "prefab" => UnityAssetKind::Prefab,
        "unity" => UnityAssetKind::Scene,
        "asset" => UnityAssetKind::ScriptableObject,
        _ => return None,
    };

    let parsed = parse_unity_yaml(content);
    let mut game_objects = Vec::new();

    let mut transform_to_go: HashMap<i64, i64> = HashMap::new();
    let mut go_to_transform: HashMap<i64, i64> = HashMap::new();
    for (tr_id, tr) in &parsed.transforms {
        transform_to_go.insert(*tr_id, tr.game_object_id);
        go_to_transform.insert(tr.game_object_id, *tr_id);
    }

    let mut go_to_monos: HashMap<i64, Vec<&super::yaml_parser::RawMonoBehaviour>> = HashMap::new();
    for mono in parsed.mono_behaviours.values() {
        go_to_monos.entry(mono.game_object_id).or_default().push(mono);
    }

    let mut go_to_other_comps: HashMap<i64, Vec<String>> = HashMap::new();
    for comp in parsed.other_components.values() {
        if !comp.type_name.is_empty() {
            go_to_other_comps
                .entry(comp.game_object_id)
                .or_default()
                .push(comp.type_name.clone());
        }
    }

    for (go_id, raw_go) in &parsed.game_objects {

        let mut parent_name = None;
        let mut children_names = Vec::new();

        if let Some(&tr_id) = go_to_transform.get(go_id) {
            if let Some(tr) = parsed.transforms.get(&tr_id) {

                if tr.father_id != 0 {
                    if let Some(&parent_go_id) = transform_to_go.get(&tr.father_id) {
                        if let Some(parent_go) = parsed.game_objects.get(&parent_go_id) {
                            parent_name = Some(parent_go.name.clone());
                        }
                    }
                }

                for child_tr_id in &tr.children_ids {
                    if let Some(&child_go_id) = transform_to_go.get(child_tr_id) {
                        if let Some(child_go) = parsed.game_objects.get(&child_go_id) {
                            children_names.push(child_go.name.clone());
                        }
                    }
                }
            }
        }

        let mut attached_scripts = Vec::new();
        if let Some(monos) = go_to_monos.get(go_id) {
            for mono in monos {
                if let Some(ref guid) = mono.script_guid {
                    if let Some(script_path) = guid_index.guid_to_path.get(guid) {
                        let script_name = Path::new(script_path)
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| script_path.clone());

                        attached_scripts.push(UnityScriptRef {
                            script_name,
                            script_path: script_path.clone(),
                            script_guid: guid.clone(),
                        });
                    }
                }
            }
        }

        let attached_components = go_to_other_comps.remove(go_id).unwrap_or_default();

        game_objects.push(UnityGameObjectNode {
            name: raw_go.name.clone(),
            tag: raw_go.tag.clone(),
            parent_name,
            children_names,
            attached_scripts,
            attached_components,
        });
    }

    game_objects.sort_by(|a, b| {
        match (a.parent_name.is_none(), b.parent_name.is_none()) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        }
    });

    Some(UnityAssetFile {
        file_path: file_path.to_string(),
        kind,
        game_objects,
    })
}

impl UnityProjectAnalysis {
    pub fn build(assets: Vec<UnityAssetFile>) -> Self {
        let mut script_attachments: HashMap<String, Vec<UnityAttachmentUsage>> = HashMap::new();

        for asset in &assets {
            for go in &asset.game_objects {
                for script in &go.attached_scripts {
                    let usage = UnityAttachmentUsage {
                        asset_path: asset.file_path.clone(),
                        game_object_name: go.name.clone(),
                        asset_kind: asset.kind.clone(),
                    };

                    script_attachments
                        .entry(script.script_name.clone())
                        .or_default()
                        .push(usage.clone());

                    script_attachments
                        .entry(script.script_path.clone())
                        .or_default()
                        .push(usage);
                }
            }
        }

        Self {
            assets,
            script_attachments,
        }
    }
}
