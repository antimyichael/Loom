use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoomWatchConfig {
    #[serde(default = "default_interval_minutes")]
    pub interval_minutes: u64,
    #[serde(default = "default_debounce_seconds")]
    pub debounce_seconds: u64,
}

fn default_interval_minutes() -> u64 {
    2
}

fn default_debounce_seconds() -> u64 {
    5
}

impl Default for LoomWatchConfig {
    fn default() -> Self {
        Self {
            interval_minutes: default_interval_minutes(),
            debounce_seconds: default_debounce_seconds(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoomVaultConfig {
    #[serde(default = "default_vault_name")]
    pub name: String,
}

fn default_vault_name() -> String {
    "LoomVault".to_string()
}

impl Default for LoomVaultConfig {
    fn default() -> Self {
        Self {
            name: default_vault_name(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LoomConfig {
    #[serde(default)]
    pub watch: LoomWatchConfig,
    #[serde(default)]
    pub vault: LoomVaultConfig,
}

pub fn load_config(project_root: &Path) -> LoomConfig {
    let config_path = project_root.join("loom.config.json");
    if !config_path.exists() {
        return LoomConfig::default();
    }

    match std::fs::read_to_string(&config_path) {
        Ok(content) => match serde_json::from_str::<LoomConfig>(&content) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("⚠ loom.config.json is invalid ({e}) — using defaults.");
                LoomConfig::default()
            }
        },
        Err(e) => {
            eprintln!("⚠ Could not read loom.config.json: {e} — using defaults.");
            LoomConfig::default()
        }
    }
}
