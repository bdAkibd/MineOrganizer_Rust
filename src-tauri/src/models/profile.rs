use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Profile {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub instance_id: Option<String>,
    #[serde(default)]
    pub instance_path: Option<String>,
    #[serde(default)]
    pub minecraft_version: Option<String>,
    #[serde(default)]
    pub loader: Option<String>,
    #[serde(default)]
    pub loader_version: Option<String>,
    #[serde(default)]
    pub enabled_mod_hashes: Vec<String>,
    #[serde(default)]
    pub enabled_resourcepack_hashes: Vec<String>,
    #[serde(default)]
    pub enabled_shaderpack_hashes: Vec<String>,
    #[serde(default)]
    pub active_mods: Vec<String>,
    #[serde(default)]
    pub active_assets: Vec<String>,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}
