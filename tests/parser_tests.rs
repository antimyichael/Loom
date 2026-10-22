use loom_code::parser::{get_language_for_extension, parse_file};
use loom_code::types::SymbolKind;

#[test]
fn test_typescript_parsing() {
    let code = r#"
import { helper } from './utils';
export const API_URL = "https://api.example.com";
export class UserService {
    private db: Database;
    public getUser(id: string): User {
        return this.db.find(id);
    }
}
export function formatUser(user: User): string {
    return user.name.toUpperCase();
}
"#;
    let symbols = parse_file("src/user.ts", code);
    assert!(symbols.iter().any(|s| s.name == "UserService" && s.kind == SymbolKind::Class));
    assert!(symbols.iter().any(|s| s.name == "getUser" && s.kind == SymbolKind::Method && s.parent_class.as_deref() == Some("UserService")));
    assert!(symbols.iter().any(|s| s.name == "formatUser" && s.kind == SymbolKind::Function));
    assert!(symbols.iter().any(|s| s.name == "API_URL" && s.kind == SymbolKind::Variable));
}

#[test]
fn test_rust_parsing() {
    let code = r#"
use std::collections::HashMap;

pub struct CacheManager {
    store: HashMap<String, String>,
}

impl CacheManager {
    pub fn new() -> Self {
        Self { store: HashMap::new() }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.store.get(key)
    }
}

pub fn global_helper() {
    println!("Helper");
}
"#;
    let symbols = parse_file("src/cache.rs", code);
    assert!(symbols.iter().any(|s| s.name == "CacheManager" && s.kind == SymbolKind::Class));
    let get_method = symbols.iter().find(|s| s.name == "get");
    assert!(get_method.is_some());
    assert_eq!(get_method.unwrap().kind, SymbolKind::Method);
    assert_eq!(get_method.unwrap().parent_class.as_deref(), Some("CacheManager"));
    assert!(symbols.iter().any(|s| s.name == "global_helper" && s.kind == SymbolKind::Function));
    assert!(symbols.iter().any(|s| s.kind == SymbolKind::Import));
}

#[test]
fn test_go_parsing() {
    let code = r#"
package service

import "fmt"

type Server struct {
    port int
}

func (s *Server) Start() error {
    fmt.Println("Starting")
    return nil
}

func NewServer(port int) *Server {
    return &Server{port: port}
}
"#;
    let symbols = parse_file("server.go", code);
    assert!(symbols.iter().any(|s| s.name == "Server" && s.kind == SymbolKind::Class));
    let start_method = symbols.iter().find(|s| s.name == "Start");
    assert!(start_method.is_some());
    assert_eq!(start_method.unwrap().kind, SymbolKind::Method);
    assert_eq!(start_method.unwrap().parent_class.as_deref(), Some("Server"));
    assert!(symbols.iter().any(|s| s.name == "NewServer" && s.kind == SymbolKind::Function));
    assert!(symbols.iter().any(|s| s.name == "fmt" && s.kind == SymbolKind::Import));
}

#[test]
fn test_ruby_parsing() {
    let code = r#"
require 'json'

class ReportGenerator
  def generate(data)
    format_data(data)
  end

  def format_data(d)
    d.to_s
  end
end
"#;
    let symbols = parse_file("report.rb", code);
    assert!(symbols.iter().any(|s| s.name == "ReportGenerator" && s.kind == SymbolKind::Class));
    let gen_method = symbols.iter().find(|s| s.name == "generate");
    assert!(gen_method.is_some());
    assert_eq!(gen_method.unwrap().kind, SymbolKind::Method);
    assert_eq!(gen_method.unwrap().parent_class.as_deref(), Some("ReportGenerator"));
}

#[test]
fn test_c_parsing() {
    let code = r#"
#include <stdio.h>

struct Vector2 {
    float x;
    float y;
};

void normalize(struct Vector2* v) {
    printf("normalizing");
}
"#;
    let symbols = parse_file("math.c", code);
    assert!(symbols.iter().any(|s| s.name == "Vector2" && s.kind == SymbolKind::Class));
    assert!(symbols.iter().any(|s| s.name == "normalize" && s.kind == SymbolKind::Function));
    assert!(symbols.iter().any(|s| s.name == "stdio.h" && s.kind == SymbolKind::Import));
}

