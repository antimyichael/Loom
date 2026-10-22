use loom_code::parser::luau::extract_symbols_luau;
use loom_code::parser::parse_file;
use loom_code::roblox::rbx_xml::parse_rbx_xml;
use loom_code::roblox::rojo::find_and_parse_rojo_project;
use loom_code::roblox::{build_roblox_model_note, is_roblox_project, scan_and_analyze_roblox};
use loom_code::types::SymbolKind;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_luau_parsing() {
    let code = r#"
export type Vector2D = { x: number, y: number }
type PlayerState = "Idle" | "Walking" | "Running"

local Player = {}
Player.__index = Player

local Players = game:GetService("Players")
local ReplicatedStorage = game:GetService("ReplicatedStorage")
local Utils = require(ReplicatedStorage.Common.Utils)
local Signal = require(script.Parent.Signal)
local Config = require("./Config")

local MAX_HEALTH = 100

function Player.new(name: string): Player
    local self = setmetatable({}, Player)
    self.Name = name
    self.Health = MAX_HEALTH
    return self
end

function Player:TakeDamage(amount: number): boolean
    self.Health -= amount
    Signal:Fire("Damaged", self.Health)
    return self.Health <= 0
end

local function internalHelper()
    task.spawn(function()
        print("Async task")
    end)
end
"#;

    let symbols = extract_symbols_luau(code, "src/shared/Player.luau");

    assert!(symbols.iter().any(|s| s.name == "Vector2D" && s.kind == SymbolKind::Class));
    assert!(symbols.iter().any(|s| s.name == "PlayerState" && s.kind == SymbolKind::Class));
    assert!(symbols.iter().any(|s| s.name == "Player" && s.kind == SymbolKind::Class));

    let method_new = symbols.iter().find(|s| s.name == "new");
    assert!(method_new.is_some());
    assert_eq!(method_new.unwrap().parent_class.as_deref(), Some("Player"));

    let method_damage = symbols.iter().find(|s| s.name == "TakeDamage");
    assert!(method_damage.is_some());
    assert_eq!(method_damage.unwrap().parent_class.as_deref(), Some("Player"));

    let helper = symbols.iter().find(|s| s.name == "internalHelper");
    assert!(helper.is_some());
    assert_eq!(helper.unwrap().kind, SymbolKind::Function);

    assert!(symbols.iter().any(|s| s.name == "Players" && s.kind == SymbolKind::Import));
    assert!(symbols.iter().any(|s| s.name == "ReplicatedStorage" && s.kind == SymbolKind::Import));
    assert!(symbols.iter().any(|s| s.name == "ReplicatedStorage.Common.Utils" && s.kind == SymbolKind::Import));
    assert!(symbols.iter().any(|s| s.name == "script.Parent.Signal" && s.kind == SymbolKind::Import));
    assert!(symbols.iter().any(|s| s.name == "./Config" && s.kind == SymbolKind::Import));

    assert!(symbols.iter().any(|s| s.name == "MAX_HEALTH" && s.kind == SymbolKind::Variable));
}

#[test]
fn test_parse_file_dispatch_for_luau() {
    let code = "function greet(): ()\n  print('hello')\nend";
    let symbols = parse_file("Combat.server.luau", code);
    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].name, "greet");
}

#[test]
fn test_rojo_project_mapping() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let project_json = r#"{
  "name": "LoomGame",
  "tree": {
    "$className": "DataModel",
    "ReplicatedStorage": {
      "Common": {
        "$path": "src/shared"
      }
    },
    "ServerScriptService": {
      "$path": "src/server"
    },
    "StarterPlayer": {
      "StarterPlayerScripts": {
        "$path": "src/client"
      }
    }
  }
}"#;

    fs::write(root.join("default.project.json"), project_json).unwrap();

    let shared_dir = root.join("src").join("shared");
    fs::create_dir_all(&shared_dir).unwrap();
    fs::write(shared_dir.join("Utils.luau"), "return {}").unwrap();

    let server_dir = root.join("src").join("server").join("Combat");
    fs::create_dir_all(&server_dir).unwrap();
    fs::write(server_dir.join("Hitbox.server.luau"), "print('Hitbox')").unwrap();

    let client_dir = root.join("src").join("client").join("Controllers");
    fs::create_dir_all(&client_dir).unwrap();
    fs::write(client_dir.join("Camera.client.luau"), "print('Camera')").unwrap();

    assert!(is_roblox_project(root));

    let mapping = find_and_parse_rojo_project(root).unwrap();

    assert_eq!(
        mapping.get("src/shared/Utils.luau").map(|s| s.as_str()),
        Some("game.ReplicatedStorage.Common.Utils")
    );
    assert_eq!(
        mapping.get("src/server/Combat/Hitbox.server.luau").map(|s| s.as_str()),
        Some("game.ServerScriptService.Combat.Hitbox")
    );
    assert_eq!(
        mapping.get("src/client/Controllers/Camera.client.luau").map(|s| s.as_str()),
        Some("game.StarterPlayer.StarterPlayerScripts.Controllers.Camera")
    );
}

#[test]
fn test_rbx_xml_model_parsing() {
    let xml = r#"<roblox version="4">
  <Item class="Model">
    <Properties>
      <string name="Name">Vehicle</string>
    </Properties>
    <Item class="Part">
      <Properties>
        <string name="Name">Chassis</string>
      </Properties>
    </Item>
    <Item class="Script">
      <Properties>
        <string name="Name">DriveScript</string>
        <ProtectedString name="Source">print('Vroom')</ProtectedString>
      </Properties>
    </Item>
  </Item>
</roblox>"#;

    let model = parse_rbx_xml("assets/Vehicle.rbxmx", xml).unwrap();
    assert_eq!(model.file_path, "assets/Vehicle.rbxmx");
    assert_eq!(model.total_instances, 3);
    assert_eq!(model.scripts.len(), 1);
    assert_eq!(model.scripts[0].script_name, "DriveScript");
    assert_eq!(model.scripts[0].script_type, "Script");
    assert_eq!(model.scripts[0].parent_name, "Vehicle");

    let note = build_roblox_model_note(&model);
    assert!(note.contains("# Vehicle.rbxmx"));
    assert!(note.contains("DriveScript"));
    assert!(note.contains("Chassis"));
}

#[test]
fn test_roblox_project_scan_end_to_end() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let project_json = r#"{
  "name": "Game",
  "tree": {
    "$className": "DataModel",
    "ReplicatedStorage": {
      "$path": "src"
    }
  }
}"#;

    fs::write(root.join("default.project.json"), project_json).unwrap();

    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("GameManager.luau"), "local M = {}\nreturn M").unwrap();

    let models = root.join("models");
    fs::create_dir_all(&models).unwrap();
    let model_xml = r#"<roblox version="4">
  <Item class="Model">
    <Properties>
      <string name="Name">Coin</string>
    </Properties>
    <Item class="Script">
      <Properties>
        <string name="Name">CoinPickup</string>
      </Properties>
    </Item>
  </Item>
</roblox>"#;
    fs::write(models.join("Coin.rbxmx"), model_xml).unwrap();

    let analysis = scan_and_analyze_roblox(root);
    assert_eq!(analysis.models.len(), 1);
    assert!(analysis.file_to_datamodel.contains_key("src/GameManager.luau"));
    assert!(analysis.script_attachments.contains_key("CoinPickup"));
}
