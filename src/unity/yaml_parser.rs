use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct RawGameObject {
    #[allow(dead_code)]
    pub file_id: i64,
    pub name: String,
    pub tag: Option<String>,
    pub component_ids: Vec<i64>,
}

#[derive(Debug, Clone, Default)]
pub struct RawTransform {
    #[allow(dead_code)]
    pub file_id: i64,
    pub game_object_id: i64,
    pub father_id: i64,
    pub children_ids: Vec<i64>,
}

#[derive(Debug, Clone, Default)]
pub struct RawMonoBehaviour {
    #[allow(dead_code)]
    pub file_id: i64,
    pub game_object_id: i64,
    pub script_guid: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RawComponent {
    #[allow(dead_code)]
    pub file_id: i64,
    pub type_name: String,
    pub game_object_id: i64,
}

#[derive(Debug, Clone, Default)]
pub struct ParsedUnityFile {
    pub game_objects: HashMap<i64, RawGameObject>,
    pub transforms: HashMap<i64, RawTransform>,
    pub mono_behaviours: HashMap<i64, RawMonoBehaviour>,
    pub other_components: HashMap<i64, RawComponent>,
}

fn extract_file_id(val: &str) -> Option<i64> {

    if let Some(pos) = val.find("fileID:") {
        let after = &val[pos + 7..];
        let num_str: String = after
            .chars()
            .skip_while(|c| c.is_whitespace())
            .take_while(|c| c.is_ascii_digit() || *c == '-')
            .collect();
        return num_str.parse::<i64>().ok();
    }
    None
}

fn extract_guid(val: &str) -> Option<String> {

    if let Some(pos) = val.find("guid:") {
        let after = &val[pos + 5..];
        let guid: String = after
            .chars()
            .skip_while(|c| c.is_whitespace())
            .take_while(|c| c.is_ascii_alphanumeric())
            .collect();
        if !guid.is_empty() {
            return Some(guid);
        }
    }
    None
}

pub fn parse_unity_yaml(content: &str) -> ParsedUnityFile {
    let mut result = ParsedUnityFile::default();

    let mut current_class_id: Option<i64> = None;
    let mut current_file_id: Option<i64> = None;
    let mut current_type_name = String::new();

    let mut current_game_object = RawGameObject::default();
    let mut current_transform = RawTransform::default();
    let mut current_mono = RawMonoBehaviour::default();
    let mut current_other_go_id: Option<i64> = None;

    let mut in_children_list = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("--- !u!") {

            if let Some(fid) = current_file_id {
                match current_class_id {
                    Some(1) => {
                        result.game_objects.insert(fid, current_game_object.clone());
                    }
                    Some(4) | Some(224) => {
                        result.transforms.insert(fid, current_transform.clone());
                    }
                    Some(114) => {
                        result.mono_behaviours.insert(fid, current_mono.clone());
                    }
                    Some(_) => {
                        if let Some(go_id) = current_other_go_id {
                            result.other_components.insert(
                                fid,
                                RawComponent {
                                    file_id: fid,
                                    type_name: current_type_name.clone(),
                                    game_object_id: go_id,
                                },
                            );
                        }
                    }
                    None => {}
                }
            }

            in_children_list = false;
            current_other_go_id = None;
            current_type_name.clear();

            let rest = &trimmed[7..];
            let mut parts = rest.split('&');
            let class_str = parts.next().unwrap_or("").trim();
            let fid_str = parts.next().unwrap_or("").trim();

            current_class_id = class_str.parse::<i64>().ok();
            current_file_id = fid_str.parse::<i64>().ok();

            if let Some(fid) = current_file_id {
                current_game_object = RawGameObject {
                    file_id: fid,
                    name: "GameObject".to_string(),
                    tag: None,
                    component_ids: Vec::new(),
                };
                current_transform = RawTransform {
                    file_id: fid,
                    game_object_id: 0,
                    father_id: 0,
                    children_ids: Vec::new(),
                };
                current_mono = RawMonoBehaviour {
                    file_id: fid,
                    game_object_id: 0,
                    script_guid: None,
                };
            }
            continue;
        }

        if trimmed.ends_with(':') && !trimmed.contains(' ') && current_type_name.is_empty() {
            current_type_name = trimmed.trim_end_matches(':').to_string();
        }

        match current_class_id {
            Some(1) => {

                if trimmed.starts_with("m_Name:") {
                    let name = trimmed["m_Name:".len()..].trim().trim_matches(['\'', '"']);
                    if !name.is_empty() {
                        current_game_object.name = name.to_string();
                    }
                } else if trimmed.starts_with("m_TagString:") {
                    let tag = trimmed["m_TagString:".len()..].trim().trim_matches(['\'', '"']);
                    if !tag.is_empty() && tag != "Untagged" {
                        current_game_object.tag = Some(tag.to_string());
                    }
                } else if trimmed.contains("component: {fileID:") {
                    if let Some(cid) = extract_file_id(trimmed) {
                        current_game_object.component_ids.push(cid);
                    }
                }
            }
            Some(4) | Some(224) => {

                if trimmed.starts_with("m_GameObject:") {
                    if let Some(go_id) = extract_file_id(trimmed) {
                        current_transform.game_object_id = go_id;
                    }
                } else if trimmed.starts_with("m_Father:") {
                    if let Some(father_id) = extract_file_id(trimmed) {
                        current_transform.father_id = father_id;
                    }
                } else if trimmed.starts_with("m_Children:") {
                    in_children_list = true;
                } else if in_children_list {
                    if trimmed.starts_with("- {fileID:") {
                        if let Some(child_id) = extract_file_id(trimmed) {
                            current_transform.children_ids.push(child_id);
                        }
                    } else if !trimmed.starts_with('-') {
                        in_children_list = false;
                    }
                }
            }
            Some(114) => {

                if trimmed.starts_with("m_GameObject:") {
                    if let Some(go_id) = extract_file_id(trimmed) {
                        current_mono.game_object_id = go_id;
                    }
                } else if trimmed.contains("m_Script:") || trimmed.contains("guid:") {
                    if let Some(guid) = extract_guid(trimmed) {
                        current_mono.script_guid = Some(guid);
                    }
                }
            }
            Some(_) => {

                if trimmed.starts_with("m_GameObject:") {
                    if let Some(go_id) = extract_file_id(trimmed) {
                        current_other_go_id = Some(go_id);
                    }
                }
            }
            None => {}
        }
    }