#[test]
fn test_universal_php_parsing() {
    let code = r#"
<?php
use App\Models\User;

class AuthController {
    public function login($request) {
        authenticate($request);
    }
}
"#;
    let symbols = parse_file("AuthController.php", code);
    assert!(symbols.iter().any(|s| s.name == "AuthController" && s.kind == SymbolKind::Class));
    assert!(symbols.iter().any(|s| s.name == "login" && s.kind == SymbolKind::Method));
}

#[test]
fn test_universal_swift_parsing() {
    let code = r#"
import Foundation

public class NetworkManager {
    public func fetchData() {
        process();
    }
}
"#;
    let symbols = parse_file("Network.swift", code);
    assert!(symbols.iter().any(|s| s.name == "NetworkManager" && s.kind == SymbolKind::Class));
    assert!(symbols.iter().any(|s| s.name == "fetchData" && s.kind == SymbolKind::Method));
    assert!(symbols.iter().any(|s| s.name == "Foundation" && s.kind == SymbolKind::Import));
}

#[test]
fn test_universal_solidity_parsing() {
    let code = r#"
import "./IERC20.sol";

contract TokenVault {
    function deposit(uint256 amount) public {
        transfer(amount);
    }
}
"#;
    let symbols = parse_file("TokenVault.sol", code);
    assert!(symbols.iter().any(|s| s.name == "TokenVault" && s.kind == SymbolKind::Class));
    assert!(symbols.iter().any(|s| s.name == "deposit" && s.kind == SymbolKind::Method));
}

#[test]
fn test_universal_zig_parsing() {
    let code = r#"
const std = @import("std");

pub const Point = struct {
    x: i32,
    y: i32,
};

pub fn calculateDistance() void {
    print();
}
"#;
    let symbols = parse_file("math.zig", code);
    assert!(symbols.iter().any(|s| s.name == "Point" && s.kind == SymbolKind::Class));
    assert!(symbols.iter().any(|s| s.name == "calculateDistance" && s.kind == SymbolKind::Function));
}

#[test]
fn test_universal_sql_parsing() {
    let code = r#"
CREATE TABLE users (
    id INT PRIMARY KEY,
    name VARCHAR(100)
);

CREATE FUNCTION calculate_tax(amount DECIMAL) RETURNS DECIMAL;
"#;
    let symbols = parse_file("schema.sql", code);
    assert!(symbols.iter().any(|s| s.name == "users" && s.kind == SymbolKind::Class));
    assert!(symbols.iter().any(|s| s.name == "calculate_tax" && s.kind == SymbolKind::Function));
}

#[test]
fn test_universal_gdscript_parsing() {
    let code = r#"
class_name PlayerNode
extends KinematicBody2D

func _physics_process(delta):
    move_and_slide()
"#;
    let symbols = parse_file("Player.gd", code);
    assert!(symbols.iter().any(|s| s.name == "PlayerNode" && s.kind == SymbolKind::Class));
    assert!(symbols.iter().any(|s| s.name == "_physics_process"));
}

#[test]
fn test_language_extensions() {
    assert_eq!(get_language_for_extension(".ts"), "typescript");
    assert_eq!(get_language_for_extension(".py"), "python");
    assert_eq!(get_language_for_extension(".rs"), "rust");
    assert_eq!(get_language_for_extension(".go"), "go");
    assert_eq!(get_language_for_extension(".rb"), "ruby");
    assert_eq!(get_language_for_extension(".php"), "php");
    assert_eq!(get_language_for_extension(".swift"), "swift");
    assert_eq!(get_language_for_extension(".scala"), "scala");
    assert_eq!(get_language_for_extension(".lua"), "lua");
    assert_eq!(get_language_for_extension(".r"), "r");
    assert_eq!(get_language_for_extension(".dart"), "dart");
    assert_eq!(get_language_for_extension(".zig"), "zig");
    assert_eq!(get_language_for_extension(".sol"), "solidity");
    assert_eq!(get_language_for_extension(".gd"), "gdscript");
    assert_eq!(get_language_for_extension(".sql"), "sql");
}
