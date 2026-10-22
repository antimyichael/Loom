use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SymbolKind {
    Class,
    Method,
    Function,
    Variable,
    Import,
}

impl std::fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolKind::Class => write!(f, "class"),
            SymbolKind::Method => write!(f, "method"),
            SymbolKind::Function => write!(f, "function"),
            SymbolKind::Variable => write!(f, "variable"),
            SymbolKind::Import => write!(f, "import"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolRef {
    pub name: String,
    pub file_path: String,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub file_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calls: Option<Vec<String>>,
    pub line: usize,
    #[serde(default)]
    pub references: Vec<SymbolRef>,
    #[serde(default)]
    pub referenced_by: Vec<SymbolRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_class: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileIndex {
    pub file_path: String,
    pub language: String,
    pub symbols: Vec<CodeSymbol>,
    pub last_parsed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectIndex {
    pub project_root: String,
    pub last_parsed: u64,
    pub files: Vec<FileIndex>,
}
