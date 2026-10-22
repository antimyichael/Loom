use crate::types::{CodeSymbol, SymbolKind};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashSet;

pub struct LanguagePattern {
    pub language_name: &'static str,
    pub class_re: Option<&'static Regex>,
    pub func_re: Option<&'static Regex>,
    #[allow(dead_code)]
    pub var_re: Option<&'static Regex>,
    pub import_re: Option<&'static Regex>,
    pub call_re: Option<&'static Regex>,
}

lazy_static! {

    static ref RE_PHP_CLASS: Regex = Regex::new(r"(?m)^\s*(?:abstract\s+|final\s+)?(?:class|interface|trait|enum)\s+([a-zA-Z_\x7f-\xff][a-zA-Z0-9_\x7f-\xff]*)").unwrap();
    static ref RE_PHP_FUNC: Regex = Regex::new(r"(?m)^\s*(?:public\s+|protected\s+|private\s+|static\s+|final\s+)*function\s+([a-zA-Z_\x7f-\xff][a-zA-Z0-9_\x7f-\xff]*)").unwrap();
    static ref RE_PHP_IMPORT: Regex = Regex::new(r#"(?m)^\s*(?:use|require|require_once|include|include_once)\s+['"]?([^;'"]+)['"]?"#).unwrap();

    static ref RE_SWIFT_CLASS: Regex = Regex::new(r"(?m)^\s*(?:public\s+|private\s+|internal\s+|fileprivate\s+|open\s+)?(?:final\s+)?(?:class|struct|protocol|enum|extension|actor)\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    static ref RE_SWIFT_FUNC: Regex = Regex::new(r"(?m)^\s*(?:public\s+|private\s+|internal\s+|fileprivate\s+|open\s+)?(?:override\s+|static\s+|class\s+|mutating\s+)?func\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    static ref RE_SWIFT_IMPORT: Regex = Regex::new(r"(?m)^\s*import\s+([a-zA-Z0-9_]+)").unwrap();

    static ref RE_SCALA_CLASS: Regex = Regex::new(r"(?m)^\s*(?:case\s+)?(?:class|object|trait|enum)\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    static ref RE_SCALA_FUNC: Regex = Regex::new(r"(?m)^\s*(?:override\s+)?def\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    static ref RE_SCALA_IMPORT: Regex = Regex::new(r"(?m)^\s*import\s+([a-zA-Z0-9_.]+)").unwrap();

    static ref RE_LUA_FUNC: Regex = Regex::new(r"(?m)^\s*(?:local\s+)?function\s+([a-zA-Z0-9_.:]+)").unwrap();
    static ref RE_LUA_IMPORT: Regex = Regex::new(r#"(?m)(?:require|dofile|loadfile)\s*\(?['"]([^'"]+)['"]\)"#).unwrap();

    static ref RE_R_FUNC: Regex = Regex::new(r"(?m)^\s*([a-zA-Z0-9_.]+)\s*(?:<-|=)\s*function\b").unwrap();
    static ref RE_R_IMPORT: Regex = Regex::new(r#"(?m)(?:library|require)\s*\((['"]?[a-zA-Z0-9_.]+['"]?)\)"#).unwrap();

    static ref RE_DART_CLASS: Regex = Regex::new(r"(?m)^\s*(?:abstract\s+|mixin\s+)?(?:class|enum|extension)\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    static ref RE_DART_FUNC: Regex = Regex::new(r"(?m)^\s*(?:[a-zA-Z0-9_<>,?\s]+\s+)?([a-zA-Z_][a-zA-Z0-9_]*)\s*\([^)]*\)\s*(?:async\s*)?(?:\{|=>)").unwrap();
    static ref RE_DART_IMPORT: Regex = Regex::new(r#"(?m)^\s*import\s+['"]([^'"]+)['"]"#).unwrap();

    static ref RE_JULIA_CLASS: Regex = Regex::new(r"(?m)^\s*(?:mutable\s+)?struct\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    static ref RE_JULIA_FUNC: Regex = Regex::new(r"(?m)^\s*function\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    static ref RE_JULIA_IMPORT: Regex = Regex::new(r"(?m)^\s*(?:using|import)\s+([a-zA-Z0-9_.,\s]+)").unwrap();

    static ref RE_ELIXIR_CLASS: Regex = Regex::new(r"(?m)^\s*defmodule\s+([A-Za-z0-9_.]+)").unwrap();
    static ref RE_ELIXIR_FUNC: Regex = Regex::new(r"(?m)^\s*defp?\s+([a-z0-9_?!]+)").unwrap();
    static ref RE_ELIXIR_IMPORT: Regex = Regex::new(r"(?m)^\s*(?:import|alias|use|require)\s+([A-Za-z0-9_.]+)").unwrap();

    static ref RE_ERLANG_CLASS: Regex = Regex::new(r"(?m)^-module\(([^)]+)\)\.").unwrap();
    static ref RE_ERLANG_FUNC: Regex = Regex::new(r"(?m)^([a-z][a-zA-Z0-9_]*)\s*\([^)]*\)\s*->").unwrap();
    static ref RE_ERLANG_IMPORT: Regex = Regex::new(r"(?m)^-include\(([^)]+)\)\.").unwrap();

    static ref RE_HASKELL_CLASS: Regex = Regex::new(r"(?m)^\s*(?:data|type|newtype|class)\s+([A-Z][a-zA-Z0-9_']*)").unwrap();
    static ref RE_HASKELL_FUNC: Regex = Regex::new(r"(?m)^([a-z][a-zA-Z0-9_']*)\s*::").unwrap();
    static ref RE_HASKELL_IMPORT: Regex = Regex::new(r"(?m)^\s*import\s+(?:qualified\s+)?([A-Za-z0-9_.]+)").unwrap();

    static ref RE_OCAML_CLASS: Regex = Regex::new(r"(?m)^\s*(?:module|type)\s+([A-Za-z0-9_']+)").unwrap();
    static ref RE_OCAML_FUNC: Regex = Regex::new(r"(?m)^\s*let\s+(?:rec\s+)?([a-z_][a-zA-Z0-9_']*)").unwrap();
    static ref RE_OCAML_IMPORT: Regex = Regex::new(r"(?m)^\s*open\s+([A-Za-z0-9_.]+)").unwrap();

    static ref RE_CLOJURE_CLASS: Regex = Regex::new(r"\((?:ns|defprotocol|defrecord)\s+([a-zA-Z0-9_.-]+)").unwrap();
    static ref RE_CLOJURE_FUNC: Regex = Regex::new(r"\((?:defn|defn-|defmacro|defmulti)\s+([a-zA-Z0-9_.-]+)").unwrap();

    static ref RE_ZIG_CLASS: Regex = Regex::new(r"(?m)^\s*(?:pub\s+)?const\s+([A-Z][a-zA-Z0-9_]*)\s*=\s*(?:struct|enum|union)\b").unwrap();
    static ref RE_ZIG_FUNC: Regex = Regex::new(r"(?m)^\s*(?:pub\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(").unwrap();
    static ref RE_ZIG_IMPORT: Regex = Regex::new(r#"@import\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap();

    static ref RE_NIM_CLASS: Regex = Regex::new(r"(?m)^\s*type\s+([a-zA-Z_][a-zA-Z0-9_]*)\*?\s*=").unwrap();
    static ref RE_NIM_FUNC: Regex = Regex::new(r"(?m)^\s*(?:proc|func|method|iterator|template|macro)\s+([a-zA-Z_][a-zA-Z0-9_]*)\*?\s*\(").unwrap();
    static ref RE_NIM_IMPORT: Regex = Regex::new(r"(?m)^\s*import\s+([a-zA-Z0-9_,\s]+)").unwrap();

    static ref RE_PERL_CLASS: Regex = Regex::new(r"(?m)^\s*package\s+([a-zA-Z0-9_:]+)").unwrap();
    static ref RE_PERL_FUNC: Regex = Regex::new(r"(?m)^\s*sub\s+([a-zA-Z0-9_]+)").unwrap();
    static ref RE_PERL_IMPORT: Regex = Regex::new(r"(?m)^\s*(?:use|require)\s+([a-zA-Z0-9_:]+)").unwrap();

    static ref RE_SQL_CLASS: Regex = Regex::new(r"(?mi)^\s*CREATE\s+(?:OR\s+REPLACE\s+)?(?:TABLE|VIEW)\s+(?:IF\s+NOT\s+EXISTS\s+)?([a-zA-Z0-9_.]+)").unwrap();
    static ref RE_SQL_FUNC: Regex = Regex::new(r"(?mi)^\s*CREATE\s+(?:OR\s+REPLACE\s+)?(?:FUNCTION|PROCEDURE)\s+([a-zA-Z0-9_.]+)").unwrap();

    static ref RE_SOLIDITY_CLASS: Regex = Regex::new(r"(?m)^\s*(?:contract|interface|library)\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    static ref RE_SOLIDITY_FUNC: Regex = Regex::new(r"(?m)^\s*function\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    static ref RE_SOLIDITY_IMPORT: Regex = Regex::new(r#"(?m)^\s*import\s+['"]([^'"]+)['"]"#).unwrap();

    static ref RE_GDSCRIPT_CLASS: Regex = Regex::new(r"(?m)^\s*class_name\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    static ref RE_GDSCRIPT_FUNC: Regex = Regex::new(r"(?m)^\s*func\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    static ref RE_GDSCRIPT_IMPORT: Regex = Regex::new(r#"(?m)(?:preload|load)\s*\(\s*['"]([^'"]+)['"]\s*\)"#).unwrap();

    static ref RE_PWSH_CLASS: Regex = Regex::new(r"(?mi)^\s*class\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    static ref RE_PWSH_FUNC: Regex = Regex::new(r"(?mi)^\s*function\s+([a-zA-Z0-9_-]+)").unwrap();

    static ref RE_FSHARP_CLASS: Regex = Regex::new(r"(?m)^\s*type\s+([A-Za-z0-9_]+)").unwrap();
    static ref RE_FSHARP_FUNC: Regex = Regex::new(r"(?m)^\s*let\s+(?:rec\s+)?([a-zA-Z0-9_]+)").unwrap();
    static ref RE_FSHARP_IMPORT: Regex = Regex::new(r"(?m)^\s*open\s+([A-Za-z0-9_.]+)").unwrap();

    static ref RE_GROOVY_CLASS: Regex = Regex::new(r"(?m)^\s*(?:class|interface|trait|enum)\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    static ref RE_GROOVY_FUNC: Regex = Regex::new(r"(?m)^\s*(?:def\s+)?([a-zA-Z_][a-zA-Z0-9_]*)\s*\([^)]*\)\s*\{").unwrap();
    static ref RE_GROOVY_IMPORT: Regex = Regex::new(r"(?m)^\s*import\s+([a-zA-Z0-9_.*]+)").unwrap();

    static ref RE_FORTRAN_CLASS: Regex = Regex::new(r"(?mi)^\s*module\s+([a-zA-Z0-9_]+)").unwrap();
    static ref RE_FORTRAN_FUNC: Regex = Regex::new(r"(?mi)^\s*(?:subroutine|function)\s+([a-zA-Z0-9_]+)").unwrap();

    static ref RE_PASCAL_CLASS: Regex = Regex::new(r"(?mi)^\s*([a-zA-Z0-9_]+)\s*=\s*(?:class|interface|record)\b").unwrap();
    static ref RE_PASCAL_FUNC: Regex = Regex::new(r"(?mi)^\s*(?:procedure|function)\s+([a-zA-Z0-9_.:]+)").unwrap();

    static ref RE_ADA_CLASS: Regex = Regex::new(r"(?mi)^\s*package\s+(?:body\s+)?([a-zA-Z0-9_.]+)").unwrap();
    static ref RE_ADA_FUNC: Regex = Regex::new(r"(?mi)^\s*(?:procedure|function)\s+([a-zA-Z0-9_]+)").unwrap();

    static ref RE_ELM_CLASS: Regex = Regex::new(r"(?m)^\s*type\s+(?:alias\s+)?([A-Z][a-zA-Z0-9_]*)").unwrap();
    static ref RE_ELM_FUNC: Regex = Regex::new(r"(?m)^([a-z][a-zA-Z0-9_]*)\s*:").unwrap();
    static ref RE_ELM_IMPORT: Regex = Regex::new(r"(?m)^\s*import\s+([A-Za-z0-9_.]+)").unwrap();

    static ref RE_V_CLASS: Regex = Regex::new(r"(?m)^\s*(?:pub\s+)?(?:struct|interface|enum)\s+([A-Z][a-zA-Z0-9_]*)").unwrap();
    static ref RE_V_FUNC: Regex = Regex::new(r"(?m)^\s*(?:pub\s+)?fn\s+(?:\([^)]*\)\s+)?([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    static ref RE_V_IMPORT: Regex = Regex::new(r"(?m)^\s*import\s+([a-zA-Z0-9_.]+)").unwrap();

    static ref RE_SHADER_CLASS: Regex = Regex::new(r"(?m)^\s*struct\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    static ref RE_SHADER_FUNC: Regex = Regex::new(r"(?m)^\s*(?:fn\s+|void\s+|float\s+|vec[234]\s+|mat[234]\s+)?([a-zA-Z_][a-zA-Z0-9_]*)\s*\([^)]*\)\s*(?:->\s*[a-zA-Z0-9_]+\s*)?\{").unwrap();

    static ref RE_HCL_CLASS: Regex = Regex::new(r#"(?m)^\s*(?:resource|data|module)\s+["']([^"']+)["']\s+["']([^"']+)["']"#).unwrap();

    static ref RE_PROTO_CLASS: Regex = Regex::new(r"(?m)^\s*(?:message|enum|service)\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    static ref RE_PROTO_IMPORT: Regex = Regex::new(r#"(?m)^\s*import\s+["']([^"']+)["']"#).unwrap();

    static ref RE_GQL_CLASS: Regex = Regex::new(r"(?m)^\s*(?:type|interface|input|enum|union)\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();

    static ref RE_APEX_CLASS: Regex = Regex::new(r"(?mi)^\s*(?:public|private|global)\s+(?:virtual|abstract)?\s*(?:class|interface|enum)\s+([a-zA-Z0-9_]+)").unwrap();
    static ref RE_APEX_FUNC: Regex = Regex::new(r"(?mi)^\s*(?:public|private|global|static|testMethod|\s+)*\s+[a-zA-Z0-9_<>,]+\s+([a-zA-Z0-9_]+)\s*\(").unwrap();

    static ref RE_D_CLASS: Regex = Regex::new(r"(?m)^\s*(?:class|struct|interface|enum)\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    static ref RE_D_FUNC: Regex = Regex::new(r"(?m)^\s*(?:[a-zA-Z0-9_]+\s+)+([a-zA-Z_][a-zA-Z0-9_]*)\s*\([^)]*\)\s*\{").unwrap();
    static ref RE_D_IMPORT: Regex = Regex::new(r"(?m)^\s*import\s+([a-zA-Z0-9_.,\s]+);").unwrap();

    static ref RE_CRYSTAL_CLASS: Regex = Regex::new(r"(?m)^\s*(?:class|module|struct|enum)\s+([A-Z][a-zA-Z0-9_]*)").unwrap();
    static ref RE_CRYSTAL_FUNC: Regex = Regex::new(r"(?m)^\s*def\s+([a-zA-Z_][a-zA-Z0-9_!?]*)").unwrap();
    static ref RE_CRYSTAL_IMPORT: Regex = Regex::new(r#"(?m)^\s*require\s+["']([^"']+)["']"#).unwrap();

    static ref RE_CALL: Regex = Regex::new(r"\b([a-zA-Z_][a-zA-Z0-9_]*)\s*\(").unwrap();
}

pub fn get_universal_pattern(ext: &str) -> Option<LanguagePattern> {
    match ext {
        ".php" => Some(LanguagePattern {
            language_name: "php",
            class_re: Some(&RE_PHP_CLASS),
            func_re: Some(&RE_PHP_FUNC),
            var_re: None,
            import_re: Some(&RE_PHP_IMPORT),
            call_re: Some(&RE_CALL),
        }),
        ".swift" => Some(LanguagePattern {
            language_name: "swift",
            class_re: Some(&RE_SWIFT_CLASS),
            func_re: Some(&RE_SWIFT_FUNC),
            var_re: None,
            import_re: Some(&RE_SWIFT_IMPORT),
            call_re: Some(&RE_CALL),
        }),
        ".scala" | ".sc" => Some(LanguagePattern {
            language_name: "scala",
            class_re: Some(&RE_SCALA_CLASS),
            func_re: Some(&RE_SCALA_FUNC),
            var_re: None,
            import_re: Some(&RE_SCALA_IMPORT),
            call_re: Some(&RE_CALL),
        }),
        ".lua" => Some(LanguagePattern {
            language_name: "lua",
            class_re: None,
            func_re: Some(&RE_LUA_FUNC),
            var_re: None,
            import_re: Some(&RE_LUA_IMPORT),
            call_re: Some(&RE_CALL),
        }),
        ".r" | ".R" => Some(LanguagePattern {
            language_name: "r",
            class_re: None,
            func_re: Some(&RE_R_FUNC),
            var_re: None,
            import_re: Some(&RE_R_IMPORT),
            call_re: Some(&RE_CALL),
        }),
        ".dart" => Some(LanguagePattern {
            language_name: "dart",
            class_re: Some(&RE_DART_CLASS),
            func_re: Some(&RE_DART_FUNC),
            var_re: None,
            import_re: Some(&RE_DART_IMPORT),
            call_re: Some(&RE_CALL),
        }),
        ".jl" => Some(LanguagePattern {
            language_name: "julia",
            class_re: Some(&RE_JULIA_CLASS),
            func_re: Some(&RE_JULIA_FUNC),
            var_re: None,
            import_re: Some(&RE_JULIA_IMPORT),
            call_re: Some(&RE_CALL),
        }),
        ".ex" | ".exs" => Some(LanguagePattern {
            language_name: "elixir",
            class_re: Some(&RE_ELIXIR_CLASS),
            func_re: Some(&RE_ELIXIR_FUNC),
            var_re: None,
            import_re: Some(&RE_ELIXIR_IMPORT),
            call_re: Some(&RE_CALL),
        }),
        ".erl" | ".hrl" => Some(LanguagePattern {
            language_name: "erlang",
            class_re: Some(&RE_ERLANG_CLASS),
            func_re: Some(&RE_ERLANG_FUNC),
            var_re: None,
            import_re: Some(&RE_ERLANG_IMPORT),
            call_re: Some(&RE_CALL),
        }),
        ".hs" => Some(LanguagePattern {
            language_name: "haskell",
            class_re: Some(&RE_HASKELL_CLASS),
            func_re: Some(&RE_HASKELL_FUNC),
            var_re: None,
            import_re: Some(&RE_HASKELL_IMPORT),
            call_re: None,
        }),
        ".ml" | ".mli" => Some(LanguagePattern {
            language_name: "ocaml",
            class_re: Some(&RE_OCAML_CLASS),
            func_re: Some(&RE_OCAML_FUNC),
            var_re: None,
            import_re: Some(&RE_OCAML_IMPORT),
            call_re: None,
        }),
        ".clj" | ".cljs" | ".cljc" => Some(LanguagePattern {
            language_name: "clojure",
            class_re: Some(&RE_CLOJURE_CLASS),
            func_re: Some(&RE_CLOJURE_FUNC),
            var_re: None,
            import_re: None,
            call_re: None,
        }),
        ".zig" => Some(LanguagePattern {
            language_name: "zig",
            class_re: Some(&RE_ZIG_CLASS),
            func_re: Some(&RE_ZIG_FUNC),
            var_re: None,
            import_re: Some(&RE_ZIG_IMPORT),
            call_re: Some(&RE_CALL),
        }),
        ".nim" => Some(LanguagePattern {
            language_name: "nim",
            class_re: Some(&RE_NIM_CLASS),
            func_re: Some(&RE_NIM_FUNC),
            var_re: None,
            import_re: Some(&RE_NIM_IMPORT),
            call_re: Some(&RE_CALL),
        }),
        ".pl" | ".pm" => Some(LanguagePattern {
            language_name: "perl",
            class_re: Some(&RE_PERL_CLASS),
            func_re: Some(&RE_PERL_FUNC),
            var_re: None,
            import_re: Some(&RE_PERL_IMPORT),
            call_re: Some(&RE_CALL),
        }),
        ".sql" => Some(LanguagePattern {
            language_name: "sql",
            class_re: Some(&RE_SQL_CLASS),
            func_re: Some(&RE_SQL_FUNC),
            var_re: None,
            import_re: None,
            call_re: None,
        }),
        ".sol" => Some(LanguagePattern {
            language_name: "solidity",
            class_re: Some(&RE_SOLIDITY_CLASS),
            func_re: Some(&RE_SOLIDITY_FUNC),
            var_re: None,
            import_re: Some(&RE_SOLIDITY_IMPORT),
            call_re: Some(&RE_CALL),
        }),
        ".gd" => Some(LanguagePattern {
            language_name: "gdscript",
            class_re: Some(&RE_GDSCRIPT_CLASS),
            func_re: Some(&RE_GDSCRIPT_FUNC),
            var_re: None,
            import_re: Some(&RE_GDSCRIPT_IMPORT),
            call_re: Some(&RE_CALL),
        }),
        ".ps1" | ".psm1" => Some(LanguagePattern {
            language_name: "powershell",
            class_re: Some(&RE_PWSH_CLASS),
            func_re: Some(&RE_PWSH_FUNC),
            var_re: None,
            import_re: None,
            call_re: Some(&RE_CALL),
        }),
        ".fs" | ".fsi" | ".fsx" => Some(LanguagePattern {
            language_name: "fsharp",
            class_re: Some(&RE_FSHARP_CLASS),
            func_re: Some(&RE_FSHARP_FUNC),
            var_re: None,
            import_re: Some(&RE_FSHARP_IMPORT),
            call_re: Some(&RE_CALL),
        }),
        ".groovy" | ".gvy" => Some(LanguagePattern {
            language_name: "groovy",
            class_re: Some(&RE_GROOVY_CLASS),
            func_re: Some(&RE_GROOVY_FUNC),
            var_re: None,
            import_re: Some(&RE_GROOVY_IMPORT),
            call_re: Some(&RE_CALL),
        }),
        ".f90" | ".f" | ".for" => Some(LanguagePattern {
            language_name: "fortran",
            class_re: Some(&RE_FORTRAN_CLASS),
            func_re: Some(&RE_FORTRAN_FUNC),
            var_re: None,
            import_re: None,
            call_re: None,
        }),
        ".pas" | ".dpr" => Some(LanguagePattern {
            language_name: "pascal",
            class_re: Some(&RE_PASCAL_CLASS),
            func_re: Some(&RE_PASCAL_FUNC),
            var_re: None,
            import_re: None,
            call_re: Some(&RE_CALL),
        }),
        ".adb" | ".ads" => Some(LanguagePattern {
            language_name: "ada",
            class_re: Some(&RE_ADA_CLASS),
            func_re: Some(&RE_ADA_FUNC),
            var_re: None,
            import_re: None,
            call_re: None,
        }),
        ".elm" => Some(LanguagePattern {
            language_name: "elm",
            class_re: Some(&RE_ELM_CLASS),
            func_re: Some(&RE_ELM_FUNC),
            var_re: None,
            import_re: Some(&RE_ELM_IMPORT),
            call_re: None,
        }),
        ".v" => Some(LanguagePattern {
            language_name: "v",
            class_re: Some(&RE_V_CLASS),
            func_re: Some(&RE_V_FUNC),
            var_re: None,
            import_re: Some(&RE_V_IMPORT),
            call_re: Some(&RE_CALL),
        }),
        ".wgsl" | ".glsl" | ".hlsl" => Some(LanguagePattern {
            language_name: "shader",
            class_re: Some(&RE_SHADER_CLASS),
            func_re: Some(&RE_SHADER_FUNC),
            var_re: None,
            import_re: None,
            call_re: Some(&RE_CALL),
        }),
        ".tf" | ".hcl" => Some(LanguagePattern {
            language_name: "terraform",
            class_re: Some(&RE_HCL_CLASS),
            func_re: None,
            var_re: None,
            import_re: None,
            call_re: None,
        }),
        ".proto" => Some(LanguagePattern {
            language_name: "protobuf",
            class_re: Some(&RE_PROTO_CLASS),
            func_re: None,
            var_re: None,
            import_re: Some(&RE_PROTO_IMPORT),
            call_re: None,
        }),
        ".graphql" | ".gql" => Some(LanguagePattern {
            language_name: "graphql",
            class_re: Some(&RE_GQL_CLASS),
            func_re: None,
            var_re: None,
            import_re: None,
            call_re: None,
        }),
        ".cls" => Some(LanguagePattern {
            language_name: "apex",
            class_re: Some(&RE_APEX_CLASS),
            func_re: Some(&RE_APEX_FUNC),
            var_re: None,
            import_re: None,
            call_re: Some(&RE_CALL),
        }),
        ".d" => Some(LanguagePattern {
            language_name: "d",
            class_re: Some(&RE_D_CLASS),
            func_re: Some(&RE_D_FUNC),
            var_re: None,
            import_re: Some(&RE_D_IMPORT),
            call_re: Some(&RE_CALL),
        }),
        ".cr" => Some(LanguagePattern {
            language_name: "crystal",
            class_re: Some(&RE_CRYSTAL_CLASS),
            func_re: Some(&RE_CRYSTAL_FUNC),
            var_re: None,
            import_re: Some(&RE_CRYSTAL_IMPORT),
            call_re: Some(&RE_CALL),
        }),
        _ => None,
    }
}

pub fn extract_symbols_universal(
    source_code: &str,
    file_path: &str,
    pattern: &LanguagePattern,
) -> Vec<CodeSymbol> {
    let mut symbols = Vec::new();
    let lines: Vec<&str> = source_code.lines().collect();

    let mut current_class: Option<String> = None;

    let file_calls: Vec<String> = if let Some(re_call) = pattern.call_re {
        let mut calls_set = HashSet::new();
        for cap in re_call.captures_iter(source_code) {
            if let Some(m) = cap.get(1) {
                let name = m.as_str().trim();
                if name.len() > 1 && !is_common_keyword(name) {
                    calls_set.insert(name.to_string());
                }
            }
        }
        calls_set.into_iter().collect()
    } else {
        Vec::new()
    };

    for (line_idx, line) in lines.iter().enumerate() {
        let line_num = line_idx + 1;

        if let Some(re_class) = pattern.class_re {
            if let Some(cap) = re_class.captures(line) {
                if let Some(m) = cap.get(1) {
                    let name = m.as_str().trim();
                    if !name.is_empty() {
                        current_class = Some(name.to_string());
                        symbols.push(CodeSymbol {
                            name: name.to_string(),
                            kind: SymbolKind::Class,
                            file_path: file_path.to_string(),
                            body: Some(line.to_string()),
                            calls: None,
                            line: line_num,
                            references: Vec::new(),
                            referenced_by: Vec::new(),
                            parent_class: None,
                        });
                        continue;
                    }
                }
            }
        }

        if let Some(re_func) = pattern.func_re {
            if let Some(cap) = re_func.captures(line) {
                if let Some(m) = cap.get(1) {
                    let name = m.as_str().trim();
                    if !name.is_empty() && !is_common_keyword(name) {
                        let is_indented = line.starts_with(' ') || line.starts_with('\t');
                        let (kind, parent_class) = if is_indented && current_class.is_some() {
                            (SymbolKind::Method, current_class.clone())
                        } else {
                            if !is_indented {
                                current_class = None;
                            }
                            (SymbolKind::Function, None)
                        };

                        let calls = if !file_calls.is_empty() {
                            Some(file_calls.clone())
                        } else {
                            None
                        };

                        symbols.push(CodeSymbol {
                            name: name.to_string(),
                            kind,
                            file_path: file_path.to_string(),
                            body: Some(line.to_string()),
                            calls,
                            line: line_num,
                            references: Vec::new(),
                            referenced_by: Vec::new(),
                            parent_class,
                        });
                        continue;
                    }
                }
            }
        }

        if let Some(re_import) = pattern.import_re {
            if let Some(cap) = re_import.captures(line) {
                if let Some(m) = cap.get(1) {
                    let import_path = m.as_str().trim();
                    if !import_path.is_empty() {
                        symbols.push(CodeSymbol {
                            name: import_path.to_string(),
                            kind: SymbolKind::Import,
                            file_path: file_path.to_string(),
                            body: None,
                            calls: None,
                            line: line_num,
                            references: Vec::new(),
                            referenced_by: Vec::new(),
                            parent_class: None,
                        });
                        continue;
                    }
                }
            }
        }
    }

    symbols
}

fn is_common_keyword(name: &str) -> bool {
    matches!(
        name,
        "if" | "else" | "for" | "while" | "switch" | "case" | "return" | "break"
            | "continue" | "try" | "catch" | "finally" | "throw" | "assert" | "yield"
            | "await" | "async" | "new" | "sizeof" | "typeof"
    )
}