    if let Some(fid) = current_file_id {
        match current_class_id {
            Some(1) => {
                result.game_objects.insert(fid, current_game_object);
            }
            Some(4) | Some(224) => {
                result.transforms.insert(fid, current_transform);
            }
            Some(114) => {
                result.mono_behaviours.insert(fid, current_mono);
            }
            Some(_) => {
                if let Some(go_id) = current_other_go_id {
                    result.other_components.insert(
                        fid,
                        RawComponent {
                            file_id: fid,
                            type_name: current_type_name,
                            game_object_id: go_id,
                        },
                    );
                }
            }
            None => {}
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_unity_yaml() {
        let yaml = r#"
--- !u!1 &1000
GameObject:
  m_Name: Player
  m_TagString: PlayerTag
  m_Component:
  - component: {fileID: 1001}
  - component: {fileID: 1002}
--- !u!4 &1001
Transform:
  m_GameObject: {fileID: 1000}
  m_Father: {fileID: 0}
  m_Children:
  - {fileID: 2001}
--- !u!114 &1002
MonoBehaviour:
  m_GameObject: {fileID: 1000}
  m_Script: {fileID: 11500000, guid: abcdef1234567890abcdef1234567890, type: 3}
--- !u!1 &2000
GameObject:
  m_Name: Weapon
  m_Component:
  - component: {fileID: 2001}
--- !u!4 &2001
Transform:
  m_GameObject: {fileID: 2000}
  m_Father: {fileID: 1001}
  m_Children: []
"#;

        let parsed = parse_unity_yaml(yaml);
        assert_eq!(parsed.game_objects.len(), 2);
        assert_eq!(parsed.game_objects.get(&1000).unwrap().name, "Player");
        assert_eq!(parsed.game_objects.get(&2000).unwrap().name, "Weapon");

        assert_eq!(parsed.transforms.len(), 2);
        let player_tr = parsed.transforms.get(&1001).unwrap();
        assert_eq!(player_tr.game_object_id, 1000);
        assert_eq!(player_tr.children_ids, vec![2001]);

        assert_eq!(parsed.mono_behaviours.len(), 1);
        let mono = parsed.mono_behaviours.get(&1002).unwrap();
        assert_eq!(mono.game_object_id, 1000);
        assert_eq!(
            mono.script_guid.as_deref(),
            Some("abcdef1234567890abcdef1234567890")
        );
    }
}
