use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use indexmap::IndexMap;

pub fn default_config_path() -> PathBuf {
    dirs::home_dir().unwrap().join(".config").join("seed").join("config.toml")
}

pub fn read(path: impl AsRef<Path>) -> Option<Config> {
    let content = std::fs::read_to_string(path).ok()?;
    toml::from_str(&content).ok()
}

pub fn save(path: impl AsRef<Path>, config: &Config) -> std::io::Result<()> {
    let content = toml::to_string(config).unwrap();
    std::fs::write(path, content)
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    pub table_alias: Vec<(String, String)>,
    pub tables: Vec<Table>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Table {
    #[serde(flatten)]
    pub columns: IndexMap<String, Column>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Column {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sanitize: Option<String>,
}