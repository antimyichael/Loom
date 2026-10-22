use loom_code::unity::hierarchy::{analyze_unity_file, UnityAssetKind, UnityProjectAnalysis};
use loom_code::unity::meta::UnityGuidIndex;
use loom_code::unity::{build_unity_asset_note, is_unity_project, scan_and_analyze_unity};
use std::fs;

#[test]
fn test_unity_project_detection() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();
    assert!(!is_unity_project(root));

    fs::create_dir(root.join("Assets")).unwrap();
    assert!(is_unity_project(root));
}

#[test]
fn test_unity_yaml_and_hierarchy_analysis() {
    let mut guid_index = UnityGuidIndex::default();
    let script_guid = "11112222333344445555666677778888";
    guid_index.guid_to_path.insert(
        script_guid.to_string(),
        "Assets/Scripts/PlayerController.cs".to_string(),
    );
    guid_index.path_to_guid.insert(
        "Assets/Scripts/PlayerController.cs".to_string(),
        script_guid.to_string(),
    );

    let prefab_yaml = r#"
%YAML 1.1
%TAG !u! tag:unity3d.com,2011:
--- !u!1 &100
GameObject:
  m_Name: PlayerCharacter
  m_TagString: Player
  m_Component:
  - component: {fileID: 101}
  - component: {fileID: 102}
  - component: {fileID: 103}
--- !u!4 &101
Transform:
  m_GameObject: {fileID: 100}
  m_Father: {fileID: 0}
  m_Children:
  - {fileID: 201}
--- !u!54 &102
Rigidbody:
  m_GameObject: {fileID: 100}
--- !u!114 &103
MonoBehaviour:
  m_GameObject: {fileID: 100}
  m_Script: {fileID: 11500000, guid: 11112222333344445555666677778888, type: 3}
--- !u!1 &200
GameObject:
  m_Name: Sword
  m_Component:
  - component: {fileID: 201}
--- !u!4 &201
Transform:
  m_GameObject: {fileID: 200}
  m_Father: {fileID: 101}
  m_Children: []
"#;

    let asset = analyze_unity_file("Assets/Prefabs/Hero.prefab", prefab_yaml, &guid_index)
        .expect("should analyze prefab");

    assert_eq!(asset.kind, UnityAssetKind::Prefab);
    assert_eq!(asset.game_objects.len(), 2);

    let player = asset
        .game_objects
        .iter()
        .find(|g| g.name == "PlayerCharacter")
        .unwrap();
    assert_eq!(player.tag.as_deref(), Some("Player"));
    assert_eq!(player.parent_name, None);
    assert_eq!(player.children_names, vec!["Sword"]);
    assert!(player.attached_components.contains(&"Rigidbody".to_string()));
    assert_eq!(player.attached_scripts.len(), 1);
    assert_eq!(player.attached_scripts[0].script_name, "PlayerController");

    let sword = asset.game_objects.iter().find(|g| g.name == "Sword").unwrap();
    assert_eq!(sword.parent_name.as_deref(), Some("PlayerCharacter"));

    let note = build_unity_asset_note(&asset);
    assert!(note.contains("# Hero.prefab"));
    assert!(note.contains("tags: [loom, unity, prefab]"));
    assert!(note.contains("### PlayerCharacter"));
    assert!(note.contains("- **Children:** [[Sword]]"));
    assert!(note.contains("[[PlayerController]]"));
    assert!(note.contains("- **Components:** Rigidbody"));
    assert!(note.contains("### Sword"));
    assert!(note.contains("- **Parent:** [[PlayerCharacter]]"));

    let analysis = UnityProjectAnalysis::build(vec![asset]);
    let usages = analysis
        .script_attachments
        .get("PlayerController")
        .expect("PlayerController should have attachments");
    assert_eq!(usages.len(), 1);
    assert_eq!(usages[0].game_object_name, "PlayerCharacter");
    assert_eq!(usages[0].asset_path, "Assets/Prefabs/Hero.prefab");
}

#[test]
fn test_unity_project_scan_end_to_end() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    let assets_dir = root.join("Assets");
    let scripts_dir = assets_dir.join("Scripts");
    let prefabs_dir = assets_dir.join("Prefabs");

    fs::create_dir_all(&scripts_dir).unwrap();
    fs::create_dir_all(&prefabs_dir).unwrap();

    let script_guid = "99998888777766665555444433332222";
    fs::write(scripts_dir.join("Health.cs"), "public class Health {}").unwrap();
    fs::write(
        scripts_dir.join("Health.cs.meta"),
        format!("fileFormatVersion: 2\nguid: {}\n", script_guid),
    )
    .unwrap();

    let prefab_content = format!(
        r#"
--- !u!1 &10
GameObject:
  m_Name: Enemy
  m_Component:
  - component: {{fileID: 11}}
--- !u!114 &11
MonoBehaviour:
  m_GameObject: {{fileID: 10}}
  m_Script: {{fileID: 11500000, guid: {}, type: 3}}
"#,
        script_guid
    );
    fs::write(prefabs_dir.join("Enemy.prefab"), prefab_content).unwrap();

    let analysis = scan_and_analyze_unity(root);
    assert_eq!(analysis.assets.len(), 1);

    let enemy_prefab = &analysis.assets[0];
    assert_eq!(enemy_prefab.game_objects.len(), 1);
    assert_eq!(enemy_prefab.game_objects[0].name, "Enemy");
    assert_eq!(
        enemy_prefab.game_objects[0].attached_scripts[0].script_name,
        "Health"
    );

    let attachments = analysis.script_attachments.get("Health").unwrap();
    assert_eq!(attachments[0].game_object_name, "Enemy");
}
