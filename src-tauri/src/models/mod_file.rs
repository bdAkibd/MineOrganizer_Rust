use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ModDependency {
    pub id: String,
    #[serde(default)]
    pub version_requirement: Option<String>,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ModMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub loaders: Vec<String>,
    #[serde(default)]
    pub mc_versions: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<ModDependency>,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default = "default_mod_kind")]
    pub kind: String,
}

fn default_mod_kind() -> String {
    "Mod".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModFile {
    pub file_name: String,
    pub path: PathBuf,
    pub size: u64,
    pub sha1: String,
    #[serde(default)]
    pub sha256: Option<String>,
    #[serde(default)]
    pub sha512: Option<String>,
    pub metadata: ModMetadata,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub disabled: bool,
}
