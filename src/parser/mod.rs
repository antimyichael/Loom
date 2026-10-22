pub mod bash;
pub mod c_lang;
pub mod common;
pub mod cpp;
pub mod csharp;
pub mod go_lang;
pub mod java;
pub mod kotlin;
pub mod luau;
pub mod python;
pub mod ruby_lang;
pub mod rust_lang;
pub mod typescript;
pub mod universal;

use crate::types::CodeSymbol;
use std::path::Path;

pub const SUPPORTED_EXTENSIONS: &[&str] = &[

    ".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs",
    ".py", ".pyw",
    ".cs",
    ".java",
    ".kt", ".kts",
    ".cpp", ".cc", ".cxx", ".hpp", ".hh",
    ".c", ".h",
    ".rs",
    ".go",
    ".rb",
    ".sh", ".bash", ".zsh",

    ".php",
    ".swift",
    ".scala", ".sc",
    ".luau",
    ".lua",
    ".r", ".R",
    ".dart",
    ".jl",
    ".ex", ".exs",
    ".erl", ".hrl",
    ".hs",
    ".ml", ".mli",
    ".clj", ".cljs", ".cljc",
    ".zig",
    ".nim",
    ".pl", ".pm",
    ".sql",
    ".sol",
    ".gd",
    ".ps1", ".psm1",
    ".fs", ".fsi", ".fsx",
    ".groovy", ".gvy",
    ".f90", ".f", ".for",
    ".pas", ".dpr",
    ".adb", ".ads",
    ".elm",
    ".v",
    ".wgsl", ".glsl", ".hlsl",
    ".tf", ".hcl",
    ".proto",
    ".graphql", ".gql",
    ".cls",
    ".d",
    ".cr",
];

pub fn is_supported_file(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    SUPPORTED_EXTENSIONS
        .iter()
        .any(|ext| path_str.ends_with(ext))
}

pub fn get_language_for_extension(ext: &str) -> &'static str {
    match ext {
        ".ts" => "typescript",
        ".tsx" => "tsx",
        ".js" | ".mjs" | ".cjs" => "javascript",
        ".jsx" => "jsx",
        ".py" | ".pyw" => "python",
        ".cs" => "csharp",
        ".java" => "java",
        ".kt" | ".kts" => "kotlin",
        ".cpp" | ".cc" | ".cxx" | ".hpp" | ".hh" => "cpp",
        ".c" | ".h" => "c",
        ".rs" => "rust",
        ".go" => "go",
        ".rb" => "ruby",
        ".sh" | ".bash" | ".zsh" => "bash",
        ".luau" => "luau",
        ".lua" => "lua",
        _ => {
            if let Some(pattern) = universal::get_universal_pattern(ext) {
                pattern.language_name
            } else {
                "text"
            }
        }
    }
}

pub fn parse_file(file_path: &str, contents: &str) -> Vec<CodeSymbol> {
    let path = Path::new(file_path);
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e))
        .unwrap_or_default();

    match ext.as_str() {

        ".ts" => typescript::extract_symbols_typescript(contents, file_path, false),
        ".tsx" | ".jsx" => typescript::extract_symbols_typescript(contents, file_path, true),
        ".js" | ".mjs" | ".cjs" => typescript::extract_symbols_typescript(contents, file_path, false),
        ".py" | ".pyw" => python::extract_symbols_python(contents, file_path),
        ".cs" => csharp::extract_symbols_csharp(contents, file_path),
        ".java" => java::extract_symbols_java(contents, file_path),
        ".kt" | ".kts" => kotlin::extract_symbols_kotlin(contents, file_path),
        ".cpp" | ".cc" | ".cxx" | ".hpp" | ".hh" => cpp::extract_symbols_cpp(contents, file_path),
        ".c" | ".h" => c_lang::extract_symbols_c(contents, file_path),
        ".rs" => rust_lang::extract_symbols_rust(contents, file_path),
        ".go" => go_lang::extract_symbols_go(contents, file_path),
        ".rb" => ruby_lang::extract_symbols_ruby(contents, file_path),
        ".sh" | ".bash" | ".zsh" => bash::extract_symbols_bash(contents, file_path),
        ".luau" | ".lua" => luau::extract_symbols_luau(contents, file_path),

        _ => {
            if let Some(pattern) = universal::get_universal_pattern(&ext) {
                universal::extract_symbols_universal(contents, file_path, &pattern)
            } else {
                Vec::new()
            }
        }
    }
}
