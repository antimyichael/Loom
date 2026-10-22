use crate::types::{CodeSymbol, SymbolKind};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashSet;

lazy_static! {
    static ref RE_LUAU_TYPE: Regex = Regex::new(r"(?m)^\s*(?:export\s+)?type\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    static ref RE_LUAU_OOP_CLASS: Regex = Regex::new(r"(?m)^\s*(?:local\s+)?([A-Z][a-zA-Z0-9_]*)\s*=\s*(?:\{\}|setmetatable\b)").unwrap();
    static ref RE_LUAU_FUNC_METHOD: Regex = Regex::new(r"(?m)^\s*function\s+([a-zA-Z0-9_]+)[:.]([a-zA-Z0-9_]+)(?:<[^>]*>)?\s*\(").unwrap();
    static ref RE_LUAU_FUNC_GLOBAL_OR_LOCAL: Regex = Regex::new(r"(?m)^\s*(?:local\s+)?function\s+([a-zA-Z0-9_]+)(?:<[^>]*>)?\s*\(").unwrap();
    static ref RE_LUAU_SERVICE: Regex = Regex::new(r#"(?:game|workspace):GetService\s*\(\s*['"]([^'"]+)['"]\s*\)"#).unwrap();
    static ref RE_LUAU_REQUIRE_STR: Regex = Regex::new(r#"require\s*\(\s*['"]([^'"]+)['"]\s*\)"#).unwrap();
    static ref RE_LUAU_REQUIRE_EXPR: Regex = Regex::new(r#"require\s*\(\s*([a-zA-Z0-9_.]+)\s*\)"#).unwrap();
    static ref RE_LUAU_CALL: Regex = Regex::new(r"\b([a-zA-Z_][a-zA-Z0-9_]*)\s*[:(]").unwrap();
    static ref RE_LUAU_CONST: Regex = Regex::new(r"(?m)^\s*(?:local\s+)?([A-Z0-9_]{2,})\s*=").unwrap();
}

const LUAU_KEYWORDS: &[&str] = &[
    "and", "break", "do", "else", "elseif", "end", "false", "for", "function",
    "if", "in", "local", "nil", "not", "or", "repeat", "return", "then",
    "true", "until", "while", "type", "export", "typeof", "game", "workspace",
    "script", "self", "setmetatable", "getmetatable", "require",
];

pub fn extract_symbols_luau(contents: &str, file_path: &str) -> Vec<CodeSymbol> {
    let mut symbols = Vec::new();
    let mut seen = HashSet::new();

    let lines: Vec<&str> = contents.lines().collect();
    let get_line_num = |byte_offset: usize| -> usize {
        let mut count = 1;
        let mut curr = 0;
        for line in &lines {
            curr += line.len() + 1;
            if curr > byte_offset {
                break;
            }
            count += 1;
        }
        count
    };

    let extract_calls = |snippet: &str| -> Option<Vec<String>> {
        let mut calls = Vec::new();
        for cap in RE_LUAU_CALL.captures_iter(snippet) {
            let name = cap[1].to_string();
            if !LUAU_KEYWORDS.contains(&name.as_str()) && name.len() > 1 {
                calls.push(name);
            }
        }
        if calls.is_empty() {
            None
        } else {
            calls.sort();
            calls.dedup();
            Some(calls)
        }
    };

    let extract_body = |start_idx: usize| -> Option<String> {
        let mut depth = 0;
        let mut end_idx = lines.len();

        for (idx, line) in lines.iter().enumerate().skip(start_idx) {
            let trimmed = line.trim();
            if trimmed.starts_with("function")
                || trimmed.starts_with("local function")
                || trimmed.starts_with("if")
                || trimmed.starts_with("for")
                || trimmed.starts_with("while")
                || trimmed.starts_with("do")
                || trimmed.starts_with("repeat")
            {
                depth += 1;
            }
            if trimmed == "end" || trimmed.starts_with("end)") || trimmed.starts_with("end,") || trimmed.starts_with("until") {
                depth -= 1;
                if depth <= 0 {
                    end_idx = idx + 1;
                    break;
                }
            }
        }

        let slice = &lines[start_idx..end_idx.min(lines.len())];
        if slice.len() > 60 {
            Some(slice[..60].join("\n") + "\n...")
        } else {
            Some(slice.join("\n"))
        }
    };

    for cap in RE_LUAU_TYPE.captures_iter(contents) {
        let name = cap[1].to_string();
        if seen.insert(format!("class:{}", name)) {
            let line = get_line_num(cap.get(0).unwrap().start());
            symbols.push(CodeSymbol {
                name,
                kind: SymbolKind::Class,
                file_path: file_path.to_string(),
                body: None,
                calls: None,
                line,
                references: Vec::new(),
                referenced_by: Vec::new(),
                parent_class: None,
            });
        }
    }

    for cap in RE_LUAU_OOP_CLASS.captures_iter(contents) {
        let name = cap[1].to_string();
        if seen.insert(format!("class:{}", name)) {
            let line = get_line_num(cap.get(0).unwrap().start());
            symbols.push(CodeSymbol {
                name,
                kind: SymbolKind::Class,
                file_path: file_path.to_string(),
                body: None,
                calls: None,
                line,
                references: Vec::new(),
                referenced_by: Vec::new(),
                parent_class: None,
            });
        }
    }

    for cap in RE_LUAU_FUNC_METHOD.captures_iter(contents) {
        let parent = cap[1].to_string();
        let method_name = cap[2].to_string();
        let key = format!("method:{}:{}", parent, method_name);
        if seen.insert(key) {
            let offset = cap.get(0).unwrap().start();
            let line = get_line_num(offset);
            let body = extract_body(line.saturating_sub(1));
            let calls = body.as_deref().and_then(extract_calls);

            symbols.push(CodeSymbol {
                name: method_name,
                kind: SymbolKind::Method,
                file_path: file_path.to_string(),
                body,
                calls,
                line,
                references: Vec::new(),
                referenced_by: Vec::new(),
                parent_class: Some(parent),
            });
        }
    }

    for cap in RE_LUAU_FUNC_GLOBAL_OR_LOCAL.captures_iter(contents) {
        let name = cap[1].to_string();
        let key = format!("function:{}", name);
        if seen.insert(key) {
            let offset = cap.get(0).unwrap().start();
            let line = get_line_num(offset);
            let body = extract_body(line.saturating_sub(1));
            let calls = body.as_deref().and_then(extract_calls);

            symbols.push(CodeSymbol {
                name,
                kind: SymbolKind::Function,
                file_path: file_path.to_string(),
                body,
                calls,
                line,
                references: Vec::new(),
                referenced_by: Vec::new(),
                parent_class: None,
            });
        }
    }

    for cap in RE_LUAU_SERVICE.captures_iter(contents) {
        let service = cap[1].to_string();
        let key = format!("import:service:{}", service);
        if seen.insert(key) {
            let line = get_line_num(cap.get(0).unwrap().start());
            symbols.push(CodeSymbol {
                name: service,
                kind: SymbolKind::Import,
                file_path: file_path.to_string(),
                body: None,
                calls: None,
                line,
                references: Vec::new(),
                referenced_by: Vec::new(),
                parent_class: None,
            });
        }
    }

    for cap in RE_LUAU_REQUIRE_STR.captures_iter(contents) {
        let import_target = cap[1].to_string();
        let key = format!("import:str:{}", import_target);
        if seen.insert(key) {
            let line = get_line_num(cap.get(0).unwrap().start());
            symbols.push(CodeSymbol {
                name: import_target,
                kind: SymbolKind::Import,
                file_path: file_path.to_string(),
                body: None,
                calls: None,
                line,
                references: Vec::new(),
                referenced_by: Vec::new(),
                parent_class: None,
            });
        }
    }

    for cap in RE_LUAU_REQUIRE_EXPR.captures_iter(contents) {
        let import_target = cap[1].to_string();
        if import_target.starts_with('"') || import_target.starts_with('\'') {
            continue;
        }
        let key = format!("import:expr:{}", import_target);
        if seen.insert(key) {
            let line = get_line_num(cap.get(0).unwrap().start());
            symbols.push(CodeSymbol {
                name: import_target,
                kind: SymbolKind::Import,
                file_path: file_path.to_string(),
                body: None,
                calls: None,
                line,
                references: Vec::new(),
                referenced_by: Vec::new(),
                parent_class: None,
            });
        }
    }

    for cap in RE_LUAU_CONST.captures_iter(contents) {
        let name = cap[1].to_string();
        let key = format!("var:{}", name);
        if seen.insert(key) {
            let line = get_line_num(cap.get(0).unwrap().start());
            symbols.push(CodeSymbol {
                name,
                kind: SymbolKind::Variable,
                file_path: file_path.to_string(),
                body: None,
                calls: None,
                line,
                references: Vec::new(),
                referenced_by: Vec::new(),
                parent_class: None,
            });
        }
    }

    symbols
}
