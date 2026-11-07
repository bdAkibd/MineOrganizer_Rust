use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub launcher: String,
    pub minecraft_version: String,
    pub loader: String,
    pub loader_version: String,
    pub status: String,
    #[serde(default)]
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstanceAssets {
    pub resourcepacks: Vec<AssetItem>,
    pub shaderpacks: Vec<AssetItem>,
    pub saves: Vec<SaveItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssetItem {
    pub name: String,
    pub filename: String,
    pub path: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SaveItem {
    pub folder_name: String,
    pub level_name: String,
    pub path: String,
    pub size: u64,
}
